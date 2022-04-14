#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

mod vga_buffer;
mod interrupts;
mod gdt;

use core::arch::asm;
use core::panic::PanicInfo;
use crate::vga_buffer::{Color, ColorCode, Writer, WRITER};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    hlt_loop();
}

fn init() {
    gdt::init();
    interrupts::init();
    unsafe { interrupts::PICS.lock().initialize() };
    unsafe { asm!("sti", options(nomem, nostack)) };
}

fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();

    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::Yellow, Color::Black));
    });

    println!("PlatiniumOS v.1.0");

    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::White, Color::Black));
    });

    hlt_loop();
}