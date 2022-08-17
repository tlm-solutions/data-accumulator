#[macro_use]
extern crate diesel;
extern crate clap;
extern crate dotenv;

mod filter;
mod processor;
mod routes;
mod schema;
mod stations;
mod storage;
mod structs;

use filter::Filter;
use processor::{ProcessorDatabase, ProcessorGrpc};
pub use routes::{formatted, Station};
use stations::ClickyBuntyDatabase;
pub use storage::{CSVFile, Empty, PostgresDB, Storage};
use structs::Args;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use tokio::runtime::Builder;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Mutex, RwLock};
use std::thread;

use dump_dvb::telegrams::{TelegramMetaInformation, r09::R09Telegram };

pub type DataPipelineSender = SyncSender<(R09Telegram, TelegramMetaInformation)>;
pub type DataPipelineReceiver = Receiver<(R09Telegram, TelegramMetaInformation)>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    let database_struct = web::Data::new(ClickyBuntyDatabase::new());
    let filter = web::Data::new(RwLock::new(Filter::new()));

    let (sender_database, receiver_database) =
        mpsc::sync_channel::<(R09Telegram, TelegramMetaInformation)>(200);
    let (sender_grpc, receiver_grpc) =
        mpsc::sync_channel::<(R09Telegram, TelegramMetaInformation)>(200);

    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let mut processor_database = ProcessorDatabase::new(receiver_database);
        rt.block_on(processor_database.process_database());
    });

    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .unwrap();
        let mut processor_grpc = ProcessorGrpc::new(receiver_grpc);
        rt.block_on(processor_grpc.process_grpc());
    });

    let web_database_sender = Mutex::new(sender_database);
    let web_grpc_sender = Mutex::new(sender_grpc);

    let request_data = web::Data::new((web_grpc_sender, web_database_sender));
    println!("Listening on: {}:{}", host, port);
    HttpServer::new(move || {
        App::new()
            .app_data(filter.clone())
            .app_data(request_data.clone())
            .app_data(database_struct.clone())
            .route("/telegram/r09", web::post().to(formatted))
        //.route("/telegram/raw", web::post().to(raw))
    })
    .bind((host, port))?
    .run()
    .await
}
