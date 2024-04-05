//! # StarterDatabase
//! Database handler for all database types
use super::{
    cachedb::CacheDB,
    sql::{create_db, Database, DatabaseOpts},
};

use serde::{Deserialize, Serialize};
use sqlx::{Column, Row};
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
/// Default API return value
pub struct DefaultReturn<T> {
    pub success: bool,
    pub message: String,
    pub payload: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseReturn {
    pub data: HashMap<String, String>,
}

#[derive(Clone)]
#[cfg(feature = "postgres")]
pub struct StarterDatabase {
    pub db: Database<sqlx::PgPool>,
    pub options: DatabaseOpts,
    pub cachedb: CacheDB,
}

#[derive(Clone)]
#[cfg(feature = "mysql")]
pub struct StarterDatabase {
    pub db: Database<sqlx::MySqlPool>,
    pub options: DatabaseOpts,
    pub cachedb: CacheDB,
}

#[derive(Clone)]
#[cfg(feature = "sqlite")]
pub struct StarterDatabase {
    pub db: Database<sqlx::SqlitePool>,
    pub options: DatabaseOpts,
    pub cachedb: CacheDB,
}

impl StarterDatabase {
    pub async fn new(options: DatabaseOpts) -> StarterDatabase {
        StarterDatabase {
            db: create_db(options.clone()).await,
            options,
            cachedb: CacheDB::new().await,
        }
    }

    #[cfg(feature = "sqlite")]
    pub fn textify_row(&self, row: sqlx::sqlite::SqliteRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.get(column.name());
            out.insert(column.name().to_string(), value);
        }

        // return
        return DatabaseReturn { data: out };
    }

    #[cfg(feature = "postgres")]
    pub fn textify_row(&self, row: sqlx::postgres::PgRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.get(column.name());
            out.insert(column.name().to_string(), value);
        }

        // return
        return DatabaseReturn { data: out };
    }

    #[cfg(feature = "mysql")]
    pub fn textify_row(&self, row: sqlx::mysql::MySqlRow) -> DatabaseReturn {
        // get all columns
        let columns = row.columns();

        // create output
        let mut out: HashMap<String, String> = HashMap::new();

        for column in columns {
            let value = row.try_get::<Vec<u8>, _>(column.name());

            if value.is_ok() {
                // returned bytes instead of text :(
                // we're going to convert this to a string and then add it to the output!
                out.insert(
                    column.name().to_string(),
                    std::str::from_utf8(value.unwrap().as_slice())
                        .unwrap()
                        .to_string(),
                );
            } else {
                // already text
                let value = row.get(column.name());
                out.insert(column.name().to_string(), value);
            }
        }

        // return
        return DatabaseReturn { data: out };
    }
}
