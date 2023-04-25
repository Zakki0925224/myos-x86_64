use lazy_static::lazy_static;
use log::warn;
use spin::Mutex;

use crate::arch::asm;

use self::xhc::XHC_DRIVER;

pub mod xhc;

lazy_static! {
    pub static ref USB_DRIVER: Mutex<UsbDriver> = Mutex::new(UsbDriver::new());
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
        if let Err(err) = xhc_driver.init()
        {
            warn!("xhc: {:?}", err);
        }

        if let Err(err) = xhc_driver.start()
        {
            warn!("xhc: {:?}", err);
        }
    }

    asm::sti();

    asm::cli();

    if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
    {
        if let Err(err) = xhc_driver.scan_ports()
        {
            warn!("xhc: {:?}", err);
        }

        if let Err(err) = xhc_driver.reset_port(5)
        {
            warn!("xhc: {:?}", err);
        }

        //xhc_driver.reset_port(6);
    }

    asm::sti();

    asm::cli();

    if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
    {
        if let Err(err) = xhc_driver.alloc_address_to_device(5)
        {
            warn!("xhc: {:?}", err);
        }
    }

    asm::sti();
}
