use event_loop_console::run;

#[cfg(all(not(target_arch = "wasm32"), not(feature = "window")))]
type ReturnValue = Result<(), event_loop_console::Error>;

#[cfg(any(target_arch = "wasm32", feature = "window"))]
type ReturnValue = ();

fn main() -> ReturnValue {
    run()
}
