use super::{DeviceDriverFunction, DeviceDriverInfo};
use crate::{arch, error::Result, util::sleep};
use core::num::NonZeroU8;
use log::info;

static mut SPEAKER: Speaker = Speaker::new();

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
        let octave = NonZeroU8::new(1).unwrap();
        self.play(Pitch::C.to_freq(octave));
        sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::D.to_freq(octave));
        sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::E.to_freq(octave));
        sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::F.to_freq(octave));
        sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::G.to_freq(octave));
        sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::A.to_freq(octave));
        sleep::sleep_ms(100);
        self.stop();
        self.play(Pitch::B.to_freq(octave));
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
