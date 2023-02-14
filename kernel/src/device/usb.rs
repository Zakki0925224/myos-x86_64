use crate::bus::pci::{device_id::*, PciDeviceManager, PCI_DEVICE_MAN};

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum UsbMode
{
    Ohci,
    Uhci,
    Ehci,
    Xhci,
}

#[derive(Debug)]
pub struct Usb
{
    mode: UsbMode,
    controller_pci_bus: usize,
    controller_pci_device: usize,
    controller_pci_func: usize,
}

impl Usb
{
    pub fn new(mode: UsbMode) -> Option<Self>
    {
        let (class_code, subclass_code, prog_if) = match mode
        {
            UsbMode::Uhci => PCI_USB_UHCI_ID,
            UsbMode::Ohci => PCI_USB_OHCI_ID,
            UsbMode::Ehci => PCI_USB_EHCI_ID,
            UsbMode::Xhci => PCI_USB_XHCI_ID,
        };

        if let Some(device) = PCI_DEVICE_MAN.lock().find(class_code, subclass_code, prog_if)
        {
            let usb = Usb {
                mode,
                controller_pci_bus: device.bus,
                controller_pci_device: device.device,
                controller_pci_func: device.func,
            };

            return Some(usb);
        }

        return None;
    }
}
