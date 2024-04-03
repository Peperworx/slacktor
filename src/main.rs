use slacktor::actor::{Actor, ActorHandle, Handle, HandleMessage, Message};
use std::{error::Error, sync::Arc, time::Instant};

struct TestMessage(pub u64);
impl Message for TestMessage {
    type Response = u64;
}

struct TestMessage2;
impl Message for TestMessage2 {
    type Response = ();
}

struct TestActor(pub u64);

impl Actor for TestActor {}

impl HandleMessage<TestMessage> for TestActor {
    fn handle_message(&self, m: TestMessage) -> Result<u64, Box<dyn Error>> {
        Ok(m.0^self.0)
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
    let actor_id = actor_slab.insert(Box::new(Handle(Arc::new(TestActor(rand::random::<u64>())))));

    // Get the actor
    let actor = actor_slab
        .get(actor_id)
        .and_then(|v| v.as_any().downcast_ref::<Handle<TestActor>>())
        .cloned()
        .unwrap().0;

    // Time 1 billion messages, appending each to a vector and doing some math to prevent the
    // code being completely optimzied away. (Which is impressive that the compiler can do.)
    // On my machine, I am averaging 600-700 million messages/second.
    let num_messages = 1_000_000_000;
    let mut out = Vec::with_capacity(num_messages);
    let start = Instant::now();
    for i in 0..num_messages {
        let a = actor.handle_message(TestMessage(i as u64)).unwrap();
        out.push(a);
    }
    let elapsed = start.elapsed();
    println!("{:.2} messages/sec", num_messages as f64/elapsed.as_secs_f64());
}
