use crate::util::ascii::AsciiCodeError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Failed(&'static str),
    AsciiCodeError(AsciiCodeError),
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

pub type Result<T> = core::result::Result<T, Error>;
