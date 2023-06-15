use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    pub id: Option<Thing>,
    pub created_at: Datetime,
    pub opened_at: Option<Datetime>,
    pub closed_at: Option<Datetime>,
    pub allow_connect_again: bool,
    pub user: Thing,
}

impl Ticket {
    pub const TABLE: &str = "tickets";

    pub fn new(user: Thing) -> Ticket {
        Ticket {
            id: None,
            created_at: Datetime::default(),
            opened_at: None,
            closed_at: None,
            allow_connect_again: false,
            user
        }
    }

    pub fn make_allow_connect_again(&mut self) {
        self.allow_connect_again = true;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SnapshotDownload {
    pub id: Option<Thing>,
    pub created_at: Datetime,
    pub name: String,
    pub snapshot: Thing,
    pub user: Thing,
    pub downloaded_at: Option<Datetime>,
}

impl SnapshotDownload {
    pub const TABLE: &str = "snapshotdownloads";

    pub fn new(snapshot: Thing, user: Thing, name: String) -> SnapshotDownload {
        SnapshotDownload {
            id: None,
            created_at: Datetime::default(),
            snapshot,
            user,
            downloaded_at: None,
            name
        }
    }

    pub fn mark_as_downloaded(&mut self) {
        self.downloaded_at = Some(Datetime::default());
    }
}
