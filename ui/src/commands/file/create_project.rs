use std::{fmt::Debug, rc::Rc};

use poll_promise::Promise;

use crate::{
    client::ClientTransport, commands::Command, external::External, scopes::ApplicationScope,
};

pub struct CreateProjectCommand<
    TE: Debug + Send + 'static,
    EE: Debug,
    T: ClientTransport<TE>,
    E: External<Error = EE>,
> {
    app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
    create_project_promise: Promise<Result<String, String>>,
}

impl<TE: Debug + Send + 'static, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>>
    CreateProjectCommand<TE, EE, T, E>
{
    pub fn new(
        app_scope: Rc<ApplicationScope<TE, EE, T, E>>,
        project_name: String,
    ) -> CreateProjectCommand<TE, EE, T, E> {
        let client = app_scope.client();
        app_scope
            .state_mut()
            .set_status_message(format!("Creating Project: {}", &project_name));

        CreateProjectCommand {
            app_scope,
            create_project_promise: Promise::spawn_async(async move {
                let mut client_locked = client.lock().await;
                let result = client_locked.create_new_project(project_name).await;
                match result {
                    Ok(res) => match res.into() {
                        Ok(success) => Ok(String::new()),
                        Err(err) => Err(format!("Remote: {}", err)),
                    },
                    Err(err) => Err(format!("{:?}", err)),
                }
            }),
        }
    }

    pub fn project_created(&mut self) {}
}

impl<TE: Debug + Send + 'static, EE: Debug, T: ClientTransport<TE>, E: External<Error = EE>> Command
    for CreateProjectCommand<TE, EE, T, E>
{
    fn update(&mut self) -> bool {
        if let Some(res) = self.create_project_promise.ready() {
            match res {
                Ok(_) => {
                    self.project_created();
                    self.app_scope.state_mut().clear_status_message();
                }
                Err(e) => {
                    let mut state_mut = self.app_scope.state_mut();
                    state_mut.add_dialog(crate::state::Severity::Error, e);
                    state_mut.clear_status_message();
                    drop(state_mut);
                }
            }
            true
        } else {
            false
        }
    }
}
