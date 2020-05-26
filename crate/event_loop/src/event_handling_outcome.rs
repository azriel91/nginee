use core::cmp::Ordering;

/// Indicates what to do after running the event handler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventHandlingOutcome {
    /// Continue running the event loop.
    Continue,
    /// End the event loop execution.
    Exit,
}

impl PartialOrd for EventHandlingOutcome {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for EventHandlingOutcome {
    fn cmp(&self, other: &Self) -> Ordering {
        match (*self, *other) {
            (Self::Continue, Self::Exit) => Ordering::Less,
            (Self::Exit, Self::Continue) => Ordering::Greater,
            (Self::Continue, Self::Continue) | (Self::Exit, Self::Exit) => Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::EventHandlingOutcome;

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
