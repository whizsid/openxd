//! Data structures that using to save data in database

use serde::{Serialize, Deserialize};
use surrealdb::sql::{Id, Thing, Datetime};

#[derive(Serialize, Deserialize)]
pub struct User {
    id: Id,
    name: String,
}

impl User {
    pub const TABLE: &str = "users";
}

/// Data that should be stored in the database related to a session
#[derive(Serialize, Deserialize)]
pub struct Session {
    id: Id,
    user: Thing,
    created_at: Datetime,
    last_activity: Datetime,
}

impl Session {
    pub const TABLE:&str = "sessions";
    /// Creating a session data with an empty to save in DB
    pub fn create(user_id: Id) -> Session {
        Session {
            created_at: Datetime::default(),
            last_activity: Datetime::default(),
            user: Thing {
                tb: String::from(User::TABLE),
                id: user_id,
            },
            id: Id::String(String::from("")),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub id: Id,
    pub name: String,
    pub created_at: Datetime
}

impl Project {
    pub const TABLE: &str = "projects";

    pub fn new(id: Id, name: String) -> Project {
        Project { id, name, created_at: Datetime::default() }
    }
}
