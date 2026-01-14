use volatile::Volatile; // Importar Volatile para evitar optimizaciones del compilador en memoria mapeada.
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

// Enumeración de colores estándar VGA (modo texto).
#[allow(dead_code)] // Suprimir advertencias por variantes de color no usadas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] // Representar cada variante como un byte (u8).
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// Estructura para representar un código de color completo (primer plano + fondo).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)] // Asegurar que tenga el mismo diseño en memoria que un u8.
struct ColorCode(u8);

impl ColorCode {
    // Crear un nuevo ColorCode combinando primer plano y fondo.
    fn new(foreground: Color, background: Color) -> ColorCode {
        // El fondo ocupa los 4 bits superiores, el primer plano los 4 inferiores.
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

// Estructura que representa un carácter en pantalla (código ASCII + atributos de color).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] // Usar el diseño de estructura de C (orden de campos garantizado).
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

// Dimensiones estándar del buffer de texto VGA (80x25).
const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

// Estructura que representa el buffer de memoria VGA completo.
#[repr(transparent)]
struct Buffer {
    // Matriz de caracteres. Usamos 'Volatile' porque la memoria VGA puede ser
    // modificada por el hardware o queremos asegurar que las escrituras ocurran.
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// Escritor público que maneja la lógica de escribir en el buffer VGA.
pub struct Writer {
    column_position: usize, // Posición actual del cursor en la línea.
    color_code: ColorCode,  // Color actual para nuevos caracteres.
    buffer: &'static mut Buffer, // Referencia mutable estática al buffer VGA.
}

impl Writer {
    // Escribir un byte (carácter ASCII) en la pantalla.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(), // Manejar salto de línea.
            byte => {
                // Si llegamos al final de la línea, hacer salto de línea.
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1; // Siempre escribimos en la última fila.
                let col = self.column_position;

                let color_code = self.color_code;
                // Escribir el carácter y su color en la posición actual.
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_position += 1;
            }
        }
    }

    // Escribir una cadena completa.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Carácter ASCII imprimible o salto de línea.
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // Carácter no imprimible: imprimir un cuadrado sólido (0xfe).
                _ => self.write_byte(0xfe),
            }
        }
    }

    // Mover todo el texto una línea hacia arriba y limpiar la última línea.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0; // Reiniciar cursor al inicio de la línea.
    }

    // Limpiar una fila completa (llenarla con espacios vacíos).
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// Implementar el trait fmt::Write para soportar macros de formato de Rust (write!, println!, etc.).
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    // Instancia global del Escritor VGA protegida por un Mutex.
    // Esto permite usar println! desde cualquier parte del kernel de forma segura.
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        // Mapear la dirección física 0xb8000 (buffer VGA) a nuestra estructura Buffer.
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// Macro print! para imprimir sin salto de línea.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::drivers::vga_drivers::vga::_print(format_args!($($arg)*)));
}

// Macro println! para imprimir con salto de línea.
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// Función auxiliar privada para implementar las macros de impresión.
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    // Bloquear el Mutex y escribir los argumentos formateados.
    // Usamos un bloqueado de interrupciones implícito si estuviéramos en un contexto más avanzado,
    // pero aquí confiamos en el Mutex de spin.
    WRITER.lock().write_fmt(args).unwrap();
}