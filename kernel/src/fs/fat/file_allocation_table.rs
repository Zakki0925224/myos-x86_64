#[derive(Debug, PartialEq, Eq)]
pub enum ClusterType {
    Free,
    Reserved,
    Data(usize),
    Bad(usize),
    EndOfChain,
}
