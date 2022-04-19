use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use crate::error;
use crate::shell::commands::*;

pub struct CommandRunner {
    commands: BTreeMap<String, Box<dyn Command>>
}

impl CommandRunner {
    pub fn new() -> CommandRunner {
        CommandRunner {
            commands: CommandRunner::default_commands()
        }
    }

    fn default_commands() -> BTreeMap<String, Box<dyn Command>> {
        let mut result: BTreeMap<String, Box<dyn Command>> = BTreeMap::new();

        result.insert(String::from("version"), Box::new(VersionCommand { }));

        result
    }

    pub fn run(&mut self, command: &str) {
        let mut arguments: Vec<String> = command.split_whitespace().map(String::from).collect();

        if arguments.len() == 0 {
            error("Input is empty.");
            return;
        }

        let id = String::from(arguments.get(0).unwrap());

        arguments.remove(0);

        let command_option = self.commands.get_mut(&id);

        match command_option {
            Some(_) => { },
            None => {
                error("Command not found.");
                return;
            }
        }

       command_option.unwrap().run(arguments)
    }
}

pub trait Command {
    fn run(&mut self, arguments: Vec<String>);
}