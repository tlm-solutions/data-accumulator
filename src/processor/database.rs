use super::{Storage, PostgresDB, CSVFile, Empty, DataPipelineReceiver};
use std::env;
use telegrams::{R09SaveTelegram, };

pub struct ProcessorDatabase {
    backend: Box<dyn Storage>,
    receiver: DataPipelineReceiver
}

impl ProcessorDatabase {
    pub fn new(receiver: DataPipelineReceiver) -> ProcessorDatabase{
        let backend = env::var("DATABASE_BACKEND").expect("You need to specify a DATABASE_BACKEND");

        if backend == "POSTGRES" {
            ProcessorDatabase {
                backend: Box::new(PostgresDB::new()),
                receiver: receiver
            }
        } else if backend == "CSVFILE" {
            ProcessorDatabase {
                backend: Box::new(CSVFile::new()),
                receiver: receiver
            }
        } else {
            println!("[WARNING] NO Backend specified!");

            ProcessorDatabase {
                backend: Box::new(Empty::new()),
                receiver: receiver
            }
        } 
    }

    pub async fn process_database(&mut self) {
        loop {
            let (telegram, meta) = self.receiver.recv().unwrap();
            println!(
                "[ProcessorDatabase] post: queue size: {}",
                self.receiver.try_iter().count()
            );
            let save_telegram = R09SaveTelegram::from(telegram, meta);

            self.backend.write(save_telegram).await;
        }
    }

}

