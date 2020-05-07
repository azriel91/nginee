use core::{future::Future, marker::Unpin};

/// Receives events and runs an event handler function.
#[derive(Debug)]
pub struct EventLoop;

impl EventLoop {
    /// Runs the event loop forever.
    #[cfg_attr(tarpaulin, skip)]
    pub async fn run(mut event_handler: impl Future<Output = ()> + Unpin) -> ! {
        loop {
            Self::run_once(&mut event_handler).await;
        }
    }

    /// Runs the event loop once.
    pub async fn run_once(event_handler: impl Future<Output = ()>) {
        event_handler.await
    }
}
