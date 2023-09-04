use alloc::{string::String, vec::Vec};

use crate::{arch::qemu, bus::pci::PCI_DEVICE_MAN, env, print, println, util::ascii::AsciiCode};

pub struct Terminal {
    input_buf: Vec<char>,
}

impl Terminal {
    pub fn new() -> Self {
        return Self {
            input_buf: Vec::new(),
        };
    }

    pub fn clear(&mut self) {
        self.input_buf.clear();
        print!("$> ");
    }

    pub fn input_char(&mut self, code: AsciiCode) {
        match code {
            AsciiCode::CarriageReturn => {
                println!();
                self.exec_command();
                self.clear();
            }
            code => {
                let code = code as u8;
                if code >= AsciiCode::Space as u8 && code <= AsciiCode::Tilde as u8 {
                    self.input_buf.push(code as char);
                    print!("{}", code as char);
                }
            }
        }
    }

    fn exec_command(&self) {
        if self.input_buf.len() == 0 {
            return;
        }

        let input_str: String = self.input_buf.iter().collect();

        match input_str.as_str() {
            "info" => env::print_info(),
            "lspci" => PCI_DEVICE_MAN.lock().debug(),
            "exit" => qemu::exit(0),
            s => println!("Command \"{}\" was not found", s),
        }
    }
}
