use std::time::Instant;

use slacktor::{
    actor::{Handler, Message},
    Slacktor,
};

struct TestMessage(pub u64);

impl Message for TestMessage {
    type Result = u64;
}

struct TestActor(pub u64);

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

    let a = system.get::<TestActor>(actor_id).unwrap();

    // Time 1 billion messages, appending each to a vector and doing some math to prevent the
    // code being completely optimzied away. (Which is impressive that the compiler can do.)
    // On my machine, I am averaging 600-700 million messages/second.
    let num_messages = 1_000_000_000;
    let mut out = Vec::with_capacity(num_messages);
    let start = Instant::now();
    for i in 0..num_messages {
        let v = a.send(TestMessage(i as u64));
        out.push(v);
    }
    let elapsed = start.elapsed();
    println!(
        "{:.2} messages/sec",
        num_messages as f64 / elapsed.as_secs_f64()
    );

    // With no storing it to the vector, rustc can optimize everything away
    // This doesn't happen with other actor frameworks.
    // Sadly, this optimization does not occur when using tokio, async std, or smol.
    let num_messages: u64 = 1_000_000_000;
    let start = Instant::now();
    for i in 0..num_messages {
        let _v = a.send(TestMessage(i as u64));
    }
    let elapsed = start.elapsed();
    println!("{:.2} m/s", num_messages as f64 / elapsed.as_secs_f64());
}
