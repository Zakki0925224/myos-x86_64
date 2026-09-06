use crate::arch::x86_64::cpu;

pub fn init() {
    // check TSC available
    let info = cpu::version_info();
    if !info.feature_tsc {
        panic!("TSC not available");
    }
}
