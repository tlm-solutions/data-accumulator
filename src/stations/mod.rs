use actix_diesel::Database;
use diesel::pg::PgConnection;
use log::info;

use std::env;

pub struct ClickyBuntyDatabase {
    pub db: Option<Database<PgConnection>>,
}

impl ClickyBuntyDatabase {
    pub fn new() -> ClickyBuntyDatabase {
        let default_postgres_host = String::from("localhost:5433");
        let default_postgres_port = String::from("5432");
        let default_postgres_pw = String::from("default_pw");

        let postgres_host = format!(
            "postgres://dvbdump:{}@{}:{}/dvbdump",
            env::var("POSTGRES_DVBDUMP_PASSWORD").unwrap_or(default_postgres_pw.clone()),
            env::var("POSTGRES_HOST").unwrap_or(default_postgres_host.clone()),
            env::var("POSTGRES_PORT").unwrap_or(default_postgres_port.clone())
        );

        info!("Connecting to postgres database {}", &postgres_host);
        let db = Database::builder().open(postgres_host);

        ClickyBuntyDatabase { 
            db: Some(db) 
        }
    }

    pub fn offline() -> ClickyBuntyDatabase {
        ClickyBuntyDatabase {
            db: None
        }
    }
}
