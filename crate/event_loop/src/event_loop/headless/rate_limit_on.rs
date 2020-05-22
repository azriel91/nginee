use std::error::Error;

use futures::stream::{self, Stream, StreamExt};
use governor::prelude::StreamRateLimitExt;

use crate::{
    event_loop::common::{EventHandlersExt, RateLimiter},
    EventHandlingOutcome, EventLoop,
};

impl<E> EventLoop<E>
where
    E: Error + Send + 'static,
{
    /// Runs the event loop until `Exit` is signalled or an error occurs.
    pub async fn run(mut self) -> Result<(), E> {
        let rate_limiters = self.event_handlers.rate_limiters();
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
