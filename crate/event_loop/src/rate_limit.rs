use std::{num::NonZeroU32, time::Duration};

use governor::Quota;

use crate::Error;

/// Rate limit event handler execution.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RateLimit {
    /// Maximum number of ticks per second the event handler may run.
    ///
    /// The [`RateLimit::fps`] function is provided to construct this variant
    /// with error checking.
    Fps(Quota),
    /// Minimum duration between two invocations of the event handler.
    ///
    /// The [`RateLimit::interval`] function is provided to construct this
    /// variant.
    Interval(Option<Quota>),
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
            .map(|fps| RateLimit::Fps(Quota::per_second(fps)))
    }

    /// Returns a `RateLimit::Interval`.
    ///
    /// # Parameters
    ///
    /// * `interval`: Duration to wait between invocations of the event handler.
    pub fn interval(interval: Duration) -> Self {
        let quota = Quota::with_period(interval);

        RateLimit::Interval(quota)
    }

    /// Returns the quota, if any.
    pub fn quota(self) -> Option<Quota> {
        match self {
            RateLimit::Fps(quota) => Some(quota),
            RateLimit::Interval(quota) => quota,
        }
    }
}
