#![no_std]
#![no_main]
#![feature(start)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(sync_unsafe_cell)]

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

use crate::{
    arch::{idt, task},
    device::console,
};
use alloc::alloc::Layout;
use arch::{
    apic, asm, gdt, syscall,
    task::{Executor, Task},
};
use common::boot_info::BootInfo;
use fs::initramfs;
use log::*;
use serial::ComPort;
use util::logger;

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: *const BootInfo) -> ! {
    let boot_info = unsafe { boot_info.read() };

    // initialize local APIC timer
    apic::timer::init();

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

    // initialize GDT
    gdt::init();
    // initialize PIC and IDT
    idt::init_pic();
    idt::init_idt();

    // initialize memory management
    mem::init(boot_info.get_mem_map());

    // initialize syscall configurations
    syscall::init();

    // initialize pci, usb
    bus::init();

    // initialize device drivers
    device::init();

    env::print_info();

    // initramfs
    initramfs::init(boot_info.initramfs_start_virt_addr.into());

    let mut executor = Executor::new();
    executor.spawn(Task::new(test()));
    executor.spawn(Task::new(test2()));
    executor.spawn(Task::new(serial_receive_task()));
    executor.run();

    loop {
        asm::hlt();
    }
}

async fn test() {
    for i in 0..100 {
        println!("hoge({})", i);
        task::exec_yield().await;
    }
}

async fn test2() {
    for i in 0..100 {
        println!("huga({})", i);
        task::exec_yield().await;
    }
}

async fn serial_receive_task() {
    loop {
        let ascii_code = match serial::receive_data() {
            Some(data) => match data.try_into() {
                Ok(c) => c,
                Err(_) => {
                    task::exec_yield().await;
                    continue;
                }
            },
            None => {
                task::exec_yield().await;
                continue;
            }
        };

        if console::input(ascii_code).is_err() {
            error!("Console is locked");
        }
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}
