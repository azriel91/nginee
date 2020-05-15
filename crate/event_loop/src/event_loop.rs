use std::error::Error;

use crate::EventHandler;

#[cfg(all(target_arch = "wasm32", feature = "rate_limit"))]
mod rate_limit_async_std;
#[cfg(all(not(target_arch = "wasm32"), feature = "rate_limit"))]
mod rate_limit_governor;
#[cfg(not(feature = "rate_limit"))]
mod rate_limit_off;

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
}
