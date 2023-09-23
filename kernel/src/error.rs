#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Failed(&'static str),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        return Error::Failed(s);
    }
}

pub type Result<T> = core::result::Result<T, Error>;
