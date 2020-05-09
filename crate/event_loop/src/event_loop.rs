use futures::stream::{self, StreamExt};

use crate::EventHandler;

/// Receives events and runs an event handler function.
#[derive(Debug)]
pub struct EventLoop {
    /// `EventHandler`s to run during event loop execution.
    event_handlers: Vec<EventHandler>,
}

impl EventLoop {
    /// Returns a new `EventLoop`.
    ///
    /// # Parameters
    ///
    /// * `event_handler`:
    pub fn new(event_handlers: Vec<EventHandler>) -> Self {
        Self { event_handlers }
    }

    /// Runs the event loop.
    #[cfg_attr(tarpaulin, skip)]
    pub async fn run(mut self) -> Result<(), ()> {
        loop {
            self.run_once().await;
        }
    }

    /// Runs the event loop once.
    pub async fn run_once(&mut self) {
        stream::iter(self.event_handlers.iter_mut())
            .for_each(|event_handler| event_handler.run())
            .await
    }
}
