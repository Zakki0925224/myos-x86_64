use crate::{
    acpi::AcpiError,
    device::{
        console::ConsoleError,
        usb::{
            bus::{device::UsbDeviceError, UsbBusDriverError},
            xhc::{ringbuf::RingBufferError, XhcDriverError},
        },
    },
    fs::vfs::VirtualFileSystemError,
    graphics::{
        font::FontError, frame_buf::FrameBufferError, multi_layer::LayerError,
        simple_window_manager::SimpleWindowManagerError,
    },
    mem::{bitmap::BitmapMemoryManagerError, paging::PageManagerError},
    util::{fifo::FifoError, lifo::LifoError},
};
use common::elf::Elf64Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    Failed(&'static str),
    FontError(FontError),
    FrameBufferError(FrameBufferError),
    LayerError(LayerError),
    BitmapMemoryManagerError(BitmapMemoryManagerError),
    PageManagerError(PageManagerError),
    ConsoleError(ConsoleError),
    UsbBusDriverError(UsbBusDriverError),
    UsbDeviceError(UsbDeviceError),
    XhcDriverError(XhcDriverError),
    RingBufferError(RingBufferError),
    FifoError(FifoError),
    LifoError(LifoError),
    IndexOutOfBoundsError(usize),
    VirtualFileSystemError(VirtualFileSystemError),
    Elf64Error(Elf64Error),
    SimpleWindowManagerError(SimpleWindowManagerError),
    AcpiError(AcpiError),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Self::Failed(s)
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

impl From<UsbBusDriverError> for Error {
    fn from(err: UsbBusDriverError) -> Self {
        Self::UsbBusDriverError(err)
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

impl From<LifoError> for Error {
    fn from(err: LifoError) -> Self {
        Self::LifoError(err)
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

impl From<AcpiError> for Error {
    fn from(err: AcpiError) -> Self {
        Self::AcpiError(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
