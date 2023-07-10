use core::{
    ptr::null,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use alloc::collections::VecDeque;

use super::Task;

pub struct Executor {
    task_queue: VecDeque<Task>,
}

impl Executor {
    pub fn new() -> Executor {
        return Executor {
            // if use VecDeque::new(), occures unsafe precondition violated when push data
            task_queue: VecDeque::with_capacity(16),
        };
    }

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task),
            }
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        return dummy_raw_waker();
    }
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    return RawWaker::new(null() as *const (), vtable);
}

fn dummy_waker() -> Waker {
    return unsafe { Waker::from_raw(dummy_raw_waker()) };
}
