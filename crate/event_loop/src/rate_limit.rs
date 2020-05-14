use std::{num::NonZeroU16, time::Duration};

use crate::Error;

/// Rate limit event handler execution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RateLimit {
    /// Maximum number of ticks per second the event handler may run.
    ///
    /// This will be converted to a millisecond interval, so 60 FPS is
    /// equivalent to 16 ms (1000 / 60);
    ///
    /// The [`RateLimit::fps`] function is provided to construct this variant
    /// with error checking.
    Fps(NonZeroU16),
    /// Minimum duration between two invocations of the event handler.
    Interval(Duration),
}

impl RateLimit {
    /// Returns a `RateLimit::Fps` after validating the fps provided is
    /// non-zero.
    ///
    /// # Parameters
    ///
    /// * `fps`: Maximum number of ticks per second the event handler may run.
    pub fn fps(fps: u16) -> Result<Self, Error> {
        NonZeroU16::new(fps)
            .ok_or(Error::RateLimitFpsZero)
            .map(RateLimit::Fps)
    }
}

impl From<RateLimit> for Duration {
    fn from(rate_limit: RateLimit) -> Duration {
        match rate_limit {
            RateLimit::Fps(fps) => {
                let millis = 1000u64 / u64::from(fps.get());
                Duration::from_millis(millis)
            }
            RateLimit::Interval(duration) => duration,
        }
    }
}
