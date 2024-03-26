use crate::{
    arch::context::{Context, ContextMode},
    error::Result,
    mem::{
        self,
        bitmap::{self, MemoryFrameInfo},
        paging::PAGE_SIZE,
    },
    util::mutex::{Mutex, MutexError},
};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use core::{
    future::Future,
    pin::Pin,
    ptr::null,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    task::{Context as ExecutorContext, Poll, RawWaker, RawWakerVTable, Waker},
};
use log::info;

static mut TASK_EXECUTOR: Mutex<Executor> = Mutex::new(Executor::new());

static mut KERNEL_TASK: Mutex<Option<Task>> = Mutex::new(None);
static mut USER_TASK: Mutex<Option<Task>> = Mutex::new(None);

// preemptive multitask
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

#[derive(Debug)]
struct TaskId(u64);

impl TaskId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

struct ExecutorTask {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl ExecutorTask {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
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
    task_queue: Option<VecDeque<ExecutorTask>>,
}

impl Executor {
    pub const fn new() -> Self {
        // if use VecDeque::new(), occures unsafe precondition violated when push data
        // -> this is a bug for my own allocator
        //task_queue: VecDeque::with_capacity(16),
        Self { task_queue: None }
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue().pop_front() {
            let waker = dummy_waker();
            let mut context = ExecutorContext::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => info!("task: Done (id: {})", task.id.get()),
                Poll::Pending => self.task_queue().push_back(task),
            }
        }
    }

    pub fn spawn(&mut self, task: ExecutorTask) {
        self.task_queue().push_back(task);
    }

    fn task_queue(&mut self) -> &mut VecDeque<ExecutorTask> {
        if self.task_queue.is_none() {
            self.task_queue = Some(VecDeque::new());
        }
        self.task_queue.as_mut().unwrap()
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

pub fn spawn(future: impl Future<Output = ()> + 'static) -> Result<()> {
    if let Ok(mut executor) = unsafe { TASK_EXECUTOR.try_lock() } {
        let task = ExecutorTask::new(future);
        executor.spawn(task);
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn run() -> Result<()> {
    if let Ok(mut executor) = unsafe { TASK_EXECUTOR.try_lock() } {
        executor.run();
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

// non-preemptive multitask
#[derive(Debug)]
struct Task {
    id: TaskId,
    stack_mem_frame_info: MemoryFrameInfo,
    stack_size: usize,
    context: Context,
}

impl Drop for Task {
    fn drop(&mut self) {
        mem::bitmap::dealloc_mem_frame(self.stack_mem_frame_info).unwrap();
        info!("task: Dropped tid: {}", self.id.get());
    }
}

impl Task {
    pub fn new(
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

        let rsp = stack_mem_frame_info.frame_start_virt_addr.get() + stack_size as u64;
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
        })
    }

    pub fn switch_to(&self, next_task: &Task) {
        info!(
            "task: Switch context tid: {} to {}",
            self.id.get(),
            next_task.id.get()
        );
        self.context.switch_to(&next_task.context);
    }
}

pub fn exec_user_task(entry: extern "sysv64" fn(), file_name: &str, args: &[&str]) -> Result<()> {
    let mut arg_bytes: Vec<u8> = Vec::new();
    arg_bytes.extend_from_slice(file_name.as_bytes());
    arg_bytes.push(0);

    for a in args {
        arg_bytes.extend_from_slice(a.as_bytes());
        arg_bytes.push(0);
    }

    let args_mem_frame_info = bitmap::alloc_mem_frame((arg_bytes.len() / PAGE_SIZE).max(1))?;
    bitmap::mem_clear(&args_mem_frame_info)?;
    args_mem_frame_info.set_permissions_to_user()?;
    args_mem_frame_info
        .frame_start_virt_addr
        .copy_from_nonoverlapping(arg_bytes.as_ptr(), arg_bytes.len());

    let task = Task::new(
        1024 * 1024,
        Some(entry),
        args.len() as u64 + 1,
        args_mem_frame_info.frame_start_virt_addr.get(),
        ContextMode::User,
    )?;

    if let (Ok(mut kernel_task), Ok(mut user_task)) =
        unsafe { (KERNEL_TASK.try_lock(), USER_TASK.try_lock()) }
    {
        if kernel_task.is_none() {
            // stack is unused, because already allocated static area for kernel stack
            *kernel_task = Some(Task::new(0, None, 0, 0, ContextMode::Kernel)?)
        }

        *user_task = Some(task);
        kernel_task
            .as_ref()
            .unwrap()
            .switch_to(user_task.as_ref().unwrap());

        // returned
        args_mem_frame_info.set_permissions_to_supervisor()?;
        bitmap::dealloc_mem_frame(args_mem_frame_info)?;

        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn return_to_kernel_task() {
    let (kernel_task, user_task) =
        unsafe { (KERNEL_TASK.get_force_mut(), USER_TASK.get_force_mut()) };
    user_task
        .as_ref()
        .unwrap()
        .switch_to(kernel_task.as_ref().unwrap());

    unreachable!();
}
