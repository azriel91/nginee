#[cfg(test)]
mod tests {
    use std::{num::NonZeroU32, time::Duration};

    use nginee::event_loop::{Error, RateLimit};

    #[test]
    fn fps_zero_returns_error() {
        assert_eq!(Err(Error::RateLimitFpsZero), RateLimit::fps(0));
    }

    #[test]
    fn fps_non_zero_returns_ok() {
        assert_eq!(
            Ok(RateLimit::Fps(NonZeroU32::new(1).unwrap())),
            RateLimit::fps(1)
        );
    }

    #[test]
    fn duration_from_60_fps_returns_16_666_ns() -> Result<(), Error> {
        assert_eq!(
            Duration::from_nanos(16_666), // rounds down
            Duration::from(RateLimit::fps(60)?)
        );
        Ok(())
    }

    #[test]
    fn duration_from_interval_returns_itself() {
        assert_eq!(
            Duration::from_millis(11),
            Duration::from(RateLimit::Interval(Duration::from_millis(11)))
        );
    }
}
