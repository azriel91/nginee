use std::fmt;

/// Errors for the `nginee_event_loop` crate.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    /// The user provided zero for the FPS value.
    RateLimitFpsZero,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FPS must be greater than zero.")
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use crate::Error;

    #[test]
    fn display_returns_human_readable_message() {
        assert_eq!(
            "FPS must be greater than zero.",
            Error::RateLimitFpsZero.to_string()
        )
    }
}
