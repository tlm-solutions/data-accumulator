use super::filter::{Filter, DEPULICATION_BUFFER_SIZE};
use super::{ApplicationState, DbPool};

use crate::diesel::ExpressionMethods;
use crate::diesel::QueryDsl;

use dump_dvb::telegrams::{
    TelegramMetaInformation, 
    AuthenticationMeta,
    r09::R09ReceiveTelegram,
    raw::RawReceiveTelegram
};

use diesel::pg::PgConnection;
use actix_web::Responder;
use actix_web::{web, HttpRequest};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use log::{info, warn, error};

use std::sync::Mutex;
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
    pub deactivated: bool
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    success: bool,
}

fn authenticate(conn: &PgConnection, auth: &AuthenticationMeta, offline: bool) -> Option<(TelegramMetaInformation, bool)> {
    if offline {
        return Some(
            (
                TelegramMetaInformation {
                    time: Utc::now().naive_utc(),
                    station: Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap(),
                    region: -1
                },
                true
            )
        );
    }

    let station;
    {
        use crate::schema::stations::id;
        use crate::schema::stations::dsl::stations;
        use crate::diesel::RunQueryDsl;

        match stations 
            .filter(id.eq(auth.station))
            .first::<Station>(conn) {
            Ok(data) => {
                station = data;
            }
            Err(e) => {
                error!("Err: {:?}", e);
                return None;
            }
        };
    }
    if station.id != auth.station
        || station.token != Some(auth.token.clone())
        || station.deactivated
    {
        // authentication for telegram failed !
        return None;
    }


    Some((TelegramMetaInformation {
        time: Utc::now().naive_utc(),
        station: station.id,
        region: station.region,
    }, station.approved))
}

// checks if the given telegram hash is contained in the filter class
async fn deduplicate(_conn: &PgConnection, filter: &mut Filter, telegram_hash: u64) -> bool {
    // checks if the given telegram is already in the buffer
    let contained = filter.last_elements.contains(&telegram_hash);

    if contained {
        return true;
    }

    // updates the buffer adding the new telegram
    let index = filter.iterator;
    filter.last_elements[index] = telegram_hash;
    filter.iterator = (filter.iterator + 1) % DEPULICATION_BUFFER_SIZE;

    contained
}


// /telegrams/r09/
pub async fn receiving_r09(
    pool: web::Data<DbPool>,
    app_state: web::Data<Mutex<ApplicationState>>,
    telegram: web::Json<R09ReceiveTelegram>,
    _req: HttpRequest,
) -> impl Responder {
    info!("[DEBUG] Station: {} Received Telegram: {:?}", &telegram.auth.station, &telegram.data);

    if app_state.is_poisoned() {
        warn!("cannot unwrap app state because the lock is poisenous");
        return web::Json(Response { success: false });
    }

    let telegram_hash = Filter::calculate_hash(&*telegram).await;
    let conn = pool.get().expect("couldn't get db connection from pool");
    let contained = deduplicate(&conn, &mut(app_state.lock().unwrap().filter.lock().unwrap()), telegram_hash).await;

    if contained {
        return web::Json(Response { success: false });
    }

    info!(
        "[main] Received Telegram! {} {:?}",
        &telegram.auth.station, &telegram
    );

    let meta: TelegramMetaInformation;
    let approved;

    match authenticate(&conn, &telegram.auth, app_state.lock().unwrap().offline) {
        Some((received_meta, received_approved)) => {
            meta = received_meta;
            approved = received_approved;
        }
        None => {
            return web::Json(Response { success: false });
        }
    } 

    if approved {
        match app_state.lock().unwrap().grpc_sender
            .lock()
            .unwrap()
            .try_send(((*telegram).data.clone(), meta.clone()))
        {
            Err(err) => {
                warn!("[main] Channel GRPC has problems! {:?}", err);
            }
            _ => {}
        }
    }

    match app_state.lock().unwrap().database_r09_sender
        .lock()
        .unwrap()
        .try_send((((*telegram).data.clone()), meta))
    {
        Err(err) => {
            warn!("[main] Channel Database has problems! {:?}", err);
        }
        _ => {}
    }

    web::Json(Response { success: true })
}

// /telegrams/raw/
pub async fn receiving_raw(
    pool: web::Data<DbPool>,
    app_state: web::Data<Mutex<ApplicationState>>,
    telegram: web::Json<RawReceiveTelegram>,
    _req: HttpRequest,
) -> impl Responder {
    info!("[DEBUG] Station: {} Received Telegram: {:?}", &telegram.auth.station, &telegram.data);

    if app_state.is_poisoned() {
        warn!("cannot unwrap app state because the lock is poisenous");
        return web::Json(Response { success: false });
    }

    let telegram_hash = Filter::calculate_hash(&*telegram).await;
    let conn = pool.get().expect("couldn't get db connection from pool");
    let contained = deduplicate(&conn, &mut(app_state.lock().unwrap().filter.lock().unwrap()), telegram_hash).await;

    if contained {
        return web::Json(Response { success: false });
    }

    let meta: TelegramMetaInformation;
    match authenticate(&conn, &telegram.auth, false) {
        Some((received_meta, _)) => {
            meta = received_meta;
        }
        None => {
            return web::Json(Response { success: false });
        }
    } 

    match app_state.lock().unwrap().database_raw_sender
        .lock()
        .unwrap()
        .try_send((((*telegram).data.clone()), meta))
    {
        Err(err) => {
            warn!("[main] Channel Database has problems! {:?}", err);
        }
        _ => {}
    }

    web::Json(Response { success: true })
}
