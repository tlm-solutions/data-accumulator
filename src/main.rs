extern crate clap;

mod structs;
mod processor;

use structs::{Response, Args, StopConfig};
use processor::{Processor, Telegram, RawData, DEPULICATION_BUFFER_SIZE};

use dvb_dump::receives_telegrams_client::{ReceivesTelegramsClient};
use dvb_dump::{ ReturnCode, ReducedTelegram };

pub mod dvb_dump{
    tonic::include_proto!("dvbdump");
}


use serde_json::Map;
use actix_web::{web, App, HttpServer, Responder};
use std::env;
use std::sync::{RwLock};
use clap::Parser;
use std::collections::HashMap;

async fn formatted(processor: web::Data<RwLock<Processor>>, telegram: web::Json<Telegram>) -> impl Responder {

    let telegram_hash = Processor::calculate_hash(&telegram).await;
    let contained;
    // checks if the given telegram is already in the buffer
     {
        let readable_processor = processor.read().unwrap();
        contained = readable_processor.last_elements.contains(&telegram_hash);
    }
    // updates the buffer adding the new telegram
    {
        let mut writeable_processor = processor.write().unwrap();
        let index = writeable_processor.iterator;
        writeable_processor.last_elements[index] = telegram_hash;
        writeable_processor.iterator = (writeable_processor.iterator + 1) % DEPULICATION_BUFFER_SIZE;
    }

    // do more processing
    if !contained {
        let default_file = String::from("/var/lib/data-accumulator/formatte_data.csv");
        let csv_file = env::var("PATH_FORMATTED_DATA").unwrap_or(default_file);

        //let default_public_api = String::from("127.0.0.1:50051");
        //let url_public_api = env::var("PUBLIC_API").unwrap_or(default_public_api);
        let default_public_api = String::from("./stops.json");
        let url_public_api = env::var("STOPS_CONFIG").unwrap_or(default_public_api);

        println!("NEW Received Formatted Record: {:?}", &telegram);
        let mut client = ReceivesTelegramsClient::connect("http://127.0.0.1:50051").await.unwrap();

        const FILE_STR: &'static str = include_str!("../stops.json");
        let parsed: HashMap<String, StopConfig> = serde_json::from_str(&FILE_STR).expect("JSON was not well-formatted");
        //let junction_string = self.junction.to_string();
        //let junction = parsed.get(&junction_string).map(|u| u.as_str().unwrap()).unwrap_or(&junction_string);
        let mut lat;
        let mut lon;
        println!("X: {} {}", telegram.junction.to_string(), parsed.contains_key(&telegram.junction.to_string()));
        match parsed.get(&telegram.junction.to_string()) {
            Some(data) => {
                println!("KNOWN Station: {} -> {}", telegram.junction, data.name);
                lat = data.lat;
                lon = data.lon;
            }
            None => {
                println!("UNKOWN");
                lat = 0f64;
                lon = 0f64;
            }
        }

        let request = tonic::Request::new(ReducedTelegram {
            time_stamp: telegram.time_stamp,
            position_id: telegram.junction,
            line: telegram.line,
            delay: ((telegram.sign_of_deviation as i32) * 2 - 1) * telegram.value_of_deviation as i32,
            direction: telegram.run_number,
            destination_number: telegram.junction_number,
            status: 0,
            lat: lat as f32,
            lon: lon as f32
        });

        let response = client.receive_new(request).await;

        //file_write.await;
        //response.await;
    } else {
        println!("DROPPED TELEGRAM");
    }

    web::Json(Response { success: true })
}

async fn raw(telegram: web::Json<RawData>) -> impl Responder {
    let default_file = String::from("/var/lib/data-accumulator/raw_data.csv");
    let csv_file = env::var("PATH_RAW_DATA").unwrap_or(default_file);

    println!("Received Formatted Record: {:?}", &telegram);
    Processor::dump_to_file(&csv_file, &telegram).await;

    web::Json(Response { success: true })
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    println!("Listening on: {}:{}", host, port);
    let data = web::Data::new(RwLock::new(Processor::new())); 
    HttpServer::new(move || App::new()
                    .app_data(data.clone())
                    .route("/formatted_telegram", web::post().to(formatted))
                    .route("/raw_telegram", web::post().to(raw))

                    )
        .bind((host, port))?
        .run()
        .await
}


