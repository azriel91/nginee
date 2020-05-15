#[cfg(target_arch = "wasm32")]
mod async_std;

#[cfg(target_arch = "wasm32")]
pub use self::async_std::RateLimit;

#[cfg(not(target_arch = "wasm32"))]
mod governor;

#[cfg(not(target_arch = "wasm32"))]
pub use self::governor::RateLimit;
