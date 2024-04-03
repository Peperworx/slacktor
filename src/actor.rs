use std::{any::Any, sync::Arc};


/// # [`Actor`]`
/// This trait is automatically implemented for every item that is
/// `Send + Sync + 'static`, which are the only requirements for an actor.
pub trait Actor: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Actor for T {}

/// # [`Handler`]`
/// Implement this trait for all actors that wish to recieve the message T.
pub trait Handler<T: Message>: Actor {
    fn handle_message(&self, message: T) -> T::Result;
}

/// # [`Message`]`
/// A message sent to an actor
pub trait Message: Send + Sync + 'static {
    /// # [`Message::Result`]
    /// The value that the a message handler returns to the caller
    type Result: Send + Sync + 'static;
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
    pub fn send<M: Message>(&self, message: M) -> M::Result
    where A: Handler<M> {
        self.0.handle_message(message)
    }
}

impl<A: Actor> ActorRef for ActorHandle<A> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// `A` may not impl Clone, but Arc does.
impl<A: Actor> Clone for ActorHandle<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// # [`ActorRef`]
/// Internal trait used for representing actors when stored in the slab.
pub(crate) trait ActorRef {
    fn as_any(&self) -> &dyn Any;
}