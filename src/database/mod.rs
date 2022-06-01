extern crate postgres;

use postgres::{Client, NoTls};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::cmp::PartialEq;
use std::env;
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    User = 6,
    Administrator = 0,
}

impl Role {
    pub fn from(role: u32) -> Role {
        match role {
            0 => Role::Administrator,
            _ => Role::User,
        }
    }

    pub fn as_int(&self) -> u32 {
        match self {
            Role::Administrator => 0,
            _ => 6,
        }
    }
}

pub struct User {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password: String,
    pub role: Role,
}

impl User {
    pub fn is_admin(&self) -> bool {
        self.role == Role::Administrator
    }
}

#[derive(Serialize)]
pub struct Region {
    pub id: u32,
    pub name: String,
    pub transport_company: String,
    pub frequency: u64,
    pub protocol: String,
}

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

pub struct DataBaseConnection {
    postgres: Client,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("User", 4)?;
        s.serialize_field("id", &self.id.to_string())?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("email", &self.email)?;
        s.serialize_field("password", &self.password)?;
        s.end()
    }
}

impl Serialize for Station {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Station", 8).unwrap();
        s.serialize_field("token", &self.token)?;
        s.serialize_field("id", &self.id)?;
        s.serialize_field("name", &self.name)?;
        s.serialize_field("lat", &self.lat)?;
        s.serialize_field("lon", &self.lon)?;
        s.serialize_field("region", &self.region)?;
        s.serialize_field("owner", &self.owner.to_string())?;
        s.serialize_field("approved", &self.approved)?;
        s.end()
    }
}

impl DataBaseConnection {
    pub async fn new() -> DataBaseConnection {
        let default_postgres_host = String::from("localhost:5433");

        let postgres_host = format!(
            "posgresql://dvbdump:{}@{}/dvbdump",
            env::var("POSTGRES_PASSWORD").unwrap(),
            env::var("POSTGRES").unwrap_or(default_postgres_host)
        );

        println!("Connecting to Database at {}", postgres_host);
        let mut database = DataBaseConnection {
            postgres: Client::connect(&postgres_host, NoTls).unwrap(),
        };

        database.create_tables().await;

        return database;
    }

    pub async fn create_tables(&mut self) {
        self.postgres
            .execute(
                "CREATE TABLE users (
                    id              UUID PRIMARY KEY,
                    name            TEXT NOT NULL,
                    email           TEXT NOT NULL,
                    password        VARCHAR(32) NOT NULL,
                    role            INT NOT NULL
                  )",
                &[],
            )
            .unwrap();

        self.postgres
            .execute(
                "CREATE TABLE regions (
                    id              SERIAL PRIMARY KEY,
                    name            TEXT NOT NULL,
                    transport_company TEXT NOT NULL,
                    frequency       INT NOT NULL,
                    protocol        TEXT NOT NULL
                  )",
                &[],
            )
            .unwrap();

        self.postgres
            .execute(
                "CREATE TABLE stations (
                    id              SERIAL PRIMARY KEY,
                    token           VARCHAR(32),
                    name            TEXT NOT NULL,
                    lat             DOUBLE NOT NULL CONSTRAINT lat <= 180 CONSTRAINT lat >= -180,
                    lon             DOUBLE NOT NULL CONSTRAINT lon <= 90 CONSTRAINT lon >= -90,
                    region          SERIAL REFERENCES regions (id) NOT NULL,
                    owner           UUID REFERENCES users (id) NOT NULL,
                    approved        BOOLEAN NOT NULL
                  )",
                &[],
            )
            .unwrap();
    }

    pub async fn query_station(&mut self, token: &u32) -> Option<Station> {
        match self.postgres.query_one(
            "SELECT token, id, name, lat, lon, region, owner, approved FROM stations WHERE id=$1",
            &[&token],
        ) {
            Ok(data) => Some(Station {
                token: Some(data.get(0)),
                id: data.get(1),
                name: data.get(2),
                lat: data.get(3),
                lon: data.get(4),
                region: data.get(5),
                owner: Uuid::parse_str(data.get(6)).unwrap(),
                approved: data.get(7),
            }),
            _ => None,
        }
    }

    pub async fn query_region(&mut self, id: &u32) -> Option<Region> {
        match self.postgres.query_one(
            "SELECT id, name, transport_company, frequency, protocol FROM stations WHERE id=$1",
            &[id],
        ) {
            Ok(data) => Some(Region {
                id: data.get(0),
                name: data.get(1),
                transport_company: data.get(2),
                frequency: data.get::<usize, i64>(3) as u64,
                protocol: data.get(4),
            }),
            _ => None,
        }
    }
    pub async fn query_user(&mut self, name: &String) -> Option<User> {
        match self.postgres.query_one(
            "SELECT id, name, email, password FROM users WHERE name=$1",
            &[&name],
        ) {
            Ok(data) => Some(User {
                id: Uuid::parse_str(data.get(0)).unwrap(),
                name: data.get(1),
                email: data.get(2),
                password: data.get(3),
                role: Role::from(data.get(4)),
            }),
            _ => None,
        }
    }

    pub async fn query_user_by_id(&mut self, id: &String) -> Option<User> {
        match self.postgres.query_one(
            "SELECT id, name, email, password FROM users WHERE id=$1",
            &[id],
        ) {
            Ok(data) => Some(User {
                id: Uuid::parse_str(data.get(0)).unwrap(),
                name: data.get(1),
                email: data.get(2),
                password: data.get(3),
                role: Role::from(data.get(4)),
            }),
            _ => None,
        }
    }
    pub async fn check_region_exists(&mut self, id: u32) -> bool {
        match self
            .postgres
            .query_one("SELECT 1 FROM regions WHERE id = $1", &[&id])
        {
            Ok(_) => true,
            _ => false,
        }
    }

    pub async fn list_stations(
        &mut self,
        owner: Option<String>,
        region: Option<u32>,
    ) -> Vec<Station> {
        let mut station_list: Vec<Station> = Vec::new();
        let argumnet_count = owner.clone().map_or_else(|| 0, |_| 1) + region.map_or_else(|| 0, |_| 1);

        let owner_query = owner.clone().map_or_else(
            || String::from(""),
            |_| format!("WHERE owner=${} ", argumnet_count - 1),
        );
        let region_query = region.map_or_else(
            || String::from(""),
            |_| format!("WHERE region=${}", argumnet_count),
        );

        let query = format!(
            "SELECT id, name, lat, lon, region, owner, approved FROM stations {}{}",
            owner_query, region_query
        );

        let results;

        println!("Query {}", &query);
        if owner.is_some() && region.is_some() {
            results = self
                .postgres
                .query(&query, &[&owner.unwrap().to_string(), &region.unwrap()])
                .unwrap();
        } else if owner.is_some() {
            results = self
                .postgres
                .query(&query, &[&owner.unwrap().to_string()])
                .unwrap();
        } else if region.is_some() {
            results = self.postgres.query(&query, &[&region.unwrap()]).unwrap();
        } else {
            results = self.postgres.query(&query, &[]).unwrap();
        }

        for row in results {
            station_list.push(Station {
                id: row.get(0),
                token: None,
                name: row.get(1),
                lat: row.get(2),
                lon: row.get(3),
                region: row.get(4),
                owner: Uuid::parse_str(row.get(5)).unwrap(),
                approved: row.get(6),
            });
        }

        station_list
    }

    pub async fn list_regions(&mut self) -> Vec<Region> {
        let mut results = Vec::new();
        for row in self
            .postgres
            .query(
                "SELECT id, name, transport_company, frequency, protocol FROM regions",
                &[],
            )
            .unwrap()
        {
            results.push(Region {
                id: row.get(0),
                name: row.get(1),
                transport_company: row.get(2),
                frequency: row.get::<usize, i64>(3) as u64,
                protocol: row.get(4),
            });
        }
        results
    }

    pub async fn create_user(&mut self, user: &User) -> bool {
        self.postgres
            .execute(
                "INSERT INTO users (id, name, email, password, role) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &user.id.to_string(),
                    &user.name,
                    &user.email,
                    &user.password,
                    &user.role.as_int(),
                ],
            )
            .is_ok()
    }

    pub async fn create_region(&mut self, user: &Region) -> bool {
        self.postgres
            .execute(
                "INSERT INTO regions (id, name, transport_company, frequency, protocol) VALUES ($1, $2, $3, $4, $5)",
                &[
                    &user.id.to_string(),
                    &user.name,
                    &user.transport_company,
                    &(user.frequency as i64),
                    &user.protocol,
                ],
            )
            .is_ok()
    }

    pub async fn create_station(&mut self, station: &Station) -> bool {
        self.postgres.execute(
            "INSERT INTO users (token, name, lat, lon, region, owner, approved) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            &[
                &station.token,
                &station.name,
                &station.lat,
                &station.lon,
                &station.region,
                &station.owner.to_string(),
                &station.approved
            ],
        ).is_ok()
    }

    pub async fn first_user(&mut self) -> bool {
        match self.postgres.query_one("SELECT 1 FROM users", &[]) {
            Ok(_) => true,
            _ => false,
        }
    }

    pub async fn is_administrator(&mut self, uid: &String) -> bool {
        match self
            .postgres
            .query_one("SELECT role FROM users WHERE id = $1", &[uid])
        {
            Ok(row) => row.get::<usize, i32>(0) == 0,
            _ => false,
        }
    }

    pub async fn get_owner_from_station(&mut self, region_id: &u32) -> Option<String> {
        match self.query_station(region_id).await {
            Some(region) => Some(region.owner.to_string()),
            _ => None,
        }
    }

    pub async fn delete_user(&mut self, uid: &String) -> bool {
        self.postgres
            .execute("DELETE FROM users WHERE id=$1", &[uid])
            .is_ok()
    }

    pub async fn delete_region(&mut self, id: &u32) -> bool {
        self.postgres
            .execute("DELETE FROM users WHERE id=$1", &[id])
            .is_ok()
    }

    pub async fn delete_station(&mut self, id: &u32) -> bool {
        self.postgres
            .execute("DELETE FROM users WHERE id=$1", &[id])
            .is_ok()
    }

    pub async fn update_user(&mut self, user: &User) -> bool {
        self.postgres
            .execute(
                "UPDATE users SET name=$1, email=$2, password=$3, role=$4 WHERE id=$5",
                &[
                    &user.name,
                    &user.email,
                    &user.password,
                    &user.role.as_int(),
                    &user.id.to_string(),
                ],
            )
            .is_ok()
    }

    pub async fn update_station(&mut self, station: &Station) -> bool {
        self.postgres
            .execute(
                "UPDATE station SET name=$1, lat=$2, lon=$3, region=$4 WHERE id=$5",
                &[
                    &station.name,
                    &station.lat,
                    &station.lon,
                    &station.region,
                    &station.id,
                ],
            )
            .is_ok()
    }

    pub async fn update_region(&mut self, region: &Region) -> bool {
        self.postgres.execute("UPDATE region SET name=$1, transport_company=$2, frequency=$3, protocol=$4 WHERE id=$5",
                              &[&region.name, &region.transport_company, &(region.frequency as i64), &region.protocol, &(region.id as i64)]).is_ok()
    }

    pub async fn set_approved(&mut self, id: &u32, approved: bool) -> bool {
        self.postgres
            .execute(
                "UPDATE station SET approved=$1 WHERE id=$2",
                &[&approved, id],
            )
            .is_ok()
    }

    pub async fn set_token(&mut self, id: &u32, token: &String) -> bool {
        self.postgres
            .execute("UPDATE station SET token=$1 WHERE id=$2", &[token, id])
            .is_ok()
    }
}
