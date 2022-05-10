
use super::{Telegram};

use csv::{WriterBuilder};
use std::fs::{File, OpenOptions};
use serde::{Deserialize, Serialize};
use influxdb::{Client};
use influxdb::InfluxDbWriteable;
use chrono::{DateTime, Utc, TimeZone};
use tokio::runtime::Handle;
use futures::executor;

#[derive(Deserialize, Serialize, Debug, InfluxDbWriteable)]
pub struct SaveTelegram {
    pub time: DateTime<Utc>,
    pub ip: String,
    pub station_id: u32,
    pub line: String,
    pub destination_number: String,
    pub priority: u32,
    pub delay: i32,
    pub reporting_point: u32,
    pub direction_request: u32,
    pub run_number: String,
    pub reserve: u32,
    pub train_length: u32,
    pub junction: u32,
    pub junction_number: u32,
    pub request_status: u32,
}

impl SaveTelegram {
    pub fn from(telegram: &Telegram, ip: &str) -> SaveTelegram {
        SaveTelegram {
            time: Utc.timestamp(telegram.time_stamp as i64, 0),
            ip: ip.to_owned(),
            station_id: telegram.reporting_point,
            line: telegram.line.clone(),
            destination_number: telegram.destination_number.clone(),
            priority: telegram.priority,
            delay: (telegram.sign_of_deviation as i32 - 2i32) * telegram.value_of_deviation as i32,
            reporting_point: telegram.reporting_point,
            direction_request: telegram.request_for_priority,
            run_number: telegram.run_number.clone(),
            reserve: telegram.reserve,
            train_length: telegram.train_length,
            junction: telegram.junction,
            junction_number: telegram.junction_number,
            request_status: telegram.request_status
        }
    }
}

pub trait Storage {
    fn new(resource: &String) -> Self;
    fn write(&mut self, data: SaveTelegram);
}

pub struct InfluxDB {
    pub uri: String,
    client: Client
}

pub struct CSVFile {
    pub file_path: String,
}

impl Storage for CSVFile {
    fn new(resource: &String) -> CSVFile {
        CSVFile {
            file_path: resource.clone(),
        }
    }
    fn write(&mut self, data: SaveTelegram) {
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

        wtr.serialize(data);
        wtr.flush();
    }
}
impl InfluxDB {
    pub fn new(resource: &String) -> InfluxDB {
        InfluxDB {
            uri: resource.to_string(),
            client: Client::new(resource, "dvbdump")
        }
    }
    pub async fn write(&mut self, data: SaveTelegram) {
        let handle = Handle::current();
        let write_result = self.client.query(data.into_query("telegramr09")).await;

        match write_result {
            Ok(_) => {
                println!("Sucessfully wrote into influxdb");
            }
            Err(_) => {
                println!("Connection Timeout to InfluxDB. Reopening Connection.");
                self.client = Client::new(&self.uri, "dvbdump");
            }
        }
    }
}

