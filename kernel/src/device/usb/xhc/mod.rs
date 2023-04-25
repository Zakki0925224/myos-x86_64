use lazy_static::lazy_static;
use log::warn;
use spin::Mutex;

use self::host::XhcDriver;

pub mod context;
pub mod device;
pub mod host;
pub mod port;
pub mod register;
pub mod ring_buffer;
pub mod trb;

lazy_static! {
    pub static ref XHC_DRIVER: Mutex<Option<XhcDriver>> = Mutex::new(match XhcDriver::new()
    {
        Ok(xhc_driver) => Some(xhc_driver),
        Err(err) =>
        {
            warn!("xhc: {:?}", err);
            None
        }
    });
}
