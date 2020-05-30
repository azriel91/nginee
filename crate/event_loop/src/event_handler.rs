use core::{
    fmt::{self, Debug},
    future::Future,
    marker::PhantomData,
    pin::Pin,
};
use std::error::Error;

use crate::EventHandlerResult;
#[cfg(feature = "rate_limit")]
use crate::RateLimit;

type HandlerTask<'f, E> = Pin<Box<dyn Future<Output = EventHandlerResult<E>> + 'f>>;

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
    pub fn new<FnFut, Fut>(mut fn_handler_logic: FnFut) -> Self
    where
        Fut: Future<Output = EventHandlerResult<E>> + 'static,
        FnFut: FnMut() -> Fut + 'static,
    {
        let fn_handler_logic = Box::new(FnHandlerWrapper {
            context: Some(()),
            fn_fut: move |_: ()| {
                let fut = fn_handler_logic();
                async move { ((), fut.await) }
            },
            marker: PhantomData,
        });

        Self {
            fn_handler_logic,
            #[cfg(feature = "rate_limit")]
            rate_limit: None,
        }
    }
}

impl<E> EventHandler<E>
where
    E: Error + Send + 'static,
{
    /// Returns a new `EventHandler`.
    ///
    /// # Parameters
    ///
    /// * `context`: The context item used to construct the event handler logic.
    /// * `handler_logic`: The logic to run.
    pub fn new_with_context<Context, FnFut, Fut>(context: Context, fn_handler_logic: FnFut) -> Self
    where
        Context: 'static,
        Fut: Future<Output = (Context, EventHandlerResult<E>)> + 'static,
        FnFut: FnMut(Context) -> Fut + 'static,
    {
        let fn_handler_logic = Box::new(FnHandlerWrapper {
            context: Some(context),
            fn_fut: fn_handler_logic,
            marker: PhantomData,
        });

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
    pub async fn run(&mut self) -> EventHandlerResult<E> {
        self.fn_handler_logic.handler_task().await
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

struct FnHandlerWrapper<Context, FnFut, Fut> {
    context: Option<Context>,
    fn_fut: FnFut,
    marker: PhantomData<Fut>,
}

impl<E, Context, FnFut, Fut> FnHandlerWrapper<Context, FnFut, Fut>
where
    E: Send + 'static,
    Fut: Future<Output = (Context, EventHandlerResult<E>)> + 'static,
    FnFut: FnMut(Context) -> Fut + 'static,
{
    async fn handler_task(&mut self) -> EventHandlerResult<E> {
        if let Some(context) = self.context.take() {
            let (context, result) = (self.fn_fut)(context).await;
            self.context = Some(context);

            result
        } else {
            unreachable!()
        }
    }
}

trait EventHandlerLogic<E> {
    fn handler_task<'f>(&'f mut self) -> HandlerTask<'f, E>;
}

impl<E, Context, FnFut, Fut> EventHandlerLogic<E> for FnHandlerWrapper<Context, FnFut, Fut>
where
    E: Send + 'static,
    Fut: Future<Output = (Context, EventHandlerResult<E>)> + 'static,
    FnFut: FnMut(Context) -> Fut + 'static,
{
    fn handler_task<'f>(&'f mut self) -> HandlerTask<'f, E> {
        Box::pin(FnHandlerWrapper::handler_task(self))
    }
}
