extern crate clap;

mod structs;
mod filter;
mod processor;
mod stations;
mod storage;

use structs::{Response, Args};
pub use filter::{Filter, Telegram, RawData, DEPULICATION_BUFFER_SIZE};
use processor::{ProcessorGrpc, ProcessorDatabase};
pub use stations::{Station};
pub use storage::{SaveTelegram, Storage, InfluxDB, CSVFile};

use actix_web::{web, App, HttpServer, Responder, HttpRequest};
use std::sync::{RwLock, Mutex};
use clap::Parser;
use std::sync::mpsc::{SyncSender};
use std::sync::mpsc;
use std::thread;

async fn formatted(filter: web::Data<RwLock<Filter>>,
                   database_sender: web::Data<Mutex<SyncSender<(Telegram, String)>>>,
                   grpc_sender: web::Data<Mutex<SyncSender<(Telegram, String)>>>,
                   telegram: web::Json<Telegram>, 
                   req: HttpRequest) -> impl Responder {

    let telegram_hash = Filter::calculate_hash(&telegram).await;
    let contained;
    // checks if the given telegram is already in the buffer
     {
        let readable_filter = filter.read().unwrap();
        contained = readable_filter.last_elements.contains(&telegram_hash);
    }

    // updates the buffer adding the new telegram
    {
        let mut writeable_filter = filter.write().unwrap();
        let index = writeable_filter.iterator;
        writeable_filter.last_elements[index] = telegram_hash;
        writeable_filter.iterator = (writeable_filter.iterator + 1) % DEPULICATION_BUFFER_SIZE;
    }

    // do more processing
    if !contained {
        let ip_address;
        if let Some(val) = req.peer_addr() {
            ip_address = val.ip().to_string();
        } else {
            return web::Json(Response { success: false });
        }

        println!("Received Telegram! {} {:?}", &ip_address, &telegram);
        match grpc_sender.lock().unwrap().try_send(((*telegram).clone(), ip_address.clone())) {
            Err(err) => {
                println!("Channel GRPC has problems ! {:?}", err);
            }
            _ => {}
        }
        match database_sender.lock().unwrap().try_send(((*telegram).clone(), ip_address.clone())) {
            Err(err) => {
                println!("Channel Database has problems ! {:?}", err);
            },
            _ => {

            }
        }
    }

    web::Json(Response { success: true })
}

async fn raw(telegram: web::Json<RawData>) -> impl Responder {
    //let default_file = String::from("/var/lib/data-accumulator/raw_data.csv");
    //let csv_file = env::var("PATH_RAW_DATA").unwrap_or(default_file);

    println!("Received Formatted Record: {:?}", &telegram);
    //Processor::dump_to_file(&csv_file, &telegram).await;

    web::Json(Response { success: true })
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    let filter = web::Data::new(RwLock::new(Filter::new()));

    let (sender_database, receiver_database) = mpsc::sync_channel::<(Telegram, String)>(10);
    let (sender_grpc, receiver_grpc) = mpsc::sync_channel::<(Telegram, String)>(10);

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_io().enable_time().build().unwrap();
        let mut processor_database = ProcessorDatabase::new(receiver_database);
        rt.block_on(processor_database.process_database());
    });

    thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread().enable_io().enable_time().build().unwrap();
        let mut processor_grpc = ProcessorGrpc::new(receiver_grpc);
        rt.block_on(processor_grpc.process_grpc());
    });

    let web_database_sender = web::Data::new(Mutex::new(sender_database));
    let web_grpc_sender = web::Data::new(Mutex::new(sender_grpc));

    println!("Listening on: {}:{}", host, port);
    HttpServer::new(move || App::new()
                    .app_data(filter.clone())
                    .app_data(web_database_sender.clone())
                    .app_data(web_grpc_sender.clone())
                    .route("/formatted_telegram", web::post().to(formatted))
                    .route("/raw_telegram", web::post().to(raw))

                    )
        .bind((host, port))?
        .run()
        .await
}


