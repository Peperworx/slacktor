use std::{any::Any, error::Error, sync::Arc};



pub struct Handle<A: Actor>(pub Arc<A>);

impl<A: Actor> ActorHandle for Handle<A> {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<A: Actor> Clone for Handle<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait ActorHandle {
    fn as_any(&self) -> &dyn Any;
}

/// # Actor
/// This trait should be implemented for all actors.
pub trait Actor: Send + Sync + 'static {
    
}

/// # MessageHandler
/// Implement this trait for all actors that wish to recieve the message T.
pub trait HandleMessage<T: Message>: Actor {
    fn handle_message(&self, message: T) -> Result<T::Response, Box<dyn Error>>;
}

/// # Message
/// A message sent to an actor
pub trait Message: Send + Sync + 'static {
    type Response: Send + Sync + 'static;

    fn handle_message(self, handler: &mut dyn HandleMessage<Self>) -> Result<Self::Response, Box<dyn Error>>
        where Self: Sized {
        handler.handle_message(self)
    }
}