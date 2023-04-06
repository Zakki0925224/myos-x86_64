use core::mem::transmute;

use alloc::vec::Vec;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};
use pci_ids::*;

use crate::arch::{addr::PhysicalAddress, asm, register::msi::*};

const MMIO_PORT_CONF_ADDR: u32 = 0xcf8;
const MMIO_PORT_CONF_DATA: u32 = 0xcfc;
const PCI_DEVICE_NON_EXIST: u16 = 0xffff;
pub const PCI_DEVICE_BUS_LEN: usize = 256;
pub const PCI_DEVICE_DEVICE_LEN: usize = 32;
pub const PCI_DEVICE_FUNC_LEN: usize = 8;
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
    pub io_space: bool,
    pub mem_space: bool,
    pub bus_master: bool,
    pub monitor_special_cycles: bool,
    pub mem_write_and_invalidate_enable: bool,
    pub vga_palette_snoop: bool,
    pub parity_err_res: ConfigurationSpaceParityErrorResponse,
    #[skip]
    reserved1: B1,
    pub serr_enable: bool,
    pub fast_back_to_back_enable: bool,
    pub interrupt_disable: bool,
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
    pub interrupt_status_enable: bool,
    pub caps_list_available: bool,
    pub operating_frequency: ConfigurationSpaceOperatingFrequency,
    #[skip]
    reserved0: B1,
    pub fast_back_to_back_capable: bool,
    pub master_data_parity_err: bool,
    pub devsel_timing: ConfigurationSpaceDevselTiming,
    pub signaled_target_abort: bool,
    pub received_target_abort: bool,
    pub received_master_abort: bool,
    pub signaled_system_err: bool,
    pub detected_parity_err: bool,
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
        let mut data: [u32; 4] = [0; 4];
        for (i, elem) in data.iter_mut().enumerate()
        {
            if let Some(d) = read_conf_space(bus, device, func, i * 4)
            {
                *elem = d;
            }
            else
            {
                return None;
            }
        }

        return Some(unsafe { transmute::<[u32; 4], Self>(data) });
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
    MemoryAddress32BitSpace(PhysicalAddress, bool), // (addr, is prefetchable)
    MemoryAddress64BitSpace(PhysicalAddress, bool),
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
        let phys_addr = PhysicalAddress::new((bar & !0xf) as u64);
        match bar_type
        {
            0x0 => return Some(BaseAddress::MemoryAddress32BitSpace(phys_addr, prefetchable)),
            0x2 => return Some(BaseAddress::MemoryAddress64BitSpace(phys_addr, prefetchable)),
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
    pub caps_ptr: B8,
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
        let mut data: [u32; 12] = [0; 12];
        for (i, elem) in data.iter_mut().enumerate()
        {
            if let Some(d) =
                read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + (i * 4))
            {
                *elem = d;
            }
            else
            {
                return None;
            }
        }

        return Some(unsafe { transmute::<[u32; 12], Self>(data) });
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
    pub caps_ptr: B8,
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
        let mut data: [u32; 12] = [0; 12];
        for (i, elem) in data.iter_mut().enumerate()
        {
            if let Some(d) =
                read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + (i * 4))
            {
                *elem = d;
            }
            else
            {
                return None;
            }
        }

        return Some(unsafe { transmute::<[u32; 12], Self>(data) });
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
        let mut data: [u32; 14] = [0; 14];
        for (i, elem) in data.iter_mut().enumerate()
        {
            if let Some(d) =
                read_conf_space(bus, device, func, PCI_CONF_UNIQUE_FIELD_OFFSET + (i * 4))
            {
                *elem = d;
            }
            else
            {
                return None;
            }
        }

        return Some(unsafe { transmute::<[u32; 14], Self>(data) });
    }
}

#[bitfield]
#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[repr(C)]
pub struct MsiMessageControlField
{
    pub is_enable: bool,
    #[skip(setters)]
    pub multiple_msg_capable: B3,
    pub multiple_msg_enable: B3,
    #[skip(setters)]
    pub is_64bit: bool,
    #[skip(setters)]
    pub per_vec_masking: bool,
    #[skip]
    reserved: B7,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct MsiCapabilityField
{
    pub cap_id: B8,
    pub next_ptr: B8,
    pub msg_ctrl: MsiMessageControlField,
    pub msg_addr_low: MsiMessageAddressField,
    pub msg_addr_high: B32,
    pub msg_data: MsiMessageDataField,
    #[skip]
    reserved: B16,
}

impl MsiCapabilityField
{
    pub fn read(bus: usize, device: usize, func: usize, caps_ptr: usize) -> Option<Self>
    {
        let mut data: [u32; 4] = [0; 4];
        for (i, elem) in data.iter_mut().enumerate()
        {
            if let Some(d) = read_conf_space(bus, device, func, caps_ptr + (i * 4))
            {
                *elem = d;
            }
            else
            {
                return None;
            }
        }

        return Some(unsafe { transmute::<[u32; 4], Self>(data) });
    }

    pub fn write(
        &self,
        bus: usize,
        device: usize,
        func: usize,
        caps_ptr: usize,
    ) -> Result<(), &'static str>
    {
        let mut data = unsafe { transmute::<Self, [u32; 4]>(*self) };
        for (i, elem) in data.iter().enumerate()
        {
            if let Err(msg) = write_conf_space(bus, device, func, caps_ptr + (i * 4), *elem)
            {
                return Err(msg);
            }
        }

        return Ok(());
    }
}

fn read_conf_space(bus: usize, device: usize, func: usize, byte_offset: usize) -> Option<u32>
{
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
    asm::out32(MMIO_PORT_CONF_ADDR, addr);
    let result = asm::in32(MMIO_PORT_CONF_DATA);

    return Some(result);
}

fn write_conf_space(
    bus: usize,
    device: usize,
    func: usize,
    byte_offset: usize,
    data: u32,
) -> Result<(), &'static str>
{
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
    asm::out32(MMIO_PORT_CONF_ADDR, addr);
    asm::out32(MMIO_PORT_CONF_DATA, data);

    return Ok(());
}
