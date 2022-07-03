use super::{DataPipelineSender};
use super::filter::{Filter, DEPULICATION_BUFFER_SIZE};

use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;
use crate::{schema::stations, ClickyBuntyDatabase};

use telegrams::{R09ReceiveTelegram, TelegramMetaInformation};

use actix_diesel::dsl::AsyncRunQueryDsl;
use actix_web::Responder;
use actix_web::{web, HttpRequest};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use std::time::SystemTime;
use std::sync::{Mutex, RwLock};

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
    success: bool
}

// /telegrams/r09/
pub async fn formatted(
    filter: web::Data<RwLock<Filter>>,
    sender: web::Data<(
        Mutex<DataPipelineSender>,
        Mutex<DataPipelineSender>,
    )>,
    database: web::Data<ClickyBuntyDatabase>,
    telegram: web::Json<R09ReceiveTelegram>,
    _req: HttpRequest,
) -> impl Responder {
    let telegram_hash = Filter::calculate_hash(&telegram).await;
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
    // query database for this station
    let station;
    match (stations::table
        .filter(stations::id.eq(telegram.auth.station))
        .get_result_async::<Station>(&database.db))
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

    if station.id != telegram.auth.station
        || station.token != Some(telegram.auth.token.clone())
        || !station.approved
    {
        // authentication for telegram failed !
        return web::Json(Response { success: false });
    }

    let meta = TelegramMetaInformation {
        time: SystemTime::now(),
        station: station.id,
        region: station.region as u64,
        telegram_type: telegram.auth.telegram_type
    };

    println!(
        "[main] Received Telegram! {} {:?}",
        &telegram.auth.station, &telegram
    );
    match sender
        .0
        .lock()
        .unwrap()
        .try_send(((*telegram).data.clone(), meta.clone()))
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
        .try_send(((*telegram).data.clone(), meta))
    {
        Err(err) => {
            println!("[main] Channel Database has problems! {:?}", err);
        }
        _ => {}
    }

    web::Json(Response { success: true })
}

/*
pub async fn raw(telegram: web::Json<RawData>) -> impl Responder {
    //let default_file = String::from("/var/lib/data-accumulator/raw_data.csv");
    //let csv_file = env::var("PATH_RAW_DATA").unwrap_or(default_file);

    println!("Received Formatted Record: {:?}", &telegram);
    stdout().flush();
    //Processor::dump_to_file(&csv_file, &telegram).await;

    web::Json(Response { success: true })
}
*/

