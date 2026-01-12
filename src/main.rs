#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]

//Imports
use core::panic::PanicInfo;

//Modules
mod drivers;
mod gdt;
mod interrupts;
mod shell;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[no_mangle]
//This is the entrypoint of the OS
pub extern "C" fn _start() -> ! {
    println!("Hello World{}", "!");
    
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    println!("It did not crash!");

    crate::shell::run_shell();
    
    #[allow(clippy::empty_loop)]
    loop {
        x86_64::instructions::hlt();
    }
}