use nginee::event_loop::{EventHandler, EventHandlingOutcome, EventLoop};

use crate::error::Error;

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

fn countdown(mut count: u32) -> EventHandler<Error> {
    EventHandler::<Error>::new(move || {
        count -= 1;
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

fn main() -> Result<(), Error> {
    let event_loop = EventLoop::new(vec![countdown(10)]);

    smol::run(event_loop.run())
}

mod error {
    use std::fmt;

    #[derive(Debug)]
    pub struct Error;

    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Error")
        }
    }

    impl std::error::Error for Error {}
}
