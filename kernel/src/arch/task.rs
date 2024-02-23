use crate::{
    error::Result,
    util::mutex::{Mutex, MutexError},
};
use alloc::{boxed::Box, collections::VecDeque};
use core::{
    future::Future,
    pin::Pin,
    ptr::null,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};
use log::info;

static mut TASK_EXECUTOR: Mutex<Executor> = Mutex::new(Executor::new());

#[derive(Default)]
struct Yield {
    polled: AtomicBool,
}

impl Future for Yield {
    type Output = ();

    fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<()> {
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

struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Self {
        Self {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

struct Executor {
    task_queue: Option<VecDeque<Task>>,
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
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => info!("task: Done (id: {})", task.id.get()),
                Poll::Pending => self.task_queue().push_back(task),
            }
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue().push_back(task);
    }

    fn task_queue(&mut self) -> &mut VecDeque<Task> {
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
        let task = Task::new(future);
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
