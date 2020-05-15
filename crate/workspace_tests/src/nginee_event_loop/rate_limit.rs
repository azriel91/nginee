#[cfg(target_arch = "wasm32")]
mod async_std;

#[cfg(not(target_arch = "wasm32"))]
mod governor;
