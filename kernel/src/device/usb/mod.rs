use alloc::vec::Vec;
use lazy_static::lazy_static;
use log::warn;
use spin::Mutex;

use crate::{arch::asm, println};

use self::{descriptor::DescriptorType, device::*, xhc::*};

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
    NotInitialized,
    UsbDeviceError(usize, UsbDeviceError), // slot id
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
            if let Err(err) = xhc_driver.init()
            {
                return Err(UsbDriverError::XhcDriverError(err));
            }

            if let Err(err) = xhc_driver.start()
            {
                return Err(UsbDriverError::XhcDriverError(err));
            }
        }
        else
        {
            return Err(UsbDriverError::XhcDriverError(XhcDriverError::NotInitialized));
        }

        asm::sti();

        let mut port_ids = Vec::new();

        asm::cli();

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
                if let Err(err) = xhc_driver.reset_port(port_id)
                {
                    return Err(UsbDriverError::XhcDriverError(err));
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
            asm::cli();
            if let Err(err) = device.init()
            {
                warn!("usb: {:?}", err);
            }
            asm::sti();

            asm::cli();
            if let Err(err) = device.request_get_desc(DescriptorType::Device, 0)
            {
                warn!("usb: {:?}", err);
            }
            asm::sti();

            let dev_desc = device.get_dev_desc();
            let num_configs = dev_desc.num_configs() as usize;

            for i in 0..num_configs
            {
                asm::cli();
                if let Err(err) = device.request_get_desc(DescriptorType::Configration, i)
                {
                    warn!("usb: {:?}", err);
                }
                asm::sti();

                let conf_descs = device.get_conf_descs();
            }
        }

        return Ok(());
    }

    pub fn is_init() -> bool
    {
        if let Some(xhc_driver) = XHC_DRIVER.lock().as_ref()
        {
            return xhc_driver.is_init();
        }

        return false;
    }
}
