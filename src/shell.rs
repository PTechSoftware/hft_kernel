use spin::Mutex;
use lazy_static::lazy_static;
use crate::{print, println};
use pc_keyboard::DecodedKey;

const BUFFER_SIZE: usize = 256;

struct ShellBuffer {
    data: [char; BUFFER_SIZE],
    head: usize,
    tail: usize,
}

impl ShellBuffer {
    fn new() -> Self {
        ShellBuffer {
            data: ['\0'; BUFFER_SIZE],
            head: 0,
            tail: 0,
        }
    }

    fn push(&mut self, c: char) {
        if (self.tail + 1) % BUFFER_SIZE != self.head {
            self.data[self.tail] = c;
            self.tail = (self.tail + 1) % BUFFER_SIZE;
        }
    }

    fn pop(&mut self) -> Option<char> {
        if self.head == self.tail {
            None
        } else {
            let c = self.data[self.head];
            self.head = (self.head + 1) % BUFFER_SIZE;
            Some(c)
        }
    }
}

lazy_static! {
    static ref INPUT_BUFFER: Mutex<ShellBuffer> = Mutex::new(ShellBuffer::new());
}

pub fn add_key(key: DecodedKey) {
    use x86_64::instructions::interrupts;
    interrupts::without_interrupts(|| {
         match key {
            DecodedKey::Unicode(c) => INPUT_BUFFER.lock().push(c),
            DecodedKey::RawKey(_) => {} // Ignore raw keys for now
        }
    });
}

pub fn run_shell() {
    print!("> ");
    
    let mut command_buffer: [char; 100] = ['\0'; 100];
    let mut command_len = 0;

    loop {
        let mut c_opt = None;
        x86_64::instructions::interrupts::without_interrupts(|| {
             c_opt = INPUT_BUFFER.lock().pop();
        });

        if let Some(c) = c_opt {
            match c {
                '\n' => {
                    println!("");
                    if command_len > 0 {
                        // Echo command execution
                        // Here we would parse command_buffer[..command_len]
                        // Simple parser: note that we don't have String
                        print!("Command received: ");
                        for i in 0..command_len {
                            print!("{}", command_buffer[i]);
                        }
                        println!("");
                        
                        // Handle specific commands
                         if command_len == 4 
                            && command_buffer[0] == 'h' 
                            && command_buffer[1] == 'e' 
                            && command_buffer[2] == 'l' 
                            && command_buffer[3] == 'p' 
                        {
                            println!("Available commands: help, echo");
                        }
                    }
                    command_len = 0;
                    print!("> ");
                }
                // Backspace (simple handling)
                '\x08' => {
                    if command_len > 0 {
                         // We should handle visual backspace too if VGA supports it
                         // For now just logical
                         command_len -= 1;
                    }
                }
                c => {
                    if command_len < command_buffer.len() {
                        command_buffer[command_len] = c;
                        command_len += 1;
                        print!("{}", c);
                    }
                }
            }
        } else {
            x86_64::instructions::hlt();
        }
    }
}
