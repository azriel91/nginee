use crate::{event_loop::common::RateLimiter, EventHandler};

pub(crate) trait EventHandlersExt<E> {
    fn rate_limiters(&self) -> Vec<Option<RateLimiter>>;
}

impl<T, E> EventHandlersExt<E> for T
where
    T: AsRef<[EventHandler<E>]>,
{
    /// Returns `RateLimiter`s constructed from each event handler's
    /// [`RateLimit`].
    fn rate_limiters(&self) -> Vec<Option<RateLimiter>> {
        AsRef::<[EventHandler<E>]>::as_ref(self)
            .iter()
            .map(|event_handler| {
                let quota = event_handler
                    .rate_limit
                    .and_then(|rate_limit| rate_limit.quota());

                // On WASM, if you have a non-rate-limited event handler, the browser will
                // freeze when running single threaded.
                #[cfg(target_arch = "wasm32")]
                #[cfg_attr(tarpaulin, skip)]
                let quota = {
                    use governor::Quota;
                    use std::time::Duration;
                    quota.or_else(|| Quota::with_period(Duration::from_nanos(1)))
                };

                quota.map(RateLimiter::direct)
            })
            .collect::<Vec<_>>()
    }
}
