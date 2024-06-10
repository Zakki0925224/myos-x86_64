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

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use arch::{apic, asm, context, gdt, idt, qemu, syscall, task};
use bus::pci;
use common::boot_info::BootInfo;
use device::{console, ps2_mouse};
use error::Result;
use fs::{exec, file::bitmap::BitmapImage, vfs};
use graphics::{color::RgbColorCode, simple_window_manager};
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
        &boot_info.graphic_info,
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
    graphics::enable_shadow_buf();
    graphics::init_layer_man(&boot_info.graphic_info, RgbColorCode::default());
    // initialize simple window manager
    graphics::init_simple_wm();

    // initialize syscall configurations
    syscall::init();

    // initialize pci, usb
    bus::init();

    // initialize device drivers
    device::init(&boot_info.graphic_info);

    // initialize initramfs, VFS
    fs::init(
        boot_info.initramfs_start_virt_addr.into(),
        &boot_info.kernel_config,
    );

    env::print_info();

    // execute init app
    let init_app_exec_args = boot_info.kernel_config.init_app_exec_args;
    if let Some(args) = init_app_exec_args {
        let splited: Vec<&str> = args.split(" ").collect();

        if splited.len() == 0 || splited[0] == "" {
            error!("Invalid init app exec args: {:?}", args);
        } else {
            if let Err(err) = fs::exec::exec_elf(splited[0], &splited[1..]) {
                error!("{:?}", err);
            }
        }
    }

    // tasks
    task::spawn(poll_serial()).unwrap();
    task::spawn(poll_ps2_mouse()).unwrap();
    task::run().unwrap();

    // unreachable?
    loop {
        asm::hlt();
    }
}

async fn poll_serial() {
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

        let cwd_path = vfs::cwd_path().unwrap_or(String::from("<UNKNOWN>"));
        print!("\n[{}]$ ", cwd_path);
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
        let mouse_event = match ps2_mouse::update() {
            Ok(Some(e)) => e,
            _ => {
                task::exec_yield().await;
                continue;
            }
        };

        if !is_created_mouse_pointer_layer
            && simple_window_manager::create_mouse_pointer_layer(&pointer_bmp).is_ok()
        {
            is_created_mouse_pointer_layer = true;
        }

        if is_created_mouse_pointer_layer {
            let _ = simple_window_manager::move_mouse_pointer(mouse_event);
        }
        task::exec_yield().await;
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
                let fd_num = vfs::open_file(args[1])?;
                let file = vfs::read_file(&fd_num)?;
                vfs::close_file(&fd_num)?;
                hexdump::hexdump(&file);
            }
        }
        "exec" => {
            if args.len() >= 2 {
                exec::exec_elf(args[1], &args[2..])?;
            }
        }
        "window" => {
            let _ =
                simple_window_manager::create_window("test window".to_string(), 200, 50, 300, 200);
        }
        "" => (),
        cmd => error!("Command {:?} was not found", cmd),
    }

    Ok(())
}
