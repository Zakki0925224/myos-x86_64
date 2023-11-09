pub mod console;
pub mod ps2_keyboard;
pub mod usb;

pub fn init() {
    // initialize ps/2 keyboard
    ps2_keyboard::init();
}
