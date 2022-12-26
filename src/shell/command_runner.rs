use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use crate::{error, println};
use crate::shell::commands::*;
use crate::shell::SHELL_ENVIRONMENT;

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
        result.insert(String::from("echo"), Box::new(EchoCommand { }));
        result.insert(String::from("calc"), Box::new(CalcCommand { }));
        result.insert(String::from("set"), Box::new(SetCommand { }));
        result.insert(String::from("color"), Box::new(ColorCommand { }));

        result
    }

    pub fn run(&mut self, command: &str) {
        let mut arguments: Vec<String> = command.split_whitespace().map(String::from).collect();

        if arguments.len() == 0 {
            error("Input is empty.");
            return;
        }

        println!("{}", self.run_command(arguments))
    }

    pub fn run_command(&mut self, arguments: Vec<String>) -> String {
        let mut arguments: Vec<String> = arguments;
        let id = arguments[0].clone();
        arguments.remove(0);

        let command_option = self.commands.get_mut(&id);

        if let None = command_option {
            error("Command not found.");
            return String::new();
        }

        command_option.unwrap().run(Self::process_arguments(arguments))
    }

    fn process_arguments(arguments: Vec<String>) -> Vec<String> {
        let mut result: Vec<String> = Vec::new();
        let mut string: String = String::new();

        let mut is_in_string: bool = false;

        let mut inline_command: Vec<String> = Vec::new();
        let mut is_in_inline_command: bool = false;

        for argument in arguments {
            if argument.starts_with("$") {
                if argument.starts_with("$(") {
                    is_in_inline_command = true;
                    inline_command.push(String::from(argument.trim_start_matches("$(")));
                } else {
                    let variable_name = argument.trim_start_matches("$");
                    let variables = SHELL_ENVIRONMENT.lock().variables.clone();
                    let variable_value = variables.get(&variable_name.to_string().clone());

                    if let None = variable_value {
                        error("Variable not found.");
                        return Vec::new();
                    }

                    result.push(variable_value.unwrap().to_string());
                }
            } else if argument.ends_with(")") && is_in_inline_command {
                inline_command.push(String::from(argument.trim_end_matches(")")));
                is_in_inline_command = false;

                let mut command_runner = CommandRunner::new();
                result.push(command_runner.run_command(inline_command.clone()));
                inline_command.clear();
            } else if is_in_inline_command {
                inline_command.push(argument);
            } else if argument.starts_with("\\") {
                if argument == "\\n" {
                    result.push("\n".to_string());
                } else {
                    error("unknown escape sequence.");
                }
            } else if argument.starts_with("\"") && argument.ends_with("\"") {
                result.push(argument.trim_start_matches("\"").trim_end_matches("\"").to_string());
            } else if argument.starts_with("\"") {
                string = argument.trim_start_matches("\"").to_string();
                is_in_string = true;
            } else if argument.starts_with("\"") && is_in_string {
                string.push_str(" ");
                result.push(string);
                string = String::new();
            } else if argument.ends_with("\"") && is_in_string {
                string.push_str(" ");
                string.push_str(argument.trim_end_matches("\""));
                result.push(string);
                string = String::new();
            } else if is_in_string {
                string.push_str(" ");
                string.push_str(&argument);
            } else {
                result.push(argument);
            }
        }

        result
    }
}

pub trait Command {
    fn run(&mut self, arguments: Vec<String>) -> String;
}