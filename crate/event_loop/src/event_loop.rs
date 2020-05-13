use std::error::Error;
#[cfg(feature = "rate_limit")]
use std::time::Duration;

use futures::stream::{self, StreamExt, TryStreamExt};

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
    pub async fn run(mut self) -> Result<(), E> {
        loop {
            match self.run_once().await {
                Ok(EventHandlingOutcome::Continue) => {}
                Ok(EventHandlingOutcome::Exit) => return Ok(()),
                Err(e) => return Err(e),
            }
        }
    }

    /// Runs the event loop once.
    pub async fn run_once(&mut self) -> Result<EventHandlingOutcome, E> {
        let stream = stream::iter(self.event_handlers.iter_mut());

        #[cfg(feature = "rate_limit")]
        let stream = stream
            .zip(async_std::stream::interval(Duration::from_millis(0)))
            .map(|(item, _)| item);

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
