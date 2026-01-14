#![no_std] // No enlazar con la biblioteca estándar de Rust (std), necesaria para un sistema operativo bare-metal.
#![no_main] // Desactivar el punto de entrada estándar 'main' de Rust, ya que no tenemos runtime.
#![feature(abi_x86_interrupt)] // Habilitar la característica experimental para manejar interrupciones x86.

// Importaciones
use core::panic::PanicInfo; // Importar la estructura PanicInfo para manejar errores fatales.

// Declaración de módulos
mod processor;      // Módulo para Control del Procesador (GDT, IDT).
mod memory;         // Módulo para Gestión de Memoria.
mod execution;      // Módulo para Ejecución y Tareas.
mod drivers;        // Módulo para controladores de hardware (ej. VGA).
mod shell;          // Módulo para la interfaz de línea de comandos.

/// Esta función se llama cuando ocurre un pánico (error irrecuperable).
#[panic_handler] // Atributo que marca esta función como el manejador de pánico.
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info); // Imprimir la información del error en la pantalla.
    loop {} // Bucle infinito para detener el sistema, ya que no podemos continuar.
}

const WELCOME: &str = "    
###############################################################################
#                                                                             #
# @@@@@@@@  @@@@@@@@@@                 @@@              @@@@@@      (@@@@@@   #
# @@@   #@@    @@@                     @@@            @@@    @@@   @@@        #
# @@@   #@@    @@@      @@@@    @@@@   @@@           @@@      @@@  @@@        #
# @@@@@@@@@    @@@    @@___@  @@      @@@# @@@       @@@      @@@  @@@@@@@@   #
# @@@          @@@    @@      @@      @@@  @@@       @@@     @@@        @@@   #
# @@@          @@@     @@@@@@   @@@@   @@@  @@@        @@@@@@@     @@@@@@@@   #
#                                                                             #
#############################################################################";

#[no_mangle] // Desactivar el 'name mangling' para que el linker pueda encontrar la función '_start'.
// Este es el punto de entrada del sistema operativo.
pub extern "C" fn _start() -> ! {
    println!("{}", WELCOME); // Imprimir un mensaje de bienvenida para verificar la salida VGA.
    
    processor::gdt::init(); // Inicializar la Global Descriptor Table (GDT).
    processor::idt::init_idt(); // Inicializar la Interrupt Descriptor Table (IDT).
    unsafe { processor::idt::PICS.lock().initialize() }; // Inicializar el Programmable Interrupt Controller (PIC).
    x86_64::instructions::interrupts::enable(); // Habilitar interrupciones de CPU (sti).

    crate::shell::run_shell(); // Iniciar el bucle de la shell interactiva.
    
    #[allow(clippy::empty_loop)] // Permitir un bucle vacío sin advertencias del linter.
    loop {
        x86_64::instructions::hlt(); // Detener el CPU hasta la siguiente interrupción (ahorra energía).
    }
}