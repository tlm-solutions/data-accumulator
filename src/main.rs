mod structs;

use serde::{Serialize};
use std::fs::{File, OpenOptions};
use actix_web::{web, App, HttpServer, Responder};
use structs::{Telegram, Response, RawData};
use std::env;
use csv::{WriterBuilder};

async fn dump_to_file<T: Serialize>(file_path: &str, data: &T ) {
    println!("FILE: {} {}", file_path, std::path::Path::new(file_path).exists());

    let file: File;
    let mut file_existed: bool = true;
    if std::path::Path::new(file_path).exists() {
        println!("File Exists ... ");
        file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&file_path)
            .unwrap();

        file_existed = false;
    } else {
        file = File::create(file_path).unwrap();
    }
    let mut wtr = WriterBuilder::new()
         .has_headers(file_existed)
         .from_writer(file);
    //let mut wtr = csv::Writer::from_writer(file);
    wtr.serialize(&data);
    wtr.flush();
}


async fn formatted(telegram: web::Json<Telegram>) -> impl Responder {
    let default_file = String::from("/var/lib/data-accumulator/formatte_data.csv");
    let csv_file = env::var("PATH_FORMATTED_DATA").unwrap_or(default_file);

    println!("Received Formatted Record: {:?}", &telegram);
    dump_to_file(&csv_file, &telegram).await;

    web::Json(Response { success: true })
}

async fn raw(telegram: web::Json<RawData>) -> impl Responder {
    let default_file = String::from("/var/lib/data-accumulator/raw_data.csv");
    let csv_file = env::var("PATH_RAW_DATA").unwrap_or(default_file);

    println!("Received Formatted Record: {:?}", &telegram);
    dump_to_file(&csv_file, &telegram).await;

    web::Json(Response { success: true })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting Server ... ");
    let host = "127.0.0.1";
    let port = 8080;
    println!("Listening on: {}:{}", host, port);

    HttpServer::new(|| App::new()
                    .route("/formatted_telegram", web::post().to(formatted))
                    .route("/raw_telegram", web::post().to(raw))

                    )
        .bind((host, port))?
        .run()
        .await
}


