use std::{error::Error, time::Duration};

use futures::{
    executor::LocalPool,
    stream::{self, StreamExt},
    TryStreamExt,
};
use governor::clock::{Clock, DefaultClock};
use instant::Instant;
use winit::event_loop::ControlFlow;

use crate::{event_loop::common::EventHandlersExt, EventHandlingOutcome, EventLoop};

impl<E, UserEvent> EventLoop<E, UserEvent>
where
    E: Error + Send + 'static,
    UserEvent: 'static,
{
    /// Runs the event loop until `Exit` is signalled or an error occurs.
    pub async fn run(self) -> ! {
        let EventLoop {
            mut event_handlers,
            winit_event_loop,
        } = self;

        let rate_limiters = event_handlers.rate_limiters();

        let mut local_pool = LocalPool::new();
        let clock = DefaultClock::default();

        winit_event_loop.run(move |_event, _, control_flow| {
            // Run event handlers that are ready.

            // We cannot run event handlers in this closure, as it isn't `async`, but we can
            // submit them to a local executor to be run on the main thread.

            let event_handlers = &mut event_handlers;
            let rate_limiters = &rate_limiters;
            let clock = &clock;
            let event_handlers_task = async move {
                stream::iter(rate_limiters.iter().zip(event_handlers.iter_mut()))
                    .map(Result::Ok)
                    .try_fold(
                        (Ok(EventHandlingOutcome::Continue), None),
                        |(mut outcome_cumulative, mut duration_to_wait),
                         (rate_limiter, event_handler)| async move {
                            if let Some(rate_limiter) = rate_limiter {
                                match rate_limiter.check() {
                                    Ok(()) => {
                                        let outcome = event_handler.run().await;
                                        outcome_cumulative =
                                            Self::outcome_merge(outcome_cumulative, outcome);
                                        duration_to_wait = match rate_limiter.check() {
                                            Ok(()) => Some(Duration::from_millis(0)),
                                            Err(not_until) => {
                                                let duration_to_wait_rate_limiter =
                                                    not_until.wait_time_from(clock.now());
                                                duration_to_wait
                                                    .map(|duration| {
                                                        std::cmp::min(
                                                            duration,
                                                            duration_to_wait_rate_limiter,
                                                        )
                                                    })
                                                    .or_else(|| Some(duration_to_wait_rate_limiter))
                                            }
                                        };
                                    }
                                    Err(not_until) => {
                                        let duration_to_wait_rate_limiter =
                                            not_until.wait_time_from(clock.now());
                                        duration_to_wait = duration_to_wait
                                            .map(|duration| {
                                                std::cmp::min(
                                                    duration,
                                                    duration_to_wait_rate_limiter,
                                                )
                                            })
                                            .or_else(|| Some(duration_to_wait_rate_limiter));
                                    }
                                }
                            } else {
                                let outcome = event_handler.run().await;
                                outcome_cumulative =
                                    Self::outcome_merge(outcome_cumulative, outcome);
                                duration_to_wait = Some(Duration::from_millis(0));
                            }

                            Result::<_, E>::Ok((outcome_cumulative, duration_to_wait))
                        },
                    )
                    .await
            };

            // Run the event handlers
            let outcome = local_pool.run_until(event_handlers_task);
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
                        Err(_e) => {
                            // TODO: error reporting
                            ControlFlow::Exit
                        }
                    }
                }
                Err(_e) => {
                    // TODO: error handling
                    ControlFlow::Exit
                }
            }
        });
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
