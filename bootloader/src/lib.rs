#![no_std]
#![no_main]
#![feature(abi_efiapi)]

use core::fmt::Write;
use uefi::{prelude::*, table::runtime::ResetType};

#[entry]
fn efi_main(_handle: Handle, mut system_table: SystemTable<Boot>) -> Status
{
    uefi_services::init(&mut system_table).unwrap();

    system_table.stdout().reset(false).unwrap();
    writeln!(system_table.stdout(), "Hello world!");
    system_table.stdout().reset(false).unwrap();
    system_table.runtime_services().reset(ResetType::Shutdown, Status::SUCCESS, None);
}