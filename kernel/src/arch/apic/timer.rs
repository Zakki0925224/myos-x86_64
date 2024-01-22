use crate::arch::addr::*;

static LOCAL_APIC_TIMER: Timer = Timer::new();

pub struct Timer {
    lvt_timer_virt_addr: VirtualAddress,
    initial_cnt_virt_addr: VirtualAddress,
    current_cnt_virt_addr: VirtualAddress,
    divide_conf_virt_addr: VirtualAddress,
}

impl Timer {
    pub const fn new() -> Self {
        Self {
            lvt_timer_virt_addr: VirtualAddress::new(0xfee00320),
            initial_cnt_virt_addr: VirtualAddress::new(0xfee00380),
            current_cnt_virt_addr: VirtualAddress::new(0xfee00390),
            divide_conf_virt_addr: VirtualAddress::new(0xfee003e0),
        }
    }

    pub fn init(&self) {
        self.stop();
        self.divide_conf_virt_addr.write_volatile::<u32>(0b1011);
        self.lvt_timer_virt_addr
            .write_volatile::<u32>((1 << 16) | 32); // masked, oneshot
    }

    pub fn start(&self) {
        self.initial_cnt_virt_addr.write_volatile::<u32>(u32::MAX);
    }

    pub fn stop(&self) {
        self.initial_cnt_virt_addr.write_volatile::<u32>(0);
    }

    pub fn elapsed(&self) -> u32 {
        let elapsed = u32::MAX - self.current_cnt_virt_addr.read_volatile::<u32>();
        elapsed
    }

    pub fn is_measuring(&self) -> bool {
        self.elapsed() != u32::MAX
    }
}

pub fn init() {
    LOCAL_APIC_TIMER.init();
}

pub fn start() {
    LOCAL_APIC_TIMER.start();
}

pub fn stop() {
    LOCAL_APIC_TIMER.stop();
}

pub fn elapsed() -> u32 {
    LOCAL_APIC_TIMER.elapsed()
}

pub fn is_measuring() -> bool {
    LOCAL_APIC_TIMER.is_measuring()
}
