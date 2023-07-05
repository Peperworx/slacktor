use slacktor::actor::{Actor, HandleMessage, Message};
use std::error::Error;

struct TestMessage;
impl Message for TestMessage {
    type Response = ();
}

struct TestMessage2;
impl Message for TestMessage2 {
    type Response = ();
}

struct TestActor;

impl Actor for TestActor {
    
}

impl HandleMessage<TestMessage> for TestActor {
    fn handle_message(&mut self, _: TestMessage) -> Result<(), Box<dyn Error>> {
        println!("Handled");
        Ok(())
    }
}

impl HandleMessage<TestMessage2> for TestActor {
    fn handle_message(&mut self, _: TestMessage2) -> Result<(), Box<dyn Error>> {
        println!("Handled2");
        Ok(())
    }
}

fn main() {
    let mut a = TestActor;

    let m = TestMessage2;

    m.handle_message(&mut a).unwrap();
}