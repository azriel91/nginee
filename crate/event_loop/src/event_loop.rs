use std::error::Error;
#[cfg(feature = "rate_limit")]
use std::time::Duration;

#[cfg(feature = "rate_limit")]
use futures::stream::Stream;
#[cfg(not(feature = "rate_limit"))]
use futures::stream::TryStreamExt;
use futures::stream::{self, StreamExt};

use crate::{EventHandler, EventHandlingOutcome};

/// Receives events and runs an event handler function.
///
/// # Type Parameters
///
/// * `E`: Error type.
#[derive(Debug)]
pub struct EventLoop<E> {
    /// `EventHandler`s to run during event loop execution.
    event_handlers: Vec<EventHandler<E>>,
}

impl<E> EventLoop<E>
where
    E: Error,
{
    /// Returns a new `EventLoop`.
    ///
    /// # Parameters
    ///
    /// * `event_handlers`: The logic to run for each event loop execution.
    pub fn new(event_handlers: Vec<EventHandler<E>>) -> Self {
        Self { event_handlers }
    }

    /// Runs the event loop until `Exit` is signalled or an error occurs.
    #[cfg(feature = "rate_limit")]
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

    #[cfg(feature = "rate_limit")]
    fn interval_streams(&mut self) -> impl Stream<Item = usize> {
        let interval_streams = self
            .event_handlers
            .iter()
            .enumerate()
            .map(|(index, event_handler)| {
                let duration = event_handler
                    .rate_limit
                    .map(Duration::from)
                    // TODO: Using `Duration::from_millis(0)` causes this stream to starve all other
                    // streams from running.
                    .unwrap_or_else(|| Duration::from_millis(1));
                async_std::stream::interval(duration).map(move |_| index)
            })
            .collect::<Vec<_>>();

        stream::select_all(interval_streams)
    }

    /// Runs the event loop until `Exit` is signalled or an error occurs.
    #[cfg(not(feature = "rate_limit"))]
    pub async fn run(mut self) -> Result<(), E> {
        loop {
            match self.run_once().await {
                Ok(EventHandlingOutcome::Continue) => {}
                Ok(EventHandlingOutcome::Exit) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    #[cfg(not(feature = "rate_limit"))]
    async fn run_once(&mut self) -> Result<EventHandlingOutcome, E> {
        let stream = stream::iter(self.event_handlers.iter_mut());

        stream
            .map(Result::<_, E>::Ok)
            .try_fold(
                EventHandlingOutcome::Continue,
                |outcome_cumulative, event_handler| async move {
                    event_handler
                        .run()
                        .await
                        .map(|outcome| core::cmp::max(outcome_cumulative, outcome))
                },
            )
            .await
    }
}
