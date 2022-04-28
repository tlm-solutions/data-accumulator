pub mod telegram;

pub use telegram::{Telegram, RawData};

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::Hash;
use csv::{WriterBuilder};
use serde::{Serialize};
use std::fs::{File, OpenOptions};

pub const DEPULICATION_BUFFER_SIZE: usize = 20;

pub struct Processor {
    pub last_elements: [u64; DEPULICATION_BUFFER_SIZE], // vector of hashes for deduplication
    pub iterator: usize, // keeps track of the oldest element
}

impl Processor {
    pub fn new() -> Processor {
        Processor {
            last_elements: [0; DEPULICATION_BUFFER_SIZE],
            iterator: 0
        }
    }

    pub async fn calculate_hash(t: &Telegram) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    pub async fn process(&mut self, telegram: Telegram) {
        
    }

    pub async fn dump_to_file<T: Serialize>(file_path: &str, data: &T ) {
        let file: File;
        let mut file_existed: bool = true;
        if std::path::Path::new(file_path).exists() {
            file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&file_path)
                .unwrap();

            file_existed = false;
        } else {
            file = File::create(file_path).unwrap();
        }
        let mut wtr = WriterBuilder::new()
             .has_headers(file_existed)
             .from_writer(file);

        wtr.serialize(&data);
        wtr.flush();
    }
}




