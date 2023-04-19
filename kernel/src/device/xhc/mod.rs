use lazy_static::lazy_static;
use spin::Mutex;

use crate::device::xhc::host::XhcDriver;

pub mod context;
pub mod host;
pub mod port;
pub mod register;
pub mod ring_buffer;
pub mod slot;
pub mod trb;

lazy_static! {
    pub static ref XHC_DRIVER: Mutex<Option<XhcDriver>> = Mutex::new(XhcDriver::new());
}
