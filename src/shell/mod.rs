use alloc::collections::BTreeMap;
use lazy_static::lazy_static;
use crate::{Color, ColorCode, OS_VERSION, print, println, WRITER};
use crate::reading::READER;
use crate::shell::command_runner::CommandRunner;
use spin::Mutex;
use alloc::vec::Vec;
use alloc::string::String;

mod command_runner;
mod commands;
mod calculator;

lazy_static! {
    pub static ref SHELL_HISTORY: Mutex<ShellHistory> = Mutex::new(ShellHistory::new());
}

lazy_static! {
    pub static ref SHELL_ENVIRONMENT: Mutex<ShellEnvironment> = Mutex::new(ShellEnvironment::new());
}

pub struct ShellEnvironment {
    pub variables: BTreeMap<String, String>
}

impl ShellEnvironment {
    pub fn new() -> ShellEnvironment {
        ShellEnvironment {
            variables: BTreeMap::new()
        }
    }
}

pub struct ShellHistory {
    pub history: Vec<String>,
    pub index: u64
}

impl ShellHistory {
    pub fn new() -> ShellHistory {
        ShellHistory {
            history: Vec::new(),
            index: 0
        }
    }
}

pub fn initial_run() {
    print!("> ");
    READER.lock().awaits_input = true;
    READER.lock().column_position_start = 0;
}

pub fn run() {
    if READER.lock().awaits_input {
        return;
    }

    let row = WRITER.lock().row_position;
    let mut input = WRITER.lock().row_into_string(row - 1);

    // Remove input indicator.
    input.remove(0);
    input.remove(0);

    SHELL_HISTORY.lock().history.push(input.clone());
    let length = SHELL_HISTORY.lock().history.len() as u64;
    SHELL_HISTORY.lock().index = length;

    let mut command_runner = CommandRunner::new();

    command_runner.run(&input);

    READER.lock().column_position_start = 0;

    print!("> ");
    READER.lock().awaits_input = true;
    READER.lock().column_position_start = 0;
}

pub fn print_info_message() {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::Yellow, Color::Black));
    });

    println!("PlatiniumOS {}", OS_VERSION);

    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::White, Color::Black));
    });
}