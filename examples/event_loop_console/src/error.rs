use std::fmt;

#[derive(Debug)]
pub struct Error(async_std::io::Error);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error")
    }
}

impl std::error::Error for Error {}

impl From<async_std::io::Error> for Error {
    fn from(e: async_std::io::Error) -> Self {
        Self(e)
    }
}
