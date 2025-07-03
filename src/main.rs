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
static HELLO: &[u8] = b"Hello World!";

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    vga::print_something();
    loop {}
}