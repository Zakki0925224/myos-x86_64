use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;

use crate::{
    graphics::color::*,
    mem::buffer::fifo::{Fifo, FifoError},
    util::ascii::AsciiCode,
};

use super::serial::SERIAL;

const IO_BUF_LEN: usize = 512;
const IO_BUF_DEFAULT_VALUE: ConsoleCharacter = ConsoleCharacter {
    back_color: COLOR_BLACK,
    fore_color: COLOR_WHITE,
    ascii_code: AsciiCode::Null,
};

type IoBufferType = Fifo<ConsoleCharacter, IO_BUF_LEN>;

// kernel console
lazy_static! {
    pub static ref CONSOLE: Mutex<Console> = Mutex::new(Console::new(true));
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
    cursor_pos: usize,
    use_serial_port: bool,
}

impl Console {
    pub fn new(use_serial_port: bool) -> Self {
        return Self {
            input_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            output_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            err_output_buf: Fifo::new(IO_BUF_DEFAULT_VALUE),
            buf_default_value: IO_BUF_DEFAULT_VALUE,
            cursor_pos: 0,
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

    // pub fn get_buf_len(&self, buf_type: BufferType) -> usize {
    //     let buf = match buf_type {
    //         BufferType::Input => &self.input_buf,
    //         BufferType::Output => &self.output_buf,
    //         BufferType::ErrorOutput => &self.err_output_buf,
    //     };

    //     return buf.len();
    // }

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

        if !SERIAL.is_locked()
            && (buf_type == BufferType::Output || buf_type == BufferType::ErrorOutput)
            && self.use_serial_port
        {
            SERIAL.lock().send_data(value.ascii_code as u8);
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

// TODO
impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let buf_type = BufferType::Output;
        for c in s.chars() {
            let ascii_code = (c as u8).into();
            if self.write(ascii_code, buf_type).is_err() {
                self.reset_buf(buf_type);
                self.write(ascii_code, buf_type).unwrap();
            }
        }

        return Ok(());
    }
}
