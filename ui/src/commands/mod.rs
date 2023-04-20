pub mod file;

pub trait Command {
    // Updating the command status
    fn update(&mut self) -> bool;
}

pub struct Executor {
    commands: Vec<Box<dyn Command + 'static>>
}

impl Executor {

    pub fn new() -> Executor {
        Executor { commands: vec![] }
    }

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

    pub fn execute<T: Command + 'static>(&mut self, cmd: T) {
        self.commands.push(Box::new(cmd));
    }
}
