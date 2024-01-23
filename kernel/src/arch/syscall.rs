use crate::arch::register::model_specific::*;
use core::arch::global_asm;
use log::info;

global_asm!(
    ".global syscall_handler",
    ".global asm_syscall_handler",
    "asm_syscall_handler:",
    " push rcx",
    " push r11",
    " push rbx",
    " push rbp",
    " push r15",
    " push r14",
    " push r13",
    " push r12",
    " push r10",
    " push r9",
    " push r8",
    " push rdi",
    " push rsi",
    " push rdx",
    " push rax",
    " mov rbp, rsp",
    " mov rdi, rsp",
    " and rsp, -16",
    " call syscall_handler",
    " mov rsp, rbp",
    " pop rax",
    " pop rdx",
    " pop rsi",
    " pop rdi",
    " pop r8",
    " pop r9",
    " pop r10",
    " pop r12",
    " pop r13",
    " pop r14",
    " pop r15",
    " pop rbp",
    " pop rbx",
    " pop r11",
    " pop rcx",
    " sysretq"
);

extern "C" {
    fn asm_syscall_handler();
}

#[no_mangle]
extern "sysv64" fn syscall_handler(args: &[u64; 16]) {
    info!("syscall: Called!(args: {:?})", args);
}

pub fn init() {
    let mut efer = ExtendedFeatureEnableRegister::read();
    efer.set_system_call_enable(true);
    efer.write();

    let mut lstar = LongModeSystemCallTargetAddressRegister::read();
    lstar.set_target_addr(asm_syscall_handler as *const () as u64);
    lstar.write();

    let mut star = SystemCallTargetAddressRegister::read();
    star.set_target_addr((8 << 32) | ((16 | 3) << 48));
    star.write();

    let mut fmask = SystemCallFlagMaskRegister::read();
    fmask.set_value(0);
    fmask.write();

    info!("arch: Enabled syscall");
}
