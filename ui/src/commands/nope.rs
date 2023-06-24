use super::Command;

/// A command that doing nothing and complete immediately
pub struct NopeCommand;

impl NopeCommand {
    pub fn new() -> NopeCommand {
        NopeCommand
    }
}

impl Command for NopeCommand {
    fn update(&mut self) -> bool {
        true
    }
}
