use core::{
    fmt::{self, Debug},
    pin::Pin,
};
use std::{
    error::Error,
    future::Future,
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
pub struct EventLoop<E, UserEvent = ()>
where
    UserEvent: 'static,
{
    /// `EventHandler`s to run during event loop execution.
    event_handlers: Vec<EventHandler<E>>,
    /// The `winit` event loop to run.
    winit_event_loop: WinitEventLoop<UserEvent>,
    /// Task to run on exit.
    exit_handler: Option<Box<dyn FnOnce(Option<E>) -> Pin<Box<dyn Future<Output = ()>>>>>,
}

impl<E, UserEvent> Debug for EventLoop<E, UserEvent>
where
    E: Debug,
    UserEvent: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_struct = f.debug_struct("EventHandler");

        debug_struct.field("event_handlers", &self.event_handlers);
        debug_struct.field("winit_event_loop", &self.winit_event_loop);
        if self.exit_handler.is_some() {
            debug_struct.field("exit_handler", &"Some(..)");
        } else {
            debug_struct.field("exit_handler", &"None");
        }

        debug_struct.finish()
    }
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
            exit_handler: None,
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
            exit_handler: None,
        }
    }

    /// Returns a new `EventLoop`.
    ///
    /// # Parameters
    ///
    /// * `event_handlers`: The logic to run for each event loop execution.
    #[cfg(any(unix, windows))]
    pub fn new_any_thread(event_handlers: Vec<EventHandler<E>>) -> Self {
        #[cfg(unix)]
        use winit::platform::unix::EventLoopExtUnix;
        #[cfg(windows)]
        use winit::platform::windows::EventLoopExtWindows;

        let winit_event_loop = WinitEventLoop::new_any_thread();

        Self {
            event_handlers,
            winit_event_loop,
            exit_handler: None,
        }
    }
}

impl<E, UserEvent> EventLoop<E, UserEvent>
where
    E: Error,
    UserEvent: 'static,
{
    /// Sets a function to run when the event loop exits.
    ///
    /// There is only one exit event handler, so setting this twice will replace
    /// the first one.
    pub fn with_exit_handler<FnExitHandler, ExitHandler>(
        mut self,
        fn_exit_handler: FnExitHandler,
    ) -> Self
    where
        FnExitHandler: FnOnce(Option<E>) -> ExitHandler + 'static,
        ExitHandler: Future<Output = ()> + 'static,
    {
        self.exit_handler = Some(Box::new(|event_handling_outcome| {
            Box::pin(fn_exit_handler(event_handling_outcome))
        }));
        self
    }

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
