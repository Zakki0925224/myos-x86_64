#![no_std]
#![no_main]
#![feature(start)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]

mod arch;
mod bus;
mod debug_terminal;
mod device;
mod env;
mod error;
mod fs;
mod graphics;
mod mem;
mod serial;
mod util;

extern crate alloc;

use alloc::alloc::Layout;
use arch::{
    addr::{Address, VirtualAddress},
    asm,
    task::{executor::Executor, Task},
};
use common::boot_info::BootInfo;
use core::panic::PanicInfo;
use debug_terminal::Terminal;
use fs::fat::FatVolume;
use log::*;
use serial::ComPort;
use util::{ascii::AsciiCode, logger};

use crate::{
    arch::{apic::timer::LOCAL_APIC_TIMER, gdt, idt},
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

    // initialize frame buffer, console
    graphics::init(boot_info.graphic_info);

    // initialize logger
    logger::init();

    // initialize GDT (TODO: not working correctly)
    //gdt::init();
    // initialize IDT
    idt::init();

    // initialize memory management
    mem::init(boot_info.get_mem_map());

    // initialize pci
    bus::init();

    // initialize device drivers
    device::init();

    env::print_info();

    // initramfs
    let initramfs_start_virt_addr = VirtualAddress::new(boot_info.initramfs_start_virt_addr);
    let initramfs_fat_volume = FatVolume::new(initramfs_start_virt_addr);
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
        match ascii_code {
            AsciiCode::CarriageReturn => {
                println!();
            }
            code => {
                print!("{}", code as u8 as char);
            }
        }
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("Allocation error: {:?}", layout);
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("{:?}", info.message());
    error!("{:?}", info.location());

    loop {
        asm::hlt();
    }
}
