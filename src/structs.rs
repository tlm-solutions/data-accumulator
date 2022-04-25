use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Telegram {
    time_stamp: u64,
    lat: f64,
    lon: f64,
    station_id: u32,
    line: u32,
    course_number: u32,
    destination_number: u32,
    pr: u32,
    zv: u32,
    zw: u32,
    mp: u32,
    ha: u32,
    ln: u32,
    kn: u32,
    zn: u32,
    r: u32,
    zl: u32,
    junction: u32,
    junction_number: u32,
    request_status: u32
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
