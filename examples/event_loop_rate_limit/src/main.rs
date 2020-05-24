use event_loop_rate_limit::run;

#[cfg(all(not(target_arch = "wasm32"), not(feature = "window")))]
type ReturnValue = Result<(), event_loop_rate_limit::Error>;

#[cfg(any(target_arch = "wasm32", feature = "window"))]
type ReturnValue = ();

fn main() -> ReturnValue {
    run()
}
