#[cfg(test)]
mod tests {
    use nginee::event_loop::EventHandlingOutcome;

    #[test]
    fn continue_is_less_than_exit() {
        assert!(EventHandlingOutcome::Continue < EventHandlingOutcome::Exit);
    }

    #[test]
    fn exit_is_greater_than_continue() {
        assert!(EventHandlingOutcome::Exit > EventHandlingOutcome::Continue);
    }

    #[test]
    fn continue_is_equal_to_continue() {
        assert!(EventHandlingOutcome::Continue == EventHandlingOutcome::Continue);
    }

    #[test]
    fn exit_is_equal_to_exit() {
        assert!(EventHandlingOutcome::Exit == EventHandlingOutcome::Exit);
    }
}
