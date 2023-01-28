use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};
use pci_ids::{Class, Classes, Device, Vendor, Vendors};

use crate::arch::asm;

const IO_PORT_CONFIG_ADDR: u32 = 0xcf8;
const IO_PORT_CONFIG_DATA: u32 = 0xcfc;
const PCI_DEVICE_NON_EXIST: u16 = 0xffff;

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
    prog_if: B8,
    #[skip(setters)]
    pub subclass: B8,
    #[skip(setters)]
    pub class_code: B8,
    cache_line_size: B8,
    latency_timer: B8,
    header_type: B8,
    bist: B8,
}

impl ConfigurationSpaceCommonHeaderField
{
    pub fn init(dword_bytes: &[u32; 4]) -> Self
    {
        return unsafe { dword_bytes.align_to::<Self>() }.1[0];
    }

    pub fn is_exist(&self) -> bool
    {
        let device_id = self.device_id();
        let vendor_id = self.vendor_id();

        return device_id != 0
            && vendor_id != 0
            && device_id != PCI_DEVICE_NON_EXIST
            && vendor_id != PCI_DEVICE_NON_EXIST;
    }

    pub fn get_device_name(&self) -> Option<&str>
    {
        let vendor = self.get_vendor();

        if vendor.is_none() || !self.is_exist()
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

        let class = self.get_class(self.class_code());
        return if class.is_some() { Some(class.unwrap().name()) } else { None };
    }

    fn get_vendor(&self) -> Option<&Vendor>
    {
        return Vendors::iter().find(|v| v.id() == self.vendor_id());
    }

    fn get_device(&self, vendor: &Vendor) -> Option<&Device>
    {
        return vendor.devices().find(|d| d.id() == self.device_id());
    }

    fn get_class(&self, class_code: u8) -> Option<&Class>
    {
        return Classes::iter().find(|c| c.id() == class_code);
    }
}

fn read_config_space(bus: usize, device: usize, func: usize, byte_offset: usize) -> Option<u32>
{
    if bus > 255 || device > 31 || func > 7 || byte_offset >= 64 || byte_offset % 4 != 0
    {
        return None;
    }

    let addr = 0x80000000
        | (bus as u32) << 16
        | (device as u32) << 11
        | (func as u32) << 8
        | byte_offset as u32;
    asm::out32(IO_PORT_CONFIG_ADDR, addr);
    let result = asm::in32(IO_PORT_CONFIG_DATA);

    return Some(result);
}

pub fn read_config_data(
    bus: usize,
    device: usize,
    func: usize,
) -> Option<ConfigurationSpaceCommonHeaderField>
{
    let data = [
        read_config_space(bus, device, func, 0),
        read_config_space(bus, device, func, 4),
        read_config_space(bus, device, func, 8),
        read_config_space(bus, device, func, 12),
    ];

    if data.iter().filter(|&d| d.is_none()).count() != 0
    {
        return None;
    }

    let header = ConfigurationSpaceCommonHeaderField::init(&data.map(|d| d.unwrap()));
    return Some(header);
}
