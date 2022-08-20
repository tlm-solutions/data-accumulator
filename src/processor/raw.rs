use super::{CSVFile, DataPipelineReceiverRaw, Empty, PostgresDB, Storage};
use std::env;
use dump_dvb::telegrams::raw::{RawSaveTelegram, RawTelegram};

pub struct ProcessorDatabaseRaw {
    backend: Box<dyn Storage>,
    receiver_raw: DataPipelineReceiverRaw,
}

impl ProcessorDatabaseRaw {
    pub fn new(receiver_raw: DataPipelineReceiverRaw)-> ProcessorDatabaseRaw {
        let backend = env::var("DATABASE_BACKEND").expect("You need to specify a DATABASE_BACKEND");

        if backend == "POSTGRES" {
            ProcessorDatabaseRaw {
                backend: Box::new(PostgresDB::new()),
                receiver_raw: receiver_raw,
            }
        } else if backend == "CSVFILE" {
            ProcessorDatabaseRaw {
                backend: Box::new(CSVFile::new()),
                receiver_raw: receiver_raw,
            }
        } else {
            println!("[WARNING] NO Backend specified!");

            ProcessorDatabaseRaw {
                backend: Box::new(Empty::new()),
                receiver_raw: receiver_raw,
            }
        }
    }

    pub async fn process_database(&mut self) {
        loop {
            let (telegram, meta) = self.receiver_raw.recv().unwrap();
            println!(
                "[ProcessorDatabase] post: queue size: {}",
                self.receiver_raw.try_iter().count()
            );

            let save_telegram = RawSaveTelegram::from(telegram, meta);
            self.backend.write_raw(save_telegram).await;
        }
    }
}
