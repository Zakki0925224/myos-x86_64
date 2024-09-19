#![no_std]
#![no_main]
#![feature(start)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(sync_unsafe_cell)]
#![feature(naked_functions)]
#![feature(custom_test_frameworks)]
#![test_runner(test::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod arch;
mod bus;
mod device;
mod env;
mod error;
mod fs;
mod graphics;
mod mem;
mod net;
mod panic;
mod test;
mod util;

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use arch::*;
use common::boot_info::BootInfo;
use device::uart;
use fs::{file::bitmap::BitmapImage, vfs};
use graphics::{color::*, simple_window_manager};
use log::error;
use util::logger;

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_entry(boot_info: &BootInfo) -> ! {
    context::switch_kernel_stack(kernel_main, boot_info);
}

pub extern "sysv64" fn kernel_main(boot_info: &BootInfo) -> ! {
    // initialize logger
    logger::init();

    // attach uart driver
    device::uart::probe_and_attach().unwrap();

    // initialize memory management
    mem::init(boot_info.mem_map);

    // initialize GDT
    gdt::init();
    // initialize PIC and IDT
    idt::init_pic();
    idt::init_idt();

    // initialize ACPI
    if let Some(rsdp_virt_addr) = boot_info.rsdp_virt_addr {
        acpi::init(rsdp_virt_addr.into()).unwrap();
    }

    // initialize and start local APIC timer
    apic::timer::init().unwrap();
    apic::timer::start();

    // initialize frame buffer, console
    graphics::init(&boot_info.graphic_info, PN_COLOR_1, SS_COLOR_1);

    // initialize graphics shadow buffer and layer manager
    graphics::enable_shadow_buf();
    graphics::init_layer_man(&boot_info.graphic_info, RgbColorCode::default());
    // initialize simple window manager
    graphics::init_simple_wm();

    // enable syscall
    syscall::enable();

    // initialize pci, usb
    bus::init();

    // initialize device drivers
    device::init();

    // initialize initramfs, VFS
    fs::init(
        boot_info.initramfs_start_virt_addr.into(),
        &boot_info.kernel_config,
    );

    #[cfg(test)]
    test_main();

    env::print_info();

    // tasks
    let task_poll_virtio_net = async {
        loop {
            let _ = device::virtio::net::poll_normal();
            task::exec_yield().await;
        }
    };

    let task_poll_uart = async {
        loop {
            if let Ok(Some(s)) = device::uart::poll_normal() {
                if let Err(err) = device::console::exec_cmd(s) {
                    error!("{:?}", err);
                }
                device::console::print_prompt();
            }
            task::exec_yield().await;
        }
    };

    let task_poll_ps2_keyboard = async {
        loop {
            if let Ok(Some(s)) = device::ps2_keyboard::poll_normal() {
                if let Err(err) = device::console::exec_cmd(s) {
                    error!("{:?}", err);
                }
                device::console::print_prompt();
            }
            task::exec_yield().await;
        }
    };

    task::spawn(task_poll_virtio_net).unwrap();
    task::spawn(task_poll_uart).unwrap();
    task::spawn(task_poll_ps2_keyboard).unwrap();
    task::spawn(poll_ps2_mouse()).unwrap();
    task::ready().unwrap();

    // execute init app
    let init_app_exec_args = boot_info.kernel_config.init_app_exec_args;
    if let Some(args) = init_app_exec_args {
        let splited: Vec<&str> = args.split(" ").collect();

        if splited.len() == 0 || splited[0] == "" {
            error!("Invalid init app exec args: {:?}", args);
        } else if let Err(err) = fs::exec::exec_elf(splited[0], &splited[1..]) {
            error!("{:?}", err);
        }
    }

    loop {
        arch::hlt();
    }
}

async fn poll_ps2_mouse() {
    let mut is_created_mouse_pointer_layer = false;
    let mouse_pointer_bmp_fd = match vfs::open_file("/mnt/initramfs/sys/mouse_pointer.bmp") {
        Ok(fd) => fd,
        Err(_) => {
            error!("Failed to open mouse pointer bitmap");
            return;
        }
    };
    let bmp_data = match vfs::read_file(&mouse_pointer_bmp_fd) {
        Ok(data) => data,
        Err(_) => {
            error!("Failed to read mouse pointer bitmap");
            return;
        }
    };
    let pointer_bmp = BitmapImage::new(&bmp_data);
    if vfs::close_file(&mouse_pointer_bmp_fd).is_err() {
        error!("Failed to close mouse pointer bitmap");
        return;
    }

    loop {
        let mouse_event = match device::ps2_mouse::poll_normal() {
            Ok(Some(e)) => e,
            _ => {
                task::exec_yield().await;
                continue;
            }
        };

        if !is_created_mouse_pointer_layer
            && simple_window_manager::create_mouse_pointer(&pointer_bmp).is_ok()
        {
            is_created_mouse_pointer_layer = true;
        }

        if is_created_mouse_pointer_layer {
            let _ = simple_window_manager::mouse_pointer_event(mouse_event);
        }
        task::exec_yield().await;
    }
}
