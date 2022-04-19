use lazy_static::lazy_static;
use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use crate::{gdt, println, ColorCode, Color, hlt_loop, vga_buffer::WRITER};
use pic8259::ChainedPics;
use x86_64::instructions::port::Port;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::registers::control::Cr2;

pub fn init() {
    IDT.load();
}

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
            .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);

        idt
    };
}

extern "x86-interrupt" fn empty_handler(_stack_frame: InterruptStackFrame) {

}

extern "x86-interrupt" fn page_fault_handler(stack_frame: InterruptStackFrame, error_code: PageFaultErrorCode) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::LightRed, Color::Black));
    });

    println!("EXCEPTION: PAGE FAULT");
    println!("ACCESED ADDRESS: {:?}", Cr2::read());
    println!("ERRROR CODE: {:?}", error_code);
    println!("{:#?}", stack_frame);

    hlt_loop();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::LightRed, Color::Black));
    });

    println!("EXCEPTION: BREAKPOINT\n{:3?}", stack_frame);

    x86_64::instructions::interrupts::without_interrupts(|| {
        WRITER.lock().change_color_code(ColorCode::new(Color::White, Color::Black));
    });
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\nCODE:{}\n{:#?}", error_code, stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore));
    }

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}