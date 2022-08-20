use super::{CSVFile, DataPipelineReceiverR09, Empty, PostgresDB, Storage};
use std::env;
use dump_dvb::telegrams::r09::{R09SaveTelegram, R09Telegram};

pub struct ProcessorDatabaseR09 {
    backend: Box<dyn Storage>,
    receiver_r09: DataPipelineReceiverR09,
}

impl ProcessorDatabaseR09 {
    pub fn new(receiver_r09: DataPipelineReceiverR09)-> ProcessorDatabaseR09 {
        let backend = env::var("DATABASE_BACKEND").expect("You need to specify a DATABASE_BACKEND");

        if backend == "POSTGRES" {
            ProcessorDatabaseR09 {
                backend: Box::new(PostgresDB::new()),
                receiver_r09: receiver_r09,
            }
        } else if backend == "CSVFILE" {
            ProcessorDatabaseR09 {
                backend: Box::new(CSVFile::new()),
                receiver_r09: receiver_r09,
            }
        } else {
            println!("[WARNING] NO Backend specified!");

            ProcessorDatabaseR09 {
                backend: Box::new(Empty::new()),
                receiver_r09: receiver_r09,
            }
        }
    }

    pub async fn process_database(&mut self) {
        loop {
            let (telegram, meta) = self.receiver_r09.recv().unwrap();
            println!(
                "[ProcessorDatabase] post: queue size: {}",
                self.receiver_r09.try_iter().count()
            );

            let save_telegram = R09SaveTelegram::from(telegram, meta);
            self.backend.write_r09(save_telegram).await;
        }
    }
}
