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

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU32, time::Duration};

    use crate::{Error, Quota, RateLimit};

    #[test]
    fn fps_zero_returns_error() {
        assert_eq!(Err(Error::RateLimitFpsZero), RateLimit::fps(0));
    }

    #[test]
    fn fps_non_zero_returns_ok() {
        assert_eq!(
            Ok(RateLimit::Fps(Quota::per_second(
                NonZeroU32::new(1).unwrap()
            ))),
            RateLimit::fps(1)
        );
    }

    #[test]
    fn quota_from_60_fps_returns_quota() -> Result<(), Error> {
        assert_eq!(
            Some(Quota::per_second(NonZeroU32::new(60).unwrap())),
            RateLimit::fps(60)?.quota()
        );
        Ok(())
    }

    #[test]
    fn quota_from_interval_returns_itself() {
        assert_eq!(
            Quota::with_period(Duration::from_millis(11)),
            RateLimit::interval(Duration::from_millis(11)).quota()
        );
    }
}
