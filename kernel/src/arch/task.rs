use crate::{
    arch::{addr::VirtualAddress, context::*},
    error::*,
    graphics::{multi_layer::LayerId, simple_window_manager},
    mem::{
        bitmap::{self, MemoryFrameInfo},
        paging::{self, *},
    },
    util::mutex::Mutex,
};
use alloc::{boxed::Box, collections::VecDeque, ffi::CString, vec::Vec};
use common::elf::{self, *};
use core::{
    future::Future,
    pin::Pin,
    ptr::null,
    sync::atomic::*,
    task::{Context as ExecutorContext, Poll, RawWaker, RawWakerVTable, Waker},
};
use log::{debug, trace};

static mut TASK_EXECUTOR: Mutex<Executor> = Mutex::new(Executor::new());

static mut KERNEL_TASK: Mutex<Option<Task>> = Mutex::new(None);
static mut USER_TASKS: Mutex<Vec<Task>> = Mutex::new(Vec::new());
static mut USER_EXIT_STATUS: Option<u64> = None;

#[derive(Default)]
struct Yield {
    polled: AtomicBool,
}

impl Future for Yield {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut ExecutorContext) -> Poll<()> {
        if self.polled.fetch_or(true, Ordering::SeqCst) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

#[derive(Debug, Clone)]
struct TaskId(usize);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    fn get(&self) -> usize {
        self.0
    }
}

struct ExecutorTask {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl ExecutorTask {
    fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut ExecutorContext) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

struct Executor {
    task_queue: VecDeque<ExecutorTask>,
    is_ready: bool,
}

impl Executor {
    const fn new() -> Self {
        Self {
            task_queue: VecDeque::new(),
            is_ready: false,
        }
    }

    fn poll(&mut self) {
        if !self.is_ready {
            return;
        }

        if let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = ExecutorContext::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => trace!("task: Done (id: {})", task.id.get()),
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }

    fn ready(&mut self) {
        self.is_ready = true;
    }

    fn spawn(&mut self, task: ExecutorTask) {
        self.task_queue.push_back(task);
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(null() as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

pub async fn exec_yield() {
    Yield::default().await
}

pub fn poll() -> Result<()> {
    unsafe { TASK_EXECUTOR.try_lock() }?.poll();
    Ok(())
}

pub fn ready() -> Result<()> {
    unsafe { TASK_EXECUTOR.try_lock() }?.ready();
    Ok(())
}

pub fn spawn(future: impl Future<Output = ()> + 'static) -> Result<()> {
    let task = ExecutorTask::new(future);
    unsafe { TASK_EXECUTOR.try_lock() }?.spawn(task);
    Ok(())
}

#[derive(Debug, Clone)]
struct Task {
    id: TaskId,
    context: Context,
    args_mem_frame_info: Option<MemoryFrameInfo>,
    stack_mem_frame_info: MemoryFrameInfo,
    program_mem_info: Vec<(MemoryFrameInfo, MappingInfo)>,
    allocated_mem_frame_info: Vec<MemoryFrameInfo>,
    created_wd: Vec<LayerId>,
}

impl Drop for Task {
    fn drop(&mut self) {
        if let Some(args_mem_frame_info) = self.args_mem_frame_info {
            args_mem_frame_info.set_permissions_to_supervisor().unwrap();
            bitmap::dealloc_mem_frame(args_mem_frame_info).unwrap();
        }

        self.stack_mem_frame_info
            .set_permissions_to_supervisor()
            .unwrap();
        bitmap::dealloc_mem_frame(self.stack_mem_frame_info).unwrap();

        for (mem_info, mapping_info) in self.program_mem_info.iter() {
            let start = mapping_info.start;
            paging::update_mapping(&MappingInfo {
                start,
                end: mapping_info.end,
                phys_addr: start.get().into(),
                rw: ReadWrite::Write,
                us: EntryMode::Supervisor,
                pwt: PageWriteThroughLevel::WriteThrough,
            })
            .unwrap();

            assert_eq!(
                paging::calc_virt_addr(start.get().into()).unwrap().get(),
                start.get()
            );
            bitmap::dealloc_mem_frame(*mem_info).unwrap();
        }

        for mem_frame_info in self.allocated_mem_frame_info.iter() {
            mem_frame_info.set_permissions_to_supervisor().unwrap();
            bitmap::dealloc_mem_frame(*mem_frame_info).unwrap();
        }

        // destroy all created window
        for wd in self.created_wd.iter() {
            simple_window_manager::destroy_window(wd).unwrap();
        }

        trace!("task: Dropped tid: {}", self.id.get());
    }
}

impl Task {
    fn new(
        stack_size: usize, // 4KiB align
        elf64: Option<Elf64>,
        args: Option<&[&str]>, // file name + args
        mode: ContextMode,
    ) -> Result<Self> {
        // parse ELF
        let mut entry = None;
        let mut program_mem_info = Vec::new();
        if let Some(elf64) = elf64 {
            let header = elf64.header();

            if header.elf_type() != elf::Type::Executable {
                return Err(Error::Failed("The file is not an executable file"));
            }

            if header.machine() != elf::Machine::X8664 {
                return Err(Error::Failed("Unsupported ISA"));
            }

            for program_header in elf64.program_headers() {
                if program_header.segment_type() != SegmentType::Load {
                    continue;
                }

                let p_virt_addr = program_header.virt_addr;
                let p_mem_size = program_header.mem_size;
                let p_file_size = program_header.file_size;

                let pages_needed =
                    ((p_virt_addr % PAGE_SIZE as u64 + p_mem_size + PAGE_SIZE as u64 - 1)
                        / PAGE_SIZE as u64) as usize;
                let user_mem_frame_info = bitmap::alloc_mem_frame(pages_needed)?;
                bitmap::mem_clear(&user_mem_frame_info)?;
                let user_mem_frame_start_virt_addr = user_mem_frame_info.frame_start_virt_addr()?;

                // copy data
                let program_data = elf64.data_by_program_header(program_header);
                if let Some(data) = program_data {
                    user_mem_frame_start_virt_addr
                        .offset(p_virt_addr as usize % PAGE_SIZE)
                        .copy_from_nonoverlapping(data.as_ptr(), p_file_size as usize);
                }

                // update page mapping
                let start_virt_addr = (p_virt_addr / PAGE_SIZE as u64 * PAGE_SIZE as u64).into();
                let mapping_info = MappingInfo {
                    start: start_virt_addr,
                    end: start_virt_addr.offset(user_mem_frame_info.frame_size),
                    phys_addr: user_mem_frame_info.frame_start_phys_addr,
                    rw: ReadWrite::Write,
                    us: EntryMode::User,
                    pwt: PageWriteThroughLevel::WriteThrough,
                };
                paging::update_mapping(&mapping_info)?;
                program_mem_info.push((user_mem_frame_info, mapping_info));

                if header.entry_point >= p_virt_addr
                    && header.entry_point < p_virt_addr + p_mem_size
                {
                    entry = Some(header.entry_point);
                }
            }
        }

        let rip = match entry {
            Some(f) => f as u64,
            None => 0,
        };

        // stack
        let stack_mem_frame_info =
            bitmap::alloc_mem_frame(((stack_size + PAGE_SIZE - 1) / PAGE_SIZE).max(1))?;
        match mode {
            ContextMode::Kernel => stack_mem_frame_info.set_permissions_to_supervisor()?,
            ContextMode::User => stack_mem_frame_info.set_permissions_to_user()?,
        }
        let rsp =
            (stack_mem_frame_info.frame_start_virt_addr()?.get() + stack_size as u64 - 63) & !63;
        assert!(rsp % 64 == 0); // must be 64 bytes align for SSE and AVX instructions, etc.

        // args
        let mut args_mem_frame_info = None;
        let mut arg0 = 0; // args len
        let mut arg1 = 0; // args virt addr
        if let Some(args) = args {
            let mut c_args = Vec::new();
            for arg in args {
                c_args.extend(CString::new(*arg).unwrap().into_bytes_with_nul());
            }

            let mut c_args_offset = (args.len() + 2) * 8;
            let mem_frame_info =
                bitmap::alloc_mem_frame(((c_args.len() + c_args_offset) / PAGE_SIZE).max(1))?;
            bitmap::mem_clear(&mem_frame_info)?;
            match mode {
                ContextMode::Kernel => mem_frame_info.set_permissions_to_supervisor()?,
                ContextMode::User => mem_frame_info.set_permissions_to_user()?,
            }

            let args_mem_virt_addr = mem_frame_info.frame_start_virt_addr()?;
            args_mem_virt_addr
                .offset(c_args_offset)
                .copy_from_nonoverlapping(c_args.as_ptr(), c_args.len());

            let mut c_args_ref = Vec::new();
            for arg in args {
                c_args_ref.push(args_mem_virt_addr.offset(c_args_offset).get());
                c_args_offset += arg.len() + 1;
            }
            args_mem_virt_addr.copy_from_nonoverlapping(c_args_ref.as_ptr(), c_args_ref.len());

            args_mem_frame_info = Some(mem_frame_info);
            arg0 = args.len() as u64;
            arg1 = args_mem_virt_addr.get();
        }

        // context
        let mut context = Context::new();
        context.init(rip, arg0, arg1, rsp, mode);

        Ok(Self {
            id: TaskId::new(),
            context,
            args_mem_frame_info,
            stack_mem_frame_info,
            program_mem_info,
            allocated_mem_frame_info: Vec::new(),
            created_wd: Vec::new(),
        })
    }

    fn unmap_virt_addr(&self) -> Result<()> {
        for (_, mapping_info) in self.program_mem_info.iter() {
            let start = mapping_info.start;
            paging::update_mapping(&MappingInfo {
                start,
                end: mapping_info.end,
                phys_addr: start.get().into(),
                rw: ReadWrite::Write,
                us: EntryMode::Supervisor,
                pwt: PageWriteThroughLevel::WriteThrough,
            })?;

            assert_eq!(
                paging::calc_virt_addr(start.get().into()).unwrap().get(),
                start.get()
            );
        }

        Ok(())
    }

    fn remap_virt_addr(&self) -> Result<()> {
        for (_, mapping_info) in self.program_mem_info.iter() {
            paging::update_mapping(mapping_info)?;
        }

        Ok(())
    }

    fn switch_to(&self, next_task: &Task) {
        trace!(
            "task: Switch context tid: {} to {}",
            self.id.get(),
            next_task.id.get()
        );

        self.context.switch_to(&next_task.context);
    }
}

pub fn exec_user_task(elf64: Elf64, file_name: &str, args: &[&str]) -> Result<u64> {
    let kernel_task = unsafe { KERNEL_TASK.get_force_mut() };
    let user_tasks = unsafe { USER_TASKS.get_force_mut() };

    if kernel_task.is_none() {
        // stack is unused, because already allocated static area for kernel stack
        *kernel_task = Some(Task::new(0, None, None, ContextMode::Kernel)?);
    }

    let is_user = !user_tasks.is_empty();
    if is_user {
        user_tasks.last().unwrap().unmap_virt_addr()?;
    }

    let user_task = Task::new(
        1024 * 1024,
        Some(elf64),
        Some(&[&[file_name], args].concat()),
        ContextMode::User,
    );

    let task = match user_task {
        Ok(task) => task,
        Err(e) => {
            if is_user {
                user_tasks.last().unwrap().remap_virt_addr()?;
            }
            return Err(e);
        }
    };

    user_tasks.push(task);

    let is_user = user_tasks.len() > 1;
    let current_task = if is_user {
        user_tasks.get(user_tasks.len() - 2).unwrap()
    } else {
        kernel_task.as_ref().unwrap()
    };

    current_task.switch_to(user_tasks.last().unwrap());

    // returned
    drop(user_tasks.pop().unwrap());
    if let Some(task) = user_tasks.last() {
        task.remap_virt_addr()?;
    }

    // get exit status
    let exit_status = unsafe {
        let status = match USER_EXIT_STATUS {
            Some(s) => s,
            None => panic!("task: User exit status was not found"),
        };
        USER_EXIT_STATUS = None;
        status
    };

    Ok(exit_status)
}

pub fn push_allocated_mem_frame_info_for_user_task(mem_frame_info: MemoryFrameInfo) -> Result<()> {
    let user_task = unsafe { USER_TASKS.get_force_mut() }
        .iter_mut()
        .last()
        .unwrap();
    user_task.allocated_mem_frame_info.push(mem_frame_info);

    Ok(())
}

pub fn get_memory_frame_size_by_virt_addr(virt_addr: VirtualAddress) -> Result<Option<usize>> {
    let user_task = unsafe { USER_TASKS.get_force_mut() }
        .iter_mut()
        .last()
        .unwrap();

    for mem_frame_info in &user_task.allocated_mem_frame_info {
        if mem_frame_info.frame_start_virt_addr()? == virt_addr {
            return Ok(Some(mem_frame_info.frame_size));
        }
    }

    Ok(None)
}

pub fn push_wd(wd: LayerId) {
    let user_task = unsafe { USER_TASKS.get_force_mut() }
        .iter_mut()
        .last()
        .unwrap();

    user_task.created_wd.push(wd);
}

pub fn remove_wd(wd: &LayerId) {
    let user_task = unsafe { USER_TASKS.get_force_mut() }
        .iter_mut()
        .last()
        .unwrap();

    user_task.created_wd.retain(|cwd| cwd.get() != wd.get());
}

pub fn return_task(exit_status: u64) {
    unsafe {
        USER_EXIT_STATUS = Some(exit_status);
    }

    let user_tasks = unsafe { USER_TASKS.get_force_mut() };
    let current_task = user_tasks.last().unwrap();

    let before_task;
    if let Some(before_task_i) = user_tasks.len().checked_sub(2) {
        before_task = user_tasks.get(before_task_i).unwrap();
    } else {
        before_task = unsafe { KERNEL_TASK.get_force_mut() }.as_ref().unwrap();
    }

    current_task.switch_to(before_task);

    unreachable!();
}

pub fn debug_user_task() {
    debug!("===USER TASK INFO===");
    let user_task = unsafe { USER_TASKS.get_force_mut() }.last();
    if let Some(task) = user_task {
        debug_task(task);
    } else {
        debug!("User task no available");
    }
}

pub fn is_running_user_task() -> bool {
    unsafe { USER_TASKS.get_force_mut() }.len() > 1
}

fn debug_task(task: &Task) {
    let ctx = &task.context;
    debug!("task id: {}", task.id.get());
    debug!(
        "stack: (phys)0x{:x}, size: 0x{:x}bytes",
        task.stack_mem_frame_info.frame_start_phys_addr.get(),
        task.stack_mem_frame_info.frame_size,
    );
    debug!("context:");
    debug!(
        "\tcr3: 0x{:016x}, rip: 0x{:016x}, rflags: 0x{:016x},",
        ctx.cr3, ctx.rip, ctx.rflags
    );
    debug!(
        "\tcs : 0x{:016x}, ss : 0x{:016x}, fs : 0x{:016x}, gs : 0x{:016x},",
        ctx.cs, ctx.ss, ctx.fs, ctx.gs
    );
    debug!(
        "\trax: 0x{:016x}, rbx: 0x{:016x}, rcx: 0x{:016x}, rdx: 0x{:016x},",
        ctx.rax, ctx.rbx, ctx.rcx, ctx.rdx
    );
    debug!(
        "\trdi: 0x{:016x}, rsi: 0x{:016x}, rsp: 0x{:016x}, rbp: 0x{:016x},",
        ctx.rdi, ctx.rsi, ctx.rsp, ctx.rbp
    );
    debug!(
        "\tr8 : 0x{:016x}, r9 : 0x{:016x}, r10: 0x{:016x}, r11: 0x{:016x},",
        ctx.r8, ctx.r9, ctx.r10, ctx.r11
    );
    debug!(
        "\tr12: 0x{:016x}, r13: 0x{:016x}, r14: 0x{:016x}, r15: 0x{:016x}",
        ctx.r12, ctx.r13, ctx.r14, ctx.r15
    );
    debug!("allocated mem frame info:");
    for mem_frame_info in &task.allocated_mem_frame_info {
        let virt_addr = mem_frame_info.frame_start_virt_addr().unwrap();

        debug!(
            "\t(virt)0x{:x}-0x{:x}",
            virt_addr.get(),
            virt_addr.offset(mem_frame_info.frame_size).get(),
        );
    }
}
