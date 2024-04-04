use dorsal::query;

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
}

impl Database {
    pub async fn new(opts: dorsal::DatabaseOpts) -> Database {
        let db = dorsal::StarterDatabase::new(opts).await;

        Database {
            base: db.clone(),
            auth: dorsal::AuthDatabase { base: db },
        }
    }

    pub async fn init(&self) {
        let c = &self.base.db.client;

        let _ = query(
            "CREATE TABLE IF NOT EXISTS \"ExampleTable\" (
                name VARCHAR(1000000)
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
