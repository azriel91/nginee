use std::fmt;

#[derive(Debug)]
pub struct Error(tokio::io::Error);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error")
    }
}

impl std::error::Error for Error {}

impl From<tokio::io::Error> for Error {
    fn from(e: tokio::io::Error) -> Self {
        Self(e)
    }
}
