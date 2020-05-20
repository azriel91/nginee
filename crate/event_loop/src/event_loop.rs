#[cfg(not(feature = "window"))]
pub use self::headless::EventLoop;

#[cfg(feature = "window")]
pub use self::window::EventLoop;

#[cfg(feature = "rate_limit")]
mod common;
#[cfg(not(feature = "winit"))]
mod headless;
#[cfg(feature = "window")]
mod window;
