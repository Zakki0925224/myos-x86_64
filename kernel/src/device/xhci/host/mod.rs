use core::mem::size_of;

use log::info;

use crate::{arch::addr::VirtualAddress, bus::pci::{conf_space::BaseAddress, device_id::*, PCI_DEVICE_MAN}, device::xhci::host::regs::*, println};

pub mod regs;

#[derive(Debug)]
pub struct XhciHostDriver
{
    controller_pci_bus: usize,
    controller_pci_device: usize,
    controller_pci_func: usize,
    cap_reg_virt_addr: VirtualAddress,
    ope_reg_virt_addr: VirtualAddress,
    runtime_reg_virt_addr: VirtualAddress,
    int_reg_sets_virt_addr: VirtualAddress,
}

impl XhciHostDriver
{
    pub fn new() -> Option<Self>
    {
        let (class_code, subclass_code, prog_if) = PCI_USB_XHCI_ID;

        if let Some(device) =
            PCI_DEVICE_MAN.lock().find_by_class(class_code, subclass_code, prog_if)
        {
            let usb = XhciHostDriver {
                controller_pci_bus: device.bus,
                controller_pci_device: device.device,
                controller_pci_func: device.func,
                cap_reg_virt_addr: VirtualAddress::new(0),
                ope_reg_virt_addr: VirtualAddress::new(0),
                runtime_reg_virt_addr: VirtualAddress::new(0),
                int_reg_sets_virt_addr: VirtualAddress::new(0),
            };

            info!(
                "xHCI host driver: {}.{}:{} - {}",
                device.bus,
                device.device,
                device.func,
                device.conf_space_header.get_device_name().unwrap()
            );

            return Some(usb);
        }

        info!("xHCI host controller was not found");
        return None;
    }

    pub fn init(&mut self)
    {
        if let Some(controller) = PCI_DEVICE_MAN.lock().find_by_bdf(
            self.controller_pci_bus,
            self.controller_pci_device,
            self.controller_pci_func,
        )
        {
            let bars = controller.read_conf_space_non_bridge_field().unwrap().get_bars();
            println!("{:?}", bars);
            self.cap_reg_virt_addr = match bars[0].1
            {
                BaseAddress::MemoryAddress64BitSpace(addr, _) => addr,
                BaseAddress::MemoryAddress32BitSpace(addr, _) => addr,
                _ => panic!(),
            }
            .get_virt_addr();

            let cap_reg = self.read_cap_reg().unwrap();

            self.ope_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.cap_reg_length() as usize);

            self.runtime_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.runtime_reg_space_offset() as usize);

            self.int_reg_sets_virt_addr = self.cap_reg_virt_addr.offset(
                cap_reg.runtime_reg_space_offset() as usize + size_of::<RuntimeRegitsers>(),
            );
        }
    }

    pub fn read_cap_reg(&self) -> Option<CapabilityRegisters>
    {
        if self.cap_reg_virt_addr.get() == 0
        {
            return None;
        }

        return Some(self.cap_reg_virt_addr.read_volatile());
    }

    pub fn read_ope_reg(&self) -> Option<OperationalRegisters>
    {
        if self.ope_reg_virt_addr.get() == 0
        {
            return None;
        }

        return Some(self.ope_reg_virt_addr.read_volatile());
    }

    pub fn read_runtime_reg(&self) -> Option<RuntimeRegitsers>
    {
        if self.runtime_reg_virt_addr.get() == 0
        {
            return None;
        }

        return Some(self.runtime_reg_virt_addr.read_volatile());
    }

    pub fn read_int_reg_sets(&self) -> Option<InterruptRegisterSets>
    {
        if self.int_reg_sets_virt_addr.get() == 0
        {
            return None;
        }

        return Some(self.int_reg_sets_virt_addr.read_volatile());
    }
}
