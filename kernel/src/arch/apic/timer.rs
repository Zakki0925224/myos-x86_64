use crate::arch::{addr::*, idt::VEC_LOCAL_APIC_TIMER_INT};

static mut LOCAL_APIC_TIMER: Timer = Timer::new();

struct Timer {
    lvt_timer_virt_addr: VirtualAddress,
    initial_cnt_virt_addr: VirtualAddress,
    //current_cnt_virt_addr: VirtualAddress,
    divide_conf_virt_addr: VirtualAddress,
    tick: usize,
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            lvt_timer_virt_addr: VirtualAddress::new(0xfee00320),
            initial_cnt_virt_addr: VirtualAddress::new(0xfee00380),
            //current_cnt_virt_addr: VirtualAddress::new(0xfee00390),
            divide_conf_virt_addr: VirtualAddress::new(0xfee003e0),
            tick: 0,
        }
    }

    pub fn init(&self) {
        self.stop();
        self.divide_conf_virt_addr.write_volatile::<u32>(0b1011);
        self.lvt_timer_virt_addr
            .write_volatile::<u32>((2 << 16) | VEC_LOCAL_APIC_TIMER_INT as u32);
        // non masked, periodic
    }

    pub fn start(&self) {
        self.initial_cnt_virt_addr
            .write_volatile::<u32>(0x0010_0000);
    }

    pub fn stop(&self) {
        self.initial_cnt_virt_addr.write_volatile::<u32>(0);
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

pub fn init() {
    unsafe { LOCAL_APIC_TIMER.init() };
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
