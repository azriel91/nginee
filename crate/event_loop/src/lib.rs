//! Provides an asynchronous event loop.

#![deny(
    anonymous_parameters,
    arithmetic_overflow,
    array_into_iter,
    missing_debug_implementations,
    missing_docs
)]

mod event_handler;
mod event_handler_result;
mod event_handling_outcome;
mod event_loop;

pub use crate::{
    event_handler::EventHandler, event_handler_result::EventHandlerResult,
    event_handling_outcome::EventHandlingOutcome, event_loop::EventLoop,
};
