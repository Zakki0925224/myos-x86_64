#[derive(Debug)]
#[repr(C)]
pub struct FsInfoSector {
    sign0: u32,
    reserved0: [u8; 480],
    sign1: u32,
    free_cnt: [u8; 4],  // free clusters count
    next_free: [u8; 4], // next free cluster number
    reserved1: [u8; 12],
    sign2: u32,
}
