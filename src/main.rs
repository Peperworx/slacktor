use slacktor::actor::{Actor, ActorHandle, Handle, HandleMessage, Message};
use std::{error::Error, sync::Arc};

struct TestMessage;
impl Message for TestMessage {
    type Response = ();
}

struct TestMessage2;
impl Message for TestMessage2 {
    type Response = ();
}

struct TestActor;

impl Actor for TestActor {}

impl HandleMessage<TestMessage> for TestActor {
    fn handle_message(&self, _: TestMessage) -> Result<(), Box<dyn Error>> {
        println!("Handled");
        Ok(())
    }
}

impl HandleMessage<TestMessage2> for TestActor {
    fn handle_message(&self, _: TestMessage2) -> Result<(), Box<dyn Error>> {
        println!("Handled2");
        Ok(())
    }
}

fn main() {
    let mut actor_slab = slab::Slab::<Box<dyn ActorHandle>>::new();

    // Add an actor
    let actor_id = actor_slab.insert(Box::new(Handle(Arc::new(TestActor))));

    // Get the actor
    let actor = actor_slab
        .get(actor_id)
        .and_then(|v| v.as_any().downcast_ref::<Handle<TestActor>>())
        .cloned()
        .unwrap().0;

    // Get a second handle to the actor
    let actor2 = actor_slab
        .get(actor_id)
        .and_then(|v| v.as_any().downcast_ref::<Handle<TestActor>>())
        .cloned()
        .unwrap().0;

    // Call a message on each
    actor.handle_message(TestMessage).unwrap();
    actor2.handle_message(TestMessage2).unwrap();
    
}
