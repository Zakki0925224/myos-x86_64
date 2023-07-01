use crate::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

pub const IO_PORT_COM1: u16 = 0x3f8;
pub const IO_PORT_COM2: u16 = 0x2f8;
pub const IO_PORT_COM3: u16 = 0x3e8;
pub const IO_PORT_COM4: u16 = 0x2e8;
pub const IO_PORT_COM5: u16 = 0x5f8;
pub const IO_PORT_COM6: u16 = 0x4f8;
pub const IO_PORT_COM7: u16 = 0x5e8;
pub const IO_PORT_COM8: u16 = 0x4e8;

lazy_static! {
    pub static ref SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort::new());
}

pub struct SerialPort {
    io_port: u16,
    is_init: bool,
}

impl SerialPort {
    pub fn new() -> Self {
        return Self {
            io_port: 0,
            is_init: false,
        };
    }

    pub fn is_init(&self) -> bool {
        return self.is_init;
    }

    pub fn get_port_num(&self) -> u16 {
        return self.io_port;
    }

    pub fn init(&mut self, io_port: u16) {
        self.io_port = io_port;
        asm::out8(self.io_port + 1, 0x00); // IER - disable all interrupts
        asm::out8(self.io_port + 3, 0x80); // LCR - enable DLAB
        asm::out8(self.io_port + 0, 0x03); // DLL - set baud late 38400 bps
        asm::out8(self.io_port + 1, 0x00); // DLM
        asm::out8(self.io_port + 3, 0x03); // LCR - disable DLAB, 8bit, no parity, 1 stop bit
        asm::out8(self.io_port + 2, 0xc7); // FCR - enable FIFO, clear TX/RX queues, 14byte threshold
        asm::out8(self.io_port + 4, 0x0b); // MCR - IRQs enabled, RTS/DSR set
        asm::out8(self.io_port + 4, 0x1e); // MCR - set loopback mode, test the serial chip
        asm::out8(self.io_port + 0, 0xae); // RBR - test the serial chip (send 0xae)

        if asm::in8(self.io_port + 0) != 0xae {
            return;
        }

        // if serial isn't faulty, set normal mode
        asm::out8(self.io_port + 4, 0x0f);
        self.is_init = true;
    }

    pub fn receive_data(&self) -> Option<u8> {
        if !self.is_init {
            return None;
        }

        let res = asm::in8(self.io_port + 5) & 1;

        if res == 0 {
            return None;
        }

        return Some(asm::in8(self.io_port));
    }

    pub fn send_data(&self, data: u8) {
        // skip send data
        if !self.is_init {
            return;
        }

        while self.is_transmit_empty() == 0 {}
        asm::out8(self.io_port, data);
    }

    fn is_transmit_empty(&self) -> u8 {
        return asm::in8(self.io_port + 5) & 0x20;
    }
}
