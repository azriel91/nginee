use std::error::Error;

use futures::{
    executor::LocalPool,
    stream::{self, StreamExt, TryStreamExt},
};
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoopWindowTarget},
};

use super::ExitHandler;
use crate::{EventHandler, EventHandlingOutcome, EventLoop};

struct EventLoopParams<'f, E> {
    event_handlers: Vec<EventHandler<E>>,
    exit_handler: Option<ExitHandler<E>>,
    local_pool: LocalPool,
    should_exit: bool,
    marker: std::marker::PhantomData<&'f E>,
}

struct EventLoopParamsRef<'f, E> {
    event_handlers: &'f mut [EventHandler<E>],
    exit_handler: &'f mut Option<ExitHandler<E>>,
    local_pool: &'f mut LocalPool,
    should_exit: &'f mut bool,
}

impl<E, UserEvent> EventLoop<E, UserEvent>
where
    E: Error + Send + 'static,
    UserEvent: 'static,
{
    /// Runs the event loop until `Exit` is signalled or an error occurs.
    #[cfg_attr(tarpaulin, skip)]
    pub async fn run(self) -> ! {
        let EventLoop {
            event_handlers,
            winit_event_loop,
            exit_handler,
        } = self;

        let event_loop_params = EventLoopParams {
            event_handlers,
            exit_handler,
            local_pool: LocalPool::new(),
            should_exit: false,
            marker: std::marker::PhantomData,
        };

        winit_event_loop.run(Self::fn_event_loop(event_loop_params));
    }

    /// Runs the event loop until `Exit` is signalled or an error occurs.
    ///
    /// For native execution, we use
    /// [`winit::event_loop::EventLoop::run_return`] as this allows tests to use
    /// native window libraries and not segfault.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn run_return(self) {
        use winit::platform::desktop::EventLoopExtDesktop;

        let EventLoop {
            mut event_handlers,
            mut winit_event_loop,
            mut exit_handler,
        } = self;

        let mut local_pool = LocalPool::new();

        let event_handlers = &mut event_handlers;
        let exit_handler = &mut exit_handler;
        let local_pool = &mut local_pool;
        let mut should_exit = false;

        while !should_exit {
            let should_exit = &mut should_exit;
            let event_loop_params_ref = EventLoopParamsRef {
                event_handlers,
                exit_handler,
                local_pool,
                should_exit,
            };

            winit_event_loop.run_return(Self::fn_event_loop_return(event_loop_params_ref));
        }
    }

    #[cfg_attr(tarpaulin, skip)]
    fn fn_event_loop<'f>(
        mut event_loop_params: EventLoopParams<'f, E>,
    ) -> impl FnMut(Event<UserEvent>, &EventLoopWindowTarget<UserEvent>, &mut ControlFlow) + 'f
    {
        move |_event, _, control_flow| {
            let EventLoopParams {
                ref mut event_handlers,
                ref mut exit_handler,
                ref mut local_pool,
                ref mut should_exit,
                ..
            } = event_loop_params;

            let mut event_loop_params_ref = EventLoopParamsRef {
                event_handlers,
                exit_handler,
                local_pool,
                should_exit,
            };

            Self::run_event_handlers(&mut event_loop_params_ref, control_flow);
        }
    }

    fn fn_event_loop_return<'f>(
        mut event_loop_params_ref: EventLoopParamsRef<'f, E>,
    ) -> impl FnMut(Event<UserEvent>, &EventLoopWindowTarget<UserEvent>, &mut ControlFlow) + 'f
    {
        move |_event, _, control_flow| {
            Self::run_event_handlers(&mut event_loop_params_ref, control_flow);
        }
    }

    fn run_event_handlers<'f>(
        EventLoopParamsRef {
            event_handlers,
            exit_handler,
            local_pool,
            should_exit,
        }: &mut EventLoopParamsRef<'f, E>,
        control_flow: &mut ControlFlow,
    ) {
        if !**should_exit {
            let event_handlers_task = Self::run_once(event_handlers);

            // Run the event handlers
            let event_handling_outcome = local_pool.run_until(event_handlers_task);
            let mut error = None;
            *control_flow = match event_handling_outcome {
                Ok(EventHandlingOutcome::Continue) => ControlFlow::Poll,
                Ok(EventHandlingOutcome::Exit) => ControlFlow::Exit,
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
