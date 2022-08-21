use super::{CSVFile, DataPipelineReceiverRaw, Empty, PostgresDB, Storage};
use dump_dvb::telegrams::raw::RawSaveTelegram;

use log::{info, warn};
use std::env;

pub struct ProcessorDatabaseRaw {
    backend: Box<dyn Storage>,
    receiver_raw: DataPipelineReceiverRaw,
}

impl ProcessorDatabaseRaw {
    pub fn new(receiver_raw: DataPipelineReceiverRaw)-> ProcessorDatabaseRaw {
        let backend = env::var("DATABASE_BACKEND").expect("You need to specify a DATABASE_BACKEND");

        if backend == "POSTGRES" {
            info!("Using PostgresDB Backend for RawTelegram Database");
            ProcessorDatabaseRaw {
                backend: Box::new(PostgresDB::new()),
                receiver_raw: receiver_raw,
            }
        } else if backend == "CSVFILE" {
            info!("Using CSVFILE Backend for RawTelegram Database");
            ProcessorDatabaseRaw {
                backend: Box::new(CSVFile::new()),
                receiver_raw: receiver_raw,
            }
        } else {
            warn!("[WARNING] NO Backend specified!");

            ProcessorDatabaseRaw {
                backend: Box::new(Empty::new()),
                receiver_raw: receiver_raw,
            }
        }
    }

    pub async fn process_database(&mut self) {
        self.backend.setup().await;
        loop {
            let (telegram, meta) = self.receiver_raw.recv().unwrap();
            info!(
                "[ProcessorDatabase] post: queue size: {}",
                self.receiver_raw.try_iter().count()
            );

            let save_telegram = RawSaveTelegram::from(telegram, meta);
            self.backend.write_raw(save_telegram).await;
        }
    }
}
