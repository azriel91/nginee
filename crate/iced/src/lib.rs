//! Provides `iced` integration with `nginee_event_loop`.

#![deny(
    anonymous_parameters,
    arithmetic_overflow,
    array_into_iter,
    missing_debug_implementations,
    missing_docs
)]

pub use crossbeam_channel as channel;

pub use crate::iced_winit::IcedWinit;

mod iced_winit;
