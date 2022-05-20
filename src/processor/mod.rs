
use std::sync::mpsc::{Receiver};
use std::env;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::{InfluxDB, Storage, CSVFile};
use super::{Telegram, SaveTelegram, Station};

use dvb_dump::receives_telegrams_client::{ReceivesTelegramsClient};
use dvb_dump::{ ReducedTelegram };

pub mod dvb_dump{
    tonic::include_proto!("dvbdump");
}

pub struct Processor {
    database: Box<dyn Storage>,
    grpc_host: String,
    receiver: Receiver<(Telegram, String)>
}

impl Processor {
    pub fn new(receiver: Receiver<(Telegram, String)>) -> Processor {
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

        let default_grpc_host = String::from("http://127.0.0.1:50051");
        let grpc_host = env::var("GRPC_HOST").unwrap_or(default_grpc_host);

        Processor {
            database: storage,
            grpc_host: String::from(grpc_host),
            receiver: receiver
        }
    }

    pub async fn processing_loop(&mut self) {
        self.database.setup().await;
        loop {
            let (telegram, ip) = self.receiver.recv().unwrap();
            let save = SaveTelegram::from(&telegram, &ip);
            self.database.write(save).await;

            // dont cry code reader this will TM be replaced by postgress look up 
            // revol-xut May the 8 2022
            let stations = HashMap::from([
                (String::from("10.13.37.100"), Station {
                    name: String::from("Barkhausen/Turmlabor"),
                    lat: 51.026107,
                    lon: 13.623566,
                    station_id: 0,
                    region_id: 0  
                }),
                (String::from("127.0.0.1"), Station {
                    name: String::from("Zentralwerk"),
                    lat: 51.0810632,
                    lon: 13.7280758,
                    station_id: 1,
                    region_id: 0,
                }),
                (String::from("10.13.37.102"), Station {
                    name: String::from(""),
                    lat: 51.0810632,
                    lon: 13.7280758,
                    station_id: 2,
                    region_id: 1 
                }),
            ]);

            println!("IP: {}", &ip);
            match stations.get(&ip) {
                Some(station) => {
                    let start = SystemTime::now();
                    let since_the_epoch = start
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_secs();

                    let request = tonic::Request::new(ReducedTelegram {
                        time_stamp: since_the_epoch,
                        position_id: telegram.junction,
                        direction: telegram.request_for_priority,
                        status: telegram.request_status,
                        line: telegram.line.parse::<u32>().unwrap_or(0),
                        delay: ((telegram.sign_of_deviation as i32) * -2 + 1) * telegram.value_of_deviation as i32,
                        destination_number: telegram.destination_number.parse::<u32>().unwrap_or(0),
                        run_number: telegram.run_number.parse::<u32>().unwrap_or(0),
                        train_length: telegram.train_length,
                        region_code: station.region_id
                    });

                    match ReceivesTelegramsClient::connect(self.grpc_host.clone()).await {
                        Ok(mut client) => {
                            client.receive_new(request).await;
                        }
                        Err(_) => {
                            println!("Cannot connect to GRPC Host");
                        }
                    };
                }
                _ => {}
            };
        }
    }
}


