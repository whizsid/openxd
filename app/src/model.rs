//! Data structures that using to save data in database

use serde::{Deserialize, Serialize};
use surrealdb::sql::{Datetime, Thing, Id};

use crate::{action::AnyAction, storage::{StorageId, StorageIdWithoutSerde}, oxd::OxdXml};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Option<Thing>,
    pub name: String,
}

impl User {
    pub const TABLE: &str = "users";

    pub fn new(name: String) -> User {
        User { id: None , name }
    }

    pub fn new_with_id(id: String, name: String) -> User {
        User { id: Some(thing(User::TABLE, id)), name}
    }
}

/// Data that should be stored in the database related to a session
#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: Option<Thing>,
    pub user: Thing,
    pub created_at: Datetime,
    pub last_activity: Datetime,
    pub closed_at: Option<Datetime>,
    pub current_tab: Option<Thing>,
    /// Pixel size
    pub screen_size: (u32, u32)
}

impl Session {
    pub const TABLE: &str = "sessions";
    /// Creating a session data with an empty to save in DB
    pub fn create(user: Thing) -> Session {
        Session {
            created_at: Datetime::default(),
            last_activity: Datetime::default(),
            closed_at: None,
            user,
            id: None,
            current_tab: None,
            screen_size: (0, 0)
        }
    }

    pub fn set_current_tab(&mut self, tab: Thing) {
        self.current_tab = Some(tab);
    }

    pub fn mark_closed(&mut self) {
        self.closed_at = Some(Datetime::default());
    }

    pub fn change_size(&mut self, width: u32, height: u32) {
        self.screen_size = (width, height);
    }
}

/// Users can work on multiple projects and multiple branches in parallel
/// by creating separate tabs. A tab is representing a tab in the editor.
#[derive(Deserialize, Serialize, Clone)]
pub struct Tab {
    pub id: Option<Thing>,
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
    pub snapshot: Thing,
}

impl Tab {
    pub const TABLE: &str = "tabs";

    pub fn new<SI: StorageId>(
        name: String,
        session: Thing,
        head: Thing,
        branch: Thing,
        snapshot: Thing,
    ) -> Tab {
        Tab {
            id: None,
            name,
            session,
            created_at: Datetime::default(),
            exited_at: None,
            head,
            branch,
            snapshot,
        }
    }
}

/// Actions that not taken to any commit.
/// Users can undo and redo over those actions
#[derive(Serialize, Deserialize, Clone)]
pub struct TabAction {
    pub id: Option<Thing>,
    /// Tab related to action
    pub tab: Thing,
    pub created_at: Datetime,
    /// Action data
    pub action: AnyAction,
}

/// When a user trying to change the branch/ commit in the tab, they
/// can stash the pending unsaved actions like in the git.
#[derive(Serialize, Deserialize, Clone)]
pub struct Stash {
    pub id: Option<Thing>,
    /// Tab related to stash
    pub tab: Thing,
    /// Commit that we started to working on
    pub head: Thing,
    /// Stashed time
    pub created_at: Datetime,
    /// Related branch
    pub branch: Thing,
    /// Snapshot when stashing the changes
    pub snapshot: Thing,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub id: Option<Thing>,
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

    pub fn new(id: Thing, name: String, slug: String, branch: Thing, user: Thing) -> Project {
        Project {
            id: Some(id),
            name,
            slug,
            created_at: Datetime::default(),
            default_branch: branch,
            owner: user,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Branch {
    pub id: Option<Thing>,
    /// Name of the branch. This should be a URL friendly word
    pub name: String,
    pub created_at: Datetime,
    pub head: Option<Thing>,
}

impl Branch {
    pub const TABLE: &str = "branches";

    pub fn new<A: StorageId>(name: String, head: Option<Thing>) -> Branch {
        Branch {
            id: None,
            name,
            created_at: Datetime::default(),
            head,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Commit {
    pub id: Option<Thing>,
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

    pub fn new<A: StorageId>(
        message: String,
        branch: Thing,
        user: Thing,
        head: Option<Thing>,
        snapshot: Thing,
    ) -> Commit {
        Commit {
            id: None,
            message,
            created_at: Datetime::default(),
            branch,
            user,
            head,
            snapshot,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Snapshot<A: StorageIdWithoutSerde> {
    pub id: Option<Thing>,
    pub oxd: OxdXml<A>
}

impl <A: StorageId> Snapshot<A> {

    pub const TABLE: &str = "snapshots";

    pub fn new(oxd: OxdXml<A>) -> Snapshot<A> {
        Snapshot { id: None , oxd }
    }
}

pub fn thing<T: Into<String>, I: Into<Id>>(table: T, id: I) -> Thing {
    Thing {tb: table.into(), id: id.into()}
}
