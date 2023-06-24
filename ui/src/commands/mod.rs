//! Command collection
//!
//! Those commands are triggering from the UI by the user. All the commands are async and
//! using the `poll_promise` crate to track about the commands.
pub mod file;
pub mod tab;
pub mod nope;

/// Commands should implement this trait
pub trait Command {
    // Updating the command status
    //
    // This method will running in each iteration of the event loop. Updating the status of the
    // command should done in this method. The return value is the whether this command is a
    // completed or not. Command executor will remove this command and not calling the `update`
    // method again once it returned a `true`
    fn update(&mut self) -> bool;
}

/// Command executor
///
/// The command executor is the interface to schedule async commands. This executor will help you
/// to execute your asynchronous tasks in background using the `poll_promise` crate.
pub struct Executor {
    commands: Vec<Box<dyn Command + 'static>>
}

impl Executor {

    pub fn new() -> Executor {
        Executor { commands: vec![] }
    }

    /// Updating the status of all commands
    ///
    /// This method is running in each iteration of the event loop. And it will also iterate over
    /// the commands and update the status of each command. If a command marked as completed, this
    /// method will automatically remove them.
    pub fn update(&mut self) {
        let len_commands = self.commands.len();
        for i in 0..len_commands {
            let mut_command = self.commands.get_mut(i).unwrap();
            let finished = mut_command.update();
            if finished {
                self.commands.remove(i);
            }
        }
    }

    /// Schedule a command to run
    pub fn execute<T: Command + 'static>(&mut self, cmd: T) {
        self.commands.push(Box::new(cmd));
    }

    /// Schedule a boxed command to run
    pub fn execute_boxed(&mut self, cmd: Box<dyn Command + 'static>) {
        self.commands.push(cmd);
    }
}
