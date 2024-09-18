use crate::{
    acpi,
    arch::{addr::*, idt::VEC_LOCAL_APIC_TIMER_INT},
    error::Result,
};

const LVT_TIMER_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00320);
const INIT_CNT_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee00380);
const DIV_CONF_VIRT_ADDR: VirtualAddress = VirtualAddress::new(0xfee003e0);

static mut LOCAL_APIC_TIMER: Timer = Timer::new();

struct Timer {
    tick: usize,
    freq: Option<usize>,
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            tick: 0,
            freq: None,
        }
    }

    pub unsafe fn init(&mut self) -> Result<()> {
        // calc freq
        self.stop();
        (DIV_CONF_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0b1011);
        (LVT_TIMER_VIRT_ADDR.as_ptr_mut() as *mut u32)
            .write_volatile((2 << 16) | VEC_LOCAL_APIC_TIMER_INT as u32);
        // non masked, periodic
        self.start();
        acpi::pm_timer_wait_ms(1)?;
        let tick = self.get_current_tick();
        self.stop();
        self.freq = Some(tick);
        self.tick = 0;

        Ok(())
    }

    pub unsafe fn start(&self) {
        (INIT_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0x0010_0000);
    }

    pub unsafe fn stop(&mut self) {
        (INIT_CNT_VIRT_ADDR.as_ptr_mut() as *mut u32).write_volatile(0);
    }

    pub fn tick(&mut self) {
        if self.tick == usize::MAX {
            self.tick = 0;
        }

        self.tick += 1;
    }

    pub fn get_current_tick(&self) -> usize {
        self.tick
    }
}

pub fn init() -> Result<()> {
    unsafe { LOCAL_APIC_TIMER.init() }
}

pub fn start() {
    unsafe { LOCAL_APIC_TIMER.start() };
}

// pub fn stop() {
//     unsafe { LOCAL_APIC_TIMER.stop() };
// }

pub fn tick() {
    unsafe { LOCAL_APIC_TIMER.tick() };
}

pub fn get_current_tick() -> usize {
    unsafe { LOCAL_APIC_TIMER.get_current_tick() }
}

pub fn tick_to_ms(tick: usize) -> usize {
    match unsafe { LOCAL_APIC_TIMER.freq } {
        Some(freq) => tick / freq,
        None => 0,
    }
}
