use std::{
    error::Error,
    ops::{Deref, DerefMut},
};

use winit::event_loop::EventLoop as WinitEventLoop;

use crate::EventHandler;

#[cfg(not(feature = "rate_limit"))]
mod rate_limit_off;
#[cfg(feature = "rate_limit")]
mod rate_limit_on;

/// Receives events and runs an event handler function.
///
/// This abstracts away running the synchronous winit event loop.
///
/// See:
///
/// * <https://github.com/ryanisaacg/blinds>
/// * <https://github.com/rust-windowing/winit/issues/1199>
/// * <https://github.com/osspial/winit-async>
///
/// # Type Parameters
///
/// * `E`: Error type.
/// * `UserEvent`: Custom user event type, defaults to `()`.
#[derive(Debug)]
pub struct EventLoop<E, UserEvent = ()>
where
    UserEvent: 'static,
{
    /// `EventHandler`s to run during event loop execution.
    event_handlers: Vec<EventHandler<E>>,
    /// The `winit` event loop to run.
    winit_event_loop: WinitEventLoop<UserEvent>,
}

impl<E> EventLoop<E, ()>
where
    E: Error,
{
    /// Returns a new `EventLoop`.
    ///
    /// # Parameters
    ///
    /// * `event_handlers`: The logic to run for each event loop execution.
    pub fn new(event_handlers: Vec<EventHandler<E>>) -> Self {
        let winit_event_loop = WinitEventLoop::with_user_event();

        Self {
            event_handlers,
            winit_event_loop,
        }
    }

    /// Returns a new `EventLoop`.
    ///
    /// # Parameters
    ///
    /// * `event_handlers`: The logic to run for each event loop execution.
    pub fn new_with_event<UserEvent>(
        event_handlers: Vec<EventHandler<E>>,
    ) -> EventLoop<E, UserEvent> {
        let winit_event_loop = WinitEventLoop::with_user_event();

        EventLoop::<E, UserEvent> {
            event_handlers,
            winit_event_loop,
        }
    }
}

impl<E, UserEvent> EventLoop<E, UserEvent>
where
    E: Error,
    UserEvent: 'static,
{
    /// Returns the `WinitEventLoop`.
    pub fn winit_event_loop(&self) -> &WinitEventLoop<UserEvent> {
        &self.winit_event_loop
    }
}

impl<E, UserEvent> Deref for EventLoop<E, UserEvent>
where
    E: Error,
    UserEvent: 'static,
{
    type Target = WinitEventLoop<UserEvent>;

    fn deref(&self) -> &Self::Target {
        &self.winit_event_loop
    }
}

impl<E, UserEvent> DerefMut for EventLoop<E, UserEvent>
where
    E: Error,
    UserEvent: 'static,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.winit_event_loop
    }
}
