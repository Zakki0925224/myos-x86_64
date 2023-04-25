use lazy_static::lazy_static;
use log::warn;
use spin::Mutex;

use crate::arch::asm;

use self::xhc::{host::*, XHC_DRIVER};

pub mod xhc;

lazy_static! {
    pub static ref USB_DRIVER: Mutex<UsbDriver> = Mutex::new(UsbDriver::new());
}

#[derive(Debug)]
pub enum UsbDriverError
{
    XhcDriverError(XhcDriverError),
}

#[derive(Debug)]
pub struct UsbDriver;

impl UsbDriver
{
    pub fn new() -> Self { return Self {}; }
}

pub fn init()
{
    asm::cli();

    if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
    {
        match xhc_driver.init()
        {
            Err(err) => warn!("xhc: {:?}", err),
            _ => (),
        }
        match xhc_driver.start()
        {
            Err(err) => warn!("xhc: {:?}", err),
            _ => (),
        }
    }

    asm::sti();

    asm::cli();

    if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
    {
        xhc_driver.scan_ports();
        xhc_driver.reset_port(5);
        //xhc_driver.reset_port(6);
    }

    asm::sti();

    asm::cli();

    if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
    {
        xhc_driver.alloc_address_to_device(5);
    }

    asm::sti();
}
