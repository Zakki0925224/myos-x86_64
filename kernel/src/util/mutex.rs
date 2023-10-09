#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutexError {
    Locked,
}
