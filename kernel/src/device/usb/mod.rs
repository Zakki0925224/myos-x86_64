use alloc::vec::Vec;
use lazy_static::lazy_static;
use spin::Mutex;

use crate::{arch::asm, println};

use self::xhc::*;

pub mod xhc;

lazy_static! {
    pub static ref USB_DRIVER: Mutex<UsbDriver> = Mutex::new(UsbDriver::new());
}

#[derive(Debug)]
pub enum UsbDriverError
{
    NotInitialized,
    XhcDriverError(XhcDriverError),
}

#[derive(Debug)]
pub struct UsbDriver;

impl UsbDriver
{
    pub fn new() -> Self { return Self {}; }

    pub fn init(&self) -> Result<(), UsbDriverError>
    {
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
                if let Err(err) = xhc_driver.alloc_address_to_device(port_id)
                {
                    return Err(UsbDriverError::XhcDriverError(err));
                }
            }

            asm::sti();
        }

        println!("{:?}", XHC_DRIVER.lock().as_ref().unwrap().ports);

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
