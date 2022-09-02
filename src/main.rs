extern crate diesel;
extern crate clap;
extern crate r2d2;

mod filter;
mod processor;
mod routes;
mod structs;

use processor::ProcessorGrpc;
pub use routes::{receiving_r09, receiving_raw};
use structs::Args;

use actix_web::{web, App, HttpServer};
use clap::Parser;
use tokio::runtime::Builder;
use env_logger;
use log::{info, debug};
use diesel::{PgConnection, r2d2::ConnectionManager};
use r2d2::Pool;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Mutex, Arc};
use std::thread;
use std::env;

use dump_dvb::telegrams::{TelegramMetaInformation, r09::R09Telegram};

pub type DataPipelineSenderR09 = SyncSender<(R09Telegram, TelegramMetaInformation)>;
pub type DataPipelineReceiverR09 = Receiver<(R09Telegram, TelegramMetaInformation)>;

pub struct ApplicationState {
    grpc_sender: Arc<Mutex<DataPipelineSenderR09>>,
}

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

impl ApplicationState {
    fn new(grpc_sender: Arc<Mutex<DataPipelineSenderR09>>) -> ApplicationState {
        ApplicationState {
            grpc_sender: grpc_sender
        }
    }
}

pub fn create_db_pool() -> DbPool { 
    let default_postgres_host = String::from("localhost:5433");
    let default_postgres_port = String::from("5432");
    let default_postgres_pw = String::from("default_pw");

    let database_url = format!(
        "postgres://dvbdump:{}@{}:{}/dvbdump",
        env::var("POSTGRES_DVBDUMP_PASSWORD").unwrap_or(default_postgres_pw.clone()),
        env::var("POSTGRES_HOST").unwrap_or(default_postgres_host.clone()),
        env::var("POSTGRES_PORT").unwrap_or(default_postgres_port.clone())
    );

    debug!("Connecting to postgres database {}", &database_url);
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::new(manager).expect("Failed to create pool.")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    env_logger::init();

    info!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    let (sender_grpc, receiver_grpc) =
        mpsc::sync_channel::<(R09Telegram, TelegramMetaInformation)>(200);

    thread::spawn(move || {
        let rt = Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .expect("cannot spawn processor grpc");
        let mut processor_grpc = ProcessorGrpc::new(receiver_grpc);
        rt.block_on(processor_grpc.process_grpc());
    });

    let arc_sender_grpc = Arc::new(Mutex::new(sender_grpc));
    let postgres_pool = web::Data::new(create_db_pool());

    debug!("Listening on: {}:{}", host, port);
    HttpServer::new(move || {
        let app_state = web::Data::new(Mutex::new(ApplicationState::new(
            arc_sender_grpc.clone()
        )));

        App::new()
            .app_data(postgres_pool.clone())
            .app_data(app_state)
            .route("/telegram/r09", web::post().to(receiving_r09))
            .route("/telegram/raw", web::post().to(receiving_raw))
    })
    .bind((host, port))?
    .run()
    .await
}
