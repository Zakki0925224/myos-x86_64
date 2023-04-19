use crate::arch::asm;

use self::xhc::XHC_DRIVER;

pub mod xhc;

pub fn init()
{
    asm::cli();

    if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
    {
        xhc_driver.init();
        xhc_driver.start();
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
