#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://stellular.net/static/favicon.svg")]
#![doc(
    html_logo_url = "https://code.stellular.org/repo-avatars/cc8d0efab0759fa6310b75fd5759c33169ee0ab354a958172ed4425a66d2593b"
)]
#![doc(issue_tracker_base_url = "https://code.stellular.org/stellular/dorsal/issues/")]

pub mod config;
pub mod db;
pub mod utility;

// databases
pub use db::cachedb::CacheDB;
pub use db::db::{DefaultReturn, StarterDatabase};
pub use db::special::auth_db::AuthDatabase;
pub use db::special::log_db::LogDatabase;
pub use db::sql::DatabaseOpts;

pub use sqlx::query;

// ...
pub use config::collect_arguments;
pub use config::get_named_argument;
pub use config::get_var;
