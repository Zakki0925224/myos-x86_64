use super::context::{Context, Stack, STACK_SIZE};
use crate::util::id::UniqueIdU64;
use alloc::boxed::Box;

type ProcessId = UniqueIdU64;

enum ProcessState {
    Created,
    Waiting,
    Running,
    Blocked,
    Terminated(i32), // exit code
}

struct Process {
    id: ProcessId,
    state: ProcessState,
    stack: Stack<STACK_SIZE>,
    context: Box<Context>,
}

impl Process {
    pub fn new() -> Self {
        Self {
            id: ProcessId::new(),
            state: ProcessState::Created,
            stack: Stack::new(),
            context: Box::new(Context::new()),
        }
    }
}
