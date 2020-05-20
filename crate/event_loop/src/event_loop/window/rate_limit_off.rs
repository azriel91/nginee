use std::error::Error;

use futures::{
    executor::LocalPool,
    task::{FutureObj, Spawn},
};
use winit::event_loop::ControlFlow;

use crate::{EventHandlingOutcome, EventLoop};

impl<E, UserEvent> EventLoop<E, UserEvent>
where
    E: Error + Send + 'static,
    UserEvent: 'static,
{
    /// Runs the event loop until `Exit` is signalled or an error occurs.
    pub fn run(self) -> ! {
        let EventLoop {
            mut event_handlers,
            winit_event_loop,
        } = self;

        let mut local_pool = LocalPool::new();

        let (ehr_channel_tx, ehr_channel_rx) = crossbeam_channel::bounded(event_handlers.len());

        winit_event_loop.run(move |_event, _, control_flow| {
            // Run event handlers that are ready.

            // We cannot run them in this closure, as it isn't `async`, but we can submit
            // them to the executor to be run on the main thread.

            let spawner = local_pool.spawner();
            let spawn_result = event_handlers
                .iter_mut()
                .map(|event_handler| event_handler.handler_task(ehr_channel_tx.clone()))
                .try_for_each(|event_handler_task| {
                    spawner.spawn_obj(FutureObj::new(event_handler_task))
                });

            if let Err(_e) = spawn_result {
                // TODO: error handling
            }

            // Run the event handlers
            local_pool.run_until_stalled();

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
}
