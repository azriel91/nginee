#[cfg(test)]
mod tests {
    #[cfg(feature = "rate_limit")]
    use std::time::Duration;

    use crossbeam::channel::{self, SendError, Sender};

    #[cfg(feature = "rate_limit")]
    use nginee::event_loop::RateLimit;
    use nginee::event_loop::{EventHandler, EventHandlingOutcome, EventLoop};

    #[test]
    fn run_runs_event_handlers_until_exit_is_signalled() -> Result<(), SendError<()>> {
        let (tx, rx) = channel::bounded(10);
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
        let (tx, _rx) = channel::bounded(10);
        let event_handler_send = sender(tx);
        let event_handler_countdown = countdown(3);

        let event_loop =
            EventLoop::new(vec![event_handler_send, errorer(), event_handler_countdown]);

        assert_eq!(SendError(()), smol::run(event_loop.run()).unwrap_err());

        Ok(())
    }

    #[cfg(feature = "rate_limit")]
    #[test]
    fn event_handlers_are_rate_limited_independently() -> Result<(), SendError<()>> {
        let (tx0, rx0) = channel::unbounded();
        let event_handler_send_0 = sender(tx0);

        let (tx1, rx1) = channel::bounded(10);
        let event_handler_send_1 =
            sender(tx1).with_rate_limit(RateLimit::Interval(Duration::from_millis(2)));
        let event_handler_countdown =
            countdown(3).with_rate_limit(RateLimit::Interval(Duration::from_millis(3)));

        let event_loop = EventLoop::new(vec![
            event_handler_countdown,
            event_handler_send_1,
            event_handler_send_0,
        ]);

        smol::run(event_loop.run())?;

        let count_0 = rx0.try_iter().collect::<Vec<()>>().len();
        let count_1 = rx1.try_iter().collect::<Vec<()>>().len();

        assert!(count_0 >= 8);
        assert!(count_1 >= 4);
        assert!(count_1 <= 6);

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
