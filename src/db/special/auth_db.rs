use serde::{Deserialize, Serialize};

use crate::{utility, DefaultReturn, StarterDatabase};

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
    pub elevation: i32, // this marks the level of the role, 0 should always be member
    // users cannot manage users of a higher elevation than them
    pub name: String,             // role name, shown on user profiles
    pub permissions: Vec<String>, // a vec of user permissions (ex: "ManagePastes")
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct FullUser<M> {
    pub user: UserState<M>,
    pub level: RoleLevel,
}

#[derive(Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct RoleLevelLog {
    pub id: String,
    pub level: RoleLevel,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMetadata {
    pub about: String,
    pub avatar_url: Option<String>,
    pub secondary_token: Option<String>,
    pub allow_mail: Option<String>, // yes/no
    pub nickname: Option<String>,   // user display name
}

// database
#[derive(Clone)]
pub struct AuthDatabase {
    pub base: StarterDatabase,
}

impl AuthDatabase {
    pub async fn new(base: StarterDatabase) -> AuthDatabase {
        AuthDatabase { base }
    }

    // users

    // GET
    /// Get a user by their hashed ID
    ///
    /// # Arguments:
    /// * `hashed` - `String` of the user's hashed ID
    pub async fn get_user_by_hashed(
        &self,
        hashed: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        // fetch from database
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Users\" WHERE \"id_hashed\" = ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"id_hashed\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&hashed)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

        let role = row.get("role").unwrap().to_string();
        if role == "banned" {
            return DefaultReturn {
                success: false,
                message: String::from("User is banned"),
                payload: Option::None,
            };
        }

        // ...
        let meta = row.get("metadata"); // for compatability - users did not have metadata until Bundlrs v0.10.6
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role: role.clone(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: if meta.is_some() {
                meta.unwrap().to_string()
            } else {
                String::new()
            },
        };

        // fetch level from role
        let level = self.get_level_by_role(role).await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists"),
            payload: Option::Some(FullUser {
                user,
                level: level.payload.level,
            }),
        };
    }

    /// Get a user by their unhashed ID (hashes ID and then calls [`PawsDB::get_user_by_hashed()`])
    ///
    /// Calls [`PawsDB::get_user_by_unhashed_st()`] if user is invalid.
    ///
    /// # Arguments:
    /// * `unhashed` - `String` of the user's unhashed ID
    pub async fn get_user_by_unhashed(
        &self,
        unhashed: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        let res = self
            .get_user_by_hashed(utility::hash(unhashed.clone()))
            .await;

        if res.success == false {
            // treat unhashed as a secondary token and try again
            return self.get_user_by_unhashed_st(unhashed).await;
        }

        res
    }

    /// Get a user by their unhashed secondary token
    ///
    /// # Arguments:
    /// * `unhashed` - `String` of the user's unhashed secondary token
    pub async fn get_user_by_unhashed_st(
        &self,
        unhashed: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        // fetch from database
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Users\" WHERE \"metadata\" LIKE ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"metadata\" LIKE $1"
        };

        let c = &self.base.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!(
                "%\"secondary_token\":\"{}\"%",
                crate::utility::hash(unhashed)
            ))
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

        let role = row.get("role").unwrap().to_string();
        if role == "banned" {
            return DefaultReturn {
                success: false,
                message: String::from("User is banned"),
                payload: Option::None,
            };
        }

        // ...
        let meta = row.get("metadata"); // for compatability - users did not have metadata until Bundlrs v0.10.6
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role: role.clone(),
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: if meta.is_some() {
                meta.unwrap().to_string()
            } else {
                String::new()
            },
        };

        // fetch level from role
        let level = self.get_level_by_role(role).await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists"),
            payload: Option::Some(FullUser {
                user,
                level: level.payload.level,
            }),
        };
    }

    /// Get a user by their username
    ///
    /// # Arguments:
    /// * `username` - `String` of the user's username
    pub async fn get_user_by_username(
        &self,
        username: String,
    ) -> DefaultReturn<Option<FullUser<String>>> {
        // check in cache
        let cached = self.base.cachedb.get(format!("user:{}", username)).await;

        if cached.is_some() {
            // ...
            let user = serde_json::from_str::<UserState<String>>(cached.unwrap().as_str()).unwrap();

            // get role
            let role = user.role.clone();
            if role == "banned" {
                // account banned - we're going to act like it simply does not exist
                return DefaultReturn {
                    success: false,
                    message: String::from("User is banned"),
                    payload: Option::None,
                };
            }

            // fetch level from role
            let level = self.get_level_by_role(role.clone()).await;

            // ...
            return DefaultReturn {
                success: true,
                message: String::from("User exists (cache)"),
                payload: Option::Some(FullUser {
                    user,
                    level: level.payload.level,
                }),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Users\" WHERE \"username\" = ?"
        } else {
            "SELECT * FROM \"Users\" WHERE \"username\" = $1"
        };

        let c = &self.base.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&username)
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: false,
                message: String::from("User does not exist"),
                payload: Option::None,
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

        let role = row.get("role").unwrap().to_string();
        if role == "banned" {
            // account banned - we're going to act like it simply does not exist
            return DefaultReturn {
                success: false,
                message: String::from("User is banned"),
                payload: Option::None,
            };
        }

        // fetch level from role
        let level = self.get_level_by_role(role.clone()).await;

        // store in cache
        let meta = row.get("metadata");
        let user = UserState {
            username: row.get("username").unwrap().to_string(),
            id_hashed: row.get("id_hashed").unwrap().to_string(),
            role,
            timestamp: row.get("timestamp").unwrap().parse::<u128>().unwrap(),
            metadata: if meta.is_some() {
                meta.unwrap().to_string()
            } else {
                String::new()
            },
        };

        self.base
            .cachedb
            .set(
                format!("user:{}", username),
                serde_json::to_string::<UserState<String>>(&user).unwrap(),
            )
            .await;

        // return
        return DefaultReturn {
            success: true,
            message: String::from("User exists (new)"),
            payload: Option::Some(FullUser {
                user,
                level: level.payload.level,
            }),
        };
    }

    /// Get a [`RoleLevel`] by its `name`
    ///
    /// # Arguments:
    /// * `name` - `String` of the level's role name
    pub async fn get_level_by_role(&self, name: String) -> DefaultReturn<RoleLevelLog> {
        // check if level already exists in cache
        let cached = self.base.cachedb.get(format!("level:{}", name)).await;

        if cached.is_some() {
            return DefaultReturn {
                success: true,
                message: String::from("Level exists (cache)"),
                payload: serde_json::from_str::<RoleLevelLog>(cached.unwrap().as_str()).unwrap(),
            };
        }

        // ...
        let query: &str = if (self.base.db._type == "sqlite") | (self.base.db._type == "mysql") {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'level' AND \"content\" LIKE ?"
        } else {
            "SELECT * FROM \"Logs\" WHERE \"logtype\" = 'level' AND \"content\" LIKE $1"
        };

        let c = &self.base.db.client;
        let res = sqlx::query(query)
            .bind::<&String>(&format!("%\"name\":\"{}\"%", name))
            .fetch_one(c)
            .await;

        if res.is_err() {
            return DefaultReturn {
                success: true,
                message: String::from("Level does not exist, using default"),
                payload: RoleLevelLog {
                    id: String::new(),
                    level: RoleLevel {
                        name: String::from("member"),
                        elevation: 0,
                        permissions: Vec::new(),
                    },
                },
            };
        }

        // ...
        let row = res.unwrap();
        let row = self.base.textify_row(row).data;

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
        return DefaultReturn {
            success: true,
            message: String::from("Level exists (new)"),
            payload: level,
        };
    }
}
