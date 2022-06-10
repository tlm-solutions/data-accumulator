
//mod schema;
//mod models;
//mod schema;
//


/*use diesel::prelude::*;
use diesel::pg::PgConnection;
use dotenv::dotenv;
use std::env;
use uuid::Uuid;

//use schema::*;
use models::{Region, User, Station};
use diesel::prelude::*;
//use schema::users::dsl::*;
//use schema::stations::dsl::*;

struct Database {
    connection: PgConnection
}

impl Database {
    pub fn new() -> Database {
        Database {
            connection: Database::establish_connection()
        }
    }

    pub fn establish_connection() -> PgConnection {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
        PgConnection::establish(&database_url)
            .expect(&format!("Error connecting to {}", database_url))
    }

    pub fn allow(&self, token: &String) -> bool {
        true
    }
}


*/
