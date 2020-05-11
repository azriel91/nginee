use nginee::event_loop::{EventHandler, EventHandlingOutcome, EventLoop};

use crate::error::Error;

fn countdown(mut count: u32) -> EventHandler<Error> {
    EventHandler::<Error>::new(move || {
        count -= 1;
        async move {
            println!("{}", count);

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
