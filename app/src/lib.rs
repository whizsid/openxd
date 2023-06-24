use asset::{GetAssets, ReplaceAsset};
use client::{Client, ClientTransport};
use helpers::remove_symbols_and_extra_spaces;
use log::{warn, info};
use oxd::OxdXml;
use std::error::Error as StdError;
use std::{collections::HashMap, fmt::Debug, marker::PhantomData, sync::Arc};
use storage::{Storage, StorageId};
use surrealdb::{sql::Id, Connection, Surreal};
use transport::{ui::UIMessage, ReceiveError};

pub mod action;
mod asset;
mod client;
pub mod external;
pub mod helpers;
pub mod model;
pub mod oxd;
pub mod storage;

use model::{
    thing, Branch, Commit, Project, Session as SessionModel, Snapshot, Tab, TabAction, User,
};

pub static OXD_VERSION: &str = "0.0.1";
pub static DEFAULT_BRANCH: &str = "main";

pub struct App<D: Connection> {
    db: Arc<Surreal<D>>,
}

impl<D: Connection> App<D> {
    pub fn new(db: Arc<Surreal<D>>) -> App<D> {
        App { db }
    }

    /// Creating a user session in editor
    pub async fn create_session<
        SE: Debug + StdError,
        SI: StorageId,
        S: Storage<SE, SI>,
        TE: Debug + Send,
        T: ClientTransport<TE>,
    >(
        &mut self,
        user_id: String,
        internal_client: T,
        storage: Arc<S>,
    ) -> Result<Session<SE, SI, S, TE, T, D>, surrealdb::Error> {
        let session_data: SessionModel = self
            .db
            .create(SessionModel::TABLE)
            .content(SessionModel::create(thing(User::TABLE, user_id.clone())))
            .await?;
        Ok(Session::new(
            session_data,
            internal_client,
            user_id.clone(),
            self.db.clone(),
            storage,
        ))
    }

    /// Accessing the internal database connection
    pub fn database(&self) -> Arc<Surreal<D>> {
        self.db.clone()
    }
}

/// A user session in a designer.
///
/// This is an intermidiate situation between ticket and a project.
/// So there can be sessions without any project. Also there can be multiple
/// tickets with the same session. Because someone can reuse the same session
/// after closing the editor without saving.
pub struct Session<
    SE: Debug + StdError,
    SI: StorageId,
    S: Storage<SE, SI>,
    TE: Debug + Send,
    T: ClientTransport<TE>,
    D: Connection,
> {
    client: Client<TE, T>,
    db: Arc<Surreal<D>>,
    data: SessionModel,
    user_id: String,
    storage: Arc<S>,
    _phantom: PhantomData<(SE, SI)>,
}

impl<
        SE: Debug + StdError,
        SI: StorageId,
        S: Storage<SE, SI>,
        TE: Debug + Send,
        T: ClientTransport<TE>,
        D: Connection,
    > Session<SE, SI, S, TE, T, D>
{
    pub fn new(
        data: SessionModel,
        internal_client: T,
        user_id: String,
        db: Arc<Surreal<D>>,
        storage: Arc<S>,
    ) -> Session<SE, SI, S, TE, T, D> {
        Session {
            client: Client::new(internal_client),
            data,
            user_id,
            db,
            storage,
            _phantom: PhantomData,
        }
    }

    pub async fn receive_message(&mut self) -> Result<UIMessage, ReceiveError> {
        self.client.receive().await
    }

    /// Starting the session
    pub async fn handle_message(&mut self, message: UIMessage) {
        match message {
            UIMessage::OpenFile(message) => match self.add_tab_with_project(message).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("Failed to add opened project as a tab:- {:?}", e);
                    self.client.error(e).await.unwrap();
                }
            },
            UIMessage::NewProject(message) => {
                let project_created = self.create_project(message).await;
                match project_created {
                    Ok(project_id) => match self.add_tab_with_project(project_id).await {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Failed to add created project as a tab:- {:?}", e);
                            self.client.error(e).await.unwrap();
                        }
                    },
                    Err(err) => {
                        warn!("Failed to create the project:- {:?}", err);
                        self.client.error(err).await.unwrap();
                    }
                }
            }
            UIMessage::Ping => {
                self.client.pong().await.unwrap();
            }
            UIMessage::Resize(width, height) => {
                self.resize(width, height);
            }
            UIMessage::CloseTab(tab_id) => {
                self.remove_tab(tab_id).await;
            }
            _ => {}
        }
    }

    pub async fn create_project(
        &mut self,
        project_name: String,
    ) -> Result<String, CreateProjectError> {
        let oxd = OxdXml::<SI>::new();
        let snapshot = Snapshot::new(oxd);

        let created_oxd: Snapshot<SI> = self
            .db
            .create(Snapshot::<SI>::TABLE)
            .content(snapshot)
            .await?;

        let branch = Branch::new::<SI>(String::from(DEFAULT_BRANCH), None);
        let created_branch: Branch = self.db.create(Branch::TABLE).content(branch).await?;

        let commit = Commit::new::<SI>(
            String::from("Initial Commit"),
            created_branch.id.clone().unwrap(),
            thing(User::TABLE, self.user_id.clone()),
            None,
            created_oxd.id.unwrap(),
        );
        let _created_commit: Commit = self.db.create(Commit::TABLE).content(commit).await?;

        let file_name_without_sym_spc = remove_symbols_and_extra_spaces(project_name.clone());
        let slug = file_name_without_sym_spc.to_lowercase().replace("", "-");

        let project = Project::new(
            thing(Project::TABLE, Id::rand()),
            String::from(file_name_without_sym_spc),
            slug,
            created_branch.id.unwrap(),
            thing(User::TABLE, self.user_id.clone()),
        );
        let created_project: Project = self.db.create(Project::TABLE).content(project).await?;

        Ok(created_project.id.unwrap().id.to_string())
    }

    pub async fn add_tab_with_project(
        &mut self,
        project_id: String,
    ) -> Result<(), AddTabError<SE>> {
        let project: Option<Project> = self.db.select((Project::TABLE, project_id)).await?;

        if let None = project {
            return Err(AddTabError::ProjectNotFound);
        }

        let project = project.unwrap();

        let default_branch: Branch = self.db.select(project.default_branch).await?;

        let mut commit_res = self.db.query("SELECT * FROM type::table($table) WHERE branch = $branch ORDER BY created_at DESC LIMIT 1")
            .bind(("table", Commit::TABLE))
            .bind(("branch", default_branch.id.clone())).await?;

        let commit: Option<Commit> = commit_res.take(0)?;
        let commit = commit.unwrap();

        let mut snapshot: Snapshot<SI> = self.db.select(commit.snapshot).await?;
        snapshot.id = None;
        let oxd = snapshot.oxd;
        let assets = oxd.get_assets();
        let mut replaced_assets: HashMap<SI, SI> = HashMap::new();
        for asset in assets {
            let duplicated = self
                .storage
                .duplicate(asset.clone())
                .await
                .map_err(AddTabError::Storage)?;
            replaced_assets.insert(asset, duplicated);
        }

        let replaced_oxd = oxd.replace_asset(&mut replaced_assets);
        let replaced_snapshot = Snapshot::new(replaced_oxd);
        let created_snapshot: Snapshot<SI> = self
            .db
            .create(Snapshot::<SI>::TABLE)
            .content(replaced_snapshot)
            .await?;

        let tab = Tab::new::<SI>(
            project.name,
            self.data.id.clone().unwrap(),
            commit.id.unwrap(),
            default_branch.id.unwrap(),
            created_snapshot.id.unwrap(),
        );
        let created_tab: Tab = self.db.create(Tab::TABLE).content(tab).await?;

        let mut updated_session = self.data.clone();
        updated_session.set_current_tab(created_tab.id.clone().unwrap());
        self.data = updated_session.clone();

        let _: Option<SessionModel> = self
            .db
            .update(updated_session.id.clone().unwrap())
            .content(updated_session)
            .await?;

        let zoom: f64 = 0.5;

        self.client
            .tab_created(
                created_tab.name,
                created_tab.id.unwrap().id.to_string(),
                zoom,
                vec![],
            )
            .await
            .unwrap();

        Ok(())
    }

    pub async fn close(&mut self) {
        let mut session = self.data.clone();
        session.mark_closed();
        let _res: Option<SessionModel> = self
            .db
            .update((SessionModel::TABLE, self.data.id.clone().unwrap().id))
            .content(session)
            .await
            .unwrap();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.data.change_size(width, height);
    }

    pub async fn remove_tab(&mut self, tab_id: String) {
        let mut changes_res = self
            .db
            .query("SELECT * FROM type::table($tab_action_table) WHERE tab=type::thing($tab)")
            .bind(("tab_action_table", TabAction::TABLE))
            .bind(("tab", &tab_id))
            .await
            .unwrap();
        let changes: Vec<TabAction> = changes_res.take(0).unwrap();

        for change in changes {
            // Remove added assets
            let _deleted: Option<TabAction> = self.db.delete(change.id.unwrap()).await.unwrap();
        }

        let _deleted_tab: Option<Tab> = self.db.delete(thing(Tab::TABLE, &tab_id)).await.unwrap();
        info!("Removed tab:- {}", &tab_id);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateProjectError {
    #[error("could not read/write the data from database")]
    Db(#[from] surrealdb::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum AddTabError<SE: Debug> {
    #[error("could not read/write the data fromd database")]
    Db(#[from] surrealdb::Error),

    #[error("project id is not a valid id")]
    ProjectNotFound,

    #[error("asset upload/download error")]
    Storage(SE),
}
