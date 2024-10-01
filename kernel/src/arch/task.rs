use crate::{
    arch::context::{Context, ContextMode},
    error::Result,
    mem::{
        self,
        bitmap::{self, MemoryFrameInfo},
        paging::PAGE_SIZE,
    },
    println,
    util::mutex::Mutex,
};
use alloc::{boxed::Box, collections::VecDeque, ffi::CString, vec::Vec};
use common::elf::Elf64;
use core::{
    future::Future,
    pin::Pin,
    ptr::null,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    task::{Context as ExecutorContext, Poll, RawWaker, RawWakerVTable, Waker},
};
use log::info;

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
                Poll::Ready(()) => info!("task: Done (id: {})", task.id.get()),
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
    stack_mem_frame_info: MemoryFrameInfo,
    stack_size: usize,
    context: Context,
    allocated_mem_frame_info: Vec<MemoryFrameInfo>,
}

impl Drop for Task {
    fn drop(&mut self) {
        self.stack_mem_frame_info
            .set_permissions_to_supervisor()
            .unwrap();
        mem::bitmap::dealloc_mem_frame(self.stack_mem_frame_info).unwrap();

        for mem_frame_info in &self.allocated_mem_frame_info {
            mem_frame_info.set_permissions_to_supervisor().unwrap();
            mem::bitmap::dealloc_mem_frame(*mem_frame_info).unwrap();
        }

        info!("task: Dropped tid: {}", self.id.get());
    }
}

impl Task {
    fn new(
        stack_size: usize,
        entry: Option<extern "sysv64" fn()>,
        arg0: u64,
        arg1: u64,
        mode: ContextMode,
    ) -> Result<Self> {
        let stack_mem_frame_info = mem::bitmap::alloc_mem_frame((stack_size / PAGE_SIZE).max(1))?;

        match mode {
            ContextMode::Kernel => stack_mem_frame_info.set_permissions_to_supervisor()?,
            ContextMode::User => stack_mem_frame_info.set_permissions_to_user()?,
        }

        let rsp = stack_mem_frame_info.frame_start_virt_addr()?.get() + stack_size as u64 - 1;
        let rip = match entry {
            Some(f) => f as u64,
            None => 0,
        };

        let mut context = Context::new();
        context.init(rip, arg0, arg1, rsp, mode);

        Ok(Self {
            id: TaskId::new(),
            stack_mem_frame_info,
            stack_size,
            context,
            allocated_mem_frame_info: Vec::new(),
        })
    }

    fn switch_to(&self, next_task: &Task) {
        info!(
            "task: Switch context tid: {} to {}",
            self.id.get(),
            next_task.id.get()
        );
        self.context.switch_to(&next_task.context);
    }
}

// pub fn exec_user_task(entry: extern "sysv64" fn(), file_name: &str, args: &[&str]) -> Result<u64> {
//     // write args to memory
//     let mut c_args = CString::new(file_name).unwrap().into_bytes_with_nul();
//     for arg in args {
//         c_args.extend(CString::new(*arg).unwrap().into_bytes_with_nul());
//     }

//     let mut c_args_offset = (args.len() + 2) * 8;
//     let args_mem_frame_info =
//         bitmap::alloc_mem_frame(((c_args.len() + c_args_offset) / PAGE_SIZE).max(1))?;
//     bitmap::mem_clear(&args_mem_frame_info)?;
//     args_mem_frame_info.set_permissions_to_user()?;
//     let args_mem_virt_addr = args_mem_frame_info.frame_start_virt_addr()?;

//     args_mem_virt_addr
//         .offset(c_args_offset)
//         .copy_from_nonoverlapping(c_args.as_ptr(), c_args.len());

//     let mut c_args_ref = Vec::new();
//     c_args_ref.push(args_mem_virt_addr.offset(c_args_offset).get());
//     c_args_offset += file_name.len() + 1;
//     for arg in args {
//         c_args_ref.push(args_mem_virt_addr.offset(c_args_offset).get());
//         c_args_offset += arg.len() + 1;
//     }

//     args_mem_virt_addr.copy_from_nonoverlapping(c_args_ref.as_ptr(), c_args_ref.len());

//     let task = Task::new(
//         1024 * 1024,
//         Some(entry),
//         args.len() as u64 + 1,
//         args_mem_virt_addr.get(),
//         ContextMode::User,
//     )?;
//     debug_task(&task);

//     let kernel_task = unsafe { KERNEL_TASK.get_force_mut() };
//     let user_tasks = unsafe { USER_TASKS.get_force_mut() };

//     if kernel_task.is_none() {
//         // stack is unused, because already allocated static area for kernel stack
//         *kernel_task = Some(Task::new(0, None, 0, 0, ContextMode::Kernel)?);
//     }

//     user_tasks.push(task);

//     let current_task = if user_tasks.len() == 1 {
//         kernel_task.as_ref().unwrap()
//     } else {
//         user_tasks.get(user_tasks.len() - 2).unwrap()
//     };

//     current_task.switch_to(user_tasks.last().unwrap());

//     // returned
//     let _ = user_tasks.pop();
//     args_mem_frame_info.set_permissions_to_supervisor()?;
//     bitmap::dealloc_mem_frame(args_mem_frame_info)?;

//     // get exit status
//     let exit_status = unsafe {
//         let status = match USER_EXIT_STATUS {
//             Some(s) => s,
//             None => panic!("task: User exit status was not found"),
//         };
//         USER_EXIT_STATUS = None;
//         status
//     };

//     return Ok(exit_status);
// }

pub fn exec_user_task(elf64: Elf64, file_name: &str, args: &[&str]) -> Result<u64> {}

pub fn push_allocated_mem_frame_info_for_user_task(mem_frame_info: MemoryFrameInfo) -> Result<()> {
    let user_task = unsafe { USER_TASKS.get_force_mut() }
        .iter_mut()
        .last()
        .unwrap();
    user_task.allocated_mem_frame_info.push(mem_frame_info);

    Ok(())
}

pub fn return_task(exit_status: u64) {
    unsafe {
        USER_EXIT_STATUS = Some(exit_status);
    }

    let current_task = unsafe { USER_TASKS.get_force_mut() }.pop().unwrap();
    let before_task = unsafe { USER_TASKS.get_force_mut() }
        .last()
        .unwrap_or(unsafe { KERNEL_TASK.get_force_mut() }.as_ref().unwrap());
    current_task.switch_to(before_task);

    unreachable!();
}

pub fn debug_user_task() {
    println!("===USER TASK INFO===");
    let user_task = unsafe { USER_TASKS.get_force_mut() }.last();
    if let Some(task) = user_task {
        debug_task(task);
    } else {
        println!("User task no available");
    }
}

fn debug_task(task: &Task) {
    let ctx = &task.context;
    println!("task id: {}", task.id.get());
    println!(
        "stack: (phys)0x{:x}, size: 0x{:x}bytes",
        task.stack_mem_frame_info.frame_start_phys_addr.get(),
        task.stack_size
    );
    println!("context:");
    println!(
        "\tcr3: 0x{:016x}, rip: 0x{:016x}, rflags: 0x{:016x},",
        ctx.cr3, ctx.rip, ctx.rflags
    );
    println!(
        "\tcs : 0x{:016x}, ss : 0x{:016x}, fs : 0x{:016x}, gs : 0x{:016x},",
        ctx.cs, ctx.ss, ctx.fs, ctx.gs
    );
    println!(
        "\trax: 0x{:016x}, rbx: 0x{:016x}, rcx: 0x{:016x}, rdx: 0x{:016x},",
        ctx.rax, ctx.rbx, ctx.rcx, ctx.rdx
    );
    println!(
        "\trdi: 0x{:016x}, rsi: 0x{:016x}, rsp: 0x{:016x}, rbp: 0x{:016x},",
        ctx.rdi, ctx.rsi, ctx.rsp, ctx.rbp
    );
    println!(
        "\tr8 : 0x{:016x}, r9 : 0x{:016x}, r10: 0x{:016x}, r11: 0x{:016x},",
        ctx.r8, ctx.r9, ctx.r10, ctx.r11
    );
    println!(
        "\tr12: 0x{:016x}, r13: 0x{:016x}, r14: 0x{:016x}, r15: 0x{:016x}",
        ctx.r12, ctx.r13, ctx.r14, ctx.r15
    );
}
