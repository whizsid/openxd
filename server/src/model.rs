use app::{model::User, oxd::OxdXml};
use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Id, Thing};

#[derive(Serialize, Deserialize)]
pub struct Ticket {
    pub id: Id,
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
    pub id: Id,
    pub created_at: Datetime,
    pub snapshot: Thing,
    pub user: Thing,
    pub downloaded_at: Option<Datetime>,
}

impl SnapshotDownload {
    pub const TABLE: &str = "snapshot-downloads" ;

    pub fn new(snapshot_id: String, user_id: String) -> SnapshotDownload {
        SnapshotDownload {
            id: Id::String(String::new()),
            created_at: Datetime::default(),
            snapshot: Thing {
                tb: String::from(OxdXml::<crate::storage::StorageId>::TABLE),
                id: Id::String(snapshot_id),
            },
            user: Thing {
                tb: String::from(User::TABLE),
                id: Id::String(user_id),
            },
            downloaded_at: None,
        }
    }

    pub fn mark_as_downloaded(&mut self) {
        self.downloaded_at = Some(Datetime::default());
    }
}
