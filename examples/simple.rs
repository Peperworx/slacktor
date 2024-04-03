use std::time::Instant;

use slacktor::{
    actor::{Actor, Handler, Message},
    Slacktor,
};

struct TestMessage(pub u64);

impl Message for TestMessage {
    type Result = u64;
}

struct TestActor(pub u64);

impl Actor for TestActor {
    fn destroy(&self) {
        println!("destroying");
    }
}

impl Handler<TestMessage> for TestActor {
    fn handle_message(&self, m: TestMessage) -> u64 {
        m.0 ^ self.0
    }
}

fn main() {
    // Create a slacktor instance
    let mut system = Slacktor::new();

    // Create a new actor
    let actor_id = system.spawn(TestActor(rand::random::<u64>()));

    // Get a reference to the actor
    let a = ();

    // Time 1 billion messages, appending each to a vector and doing some math to prevent the
    // code being completely optimzied away.
    let num_messages = 1_000_000_000;
    let mut out = Vec::with_capacity(num_messages);
    let start = Instant::now();
    for i in 0..num_messages {
        // Send the message
        let v = system.get::<TestActor>(actor_id).unwrap().send(TestMessage(i as u64));
        out.push(v);
    }
    let elapsed = start.elapsed();
    println!(
        "{:.2} messages/sec",
        num_messages as f64 / elapsed.as_secs_f64()
    );

    system.kill(actor_id);
}