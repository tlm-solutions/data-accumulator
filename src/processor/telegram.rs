use serde::{Deserialize, Serialize};
use std::hash::{Hash};
use std::hash::Hasher;
use std::convert::From;

#[derive(Deserialize, Serialize, Debug)]
pub struct Telegram {
    pub time_stamp: u64,
    pub lat: f64,
    pub lon: f64,
    pub station_id: u32,
    pub line: u32,
    pub destination_number: u32,
    pub priority: u32,
    pub sign_of_deviation: u32,
    pub value_of_deviation: u32,
    pub reporting_point: u32,
    pub request_for_priority: u32,
    pub run_number: u32,
    pub reserve: u32,
    pub train_length: u32,
    pub junction: u32,
    pub junction_number: u32,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct RawData {
    time_stamp: u64,
    lat: f64,
    lon: f64,
    station_id: u32,
    raw_data: Vec<u8>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ReducedTelegram {
    pub time_stamp: u64,
    pub position_id: u64,
    pub line: u32,
    pub delay: i32,
    pub direction: u8,
    pub destination_number: u32
}

impl Hash for Telegram {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.line.hash(state);
        self.destination_number.hash(state);
        self.priority.hash(state);
        self.sign_of_deviation.hash(state);
        self.value_of_deviation.hash(state);
        self.reporting_point.hash(state);
        self.request_for_priority.hash(state);
        self.run_number.hash(state);
        self.reserve.hash(state);
        self.train_length.hash(state);
        self.junction.hash(state);
        self.junction_number.hash(state);
    }
}

/*impl From<Telegram> for ReducedTelegram {
    fn from(tele: &Telegram) -> Self {
        let delay = (tele.sign_of_deviation * 2 - 1) * tele.value_of_deviation;
        ReducedTelegram {
            time_stamp: tele.time_stamp,
            position_id: tele.reporting_point,
            line: tele.line,
            delay: delay,
            direction: tele.run_number,
            destination_number: tele.destination_number
        }
    }
} */
