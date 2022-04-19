use alloc::string::String;
use alloc::vec::Vec;
use core::arch::asm;
use crate::{error, OS_VERSION, println, qemuShutdown};
use crate::shell::command_runner::Command;

pub struct VersionCommand;

impl Command for VersionCommand {
    fn run(&mut self, arguments: Vec<String>) {
        if arguments.len() == 0 {
            println!("{}", OS_VERSION);
        } else {
            error("version expects 0 arguments.");
        }
    }
}