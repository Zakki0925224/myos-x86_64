#[derive(Debug, PartialEq, Eq)]
pub enum ClusterType {
    Free(usize),
    Reserved(usize),
    Data(usize),
    Bad(usize),
    EndOfChain(usize),
}
