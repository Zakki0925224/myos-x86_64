use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::warn;
use spin::Mutex;

use crate::arch::asm;

use self::{descriptor::{Descriptor, DescriptorType}, device::*, xhc::*};

pub mod descriptor;
pub mod device;
pub mod setup_trb;
pub mod xhc;

lazy_static! {
    pub static ref USB_DRIVER: Mutex<UsbDriver> = Mutex::new(UsbDriver::new());
}

#[derive(Debug)]
pub enum UsbDriverError
{
    UsbDeviceError
    {
        slot_id: usize,
        error: UsbDeviceError,
    },
    XhcDriverError(XhcDriverError),
}

#[derive(Debug)]
pub struct UsbDriver
{
    devices: Vec<UsbDevice>,
}

impl UsbDriver
{
    pub fn new() -> Self { return Self { devices: Vec::new() }; }

    pub fn init(&mut self) -> Result<(), UsbDriverError>
    {
        self.devices = Vec::new();

        asm::cli();

        if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
        {
            match xhc_driver.init()
            {
                Ok(_) => (),
                Err(err) => return Err(UsbDriverError::XhcDriverError(err)),
            }

            match xhc_driver.start()
            {
                Ok(_) => (),
                Err(err) => return Err(UsbDriverError::XhcDriverError(err)),
            }
        }
        else
        {
            return Err(UsbDriverError::XhcDriverError(XhcDriverError::NotInitialized));
        }

        asm::sti();

        let mut port_ids = Vec::new();

        asm::cli();

        // always return some()
        if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
        {
            match xhc_driver.scan_ports()
            {
                Ok(ids) => port_ids = ids,
                Err(err) => return Err(UsbDriverError::XhcDriverError(err)),
            }
        }

        asm::sti();

        for port_id in port_ids
        {
            asm::cli();

            if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
            {
                match xhc_driver.reset_port(port_id)
                {
                    Ok(_) => (),
                    Err(err) => return Err(UsbDriverError::XhcDriverError(err)),
                }
            }

            asm::sti();

            asm::cli();

            if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
            {
                match xhc_driver.alloc_address_to_device(port_id)
                {
                    Ok(device) => self.devices.push(device),
                    Err(err) => return Err(UsbDriverError::XhcDriverError(err)),
                }
            }

            asm::sti();
        }

        for device in self.devices.iter_mut()
        {
            let slot_id = device.slot_id();

            asm::cli();
            match device.init()
            {
                Ok(_) => (),
                Err(error) => return Err(UsbDriverError::UsbDeviceError { slot_id, error }),
            }
            asm::sti();

            device.read_dev_desc();

            let mut set_interface = false;

            for i in 0..device.get_num_confs()
            {
                asm::cli();
                match device.request_to_get_desc(DescriptorType::Configration, i)
                {
                    Ok(_) => (),
                    Err(error) => return Err(UsbDriverError::UsbDeviceError { slot_id, error }),
                }
                asm::sti();

                device.read_conf_descs();

                for interface_desc in device.get_interface_descs()
                {
                    if interface_desc.class() == 3
                        && interface_desc.sub_class() == 1
                        && interface_desc.protocol() == 1
                    {
                        let conf_desc = match &device.get_conf_descs()[0]
                        {
                            Descriptor::Configuration(desc) => desc,
                            _ => unreachable!(),
                        };

                        asm::cli();
                        match device.request_to_set_conf(conf_desc.conf_value())
                        {
                            Ok(_) => (),
                            Err(error) =>
                            {
                                return Err(UsbDriverError::UsbDeviceError { slot_id, error })
                            }
                        }
                        asm::sti();

                        set_interface = true;
                        break;
                    }
                }
            }

            if !set_interface
            {
                warn!("Unsupported device, skip configuring... (slot id: {})", slot_id);
                continue;
            }

            asm::cli();
            match device.configure_endpoint()
            {
                Ok(_) => (),
                Err(error) => return Err(UsbDriverError::UsbDeviceError { slot_id, error }),
            }
            asm::sti();

            // asm::cli();
            // match device.request_to_set_interface()
            // {
            //     Ok(_) => (),
            //     Err(error) => return Err(UsbDriverError::UsbDeviceError { slot_id, error }),
            // }
            // asm::sti();

            asm::cli();
            match device.request_to_use_boot_protocol()
            {
                Ok(_) => (),
                Err(error) => return Err(UsbDriverError::UsbDeviceError { slot_id, error }),
            }
            asm::sti();
        }

        return Ok(());
    }

    pub fn is_init() -> bool
    {
        return match XHC_DRIVER.lock().as_ref()
        {
            Some(xhc_driver) => xhc_driver.is_init(),
            None => false,
        };
    }

    pub fn find_device_by_slot_id(&mut self, slot_id: usize) -> Option<&mut UsbDevice>
    {
        return self.devices.iter_mut().find(|d| d.slot_id() == slot_id);
    }
}
