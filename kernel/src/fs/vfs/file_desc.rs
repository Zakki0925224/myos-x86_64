pub type FileDescriptorNumber = u16;

pub const FDN_STDIN: FileDescriptorNumber = 0;
pub const FDN_STDOUT: FileDescriptorNumber = 1;
pub const FDN_STDERR: FileDescriptorNumber = 2;

pub trait FileDescriptor {
    fn read(buf: *mut u8, len: usize) -> FileDescriptorNumber;
    //fn write();
}
