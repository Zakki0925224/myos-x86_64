use crate::{
    acpi,
    arch::{addr::*, idt::VEC_LOCAL_APIC_TIMER_INT},
    error::Result,
};
use core::num::{NonZero, NonZeroUsize};
use log::info;

const LVT_TIMER_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00320);
const INIT_CNT_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00380);
const DIV_CONF_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee003e0);

static mut LOCAL_APIC_TIMER: Timer = Timer::new();

struct Timer {
    tick: usize,
    freq: Option<NonZero<usize>>,
}

impl Timer {
    const fn new() -> Self {
        Self {
            tick: 0,
            freq: None,
        }
    }

    unsafe fn init(&mut self) -> Result<()> {
        // calc freq
        self.stop();
        (DIV_CONF_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0b1011);
        (LVT_TIMER_VIRT_ADDR.as_ptr_mut() as *mut u32)
            .write_volatile((2 << 16) | VEC_LOCAL_APIC_TIMER_INT as u32);
        // non masked, periodic
        self.start();
        acpi::pm_timer_wait_ms(10)?;
        let tick = self.get_current_tick();
        self.stop();

        self.freq = NonZeroUsize::new(tick);
        self.tick = 0;

        Ok(())
    }

    unsafe fn start(&self) {
        (INIT_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0x0010_0000);
    }

    unsafe fn stop(&mut self) {
        (INIT_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0);
    }

    fn tick(&mut self) {
        if self.tick == usize::MAX {
            self.tick = 0;
        }

        self.tick += 1;
    }

    fn get_current_tick(&self) -> usize {
        self.tick
    }
}

pub fn init() -> Result<()> {
    unsafe { LOCAL_APIC_TIMER.init() }?;

    info!("acpi: Local APIC timer initialized");
    Ok(())
}

pub fn start() {
    unsafe { LOCAL_APIC_TIMER.start() };

    info!("acpi: Local APIC timer started");
}

pub fn tick() {
    unsafe { LOCAL_APIC_TIMER.tick() };
}

pub fn get_current_tick() -> usize {
    unsafe { LOCAL_APIC_TIMER.get_current_tick() }
}

pub fn get_current_ms() -> Option<usize> {
    let freq = unsafe { LOCAL_APIC_TIMER.freq }?;
    Some(get_current_tick() / freq * 10)
}
