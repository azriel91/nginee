use std::{error::Error, time::Duration};

use futures::stream::{self, Stream, StreamExt};

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
                        // On WASM, without a `sleep` call, the browser will hang.
                        // Even if we sleep for a 0 duration, the browser will remain responsive.
                        .unwrap_or_else(|| Duration::from_nanos(0));

                    // Note: this does not take into account processing time.
                    stream::unfold(duration, move |duration| async move {
                        async_std::task::sleep(duration).await;
                        Some((index, duration))
                    })
                    .boxed()
                });

        stream::select_all(interval_streams)
    }
}
