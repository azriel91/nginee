use std::error::Error;

#[cfg(not(feature = "rate_limit"))]
use futures::stream::TryStreamExt;
use futures::stream::{self, StreamExt};

use crate::{EventHandlingOutcome, EventLoop};

impl<E> EventLoop<E>
where
    E: Error,
{
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
