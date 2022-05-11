pub mod telegram;

pub use telegram::{Telegram, RawData};

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::Hash;

pub const DEPULICATION_BUFFER_SIZE: usize = 20;

pub struct Filter {
    pub last_elements: [u64; DEPULICATION_BUFFER_SIZE], // vector of hashes for deduplication
    pub iterator: usize, // keeps track of the oldest element
}

impl Filter {
    pub fn new() -> Filter {
        Filter {
            last_elements: [0; DEPULICATION_BUFFER_SIZE],
            iterator: 0, 
        }
    }

    pub async fn calculate_hash(t: &Telegram) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }
}




