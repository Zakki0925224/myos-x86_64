use core::fmt::{self, Write};

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    graphics::{color::*, frame_buf_console::FRAME_BUF_CONSOLE},
    mem::buffer::fifo::{Fifo, FifoError},
    serial,
    util::ascii::AsciiCode,
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

#[derive(Debug, Clone, Copy)]
pub enum ConsoleError {
    IoBufferError {
        buf_type: BufferType,
        err: FifoError,
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
        return Self {
            input_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            output_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            err_output_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            buf_default_value: IO_BUF_DEFAULT_VALUE,
            use_serial_port,
        };
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

    pub fn write(
        &mut self,
        ascii_code: AsciiCode,
        buf_type: BufferType,
    ) -> Result<(), ConsoleError> {
        let buf = match buf_type {
            BufferType::Input => &mut self.input_buf,
            BufferType::Output => &mut self.output_buf,
            BufferType::ErrorOutput => &mut self.err_output_buf,
        };
        let mut value = self.buf_default_value;
        value.ascii_code = ascii_code;

        match buf.enqueue(value) {
            Ok(_) => (),
            Err(err) => return Err(ConsoleError::IoBufferError { buf_type, err }),
        };

        if (buf_type == BufferType::Output || buf_type == BufferType::ErrorOutput)
            && self.use_serial_port
        {
            serial::send_data(value.ascii_code as u8);
        }

        return Ok(());
    }

    pub fn read(&mut self, buf_type: BufferType) -> Option<ConsoleCharacter> {
        let buf = match buf_type {
            BufferType::Input => &mut self.input_buf,
            BufferType::Output => &mut self.output_buf,
            BufferType::ErrorOutput => &mut self.err_output_buf,
        };

        let value = match buf.dequeue() {
            Ok(value) => value,
            Err(_) => return None,
        };

        return Some(value);
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

        return Ok(());
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    if let Some(mut console) = CONSOLE.try_lock() {
        console.write_fmt(args).unwrap();
    }

    if let Some(mut frame_buf_console) = FRAME_BUF_CONSOLE.try_lock() {
        frame_buf_console.write_fmt(args).unwrap();
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

pub fn input(ascii_code: AsciiCode) {
    if let Some(mut console) = CONSOLE.try_lock() {
        if let Err(_) = console.write(ascii_code, BufferType::Input) {
            console.reset_buf(BufferType::Input);
            console.write(ascii_code, BufferType::Input).unwrap();
        }
    }
}
