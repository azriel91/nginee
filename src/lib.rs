//! `nginee` is a toy `async` game engine.

#![deny(
    anonymous_parameters,
    arithmetic_overflow,
    array_into_iter,
    missing_debug_implementations,
    missing_docs
)]

#[cfg(feature = "event_loop")]
pub use nginee_event_loop as event_loop;
#[cfg(feature = "iced")]
pub use nginee_iced as iced;
