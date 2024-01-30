use super::initramfs;
use crate::{
    arch::gdt,
    mem::{
        bitmap,
        paging::{
            page_table::{EntryMode, ReadWrite},
            PAGE_SIZE,
        },
    },
};
use common::elf::{self, Elf64};
use core::mem;
use log::{error, info};

pub fn exec_elf(file_name: &str, args: &[&str]) {
    info!("exec: args: {:?}", args);

    let (_, elf_data) = match initramfs::get_file(file_name) {
        Ok(res) => match res {
            Some((meta, data)) => (meta, data),
            None => {
                error!("exec: The file \"{}\" does not exist", file_name);
                return;
            }
        },
        Err(_) => {
            error!("exec: Initramfs is locked");
            return;
        }
    };

    let elf64 = match Elf64::new(&elf_data) {
        Ok(e) => e,
        Err(_) => {
            error!("exec: The file \"{}\" is not an executable file", file_name);
            return;
        }
    };

    let header = elf64.read_header();

    if header.elf_type() != elf::Type::Executable {
        error!("exec: The file \"{}\" is not an executable file", file_name);
        return;
    }

    if header.machine() != elf::Machine::X8664 {
        error!("exec: Unsupported ISA");
        return;
    }

    let app_stack = bitmap::alloc_mem_frame(8).unwrap();
    let app_mem = bitmap::alloc_mem_frame(elf_data.len() / PAGE_SIZE + 1).unwrap();

    // copy elf data
    app_mem.get_frame_start_virt_addr().write_volatile(elf_data);

    app_stack
        .set_permissions(ReadWrite::Write, EntryMode::User)
        .unwrap();
    app_mem
        .set_permissions(ReadWrite::Write, EntryMode::User)
        .unwrap();

    let entry_point: extern "sysv64" fn() -> i32 = unsafe {
        mem::transmute(
            app_mem
                .get_frame_start_virt_addr()
                .offset(header.entry_point as usize),
        )
    };

    // TODO
    // set to user segment
    gdt::set_seg_reg_to_user();

    let ret = entry_point();

    // set to kernel segemnt
    gdt::set_seg_reg_to_kernel();

    app_stack
        .set_permissions(ReadWrite::Write, EntryMode::Supervisor)
        .unwrap();
    app_mem
        .set_permissions(ReadWrite::Write, EntryMode::Supervisor)
        .unwrap();

    bitmap::dealloc_mem_frame(app_stack).unwrap();
    bitmap::dealloc_mem_frame(app_mem).unwrap();

    info!("exec: Exited ({})", ret);
}
