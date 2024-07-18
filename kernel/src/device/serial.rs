use crate::{
    arch::addr::IoPortAddress,
    error::{Error, Result},
    util::mutex::Mutex,
};

static mut SERIAL: Mutex<Option<SerialPort>> = Mutex::new(None);

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

struct SerialPort {
    io_port_addr: IoPortAddress,
}

impl SerialPort {
    pub fn new(com_port: ComPort) -> Result<Self> {
        let io_port_addr = IoPortAddress::new(com_port as u32);

        io_port_addr.offset(1).out8(0x00); // IER - disable all interrupts
        io_port_addr.offset(3).out8(0x80); // LCR - enable DLAB
        io_port_addr.offset(0).out8(0x03); // DLL - set baud late 38400 bps
        io_port_addr.offset(1).out8(0x00); // DLM
        io_port_addr.offset(3).out8(0x03); // LCR - disable DLAB, 8bit, no parity, 1 stop bit
        io_port_addr.offset(2).out8(0xc7); // FCR - enable FIFO, clear TX/RX queues, 14byte threshold
        io_port_addr.offset(4).out8(0x0b); // MCR - IRQs enabled, RTS/DSR set
        io_port_addr.offset(4).out8(0x1e); // MCR - set loopback mode, test the serial chip
        io_port_addr.offset(0).out8(0xae); // RBR - test the serial chip (send 0xae)

        if io_port_addr.offset(0).in8() != 0xae {
            return Err(Error::Failed("Failed to initialize serial port"));
        }

        // if serial isn't faulty, set normal mode
        io_port_addr.offset(4).out8(0x0f);
        let serial = Self { io_port_addr };

        Ok(serial)
    }

    pub fn receive_data(&self) -> Option<u8> {
        if !self.is_received_data() {
            return None;
        }

        Some(self.io_port_addr.in8())
    }

    pub fn send_data(&self, data: u8) {
        while !self.is_transmit_empty() {}
        self.io_port_addr.out8(data);
    }

    fn is_received_data(&self) -> bool {
        self.io_port_addr.offset(5).in8() & 0x01 != 0
    }

    fn is_transmit_empty(&self) -> bool {
        self.io_port_addr.offset(5).in8() & 0x20 != 0
    }
}

pub fn init(com_port: ComPort) -> Result<()> {
    *unsafe { SERIAL.try_lock() }? = Some(SerialPort::new(com_port)?);
    return Ok(());
}

pub fn receive() -> Result<Option<u8>> {
    Ok(unsafe { SERIAL.try_lock() }?
        .as_ref()
        .ok_or(Error::Failed("Serial port was not initialized"))?
        .receive_data())
}

pub fn send(data: u8) -> Result<()> {
    unsafe { SERIAL.try_lock() }?
        .as_ref()
        .ok_or(Error::Failed("Serial port was not initialized"))?
        .send_data(data);
    Ok(())
}
