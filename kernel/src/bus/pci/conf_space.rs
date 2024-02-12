use crate::arch::{addr::*, register::msi::*};
use alloc::vec::Vec;
use core::mem::transmute;
use pci_ids::*;

const PCI_PORT_CONF_REG_ADDR: PhysicalAddress = PhysicalAddress::new(0xcf8);
const PCI_PORT_CONF_DATA_REG_ADDR: PhysicalAddress = PhysicalAddress::new(0xcfc);
const PCI_DEVICE_NON_EXIST: u16 = 0xffff;
pub const PCI_DEVICE_BUS_LEN: usize = 256;
pub const PCI_DEVICE_DEVICE_LEN: usize = 32;
pub const PCI_DEVICE_FUNC_LEN: usize = 8;
const PCI_CONF_UNIQUE_FIELD_OFFSET: usize = 16;

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ConfigurationSpaceCommandRegister(u16);

#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct ConfigurationSpaceStatusRegister(u16);

impl ConfigurationSpaceStatusRegister {
    pub fn caps_list_available(&self) -> bool {
        (self.0 & 0x10) != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ConfigurationSpaceHeaderType {
    NonBridge,
    PciToPciBridge,
    PciToCardBusBridge,
    MultiFunction,
    Invalid(u8),
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpaceCommonHeaderField {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: ConfigurationSpaceCommandRegister,
    pub status: ConfigurationSpaceStatusRegister,
    pub revision_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class_code: u8,
    cache_line_size: u8,
    latency_timer: u8,
    header_type: u8,
    bist: u8,
}

impl ConfigurationSpaceCommonHeaderField {
    pub fn read(bus: usize, device: usize, func: usize) -> Option<Self> {
        let mut data: [u32; 4] = [0; 4];
        for (i, elem) in data.iter_mut().enumerate() {
            if let Some(d) = read_conf_space(bus, device, func, i * 4) {
                *elem = d;
            } else {
                return None;
            }
        }

        Some(unsafe { transmute::<[u32; 4], Self>(data) })
    }

    pub fn is_exist(&self) -> bool {
        self.vendor_id != PCI_DEVICE_NON_EXIST
    }

    pub fn get_device_name(&self) -> Option<&str> {
        let vendor = self.get_vendor();
        if !self.is_exist() || vendor.is_none() {
            return None;
        }

        let device = self.get_device(&vendor.unwrap());

        if device.is_some() {
            Some(device.unwrap().name())
        } else {
            None
        }
    }

    pub fn get_vendor_name(&self) -> Option<&str> {
        if !self.is_exist() {
            return None;
        }

        let vendor = self.get_vendor();

        if vendor.is_some() {
            Some(vendor.unwrap().name())
        } else {
            None
        }
    }

    pub fn get_class_name(&self) -> Option<&str> {
        if !self.is_exist() {
            return None;
        }

        let class = self.get_class();

        if class.is_some() {
            Some(class.unwrap().name())
        } else {
            None
        }
    }

    pub fn get_subclass_name(&self) -> Option<&str> {
        let subclass = self.get_subclass();
        if !self.is_exist() || subclass.is_none() {
            return None;
        }

        Some(subclass.unwrap().name())
    }

    pub fn get_header_type(&self) -> ConfigurationSpaceHeaderType {
        match self.header_type {
            0x00 => ConfigurationSpaceHeaderType::NonBridge,
            0x01 => ConfigurationSpaceHeaderType::PciToPciBridge,
            0x02 => ConfigurationSpaceHeaderType::PciToCardBusBridge,
            other => {
                if other & 0x80 != 0 {
                    ConfigurationSpaceHeaderType::MultiFunction
                } else {
                    ConfigurationSpaceHeaderType::Invalid(other)
                }
            }
        }
    }

    fn get_vendor(&self) -> Option<&Vendor> {
        Vendors::iter().find(|v| v.id() == self.vendor_id)
    }

    fn get_device(&self, vendor: &Vendor) -> Option<&Device> {
        vendor.devices().find(|d| d.id() == self.device_id)
    }

    fn get_class(&self) -> Option<&Class> {
        Classes::iter().find(|c| c.id() == self.class_code)
    }

    fn get_subclass(&self) -> Option<&Subclass> {
        match self.get_class() {
            Some(class) => class.subclasses().find(|c| c.id() == self.subclass),
            None => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseAddress {
    MemoryAddress32BitSpace(PhysicalAddress, bool), // (addr, is prefetchable)
    MemoryAddress64BitSpace(PhysicalAddress, bool),
    MmioAddressSpace(u32),
}
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct BaseAddressRegister(u32);

impl BaseAddressRegister {
    pub fn read(&self) -> u32 {
        self.0
    }

    pub fn get_base_addr(&self) -> Option<BaseAddress> {
        let bar = self.read();

        if bar == 0 {
            return None;
        }

        if bar & 0x1 != 0 {
            let addr = bar & !0x3;
            return Some(BaseAddress::MmioAddressSpace(addr));
        }

        let bar_type = (bar >> 1) & 0x3;
        let prefetchable = bar & 0x8 != 0;
        let phys_addr = PhysicalAddress::new((bar & !0xf) as u64);

        match bar_type {
            0x0 => Some(BaseAddress::MemoryAddress32BitSpace(
                phys_addr,
                prefetchable,
            )),
            0x2 => Some(BaseAddress::MemoryAddress64BitSpace(
                phys_addr,
                prefetchable,
            )),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpaceNonBridgeField {
    bar0: BaseAddressRegister,
    bar1: BaseAddressRegister,
    bar2: BaseAddressRegister,
    bar3: BaseAddressRegister,
    bar4: BaseAddressRegister,
    bar5: BaseAddressRegister,
    cardbus_cis_ptr: u32,
    subsystem_vendor_id: u16,
    subsystem_id: u16,
    expansion_rom_base_addr: u32,
    pub caps_ptr: u8,
    reserved0: [u8; 3],
    reserved1: u32,
    int_line: u8,
    int_pin: u8,
    min_grant: u8,
    max_latency: u8,
}

impl ConfigurationSpaceNonBridgeField {
    pub fn read(bus: usize, device: usize, func: usize) -> Option<Self> {
        let mut data: [u32; 12] = [0; 12];
        for (i, elem) in data.iter_mut().enumerate() {
            if let Some(d) =
                read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + (i * 4))
            {
                *elem = d;
            } else {
                return None;
            }
        }

        Some(unsafe { transmute::<[u32; 12], Self>(data) })
    }

    pub fn get_bars(&self) -> Vec<(usize, BaseAddress)> {
        let mut bars = Vec::new();
        bars.push((0, self.bar0));
        bars.push((1, self.bar1));
        bars.push((2, self.bar2));
        bars.push((3, self.bar3));
        bars.push((4, self.bar4));
        bars.push((5, self.bar5));

        let mut base_addrs = Vec::new();

        let mut i = 0;
        while i < bars.len() {
            let (_, bar) = &bars[i];
            match bar.get_base_addr() {
                Some(BaseAddress::MemoryAddress64BitSpace(addr, is_pref)) => {
                    let (_, next_bar) = &bars[i + 1];
                    let addr = (next_bar.read() as u64) << 32 | addr.get();
                    let phys_addr = PhysicalAddress::new(addr);
                    let base_addr = BaseAddress::MemoryAddress64BitSpace(phys_addr, is_pref);
                    base_addrs.push((i, base_addr));
                    bars[i] = bars.remove(i + 1);
                }
                None => (),
                Some(base_addr) => base_addrs.push((i, base_addr)),
            }

            i += 1;
        }

        base_addrs
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpacePciToPciBridgeField {
    pub bar0: BaseAddressRegister,
    pub bar1: BaseAddressRegister,
    primary_bus_num: u8,
    secondary_bus_num: u8,
    subordinate_bus_num: u8,
    secondary_latency_timer: u8,
    io_base_low: u8,
    io_limit_low: u8,
    seconday_status: u16,
    mem_base: u16,
    mem_limit: u16,
    pref_mem_base_low: u16,
    pref_mem_limit_low: u16,
    pref_mem_base_high: u32,
    pref_mem_limit_high: u32,
    io_base_high: u16,
    io_limit_high: u16,
    pub caps_ptr: u8,
    reserved: [u8; 3],
    expansion_rom_base_addr: u32,
    int_line: u8,
    int_pin: u8,
    bridge_ctrl: u16,
}

impl ConfigurationSpacePciToPciBridgeField {
    pub fn read(bus: usize, device: usize, func: usize) -> Option<Self> {
        let mut data: [u32; 12] = [0; 12];
        for (i, elem) in data.iter_mut().enumerate() {
            if let Some(d) =
                read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + (i * 4))
            {
                *elem = d;
            } else {
                return None;
            }
        }

        Some(unsafe { transmute::<[u32; 12], Self>(data) })
    }

    pub fn get_bars(&self) -> Vec<(usize, BaseAddress)> {
        let mut bars = Vec::new();
        bars.push((0, self.bar0));
        bars.push((1, self.bar1));

        let mut base_addrs = Vec::new();

        let mut i = 0;
        while i < bars.len() {
            let (_, bar) = &bars[i];
            match bar.get_base_addr() {
                Some(BaseAddress::MemoryAddress64BitSpace(addr, is_pref)) => {
                    let (_, next_bar) = &bars[i + 1];
                    let addr = (next_bar.read() as u64) << 32 | addr.get();
                    let phys_addr = PhysicalAddress::new(addr);
                    let base_addr = BaseAddress::MemoryAddress64BitSpace(phys_addr, is_pref);
                    base_addrs.push((i, base_addr));
                    bars[i] = bars.remove(i + 1);
                }
                None => (),
                Some(base_addr) => base_addrs.push((i, base_addr)),
            }

            i += 1;
        }

        base_addrs
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct ConfigurationSpacePciToCardBusField {
    cardbus_socket_or_exca_base_addr: u32,
    caps_list_offset: u8,
    reserved: u8,
    secondary_status: u16,
    pci_bus_num: u8,
    cardbus_bus_num: u8,
    subordinate_bus_num: u8,
    cardbus_latency_timer: u8,
    mem_base_addr0: u32,
    mem_limit0: u32,
    mem_base_addr1: u32,
    mem_limit1: u32,
    io_base_addr0: u32,
    io_limit0: u32,
    io_base_addr1: u32,
    io_limit1: u32,
    int_line: u8,
    int_pin: u8,
    bridge_ctrl: u16,
    subsystem_device_id: u16,
    subsystem_vendor_id: u16,
    pc_card_legacy_mode_base_addr: u32,
}

impl ConfigurationSpacePciToCardBusField {
    pub fn read(bus: usize, device: usize, func: usize) -> Option<Self> {
        let mut data: [u32; 14] = [0; 14];
        for (i, elem) in data.iter_mut().enumerate() {
            if let Some(d) =
                read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + (i * 4))
            {
                *elem = d;
            } else {
                return None;
            }
        }

        Some(unsafe { transmute::<[u32; 14], Self>(data) })
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(transparent)]
pub struct MsiMessageControlField(u16);

impl MsiMessageControlField {
    pub fn set_is_enable(&mut self, value: bool) {
        self.0 = (self.0 & !0x1) | (value as u16);
    }

    pub fn set_multiple_msg_enable(&mut self, value: u8) {
        let value = value & 0x7; // 3 bits
        self.0 = (self.0 & !0x70) | ((value as u16) << 4);
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct MsiCapabilityField {
    pub cap_id: u8,
    pub next_ptr: u8,
    pub msg_ctrl: MsiMessageControlField,
    pub msg_addr_low: MsiMessageAddressField,
    pub msg_addr_high: u32,
    pub msg_data: MsiMessageDataField,
    reserved: u64,
}

impl MsiCapabilityField {
    pub fn read(bus: usize, device: usize, func: usize, caps_ptr: usize) -> Option<Self> {
        let mut data: [u32; 6] = [0; 6];
        for (i, elem) in data.iter_mut().enumerate() {
            if let Some(d) = read_conf_space(bus, device, func, caps_ptr + (i * 4)) {
                *elem = d;
            } else {
                return None;
            }
        }

        Some(unsafe { transmute::<[u32; 6], Self>(data) })
    }

    pub fn write(
        &self,
        bus: usize,
        device: usize,
        func: usize,
        caps_ptr: usize,
    ) -> Result<(), &'static str> {
        let data = unsafe { transmute::<Self, [u32; 6]>(*self) };
        for (i, elem) in data.iter().enumerate() {
            if let Err(msg) = write_conf_space(bus, device, func, caps_ptr + (i * 4), *elem) {
                return Err(msg);
            }
        }

        Ok(())
    }
}

fn read_conf_space(bus: usize, device: usize, func: usize, byte_offset: usize) -> Option<u32> {
    if bus >= PCI_DEVICE_BUS_LEN
        || device >= PCI_DEVICE_DEVICE_LEN
        || func >= PCI_DEVICE_FUNC_LEN
        || byte_offset % 4 != 0
    {
        return None;
    }

    let addr = 0x80000000
        | (bus as u32) << 16
        | (device as u32) << 11
        | (func as u32) << 8
        | byte_offset as u32;
    PCI_PORT_CONF_REG_ADDR.out32(addr);

    Some(PCI_PORT_CONF_DATA_REG_ADDR.in32())
}

fn write_conf_space(
    bus: usize,
    device: usize,
    func: usize,
    byte_offset: usize,
    data: u32,
) -> Result<(), &'static str> {
    if bus >= PCI_DEVICE_BUS_LEN
        || device >= PCI_DEVICE_DEVICE_LEN
        || func >= PCI_DEVICE_FUNC_LEN
        || byte_offset % 4 != 0
    {
        return Err("Invalid args");
    }

    let addr = 0x80000000
        | (bus as u32) << 16
        | (device as u32) << 11
        | (func as u32) << 8
        | byte_offset as u32;
    PCI_PORT_CONF_REG_ADDR.out32(addr);
    PCI_PORT_CONF_DATA_REG_ADDR.out32(data);

    Ok(())
}
