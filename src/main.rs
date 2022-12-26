#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(box_into_inner)]
#![feature(asm_const)]

#![no_std]
#![no_main]

mod vga_buffer;
mod interrupts;
mod gdt;
mod memory;
mod allocator;
mod shell;
mod reading;
mod task;

use core::arch::asm;
use core::panic::PanicInfo;
use bootloader::{BootInfo, entry_point};
use crossbeam_queue::ArrayQueue;
use x86_64::VirtAddr;
use crate::memory::BootInfoFrameAllocator;
use crate::task::executor::Executor;
use crate::task::keyboard::SCANCODE_QUEUE;
use crate::task::{keyboard, Task};
use crate::vga_buffer::{Color, ColorCode, WRITER};

extern crate alloc;

entry_point!(kernel_main);

pub static OS_VERSION: &str = "1.0";

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    println!("Starting kernel");

    gdt::init();
    println!("[GDT] Initialized");

    interrupts::init();
    println!("[Interrupts] Initialized");

    unsafe { interrupts::PICS.lock().initialize() };
    println!("[PICS] Initialized");

    unsafe { asm!("sti", options(nomem, nostack)) };

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("Heap initialization failed");

    println!("[HEAP] Initialized");

    SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100)).expect("Scancode Queue should be initialized only once.");
    println!("[SCANCODE QUEUE] Initialized");

    unsafe {
        // Disable VGA cursor
        let mut port: u16 = 0x3D4;
        let mut value: u8 = 0xA;

        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));

        port = 0x3D5;
        value = 0x20;

        asm!("out dx, al", in("dx") port, in("al") value, options(nomem, nostack, preserves_flags));
    }

    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().clear();
    });

    println!("Ready");

    WRITER.lock().clear();

    shell::print_info_message();
    shell::initial_run();

    let mut executor = Executor::new();

    executor.spawn(Task::new(keyboard::input_handler()));

    executor.run();
}

fn hlt_loop() -> ! {
    loop {
        unsafe {
            asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::LightRed, Color::Black));
    });

    println!("{}", info);

    hlt_loop();
}

fn error(message: &str) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::LightRed, Color::Black));
    });

    println!("ERROR: {}", message);

    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::White, Color::Black));
    });
}

fn warning(message: &str) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::Yellow, Color::Black));
    });

    println!("{}", message);

    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::White, Color::Black));
    });
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}