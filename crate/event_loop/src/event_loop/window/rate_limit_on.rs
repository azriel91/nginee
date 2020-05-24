use core::future::Future;
use std::{error::Error, time::Duration};

use futures::{
    executor::LocalPool,
    stream::{self, StreamExt},
    TryStreamExt,
};
use governor::clock::{Clock, DefaultClock};
use instant::Instant;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoopWindowTarget},
};

use super::ExitHandler;
use crate::{
    event_loop::common::{EventHandlersExt, RateLimiter},
    EventHandler, EventHandlingOutcome, EventLoop,
};

#[cfg(not(target_arch = "wasm32"))]
struct EventLoopParams<'f, E> {
    event_handlers: &'f mut [EventHandler<E>],
    exit_handler: &'f mut Option<ExitHandler<E>>,
    rate_limiters: &'f [Option<RateLimiter>],
    clock: &'f DefaultClock,
    local_pool: &'f mut LocalPool,
    should_exit: &'f mut bool,
}

#[cfg(target_arch = "wasm32")]
struct EventLoopParams<'f, E> {
    event_handlers: Vec<EventHandler<E>>,
    exit_handler: Option<ExitHandler<E>>,
    rate_limiters: Vec<Option<RateLimiter>>,
    clock: DefaultClock,
    local_pool: LocalPool,
    should_exit: bool,
    marker: std::marker::PhantomData<&'f E>,
}

impl<E, UserEvent> EventLoop<E, UserEvent>
where
    E: Error + Send + 'static,
    UserEvent: 'static,
{
    /// Runs the event loop until `Exit` is signalled or an error occurs.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn run(self) {
        use winit::platform::desktop::EventLoopExtDesktop;

        let EventLoop {
            mut event_handlers,
            mut winit_event_loop,
            mut exit_handler,
            is_in_main_thread,
        } = self;
        let rate_limiters = event_handlers.rate_limiters();

        let mut local_pool = LocalPool::new();
        let clock = DefaultClock::default();

        let event_handlers = &mut event_handlers;
        let exit_handler = &mut exit_handler;
        let rate_limiters = &rate_limiters;
        let clock = &clock;
        let local_pool = &mut local_pool;
        let mut should_exit = false;

        while !should_exit {
            let should_exit = &mut should_exit;
            let event_loop_params = EventLoopParams {
                event_handlers,
                exit_handler,
                rate_limiters,
                clock,
                local_pool,
                should_exit,
            };
            winit_event_loop.run_return(Self::fn_event_loop(event_loop_params));
        }

        if is_in_main_thread {
            std::process::exit(0);
        }
    }

    /// Runs the event loop until `Exit` is signalled or an error occurs.
    #[cfg(target_arch = "wasm32")]
    #[cfg_attr(tarpaulin, skip)]
    pub async fn run(self) {
        let EventLoop {
            event_handlers,
            winit_event_loop,
            exit_handler,
            is_in_main_thread,
        } = self;

        let rate_limiters = event_handlers.rate_limiters();

        let event_loop_params = EventLoopParams {
            event_handlers,
            exit_handler,
            rate_limiters,
            clock: DefaultClock::default(),
            local_pool: LocalPool::new(),
            should_exit: false,
            marker: std::marker::PhantomData,
        };

        winit_event_loop.run(Self::fn_event_loop(event_loop_params));
    }

    fn fn_event_loop<'f>(
        mut event_loop_params: EventLoopParams<'f, E>,
    ) -> impl FnMut(Event<UserEvent>, &EventLoopWindowTarget<UserEvent>, &mut ControlFlow) + 'f
    {
        move |_event, _, control_flow| {
            // Run event handlers that are ready.

            // We cannot run event handlers in this closure, as it isn't `async`, but we can
            // submit them to a local executor to be run on the main thread.

            let EventLoopParams {
                ref mut event_handlers,
                ref mut exit_handler,
                ref rate_limiters,
                ref clock,
                ref mut local_pool,
                ref mut should_exit,
                ..
            } = event_loop_params;

            if !**should_exit {
                let event_handlers_task =
                    Self::event_handler_task(event_handlers, rate_limiters, clock);

                // Run the event handlers
                let outcome = local_pool.run_until(event_handlers_task);
                let mut error = None;
                *control_flow = match outcome {
                    Ok((event_handling_outcome, duration_to_wait)) => {
                        match event_handling_outcome {
                            Ok(EventHandlingOutcome::Continue) => {
                                match duration_to_wait {
                                    Some(duration) => {
                                        // Need to do this, because rate limit instant may be a
                                        // different type, such as `QuantaInstant`.
                                        let instant = Instant::now() + duration;
                                        ControlFlow::WaitUntil(instant)
                                    }
                                    None => ControlFlow::Poll,
                                }
                            }
                            Ok(EventHandlingOutcome::Exit) => ControlFlow::Exit,
                            Err(e) => {
                                error = Some(e);
                                ControlFlow::Exit
                            }
                        }
                    }
                    Err(e) => {
                        error = Some(e);
                        ControlFlow::Exit
                    }
                };

                if *control_flow == ControlFlow::Exit {
                    **should_exit = true;
                    if let Some(exit_handler) = exit_handler.take() {
                        local_pool.run_until(exit_handler(error));
                    }
                }
            }
        }
    }

    fn event_handler_task<'f>(
        event_handlers: &'f mut [EventHandler<E>],
        rate_limiters: &'f [Option<RateLimiter>],
        clock: &'f DefaultClock,
    ) -> impl Future<Output = Result<(Result<EventHandlingOutcome, E>, Option<Duration>), E>> + 'f
    {
        async move {
            stream::iter(rate_limiters.iter().zip(event_handlers.iter_mut()))
                .map(Result::Ok)
                .try_fold(
                    (Ok(EventHandlingOutcome::Continue), None),
                    |(outcome_cumulative, duration_to_wait), (rate_limiter, event_handler)| {
                        Self::run_event_handler(
                            clock,
                            outcome_cumulative,
                            duration_to_wait,
                            rate_limiter,
                            event_handler,
                        )
                    },
                )
                .await
        }
    }

    /// Returns a future that
    fn run_event_handler<'f>(
        clock: &'f DefaultClock,
        mut outcome_cumulative: Result<EventHandlingOutcome, E>,
        mut duration_to_wait: Option<Duration>,
        rate_limiter: &'f Option<RateLimiter>,
        event_handler: &'f mut EventHandler<E>,
    ) -> impl Future<Output = Result<(Result<EventHandlingOutcome, E>, Option<Duration>), E>> + 'f
    {
        async move {
            if let Some(rate_limiter) = rate_limiter {
                match rate_limiter.check() {
                    Ok(()) => {
                        let outcome = event_handler.run().await;
                        outcome_cumulative = Self::outcome_merge(outcome_cumulative, outcome);
                        duration_to_wait = match rate_limiter.check() {
                            Ok(()) => Some(Duration::from_millis(0)),
                            Err(not_until) => {
                                let duration_to_wait_rate_limiter =
                                    not_until.wait_time_from(clock.now());
                                duration_to_wait
                                    .map(|duration| {
                                        std::cmp::min(duration, duration_to_wait_rate_limiter)
                                    })
                                    .or_else(|| Some(duration_to_wait_rate_limiter))
                            }
                        };
                    }
                    Err(not_until) => {
                        let duration_to_wait_rate_limiter = not_until.wait_time_from(clock.now());
                        duration_to_wait = duration_to_wait
                            .map(|duration| std::cmp::min(duration, duration_to_wait_rate_limiter))
                            .or_else(|| Some(duration_to_wait_rate_limiter));
                    }
                }
            } else {
                let outcome = event_handler.run().await;
                outcome_cumulative = Self::outcome_merge(outcome_cumulative, outcome);
                duration_to_wait = Some(Duration::from_millis(0));
            }

            Result::<_, E>::Ok((outcome_cumulative, duration_to_wait))
        }
    }

    fn outcome_merge(
        base: Result<EventHandlingOutcome, E>,
        patch: Result<EventHandlingOutcome, E>,
    ) -> Result<EventHandlingOutcome, E> {
        match (base, patch) {
            (Err(e), _) | (Ok(_), Err(e)) => Err(e),
            (Ok(outcome_base), Ok(outcome_patch)) => Ok(std::cmp::max(outcome_base, outcome_patch)),
        }
    }
}
