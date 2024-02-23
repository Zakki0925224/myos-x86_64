use super::context::{Context, ContextMode};
use crate::{
    error::Result,
    mem::{self, bitmap::MemoryFrameInfo, paging::PAGE_SIZE},
    util::mutex::{Mutex, MutexError},
};
use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::sync::atomic::{AtomicU64, Ordering};

static mut PROCESS_TABLE: Mutex<Option<ProcessTable>> = Mutex::new(None);

#[derive(Debug)]
struct ProcessId(u64);

impl ProcessId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }

    pub fn get(&self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ProcessState {
    Created,
    Waiting,
    Running,
    Blocked,
    Terminated(i32), // exit code
}

#[derive(Debug)]
struct Process {
    id: ProcessId,
    name: String,
    state: ProcessState,
    stack_mem_frame_info: MemoryFrameInfo,
    stack_size: usize,
    context: Context,
}

impl Drop for Process {
    fn drop(&mut self) {
        mem::bitmap::dealloc_mem_frame(self.stack_mem_frame_info).unwrap();
    }
}

impl Process {
    pub fn new(
        name: String,
        stack_size: usize,
        entry: Option<extern "sysv64" fn()>,
        mode: ContextMode,
    ) -> Result<Self> {
        let stack_mem_frame_info = mem::bitmap::alloc_mem_frame((stack_size / PAGE_SIZE).max(1))?;
        let rsp = stack_mem_frame_info.get_frame_start_virt_addr().get() + stack_size as u64;
        let rip = match entry {
            Some(f) => f as u64,
            None => 0,
        };

        let mut context = Context::new();
        context.init(rip, 0, 0, rsp, mode);

        Ok(Self {
            id: ProcessId::new(),
            name,
            state: ProcessState::Created,
            stack_mem_frame_info,
            stack_size,
            context,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessTableError {
    NotInitialized,
    InvalidProcessIdError(u64),
    InvalidProcessStateError { pid: u64, state: ProcessState },
}

struct ProcessTable(Vec<Process>);

impl ProcessTable {
    pub fn new() -> Result<Self> {
        let kernel_p = Process::new("kernel".to_string(), 0, None, ContextMode::Kernel)?;
        Ok(Self(vec![kernel_p]))
    }

    pub fn create_process(
        &mut self,
        p_name: String,
        p_stack_size: usize,
        entry: extern "sysv64" fn(),
        mode: ContextMode,
    ) -> Result<()> {
        let p = Process::new(p_name, p_stack_size, Some(entry), mode)?;
        self.0.push(p);

        Ok(())
    }

    pub fn switch_process(&self, current_pid: u64, next_pid: u64) -> Result<()> {
        let current_p = match self.0.iter().find(|p| p.id.get() == current_pid) {
            Some(p) => p,
            None => return Err(ProcessTableError::InvalidProcessIdError(current_pid).into()),
        };

        let next_p = match self.0.iter().find(|p| p.id.get() == next_pid) {
            Some(p) => p,
            None => return Err(ProcessTableError::InvalidProcessIdError(next_pid).into()),
        };

        current_p.context.switch_to(&next_p.context);

        Ok(())
    }
}

pub fn init_table() -> Result<()> {
    if let Ok(mut p_table) = unsafe { PROCESS_TABLE.try_lock() } {
        *p_table = match ProcessTable::new() {
            Ok(t) => Some(t),
            Err(err) => return Err(err),
        };
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn create_process(
    p_name: String,
    p_stack_size: usize,
    entry: extern "sysv64" fn(),
    mode: ContextMode,
) -> Result<()> {
    if let Ok(mut p_table) = unsafe { PROCESS_TABLE.try_lock() } {
        if let Some(p_table) = p_table.as_mut() {
            return p_table.create_process(p_name, p_stack_size, entry, mode);
        }

        return Err(ProcessTableError::NotInitialized.into());
    }

    Err(MutexError::Locked.into())
}

pub fn switch_process(current_pid: u64, next_pid: u64) -> Result<()> {
    if let Some(p_table) = unsafe { PROCESS_TABLE.get_force_mut() } {
        return p_table.switch_process(current_pid, next_pid);
    }

    Err(ProcessTableError::NotInitialized.into())
}
