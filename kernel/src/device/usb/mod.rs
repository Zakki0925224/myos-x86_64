use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::{info, warn};
use spin::Mutex;

use crate::arch::asm;

use self::{
    descriptor::{Descriptor, DescriptorType},
    device::*,
    xhc::{context::endpoint::EndpointType, *},
};

pub mod descriptor;
pub mod device;
pub mod hid_keyboard;
pub mod setup_trb;
pub mod xhc;

lazy_static! {
    pub static ref USB_DRIVER: Mutex<UsbDriver> = Mutex::new(UsbDriver::new());
}

#[derive(Debug)]
pub enum UsbDriverError {
    UsbDeviceError { slot_id: usize, err: UsbDeviceError },
    XhcDriverError(XhcDriverError),
}

#[derive(Debug)]
pub struct UsbDriver {
    devices: Vec<UsbDevice>,
}

impl UsbDriver {
    pub fn new() -> Self {
        return Self {
            devices: Vec::new(),
        };
    }

    pub fn init(&mut self) -> Result<(), UsbDriverError> {
        let mut result = Ok(());
        self.devices = Vec::new();

        asm::disabled_int_func(|| {
            if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut() {
                if let Err(err) = xhc_driver.init() {
                    result = Err(UsbDriverError::XhcDriverError(err));
                    return;
                }

                if let Err(err) = xhc_driver.start() {
                    result = Err(UsbDriverError::XhcDriverError(err));
                    return;
                }
            } else {
                result = Err(UsbDriverError::XhcDriverError(
                    XhcDriverError::NotInitialized,
                ));
            }
        });

        if result.is_err() {
            return result;
        }

        let mut port_ids = Vec::new();

        asm::disabled_int_func(|| {
            if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut() {
                match xhc_driver.scan_ports() {
                    Ok(ids) => port_ids = ids,
                    Err(err) => result = Err(UsbDriverError::XhcDriverError(err)),
                }
            }
        });

        if result.is_err() {
            return result;
        }

        for port_id in port_ids {
            asm::disabled_int_func(|| {
                if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut() {
                    if let Err(err) = xhc_driver.reset_port(port_id) {
                        result = Err(UsbDriverError::XhcDriverError(err));
                    }
                }
            });

            if result.is_err() {
                return result;
            }

            asm::disabled_int_func(|| {
                if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut() {
                    match xhc_driver.alloc_address_to_device(port_id) {
                        Ok(device) => self.devices.push(device),
                        Err(err) => result = Err(UsbDriverError::XhcDriverError(err)),
                    }
                }
            });

            if result.is_err() {
                return result;
            }
        }

        for device in self.devices.iter_mut() {
            let slot_id = device.slot_id();

            asm::disabled_int_func(|| {
                if let Err(err) = device.init() {
                    result = Err(UsbDriverError::UsbDeviceError { slot_id, err });
                }
            });

            if result.is_err() {
                return result;
            }

            device.read_dev_desc();

            asm::disabled_int_func(|| {
                if let Err(err) = device.request_to_get_desc(DescriptorType::Configration, 0) {
                    result = Err(UsbDriverError::UsbDeviceError { slot_id, err });
                }
            });

            if result.is_err() {
                return result;
            }

            device.read_conf_descs();

            let boot_interface = match device
                .get_interface_descs()
                .iter()
                .find(|d| d.class() == 3 && d.sub_class() == 1 && d.protocol() == 1)
            {
                Some(d) => **d,
                None => {
                    warn!(
                        "usb: Unsupported device, skip configuring... (slot id: {})",
                        slot_id
                    );
                    continue;
                }
            };

            let conf_desc = match device.get_conf_descs()[0].clone() {
                Descriptor::Configuration(desc) => desc,
                _ => unreachable!(),
            };

            asm::disabled_int_func(|| {
                if let Err(err) = device.configure_endpoint(EndpointType::InterruptIn) {
                    result = Err(UsbDriverError::UsbDeviceError { slot_id, err });
                }
            });

            if result.is_err() {
                return result;
            }

            asm::disabled_int_func(|| {
                if let Err(err) = device.request_to_set_conf(conf_desc.conf_value()) {
                    result = Err(UsbDriverError::UsbDeviceError { slot_id, err });
                }
            });

            if result.is_err() {
                return result;
            }

            asm::disabled_int_func(|| {
                if let Err(err) = device.request_to_set_interface(boot_interface) {
                    result = Err(UsbDriverError::UsbDeviceError { slot_id, err });
                }
            });

            if result.is_err() {
                return result;
            }

            asm::disabled_int_func(|| {
                if let Err(err) = device.request_to_set_protocol(boot_interface, 0) {
                    result = Err(UsbDriverError::UsbDeviceError { slot_id, err });
                }
            });

            if result.is_err() {
                return result;
            }

            asm::disabled_int_func(|| {
                if let Err(err) = device.configure_endpoint_transfer_ring() {
                    result = Err(UsbDriverError::UsbDeviceError { slot_id, err });
                }
            });

            if result.is_err() {
                return result;
            }

            device.is_configured = true;
            info!("usb: Configured device (slot id: {})", slot_id);
        }

        return result;
    }

    pub fn is_init() -> bool {
        return match XHC_DRIVER.lock().as_ref() {
            Some(xhc_driver) => xhc_driver.is_init(),
            None => false,
        };
    }

    pub fn find_device_by_slot_id(&mut self, slot_id: usize) -> Option<&mut UsbDevice> {
        return self.devices.iter_mut().find(|d| d.slot_id() == slot_id);
    }
}
