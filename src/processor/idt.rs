use crate::println;
use crate::processor::gdt;
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

// Definir los offsets base para el PIC (Programmable Interrupt Controller).
// El PIC mapea interrupciones de hardware a vectores de interrupción del CPU.
// Offset 32: Las primeras 32 interrupciones están reservadas para excepciones del CPU,
// así que mapeamos el PIC para que empiece en el índice 32.
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8; // El segundo PIC gestiona las siguientes 8 interrupciones.

// Instancia estática del controlador PIC, protegida por un Mutex.
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

// Enum para mapear interacciones comunes a sus índices en la IDT.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,        // Temporizador (IRQ0)
    Keyboard = PIC_1_OFFSET + 1, // Teclado (IRQ1)
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

lazy_static! {
    // Definir la Interrupt Descriptor Table (IDT) de manera estática y perezosa.
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        // Asignar el manejador para excepciones de Breakpoint.
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            // Asignar manejador para Double Fault y configurar su pila dedicada (definida en GDT).
            // Esto es crucial para evitar bucles infinitos de fallos de pila.
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        // Asignar manejador para fallos de página (Page Fault).
        idt.page_fault.set_handler_fn(page_fault_handler);

        // Asignar manejadores para interrupciones de hardware.
        idt[InterruptIndex::Timer.as_usize()]
            .set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()]
            .set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

// Función para cargar la IDT en el CPU.
pub fn init_idt() {
    IDT.load();
}

// Manejador para excepciones de interrupción (Breakpoint).
// Se llama cuando se encuentra la instrucción `int3`.
extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

// Manejador para excepciones de doble fallo (Double Fault).
// Se llama cuando el CPU no puede invocar al manejador de excepciones.
extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

// Manejador para fallos de paginación (Page Fault).
// Se llama cuando se accede a una dirección de memoria no válida o no permitida.
extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read()); // Cr2 contiene la dirección que causó el fallo.
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    loop {}
}

// Manejador de la interrupción del temporizador.
extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // println!("."); // Comentado para evitar llenar la pantalla.
    unsafe {
        // Notificar al PIC que hemos terminado de manejar la interrupción.
        PICS.lock()+
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

// Manejador de la interrupción del teclado.
extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
    use spin::Mutex;
    use x86_64::instructions::port::Port;

    // Inicializar el decodificador de teclado estático y protegido por Mutex.
    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> = Mutex::new(
            Keyboard::new(layouts::Us104Key, ScancodeSet1, HandleControl::Ignore)
        );
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60); // El puerto de datos del teclado es 0x60.

    let scancode: u8 = unsafe { port.read() }; // Leer el scancode del puerto.
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        // Procesar el evento y convertirlo a un carácter opcional.
        if let Some(key) = keyboard.process_keyevent(key_event) {
            crate::shell::add_key(key); // Enviar la tecla a la shell.
        }
    }

    unsafe {
        // Notificar al PIC de fin de interrupción.
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
