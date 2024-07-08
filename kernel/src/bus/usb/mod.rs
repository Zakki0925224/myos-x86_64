use self::{
    descriptor::{Descriptor, DescriptorType},
    device::*,
    xhc::context::endpoint::EndpointType,
};
use crate::{
    arch,
    error::{Error, Result},
    util::mutex::Mutex,
};
use alloc::{boxed::Box, vec::Vec};
use log::{info, warn};

pub mod descriptor;
pub mod device;
pub mod setup_trb;
pub mod xhc;

static mut USB_DRIVER: Mutex<UsbDriver> = Mutex::new(UsbDriver::new());

#[derive(Debug, Clone, PartialEq)]
pub enum UsbDriverError {
    UsbDeviceError { slot_id: usize, err: Box<Error> },
    UsbDeviceNotExitstError,
}

#[derive(Debug)]
pub struct UsbDriver {
    devices: Vec<UsbDevice>,
}

impl UsbDriver {
    pub const fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        let mut result = Ok(());
        self.devices = Vec::new();

        arch::disabled_int_func(|| {
            if let Err(err) = xhc::init() {
                result = Err(err);
                return;
            }

            if let Err(err) = xhc::start() {
                result = Err(err);
                return;
            }
        });

        if result.is_err() {
            return result;
        }

        let mut port_ids = Vec::new();

        arch::disabled_int_func(|| {
            result = match xhc::scan_ports() {
                Ok(ids) => {
                    port_ids = ids;
                    Ok(())
                }
                Err(err) => Err(err),
            }
        });

        if result.is_err() {
            return result;
        }

        for port_id in port_ids {
            arch::disabled_int_func(|| {
                result = xhc::reset_port(port_id);
            });

            if result.is_err() {
                return result;
            }

            arch::disabled_int_func(|| {
                result = match xhc::alloc_address_to_device(port_id) {
                    Ok(device) => {
                        self.devices.push(device);
                        Ok(())
                    }
                    Err(err) => Err(err),
                }
            });

            if result.is_err() {
                return result;
            }
        }

        for device in self.devices.iter_mut() {
            let slot_id = device.slot_id();

            arch::disabled_int_func(|| {
                if let Err(err) = device.init() {
                    result = Err(UsbDriverError::UsbDeviceError {
                        slot_id,
                        err: Box::new(err),
                    }
                    .into());
                }
            });

            if result.is_err() {
                return result;
            }

            device.read_dev_desc();

            arch::disabled_int_func(|| {
                if let Err(err) = device.request_to_get_desc(DescriptorType::Configration, 0) {
                    result = Err(UsbDriverError::UsbDeviceError {
                        slot_id,
                        err: Box::new(err),
                    }
                    .into());
                }
            });

            if result.is_err() {
                return result;
            }

            device.read_conf_descs();

            let boot_interface = match device
                .get_interface_descs()
                .iter()
                .find(|d| d.class == 3 && d.sub_class == 1 && d.protocol == 1)
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

            arch::disabled_int_func(|| {
                if let Err(err) = device.configure_endpoint(EndpointType::InterruptIn) {
                    result = Err(UsbDriverError::UsbDeviceError {
                        slot_id,
                        err: Box::new(err),
                    }
                    .into());
                }
            });

            if result.is_err() {
                return result;
            }

            arch::disabled_int_func(|| {
                if let Err(err) = device.request_to_set_conf(conf_desc.conf_value) {
                    result = Err(UsbDriverError::UsbDeviceError {
                        slot_id,
                        err: Box::new(err),
                    }
                    .into());
                }
            });

            if result.is_err() {
                return result;
            }

            arch::disabled_int_func(|| {
                if let Err(err) = device.request_to_set_interface(boot_interface) {
                    result = Err(UsbDriverError::UsbDeviceError {
                        slot_id,
                        err: Box::new(err),
                    }
                    .into());
                }
            });

            if result.is_err() {
                return result;
            }

            arch::disabled_int_func(|| {
                if let Err(err) = device.request_to_set_protocol(boot_interface, 0) {
                    result = Err(UsbDriverError::UsbDeviceError {
                        slot_id,
                        err: Box::new(err),
                    }
                    .into());
                }
            });

            if result.is_err() {
                return result;
            }
            arch::disabled_int_func(|| {
                if let Err(err) = device.configure_endpoint_transfer_ring() {
                    result = Err(UsbDriverError::UsbDeviceError {
                        slot_id,
                        err: Box::new(err),
                    }
                    .into());
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

    pub fn find_device_by_slot_id(&self, slot_id: usize) -> Option<UsbDevice> {
        self.devices
            .iter()
            .find(|d| d.slot_id() == slot_id)
            .cloned()
    }

    pub fn update_device(&mut self, device: UsbDevice) -> Result<()> {
        if let Some(d) = self
            .devices
            .iter_mut()
            .find(|d| d.slot_id() == device.slot_id())
        {
            *d = device;
        } else {
            return Err(UsbDriverError::UsbDeviceNotExitstError.into());
        }

        Ok(())
    }
}

pub fn init() -> Result<()> {
    unsafe { USB_DRIVER.try_lock() }?.init()
}

pub fn find_device_by_slot_id(slot_id: usize) -> Result<Option<UsbDevice>> {
    Ok(unsafe { USB_DRIVER.try_lock() }?.find_device_by_slot_id(slot_id))
}

pub fn update_device(device: UsbDevice) -> Result<()> {
    unsafe { USB_DRIVER.try_lock() }?.update_device(device)
}
