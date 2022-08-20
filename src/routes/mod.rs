use super::filter::{Filter, DEPULICATION_BUFFER_SIZE};
use super::DataPipelineSender;

use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;
use crate::{schema::stations, ClickyBuntyDatabase};

use dump_dvb::telegrams::{
    TelegramMetaInformation, 
    r09::R09ReceiveTelegram,
    raw::RawReceiveTelegram,
};

use actix_diesel::dsl::AsyncRunQueryDsl;
use actix_web::Responder;
use actix_web::{web, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use std::sync::{Mutex, RwLock, Arc};
use chrono::Utc;

#[derive(Queryable, Debug, Clone)]
pub struct Station {
    pub id: Uuid,
    pub token: Option<String>,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub region: i32,
    pub owner: Uuid,
    pub approved: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    success: bool,
}

// /telegrams/r09/
pub async fn receiving_r09(
    filter: web::Data<Arc<RwLock<Filter>>>,
    sender: web::Data<Arc<(Mutex<DataPipelineSender>, Mutex<DataPipelineSender>)>>,
    database: web::Data<Arc<Mutex<ClickyBuntyDatabase>>>,
    telegram: web::Json<R09ReceiveTelegram>,
    _req: HttpRequest,
) -> impl Responder {
    println!("[DEBUG] Received Telegram: {:?}", &telegram);
    let telegram_hash = Filter::calculate_hash(&*telegram).await;
    let contained;
    // checks if the given telegram is already in the buffer
    {
        let readable_filter = filter.read().unwrap();
        contained = readable_filter.last_elements.contains(&telegram_hash);
    }

    if contained {
        return web::Json(Response { success: false });
    }
    
    // updates the buffer adding the new telegram
    {
        let mut writeable_filter = filter.write().unwrap();
        let index = writeable_filter.iterator;
        writeable_filter.last_elements[index] = telegram_hash;
        writeable_filter.iterator = (writeable_filter.iterator + 1) % DEPULICATION_BUFFER_SIZE;
    }

    let meta: TelegramMetaInformation;
    if database.lock().unwrap().db.is_none() {
        let station;
        {
            // query database for this station
            match (stations::table
                .filter(stations::id.eq(telegram.auth.station))
                .get_result_async::<Station>(&database.lock().unwrap().db.as_ref().unwrap()))
            .await
            {
                Ok(data) => {
                    station = data;
                }
                Err(e) => {
                    println!("Err: {:?}", e);
                    return web::Json(Response { success: false });
                }
            };
        }
        if station.id != telegram.auth.station
            || station.token != Some(telegram.auth.token.clone())
            || !station.approved
        {
            // authentication for telegram failed !
            return web::Json(Response { success: false });
        }
        meta = TelegramMetaInformation {
            time: Utc::now().naive_utc(),
            station: station.id,
            region: station.region,
        };
        println!(
            "[main] Received Telegram! {} {:?}",
            &telegram.auth.station, &telegram
        );

    } else {
        // offline flag is set throw data out unauthenticated
        meta = TelegramMetaInformation {
            time: Utc::now().naive_utc(),
            station: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            region: -1 //TODO: change
        }
    }

    match sender
        .0
        .lock()
        .unwrap()
        .try_send((Box::new((*telegram).data.clone()), meta.clone()))
    {
        Err(err) => {
            println!("[main] Channel GRPC has problems! {:?}", err);
        }
        _ => {}
    }
    match sender
        .1
        .lock()
        .unwrap()
        .try_send((Box::new((*telegram).data.clone()), meta))
    {
        Err(err) => {
            println!("[main] Channel Database has problems! {:?}", err);
        }
        _ => {}
    }

    web::Json(Response { success: true })
}

pub async fn receiving_raw(
    filter: web::Data<Arc<RwLock<Filter>>>,
    sender: web::Data<Arc<(Mutex<DataPipelineSender>, Mutex<DataPipelineSender>)>>,
    database: web::Data<Arc<Mutex<ClickyBuntyDatabase>>>,
    telegram: web::Json<RawReceiveTelegram>,
    _req: HttpRequest,
) -> impl Responder {
    println!("[DEBUG] Received Telegram: {:?}", &telegram);
    let telegram_hash = Filter::calculate_hash(&*telegram).await;
    let contained;
    // checks if the given telegram is already in the buffer
    {
        let readable_filter = filter.read().unwrap();
        contained = readable_filter.last_elements.contains(&telegram_hash);
    }

    if contained {
        return web::Json(Response { success: false });
    }
    
    // updates the buffer adding the new telegram
    {
        let mut writeable_filter = filter.write().unwrap();
        let index = writeable_filter.iterator;
        writeable_filter.last_elements[index] = telegram_hash;
        writeable_filter.iterator = (writeable_filter.iterator + 1) % DEPULICATION_BUFFER_SIZE;
    }

    let meta: TelegramMetaInformation;

    if database.lock().unwrap().db.is_none() {
        let station;
        {
            // query database for this station
            match (stations::table
                .filter(stations::id.eq(telegram.auth.station))
                .get_result_async::<Station>(&database.lock().unwrap().db.as_ref().unwrap()))
            .await
            {
                Ok(data) => {
                    station = data;
                }
                Err(e) => {
                    println!("Err: {:?}", e);
                    return web::Json(Response { success: false });
                }
            };
        }
        if station.id != telegram.auth.station
            || station.token != Some(telegram.auth.token.clone())
            || !station.approved
        {
            // authentication for telegram failed !
            return web::Json(Response { success: false });
        }
        meta = TelegramMetaInformation {
            time: Utc::now().naive_utc(),
            station: station.id,
            region: station.region,
        };
        println!(
            "[main] Received Telegram! {} {:?}",
            &telegram.auth.station, &telegram
        );

    } else {
        // offline flag is set throw data out unauthenticated
        meta = TelegramMetaInformation {
            time: Utc::now().naive_utc(),
            station: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
            region: -1 //TODO: change
        }
    }

    match sender
        .1
        .lock()
        .unwrap()
        .try_send((Box::new((*telegram).data.clone()), meta))
    {
        Err(err) => {
            println!("[main] Channel Database has problems! {:?}", err);
        }
        _ => {}
    }

    web::Json(Response { success: true })
}

