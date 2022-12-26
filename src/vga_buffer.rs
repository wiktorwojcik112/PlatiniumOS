use alloc::string::String;
use core::fmt;
use volatile::Volatile;
use lazy_static::lazy_static;
use spin::Mutex;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT]
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        row_position: 0,
        color_code: ColorCode::new(Color::White, Color::Black),
        buffer: unsafe { &mut *(0xB8000 as *mut Buffer) }
    });
}

pub struct Writer {
    pub column_position: usize,
    pub row_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer
}

impl Writer {
    pub fn row_into_string(&self, row: usize) -> String {
        let mut result: String = String::new();

        for col in 0..BUFFER_WIDTH {
            let screen_char = self.buffer.chars[row][col].read();

            if screen_char.ascii_character == 0x0 {
                break;
            }

            result.push(char::from(screen_char.ascii_character));
        }

        result
    }

    pub fn change_color_code(&mut self, color_code: ColorCode) {
        self.color_code = color_code;
    }

    pub fn clear(&mut self) {
        let blank = ScreenChar {
            ascii_character: 0x0,
            color_code: self.color_code
        };

        for row in 0..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(blank);
            }
        }

        self.row_position = 0;
        self.column_position = 0;
    }

    pub fn move_left(&mut self) {
        if 0 < self.column_position {
            self.default_current();
            self.column_position -= 1;
            self.highlight_current();
        }
    }

    pub fn move_right(&mut self) {
        if self.buffer.chars[self.row_position][self.column_position - 1].read().ascii_character != 0x0 && self.column_position != 0 {
            self.default_current();
            self.column_position += 1;
            self.highlight_current();
        }
    }

    pub fn backspace(&mut self) {
        let blank = ScreenChar {
            ascii_character: 0x0,
            color_code: self.color_code
        };

        if 0 < self.column_position {
            self.default_current();

            let row = self.row_position;
            let col = self.column_position;

            self.buffer.chars[row][col - 1].write(blank);
            self.column_position -= 1;

            self.highlight_current();
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
       self.default_current();

        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = self.row_position;
                let col = self.column_position;

                let color_code = self.color_code;

                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code
                });

                self.column_position += 1;
            }
        }

        self.highlight_current();
    }

    fn default_current(&mut self) {
        let defaulted = ScreenChar {
            ascii_character: self.buffer.chars[self.row_position][self.column_position].read().ascii_character,
            color_code: ColorCode::new(Color::White, Color::Black)
        };

        self.buffer.chars[self.row_position][self.column_position].write(defaulted)
    }

    fn highlight_current(&mut self) {
        let defaulted = ScreenChar {
            ascii_character: self.buffer.chars[self.row_position][self.column_position].read().ascii_character,
            color_code: ColorCode::new(Color::Black, Color::White)
        };

        self.buffer.chars[self.row_position][self.column_position].write(defaulted)
    }

    fn new_line(&mut self) {
       if self.row_position + 1 == BUFFER_HEIGHT {
            for row in 0..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();

                    if 0 < row {
                        self.buffer.chars[row - 1][col].write(character);
                    }
                }
            }

            self.clear_row(self.row_position);
        } else {
            self.row_position += 1;
        }

        self.column_position = 0;
    }

    pub fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: 0x0,
            color_code: self.color_code
        };

        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match  byte {
                0x20..=0x7E | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xFE)
            }
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}