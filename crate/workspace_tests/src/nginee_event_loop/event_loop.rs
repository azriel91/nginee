#[cfg(test)]
mod tests {
    use crossbeam::channel;

    use nginee::event_loop::{EventHandler, EventLoop};

    #[test]
    fn run_once_runs_event_handler_once() {
        let (tx, rx) = channel::unbounded();
        let event_handler = EventHandler::new(async move {
            tx.send(()).unwrap();
        });

        let mut event_loop = EventLoop::new(vec![event_handler]);

        smol::run(event_loop.run_once());

        let count = rx.try_iter().collect::<Vec<()>>().len();
        assert_eq!(1, count);
    }
}
