use std::error::Error;

use futures::{
    executor::LocalPool,
    stream::{self, StreamExt, TryStreamExt},
};
use winit::event_loop::ControlFlow;

use crate::{EventHandler, EventHandlingOutcome, EventLoop};

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

        let mut local_pool = LocalPool::new();

        winit_event_loop.run(move |_event, _, control_flow| {
            // We cannot run event handlers in this closure, as it isn't `async`, but we can
            // submit them to a local executor to be run on the main thread.

            let event_handlers_task = Self::run_once(&mut event_handlers);

            // Run the event handlers
            let event_handling_outcome = local_pool.run_until(event_handlers_task);
            *control_flow = match event_handling_outcome {
                Ok(EventHandlingOutcome::Continue) => ControlFlow::Poll,
                Ok(EventHandlingOutcome::Exit) => ControlFlow::Exit,
                Err(_e) => {
                    // TODO: error reporting
                    ControlFlow::Exit
                }
            };
        });
    }

    async fn run_once(event_handlers: &mut [EventHandler<E>]) -> Result<EventHandlingOutcome, E> {
        let stream = stream::iter(event_handlers.iter_mut());

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
