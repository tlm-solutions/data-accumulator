use serde::{Deserialize, Serialize};

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
pub struct Response {
    pub success: bool,
}
