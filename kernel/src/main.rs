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

use alloc::{string::String, vec::Vec};
use arch::{apic, asm, context, gdt, idt, qemu, syscall, task};
use bus::pci;
use common::boot_info::BootInfo;
use device::console;
use error::Result;
use fs::{exec, initramfs};
use log::error;
use serial::ComPort;
use util::{ascii::AsciiCode, logger};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_entry(boot_info: *const BootInfo) -> ! {
    context::switch_kernel_stack(kernel_main, boot_info);
}

pub extern "sysv64" fn kernel_main(boot_info: *const BootInfo) -> ! {
    let boot_info = unsafe { boot_info.read() };

    // initialize and start local APIC timer
    apic::timer::init();
    apic::timer::start();

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

    // initialize graphics layer manager
    // TODO
    //graphics::init_layer_man(boot_info.graphic_info, ColorCode::Rgb { r: 0, g: 0, b: 0 });

    // initialize syscall configurations
    syscall::init();

    // initialize pci, usb
    bus::init();

    // initialize device drivers
    device::init();

    // initramfs
    initramfs::init(boot_info.initramfs_start_virt_addr.into());

    env::print_info();

    context::save_kernel_context();

    // tasks
    task::spawn(serial_receive_task()).unwrap();
    task::run().unwrap();

    // unreachable?
    loop {
        asm::hlt();
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

        match ascii_code {
            AsciiCode::CarriageReturn => {
                println!();
            }
            code => {
                print!("{}", code as u8 as char);
            }
        }

        let cmd = match console::input(ascii_code) {
            Ok(s) => match s {
                Some(s) => s,
                None => {
                    task::exec_yield().await;
                    continue;
                }
            },
            Err(_) => {
                error!("Console is locked");
                task::exec_yield().await;
                continue;
            }
        };

        if let Err(err) = exec_cmd(cmd).await {
            error!("{:?}", err);
        }
        println!();
    }
}

async fn exec_cmd(cmd: String) -> Result<()> {
    let args: Vec<&str> = cmd.trim().split(" ").collect();

    match args[0] {
        "info" => env::print_info(),
        "lspci" => pci::lspci()?,
        "free" => mem::free(),
        "exit" => qemu::exit(0),
        "echo" => println!("{}", &cmd[4..].trim()),
        "break" => asm::int3(),
        "ls" => initramfs::ls(),
        "cd" => {
            if args.len() == 2 {
                initramfs::cd(args[1]);
            }
        }
        "cat" => {
            if args.len() == 2 {
                initramfs::cat(args[1]);
            }
        }
        "exec" => {
            if args.len() == 2 {
                exec::exec_elf(args[1], &args[2..]);
            }
        }
        "" => (),
        cmd => error!("Command {:?} was not found", cmd),
    }

    Ok(())
}
