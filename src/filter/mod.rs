use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use dump_dvb::telegrams::r09::R09Telegram;

pub const DEPULICATION_BUFFER_SIZE: usize = 30;

// Saves the hashes of last n telegrams
// if a new telegram is received we check if its contained
// in this circular buffer.
pub struct Filter {
    pub last_elements: [u64; DEPULICATION_BUFFER_SIZE], // vector of hashes for deduplication
    pub iterator: usize,                                // keeps track of the oldest element
}

impl Filter {
    pub fn new() -> Filter {
        Filter {
            last_elements: [0; DEPULICATION_BUFFER_SIZE],
            iterator: 0,
        }
    }

    pub async fn calculate_hash<T: Hash>(t: &T) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    // checks if the given telegram hash is contained in the filter class
    pub async fn deduplicate(self: &mut Filter, telegram: &R09Telegram) -> bool {
        let telegram_hash = Self::calculate_hash(telegram).await;

        // checks if the given telegram is already in the buffer
        let contained = self.last_elements.contains(&telegram_hash);

        if contained {
            return true;
        }

        // updates the buffer adding the new telegram
        let index = self.iterator;
        self.last_elements[index] = telegram_hash;
        self.iterator = (self.iterator + 1) % DEPULICATION_BUFFER_SIZE;

        contained
    }
}
