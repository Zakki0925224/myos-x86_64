use log::warn;

use self::usb::USB_DRIVER;

pub mod console;
pub mod ps2_keyboard;
pub mod usb;

pub fn init() {
    // initialize ps/2 keyboard
    ps2_keyboard::init();

    // initialize usb driver
    if let Err(err) = USB_DRIVER.lock().init() {
        warn!("usb: {:?}", err);
    }
}
