use crate::{
    graphics::{frame_buf::FrameBufferError, frame_buf_console::FrameBufferConsoleError},
    mem::{bitmap::BitmapMemoryManagerError, paging::PageManagerError},
    util::ascii::AsciiCodeError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Failed(&'static str),
    AsciiCodeError(AsciiCodeError),
    FrameBufferError(FrameBufferError),
    FrameBufferConsoleError(FrameBufferConsoleError),
    BitmapMemoryManagerError(BitmapMemoryManagerError),
    PageManagerError(PageManagerError),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        return Error::Failed(s);
    }
}

impl From<AsciiCodeError> for Error {
    fn from(err: AsciiCodeError) -> Self {
        return Error::AsciiCodeError(err);
    }
}

impl From<FrameBufferError> for Error {
    fn from(err: FrameBufferError) -> Self {
        return Error::FrameBufferError(err);
    }
}

impl From<FrameBufferConsoleError> for Error {
    fn from(err: FrameBufferConsoleError) -> Self {
        return Error::FrameBufferConsoleError(err);
    }
}

impl From<BitmapMemoryManagerError> for Error {
    fn from(err: BitmapMemoryManagerError) -> Self {
        return Error::BitmapMemoryManagerError(err);
    }
}

impl From<PageManagerError> for Error {
    fn from(err: PageManagerError) -> Self {
        return Error::PageManagerError(err);
    }
}

pub type Result<T> = core::result::Result<T, Error>;
