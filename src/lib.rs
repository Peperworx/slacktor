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

    /// # [`Slacktor::kill`]
    /// Remove's the Slacktor instance's reference to a given actor and calls the actor's `kill` function.
    /// This will cause the actor to be destroyed after every existing handle is dropped,
    /// which may or may not happen. Generally an actor will deinitialize itself, and then respond with an error
    /// to every additional message.
    #[cfg(not(feature = "async"))]
    pub fn kill(&mut self, id: usize) {
        // If the actor does not exist, exit early
        if !self.slab.contains(id) {
            return;
        }

        // Remove the actor from the slab
        let a = self.slab.remove(id);

        // Kill it
        a.kill();
    }

    /// # [`Slacktor::kill`]
    /// Remove's the Slacktor instance's reference to a given actor and calls the actor's `kill` function.
    /// This will cause the actor to be destroyed after every existing handle is dropped,
    /// which may or may not happen. Generally an actor will deinitialize itself, and then respond with an error
    /// to every additional message. Returns [`None`] if the actor did not exist
    #[cfg(feature = "async")]
    pub async fn kill<A: Actor>(&mut self, id: usize) -> Option<()> {
        // If the actor does not exist, exit early
        if !self.slab.contains(id) {
            return None;
        }

        // Remove the actor from the slab
        let a = self.slab.remove(id);
        let a = a.as_any().downcast_ref::<ActorHandle<A>>()?;

        // Kill it
        a.kill().await;

        Some(())
    }

    /// # [`Slacktor::get`]
    /// Get an actor handle given its id.
    /// Return's [`None`] if the given actor does not exist.
    pub fn get<A: Actor>(&self, id: usize) -> Option<&ActorHandle<A>> {
        self.slab.get(id)
            .and_then(|actor| actor.as_any().downcast_ref())
    }
}