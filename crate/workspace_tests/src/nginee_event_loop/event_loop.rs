#[cfg(test)]
mod tests {
    use crossbeam::channel::{self, SendError, Sender};

    use nginee::event_loop::{EventHandler, EventHandlingOutcome, EventLoop};

    #[test]
    fn run_runs_event_handlers_until_exit_is_signalled() -> Result<(), SendError<()>> {
        let (tx, rx) = channel::unbounded();
        let event_handler_send = sender(tx);
        let event_handler_countdown = countdown(3);

        let event_loop = EventLoop::new(vec![event_handler_send, event_handler_countdown]);

        smol::run(event_loop.run())?;

        let count = rx.try_iter().collect::<Vec<()>>().len();
        assert_eq!(3, count);

        Ok(())
    }

    #[test]
    fn run_returns_on_first_error() -> Result<(), SendError<()>> {
        let (tx, _rx) = channel::unbounded();
        let event_handler_send = sender(tx);
        let event_handler_countdown = countdown(3);

        let event_loop =
            EventLoop::new(vec![event_handler_send, errorer(), event_handler_countdown]);

        assert_eq!(SendError(()), smol::run(event_loop.run()).unwrap_err());

        Ok(())
    }

    fn sender(tx: Sender<()>) -> EventHandler<SendError<()>> {
        EventHandler::<SendError<()>>::new(move || {
            let tx = tx.clone();
            async move {
                tx.send(())?;

                Ok(EventHandlingOutcome::Continue)
            }
        })
    }

    fn countdown(mut count: u32) -> EventHandler<SendError<()>> {
        EventHandler::<SendError<()>>::new(move || {
            count -= 1;
            async move {
                if count > 0 {
                    Ok(EventHandlingOutcome::Continue)
                } else {
                    Ok(EventHandlingOutcome::Exit)
                }
            }
        })
    }

    fn errorer() -> EventHandler<SendError<()>> {
        EventHandler::<SendError<()>>::new(|| async move { Err(SendError(())) })
    }
}
