mod structs;
mod processor;

use structs::{Response};
use processor::{Processor, Telegram, RawData, DEPULICATION_BUFFER_SIZE};
use serde::{Serialize};
use std::fs::{File, OpenOptions};
use actix_web::{web, App, HttpServer, Responder};
use std::env;
use csv::{WriterBuilder};
use std::sync::{RwLock};

async fn formatted(processor: web::Data<RwLock<Processor>>, telegram: web::Json<Telegram>) -> impl Responder {

    let telegram_hash = Processor::calculate_hash(&telegram).await;
    println!("Hash {}", telegram_hash);
    let contained;
    match processor.read() {
        Ok(readable_processor) => {
            println!("Start: index: {} field: {:?}", &readable_processor.iterator, &readable_processor.last_elements);
            contained = readable_processor.last_elements.contains(&telegram_hash);
        }
        Err(_) => {
            println!("ALARM!");
            contained = false;
        }
    }
    println!("Contains {}", contained);
    match processor.write() {
        Ok(mut writeable_processor) => {
            println!("End: end: {} field: {:?}", &writeable_processor.iterator, &writeable_processor.last_elements);
            let index = writeable_processor.iterator;
            writeable_processor.last_elements[index] = telegram_hash;
            writeable_processor.iterator = (writeable_processor.iterator + 1) % DEPULICATION_BUFFER_SIZE;
            println!("End: end: {} field: {:?}", &writeable_processor.iterator, &writeable_processor.last_elements);
        }
        Err(_) => {
            println!("ALAARM!!");
        }
    };

    // do more processing
    if !contained {
        let default_file = String::from("/var/lib/data-accumulator/formatte_data.csv");
        let csv_file = env::var("PATH_FORMATTED_DATA").unwrap_or(default_file);

        println!("NEW Received Formatted Record: {:?}", &telegram);
        Processor::dump_to_file(&csv_file, &telegram).await;
    } else {
        println!("OLD Received Formatted Record: {:?}", &telegram);
    }

    web::Json(Response { success: true })
}

async fn raw(telegram: web::Json<RawData>) -> impl Responder {
    let default_file = String::from("/var/lib/data-accumulator/raw_data.csv");
    let csv_file = env::var("PATH_RAW_DATA").unwrap_or(default_file);

    println!("Received Formatted Record: {:?}", &telegram);
    //dump_to_file(&csv_file, &telegram).await;

    web::Json(Response { success: true })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Server ... ");
    let host = "127.0.0.1";
    let port = 8080;
    println!("Listening on: {}:{}", host, port);
    HttpServer::new(|| App::new()
                    .app_data(web::Data::new(RwLock::new(Processor::new())))
                    .route("/formatted_telegram", web::post().to(formatted))
                    .route("/raw_telegram", web::post().to(raw))

                    )
        .bind((host, port))?
        .run()
        .await
}


