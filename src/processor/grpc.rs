
use std::sync::mpsc::{Receiver};
use std::env;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use std::io::stdout;
use std::io::Write;

use super::{Telegram, Station};

use dvb_dump::receives_telegrams_client::{ReceivesTelegramsClient};
use dvb_dump::{ ReducedTelegram };

pub mod dvb_dump{
    tonic::include_proto!("dvbdump");
}

pub struct ProcessorGrpc {
    grpc_host: String,
    receiver_grpc: Receiver<(Telegram, String)>,
}

impl ProcessorGrpc {
    pub fn new(receiver_grpc: Receiver<(Telegram, String)>) -> ProcessorGrpc {
        let default_grpc_host = String::from("http://127.0.0.1:50051");
        let grpc_host = env::var("GRPC_HOST").unwrap_or(default_grpc_host);

        ProcessorGrpc {
            grpc_host: String::from(grpc_host),
            receiver_grpc: receiver_grpc
        }
    }

    pub async fn process_grpc(&mut self) {
        loop {

            println!("[ProcessorDatabase] pre: queue size: {}", self.receiver_grpc.try_iter().count());
            let (telegram, ip) = self.receiver_grpc.recv().unwrap();
            println!("[ProcessorDatabase] post: queue size: {}", self.receiver_grpc.try_iter().count());
            println!("[ProcessorGrpc] Received Telegram! {} {:?}", ip, telegram);
            stdout().flush();

            // dont cry code reader this will TM be replaced by postgress look up 
            // revol-xut May the 8 2022
            let stations = HashMap::from([
                (String::from("10.13.37.100"), Station {
                    name: String::from("Barkhausen/Turmlabor"),
                    lat: 51.027105,
                    lon: 13.723606,
                    station_id: 0,
                    region_id: 0  
                }),
                (String::from("10.13.37.101"), Station {
                    name: String::from("Zentralwerk"),
                    lat: 51.0810632,
                    lon: 13.7280758,
                    station_id: 1,
                    region_id: 0,
                }),
                (String::from("10.13.37.102"), Station {
                    name: String::from(""),
                    lat: 50.822755,
                    lon: 12.933914,
                    station_id: 2,
                    region_id: 1 
                }),
            ]);

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
                            client.receive_new(request).await.is_ok();
                        }
                        Err(_) => {
                            println!("[ProcessorGrpc] Cannot connect to GRPC Host");
                            stdout().flush();
                        }
                    };
                }
                _ => {}
            };
        }
    }
}


