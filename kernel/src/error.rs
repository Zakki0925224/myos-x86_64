use common::elf::Elf64Error;

use crate::{
    bus::usb::{
        device::UsbDeviceError,
        xhc::{ring_buffer::RingBufferError, XhcDriverError},
        UsbDriverError,
    },
    device::{console::ConsoleError, DeviceDriverError},
    fs::vfs::VirtualFileSystemError,
    graphics::{
        font::FontError, frame_buf::FrameBufferError, frame_buf_console::FrameBufferConsoleError,
        multi_layer::LayerError, simple_window_manager::SimpleWindowManagerError,
    },
    mem::{bitmap::BitmapMemoryManagerError, paging::PageManagerError},
    util::{ascii::AsciiCodeError, fifo::FifoError, mutex::MutexError},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Failed(&'static str),
    MutexError(MutexError),
    AsciiCodeError(AsciiCodeError),
    FontError(FontError),
    FrameBufferError(FrameBufferError),
    LayerError(LayerError),
    FrameBufferConsoleError(FrameBufferConsoleError),
    BitmapMemoryManagerError(BitmapMemoryManagerError),
    PageManagerError(PageManagerError),
    ConsoleError(ConsoleError),
    UsbDriverError(UsbDriverError),
    UsbDeviceError(UsbDeviceError),
    XhcDriverError(XhcDriverError),
    RingBufferError(RingBufferError),
    FifoError(FifoError),
    IndexOutOfBoundsError(usize),
    VirtualFileSystemError(VirtualFileSystemError),
    Elf64Error(Elf64Error),
    SimpleWindowManagerError(SimpleWindowManagerError),
    DeviceDriverError(DeviceDriverError),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Self::Failed(s)
    }
}

impl From<MutexError> for Error {
    fn from(err: MutexError) -> Self {
        Self::MutexError(err)
    }
}

impl From<AsciiCodeError> for Error {
    fn from(err: AsciiCodeError) -> Self {
        Self::AsciiCodeError(err)
    }
}

impl From<FontError> for Error {
    fn from(err: FontError) -> Self {
        Self::FontError(err)
    }
}

impl From<FrameBufferError> for Error {
    fn from(err: FrameBufferError) -> Self {
        Self::FrameBufferError(err)
    }
}

impl From<LayerError> for Error {
    fn from(err: LayerError) -> Self {
        Self::LayerError(err)
    }
}

impl From<FrameBufferConsoleError> for Error {
    fn from(err: FrameBufferConsoleError) -> Self {
        Self::FrameBufferConsoleError(err)
    }
}

impl From<BitmapMemoryManagerError> for Error {
    fn from(err: BitmapMemoryManagerError) -> Self {
        Self::BitmapMemoryManagerError(err)
    }
}

impl From<PageManagerError> for Error {
    fn from(err: PageManagerError) -> Self {
        Self::PageManagerError(err)
    }
}

impl From<ConsoleError> for Error {
    fn from(err: ConsoleError) -> Self {
        Self::ConsoleError(err)
    }
}

impl From<UsbDriverError> for Error {
    fn from(err: UsbDriverError) -> Self {
        Self::UsbDriverError(err)
    }
}

impl From<UsbDeviceError> for Error {
    fn from(err: UsbDeviceError) -> Self {
        Self::UsbDeviceError(err)
    }
}

impl From<XhcDriverError> for Error {
    fn from(err: XhcDriverError) -> Self {
        Self::XhcDriverError(err)
    }
}

impl From<RingBufferError> for Error {
    fn from(err: RingBufferError) -> Self {
        Self::RingBufferError(err)
    }
}

impl From<FifoError> for Error {
    fn from(err: FifoError) -> Self {
        Self::FifoError(err)
    }
}

impl From<VirtualFileSystemError> for Error {
    fn from(err: VirtualFileSystemError) -> Self {
        Self::VirtualFileSystemError(err)
    }
}

impl From<Elf64Error> for Error {
    fn from(err: Elf64Error) -> Self {
        Self::Elf64Error(err)
    }
}

impl From<SimpleWindowManagerError> for Error {
    fn from(err: SimpleWindowManagerError) -> Self {
        Self::SimpleWindowManagerError(err)
    }
}

impl From<DeviceDriverError> for Error {
    fn from(err: DeviceDriverError) -> Self {
        Self::DeviceDriverError(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
