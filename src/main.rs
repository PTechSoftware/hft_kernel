#![no_std] 
#![no_main]
//Imports
use core::panic::PanicInfo;

use crate::drivers::vga_drivers::vga;


//Modules
mod drivers;


/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[unsafe(no_mangle)]
//This is the entrypoint of the OS
pub extern "C" fn _start() -> ! {
    loop {}
}