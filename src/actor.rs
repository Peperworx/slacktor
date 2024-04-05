use core::any::Any;
use alloc::sync::Arc;

/// # [`Actor`]`
/// Trait implemented by actors
pub trait Actor: Send + Sync + 'static {
    #[cfg(feature = "async")]
    fn destroy(&self) -> impl core::future::Future<Output = ()> + Send {
        async {}
    }

    #[cfg(not(feature = "async"))]
    fn destroy(&self) {}
}

/// # [`Handler`]`
/// Implement this trait for all actors that wish to recieve the message T.
pub trait Handler<T: Message>: Actor {
    #[cfg(feature = "async")]
    fn handle_message(&self, message: T) -> impl core::future::Future<Output = T::Result> + Send;

    #[cfg(not(feature = "async"))]
    fn handle_message(&self, message: T) -> T::Result;
}

/// # [`Message`]
/// A message sent to an actor
pub trait Message: Send + Sync + 'static {
    /// # [`Message::Result`]
    /// The value that the a message handler returns to the caller
    type Result: Send + Sync + 'static;
}

/// # [`MessageSender`]
/// Implemented by [`ActorHandle`]s. Allows the actual type of the actor to be
/// abstracted away, leaving only the type of message that can be sent.
pub trait MessageSender<M: Message> {
    /// Send a message to the actor and wait for a response
    #[cfg(feature = "async")]
    async fn send(&self, message: M) -> M::Result;

    /// Send a message to the actor and wait for a response
    #[cfg(not(feature = "async"))]
    fn send(&self, message: M) -> M::Result;
}

/// # [`ActorHandle`]
/// Provides functions to send messages to a given actor.
#[repr(transparent)]
pub struct ActorHandle<A: Actor>(Arc<A>);

impl<A: Actor> ActorHandle<A> {
    /// # [`ActorHandle::new`]
    /// Creates a new actor handle wrapping the given actor.
    pub(crate) fn new(actor: A) -> Self {
        Self(Arc::new(actor))
    }

    /// # [`ActorHandle::send`]
    /// Send a message to the actor and wait for a response
    #[cfg(feature = "async")]
    pub async fn send<M: Message>(&self, message: M) -> M::Result
    where A: Handler<M> {
        self.0.handle_message(message).await
    }

    /// # [`ActorHandle::send`]
    /// Send a message to the actor and wait for a response
    #[cfg(not(feature = "async"))]
    pub fn send<M: Message>(&self, message: M) -> M::Result
    where A: Handler<M> {
        self.0.handle_message(message)
    }
    

    #[cfg(feature = "async")]
    pub async fn kill(&self) {
        self.0.destroy().await;
    }

    #[cfg(not(feature = "async"))]
    pub fn kill(&self) {
        self.0.destroy();
    }
}

impl<M: Message, A: Actor + Handler<M>> MessageSender<M> for ActorHandle<A> {
    /// Send a message to the actor and wait for a response
    #[cfg(feature = "async")]
    async fn send(&self, message: M) -> M::Result {
        self.0.handle_message(message).await
    }

    /// Send a message to the actor and wait for a response
    #[cfg(not(feature = "async"))]
    fn send(&self, message: M) -> M::Result {
        self.0.handle_message(message)
    }
}

impl<A: Actor> ActorRef for ActorHandle<A> {
    #[inline(always)]
    fn as_any(&self) -> &dyn Any {
        self
    }

    #[cfg(not(feature = "async"))]
    fn kill(&self) {
        self.0.destroy();
    }
}

// `A` may not impl Clone, but Arc does.
impl<A: Actor> Clone for ActorHandle<A> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// # [`ActorRef`]
/// Internal trait used for representing actors when stored in the slab.
pub(crate) trait ActorRef: Send + Sync + 'static {
    fn as_any(&self) -> &dyn Any;

    #[cfg(not(feature = "async"))]
    fn kill(&self);
}