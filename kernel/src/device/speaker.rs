use super::{DeviceDriverFunction, DeviceDriverInfo};
use crate::{arch, error::Result, fs::vfs};
use alloc::vec::Vec;
use core::num::NonZeroU8;
use log::info;

static mut SPEAKER_DRIVER: SpeakerDriver = SpeakerDriver::new();

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum Pitch {
    C = 65,
    Cs = 69,
    D = 73,
    Ds = 78,
    E = 82,
    F = 87,
    Fs = 92,
    G = 98,
    Gs = 104,
    A = 110,
    As = 117,
    B = 123,
}

impl Pitch {
    pub fn to_freq(&self, octave: NonZeroU8) -> u32 {
        let base_freq = *self as u32;
        let octave_mul = 1u32 << (octave.get() - 1);
        base_freq * octave_mul
    }
}

struct SpeakerDriver {
    device_driver_info: DeviceDriverInfo,
}

// https://wiki.osdev.org/PC_Speaker
impl SpeakerDriver {
    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("speaker"),
        }
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
        let octave = NonZeroU8::new(1).unwrap();
        self.play(Pitch::C.to_freq(octave));
        // sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::D.to_freq(octave));
        // sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::E.to_freq(octave));
        // sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::F.to_freq(octave));
        // sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::G.to_freq(octave));
        // sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::A.to_freq(octave));
        // sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::B.to_freq(octave));
        // sleep::sleep_ms(100);
        self.stop();
    }
}

impl DeviceDriverFunction for SpeakerDriver {
    type AttachInput = ();
    type PollNormalOutput = ();
    type PollInterruptOutput = ();

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(self.device_driver_info.clone())
    }

    fn probe(&mut self) -> Result<()> {
        Ok(())
    }

    fn attach(&mut self, _arg: Self::AttachInput) -> Result<()> {
        let dev_desc = vfs::DeviceFileDescriptor {
            get_device_driver_info,
            open,
            close,
            read,
            write,
        };
        vfs::add_dev_file(dev_desc, self.device_driver_info.name)?;
        self.device_driver_info.attached = true;
        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        unimplemented!()
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        unimplemented!()
    }

    fn open(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn close(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn write(&mut self, _data: &[u8]) -> Result<()> {
        unimplemented!()
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    unsafe { SPEAKER_DRIVER.get_device_driver_info() }
}

pub fn probe_and_attach() -> Result<()> {
    unsafe {
        SPEAKER_DRIVER.probe()?;
        SPEAKER_DRIVER.attach(())?;
        info!("{}: Attached!", get_device_driver_info()?.name);
    }

    Ok(())
}

pub fn open() -> Result<()> {
    unsafe { SPEAKER_DRIVER.open() }
}

pub fn close() -> Result<()> {
    unsafe { SPEAKER_DRIVER.close() }
}

pub fn read() -> Result<Vec<u8>> {
    unsafe { SPEAKER_DRIVER.read() }
}

pub fn write(data: &[u8]) -> Result<()> {
    unsafe { SPEAKER_DRIVER.write(data) }
}

pub fn beep() {
    unsafe { SPEAKER_DRIVER.beep() };
}
