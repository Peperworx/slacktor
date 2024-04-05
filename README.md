<div align="center">
<h1> Slacktor </h1>

[![crates.io](https://img.shields.io/crates/l/slacktor?style=for-the-badge)](https://crates.io/crates/slacktor)
[![crates.io](https://img.shields.io/crates/v/slacktor?style=for-the-badge)](https://crates.io/crates/slacktor)
[![docs.rs](https://img.shields.io/docsrs/slacktor?style=for-the-badge)](https://docs.rs/slacktor)

<p>Extremely fast bare-bones actor library written in Rust.</p>
</div>



## About

Slacktor is an extremely performant actor library. It supports no-std, has only one dependency, and is extremely simple and portable. In optimal conditions, it runs around 700 million messages/second to a single actor with no other dependencies, as seen in the `simple` example (which does use the `rand` crate just to force the compiler to not optimize the code away). Because Slacktor does not handle synchronization, utilizing rayon it is possible to acheive roughly 4.5 billion messages/second on an i9-13900H laptop CPU, as seen in the `parallel` example.

Slacktor has such a low overhead, that simply changing the u64s in `parallel` to u32's in `parallel_u32` provides a speedup to roughly 9 billion messages/second, and reducing them to a u8 in `parallel_u8` increases the speed to 21 billion messages/second. The example `no_slacktor` is the equivalent of `parallel_u8` without Slacktor, and is capable of 22 billion messages/second. Reducing allocations using an iterator method like `sum` instead of `collect`, I have measured up to 80 billion messages/second.

## Limitations and Disclaimers

Slacktor actors do not have what would be called "contexts" in other actor frameworks, a window to the outside world that allows them to interact with existing actors. It is up to the user to provide this, be it as a `RwLock`/`Mutex` of an `Arc` referencing the Slacktor instance, or through message passing. Slacktor is focused on providing a simple and performant core for actor based systems, with minmal dependencies.

Slacktor doesn't actually use message passing. Instead, it emulates message passing on top of raw function calls. This allows for extremely high performance, maintains the advantages of other actor frameworks, and provides users with complete control over their project structure. You can use Slacktor as much or as little as you want.

## How is Slacktor So Fast?

Slacktor doesn't try to handle any synchronization, concurrency, or message passing. Instead, Slacktor provides a simple abstraction over a slab of actors. Message passing is then emulated by calling the message handler as soon as `send` is called. This allows the compiler to essentially optimize away the entirery of Slacktor down to just a few function calls, while still maintaining the abstraction of message passing. 


## Benchmarks

On my laptop (i9-13900H, 32GB RAM), the following code outputs roughly 700,000,000 messages/second:
```rust
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
    let a = system.get::<TestActor>(actor_id).unwrap();

    // Time 1 billion messages, appending each to a vector and doing some math to prevent the
    // code being completely optimzied away.
    let num_messages = 1_000_000_000;
    let mut out = Vec::with_capacity(num_messages);
    let start = Instant::now();
    for i in 0..num_messages {
        // Send the message
        let v = a.send(TestMessage(i as u64));
        out.push(v);
    }
    let elapsed = start.elapsed();
    println!(
        "{:.2} messages/sec",
        num_messages as f64 / elapsed.as_secs_f64()
    );

    system.kill(actor_id);
}
```

Moving the `system.get` call into the loop drops it to roughly 400,000,000 messages/sec:

```rust
// Create a slacktor instance
let mut system = Slacktor::new();

// Create a new actor
let actor_id = system.spawn(TestActor(rand::random::<u64>()));

// Time 1 billion messages, appending each to a vector and doing some math to prevent the
// code being completely optimzied away.
let num_messages = 1_000_000_000;
let mut out = Vec::with_capacity(num_messages);
let start = Instant::now();

for i in 0..num_messages {
    // Retrieve the actor from the system and send a message
    let v = system.get::<TestActor>(actor_id).unwrap().send(TestMessage(i as u64));
    out.push(v);
}

let elapsed = start.elapsed();
println!(
    "{:.2} messages/sec",
    num_messages as f64 / elapsed.as_secs_f64()
);

system.kill(actor_id);
```

If we remove pushing the values to the vector (and retrieve the actor reference outside of the loop), the rust compiler is able to completely optimize away the loop, and the code finishes executing in 100ns:
```rust
// Create a slacktor instance
let mut system = Slacktor::new();

// Create a new actor
let actor_id = system.spawn(TestActor(rand::random::<u64>()));

// Get a reference to the actor
let a = system.get::<TestActor>(actor_id).unwrap();

// Time 1 billion messages, appending each to a vector and doing some math to prevent the
// code being completely optimzied away.
let num_messages = 1_000_000_000;
let start = Instant::now();

for i in 0..num_messages {
    // Send the message
    a.send(TestMessage(i as u64));
}

let elapsed = start.elapsed();
println!(
    "{:?}",
    elapsed
);

system.kill(actor_id);
```

Retrieving the actor reference inside of the loop in this case gives us roughly 600,000,000 messages/second.


The following equivalent code for the Actix framework can handle roughly 400,000 messages/second, and
does not allow the Rust compiler to optimize away second loop. I have reduced the number of messages to 1 million,
as 1 billion is too much for Actix to handle in a reasonable timeframe.

```rust
use std::time::Instant;

use actix::prelude::*;

#[derive(Message)]
#[rtype(u64)]
struct TestMessage(pub u64);

// Actor definition
struct TestActor(pub u64);

impl Actor for TestActor {
    type Context = Context<Self>;
}

// now we need to implement `Handler` on `Calculator` for the `Sum` message.
impl Handler<TestMessage> for TestActor {
    type Result = u64; // <- Message response type

    fn handle(&mut self, msg: TestMessage, _ctx: &mut Context<Self>) -> Self::Result {
        msg.0 ^ self.0
    }
}

#[actix::main]
async fn main() {
    let actor = TestActor(rand::random::<u64>()).start();

    let num_messages = 1_000_000;
    let mut out = Vec::with_capacity(num_messages);
    let start = Instant::now();
    
    for i in 0..num_messages {
        let a = actor.send(TestMessage(i as u64)).await.unwrap();
        out.push(a);
    }

    let elapsed = start.elapsed();
    println!("{:.2} messages/sec", num_messages as f64/elapsed.as_secs_f64());

    // Actix won't optimize away
    let num_messages = 1_000_000;
    let start = Instant::now();

    for i in 0..num_messages {
        let _a = actor.send(TestMessage(i as u64)).await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("{:.2} messages/sec", num_messages as f64/elapsed.as_secs_f64());
}
```


All of these tests were run with `cargo --release`, Cargo version `1.75.0` and rustc version `1.75.0` with lto enabled (to minimal effect).


It is safe to say that Slacktor introduces almost no overhead to any projects that use it.

Additionally, Slacktor is entirely parallelizable, so the following code utilizing Rayon is capable of acheiving roughly 4.5 billion messages per second:
```rust
// Create a slacktor instance
let mut system = Slacktor::new();

// Create a new actor
let actor_id = system.spawn(TestActor(rand::random::<u64>()));

// Get a reference to the actor
let a = system.get::<TestActor>(actor_id).unwrap();

// Time 1 billion messages, appending each to a vector and doing some math to prevent the
// code being completely optimzied away.
let num_messages = 1_000_000_000;
let start = Instant::now();

let _v = (0..num_messages).into_par_iter().map(|i| {
    // Send the message
    a.send(TestMessage(i as u64))
}).collect::<Vec<_>>();

let elapsed = start.elapsed();

println!(
    "{:.2} messages/sec",
    num_messages as f64 / elapsed.as_secs_f64()
);

system.kill(actor_id);
```

Retrieving the actor reference inside of the loop leads to a lower speed of roughly 3 billion messages/second. The `parallel_u8` example is capable of acheiving roughly 21 billion messages per second.