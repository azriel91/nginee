use std::{num::NonZeroU32, time::Duration};

use crate::Error;

/// Rate limit event handler execution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RateLimit {
    /// Maximum number of ticks per second the event handler may run.
    ///
    /// This will be converted to a nanosecond interval, so 60 FPS is
    /// equivalent to 16 667 ms (1_000_000 / 60);
    ///
    /// The [`RateLimit::fps`] function is provided to construct this variant
    /// with error checking.
    Fps(NonZeroU32),
    /// Minimum duration between two invocations of the event handler.
    ///
    /// The [`RateLimit::interval`] function is provided to construct this
    /// variant.
    Interval(Duration),
}

impl RateLimit {
    /// Returns a `RateLimit::Fps` after validating the fps provided is
    /// non-zero.
    ///
    /// # Parameters
    ///
    /// * `fps`: Maximum number of ticks per second the event handler may run.
    pub fn fps(fps: u32) -> Result<Self, Error> {
        NonZeroU32::new(fps)
            .ok_or(Error::RateLimitFpsZero)
            .map(RateLimit::Fps)
    }

    /// Returns a `RateLimit::Interval`.
    ///
    /// # Parameters
    ///
    /// * `interval`: Duration to wait between invocations of the event handler.
    pub fn interval(interval: Duration) -> Self {
        RateLimit::Interval(interval)
    }
}

impl From<RateLimit> for Duration {
    fn from(rate_limit: RateLimit) -> Duration {
        match rate_limit {
            RateLimit::Fps(fps) => {
                let nanos = 1_000_000 / u64::from(fps.get());
                Duration::from_nanos(nanos)
            }
            RateLimit::Interval(duration) => duration,
        }
    }
}
