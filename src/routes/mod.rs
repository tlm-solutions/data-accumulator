use super::{Filter, Telegram, DEPULICATION_BUFFER_SIZE, Response, RawData};
use crate::{schema::stations, ClickyBuntyDatabase};
use actix_diesel::{dsl::AsyncRunQueryDsl};
use actix_web::{
    Responder,
};
use crate::diesel::QueryDsl;
use actix_web::{web, HttpRequest};
use std::sync::{RwLock, Mutex};
use std::sync::mpsc::SyncSender;
use std::io::stdout;
use std::io::Write;
use uuid::{Uuid};
use crate::diesel::ExpressionMethods;
use crate::diesel::RunQueryDsl;

#[derive(Queryable, Debug)]
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

// /telegrams/r09/
pub async fn formatted(filter: web::Data<RwLock<Filter>>,
                    sender: web::Data<(Mutex<SyncSender<(Telegram, String)>>, Mutex<SyncSender<(Telegram, String)>>)>,
                    database: web::Data<ClickyBuntyDatabase>,
                    telegram: web::Json<Telegram>, 
                    _req: HttpRequest) -> impl Responder {
    println!("Received MEssage {:?}", &telegram);
    let telegram_hash = Filter::calculate_hash(&telegram).await;
    let contained;
    // checks if the given telegram is already in the buffer
    {
        let readable_filter = filter.read().unwrap();
        contained = readable_filter.last_elements.contains(&telegram_hash);
    }

    if contained {
        return web::Json(Response { success: false })
    }

    // updates the buffer adding the new telegram
    {
        let mut writeable_filter = filter.write().unwrap();
        let index = writeable_filter.iterator;
        writeable_filter.last_elements[index] = telegram_hash;
        writeable_filter.iterator = (writeable_filter.iterator + 1) % DEPULICATION_BUFFER_SIZE;
    }

    println!("Received Telegram: {:?}", &telegram);
    // query database for this station
    let station;
    match (stations::table
        .filter(stations::id.eq(telegram.station_id))
        .get_result_async::<Station>(&database.db)).await {
        Ok(data) => { station = data; }
        Err(e) => {
            println!("Err: {:?}", e);
            return web::Json(Response { success: false })
        }
    };
    println!("Station found: {:?}", &station);

    if station.id != telegram.station_id || station.token != Some(telegram.token.clone()) {
        // authentication for telegram failed !
        return web::Json(Response { success: false })
    }

    println!("[main] Received Telegram! {} {:?}", &telegram.station_id, &telegram);
    match sender.0.lock().unwrap().try_send(((*telegram).clone(), telegram.station_id.to_string().clone())) {
        Err(err) => {
            println!("[main] Channel GRPC has problems! {:?}", err);
        }
        _ => { }
    }
    match sender.1.lock().unwrap().try_send(((*telegram).clone(), telegram.station_id.to_string().clone())) {
        Err(err) => {
            println!("[main] Channel Database has problems! {:?}", err);
        },
        _ => { }
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


