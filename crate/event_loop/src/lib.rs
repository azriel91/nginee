//! Provides an asynchronous event loop.

#![deny(
    anonymous_parameters,
    arithmetic_overflow,
    array_into_iter,
    missing_debug_implementations,
    missing_docs
)]

mod event_loop;

pub use crate::event_loop::EventLoop;
