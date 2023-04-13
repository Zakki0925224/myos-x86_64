use self::xhc::XHC_DRIVER;

pub mod xhc;

pub fn init_device_drivers()
{
    if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
    {
        xhc_driver.init();
        xhc_driver.start();
        xhc_driver.reset_ports();
    }
}
