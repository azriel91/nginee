use core::{
    fmt::{self, Debug},
    future::Future,
    marker::PhantomData,
    pin::Pin,
};
use std::error::Error;

use crossbeam_channel::Sender;

use crate::EventHandlerResult;
#[cfg(feature = "rate_limit")]
use crate::RateLimit;

/// Wrapper type for event handler logic.
pub struct EventHandler<E> {
    /// Event handler logic.
    fn_handler_logic: Box<dyn EventHandlerLogic<E>>,
    #[cfg(feature = "rate_limit")]
    /// Rate to limit this event handler's execution.
    pub rate_limit: Option<RateLimit>,
}

impl<E> EventHandler<E>
where
    E: Error + Send + 'static,
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

        Self {
            fn_handler_logic,
            #[cfg(feature = "rate_limit")]
            rate_limit: None,
        }
    }

    /// Sets the rate limit for this event handler.
    #[cfg(feature = "rate_limit")]
    pub fn with_rate_limit(mut self, rate_limit: RateLimit) -> Self {
        self.rate_limit = Some(rate_limit);
        self
    }

    /// Runs the event handler logic.
    #[cfg(not(feature = "window"))]
    pub async fn run(&mut self) -> EventHandlerResult<E> {
        self.fn_handler_logic.handler_task().await
    }

    /// Returns a future that runs the event handler once and sends the result
    /// to a channel.
    ///
    /// # Parameters
    ///
    /// * `tx`: Sender to write the event handler result to.
    #[cfg(feature = "window")]
    pub fn handler_task(
        &mut self,
        tx: Sender<EventHandlerResult<E>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        self.fn_handler_logic.handler_task(tx)
    }
}

#[cfg_attr(tarpaulin, skip)]
impl<E> Debug for EventHandler<E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut debug_struct = f.debug_struct("EventHandler");

        debug_struct.field("fn_handler_logic", &"..");

        #[cfg(feature = "rate_limit")]
        debug_struct.field("rate_limit", &self.rate_limit);

        debug_struct.finish()
    }
}

trait EventHandlerLogic<E> {
    #[cfg(not(feature = "window"))]
    fn handler_task(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = EventHandlerResult<E>> + Send + 'static>>;

    #[cfg(feature = "window")]
    fn handler_task(
        &mut self,
        tx: Sender<EventHandlerResult<E>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>>;
}

struct EventHandlerLogicBasic<FnFut, Fut> {
    fn_handler_logic: FnFut,
    marker: PhantomData<Fut>,
}

impl<E, FnFut, Fut> EventHandlerLogic<E> for EventHandlerLogicBasic<FnFut, Fut>
where
    E: Send + 'static,
    Fut: Future<Output = EventHandlerResult<E>> + Send + 'static,
    FnFut: FnMut() -> Fut,
{
    #[cfg(not(feature = "window"))]
    fn handler_task(
        &mut self,
    ) -> Pin<Box<dyn Future<Output = EventHandlerResult<E>> + Send + 'static>> {
        Box::pin((self.fn_handler_logic)())
    }

    #[cfg(feature = "window")]
    fn handler_task(
        &mut self,
        tx: Sender<EventHandlerResult<E>>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'static>> {
        let handler_task = (self.fn_handler_logic)();
        Box::pin(async move {
            let event_handler_result = handler_task.await;

            if let Err(e) = tx.try_send(event_handler_result) {
                // TODO: error handling.
                panic!("{:?}", e);
            }
        })
    }
}
