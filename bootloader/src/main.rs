#![no_std]
#![no_main]
#![feature(abi_efiapi)]

#[macro_use]
extern crate log;

use core::{fmt::Write, arch::asm};
use uefi::prelude::*;

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status
{
    uefi_services::init(&mut system_table).unwrap();
    let boot_service = system_table.boot_services();

    info!("Running bootloader...");

    loop { unsafe { asm!("hlt") } }

    return Status::SUCCESS;
}