use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use std::env;

pub mod models;
pub mod schema;

pub fn establish_connection() -> MysqlConnection {
    let database_url = match env::var("DATABASE_URL") {
        Ok(variable) => variable,
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
    MysqlConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
