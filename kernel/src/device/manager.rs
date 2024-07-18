use super::{DeviceFunction, DeviceId, DevicePollMode};
use crate::{
    error::{Error, Result},
    util::mutex::Mutex,
};
use alloc::{boxed::Box, vec::Vec};
use log::info;

static mut DEVICE_MANAGER: Mutex<DeviceManager> = Mutex::new(DeviceManager::new());

#[derive(Debug, Clone, PartialEq)]
pub enum DeviceManagerError {
    FailedToAttachDevice { id: DeviceId, err: Box<Error> },
}

struct DeviceManager {
    devices: Vec<Mutex<Box<dyn DeviceFunction>>>,
}

impl DeviceManager {
    const fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    fn register_device(&mut self, device: Box<dyn DeviceFunction>) {
        self.devices.push(Mutex::new(device));
    }

    fn attach_all_devices(&mut self) -> Result<()> {
        for d in &mut self.devices {
            let mut d = d.try_lock()?;
            if let Err(err) = d.attach() {
                return Err(DeviceManagerError::FailedToAttachDevice {
                    id: d.get_info().id,
                    err: Box::new(err),
                }
                .into());
            }
        }

        Ok(())
    }

    fn update_all_polls(&mut self) {
        for d in &mut self.devices {
            let mut d = match d.try_lock() {
                Ok(d) => d,
                Err(_) => continue,
            };
            if d.poll_mode() != DevicePollMode::None && d.is_attached() {
                let _ = d.update_poll();
            }
        }
    }
}

pub fn register_device(device: Box<dyn DeviceFunction>) -> Result<()> {
    let device_info = device.get_info();
    unsafe { DEVICE_MANAGER.try_lock() }?.register_device(device);
    info!("dev: Registered the device: {:?}", device_info);
    Ok(())
}

pub fn attach_all_devices() -> Result<()> {
    unsafe { DEVICE_MANAGER.try_lock() }?.attach_all_devices()?;
    info!("dev: Attached all devices");
    Ok(())
}

pub fn update_all_polls() -> Result<()> {
    unsafe { DEVICE_MANAGER.try_lock() }?.update_all_polls();
    //info!("dev: Updated all polls");
    Ok(())
}
