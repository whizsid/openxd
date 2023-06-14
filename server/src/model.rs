use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing};

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    pub id: Option<Thing>,
    pub created_at: Datetime,
    pub opened_at: Option<Datetime>,
    pub closed_at: Option<Datetime>,
    pub exited_at: Option<Datetime>,
    pub allow_connect_again: bool,
    pub user: Thing,
}

impl Ticket {
    pub const TABLE: &str = "tickets";
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SnapshotDownload {
    pub id: Option<Thing>,
    pub created_at: Datetime,
    pub snapshot: Thing,
    pub user: Thing,
    pub downloaded_at: Option<Datetime>,
}

impl SnapshotDownload {
    pub const TABLE: &str = "snapshot-downloads";

    pub fn new(snapshot: Thing, user: Thing) -> SnapshotDownload {
        SnapshotDownload {
            id: None,
            created_at: Datetime::default(),
            snapshot,
            user,
            downloaded_at: None,
        }
    }

    pub fn mark_as_downloaded(&mut self) {
        self.downloaded_at = Some(Datetime::default());
    }
}
