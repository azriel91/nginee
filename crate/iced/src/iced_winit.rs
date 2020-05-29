use std::{error::Error, marker::PhantomData};

use crossbeam_channel::Receiver;
use iced_graphics::window;
use iced_winit::{
    conversion, mouse,
    program::{Program, State},
    winit::{self, event::Event, event_loop::ControlFlow, window::Window},
    Application, Clipboard, Debug, Executor, Mode, Proxy, Runtime, Settings, Size, Viewport,
};
use nginee_event_loop::{EventHandler, EventHandlingOutcome};

/// The `IcedWinit` context.
#[allow(missing_debug_implementations)]
pub struct IcedWinit<A, Exc, C, Message, SwapChain, E>
where
    A: Application + 'static,
    Exc: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer, SwapChain = SwapChain> + 'static,
    Message: std::fmt::Debug + Send + 'static,
{
    runtime: Runtime<Exc, Proxy<Message>, Message>,
    title: String,
    mode: Mode,
    debug: Debug,
    compositor: C,
    renderer: A::Renderer,
    surface: C::Surface,
    clipboard: Option<Clipboard>,
    mouse_interaction: mouse::Interaction,
    modifiers: winit::event::ModifiersState,
    viewport: Viewport,
    resized: bool,
    swap_chain: SwapChain,
    state: State<A>,
    window: Window,
    marker: PhantomData<E>,
}

impl<A, Exc, C, Message, SwapChain, E> IcedWinit<A, Exc, C, Message, SwapChain, E>
where
    A: Application + Program<Message = Message> + 'static,
    Exc: Executor + 'static,
    C: window::Compositor<Renderer = A::Renderer, SwapChain = SwapChain> + 'static,
    Message: std::fmt::Debug + Send + 'static,
    SwapChain: 'static,
    E: Error + Send + 'static,
{
    /// Initializes and returns an `IcedWinit` application context.
    pub fn init(
        event_loop: &mut winit::event_loop::EventLoop<A::Message>,
        settings: Settings<A::Flags>,
        compositor_settings: C::Settings,
    ) -> Self {
        let mut debug = Debug::new();
        debug.startup_started();

        let mut runtime = {
            let executor = Exc::new().expect("Create executor");
            let proxy = Proxy::new(event_loop.create_proxy());

            Runtime::new(executor, proxy)
        };

        let flags = settings.flags;
        let (application, init_command) = runtime.enter(|| A::new(flags));
        runtime.spawn(init_command);

        let subscription = application.subscription();
        runtime.track(subscription);

        let title = application.title();
        let mode = application.mode();

        let window = settings
            .window
            .into_builder(&title, mode, event_loop.primary_monitor())
            .build(&event_loop)
            .expect("Open window");

        let clipboard = Clipboard::new(&window);
        let mouse_interaction = mouse::Interaction::default();
        let modifiers = winit::event::ModifiersState::default();

        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        );
        let resized = false;

        let (mut compositor, mut renderer) = C::new(compositor_settings);

        let surface = compositor.create_surface(&window);

        let swap_chain =
            compositor.create_swap_chain(&surface, physical_size.width, physical_size.height);

        let state = State::new(
            application,
            viewport.logical_size(),
            &mut renderer,
            &mut debug,
        );
        debug.startup_finished();

        Self {
            runtime,
            title,
            mode,
            debug,
            compositor,
            renderer,
            surface,
            clipboard,
            mouse_interaction,
            modifiers,
            viewport,
            resized,
            swap_chain,
            state,
            window,
            marker: PhantomData,
        }
    }

    fn handle_winit_event(
        &mut self,
        event: winit::event::Event<A::Message>,
    ) -> Result<EventHandlingOutcome, E> {
        let IcedWinit {
            runtime,
            title,
            mode,
            debug,
            compositor,
            renderer,
            surface,
            clipboard,
            mouse_interaction,
            modifiers,
            viewport,
            resized,
            swap_chain,
            state,
            window,
            marker: _,
        } = self;

        match event {
            Event::MainEventsCleared => {
                let command = runtime.enter(|| {
                    state.update(
                        clipboard.as_ref().map(|c| c as _),
                        viewport.logical_size(),
                        renderer,
                        debug,
                    )
                });

                // If the application was updated
                if let Some(command) = command {
                    runtime.spawn(command);

                    let program = state.program();

                    // Update subscriptions
                    let subscription = program.subscription();
                    runtime.track(subscription);

                    // Update window title
                    let new_title = program.title();

                    if *title != new_title {
                        window.set_title(&new_title);

                        *title = new_title;
                    }

                    // Update window mode
                    let new_mode = program.mode();

                    if *mode != new_mode {
                        window.set_fullscreen(conversion::fullscreen(
                            window.current_monitor(),
                            new_mode,
                        ));

                        *mode = new_mode;
                    }
                }

                window.request_redraw();
            }
            Event::UserEvent(message) => {
                state.queue_message(message);
            }
            Event::RedrawRequested(_) => {
                debug.render_started();

                if *resized {
                    let physical_size = viewport.physical_size();

                    *swap_chain = compositor.create_swap_chain(
                        &surface,
                        physical_size.width,
                        physical_size.height,
                    );

                    *resized = false;
                }

                let new_mouse_interaction = compositor.draw(
                    renderer,
                    swap_chain,
                    &viewport,
                    state.primitive(),
                    &debug.overlay(),
                );

                debug.render_finished();

                if *mouse_interaction != new_mouse_interaction {
                    window.set_cursor_icon(conversion::mouse_interaction(new_mouse_interaction));

                    *mouse_interaction = new_mouse_interaction;
                }

                // TODO: Handle animations!
                // Maybe we can use `ControlFlow::WaitUntil` for this.
            }
            Event::WindowEvent {
                event: window_event,
                ..
            } => {
                let mut control_flow = ControlFlow::Poll;
                let control_flow = &mut control_flow;
                iced_winit::application::handle_window_event(
                    &window_event,
                    &window,
                    control_flow,
                    modifiers,
                    viewport,
                    resized,
                    debug,
                );

                if let Some(event) =
                    conversion::window_event(&window_event, viewport.scale_factor(), *modifiers)
                {
                    state.queue_event(event.clone());
                    runtime.broadcast(event);
                }

                if *control_flow == ControlFlow::Exit {
                    return Ok(EventHandlingOutcome::Exit);
                }
            }
            _ => {
                // *control_flow = ControlFlow::Wait;
            }
        }

        Ok(EventHandlingOutcome::Continue)
    }

    /// Returns an event handler that runs `IcedWinit` logic.
    ///
    /// # Parameters
    ///
    /// * `event_receiver`: winit event receiver that the IcedWinit application
    ///   responds to.
    pub fn into_event_handler(
        self,
        event_receiver: Receiver<Event<'static, A::Message>>,
    ) -> EventHandler<E, Self> {
        EventHandler::new_with_context(self, move |mut iced_winit| {
            let events = event_receiver.try_iter().collect::<Vec<_>>();
            async move {
                let outcome = events.into_iter().try_fold(
                    EventHandlingOutcome::Continue,
                    |outcome_cumulative, event| {
                        let outcome = iced_winit.handle_winit_event(event);
                        EventHandlingOutcome::merge(Ok(outcome_cumulative), outcome)
                    },
                );

                (iced_winit, outcome)
            }
        })
    }
}
