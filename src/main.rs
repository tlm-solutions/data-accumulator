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
pub use routes::{receiving_r09, Station};
use stations::ClickyBuntyDatabase;
pub use storage::{CSVFile, Empty, PostgresDB, Storage};
use structs::Args;

use actix_web::{web, App, HttpServer};
use actix_web::{middleware::Logger};
use clap::Parser;
use tokio::runtime::Builder;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Mutex, RwLock, Arc};
use std::thread;
use std::any::Any;

use dump_dvb::telegrams::{TelegramMetaInformation, r09::R09Telegram };

//pub type DataPipelineSender = SyncSender<(Box<dyn Any>, TelegramMetaInformation)>;

pub type DataPipelineSender = SyncSender<(R09Telegram, TelegramMetaInformation)>;
pub type DataPipelineReceiver = Receiver<(R09Telegram, TelegramMetaInformation)>;

pub struct ApplicationState {
    database: Mutex<ClickyBuntyDatabase>,
    database_sender: Mutex<DataPipelineSender>,
    grpc_sender: Mutex<DataPipelineSender>,
    filter: Mutex<Filter>
}

impl ApplicationState {
    fn new(database_sender: DataPipelineSender, grpc_sender: DataPipelineSender, offline: bool) -> ApplicationState {

        let database_struct;
        if offline {
            database_struct = ClickyBuntyDatabase::offline();
        } else {
            database_struct = ClickyBuntyDatabase::new();
        };

        ApplicationState {
            database: Mutex::new(database_struct),
            database_sender: Mutex::new(database_sender),
            grpc_sender: Mutex::new(grpc_sender),
            filter: Mutex::new(Filter::new())
        }
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

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

    let app_state = web::Data::new(Arc::new(ApplicationState::new(sender_database, sender_grpc, args.offline)));

    println!("Listening on: {}:{}", host, port);
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(app_state.clone())
            .route("/telegram/r09", web::post().to(receiving_r09))
            //.route("/telegram/raw", web::post().to(receiving_raw))
        //.route("/telegram/raw", web::post().to(raw))
    })
    .bind((host, port))?
    .run()
    .await
}
