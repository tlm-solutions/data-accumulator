use async_trait::async_trait;
use csv::WriterBuilder;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;
use std::fs::{File, OpenOptions};
use telegrams::{schema, R09SaveTelegram};

#[async_trait]
pub trait Storage {
    fn new() -> Self
    where
        Self: Sized;
    async fn setup(&mut self);
    async fn write(&mut self, data: R09SaveTelegram);
}

pub struct CSVFile {
    pub file_path: String,
}

pub struct PostgresDB {
    connection: PgConnection,
}

pub struct Empty {}

#[async_trait]
impl Storage for PostgresDB {
    fn new() -> PostgresDB {
        let default_postgres_host = String::from("localhost:5433");
        let default_postgres_port = String::from("5432");

        let postgres_host = format!(
            "postgres://dvbdump:{}@{}:{}/dvbdump",
            env::var("POSTGRES_PASSWORD").unwrap(),
            env::var("POSTGRES_HOST").unwrap_or(default_postgres_host.clone()),
            env::var("POSTGRES_PORT").unwrap_or(default_postgres_port.clone())
        );

        PostgresDB {
            connection: PgConnection::establish(&postgres_host)
                .expect(&format!("Error connecting to {}", postgres_host)),
        }
    }

    async fn setup(&mut self) {}

    async fn write(&mut self, data: R09SaveTelegram) {
        match diesel::insert_into(schema::r09_telegrams::table)
            .values(&data)
            .get_result::<R09SaveTelegram>(&self.connection)
        {
            Err(e) => {
                println!("Postgres Error {:?}", e);
            }
            _ => {}
        };
    }
}

#[async_trait]
impl Storage for CSVFile {
    fn new() -> CSVFile {
        let resource = env::var("CSV_FILE").expect("Need to specify a csv file");
        println!("CSV File writes to {}", resource);
        CSVFile {
            file_path: resource.clone(),
        }
    }

    async fn setup(&mut self) {}

    async fn write(&mut self, data: R09SaveTelegram) {
        let file: File;
        let mut file_existed: bool = true;
        if std::path::Path::new(&self.file_path).exists() {
            file = OpenOptions::new()
                .write(true)
                .append(true)
                .open(&self.file_path)
                .unwrap();

            file_existed = false;
        } else {
            file = File::create(&self.file_path).unwrap();
        }
        let mut wtr = WriterBuilder::new()
            .has_headers(file_existed)
            .from_writer(file);

        wtr.serialize(data).expect("Cannot serialize data");
        wtr.flush().expect("Cannot flush csv file");
    }
}

#[async_trait]
impl Storage for Empty {
    fn new() -> Empty {
        Empty {}
    }
    async fn setup(&mut self) {}
    async fn write(&mut self, _data: R09SaveTelegram) {}
}
