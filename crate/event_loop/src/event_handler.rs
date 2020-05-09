use core::{
    fmt::{self, Debug},
    future::Future,
    pin::Pin,
};

/// Wrapper type for event handler logic.
pub struct EventHandler {
    /// Event handler logic.
    pub handler_logic: Pin<Box<dyn Future<Output = ()> + Send>>,
}

impl EventHandler {
    /// Returns a new `EventHandler`.
    ///
    /// # Parameters
    ///
    /// * `handler_logic`: The logic to run
    pub fn new(handler_logic: impl Future<Output = ()> + Send + 'static) -> Self {
        let handler_logic = Box::pin(handler_logic);
        Self { handler_logic }
    }

    /// Runs the event handler logic.
    pub async fn run(&mut self) {
        self.handler_logic.as_mut().await
    }
}

#[cfg_attr(tarpaulin, skip)]
impl Debug for EventHandler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EventHandler")
            .field("handler_logic", &"..")
            .finish()
    }
}
