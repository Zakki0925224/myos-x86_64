use log::info;

use super::{DeviceDriverFunction, DeviceDriverInfo};
use crate::{arch, error::Result, util::sleep};

static mut SPEAKER: Speaker = Speaker::new();

struct Speaker;

// https://wiki.osdev.org/PC_Speaker
impl Speaker {
    const fn new() -> Self {
        Self
    }

    fn play(&self, freq: u32) {
        let div = 1193180 / freq;

        arch::out8(0x43, 0xb6);
        arch::out8(0x42, div as u8);

        let tmp = arch::in8(0x61);
        if tmp != (tmp | 3) {
            arch::out8(0x61, tmp | 3);
        }
    }

    fn stop(&self) {
        let tmp = arch::in8(0x61) & 0xfc;
        arch::out8(0x61, tmp);
    }

    fn beep(&self) {
        self.play(1000);
        sleep::sleep_ms(100);
        self.stop();
    }
}

impl DeviceDriverFunction for Speaker {
    type AttachInput = ();
    type PollNormalOutput = ();
    type PollInterruptOutput = ();

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(DeviceDriverInfo::new("speaker"))
    }

    fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    fn attach(&mut self, _arg: Self::AttachInput) -> Result<()> {
        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        unimplemented!()
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        unimplemented!()
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    unsafe { SPEAKER.get_device_driver_info() }
}

pub fn probe_and_attach() -> Result<()> {
    unsafe {
        SPEAKER.probe()?;
        SPEAKER.attach(())?;
        info!("{}: Attached!", get_device_driver_info()?.name);
    }

    Ok(())
}

pub fn beep() {
    unsafe { SPEAKER.beep() };
}
