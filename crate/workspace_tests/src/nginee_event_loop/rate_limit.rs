#[cfg(test)]
mod tests {
    use std::{num::NonZeroU32, time::Duration};

    use nginee::event_loop::{Error, Quota, RateLimit};

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
