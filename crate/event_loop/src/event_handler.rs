use core::{
    fmt::{self, Debug},
    future::Future,
    marker::PhantomData,
    pin::Pin,
};
use std::error::Error;

use crate::EventHandlerResult;

/// Wrapper type for event handler logic.
pub struct EventHandler<E> {
    /// Event handler logic.
    fn_handler_logic: Box<dyn EventHandlerLogic<E>>,
}

impl<E> EventHandler<E>
where
    E: Error,
{
    /// Returns a new `EventHandler`.
    ///
    /// # Parameters
    ///
    /// * `handler_logic`: The logic to run.
    pub fn new<FnFut, Fut>(fn_handler_logic: FnFut) -> Self
    where
        Fut: Future<Output = EventHandlerResult<E>> + Send + 'static,
        FnFut: FnMut() -> Fut + 'static,
    {
        let fn_handler_logic = {
            let event_handler_logic = EventHandlerLogicBasic {
                fn_handler_logic,
                marker: PhantomData,
            };
            Box::new(event_handler_logic)
        };

        Self { fn_handler_logic }
    }

    /// Runs the event handler logic.
    pub async fn run(&mut self) -> EventHandlerResult<E> {
        self.fn_handler_logic.run().await
    }
}

#[cfg_attr(tarpaulin, skip)]
impl<E> Debug for EventHandler<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("EventHandler")
            .field("fn_handler_logic", &"..")
            .finish()
    }
}

trait EventHandlerLogic<E> {
    fn run(&mut self) -> Pin<Box<dyn Future<Output = EventHandlerResult<E>> + Send + 'static>>;
}

struct EventHandlerLogicBasic<FnFut, Fut> {
    fn_handler_logic: FnFut,
    marker: PhantomData<Fut>,
}

impl<E, FnFut, Fut> EventHandlerLogic<E> for EventHandlerLogicBasic<FnFut, Fut>
where
    Fut: Future<Output = EventHandlerResult<E>> + Send + 'static,
    FnFut: FnMut() -> Fut,
{
    fn run(&mut self) -> Pin<Box<dyn Future<Output = EventHandlerResult<E>> + Send + 'static>> {
        Box::pin((self.fn_handler_logic)())
    }
}
