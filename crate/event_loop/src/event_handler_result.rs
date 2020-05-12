use crate::EventHandlingOutcome;

/// Result of running an event handler.
pub type EventHandlerResult<E> = Result<EventHandlingOutcome, E>;
