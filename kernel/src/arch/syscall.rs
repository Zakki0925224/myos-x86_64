use crate::{arch::register::model_specific::*, println};
use log::info;

extern "sysv64" fn syscall_handler() {
    println!("Called syscall!");
}

pub fn enable_system_call() {
    let mut efer = ExtendedFeatureEnableRegister::read();
    efer.set_system_call_enable(true);
    efer.write();

    let mut lstar = LongModeSystemCallTargetAddressRegister::read();
    lstar.set_target_addr(syscall_handler as *const () as u64);
    lstar.write();

    let mut star = SystemCallTargetAddressRegister::read();
    star.set_target_addr((8 << 32) | ((16 | 3) << 48));
    star.write();

    let mut fmask = SystemCallFlagMaskRegister::read();
    fmask.set_value(0);
    fmask.write();

    info!("arch: Enabled system call");
}
