use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref READER: Mutex<Reader> = Mutex::new(Reader {
        awaits_input: false,
        column_position_start: 0
    });
}

pub struct Reader {
    pub awaits_input: bool,
    pub column_position_start: usize,
}