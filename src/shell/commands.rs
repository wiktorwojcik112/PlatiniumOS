use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::arch::asm;
use crate::{Color, ColorCode, error, OS_VERSION, WRITER};
use crate::shell::command_runner::Command;
use crate::shell::SHELL_ENVIRONMENT;

pub struct VersionCommand;

impl Command for VersionCommand {
    fn run(&mut self, arguments: Vec<String>) -> String {
        if arguments.len() == 0 {
            OS_VERSION.to_string()
        } else {
            error("version expects 0 arguments.");
            String::new()
        }
    }
}

pub struct EchoCommand;

impl Command for EchoCommand {
    fn run(&mut self, arguments: Vec<String>) -> String {
        let mut string = String::new();
        for argument in arguments {
            string.push_str(&argument);
            string.push(' ');
        }

        string
    }
}

pub struct CalcCommand;

impl Command for CalcCommand {
    fn run(&mut self, arguments: Vec<String>) -> String {
        let calculator = crate::shell::calculator::Calculator::new(arguments);
        calculator.calculate().to_string()
    }
}

pub struct SetCommand;

impl Command for SetCommand {
    fn run(&mut self, arguments: Vec<String>) -> String {
        if arguments.len() != 2 {
            error("set expects 2 arguments.");
            return String::new();
        }

        SHELL_ENVIRONMENT.lock().variables.insert(arguments[0].to_string(), arguments[1].to_string());
        arguments[1].to_string()
    }
}

pub struct ColorCommand;

impl Command for ColorCommand {
    fn run(&mut self, arguments: Vec<String>) -> String {
        if arguments.len() != 1 {
            error("color expects 1 arguments.");
            return String::new();
        }

        match &arguments[0] as &str  {
            "red" => WRITER.lock().change_color_code(ColorCode::new(Color::Red, Color::Black)),
            "green" => WRITER.lock().change_color_code(ColorCode::new(Color::Green, Color::Black)),
            "blue" => WRITER.lock().change_color_code(ColorCode::new(Color::Blue, Color::Black)),
            "yellow" => WRITER.lock().change_color_code(ColorCode::new(Color::Yellow, Color::Black)),
            "cyan" => WRITER.lock().change_color_code(ColorCode::new(Color::Cyan, Color::Black)),
            "magenta" => WRITER.lock().change_color_code(ColorCode::new(Color::Magenta, Color::Black)),
            "white" => WRITER.lock().change_color_code(ColorCode::new(Color::White, Color::Black)),
            "black" => WRITER.lock().change_color_code(ColorCode::new(Color::Black, Color::Black)),
            _ => { error("invalid color.") }
        }

        String::new()
    }
}

pub struct HelpCommand;

impl Command for HelpCommand {
    fn run(&mut self, arguments: Vec<String>) -> String {
        if arguments.len() == 0 {
            return String::from("available commands: version, echo, calc, set, color, help");
        } else if arguments.len() != 1 {
            error("help expects 1 arguments.");
            return String::new();
        }

        return match &arguments[0] as &str {
            "version" => "version - (0 arguments) prints the version of the operating system.",
            "echo" => "echo - (many arguments; any type) prints the arguments.",
            "calc" => "calc - (many arguments; + - * / numbers) calculates the arguments.",
            "set" => "set - (2 arguments; key value) sets a variable.",
            "color" => "color - (1 argument; red green blue yellow cyan magenta white black) changes the color of the text.",
            "help" => "help - (1 argument; command) prints the help for the command.",
            _ => { error("invalid command."); "" }
        }.to_string();
    }
}