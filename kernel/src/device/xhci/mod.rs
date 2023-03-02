use core::mem::size_of;

use log::info;

use crate::{bus::pci::{conf_space::BaseAddress, device_id::*, PCI_DEVICE_MAN}, device::xhci::host_controller::*, println};

pub mod host_controller;

#[derive(Debug)]
pub struct Xhci
{
    controller_pci_bus: usize,
    controller_pci_device: usize,
    controller_pci_func: usize,
}

impl Xhci
{
    pub fn new() -> Option<Self>
    {
        let (class_code, subclass_code, prog_if) = PCI_USB_XHCI_ID;

        if let Some(device) =
            PCI_DEVICE_MAN.lock().find_by_class(class_code, subclass_code, prog_if)
        {
            let usb = Xhci {
                controller_pci_bus: device.bus,
                controller_pci_device: device.device,
                controller_pci_func: device.func,
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

    pub fn init(&self)
    {
        if let Some(controller) = PCI_DEVICE_MAN.lock().find_by_bdf(
            self.controller_pci_bus,
            self.controller_pci_device,
            self.controller_pci_func,
        )
        {
            let bars = controller.read_conf_space_non_bridge_field().unwrap().get_bars();
            println!("{:?}", bars);
            let addr = match bars[0].1
            {
                BaseAddress::MemoryAddress64BitSpace(addr, _) => addr,
                BaseAddress::MemoryAddress32BitSpace(addr, _) => addr,
                _ => panic!(),
            };

            println!("addr: 0x{:x}", addr.get_virt_addr().get());
            let cap_reg: CapabilityRegisters = addr.get_virt_addr().read_volatile();
            println!("{:?}", cap_reg);

            let ope_reg: OperationalRegisters =
                addr.offset(cap_reg.cap_reg_length() as usize).get_virt_addr().read_volatile();
            println!("{:?}", ope_reg);

            let runtime_reg: RuntimeRegitsers = addr
                .offset(cap_reg.runtime_reg_space_offset() as usize)
                .get_virt_addr()
                .read_volatile();
            println!("{:?}", runtime_reg);
            let irs: InterruptRegisterSets = addr
                .offset(cap_reg.runtime_reg_space_offset() as usize + size_of::<RuntimeRegitsers>())
                .get_virt_addr()
                .read_volatile();
            println!("{:?}", irs);
        }
    }
}
