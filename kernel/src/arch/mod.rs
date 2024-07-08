use register::{
    control::{Cr0, Cr4},
    Register,
};

pub mod addr;
pub mod apic;
pub mod asm;
pub mod context;
pub mod gdt;
pub mod idt;
pub mod qemu;
pub mod register;
pub mod syscall;
pub mod task;
pub mod tss;

// TODO
pub fn enable_sse() {
    let mut cr0 = Cr0::read();
    cr0.set_emulation(false);
    cr0.set_monitor_coporsessor(true);
    cr0.write();
    cr0 = Cr0::read();
    assert_eq!(cr0.emulation(), true);
    assert_eq!(cr0.monitor_coprocessor(), true);

    let mut cr4 = Cr4::read();
    cr4.set_osfxsr(true);
    cr4.set_osxmmexcept(true);
    cr4.write();
    cr4 = Cr4::read();
    assert_eq!(cr4.osfxsr(), true);
    assert_eq!(cr4.osxmmexcept(), true);
}
