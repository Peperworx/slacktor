use std::sync::Arc;

use actor::{Actor, ActorHandle, ActorRef};



pub mod actor;

pub struct Slacktor {
    /// The underying slab that stores actors.
    slab: slab::Slab<Arc<dyn ActorRef>>,
}

impl Slacktor {
    /// # [`Slacktor::new`]
    /// Creates a new [`Slacktor`] instance
    pub const fn new() -> Self {
        Self {
            slab: slab::Slab::new(),
        }
    }

    /// # [`Slacktor::spawn`]
    /// Create a new actor and return it's id.
    pub fn spawn<A: Actor>(&mut self, actor: A) -> usize {
        self.slab.insert(Arc::new(ActorHandle::new(actor)))
    }

    /// # [`Slacktor::get`]
    /// Get an actor ref given its id.
    /// Return's [`None`] if the given actor does not exist.
    pub fn get<A: Actor>(&self, id: usize) -> Option<&ActorHandle<A>> {
        self.slab.get(id)
            .and_then(|actor| actor.as_any().downcast_ref())
    }
}