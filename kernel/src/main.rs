#![no_std]
#![no_main]
#![feature(start)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod arch;
mod bus;
mod device;
mod env;
mod error;
mod fs;
mod graphics;
mod mem;
mod panic;
mod serial;
mod util;

extern crate alloc;

use alloc::alloc::Layout;
use arch::{
    asm,
    task::{executor::Executor, Task},
};
use common::boot_info::BootInfo;
use log::*;
use serial::ComPort;
use util::{ascii::AsciiCode, logger};

use crate::{
    arch::{apic::timer::LOCAL_APIC_TIMER, idt},
    device::console,
};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: *const BootInfo) -> ! {
    let boot_info = unsafe { boot_info.read() };

    // initialize local APIC timer
    LOCAL_APIC_TIMER.init();

    // initialize serial
    serial::init(ComPort::Com1);

    // initialize logger
    logger::init();

    // initialize frame buffer, console
    graphics::init(
        boot_info.graphic_info,
        (3, 26, 0).into(),
        (18, 202, 99).into(),
    );

    // initialize GDT (TODO: not working correctly)
    //gdt::init();
    // initialize PIC and IDT
    idt::init_pic();
    idt::init_idt();

    // initialize memory management
    mem::init(boot_info.get_mem_map());

    // initialize pci, usb
    bus::init();

    // initialize device drivers
    device::init();

    env::print_info();

    // initramfs
    //let initramfs_start_virt_addr = VirtualAddress::new(boot_info.initramfs_start_virt_addr);
    //let initramfs_fat_volume = FatVolume::new(initramfs_start_virt_addr);
    //initramfs_fat_volume.debug();

    let mut executor = Executor::new();
    executor.spawn(Task::new(console_task()));
    //executor.spawn(Task::new(serial_terminal_task()));
    executor.run();

    loop {
        asm::hlt();
    }
}

async fn console_task() {
    loop {
        let ascii_code = match serial::receive_data() {
            Some(data) => match data.try_into() {
                Ok(c) => c,
                Err(_) => continue,
            },
            None => continue,
        };

        console::input(ascii_code);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
