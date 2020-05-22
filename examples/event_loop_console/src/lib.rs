use std::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use async_std::io::{self, prelude::WriteExt};
use async_std::task;
use nginee::event_loop::{EventHandler, EventHandlingOutcome, EventLoop};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub use crate::error::Error;

mod error;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(inline_js = r#"
    export function display(s) {
        let terminal_element = document.querySelector('#terminal');
        if (terminal_element != null) {
            terminal_element.innerText = s;
        } else {
            console.error("Could not find `#terminal` element in HTML document.");
        }
    }
"#)]
extern "C" {
    fn display(s: &str);
}

#[cfg(not(target_arch = "wasm32"))]
async fn display(s: &str) -> Result<(), io::Error> {
    // Move cursor up
    io::stdout().write(b"\x1b[1A").await?;
    // Clear the line
    io::stdout().write(b"\r\x1b[2K").await?;
    // Write the string
    io::stdout().write(s.as_bytes()).await?;
    io::stdout().write(b"\n").await?;
    Ok(())
}

pub fn countdown(mut count: u32) -> EventHandler<Error> {
    EventHandler::<Error>::new(move || {
        count = count.saturating_sub(1);
        async move {
            #[cfg(not(target_arch = "wasm32"))]
            display(&format!("{}", count)).await?;
            #[cfg(target_arch = "wasm32")]
            display(&format!("{}", count));

            task::sleep(Duration::from_millis(50)).await;

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
    let event_loop = EventLoop::new(vec![countdown(100)]);

    task::block_on(event_loop.run())
}
