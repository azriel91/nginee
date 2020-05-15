use std::{error::Error, time::Duration};

use futures::stream::{self, Stream, StreamExt};
use futures_test::stream::StreamTestExt;

use crate::{EventHandlingOutcome, EventLoop};

impl<E> EventLoop<E>
where
    E: Error,
{
    /// Runs the event loop until `Exit` is signalled or an error occurs.
    pub async fn run(mut self) -> Result<(), E> {
        let mut interval_streams = self.interval_streams();

        while let Some(index) = interval_streams.next().await {
            let event_handler = &mut self.event_handlers[index];
            match event_handler.run().await {
                Ok(EventHandlingOutcome::Continue) => {}
                Ok(EventHandlingOutcome::Exit) => return Ok(()),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    fn interval_streams(&mut self) -> impl Stream<Item = usize> {
        let interval_streams =
            self.event_handlers
                .iter()
                .enumerate()
                .map(|(index, event_handler)| {
                    let duration = event_handler
                        .rate_limit
                        .map(Duration::from)
                        .unwrap_or_else(|| Duration::from_millis(0));

                    async_std::stream::interval(duration).map(move |_| index)
                });

        // `interleave_pending` is a hack so that non-rate-limited streams don't starve
        // rate-limited streams.
        //
        // <https://docs.rs/futures-test/0.3.5/futures_test/stream/trait.StreamTestExt.html#method.interleave_pending>
        stream::select_all(interval_streams).interleave_pending()
    }
}
