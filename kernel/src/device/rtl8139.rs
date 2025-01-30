use super::{DeviceDriverFunction, DeviceDriverInfo};
use crate::{
    addr::IoPortAddress,
    device,
    error::{Error, Result},
    util::mutex::Mutex,
};
use alloc::{boxed::Box, vec::Vec};
use log::{debug, info};

const RX_BUF_SIZE: usize = 8192;
const TX_BUF_SIZE: usize = 8192;

static mut RTL8139_DRIVER: Mutex<Rtl8139Driver> = Mutex::new(Rtl8139Driver::new());

struct IoRegister(IoPortAddress);

impl IoRegister {
    fn new(base: IoPortAddress) -> Self {
        Self(base)
    }

    fn io_port_base(&self) -> &IoPortAddress {
        &self.0
    }

    fn read_mac_addr(&self) -> [u8; 6] {
        let mut mac_addr = [0; 6];
        for i in 0..6 {
            mac_addr[i] = self.io_port_base().offset(i).in8();
        }
        mac_addr
    }

    fn read_multicast_addr(&self) -> [u8; 8] {
        let mut multicast_addr = [0; 8];
        for i in 0..8 {
            multicast_addr[i] = self.io_port_base().offset(0x08 + i).in8();
        }
        multicast_addr
    }

    fn write_tx_status(&self, data: u32, index: usize) {
        self.io_port_base().offset(0x10 + 4 * index).out32(data);
    }

    fn write_tx_start_addr(&self, addr: u32, index: usize) {
        self.io_port_base().offset(0x20 + 4 * index).out32(addr);
    }

    fn write_rx_buf_addr(&self, addr: u32) {
        self.io_port_base().offset(0x30).out32(addr);
    }

    fn read_cmd(&self) -> u8 {
        self.io_port_base().offset(0x37).in8()
    }

    fn write_cmd(&self, data: u8) {
        self.io_port_base().offset(0x37).out8(data);
    }

    fn write_int_mask(&self, imr: u16) {
        self.io_port_base().offset(0x3c).out16(imr);
    }

    fn read_int_status(&self) -> u16 {
        self.io_port_base().offset(0x3e).in16()
    }

    fn write_int_status(&self, data: u16) {
        self.io_port_base().offset(0x3e).out16(data);
    }

    fn write_rx_conf(&self, rcr: u32) {
        self.io_port_base().offset(0x44).out32(rcr);
    }

    fn write_conf1(&self, data: u8) {
        self.io_port_base().offset(0x52).out8(data);
    }
}

#[repr(C, align(16))]
struct RxBuffer {
    buf: [u8; RX_BUF_SIZE],
    packet_ptr: usize,
}

impl RxBuffer {
    const fn new() -> Self {
        Self {
            buf: [0; RX_BUF_SIZE],
            packet_ptr: 0,
        }
    }

    fn buf_ptr(&self) -> *const u8 {
        self.buf.as_ptr()
    }
}

struct TxBuffer {
    buf: Option<Vec<Box<[u8]>>>,
    buf_len: usize,
    packet_ptr: usize,
}

impl TxBuffer {
    const fn new() -> Self {
        Self {
            buf: None,
            buf_len: 4,
            packet_ptr: 0,
        }
    }

    fn push(&mut self, packet: Box<[u8]>) {
        if self.buf.is_none() {
            self.buf = Some(Vec::with_capacity(self.buf_len));
        }

        let buf = self.buf.as_mut().unwrap();
        if buf.len() < self.buf_len {
            buf.push(packet);
        } else {
            buf[self.packet_ptr] = packet;
        }

        self.packet_ptr = (self.packet_ptr + 1) % self.buf_len;
    }
}

// https://wiki.osdev.org/RTL8139
struct Rtl8139Driver {
    device_driver_info: DeviceDriverInfo,
    pci_device_bdf: Option<(usize, usize, usize)>,
    io_register: Option<IoRegister>,
    rx_buf: RxBuffer,
    tx_buf: TxBuffer,
}

impl Rtl8139Driver {
    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("rtl8139"),
            pci_device_bdf: None,
            io_register: None,
            rx_buf: RxBuffer::new(),
            tx_buf: TxBuffer::new(),
        }
    }

    fn io_register(&self) -> Result<&IoRegister> {
        self.io_register
            .as_ref()
            .ok_or(Error::Failed("I/O register is not initialized"))
    }

    fn send_packet(&mut self, packet: Box<[u8]>) -> Result<()> {
        let io_register = self.io_register()?;
        let packet_len = packet.len();
        let tx_packet_ptr = self.tx_buf.packet_ptr;

        io_register.write_tx_start_addr(packet.as_ref().as_ptr() as u32, tx_packet_ptr);
        io_register.write_tx_status(packet_len as u32, tx_packet_ptr);
        self.tx_buf.push(packet);

        Ok(())
    }
}

impl DeviceDriverFunction for Rtl8139Driver {
    type AttachInput = ();
    type PollNormalOutput = ();
    type PollInterruptOutput = ();

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(self.device_driver_info.clone())
    }

    fn probe(&mut self) -> Result<()> {
        device::pci_bus::find_device_by_vendor_and_device_id(0x10ec, 0x8139, |d| {
            self.pci_device_bdf = Some(d.bdf());
            Ok(())
        })?;

        Ok(())
    }

    fn attach(&mut self, _arg: Self::AttachInput) -> Result<()> {
        let (bus, device, func) = self.pci_device_bdf.ok_or("Device driver is not probed")?;

        device::pci_bus::configure_device(bus, device, func, |d| {
            // enable PCI bus mastering and disable interrupt
            let mut conf_space_header = d.read_conf_space_header()?;
            conf_space_header.command.write_bus_master_enable(true);
            conf_space_header.command.write_int_disable(true);
            d.write_conf_space_header(conf_space_header)?;

            // read I/O port base
            let conf_space = d.read_conf_space_non_bridge_field()?;
            let bars = conf_space.get_bars()?;
            let (_, mmio_bar) = bars
                .get(0)
                .ok_or("Failed to read MMIO base address register")?;
            let io_port_base: IoPortAddress = match mmio_bar {
                device::pci_bus::conf_space::BaseAddress::MmioAddressSpace(addr) => *addr,
                _ => return Err(Error::Failed("Invalid base address register")),
            }
            .into();
            self.io_register = Some(IoRegister::new(io_port_base));
            let io_register = self.io_register()?;

            // start device
            io_register.write_conf1(0x0);

            // software reset
            io_register.write_cmd(0x10);
            loop {
                // checking reset bit
                if io_register.read_cmd() & 0x10 == 0 {
                    break;
                }
            }

            // set RX buffer address
            let rx_buf_addr = self.rx_buf.buf_ptr() as u64;
            if rx_buf_addr % 16 != 0 {
                return Err(Error::Failed("RX buffer address is not aligned"));
            }

            if rx_buf_addr > u32::MAX as u64 {
                return Err(Error::Failed("RX buffer address is too large"));
            }

            io_register.write_rx_buf_addr(rx_buf_addr as u32);

            // configre interrupt mask
            io_register.write_int_mask(0x5); // TOK, ROK

            // configure RX buffer
            io_register.write_rx_conf(0xf | (1 << 7)); // AB+AM+APM+AAP, WRAP

            // enable rx/tx
            io_register.write_cmd(0x0c); // TE, RE

            // configure interrupt
            // TODO: not working
            // let vec_num = idt::set_handler_dyn_vec(
            //     idt::InterruptHandler::Normal(poll_int_rt8139_driver),
            //     idt::GateType::Interrupt,
            // )?;
            // debug!(
            //     "{}: Interrupt vector number: {}",
            //     self.device_driver_info.name, vec_num
            // );
            // d.write_interrupt_line(vec_num)?;

            let mac_addr = io_register.read_mac_addr();
            debug!(
                "{}: MAC address: {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                self.device_driver_info.name,
                mac_addr[0],
                mac_addr[1],
                mac_addr[2],
                mac_addr[3],
                mac_addr[4],
                mac_addr[5]
            );

            // it works!
            self.send_packet(Box::new([0x00, 0x01, 0x02, 0x03, 0x04]))?;

            Ok(())
        })?;

        self.device_driver_info.attached = true;
        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        let io_register = self.io_register()?;
        let status = io_register.read_int_status();
        // clear interrupt status
        io_register.write_int_status(0x05);

        // TOK
        if status & (1 << 2) != 0 {
            debug!("{}: TOK", self.device_driver_info.name);
        }

        // ROK
        if status & 1 != 0 {
            debug!("{}: ROK", self.device_driver_info.name);
        }

        Ok(())
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        let io_register = self.io_register()?;
        let status = io_register.read_int_status();
        // clear interrupt status
        io_register.write_int_status(0x05);

        debug!("{}: status: 0x{:04x}", self.device_driver_info.name, status);

        // TOK
        if status & (1 << 2) != 0 {
            debug!("{}: TOK", self.device_driver_info.name);
        }

        // ROK
        if status & 1 != 0 {
            debug!("{}: ROK", self.device_driver_info.name);
        }

        Ok(())
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn write(&mut self, _data: &[u8]) -> Result<()> {
        unimplemented!()
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    let driver = unsafe { RTL8139_DRIVER.try_lock() }?;
    driver.get_device_driver_info()
}

pub fn probe_and_attach() -> Result<()> {
    let mut driver = unsafe { RTL8139_DRIVER.try_lock() }?;
    driver.probe()?;
    driver.attach(())?;
    info!("{}: Attached!", driver.get_device_driver_info()?.name);
    Ok(())
}

pub fn poll_normal() -> Result<()> {
    if let Ok(mut driver) = unsafe { RTL8139_DRIVER.try_lock() } {
        driver.poll_normal()?;
    }
    Ok(())
}

// extern "x86-interrupt" fn poll_int_rt8139_driver() {
//     if let Ok(mut driver) = unsafe { RTL8139_DRIVER.try_lock() } {
//         let _ = driver.poll_int();
//     }
//     idt::notify_end_of_int();
// }
