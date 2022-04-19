use crate::{Color, ColorCode, OS_VERSION, print, println, WRITER};
use crate::reading::READER;
use crate::shell::command_runner::CommandRunner;

mod command_runner;
mod commands;

pub fn initial_run() {
    print!(">");
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

    let mut command_runner = CommandRunner::new();

    command_runner.run(&input);

    READER.lock().column_position_start = 0;

    print!(">");
    print!(" ");
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