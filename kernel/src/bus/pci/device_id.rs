const PCI_USB_CLASS_CODE: (u8, u8) = (0x0c, 0x03); // (class, subclass)
pub const PCI_USB_UHCI_ID: (u8, u8, u8) = (PCI_USB_CLASS_CODE.0, PCI_USB_CLASS_CODE.1, 0x00);
pub const PCI_USB_OHCI_ID: (u8, u8, u8) = (PCI_USB_CLASS_CODE.0, PCI_USB_CLASS_CODE.1, 0x10);
pub const PCI_USB_EHCI_ID: (u8, u8, u8) = (PCI_USB_CLASS_CODE.0, PCI_USB_CLASS_CODE.1, 0x20);
pub const PCI_USB_XHCI_ID: (u8, u8, u8) = (PCI_USB_CLASS_CODE.0, PCI_USB_CLASS_CODE.1, 0x30);
