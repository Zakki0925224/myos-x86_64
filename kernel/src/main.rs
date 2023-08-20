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
mod fs;
mod graphics;
mod mem;
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
use device::serial::SERIAL;
use fs::fat::FatVolume;
use log::*;

use crate::arch::{apic::timer::LOCAL_APIC_TIMER, gdt, idt};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: *const BootInfo) -> ! {
    let boot_info = unsafe { boot_info.read() };

    // initialize local APIC timer
    LOCAL_APIC_TIMER.init();

    // initialize frame buffer, serial, terminal, logger
    graphics::init(boot_info.graphic_info);

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

    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(example_task()));
    executor.run();

    // initramfs
    let initramfs_start_virt_addr = VirtualAddress::new(boot_info.initramfs_start_virt_addr);
    let initramfs_fat_volume = FatVolume::new(initramfs_start_virt_addr);
    //initramfs_fat_volume.debug();

    loop {
        if !SERIAL.is_locked() {
            asm::cli();
            let data = SERIAL.lock().receive_data();
            println!("data: {:?}", data);
            asm::sti();
        }

        //asm::hlt();
    }
}

async fn async_num() -> u32 {
    return 42;
}

async fn example_task() {
    let num = async_num().await;
    println!("async num: {}", num);
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
