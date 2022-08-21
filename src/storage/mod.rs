use dump_dvb::telegrams::{r09::R09SaveTelegram, raw::RawSaveTelegram};
use dump_dvb::schema;

use async_trait::async_trait;
use csv::WriterBuilder;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use serde::Serialize;
use log::{warn, error};
use libc::chown;

use std::fs::{File, OpenOptions};
use std::path::Path;
use std::env;
use std::ffi::CString;
use libc::c_char;

#[async_trait]
pub trait Storage {
    fn new() -> Self
    where
        Self: Sized;
    async fn setup(&mut self);
    async fn write_r09(&mut self, data: R09SaveTelegram);
    async fn write_raw(&mut self, data: RawSaveTelegram);
}

pub struct CSVFile {
    pub file_path_r09: Option<String>,
    pub file_path_raw: Option<String>,
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
        let default_postgres_pw = String::from("default_pw");

        let postgres_host = format!(
            "postgres://telegrams:{}@{}:{}/telegrams",
            env::var("POSTGRES_TELEGRAMS_PASSWORD").unwrap_or(default_postgres_pw.clone()),
            env::var("POSTGRES_HOST").unwrap_or(default_postgres_host.clone()),
            env::var("POSTGRES_PORT").unwrap_or(default_postgres_port.clone())
        );

        PostgresDB {
            connection: PgConnection::establish(&postgres_host)
                .expect(&format!("Error connecting to {}", postgres_host)),
        }
    }

    async fn setup(&mut self) {}

    async fn write_r09(&mut self, data: R09SaveTelegram) {
        match diesel::insert_into(schema::r09_telegrams::table)
            .values(&data)
            .execute(&self.connection)
        {
            Err(e) => {
                warn!("Postgres Error {:?}", e);
            }
            _ => {}
        };
    }

    async fn write_raw(&mut self, data: RawSaveTelegram) {
        match diesel::insert_into(schema::raw_telegrams::table)
            .values(&data)
            .execute(&self.connection)
        {
            Err(e) => {
                warn!("Postgres Error {:?}", e);
            }
            _ => {}
        };
    }
}

impl CSVFile {
    fn write<T: Serialize>(file_path: &String, telegram: T) {
        let file: File;
        file = OpenOptions::new()
            .create(false)
            .read(true)
            .write(true)
            .append(true)
            .open(file_path)
            .unwrap();

        let mut wtr = WriterBuilder::new()
            .has_headers(true)
            .from_writer(file);

        wtr.serialize(telegram).expect("Cannot serialize data");
        wtr.flush().expect("Cannot flush csv file");
    }

    fn create_file(file_path: &String) {
        if !Path::new(file_path).exists() {
            match std::fs::File::create(file_path) {
                Ok(file) => {
                    let _wtr = WriterBuilder::new()
                        .has_headers(false)
                        .from_writer(file);

                    // TODO: this needs to be reworked when chown leaves unstable
                    let c_str = CString::new(file_path.as_str()).unwrap();
                    let c_path: *const c_char = c_str.as_ptr() as *const c_char;
                    unsafe {
                        // TODO: this is ugly
                        chown(c_path, 1501, 1501);
                    }
                }
                Err(e) => {
                    error!("cannot create file {} with error {:?}", file_path, e);
                }
            }
        }
    }
}

#[async_trait]
impl Storage for CSVFile {
    fn new() -> CSVFile {
        CSVFile {
            file_path_r09: env::var("CSV_FILE_R09").ok(),
            file_path_raw: env::var("CSV_FILE_RAW").ok(),
        }
    }

    async fn setup(&mut self) {
        match &self.file_path_r09 {
            Some(file_path) => { CSVFile::create_file(&file_path); }
            None => {}
        }
        match &self.file_path_raw {
            Some(file_path) => { CSVFile::create_file(&file_path); }
            None => {}
        }
    }


    async fn write_r09(&mut self, data: R09SaveTelegram) {
        match &self.file_path_r09 {
            Some(file_path) => {
                CSVFile::write::<R09SaveTelegram>(&file_path, data);
            }
            None => {}
        }
    }
    async fn write_raw(&mut self, data: RawSaveTelegram) {
        match &self.file_path_raw {
            Some(file_path) => {
                CSVFile::write::<RawSaveTelegram>(&file_path, data);
            }
            None => {}
        }
    }
}

#[async_trait]
impl Storage for Empty {
    fn new() -> Empty {
        Empty {}
    }
    async fn setup(&mut self) {}
    async fn write_r09(&mut self, _data: R09SaveTelegram) {}
    async fn write_raw(&mut self, _data: RawSaveTelegram) {}
}
