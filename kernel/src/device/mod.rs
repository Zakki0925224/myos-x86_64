use log::warn;

use self::usb::USB_DRIVER;

pub mod usb;

pub fn init()
{
    // initialize usb driver
    if let Err(err) = USB_DRIVER.lock().init()
    {
        warn!("usb: {:?}", err);
    }
}
