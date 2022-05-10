pub mod telegram;

pub use telegram::{Telegram, RawData};
use super::{SaveTelegram, Storage, InfluxDB};

use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::Hash;
use std::env;

pub const DEPULICATION_BUFFER_SIZE: usize = 20;

pub struct Processor {
    pub last_elements: [u64; DEPULICATION_BUFFER_SIZE], // vector of hashes for deduplication
    pub iterator: usize, // keeps track of the oldest element
    pub data_sink: InfluxDB
}

impl Processor {
    pub fn new() -> Processor {
        let default_influx_host = String::from("http://127.0.0.1:8082");
        let influx_host = env::var("INFLUXDB_HOST").unwrap_or(default_influx_host);

        Processor {
            last_elements: [0; DEPULICATION_BUFFER_SIZE],
            iterator: 0, 
            data_sink: InfluxDB::new(&influx_host)
        }
    }

    pub async fn calculate_hash(t: &Telegram) -> u64 {
        let mut s = DefaultHasher::new();
        t.hash(&mut s);
        s.finish()
    }

    pub async fn write(&mut self, telegram: SaveTelegram) {
        self.data_sink.write(telegram);
    }

}




