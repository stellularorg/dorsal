use crate::{utility, DefaultReturn, StarterDatabase};
use serde::{Deserialize, Serialize};

#[derive(Default, PartialEq, Clone, Serialize, Deserialize)]
pub struct Log {
    // selectors
    pub id: String,
    pub logtype: String,
    // dates
    pub timestamp: u128,
    // ...
    pub content: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogIdentifier {
    pub id: String,
}

// ...
/// Log database errors
#[derive(Debug)]
pub enum LogError {
    ValueError,
    NotFound,
    Other,
}

impl LogError {
    pub fn to_string(&self) -> String {
        use LogError::*;
        match self {
            ValueError => String::from("One of the field values given is invalid."),
            NotFound => String::from("No asset with this selector could be found."),
            _ => String::from("An unspecified error has occured"),
        }
    }
}

impl<T: Default> Into<DefaultReturn<T>> for LogError {
    fn into(self) -> DefaultReturn<T> {
        DefaultReturn {
            success: false,
            message: self.to_string(),
            payload: T::default(),
        }
    }
}

pub type Result<T> = std::result::Result<T, LogError>;

// database
#[derive(Clone)]
pub struct LogDatabase {
    pub base: StarterDatabase,
}

impl LogDatabase {
    pub async fn new(base: StarterDatabase) -> LogDatabase {
        LogDatabase { base }
    }

    // logs

    // GET
    /// Get a log by its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    pub async fn get_log_by_id(&self, id: String) -> Result<Log> {
        // check in cache
        let cached = self.base.cachedb.get(format!("log:{}", id)).await;

        if cached.is_some() {
            // ...
            let log = serde_json::from_str::<Log>(cached.unwrap().as_str()).unwrap();

            // return
            return Ok(log);
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"id\" = ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"id\" = $1"
        };

        let c = &self.base.db.client;
        let row = match sqlx::query(query).bind::<&String>(&id).fetch_one(c).await {
            Ok(r) => self.base.textify_row(r).data,
            Err(_) => return Err(LogError::Other),
        };

        // store in cache
        let log = Log {
            id: row.get("id").unwrap().to_string(),
            logtype: row.get("logtype").unwrap().to_string(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            content: row.get("content").unwrap().to_string(),
        };

        self.base
            .cachedb
            .set(
                format!("log:{}", id),
                serde_json::to_string::<Log>(&log).unwrap(),
            )
            .await;

        // return
        return Ok(log);
    }

    // SET
    /// Create a log given its type and content
    ///
    /// # Arguments:
    /// * `logtype` - `String` of the log's `logtype`
    /// * `content` - `String` of the log's `content`
    pub async fn create_log(&self, logtype: String, content: String) -> Result<()> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "INSERT INTO \"Logs\" VALUES (?, ?, ?, ?)"
        } else {
            "INSERT INTO \"Logs\" VALUES ($1, $2, $3, $4)"
        };

        let log_id: String = utility::random_id();

        let c = &self.base.db.client;
        match sqlx::query(query)
            .bind::<&String>(&log_id)
            .bind::<String>(logtype)
            .bind::<String>(utility::unix_epoch_timestamp().to_string())
            .bind::<String>(content)
            .execute(c)
            .await
        {
            Ok(_) => return Ok(()),
            Err(_) => return Err(LogError::Other),
        };
    }

    /// Edit a log given its ID
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    /// * `content` - `String` of the log's new content
    pub async fn edit_log(&self, id: String, content: String) -> Result<()> {
        // make sure log exists
        if let Err(e) = self.get_log_by_id(id.clone()).await {
            return Err(e);
        }

        // update log
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "UPDATE \"Logs\" SET \"content\" = ? WHERE \"id\" = ?"
        } else {
            "UPDATE \"Logs\" SET (\"content\") = ($1) WHERE \"id\" = $2"
        };

        let c = &self.base.db.client;
        match sqlx::query(query)
            .bind::<&String>(&content)
            .bind::<&String>(&id)
            .execute(c)
            .await
        {
            Ok(_) => {
                // update cache
                self.base.cachedb.remove(format!("log:{}", id)).await;

                // return
                Ok(())
            }
            Err(_) => Err(LogError::Other),
        }
    }

    /// Delete a log given its id
    ///
    /// # Arguments:
    /// * `id` - `String` of the log's `id`
    pub async fn delete_log(&self, id: String) -> Result<()> {
        // make sure log exists
        if let Err(e) = self.get_log_by_id(id.clone()).await {
            return Err(e);
        };

        // update log
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "DELETE FROM \"Logs\" WHERE \"id\" = ?"
        } else {
            "DELETE FROM \"Logs\" WHERE \"id\" = $1"
        };

        let c = &self.base.db.client;
        match sqlx::query(query).bind::<&String>(&id).execute(c).await {
            Ok(_) => {
                // update cache
                self.base.cachedb.remove(format!("log:{}", id)).await;

                // return
                return Ok(());
            }
            Err(_) => return Err(LogError::Other),
        }
    }
}
