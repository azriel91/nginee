#[cfg(test)]
mod tests {
    use nginee::event_loop::EventLoop;

    #[test]
    fn run_once_runs_event_handler_once() {
        let mut count = 0;
        let event_handler = async {
            count = count + 1;
        };

        smol::run(EventLoop::run_once(event_handler));

        assert_eq!(1, count);
    }
}
