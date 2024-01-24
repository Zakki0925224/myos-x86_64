use crate::{
    bus::usb::{
        device::UsbDeviceError,
        xhc::{ring_buffer::RingBufferError, XhcDriverError},
        UsbDriverError,
    },
    device::console::ConsoleError,
    graphics::{frame_buf::FrameBufferError, frame_buf_console::FrameBufferConsoleError},
    mem::{bitmap::BitmapMemoryManagerError, paging::PageManagerError},
    util::{ascii::AsciiCodeError, fifo::FifoError, mutex::MutexError},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Failed(&'static str),
    MutexError(MutexError),
    AsciiCodeError(AsciiCodeError),
    FrameBufferError(FrameBufferError),
    FrameBufferConsoleError(FrameBufferConsoleError),
    BitmapMemoryManagerError(BitmapMemoryManagerError),
    PageManagerError(PageManagerError),
    ConsoleError(ConsoleError),
    UsbDriverError(UsbDriverError),
    UsbDeviceError(UsbDeviceError),
    XhcDriverError(XhcDriverError),
    RingBufferError(RingBufferError),
    FifoError(FifoError),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Failed(s)
    }
}

impl From<MutexError> for Error {
    fn from(err: MutexError) -> Self {
        Error::MutexError(err)
    }
}

impl From<AsciiCodeError> for Error {
    fn from(err: AsciiCodeError) -> Self {
        Error::AsciiCodeError(err)
    }
}

impl From<FrameBufferError> for Error {
    fn from(err: FrameBufferError) -> Self {
        Error::FrameBufferError(err)
    }
}

impl From<FrameBufferConsoleError> for Error {
    fn from(err: FrameBufferConsoleError) -> Self {
        Error::FrameBufferConsoleError(err)
    }
}

impl From<BitmapMemoryManagerError> for Error {
    fn from(err: BitmapMemoryManagerError) -> Self {
        Error::BitmapMemoryManagerError(err)
    }
}

impl From<PageManagerError> for Error {
    fn from(err: PageManagerError) -> Self {
        Error::PageManagerError(err)
    }
}

impl From<ConsoleError> for Error {
    fn from(err: ConsoleError) -> Self {
        Error::ConsoleError(err)
    }
}

impl From<UsbDriverError> for Error {
    fn from(err: UsbDriverError) -> Self {
        Error::UsbDriverError(err)
    }
}

impl From<UsbDeviceError> for Error {
    fn from(err: UsbDeviceError) -> Self {
        Error::UsbDeviceError(err)
    }
}

impl From<XhcDriverError> for Error {
    fn from(err: XhcDriverError) -> Self {
        Error::XhcDriverError(err)
    }
}

impl From<RingBufferError> for Error {
    fn from(err: RingBufferError) -> Self {
        Error::RingBufferError(err)
    }
}

impl From<FifoError> for Error {
    fn from(err: FifoError) -> Self {
        Error::FifoError(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
