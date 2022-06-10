use serde::{Serialize, Deserialize};
use diesel_derives::Queryable;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub enum Role {
    Administrator = 0,
    User = 6
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Region {
    pub id: u32,
    pub name: String,
    pub transport_company: String,
    pub frequency: u64,
    pub protocol: String,
}

#[derive(Serialize, Deserialize, Queryable, Debug)]
pub struct Station {
    pub token: Option<String>,
    pub id: u32,
    pub name: String,
    pub lat: f64,
    pub lon: f64,
    pub region: u32,
    pub owner: Uuid,
    pub approved: bool,
}

