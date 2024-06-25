use super::log_db::{Log, LogError, Result as LogResult};
use crate::{AuthDatabase, DefaultReturn, LogDatabase, StarterDatabase};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub service: String, // identifier for the service the notification is from
    pub user: String,    // the user that is being notified
    pub content: String, // notification text
    pub address: String, // notification redirect url
}

// ...
/// Notification database errors
#[derive(Debug)]
pub enum NotificationError {
    ValueError,
    NotFound,
    Other,
}

impl NotificationError {
    pub fn to_string(&self) -> String {
        use NotificationError::*;
        match self {
            ValueError => String::from("One of the field values given is invalid."),
            NotFound => String::from("No asset with this selector could be found."),
            _ => String::from("An unspecified error has occured"),
        }
    }
}

impl<T: Default> Into<DefaultReturn<T>> for NotificationError {
    fn into(self) -> DefaultReturn<T> {
        DefaultReturn {
            success: false,
            message: self.to_string(),
            payload: T::default(),
        }
    }
}

pub type Result<T> = std::result::Result<T, NotificationError>;

// database
#[derive(Clone)]
pub struct NotificationDatabase {
    pub base: StarterDatabase,
    pub auth: AuthDatabase,
    pub logs: LogDatabase,
}

impl NotificationDatabase {
    pub async fn new(
        base: StarterDatabase,
        auth: AuthDatabase,
        logs: LogDatabase,
    ) -> NotificationDatabase {
        NotificationDatabase { base, auth, logs }
    }

    // notifications

    // GET
    /// Get the [`Notification`]s that belong to the given `user`
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn get_user_notifications(
        &self,
        user: String,
        offset: Option<i32>,
    ) -> Result<Vec<Log>> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'notification' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'notification' ORDER BY \"timestamp\" DESC LIMIT 50 OFFSET $2"
        };

        let c = &self.base.db.client;
        let rows = match sqlx::query(query)
            .bind::<&String>(&format!("%\"user\":\"{user}\"%"))
            .bind(if offset.is_some() { offset.unwrap() } else { 0 })
            .fetch_all(c)
            .await
        {
            Ok(r) => r,
            Err(_) => return Err(NotificationError::Other),
        };

        // ...
        let mut output: Vec<Log> = Vec::new();

        for row in rows {
            let row = self.base.textify_row(row).data;
            output.push(Log {
                id: row.get("id").unwrap().to_string(),
                logtype: row.get("logtype").unwrap().to_string(),
                timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
                content: row.get("content").unwrap().to_string(),
            });
        }

        // return
        Ok(output)
    }

    /// Check if the given `user` has an active notification
    ///
    /// # Arguments:
    /// * `user` - username of user to check
    pub async fn user_has_notification(&self, user: String) -> Result<bool> {
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE ? AND \"logtype\" = 'notification' LIMIT 1"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"content\" LIKE $1 AND \"logtype\" = 'notification' LIMIT 1"
        };

        let c = &self.base.db.client;
        match sqlx::query(query)
            .bind::<&String>(&format!("%\"user\":\"{user}\"%"))
            .fetch_one(c)
            .await
        {
            Ok(_) => Ok(false),
            Err(_) => Ok(true),
        }
    }

    // SET
    /// Create a new [`Notification`] for a given [`UserState`]
    ///
    /// # Arguments:
    /// * `props` - [`Notification`]
    pub async fn push_user_notification(&self, props: &mut Notification) -> LogResult<()> {
        let p: &mut Notification = props; // borrowed props

        // make sure user exists
        match self.auth.get_user_by_username(p.user.to_owned()).await {
            Ok(ua) => ua,
            Err(_) => return Err(LogError::Other),
        };

        // return
        self.logs
            .create_log(
                String::from("notification"),
                serde_json::to_string::<Notification>(&p).unwrap(),
            )
            .await
    }
}
