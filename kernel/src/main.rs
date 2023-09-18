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
mod fs;
mod graphics;
mod mem;
mod util;

extern crate alloc;

use alloc::{alloc::Layout, vec};
use arch::{
    addr::{Address, VirtualAddress},
    asm,
    task::{executor::Executor, Task},
};
use common::boot_info::BootInfo;
use core::panic::PanicInfo;
use debug_terminal::Terminal;
use device::{
    console::{BufferType, CONSOLE},
    serial::SERIAL,
};
use fs::fat::FatVolume;
use log::*;
use util::ascii::AsciiCode;

use crate::arch::{apic::timer::LOCAL_APIC_TIMER, gdt, idt};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_main(boot_info: *const BootInfo) -> ! {
    let boot_info = unsafe { boot_info.read() };

    // initialize local APIC timer
    LOCAL_APIC_TIMER.init();

    // initialize frame buffer, serial, console, logger
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

    // initramfs
    let initramfs_start_virt_addr = VirtualAddress::new(boot_info.initramfs_start_virt_addr);
    let initramfs_fat_volume = FatVolume::new(initramfs_start_virt_addr);
    //initramfs_fat_volume.debug();

    // let mut console = device::console::Console::new();
    let buf_type = device::console::BufferType::Input;
    CONSOLE.lock().write(AsciiCode::SmallA, buf_type).unwrap();
    CONSOLE.lock().write(AsciiCode::SmallB, buf_type).unwrap();
    CONSOLE.lock().write(AsciiCode::SmallC, buf_type).unwrap();

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
        if SERIAL.is_locked() {
            continue;
        }

        asm::disabled_int_func(|| {
            let data = match SERIAL.lock().receive_data() {
                Some(data) => data,
                None => return,
            };

            let result = CONSOLE.lock().write(data.into(), BufferType::Input);

            if let Err(_) = result {
                //warn!("console: {:?}", err);

                // reset buffer and resend
                CONSOLE.lock().reset_buf(BufferType::Input);
                CONSOLE
                    .lock()
                    .write(data.into(), BufferType::Input)
                    .unwrap();
            }

            let result = CONSOLE.lock().write(data.into(), BufferType::Output);
            match result {
                Ok(_) => {}
                Err(_) => {
                    //warn!("console: {:?}", err);

                    // reset buffer and resend
                    CONSOLE.lock().reset_buf(BufferType::Output);
                    CONSOLE
                        .lock()
                        .write(data.into(), BufferType::Output)
                        .unwrap();
                }
            }
        });
    }
}

async fn serial_terminal_task() {
    info!("Starting debug terminal...");
    let mut terminal = Terminal::new();
    terminal.clear();

    loop {
        if SERIAL.is_locked() {
            continue;
        }

        asm::disabled_int_func(|| {
            let data = SERIAL.lock().receive_data();
            if let Some(data) = data {
                // skip invalid data
                if data <= AsciiCode::Delete as u8 {
                    terminal.input_char(data.into());
                }
            }
        });
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
