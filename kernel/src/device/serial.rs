use crate::arch::asm;

pub const IO_PORT_COM1: u16 = 0x3f8;
pub const IO_PORT_COM2: u16 = 0x2f8;
pub const IO_PORT_COM3: u16 = 0x3e8;
pub const IO_PORT_COM4: u16 = 0x2e8;
pub const IO_PORT_COM5: u16 = 0x5f8;
pub const IO_PORT_COM6: u16 = 0x4f8;
pub const IO_PORT_COM7: u16 = 0x5e8;
pub const IO_PORT_COM8: u16 = 0x4e8;

pub struct SerialPort
{
    io_port: u16,
    is_init: bool,
}

impl SerialPort
{
    pub fn new(io_port: u16) -> Self
    {
        return Self { io_port,
                      is_init: false };
    }

    pub fn init(&mut self)
    {
        asm::out8(self.io_port + 1, 0x00); // disable all interrupts
        asm::out8(self.io_port + 3, 0x80); // enable DLAB
        asm::out8(self.io_port + 0, 0x03); // set baud late 38400 bps
        asm::out8(self.io_port + 1, 0x00); // re disable all interrupts
        asm::out8(self.io_port + 3, 0x03); // 8bit, no parity, 1 stop bit
        asm::out8(self.io_port + 2, 0xc7); // enable FIFO, clear TX/RX queues and set interrupt watermakrk at 14 bytes
        asm::out8(self.io_port + 4, 0x0b); // IRQs enabled, RTS/DSR set
        asm::out8(self.io_port + 4, 0x1e); // set loopback mode, test the serial chip
        asm::out8(self.io_port + 0, 0xae); // test the serial chip (send 0xae)

        if asm::in8(self.io_port + 0) != 0xae
        {
            return;
        }

        // if serial isn't faulty, set normal mode
        asm::out8(self.io_port + 4, 0x0f);
        self.is_init = true;
    }

    pub fn receive_data(&self) -> Result<u8, &str>
    {
        if !self.is_init
        {
            return Err("Serial port wasn't initialized");
        }

        let res = asm::in8(self.io_port + 5) & 1;

        if res == 0
        {
            return Err("Hasn't received data");
        }

        return Ok(asm::in8(self.io_port));
    }

    pub fn send_data(&self, data: u8) -> Result<(), &str>
    {
        if !self.is_init
        {
            return Err("Serial port wasn't initialized");
        }

        while self.is_transmit_empty() == 0
        {}
        asm::out8(self.io_port, data);
        return Ok(());
    }

    fn is_transmit_empty(&self) -> u8 { return asm::in8(self.io_port + 5) & 0x20; }
}
