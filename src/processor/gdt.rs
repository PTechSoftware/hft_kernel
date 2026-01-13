use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;
use lazy_static::lazy_static;

// Definir el índice para la pila de Double Fault en la Interrupt Stack Table (IST).
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    // Definir la Task State Segment (TSS) estática.
    // Usamos lazy_static porque TSS necesita inicialización en tiempo de ejecución (no constante).
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        // Configurar la pila para el manejo de "Double Fault".
        // Esto es crítico porque un Double Fault ocurre cuando el kernel se queda sin pila,
        // así que necesitamos cambiar a una pila nueva y segura para manejarlo.
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5; // Tamaño de la pila: 20 KB (5 páginas de 4KB).
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE]; // Espacio de memoria estático para la pila.

            // Calcular la dirección virtual de inicio y fin de la pila.
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE;
            stack_end // En x86 la pila crece hacia abajo, así que el puntero apunta al final.
        };
        tss
    };
}

lazy_static! {
    // Definir la Global Descriptor Table (GDT) estática.
    // La GDT contiene entradas para el segmento de código y el segmento TSS.
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        // Agregar un descriptor para el segmento de código del kernel.
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // Agregar un descriptor para el segmento TSS.
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

// Estructura auxiliar para guardar los selectores de segmento que necesitamos recargar.
struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

// Función pública para inicializar la GDT.
pub fn init() {
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS, Segment};

    GDT.0.load(); // Cargar la GDT utilizando la instrucción LGDT.
    unsafe {
        // Recargar el registro de segmento de código (CS) con nuestro nuevo selector.
        // Esto es necesario porque el selector antiguo podría no ser válido o tener permisos incorrectos.
        CS::set_reg(GDT.1.code_selector);
        // Cargar el registro TR ("Task Register") con el selector de nuestro TSS.
        load_tss(GDT.1.tss_selector);
    }
}
