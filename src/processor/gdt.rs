use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

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

        // Kernel Code Segment (Ring 0)
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        // Kernel Data Segment (Ring 0)
        let data_selector = gdt.add_entry(Descriptor::kernel_data_segment());

        // User Data Segment (Ring 3)
        let user_data_selector = gdt.add_entry(Descriptor::user_data_segment());
        // User Code Segment (Ring 3)
        let user_code_selector = gdt.add_entry(Descriptor::user_code_segment());

        // TSS Segment
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));

        (gdt, Selectors {
            code_selector,
            data_selector,
            user_code_selector,
            user_data_selector,
            tss_selector
        })
    };
}

// Estructura auxiliar para guardar los selectores de segmento que necesitamos recargar.
#[allow(dead_code)]
pub struct Selectors {
    pub code_selector: SegmentSelector,
    pub data_selector: SegmentSelector,
    pub user_code_selector: SegmentSelector,
    pub user_data_selector: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

// Función pública para inicializar la GDT.
pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load(); // Cargar la GDT utilizando la instrucción LGDT.
    unsafe {
        // Recargar el registro de segmento de código (CS) con nuestro nuevo selector.
        // Esto es necesario porque el selector antiguo podría no ser válido o tener permisos incorrectos.
        CS::set_reg(GDT.1.code_selector);
        // Cargar el registro TR ("Task Register") con el selector de nuestro TSS.
        load_tss(GDT.1.tss_selector);
    }
}
