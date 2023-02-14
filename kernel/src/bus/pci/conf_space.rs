use alloc::vec::Vec;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};
use pci_ids::*;

use crate::arch::{addr::VirtualAddress, asm};

const MMIO_PORT_CONF_ADDR: u32 = 0xcf8;
const MMIO_PORT_CONF_DATA: u32 = 0xcfc;
const PCI_DEVICE_NON_EXIST: u16 = 0xffff;
pub const PCI_DEVICE_BUS_LEN: usize = 256;
pub const PCI_DEVICE_DEVICE_LEN: usize = 32;
pub const PCI_DEVICE_FUNC_LEN: usize = 8;
const PCI_CONF_MAX_OFFSET: usize = 124;
const PCI_CONF_UNIQUE_FIELD_OFFSET: usize = 16;

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum ConfigurationSpaceParityErrorResponse
{
    Normal = 0x0,
    SetDetectedParityErrorStatus = 0x1,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpaceCommandRegister
{
    io_space: bool,
    mem_space: bool,
    bus_master: bool,
    monitor_special_cycles: bool,
    mem_write_and_invalidate_enable: bool,
    vga_palette_snoop: bool,
    parity_err_res: ConfigurationSpaceParityErrorResponse,
    #[skip]
    reserved1: B1,
    serr_enable: bool,
    fast_back_to_back_enable: bool,
    interrupt_disable: bool,
    #[skip]
    reserved0: B5,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 2]
pub enum ConfigurationSpaceDevselTiming
{
    Fast = 0x0,
    Medium = 0x1,
    Slow = 0x2,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 1]
pub enum ConfigurationSpaceOperatingFrequency
{
    Capable33Mhz = 0x0,
    Capable66Mhz = 0x1,
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpaceStatusRegister
{
    #[skip]
    reserved1: B3,
    interrupt_status_enable: bool,
    capabilities_list_available: bool,
    operating_frequency: ConfigurationSpaceOperatingFrequency,
    #[skip]
    reserved0: B1,
    fast_back_to_back_capable: bool,
    master_data_parity_err: bool,
    devsel_timing: ConfigurationSpaceDevselTiming,
    signaled_target_abort: bool,
    received_target_abort: bool,
    received_master_abort: bool,
    signaled_system_err: bool,
    detected_parity_err: bool,
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 8]
pub enum ConfigurationSpaceHeaderType
{
    NonBridge = 0x0,
    PciToPciBridge = 0x1,
    PciToCardBusBridge = 0x2,
    MutliFunction = 0x80,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpaceCommonHeaderField
{
    #[skip(setters)]
    pub vendor_id: B16,
    #[skip(setters)]
    pub device_id: B16,
    pub command: ConfigurationSpaceCommandRegister,
    pub status: ConfigurationSpaceStatusRegister,
    #[skip(setters)]
    pub revision_id: B8,
    pub prog_if: B8,
    #[skip(setters)]
    pub subclass: B8,
    #[skip(setters)]
    pub class_code: B8,
    cache_line_size: B8,
    latency_timer: B8,
    #[skip(setters)]
    pub header_type: ConfigurationSpaceHeaderType,
    bist: B8,
}

impl ConfigurationSpaceCommonHeaderField
{
    pub fn read(bus: usize, device: usize, func: usize) -> Option<Self>
    {
        let data = [
            read_conf_space(bus, device, func, 0),
            read_conf_space(bus, device, func, 4),
            read_conf_space(bus, device, func, 8),
            read_conf_space(bus, device, func, 12),
        ];

        if data.iter().filter(|&d| d.is_none()).count() != 0
        {
            return None;
        }

        let data = data.map(|d| d.unwrap());
        let header = unsafe { data.align_to::<Self>() }.1[0];

        return Some(header);
    }

    pub fn is_exist(&self) -> bool { return self.vendor_id() != PCI_DEVICE_NON_EXIST; }

    pub fn get_device_name(&self) -> Option<&str>
    {
        let vendor = self.get_vendor();
        if !self.is_exist() || vendor.is_none()
        {
            return None;
        }

        let device = self.get_device(&vendor.unwrap());
        return if device.is_some() { Some(device.unwrap().name()) } else { None };
    }

    pub fn get_vendor_name(&self) -> Option<&str>
    {
        if !self.is_exist()
        {
            return None;
        }

        let vendor = self.get_vendor();
        return if vendor.is_some() { Some(vendor.unwrap().name()) } else { None };
    }

    pub fn get_class_name(&self) -> Option<&str>
    {
        if !self.is_exist()
        {
            return None;
        }

        let class = self.get_class();
        return if class.is_some() { Some(class.unwrap().name()) } else { None };
    }

    pub fn get_subclass_name(&self) -> Option<&str>
    {
        let subclass = self.get_subclass();
        if !self.is_exist() || subclass.is_none()
        {
            return None;
        }

        return Some(subclass.unwrap().name());
    }

    fn get_vendor(&self) -> Option<&Vendor>
    {
        return Vendors::iter().find(|v| v.id() == self.vendor_id());
    }

    fn get_device(&self, vendor: &Vendor) -> Option<&Device>
    {
        return vendor.devices().find(|d| d.id() == self.device_id());
    }

    fn get_class(&self) -> Option<&Class>
    {
        return Classes::iter().find(|c| c.id() == self.class_code());
    }

    fn get_subclass(&self) -> Option<&Subclass>
    {
        if let Some(class) = self.get_class()
        {
            return class.subclasses().find(|c| c.id() == self.subclass());
        }
        else
        {
            return None;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseAddress
{
    MemoryAddress32BitSpace(VirtualAddress, bool), // (addr, is prefetchable)
    MemoryAddress64BitSpace(VirtualAddress, bool),
    MmioAddressSpace(u32),
}
#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
pub struct BaseAddressRegister(u32);

impl BaseAddressRegister
{
    pub fn read(&self) -> u32 { return unsafe { self.bytes.align_to::<u32>() }.1[0]; }

    pub fn get_base_addr(&self) -> Option<BaseAddress>
    {
        let bar = self.read();

        if bar == 0
        {
            return None;
        }

        if bar & 0x1 != 0
        {
            let addr = bar & !0x3;
            return Some(BaseAddress::MmioAddressSpace(addr));
        }

        let bar_type = (bar >> 1) & 0x3;
        let prefetchable = bar & 0x8 != 0;
        let virt_addr = VirtualAddress::new((bar & !0xf) as u64);
        match bar_type
        {
            0x0 => return Some(BaseAddress::MemoryAddress32BitSpace(virt_addr, prefetchable)),
            0x2 => return Some(BaseAddress::MemoryAddress64BitSpace(virt_addr, prefetchable)),
            _ => return None,
        }
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpaceNonBridgeField
{
    bar0: BaseAddressRegister,
    bar1: BaseAddressRegister,
    bar2: BaseAddressRegister,
    bar3: BaseAddressRegister,
    bar4: BaseAddressRegister,
    bar5: BaseAddressRegister,
    cardbus_cis_ptr: B32,
    subsystem_vendor_id: B16,
    subsystem_id: B16,
    expansion_rom_base_addr: B32,
    caps_pointer: B8,
    #[skip]
    reserved0: B24,
    #[skip]
    reserved1: B32,
    int_line: B8,
    int_pin: B8,
    min_grant: B8,
    max_latency: B8,
}

impl ConfigurationSpaceNonBridgeField
{
    pub fn read(bus: usize, device: usize, func: usize) -> Option<Self>
    {
        let data = [
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 4),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 8),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 12),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 16),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 20),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 24),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 28),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 32),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 36),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 40),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 44),
        ];

        if data.iter().filter(|&d| d.is_none()).count() != 0
        {
            return None;
        }

        let data = data.map(|d| d.unwrap());
        let header = unsafe { data.align_to::<Self>() }.1[0];

        return Some(header);
    }

    pub fn get_bars(&self) -> Vec<(usize, BaseAddress)>
    {
        let mut bars = Vec::new();
        bars.push((0, self.bar0()));
        bars.push((1, self.bar1()));
        bars.push((2, self.bar2()));
        bars.push((3, self.bar3()));
        bars.push((4, self.bar4()));
        bars.push((5, self.bar5()));

        let mut base_addrs = Vec::new();

        let mut i = 0;
        while i < bars.len()
        {
            let (_, bar) = &bars[i];
            match bar.get_base_addr()
            {
                Some(BaseAddress::MemoryAddress64BitSpace(addr, is_pref)) =>
                {
                    let (_, next_bar) = &bars[i + 1];
                    let addr = (next_bar.read() as u64) << 32 | addr.get();
                    let virt_addr = VirtualAddress::new(addr);
                    let base_addr = BaseAddress::MemoryAddress64BitSpace(virt_addr, is_pref);
                    base_addrs.push((i, base_addr));
                    bars[i] = bars.remove(i + 1);
                }
                None => (),
                Some(base_addr) => base_addrs.push((i, base_addr)),
            }

            i += 1;
        }

        return base_addrs;
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpacePciToPciBridgeField
{
    bar0: BaseAddressRegister,
    bar1: BaseAddressRegister,
    primary_bus_num: B8,
    secondary_bus_num: B8,
    subordinate_bus_num: B8,
    secondary_latency_timer: B8,
    io_base_low: B8,
    io_limit_low: B8,
    seconday_status: B16,
    mem_base: B16,
    mem_limit: B16,
    pref_mem_base_low: B16,
    pref_mem_limit_low: B16,
    pref_mem_base_high: B32,
    pref_mem_limit_high: B32,
    io_base_high: B16,
    io_limit_high: B16,
    caps_pointer: B8,
    #[skip]
    reserved: B24,
    expansion_rom_base_addr: B32,
    int_line: B8,
    int_pin: B8,
    bridge_ctrl: B16,
}

impl ConfigurationSpacePciToPciBridgeField
{
    pub fn read(bus: usize, device: usize, func: usize) -> Option<Self>
    {
        let data = [
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 4),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 8),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 12),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 16),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 20),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 24),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 28),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 32),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 36),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 40),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 44),
        ];

        if data.iter().filter(|&d| d.is_none()).count() != 0
        {
            return None;
        }

        let data = data.map(|d| d.unwrap());
        let header = unsafe { data.align_to::<Self>() }.1[0];

        return Some(header);
    }

    pub fn get_bars(&self) -> Vec<(usize, BaseAddress)>
    {
        let mut bars = Vec::new();
        bars.push((0, self.bar0()));
        bars.push((1, self.bar1()));

        let mut base_addrs = Vec::new();

        let mut i = 0;
        while i < bars.len()
        {
            let (_, bar) = &bars[i];
            match bar.get_base_addr()
            {
                Some(BaseAddress::MemoryAddress64BitSpace(addr, is_pref)) =>
                {
                    let (_, next_bar) = &bars[i + 1];
                    let addr = (next_bar.read() as u64) << 32 | addr.get();
                    let virt_addr = VirtualAddress::new(addr);
                    let base_addr = BaseAddress::MemoryAddress64BitSpace(virt_addr, is_pref);
                    base_addrs.push((i, base_addr));
                    bars[i] = bars.remove(i + 1);
                }
                None => (),
                Some(base_addr) => base_addrs.push((i, base_addr)),
            }

            i += 1;
        }

        return base_addrs;
    }
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct ConfigurationSpacePciToCardBusField
{
    cardbus_socket_or_exca_base_addr: B32,
    caps_list_offset: B8,
    #[skip]
    reserved: B8,
    secondary_status: B16,
    pci_bus_num: B8,
    cardbus_bus_num: B8,
    subordinate_bus_num: B8,
    cardbus_latency_timer: B8,
    mem_base_addr0: B32,
    mem_limit0: B32,
    mem_base_addr1: B32,
    mem_limit1: B32,
    io_base_addr0: B32,
    io_limit0: B32,
    io_base_addr1: B32,
    io_limit1: B32,
    int_line: B8,
    int_pin: B8,
    bridge_ctrl: B16,
    subsystem_device_id: B16,
    subsystem_vendor_id: B16,
    pc_card_legacy_mode_base_addr: B32,
}

impl ConfigurationSpacePciToCardBusField
{
    pub fn read(bus: usize, device: usize, func: usize) -> Option<Self>
    {
        let data = [
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 4),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 8),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 12),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 16),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 20),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 24),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 28),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 32),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 36),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 40),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 44),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 48),
            read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + 52),
        ];

        if data.iter().filter(|&d| d.is_none()).count() != 0
        {
            return None;
        }

        let data = data.map(|d| d.unwrap());
        let header = unsafe { data.align_to::<Self>() }.1[0];

        return Some(header);
    }
}

fn read_conf_space(bus: usize, device: usize, func: usize, byte_offset: usize) -> Option<u32>
{
    if bus >= PCI_DEVICE_BUS_LEN
        || device >= PCI_DEVICE_DEVICE_LEN
        || func >= PCI_DEVICE_FUNC_LEN
        || byte_offset > PCI_CONF_MAX_OFFSET
        || byte_offset % 4 != 0
    {
        return None;
    }

    let addr = 0x80000000
        | (bus as u32) << 16
        | (device as u32) << 11
        | (func as u32) << 8
        | byte_offset as u32;
    asm::out32(MMIO_PORT_CONF_ADDR, addr);
    let result = asm::in32(MMIO_PORT_CONF_DATA);

    return Some(result);
}
