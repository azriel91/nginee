use std::error::Error;

use futures::stream::{self, Stream, StreamExt};
use governor::{
    clock::DefaultClock,
    prelude::StreamRateLimitExt,
    state::{direct::NotKeyed, InMemoryState},
};

use crate::{EventHandlingOutcome, EventLoop};

type RateLimiter = governor::RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

impl<E> EventLoop<E>
where
    E: Error,
{
    /// Runs the event loop until `Exit` is signalled or an error occurs.
    pub async fn run(mut self) -> Result<(), E> {
        let rate_limiters = self.rate_limiters();
        let mut event_handler_streams = self.event_handler_streams(&rate_limiters);

        while let Some(index) = event_handler_streams.next().await {
            let event_handler = &mut self.event_handlers[index];
            match event_handler.run().await {
                Ok(EventHandlingOutcome::Continue) => {}
                Ok(EventHandlingOutcome::Exit) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        #[cfg_attr(tarpaulin, skip)]
        Ok(())
    }

    fn rate_limiters(&mut self) -> Vec<Option<RateLimiter>> {
        self.event_handlers
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

    fn event_handler_streams<'r>(
        &mut self,
        rate_limiters: &'r [Option<RateLimiter>],
    ) -> impl Stream<Item = usize> + 'r {
        let event_handler_streams = rate_limiters
            .iter()
            .enumerate()
            .map(|(index, rate_limiter)| {
                if let Some(rate_limiter) = rate_limiter {
                    stream::repeat(index).ratelimit_stream(rate_limiter).boxed()
                } else {
                    stream::repeat(index).boxed()
                }
            })
            .collect::<Vec<_>>();

        stream::select_all(event_handler_streams)
    }
}
