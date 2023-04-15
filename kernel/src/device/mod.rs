use crate::arch::asm;

use self::xhc::XHC_DRIVER;

pub mod xhc;

pub fn init_device_drivers()
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
        xhc_driver.reset_ports();
        xhc_driver.alloc_slots();
    }

    asm::sti();
}
