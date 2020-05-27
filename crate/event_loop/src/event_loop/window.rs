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

type ExitHandler<E> = Box<dyn FnOnce(Option<E>) -> Pin<Box<dyn Future<Output = ()>>>>;

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
    exit_handler: Option<ExitHandler<E>>,
    /// Whether the event loop is run in the main thread.
    is_in_main_thread: bool,
}

impl<E, UserEvent> Debug for EventLoop<E, UserEvent>
where
    E: Debug,
    UserEvent: Debug,
{
    #[cfg_attr(tarpaulin, skip)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_struct = f.debug_struct("EventHandler");

        debug_struct.field("event_handlers", &self.event_handlers);
        debug_struct.field("winit_event_loop", &self.winit_event_loop);
        if self.exit_handler.is_some() {
            debug_struct.field("exit_handler", &"Some(..)");
        } else {
            debug_struct.field("exit_handler", &"None");
        }
        debug_struct.field("is_in_main_thread", &self.is_in_main_thread);

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
    #[cfg_attr(tarpaulin, skip)]
    pub fn new(event_handlers: Vec<EventHandler<E>>) -> Self {
        let winit_event_loop = WinitEventLoop::with_user_event();

        Self {
            event_handlers,
            winit_event_loop,
            exit_handler: None,
            is_in_main_thread: true,
        }
    }

    /// Returns a new `EventLoop`.
    ///
    /// # Parameters
    ///
    /// * `event_handlers`: The logic to run for each event loop execution.
    #[cfg_attr(tarpaulin, skip)]
    pub fn new_with_event<UserEvent>(
        event_handlers: Vec<EventHandler<E>>,
    ) -> EventLoop<E, UserEvent> {
        let winit_event_loop = WinitEventLoop::with_user_event();

        EventLoop::<E, UserEvent> {
            event_handlers,
            winit_event_loop,
            exit_handler: None,
            is_in_main_thread: true,
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
            is_in_main_thread: false,
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
    #[cfg_attr(tarpaulin, skip)]
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

    #[cfg_attr(tarpaulin, skip)]
    fn deref(&self) -> &Self::Target {
        &self.winit_event_loop
    }
}

impl<E, UserEvent> DerefMut for EventLoop<E, UserEvent>
where
    E: Error,
    UserEvent: 'static,
{
    #[cfg_attr(tarpaulin, skip)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.winit_event_loop
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    #[cfg(feature = "rate_limit")]
    use std::time::Duration;

    use crossbeam_channel::{SendError, Sender};

    #[cfg(feature = "rate_limit")]
    use crate::RateLimit;
    use crate::{EventHandler, EventHandlingOutcome, EventLoop};

    #[test]
    fn run_runs_event_handlers_until_exit_is_signalled() -> Result<(), SendError<()>> {
        let (done_tx, done_rx) = crossbeam_channel::bounded(1);
        let (tx, rx) = crossbeam_channel::bounded(10);

        thread::spawn(|| {
            let event_handler_send = sender(tx);
            let event_handler_countdown = countdown(3);

            let event_loop =
                EventLoop::new_any_thread(vec![event_handler_send, event_handler_countdown])
                    .with_exit_handler(|e| async move {
                        let _ = done_tx.send(e);
                    });

            smol::run(event_loop.run_return());
        });
        let run_result = done_rx.recv();
        assert_eq!(Ok(None), run_result);

        let count = rx.try_iter().collect::<Vec<()>>().len();
        assert_eq!(3, count);

        Ok(())
    }

    #[test]
    fn run_returns_on_first_error() -> Result<(), SendError<()>> {
        let (done_tx, done_rx) = crossbeam_channel::bounded(1);
        let (tx, _rx) = crossbeam_channel::bounded(10);

        thread::spawn(|| {
            let event_handler_send = sender(tx);
            let event_handler_countdown = countdown(3);

            let event_loop = EventLoop::new_any_thread(vec![
                event_handler_send,
                errorer(),
                event_handler_countdown,
            ])
            .with_exit_handler(|e| async move {
                let _ = done_tx.send(e);
            });

            smol::run(event_loop.run_return());
        });

        let run_result = done_rx.recv();
        assert_eq!(Ok(Some(SendError(()))), run_result);

        Ok(())
    }

    #[cfg(feature = "rate_limit")]
    #[test]
    fn event_handlers_are_rate_limited_independently() -> Result<(), SendError<()>> {
        let (done_tx, done_rx) = crossbeam_channel::bounded(1);
        let (tx0, rx0) = crossbeam_channel::unbounded();
        let (tx1, rx1) = crossbeam_channel::bounded(10);

        thread::spawn(|| {
            let event_handler_send_0 = sender(tx0);

            let event_handler_send_1 =
                sender(tx1).with_rate_limit(RateLimit::interval(Duration::from_millis(2)));
            let event_handler_countdown =
                countdown(3).with_rate_limit(RateLimit::interval(Duration::from_millis(3)));

            let event_loop = EventLoop::new_any_thread(vec![
                event_handler_countdown,
                event_handler_send_1,
                event_handler_send_0,
            ])
            .with_exit_handler(|e| async move {
                let _ = done_tx.send(e);
            });
            smol::run(event_loop.run_return());
        });

        let _ = done_rx.recv();

        let count_0 = rx0.try_iter().collect::<Vec<()>>().len();
        let count_1 = rx1.try_iter().collect::<Vec<()>>().len();

        assert!(count_0 >= 8, "count_0: {}", count_0);
        assert!(count_1 >= 2, "count_1: {}", count_1);
        assert!(count_1 <= 6, "count_1: {}", count_1);

        Ok(())
    }

    fn sender(tx: Sender<()>) -> EventHandler<SendError<()>> {
        EventHandler::<SendError<()>>::new(move || {
            let tx = tx.clone();
            async move {
                tx.send(())?;

                Ok(EventHandlingOutcome::Continue)
            }
        })
    }

    fn countdown(mut count: u32) -> EventHandler<SendError<()>> {
        EventHandler::<SendError<()>>::new(move || {
            count = count.saturating_sub(1);
            async move {
                if count > 0 {
                    Ok(EventHandlingOutcome::Continue)
                } else {
                    Ok(EventHandlingOutcome::Exit)
                }
            }
        })
    }

    fn errorer() -> EventHandler<SendError<()>> {
        EventHandler::<SendError<()>>::new(|| async move { Err(SendError(())) })
    }
}
