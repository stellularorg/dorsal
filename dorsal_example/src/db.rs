use dorsal::query as sqlquery;

#[derive(Clone)]
pub struct AppData {
    pub db: Database,
    pub http_client: awc::Client,
}

// server
#[derive(Clone)]
pub struct Database {
    pub base: dorsal::StarterDatabase,
    pub auth: dorsal::AuthDatabase,
    pub logs: dorsal::LogDatabase,
    pub notifications: dorsal::NotificationDatabase,
}

impl Database {
    pub async fn new(opts: dorsal::DatabaseOpts) -> Database {
        let db = dorsal::StarterDatabase::new(opts).await;

        let auth = dorsal::AuthDatabase { base: db.clone() };
        let logs = dorsal::LogDatabase { base: db.clone() };

        Database {
            base: db.clone(),
            auth: auth.clone(),
            logs: logs.clone(),
            notifications: dorsal::NotificationDatabase {
                base: db,
                auth,
                logs,
            },
        }
    }

    pub async fn init(&self) {
        let c = &self.base.db.client;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"ExampleTable\" (
                name VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        // users and logs tables
        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Users\" (
                username VARCHAR(1000000),
                id_hashed VARCHAR(1000000),
                role VARCHAR(1000000),
                timestamp VARCHAR(1000000),
                metadata VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;

        let _ = sqlquery(
            "CREATE TABLE IF NOT EXISTS \"Logs\" (
                id VARCHAR(1000000),
                logtype VARCHAR(1000000),
                timestamp  VARCHAR(1000000),
                content VARCHAR(1000000)
            )",
        )
        .execute(c)
        .await;
    }

    // example

    // GET
    // ...

    // SET
    // ...
}
