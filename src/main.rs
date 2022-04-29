extern crate clap;

mod structs;
mod processor;

use structs::{Response, Args};
use processor::{Processor, Telegram, RawData, DEPULICATION_BUFFER_SIZE};

use actix_web::{web, App, HttpServer, Responder};
use std::env;
use std::sync::{RwLock};
use clap::Parser;

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

        println!("NEW Received Formatted Record: {:?}", &telegram);
        Processor::dump_to_file(&csv_file, &telegram).await;
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

    println!("Starting Server ... ");
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


