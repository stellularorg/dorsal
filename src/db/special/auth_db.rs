use crate::{utility, DefaultReturn, StarterDatabase};
use serde::{Deserialize, Serialize};

// guppy authentication structs
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize)]
/// A user object
pub struct UserState<M> {
    // selectors
    pub username: String,
    pub id_hashed: String, // users use their UNHASHED id to login, it is used as their session id too!
    //                        the hashed id is the only id that should ever be public!
    pub role: String,
    // dates
    pub timestamp: u128,
    // ...
    pub metadata: M,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleLevel {
    /// Marks the level of the role, 0 should always be member
    pub elevation: i32,
    /// Role name, shown on user profiles
    pub name: String,
    /// A list of user permissions (ex: "ManagePastes")
    pub permissions: Vec<String>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullUser<M> {
    pub user: UserState<M>,
    pub level: RoleLevel,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleLevelLog {
    pub id: String,
    pub level: RoleLevel,
}

impl Default for RoleLevelLog {
    fn default() -> Self {
        Self {
            id: String::new(),
            level: RoleLevel {
                name: String::from("member"),
                elevation: 0,
                permissions: Vec::new(),
            },
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMetadata {
    /// User's "about" section (markdown supported)
    pub about: String,
    /// URL of the user's avatar
    pub avatar_url: Option<String>,
    /// Secondary token that the user can login with
    pub secondary_token: Option<String>,
    /// User display name
    pub nickname: Option<String>,
    // pub permissions: Vec<String>,
}

// ...
/// Auth database errors
#[derive(Debug)]
pub enum AuthError {
    ValueError,
    NotFound,
    Banned,
    Other,
}

impl AuthError {
    pub fn to_string(&self) -> String {
        use AuthError::*;
        match self {
            ValueError => String::from("One of the field values given is invalid."),
            NotFound => String::from("User could not be found."),
            Banned => String::from("User is banned."),
            _ => String::from("An unspecified error has occured."),
        }
    }
}

impl<T: Default> Into<DefaultReturn<T>> for AuthError {
    fn into(self) -> DefaultReturn<T> {
        DefaultReturn {
            success: false,
            message: self.to_string(),
            payload: T::default(),
        }
    }
}

pub type Result<T> = std::result::Result<T, AuthError>;

// ...
#[derive(Clone)]
pub struct DatabaseOptions {
    /// The table to for database operations
    pub table: String,
    /// Table used for logs operations
    pub logs_table: String,
}

// database
#[derive(Clone)]
pub struct AuthDatabase {
    pub base: StarterDatabase,
    pub options: DatabaseOptions,
}

impl AuthDatabase {
    pub async fn new(base: StarterDatabase, options: DatabaseOptions) -> AuthDatabase {
        AuthDatabase { base, options }
    }

    // users

    // GET
    /// Get a user by their hashed ID
    ///
    /// # Arguments:
    /// * `hashed` - `String` of the user's hashed ID
    pub async fn get_user_by_hashed(&self, hashed: String) -> Result<FullUser<UserMetadata>> {
        // fetch from database
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            format!(
                "SELECT * FROM \"{}\" WHERE \"id_hashed\" = ?",
                self.options.table
            )
        } else {
            format!(
                "SELECT * FROM \"{}\" WHERE \"id_hashed\" = $1",
                self.options.table
            )
        };

        let c = &self.base.db.client;
        let row = match sqlx::query(&query)
            .bind::<&String>(&hashed)
            .fetch_one(c)
            .await
        {
            Ok(u) => self.base.textify_row(u).data,
            Err(_) => return Err(AuthError::NotFound),
        };

        // ...
        let role = row.get("role").unwrap().to_string();

        if role == "banned" {
            return Err(AuthError::Banned);
        }

        // ...
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role: role.clone(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: match serde_json::from_str(row.get("metadata").unwrap()) {
                Ok(m) => m,
                Err(_) => return Err(AuthError::Banned),
            },
        };

        // fetch level from role
        let level = self.get_level_by_role(role).await;

        // return
        Ok(FullUser {
            user,
            level: level.level,
        })
    }

    /// Get a user by their unhashed ID (hashes ID and then calls [`PawsDB::get_user_by_hashed()`])
    ///
    /// # Arguments:
    /// * `unhashed` - `String` of the user's unhashed ID
    pub async fn get_user_by_unhashed(&self, unhashed: String) -> Result<FullUser<UserMetadata>> {
        match self
            .get_user_by_hashed(utility::hash(unhashed.clone()))
            .await
        {
            Ok(r) => Ok(r),
            Err(_) => self.get_user_by_unhashed_st(unhashed).await,
        }
    }

    /// Get a user by their unhashed secondary token
    ///
    /// # Arguments:
    /// * `unhashed` - `String` of the user's unhashed secondary token
    pub async fn get_user_by_unhashed_st(
        &self,
        unhashed: String,
    ) -> Result<FullUser<UserMetadata>> {
        // fetch from database
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            format!(
                "SELECT * FROM \"{}\" WHERE \"metadata\" LIKE ?",
                self.options.table
            )
        } else {
            format!(
                "SELECT * FROM \"{}\" WHERE \"metadata\" LIKE $1",
                self.options.table
            )
        };

        let c = &self.base.db.client;
        let row = match sqlx::query(&query)
            .bind::<&String>(&format!(
                "%\"secondary_token\":\"{}\"%",
                crate::utility::hash(unhashed)
            ))
            .fetch_one(c)
            .await
        {
            Ok(r) => self.base.textify_row(r).data,
            Err(_) => return Err(AuthError::NotFound),
        };

        // ...
        let role = row.get("role").unwrap().to_string();

        if role == "banned" {
            return Err(AuthError::Banned);
        }

        // ...
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role: role.clone(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: match serde_json::from_str(row.get("metadata").unwrap()) {
                Ok(m) => m,
                Err(_) => return Err(AuthError::ValueError),
            },
        };

        // fetch level from role
        let level = self.get_level_by_role(role).await;

        // return
        Ok(FullUser {
            user,
            level: level.level,
        })
    }

    /// Get a user by their username
    ///
    /// # Arguments:
    /// * `username` - `String` of the user's username
    pub async fn get_user_by_username(&self, username: String) -> Result<FullUser<UserMetadata>> {
        // check in cache
        let cached = self.base.cachedb.get(format!("user:{}", username)).await;

        if cached.is_some() {
            // ...
            let user =
                serde_json::from_str::<UserState<UserMetadata>>(cached.unwrap().as_str()).unwrap();

            // get role
            let role = user.role.clone();

            if role == "banned" {
                // account banned - we're going to act like it simply does not exist
                return Err(AuthError::Banned);
            }

            // fetch level from role
            let level = self.get_level_by_role(role.clone()).await;

            // ...
            return Ok(FullUser {
                user,
                level: level.level,
            });
        }

        // ...
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            format!(
                "SELECT * FROM \"{}\" WHERE \"username\" = ?",
                self.options.table
            )
        } else {
            format!(
                "SELECT * FROM \"{}\" WHERE \"username\" = $1",
                self.options.table
            )
        };

        let c = &self.base.db.client;
        let row = match sqlx::query(&query)
            .bind::<&String>(&username)
            .fetch_one(c)
            .await
        {
            Ok(r) => self.base.textify_row(r).data,
            Err(_) => return Err(AuthError::NotFound),
        };

        // ...
        let role = row.get("role").unwrap().to_string();

        if role == "banned" {
            return Err(AuthError::NotFound);
        }

        // fetch level from role
        let level = self.get_level_by_role(role.clone()).await;

        // store in cache
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role,
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: match serde_json::from_str(row.get("metadata").unwrap()) {
                Ok(m) => m,
                Err(_) => return Err(AuthError::ValueError),
            },
        };

        self.base
            .cachedb
            .set(
                format!("user:{}", username),
                serde_json::to_string::<UserState<UserMetadata>>(&user).unwrap(),
            )
            .await;

        // return
        Ok(FullUser {
            user,
            level: level.level,
        })
    }

    /// Get a [`RoleLevel`] by its `name`
    ///
    /// # Arguments:
    /// * `name` - `String` of the level's role name
    pub async fn get_level_by_role(&self, name: String) -> RoleLevelLog {
        // check if level already exists in cache
        let cached = self.base.cachedb.get(format!("level:{}", name)).await;

        if cached.is_some() {
            return serde_json::from_str::<RoleLevelLog>(cached.unwrap().as_str()).unwrap();
        }

        // ...
        let query: String = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            format!(
                "SELECT * FROM \"{}\" WHERE \"logtype\" = 'level' AND \"content\" LIKE ?",
                self.options.logs_table
            )
        } else {
            format!(
                "SELECT * FROM \"{}\" WHERE \"logtype\" = 'level' AND \"content\" LIKE $1",
                self.options.logs_table
            )
        };

        let c = &self.base.db.client;
        let row = match sqlx::query(&query)
            .bind::<&String>(&format!("%\"name\":\"{}\"%", name))
            .fetch_one(c)
            .await
        {
            Ok(r) => self.base.textify_row(r).data,
            Err(_) => {
                // return default if not found
                return RoleLevelLog::default();
            }
        };

        // store in cache
        let id = row.get("id").unwrap().to_string();
        let level = serde_json::from_str::<RoleLevel>(row.get("content").unwrap()).unwrap();

        let level = RoleLevelLog { id, level };
        self.base
            .cachedb
            .set(
                format!("level:{}", name),
                serde_json::to_string::<RoleLevelLog>(&level).unwrap(),
            )
            .await;

        // return
        return level;
    }
}
