#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterType {
    Free,
    Reserved,
    Data(usize),
    Bad(usize),
    EndOfChain,
}
