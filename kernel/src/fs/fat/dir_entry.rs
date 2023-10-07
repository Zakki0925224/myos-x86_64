use alloc::string::String;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DirectoryEntry {
    name: [u8; 11],
    attr: u8,
    nt_reserved: u8,
    create_time_ms: u8,
    create_time: [u8; 2],
    create_date: [u8; 2],
    last_access_date: [u8; 2],
    first_cluster_high: [u8; 2],
    wrote_time: [u8; 2],
    wrote_date: [u8; 2],
    first_cluster_low: [u8; 2],
    file_size: [u8; 4],
}

impl DirectoryEntry {
    pub fn name(&self) -> String {
        String::from_utf8_lossy(&self.name).into_owned()
    }
}
