use spin::Mutex;
use lazy_static::lazy_static;
use crate::{print, println};
use pc_keyboard::DecodedKey;

// Tamaño del buffer circular para las entradas de teclado.
const BUFFER_SIZE: usize = 256;

// Estructura de buffer circular para almacenar caracteres del teclado.
// Esto permite que el manejador de interrupciones guarde teclas y la shell las lea después.
struct ShellBuffer {
    data: [char; BUFFER_SIZE],
    head: usize, // Índice de lectura.
    tail: usize, // Índice de escritura.
}

impl ShellBuffer {
    fn new() -> Self {
        ShellBuffer {
            data: ['\0'; BUFFER_SIZE],
            head: 0,
            tail: 0,
        }
    }

    // Añadir un carácter al buffer (productor).
    fn push(&mut self, c: char) {
        // Verificar si el buffer está lleno (tail + 1 == head).
        if (self.tail + 1) % BUFFER_SIZE != self.head {
            self.data[self.tail] = c;
            self.tail = (self.tail + 1) % BUFFER_SIZE; // Avanzar el índice circularmente.
        }
    }

    // Leer un carácter del buffer (consumidor).
    fn pop(&mut self) -> Option<char> {
        if self.head == self.tail {
            None // Buffer vacío.
        } else {
            let c = self.data[self.head];
            self.head = (self.head + 1) % BUFFER_SIZE; // Avanzar el índice de lectura.
            Some(c)
        }
    }
}

lazy_static! {
    // Instancia global estática del buffer de entrada, protegida por Mutex para acceso seguro entre hilos/interrupciones.
    static ref INPUT_BUFFER: Mutex<ShellBuffer> = Mutex::new(ShellBuffer::new());
}

// Función pública para que el manejador de teclado (en interrupts.rs) agregue teclas.
pub fn add_key(key: DecodedKey) {
    use x86_64::instructions::interrupts;
    // Desactivamos interrupciones para asegurar que no haya condiciones de carrera al escribir en el buffer.
    interrupts::without_interrupts(|| {
         match key {
            DecodedKey::Unicode(c) => INPUT_BUFFER.lock().push(c),
            DecodedKey::RawKey(_) => {} // Ignoramos teclas crudas (como F1, Shift, etc.) por ahora.
        }
    });
}

// Función principal de la shell. Ejecuta un bucle infinito procesando comandos.
pub fn run_shell() {
    print!("> "); // Muestra el prompt inicial.
    
    // Buffer local para almacenar el comando actual que se está escribiendo.
    // Tamaño fijo de 100 caracteres.
    let mut command_buffer: [char; 100] = ['\0'; 100];
    let mut command_len = 0;

    loop {
        let mut c_opt = None;
        // Leemos del buffer global deshabilitando interrupciones brevemente para evitar bloqueos.
        x86_64::instructions::interrupts::without_interrupts(|| {
             c_opt = INPUT_BUFFER.lock().pop();
        });

        if let Some(c) = c_opt {
            match c {
                '\n' => { // El usuario presionó Enter.
                    println!("");
                    if command_len > 0 {
                        // Eco de la ejecución del comando.
                        // Aquí analizaríamos command_buffer[..command_len].
                        // Parser simple: nota que no tenemos String dinámico aún.
                        print!("Command received: ");
                        for i in 0..command_len {
                            print!("{}", command_buffer[i]);
                        }
                        println!("");
                        
                        // Manejar comandos específicos manualmente.
                         if command_len == 4 
                            && command_buffer[0] == 'h' 
                            && command_buffer[1] == 'e' 
                            && command_buffer[2] == 'l' 
                            && command_buffer[3] == 'p' 
                        {
                            println!("Available commands: help, echo");
                        }
                    }
                    // Reiniciar el buffer de comando para la siguiente línea.
                    command_len = 0;
                    print!("> ");
                }
                // Manejo simple de Backspace (borrar carácter).
                '\x08' => {
                    if command_len > 0 {
                         // Deberíamos manejar el backspace visual (borrar en pantalla) también si el driver VGA lo soporta.
                         // Por ahora solo lógica interna.
                         command_len -= 1;
                    }
                }
                c => {
                    // Carácter normal: agregarlo al buffer si hay espacio.
                    if command_len < command_buffer.len() {
                        command_buffer[command_len] = c;
                        command_len += 1;
                        print!("{}", c); // Eco del carácter en pantalla.
                    }
                }
            }
        } else {
            // Si no hay entrada, poner el CPU en modo de espera para ahorrar energía.
            x86_64::instructions::hlt();
        }
    }
}
