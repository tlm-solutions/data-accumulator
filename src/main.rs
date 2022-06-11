
extern crate clap;
extern crate dotenv;
#[macro_use] extern crate diesel;

mod structs;
mod filter;
mod processor;
//mod stations;
mod storage;
mod database;
mod routes;
mod schema;

use structs::{Response, Args};
pub use filter::{Filter, Telegram, RawData, DEPULICATION_BUFFER_SIZE};
use processor::{ProcessorGrpc, ProcessorDatabase};
//pub use stations::{Station};
pub use storage::{SaveTelegram, Storage, InfluxDB, CSVFile};
pub use routes::{formatted, raw, Station};


use actix_diesel::Database;
use actix_web::{web, http::Method, middleware, App, HttpServer};
//use actix_web_async_await::{compat, compat2};
use diesel::pg::PgConnection;
use failure::Error;
use std::time::Duration;
use std::sync::{RwLock, Mutex};
use std::sync::mpsc::TryIter;
use clap::Parser;
use std::sync::mpsc::{SyncSender};
use std::sync::mpsc;
use std::thread;
use std::io::stdout;
use std::io::Write;
use std::ops::Deref;
use std::env;


pub struct ClickyBuntyDatabase {
    db: Database<PgConnection>,
}

impl ClickyBuntyDatabase {
    fn new() -> ClickyBuntyDatabase {
        let default_postgres_host = String::from("localhost:5433");
        let default_postgres_port = String::from("5432");

        let postgres_host = format!(
            "posgresql://dvbdump:{}@{}:{}/dvbdump",
            env::var("POSTGRES_PASSWORD").unwrap(),
            env::var("POSTGRES_HOST").unwrap_or(default_postgres_host.clone()),
            env::var("POSTGRES_PORT").unwrap_or(default_postgres_port.clone())
        );

        println!("Connecting to postgres database {}", &postgres_host);
        let db = Database::builder()
                .pool_max_size(10)
                .pool_min_idle(Some(0))
                .pool_max_lifetime(Some(Duration::from_secs(30 * 60)))
                .open(postgres_host);


        ClickyBuntyDatabase {
            db: db
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let args = Args::parse();

    println!("Starting Data Collection Server ... ");
    let host = args.host.as_str();
    let port = args.port;

    let database_struct = web::Data::new(ClickyBuntyDatabase::new());
    let filter = web::Data::new(RwLock::new(Filter::new()));

    let (sender_database, receiver_database) = mpsc::sync_channel::<(Telegram, String)>(200);
    let (sender_grpc, receiver_grpc) = mpsc::sync_channel::<(Telegram, String)>(200);

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

    let web_database_sender = Mutex::new(sender_database);
    let web_grpc_sender = Mutex::new(sender_grpc);

    let request_data = web::Data::new((web_grpc_sender, web_database_sender));
    println!("Listening on: {}:{}", host, port);
    HttpServer::new(move || App::new()
                    .app_data(filter.clone())
                    .app_data(request_data.clone())
                    .app_data(database_struct.clone())
                    .route("/formatted_telegram", web::post().to(formatted))
                    .route("/telegram/raw/", web::post().to(raw))
                    )
        .bind((host, port))?
        .run()
        .await
}


