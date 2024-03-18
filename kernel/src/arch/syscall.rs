use crate::arch::{
    gdt::{KERNEL_MODE_CS_VALUE, KERNEL_MODE_SS_VALUE},
    register::model_specific::*,
    task,
};
use core::arch::asm;
use log::{error, info};

#[naked]
extern "sysv64" fn asm_syscall_handler() {
    unsafe {
        asm!(
            "push rbp",
            "push rcx",
            "push r11",
            "mov rcx, r10", // rcx was updated by syscall instruction
            "mov rbp, rsp",
            "and rsp, -16",
            "call syscall_handler",
            "mov rsp, rbp",
            "pop r11",
            "pop rcx",
            "pop rbp",
            "sysretq",
            options(noreturn)
        );
    }
}

#[no_mangle]
extern "sysv64" fn syscall_handler(
    arg0: u64, // (sysv abi) rdi
    arg1: u64, // (sysv abi) rsi
    arg2: u64, // (sysv abi) rdx
    arg3: u64, // (sysv abi) rcx from r10
    arg4: u64, // (sysv abi) r8
    arg5: u64, // (sysv abi) r9
) -> u64 /* rax */ {
    let mut ret_val = 0xdeadbeef01234567;
    let args = [arg0, arg1, arg2, arg3, arg4, arg5];
    info!("syscall: Called!(args: {:?})", args);

    match arg0 {
        3 => {
            info!("syscall: test (ret: 0x{:x})", ret_val);
        }
        4 => {
            info!("syscall: exit (status: 0x{:x})", arg1);
            task::return_to_kernel_task();
        }
        num => {
            error!("syscall: Syscall number 0x{:x} is not defined", num);
            ret_val = u64::MAX;
        }
    }

    ret_val
}

pub fn init() {
    let mut efer = ExtendedFeatureEnableRegister::read();
    efer.set_system_call_enable(true);
    efer.write();

    let mut lstar = LongModeSystemCallTargetAddressRegister::read();
    lstar.set_target_addr(asm_syscall_handler as *const () as u64);
    lstar.write();

    let mut star = SystemCallTargetAddressRegister::read();
    star.set_target_addr(
        ((KERNEL_MODE_CS_VALUE as u64) << 32) | ((KERNEL_MODE_SS_VALUE as u64 | 3) << 48),
    ); // set CS and SS to kernel segment
    star.write();

    let mut fmask = SystemCallFlagMaskRegister::read();
    fmask.set_value(0);
    fmask.write();

    info!("syscall: Initialized syscall");
}
