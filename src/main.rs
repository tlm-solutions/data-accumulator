extern crate clap;
extern crate diesel;
extern crate r2d2;

mod filter;
mod processor;
mod routes;
mod structs;

use processor::ProcessorGrpc;
pub use routes::{receiving_r09, receiving_raw};
use structs::Args;

use actix_web::{web, App, HttpServer};
use actix_web_prom::{PrometheusMetrics, PrometheusMetricsBuilder};
use clap::Parser;
use diesel::{r2d2::ConnectionManager, PgConnection};
use log::{debug, info};
use r2d2::Pool;
use tokio::runtime::Builder;

use std::env;
use std::fs;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SyncSender};
use std::sync::{Arc, Mutex};
use std::thread;

use tlms::telegrams::{r09::R09Telegram, TelegramMetaInformation};

pub type DataPipelineSenderR09 = SyncSender<(R09Telegram, TelegramMetaInformation)>;
pub type DataPipelineReceiverR09 = Receiver<(R09Telegram, TelegramMetaInformation)>;

/// Struct which holds the channels to the grpc sender
pub struct ApplicationState {
    grpc_sender: Arc<Mutex<DataPipelineSenderR09>>,
}

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

impl ApplicationState {
    fn new(grpc_sender: Arc<Mutex<DataPipelineSenderR09>>) -> ApplicationState {
        ApplicationState { grpc_sender }
    }
}

/// connects to postgres database
/// default uri: postgres://dvbdump:{password}@localhost:5432/dvbdump
/// where the password is read from /run/secrets/postgres_password
pub fn create_db_pool() -> DbPool {
    let default_postgres_host = String::from("localhost");
    let default_postgres_port = String::from("5432");
    let default_postgres_user = String::from("datacare");
    let default_postgres_database = String::from("tlms");
    let default_postgres_pw_path = String::from("/run/secrets/postgres_password");

    let password_path = env::var("POSTGRES_PASSWORD_PATH").unwrap_or(default_postgres_pw_path);
    let password = fs::read_to_string(password_path).expect("cannot read password file!");

    let database_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        env::var("POSTGRES_USER").unwrap_or(default_postgres_user),
        password,
        env::var("POSTGRES_HOST").unwrap_or(default_postgres_host),
        env::var("POSTGRES_PORT").unwrap_or(default_postgres_port),
        env::var("POSTGRES_DATABASE").unwrap_or(default_postgres_database)
    );

    debug!("Connecting to postgres database {}", &database_url);
    let manager = ConnectionManager::<PgConnection>::new(database_url);

    Pool::new(manager).expect("Failed to create pool.")
}

pub fn get_prometheus() -> PrometheusMetrics {
    PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        //.const_labels(None)
        .build()
        .unwrap()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();
    env_logger::init();

    info!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    // creates the grpc channel
    let (sender_grpc, receiver_grpc) =
        mpsc::sync_channel::<(R09Telegram, TelegramMetaInformation)>(200);

    // starts the grpc sending thread
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
        let app_state = web::Data::new(Mutex::new(ApplicationState::new(arc_sender_grpc.clone())));
        let prometheus = get_prometheus();

        App::new()
            .wrap(prometheus)
            .app_data(postgres_pool.clone())
            .app_data(app_state)
            .route("/telegram/r09", web::post().to(receiving_r09))
            .route("/telegram/raw", web::post().to(receiving_raw))
    })
    .bind((host, port))?
    .run()
    .await
}
