//! Data structures that using to save data in database

use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Id, Thing};

use crate::{action::AnyAction, oxd::OxdXml, storage::StorageId};

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
    pub const TABLE: &str = "sessions";
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

/// Users can work on multiple projects and multiple branches in parallel
/// by creating separate tabs. A tab is representing a tab in the editor.
#[derive(Deserialize, Serialize, Clone)]
pub struct Tab {
    /// Users can edit the name of a tab. By default it should similiar to
    /// the project name and branch name
    pub name: String,
    /// Related user session
    pub session: Thing,
    pub created_at: Datetime,
    /// Exited time if a user exited a tab
    pub exited_at: Option<Datetime>,
    /// Starting commit
    pub head: Thing,
    /// Related branch
    pub branch: Thing,
    /// Current snapshot of the tab. This should be updated with the user
    /// actions
    pub snapshot: Thing
}

/// Actions that not taken to any commit.
/// Users can undo and redo over those actions
#[derive(Serialize, Deserialize, Clone)]
pub struct TabAction {
    pub id: Id,
    /// Tab related to action
    pub tab: Thing,
    pub created_at: Datetime,
    /// Action data
    pub action: AnyAction
}

/// When a user trying to change the branch/ commit in the tab, they
/// can stash the pending unsaved actions like in the git.
#[derive(Serialize, Deserialize, Clone)]
pub struct Stash {
    pub id: Id,
    /// Tab related to stash
    pub tab: Thing,
    /// Commit that we started to working on
    pub head: Thing,
    /// Stashed time
    pub created_at: Datetime,
    /// Related branch
    pub branch: Thing,
    /// Snapshot when stashing the changes
    pub snapshot: Thing
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: Id,
    /// Display name of the project. Can contain spaces, some symbols
    /// and mixed case letters.
    pub name: String,
    /// The text that should be displayed on the url. This should be a
    /// URL friendly word
    pub slug: String,
    pub created_at: Datetime,
    /// The default branch to accept any push without specifying a branch
    pub default_branch: Thing,
    /// The user who owned the proejct
    pub owner: Thing,
}

impl Project {
    pub const TABLE: &str = "projects";

    pub fn new(id: Id, name: String, slug: String, branch_id: Id, user_id: Id) -> Project {
        Project {
            id,
            name,
            slug,
            created_at: Datetime::default(),
            default_branch: Thing {
                tb: String::from(Branch::TABLE),
                id: branch_id,
            },
            owner: Thing {
                tb: String::from(User::TABLE),
                id: user_id
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Branch {
    pub id: Id,
    /// Name of the branch. This should be a URL friendly word
    pub name: String,
    pub created_at: Datetime,
    pub head: Option<Thing>,
}

impl Branch {
    pub const TABLE: &str = "branches";

    pub fn new<A: StorageId>(
        name: String,
        head: Option<Id>,
    ) -> Branch {
        Branch {
            id: Id::String(String::new()),
            name,
            created_at: Datetime::default(),
            head: head.map(|head_id| Thing {
                tb: String::from(Commit::TABLE),
                id: head_id,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Commit {
    pub id: Id,
    /// Commit message
    pub message: String,
    pub created_at: Datetime,
    /// The branch related to the commit
    pub branch: Thing,
    /// The user whoe authored the commit
    pub user: Thing,
    /// Previous commit. This should be empty if the first commit of the branch
    pub head: Option<Thing>,
    pub snapshot: Thing,
}

impl Commit {
    pub const TABLE: &str = "commits";

    pub fn new<A: StorageId>(message: String, branch_id: Id, user_id: Id, head: Option<Id>, snapshot_id: Id) -> Commit {
        Commit {
            id: Id::String(String::new()),
            message,
            created_at: Datetime::default(),
            branch: Thing {
                tb: String::from(Branch::TABLE),
                id: branch_id,
            },
            user: Thing {
                tb: String::from(User::TABLE),
                id: user_id,
            },
            head: head.map(|head_id| Thing {
                tb: String::from(Commit::TABLE),
                id: head_id,
            }),
            snapshot: Thing {
                tb: String::from(OxdXml::<A>::TABLE),
                id: snapshot_id
            }
        }
    }
}
