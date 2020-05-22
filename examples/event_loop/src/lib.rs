use async_std::task;
use nginee::event_loop::{EventHandler, EventHandlingOutcome, EventLoop};

pub use crate::error::Error;

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

pub fn countdown(mut count: u32) -> EventHandler<Error> {
    EventHandler::<Error>::new(move || {
        count = count.saturating_sub(1);
        async move {
            log(&format!("{}", count));

            if count > 0 {
                Ok(EventHandlingOutcome::Continue)
            } else {
                Ok(EventHandlingOutcome::Exit)
            }
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
type ReturnValue = Result<(), Error>;

#[cfg(target_arch = "wasm32")]
type ReturnValue = ();

/// Runs the application.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub fn run() -> ReturnValue {
    let event_loop = EventLoop::new(vec![countdown(10)]);

    task::block_on(event_loop.run())
}
