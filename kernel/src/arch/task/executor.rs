use core::{
    ptr::null,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use alloc::collections::VecDeque;
use log::info;

use super::Task;

pub struct Executor {
    task_queue: Option<VecDeque<Task>>,
}

impl Executor {
    pub fn new() -> Self {
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
                Poll::Ready(()) => info!("task: Done a task: (id: {})", task.id.0),
                Poll::Pending => {
                    info!("task: Pending a task: (id: {})", task.id.0);
                    self.task_queue().push_back(task)
                }
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
