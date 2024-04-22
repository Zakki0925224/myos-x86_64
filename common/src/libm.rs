#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Utsname {
    pub sysname: [u8; 64],
    pub nodename: [u8; 64],
    pub release: [u8; 64],
    pub version: [u8; 64],
    pub machine: [u8; 64],
    pub domainname: [u8; 64],
}
