use std::sync::mpsc::{Receiver};
use std::env;

use super::{InfluxDB, Storage, CSVFile};
use super::{Telegram, SaveTelegram};

pub struct ProcessorDatabase {
    database: Box<dyn Storage>,
    receiver_database: Receiver<(Telegram, String)>
}

impl ProcessorDatabase {
    pub fn new(receiver_database: Receiver<(Telegram, String)>) -> ProcessorDatabase {
        let storage: Box<dyn Storage>;

        let default_influx_host = String::from("http://localhost:8086");
        let influx_host = env::var("INFLUX_HOST").unwrap_or(default_influx_host);

        match env::var("CSV_FILE") {
            Ok(file_path) => {
                println!("Using the CSV File: {}", &file_path);
                storage = Box::new(CSVFile::new(&file_path));
            }
            Err(_) => {
                println!("Using the InfluxDB {}", &influx_host);
                storage = Box::new(InfluxDB::new(&influx_host));
            }
        }

        ProcessorDatabase {
            database: storage,
            receiver_database: receiver_database,
        }
    }

    pub async fn process_database(&mut self) {
        self.database.setup().await;

        loop {

            let (telegram, ip) = self.receiver_database.recv().unwrap();
            //println!("[ProcessorDatabase] Received Telegram! {} {:?}", ip, telegram);
            //stdout().flush();

            let save = SaveTelegram::from(&telegram, &ip);
            self.database.write(save).await;
        }
    }
}
