
/// # SlabKey
/// A generational index used as the key to a [`ActorSlab`], which allows for actor slots to be reused without worrying about accessing an incorrect actor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SlabKey {
    /// The actual index of the actor
    index: usize,
    /// The generation of this index. Incremented each time a slot is freed.
    generation: usize,
}

/// # Entry
/// This is an entry in the slab-like structure that is used to store actors.
/// This works by storing two enum variants: Free and Used.
/// More details are documented on each variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Entry<T> {
    /// # Used
    /// When the entry is in use, the value of the entry is stored here.
    Used(T),
    /// # Free
    /// When an entry is freed, the value of next_free is stored here. Then, this entry's index is stored in next_free.
    /// This works almost like a combination of a linked list and a stack. When an entry needs to be allocated, the value of next_free
    /// is where it will be stored. Then, the value in the free entry is moved into next_free, essentially popping it off the stack.
    Free(usize),
}

/// # SlabEntry
/// This wraps [`Entry`], and is used to store the generation of the entry. Every tim ethe entry is freed, the generation is incremented.
/// This prevents use-on-free from being an issue, so we just return None if the generation in the keys does not match.
struct SlabEntry<T> {
    entry: Entry<T>,
    generation: usize,
}

/// # Slab
/// A slab-like structure that is used to store actors. It can be reallocated/resized, but is initialized to a fixed size.
pub struct Slab<T> {
    /// The actual entries in the slab.
    entries: Vec<SlabEntry<T>>,
    /// The next free entry in the slab.
    next_free: usize,
    /// How many entries are currently in use.
    used: usize,
    /// The initial capacity of the slab.
    initial_capacity: usize,
}

impl<T> Slab<T> {
    /// Creates a new [`Slab`] initialized to the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            next_free: 0,
            used: 0,
            initial_capacity: capacity,
        }
    }

    /// Inserts a new entry into the slab.
    /// Returns None if the slab is full.
    pub fn insert(&mut self, value: T) -> Option<SlabKey> {

        // Get a mutable reference to the new entry
        let slab_entry = self.entries.get_mut(self.next_free);

        // If it works, then update the key
        if let Some(slab_entry) = slab_entry {

            // If the entry is used, panic. This should never happen.
            let Entry::Free(next_free) = slab_entry.entry else {
                panic!("The slab's free list is corrupted!");
            };

            // Create the key
            let key = SlabKey {
                index: self.next_free,
                generation: slab_entry.generation,
            };

            // Pop the value off the stack
            self.next_free = next_free;

            // Set the value to Used
            slab_entry.entry = Entry::Used(value);

            // Increment the used count
            self.used += 1;

            // Return the key
            Some(key)
        } else {
            // Otherwise, we need to create a new entry and update next_free to match
            // Note: This will not reallocate if the slab is full
            // In fact, lets check for that here
            if self.next_free >= self.entries.capacity() {
                return None;
            }

            // Create the key at the end of the slab vector. Note: this is at the end of the vector's
            // length, not capacity. We do not need to subtract 1, because we will be pushing a new entry,
            // which will increase the length by 1. (entries.len() is giving us the index of the entry after the last entry, which we are about to push)
            let key = SlabKey {
                index: self.entries.len(),
                generation: 0, // This will be incremented each time a slot is freed, but because this is a new entry, it will be 0
            };

            // Create and push the entry
            self.entries.push(SlabEntry {
                entry: Entry::Used(value),
                generation: 0,
            });

            // Update next_free, which is after the end of the vector.
            self.next_free = self.entries.len();

            // Increment the used count
            self.used += 1;

            // Return the key
            Some(key)
        }
    }

    /// Remove an entry from the slab
    pub fn remove(&mut self, key: &SlabKey) {
        // Get the entry. If it doesn't exist, then we can return (because it never existed in the first place).
        let Some(slab_entry) = self.entries.get_mut(key.index) else {
            return;
        };

        // If it is used, then continue to remove, otherwise do nothing.
        if let Entry::Used(_) = slab_entry.entry {
            // If the key's generation does not match, then ignore the removal.
            if slab_entry.generation != key.generation {
                return;
            }

            // Increment the genration on the slab, invalidating any old SlabKey's referencing it.
            slab_entry.generation += 1;

            // Replace the entry with a free entry pointing to next_free
            slab_entry.entry = Entry::Free(self.next_free);

            // Update next_free to the index of the this entry
            self.next_free = key.index;

            // Decrement the used count
            self.used -= 1;
        }
        
    }


    /// Reallocates the slab, increasing the capacity by the given amount. If None is given, then the slab's capacity will double.
    pub fn increase_capacity(&mut self, additional: Option<usize>) {
        // Get the additional capacity, defaulting to doubling the capacity if None is given
        let additional = additional.unwrap_or(self.entries.capacity());

        // Increase the capacity
        self.entries.reserve(additional);
    }

    /// Reallocates the entire slab to the given capacity.
    /// WARNING: This function is completely destructive of the slab's contents. 
    pub fn deallocate_to(&mut self, capacity: usize) {
        // Reallocate the Vector.
        self.entries = Vec::with_capacity(capacity);
        // Reset the next_free
        self.next_free = 0;
        // Reset the used count
        self.used = 0;
        // Update the initial capacity
        self.initial_capacity = capacity;
    }

    /// Clears the entire slab.
    pub fn clear(&mut self) {
        self.deallocate_to(self.initial_capacity);
    }

    /// Returns the number of used entries in the slab.
    /// This does not correlate to the actual memory usage of the slab whatsoever, as freed entries are not deallocated.
    pub fn len(&self) -> usize {
        self.used
    }

    /// Returns true if len is 0.
    pub fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// Gets a reference to an entry from the slab.
    pub fn get(&self, key: &SlabKey) -> Option<&T> {
        // Get the entry. If it doesn't exist, then we can return None
        let slab_entry = self.entries.get(key.index)?;

        // If it is used and the generation matches, then return the value
        if let Entry::Used(ref value) = slab_entry.entry {
            if key.generation == slab_entry.generation {
                return Some(value);
            }
        }

        // Otherwise, return None.
        None
    }

    /// Gets a mutable reference to an entry from the slab
    pub fn get_mut(&mut self, key: &SlabKey) -> Option<&mut T> {
        // Get the entry. If it doesn't exist, then we can return None
        let slab_entry = self.entries.get_mut(key.index)?;

        // If it is used and the generation matches, then return the value
        if let Entry::Used(ref mut value) = slab_entry.entry {
            if key.generation == slab_entry.generation {
                return Some(value);
            }
        }

        // Otherwise, return None.
        None
    }
}