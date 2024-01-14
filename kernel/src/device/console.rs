use core::fmt::{self, Write};

use alloc::{boxed::Box, string::String, vec::Vec};
use lazy_static::lazy_static;
use log::error;

use crate::{
    arch::{asm, qemu},
    bus::pci,
    env,
    error::{Error, Result},
    fs::initramfs,
    graphics::{color::*, frame_buf_console::FRAME_BUF_CONSOLE},
    mem::{self, buffer::fifo::Fifo},
    serial,
    util::{
        ascii::AsciiCode,
        mutex::{Mutex, MutexError},
    },
};

const IO_BUF_LEN: usize = 512;
const IO_BUF_DEFAULT_VALUE: ConsoleCharacter = ConsoleCharacter {
    back_color: COLOR_BLACK,
    fore_color: COLOR_WHITE,
    ascii_code: AsciiCode::Null,
};

type IoBufferType = Fifo<ConsoleCharacter, IO_BUF_LEN>;

// kernel console
lazy_static! {
    static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new(true));
}

#[derive(Debug, Clone, Copy)]
pub struct ConsoleCharacter {
    pub back_color: RgbColor,
    pub fore_color: RgbColor,
    pub ascii_code: AsciiCode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConsoleError {
    IoBufferError {
        buf_type: BufferType,
        err: Box<Error>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BufferType {
    Input,
    Output,
    ErrorOutput,
}

// TTY + PTS
#[derive(Debug)]
pub struct Console {
    input_buf: IoBufferType,
    output_buf: IoBufferType,
    err_output_buf: IoBufferType,
    buf_default_value: ConsoleCharacter,
    use_serial_port: bool,
}

impl Console {
    pub fn new(use_serial_port: bool) -> Self {
        Self {
            input_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            output_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            err_output_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            buf_default_value: IO_BUF_DEFAULT_VALUE,
            use_serial_port,
        }
    }

    pub fn reset_buf(&mut self, buf_type: BufferType) {
        let buf = match buf_type {
            BufferType::Input => &mut self.input_buf,
            BufferType::Output => &mut self.output_buf,
            BufferType::ErrorOutput => &mut self.err_output_buf,
        };

        buf.reset_ptr();
    }

    pub fn set_back_color(&mut self, back_color: RgbColor) {
        self.buf_default_value.back_color = back_color;
    }

    pub fn set_fore_color(&mut self, fore_color: RgbColor) {
        self.buf_default_value.fore_color = fore_color;
    }

    pub fn reset_color(&mut self) {
        self.buf_default_value.back_color = IO_BUF_DEFAULT_VALUE.back_color;
        self.buf_default_value.fore_color = IO_BUF_DEFAULT_VALUE.fore_color;
    }

    pub fn write(&mut self, ascii_code: AsciiCode, buf_type: BufferType) -> Result<()> {
        let buf = match buf_type {
            BufferType::Input => &mut self.input_buf,
            BufferType::Output => &mut self.output_buf,
            BufferType::ErrorOutput => &mut self.err_output_buf,
        };
        let mut value = self.buf_default_value;
        value.ascii_code = ascii_code;

        match buf.enqueue(value) {
            Ok(_) => (),
            Err(err) => {
                return Err(ConsoleError::IoBufferError {
                    buf_type,
                    err: Box::new(err),
                }
                .into())
            }
        };

        if (buf_type == BufferType::Output || buf_type == BufferType::ErrorOutput)
            && self.use_serial_port
        {
            serial::send_data(value.ascii_code as u8);
        }

        Ok(())
    }

    pub fn read(&mut self, buf_type: BufferType) -> Option<ConsoleCharacter> {
        let buf = match buf_type {
            BufferType::Input => &mut self.input_buf,
            BufferType::Output => &mut self.output_buf,
            BufferType::ErrorOutput => &mut self.err_output_buf,
        };

        match buf.dequeue() {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }

    pub fn get_str(&mut self, buf_type: BufferType) -> String {
        let buf = match buf_type {
            BufferType::Input => &mut self.input_buf,
            BufferType::Output => &mut self.output_buf,
            BufferType::ErrorOutput => &mut self.err_output_buf,
        };

        let mut s = String::new();

        loop {
            let ascii_code = match buf.dequeue() {
                Ok(value) => value.ascii_code,
                Err(_) => break,
            };

            s.push(ascii_code as u8 as char);
        }

        s
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let buf_type = BufferType::Output;
        for c in s.chars() {
            let ascii_code = match (c as u8).try_into() {
                Ok(c) => c,
                Err(_) => continue,
            };

            if self.write(ascii_code, buf_type).is_err() {
                self.reset_buf(buf_type);
                self.write(ascii_code, buf_type).unwrap();
            }
        }

        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    if let Ok(mut console) = CONSOLE.try_lock() {
        console.write_fmt(args).unwrap();
    }

    if let Ok(mut frame_buf_console) = FRAME_BUF_CONSOLE.try_lock() {
        if let Some(frame_buf_console) = frame_buf_console.as_mut() {
            frame_buf_console.write_fmt(args).unwrap();
        }
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::device::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println
{
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn clear_input_buf() -> Result<()> {
    if let Ok(mut console) = CONSOLE.try_lock() {
        console.reset_buf(BufferType::Input);
        return Ok(());
    } else {
        return Err(MutexError::Locked.into());
    }
}

pub fn input(ascii_code: AsciiCode) -> Result<()> {
    let mut cmd = None;

    if let Ok(mut console) = CONSOLE.try_lock() {
        if let Err(_) = console.write(ascii_code, BufferType::Input) {
            console.reset_buf(BufferType::Input);
            console.write(ascii_code, BufferType::Input).unwrap();
        }

        if ascii_code == AsciiCode::CarriageReturn || ascii_code == AsciiCode::NewLine {
            cmd = Some(console.get_str(BufferType::Input));
        }
    } else {
        return Err(MutexError::Locked.into());
    }

    match ascii_code {
        AsciiCode::CarriageReturn => {
            println!();
        }
        code => {
            print!("{}", code as u8 as char);
        }
    }

    // execute command
    if let Some(cmd) = cmd {
        let cmds: Vec<&str> = cmd.trim().split(" ").collect();
        match cmds[0] {
            "info" => env::print_info(),
            "lspci" => {
                if pci::lspci().is_err() {
                    error!("PCI manager is locked")
                }
            }
            "free" => {
                if mem::free().is_err() {
                    error!("Memory manager is locked");
                }
            }
            "exit" => {
                qemu::exit(0);
            }
            "echo" => {
                println!("{}", &cmd[4..].trim());
            }
            "break" => {
                asm::int3();
            }
            "ls" => {
                initramfs::ls();
            }
            "cd" => {
                if cmds.len() == 2 {
                    initramfs::cd(cmds[1]);
                }
            }
            "cat" => {
                if cmds.len() == 2 {
                    initramfs::cat(cmds[1]);
                }
            }
            // execute file
            "exec" => {
                if cmds.len() == 2 {
                    initramfs::exec(cmds[1]);
                }
            }
            _ => error!("Command {:?} was not found", cmds),
        }

        println!();
    }

    Ok(())
}
