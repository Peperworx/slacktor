use std::time::Instant;

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use slacktor::{
    actor::{Actor, Handler, Message},
    Slacktor,
};

struct TestMessage(pub u8);

impl Message for TestMessage {
    type Result = u8;
}

struct TestActor(pub u8);

impl Actor for TestActor {
    fn destroy(&self) {
        println!("destroying");
    }
}

impl Handler<TestMessage> for TestActor {
    fn handle_message(&self, m: TestMessage) -> u8 {
        m.0 ^ self.0
    }
}

fn main() {
    // Create a slacktor instance
    let mut system = Slacktor::new();

    // Create a new actor
    let actor_id = system.spawn(TestActor(rand::random::<u8>()));

    // Get a reference to the actor
    let a = system.get::<TestActor>(actor_id).unwrap();

    // Time 1 billion messages, appending each to a vector and doing some math to prevent the
    // code being completely optimzied away.
    let num_messages = 1_000_000_000;
    
    let start = Instant::now();
    let _v = (0..num_messages).into_par_iter().map(|i| {
        // Send the message
        a.send(TestMessage(i as u8))
    }).collect::<Vec<_>>();
    let elapsed = start.elapsed();
    println!(
        "{:.2} messages/sec",
        num_messages as f64 / elapsed.as_secs_f64()
    );

    system.kill(actor_id);
}