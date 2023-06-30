use poll_promise::Promise;
use transport::{app::TabCreatedMessage, vo::Screen};

use crate::{
    commands::Command, scopes::ApplicationScope,
};

pub struct CreateProjectCommand {
    app_scope: ApplicationScope,
    create_project_promise: Promise<Result<TabCreatedMessage, String>>,
}

impl CreateProjectCommand {
    pub fn new(
        app_scope: ApplicationScope,
        project_name: String,
    ) -> CreateProjectCommand {
        app_scope
            .state_mut()
            .set_status_message(format!("Creating Project: {}", &project_name));

        let client = app_scope.client();

        CreateProjectCommand {
            app_scope,
            create_project_promise: Promise::spawn_async(async move {
                let mut client_locked = client.lock().await;
                let result = client_locked.create_new_project(project_name).await;
                result.map_err(|e|format!("{:?}", e))
            }),
        }
    }

    pub fn project_created(
        &mut self,
        tab_name: String,
        tab_id: String,
        zoom: f64,
        screens: Vec<Screen>,
    ) {
        self.app_scope.add_project(tab_id, tab_name, zoom, screens);
    }
}

impl Command for CreateProjectCommand {
    fn update(&mut self) -> bool {
        if let Some(res) = self.create_project_promise.ready() {
            match res {
                Ok(tab_created) => {
                    self.project_created(
                        tab_created.tab_name.clone(),
                        tab_created.tab_id.clone(),
                        tab_created.zoom,
                        tab_created.screens.clone(),
                    );
                    self.app_scope.state_mut().clear_status_message();
                }
                Err(e) => {
                    let mut state_mut = self.app_scope.state_mut();
                    state_mut.add_dialog(crate::state::Severity::Error, e);
                    state_mut.clear_status_message();
                }
            }
            true
        } else {
            false
        }
    }
}
