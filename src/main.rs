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
use processor::{ProcessorDatabaseR09, ProcessorDatabaseRaw, ProcessorGrpc};
pub use routes::{receiving_r09, receiving_raw, Station};
use stations::ClickyBuntyDatabase;
pub use storage::{CSVFile, Empty, PostgresDB, Storage};
use structs::Args;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use tokio::runtime::Builder;
use env_logger;
use log::info;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Mutex, RwLock};
use std::thread;

use dump_dvb::telegrams::{TelegramMetaInformation, r09::R09Telegram, raw::RawTelegram};

pub type DataPipelineSenderRaw = SyncSender<(RawTelegram, TelegramMetaInformation)>;
pub type DataPipelineSenderR09 = SyncSender<(R09Telegram, TelegramMetaInformation)>;

pub type DataPipelineReceiverRaw = Receiver<(RawTelegram, TelegramMetaInformation)>;
pub type DataPipelineReceiverR09 = Receiver<(R09Telegram, TelegramMetaInformation)>;

pub struct ApplicationState {
    database: Mutex<ClickyBuntyDatabase>,
    database_r09_sender: Mutex<DataPipelineSenderR09>,
    database_raw_sender: Mutex<DataPipelineSenderRaw>,
    grpc_sender: Mutex<DataPipelineSenderR09>,
    filter: Mutex<Filter>
}

impl ApplicationState {
    fn new(database_r09_sender: DataPipelineSenderR09,
           database_raw_sender: DataPipelineSenderRaw,
           grpc_sender: DataPipelineSenderR09, 
           offline: bool) -> ApplicationState {

        let database_struct;
        if offline {
            database_struct = ClickyBuntyDatabase::offline();
        } else {
            database_struct = ClickyBuntyDatabase::new();
        };

        ApplicationState {
            database: Mutex::new(database_struct),
            database_r09_sender: Mutex::new(database_r09_sender),
            database_raw_sender: Mutex::new(database_raw_sender),
            grpc_sender: Mutex::new(grpc_sender),
            filter: Mutex::new(Filter::new())
        }
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let log_level = if args.verbose { "info" } else { "warn" };
    std::env::set_var("RUST_LOG", format!("actix_web={}", log_level));
    env_logger::init();

    info!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    let (sender_r09_database, receiver_r09_database) =
        mpsc::sync_channel::<(R09Telegram, TelegramMetaInformation)>(200);
    let (sender_raw_database, receiver_raw_database) =
        mpsc::sync_channel::<(RawTelegram, TelegramMetaInformation)>(200);
    let (sender_grpc, receiver_grpc) =
        mpsc::sync_channel::<(R09Telegram, TelegramMetaInformation)>(200);

    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("cannot spawn data processor r09");
        let mut processor_database_r09 = ProcessorDatabaseR09::new(receiver_r09_database);
        rt.block_on(processor_database_r09.process_database());
    });

    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("cannot spawn processsor raw");
        let mut processor_database_raw = ProcessorDatabaseRaw::new(receiver_raw_database);
        rt.block_on(processor_database_raw.process_database());
    });

    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("cannot spawn processor grpc");
        let mut processor_grpc = ProcessorGrpc::new(receiver_grpc);
        rt.block_on(processor_grpc.process_grpc());
    });

    let app_state = web::Data::new(RwLock::new(ApplicationState::new(
                    sender_r09_database, 
                    sender_raw_database, 
                    sender_grpc, 
                    args.offline
    )));

    info!("Listening on: {}:{}", host, port);
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/telegram/r09", web::post().to(receiving_r09))
            .route("/telegram/raw", web::post().to(receiving_raw))
        //.route("/telegram/raw", web::post().to(raw))
    })
    .bind((host, port))?
    .run()
    .await
}
