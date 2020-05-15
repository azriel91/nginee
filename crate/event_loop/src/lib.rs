//! Provides an asynchronous event loop.

#![deny(
    anonymous_parameters,
    arithmetic_overflow,
    array_into_iter,
    missing_debug_implementations,
    missing_docs
)]

#[doc(hidden)]
macro_rules! cfg_rate_limit {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "rate_limit")]
            #[cfg_attr(feature = "docs", doc(cfg(rate_limit)))]
            $item
        )*
    }
}

cfg_rate_limit! {
    mod error;
    mod rate_limit;

    pub use crate::error::Error;
    pub use crate::rate_limit::RateLimit;
}

mod event_handler;
mod event_handler_result;
mod event_handling_outcome;
mod event_loop;

pub use crate::{
    event_handler::EventHandler, event_handler_result::EventHandlerResult,
    event_handling_outcome::EventHandlingOutcome, event_loop::EventLoop,
};
