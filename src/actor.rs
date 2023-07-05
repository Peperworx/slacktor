use std::error::Error;



/// # Actor
/// This trait should be implemented for all actors.
pub trait Actor: Send + Sync + 'static {
    
}

/// # MessageHandler
/// Implement this trait for all actors that wish to recieve the message T.
pub trait HandleMessage<T: Message>: Actor {
    fn handle_message(&mut self, message: T) -> Result<T::Response, Box<dyn Error>>;
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