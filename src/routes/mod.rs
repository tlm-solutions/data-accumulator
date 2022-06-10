use super::{Filter, Telegram, DEPULICATION_BUFFER_SIZE, Response, RawData};
use serde::{Serialize};
use crate::{schema::stations, ClickyBuntyDatabase};
use actix_diesel::{dsl::AsyncRunQueryDsl, AsyncError};
use actix_web::{
    error::{ErrorInternalServerError, ErrorNotFound},
    HttpResponse, web::Json, web::Path, Responder, Result,
};
use crate::diesel::QueryDsl;
use actix_web::{web, App, HttpServer, HttpRequest};
use std::sync::{RwLock, Mutex};
use std::sync::mpsc::TryIter;
use clap::Parser;
use std::sync::mpsc::{SyncSender};
use std::sync::mpsc;
use std::thread;
use std::io::stdout;
use std::io::Write;
use std::ops::Deref;
//use diesel::types::Uuid;
//use diesel::sql_types::*;
use uuid::{Uuid};
//use diesel::pg::types::sql_types::Uuid;
//use diesel::sql_types::Uuid;
use diesel::prelude::*;
use crate::diesel::ExpressionMethods;

#[derive(Queryable)]
pub struct Station {
    pub id: Uuid,
    pub token: Option<String>,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub region: i32,
    pub owner: Uuid,
    pub approved: bool
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

// /telegrams/r09/
pub async fn formatted(filter: web::Data<RwLock<Filter>>,
                    sender: web::Data<(Mutex<SyncSender<(Telegram, String)>>, Mutex<SyncSender<(Telegram, String)>>)>,
                    database: web::Data<ClickyBuntyDatabase>,
                    telegram: web::Json<Telegram>, 
                    req: HttpRequest) -> impl Responder {
    let token = String::from("");
    let station_id = Uuid::new_v4();

    let database_query = stations::table
        .filter(stations::id.eq(station_id))
        .get_result_async::<Station>(&database.db);

    print_type_of(&database_query);

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

        println!("[main] Received Telegram! {} {:?}", &station_id, &telegram);
        stdout().flush();
        match sender.0.lock().unwrap().try_send(((*telegram).clone(), station_id.to_string().clone())) {
            Err(err) => {
                println!("[main] Channel GRPC has problems! {:?}", err);
                stdout().flush();
            }
            _ => { }
        }
        match sender.1.lock().unwrap().try_send(((*telegram).clone(), station_id.to_string().clone())) {
            Err(err) => {
                println!("[main] Channel Database has problems! {:?}", err);
                stdout().flush();
            },
            _ => { }
        }
    }

    web::Json(Response { success: true })
}

pub async fn raw(telegram: web::Json<RawData>) -> impl Responder {
    //let default_file = String::from("/var/lib/data-accumulator/raw_data.csv");
    //let csv_file = env::var("PATH_RAW_DATA").unwrap_or(default_file);

    println!("Received Formatted Record: {:?}", &telegram);
    stdout().flush();
    //Processor::dump_to_file(&csv_file, &telegram).await;

    web::Json(Response { success: true })
}


