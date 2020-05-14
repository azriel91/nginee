//! Provides an asynchronous event loop.

#![deny(
    anonymous_parameters,
    arithmetic_overflow,
    array_into_iter,
    missing_debug_implementations,
    missing_docs
)]

#[cfg(feature = "rate_limit")]
mod error;
mod event_handler;
mod event_handler_result;
mod event_handling_outcome;
mod event_loop;
#[cfg(feature = "rate_limit")]
mod rate_limit;

#[cfg(feature = "rate_limit")]
pub use crate::error::Error;
#[cfg(feature = "rate_limit")]
pub use crate::rate_limit::RateLimit;
pub use crate::{
    event_handler::EventHandler, event_handler_result::EventHandlerResult,
    event_handling_outcome::EventHandlingOutcome, event_loop::EventLoop,
};
