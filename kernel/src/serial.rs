use crate::arch::asm;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    static ref SERIAL: Mutex<Option<SerialPort>> = Mutex::new(None);
}

#[derive(Debug, Clone, Copy)]
#[repr(u16)]
pub enum ComPort {
    Com1 = 0x3f8,
    // Com2 = 0x2f8,
    // Com3 = 0x3e8,
    // Com4 = 0x2e8,
    // Com5 = 0x5f8,
    // Com6 = 0x4f8,
    // Com7 = 0x5e8,
    // Com8 = 0x4e8,
}

pub struct SerialPort {
    com_port: ComPort,
}

impl SerialPort {
    pub fn new(com_port: ComPort) -> Option<Self> {
        let serial = Self { com_port };
        serial.write_reg(1, 0x00); // IER - disable all interrupts
        serial.write_reg(3, 0x80); // LCR - enable DLAB
        serial.write_reg(0, 0x03); // DLL - set baud late 38400 bps
        serial.write_reg(1, 0x00); // DLM
        serial.write_reg(3, 0x03); // LCR - disable DLAB, 8bit, no parity, 1 stop bit
        serial.write_reg(2, 0xc7); // FCR - enable FIFO, clear TX/RX queues, 14byte threshold
        serial.write_reg(4, 0x0b); // MCR - IRQs enabled, RTS/DSR set
        serial.write_reg(4, 0x1e); // MCR - set loopback mode, test the serial chip
        serial.write_reg(0, 0xae); // RBR - test the serial chip (send 0xae)

        if serial.read_reg(0) != 0xae {
            return None;
        }

        // if serial isn't faulty, set normal mode
        serial.write_reg(4, 0x0f);

        return Some(serial);
    }

    pub fn receive_data(&self) -> Option<u8> {
        if !self.is_received_data() {
            return None;
        }

        return Some(self.read_reg(0));
    }

    pub fn send_data(&self, data: u8) {
        while !self.is_transmit_empty() {}
        self.write_reg(0, data);
    }

    fn is_received_data(&self) -> bool {
        return self.read_reg(5) & 0x01 != 0;
    }

    fn is_transmit_empty(&self) -> bool {
        return self.read_reg(5) & 0x20 != 0;
    }

    fn write_reg(&self, offset: u16, data: u8) {
        asm::out8(self.com_port as u16 + offset, data);
    }

    fn read_reg(&self, offset: u16) -> u8 {
        return asm::in8(self.com_port as u16 + offset);
    }
}

pub fn init(com_port: ComPort) {
    if let Some(mut serial) = SERIAL.try_lock() {
        *serial = SerialPort::new(com_port);
    }
}

pub fn receive_data() -> Option<u8> {
    if let Some(serial) = SERIAL.try_lock() {
        if let Some(serial) = serial.as_ref() {
            return serial.receive_data();
        }
    }

    return None;
}

pub fn send_data(data: u8) {
    if let Some(serial) = SERIAL.try_lock() {
        if let Some(serial) = serial.as_ref() {
            serial.send_data(data);
        }
    }
}
