#![no_std]
#![no_main]
#![feature(start)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(sync_unsafe_cell)]
#![feature(naked_functions)]

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

#[macro_use]
extern crate alloc;

use alloc::{string::String, vec::Vec};
use arch::{apic, asm, context, gdt, idt, qemu, syscall, task};
use bus::pci;
use common::boot_info::BootInfo;
use device::console;
use error::Result;
use fs::{exec, vfs};
use graphics::{color::COLOR_SILVER, draw::Draw, multi_layer};
use log::error;
use serial::ComPort;
use util::{ascii::AsciiCode, hexdump, logger};

#[no_mangle]
#[start]
pub extern "sysv64" fn kernel_entry(boot_info: &BootInfo) -> ! {
    context::switch_kernel_stack(kernel_main, boot_info);
}

pub extern "sysv64" fn kernel_main(boot_info: &BootInfo) -> ! {
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

    // initialize memory management
    mem::init(boot_info.mem_map);

    // initialize GDT
    gdt::init();
    // initialize PIC and IDT
    idt::init_pic();
    idt::init_idt();

    // initialize graphics shadow buffer and layer manager
    //graphics::enable_shadow_buf();
    //graphics::init_layer_man(boot_info.graphic_info, ColorCode::Rgb { r: 0, g: 0, b: 0 });

    // initialize syscall configurations
    syscall::init();

    // initialize pci, usb
    bus::init();

    // initialize device drivers
    device::init();

    // initialize initramfs, VFS
    fs::init(boot_info.initramfs_start_virt_addr.into());

    env::print_info();

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
        print!("\n$> ");
    }
}

async fn exec_cmd(cmd: String) -> Result<()> {
    let args: Vec<&str> = cmd.trim().split(" ").collect();

    match args[0] {
        "info" => env::print_info(),
        "lspci" => pci::lspci()?,
        "free" => mem::free(),
        "exit" => qemu::exit(0),
        "break" => asm::int3(),
        "cd" => {
            if args.len() == 2 {
                vfs::chdir(args[1])?;
            }
        }
        "ls" => {
            let file_names = vfs::cwd_file_names()?;
            for n in file_names {
                print!("{} ", n);
            }
            println!();
        }
        "hexdump" => {
            if args.len() >= 2 {
                let file = vfs::read_file(args[1])?;
                hexdump::hexdump(&file);
            }
        }
        "exec" => {
            if args.len() >= 2 {
                exec::exec_elf(args[1], &args[2..])?;
            }
        }
        "window" => {
            asm::disabled_int_func(|| {
                let mut sample_window_layer = multi_layer::create_layer(200, 50, 500, 300).unwrap();
                sample_window_layer.fill(COLOR_SILVER).unwrap();
                multi_layer::push_layer(sample_window_layer).unwrap();
            });
        }
        "" => (),
        cmd => error!("Command {:?} was not found", cmd),
    }

    Ok(())
}
