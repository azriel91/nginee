use std::error::Error;

use futures::{
    executor::LocalPool,
    stream::{self, Stream, StreamExt},
};
use governor::prelude::StreamRateLimitExt;
use winit::event_loop::ControlFlow;

use crate::{
    event_loop::common::{EventHandlersExt, RateLimiter},
    EventHandlingOutcome, EventLoop,
};

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
        let mut event_handler_streams = Self::event_handler_streams(&rate_limiters);

        let mut local_pool = LocalPool::new();

        let (ehr_channel_tx, ehr_channel_rx) = crossbeam_channel::bounded(event_handlers.len());

        winit_event_loop.run(move |_event, _, control_flow| {
            // Run event handlers that are ready.

            // We cannot run them in this closure, as it isn't `async`, but we can submit
            // them to the executor to be run on the main thread.

            let event_handlers = &mut event_handlers;
            let event_handler_streams = &mut event_handler_streams;
            let ehr_channel_tx = &ehr_channel_tx;
            let event_handlers_task = async move {
                while let Some(index) = event_handler_streams.next().await {
                    let event_handler = &mut event_handlers[index];
                    event_handler.handler_task(ehr_channel_tx.clone()).await
                }
            };

            // Run the event handlers
            local_pool.run_until(event_handlers_task);

            // Collect the results.
            let event_handling_outcome: Result<EventHandlingOutcome, E> =
                ehr_channel_rx.iter().try_fold(
                    EventHandlingOutcome::Continue,
                    |outcome_cumulative, outcome| Ok(core::cmp::max(outcome_cumulative, outcome?)),
                );

            *control_flow = match event_handling_outcome {
                Ok(EventHandlingOutcome::Continue) => ControlFlow::Poll,
                Ok(EventHandlingOutcome::Exit) => ControlFlow::Exit,
                Err(_e) => {
                    // TODO: error handling
                    ControlFlow::Exit
                }
            };
        });
    }

    fn event_handler_streams<'r>(
        rate_limiters: &'r [Option<RateLimiter>],
    ) -> impl Stream<Item = usize> + 'r {
        let event_handler_streams = rate_limiters
            .iter()
            .enumerate()
            .map(|(index, rate_limiter)| {
                if let Some(rate_limiter) = rate_limiter {
                    stream::repeat(index).ratelimit_stream(rate_limiter).boxed()
                } else {
                    stream::repeat(index).boxed()
                }
            })
            .collect::<Vec<_>>();

        stream::select_all(event_handler_streams)
    }
}
