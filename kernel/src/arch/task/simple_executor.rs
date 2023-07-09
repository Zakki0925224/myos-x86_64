use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

use alloc::collections::VecDeque;

use super::Task;

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        return SimpleExecutor {
            task_queue: VecDeque::new(),
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
    return RawWaker::new(0 as *const (), vtable);
}

fn dummy_waker() -> Waker {
    return unsafe { Waker::from_raw(dummy_raw_waker()) };
}
