use std::time::Duration;

use async_std::task;
use iced::{Application, Instance};
use iced_wgpu::Settings as CompositorSettings;
use iced_winit::{settings::Window, winit::event_loop::EventLoop as WinitEventLoop, Settings};
use instant::Instant;
use nginee::{
    event_loop::{EventHandler, EventHandlingOutcome, EventLoop, RateLimit},
    iced::{channel, IcedWinit},
};

use crate::counter::Counter;

pub use crate::error::Error;

mod counter;
mod error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
fn log(s: &str) {
    println!("{}", s);
}

pub fn timeout(timeout: Duration) -> EventHandler<Error> {
    let instant_start = Instant::now();
    EventHandler::<Error>::new(move || async move {
        let duration_remaining = timeout - instant_start.elapsed();
        if duration_remaining >= Duration::from_secs(1) {
            log(&format!(
                "{} seconds remaining",
                duration_remaining.as_secs()
            ));
            Ok(EventHandlingOutcome::Continue)
        } else {
            Ok(EventHandlingOutcome::Exit)
        }
    })
    .with_rate_limit(RateLimit::interval(Duration::from_secs(1)))
}

pub fn iced_window(
    winit_event_loop: &mut WinitEventLoop<<Counter as Application>::Message>,
) -> EventHandler<Error> {
    let iced_winit = IcedWinit::<
        Instance<Counter>,
        <Counter as Application>::Executor,
        iced_wgpu::window::Compositor,
        _,
        _,
        _,
    >::init(
        winit_event_loop,
        Settings {
            window: Window {
                size: (400, 300),
                ..Default::default()
            },
            ..Default::default()
        },
        CompositorSettings::default(),
    );

    let (_tx, rx) = channel::unbounded();

    iced_winit.into_event_handler(rx)
}

#[cfg(not(target_arch = "wasm32"))]
type ReturnValue = Result<(), Error>;

#[cfg(target_arch = "wasm32")]
type ReturnValue = ();

/// Runs the application.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run() -> ReturnValue {
    let mut event_loop = EventLoop::new_with_event(vec![timeout(Duration::from_secs(4))]);
    let iced_window = iced_window(&mut event_loop);
    event_loop.with_event_handler(iced_window);

    task::block_on(event_loop.run())
}
