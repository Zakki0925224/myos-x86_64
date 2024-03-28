use super::initramfs;
use crate::{
    arch::task,
    error::Result,
    mem::{bitmap, paging::PAGE_SIZE},
};
use common::elf::{self, Elf64};
use core::mem;
use log::{error, info};

pub fn exec_elf(file_name: &str, args: &[&str]) -> Result<()> {
    let (_, elf_data) = match initramfs::get_file(file_name) {
        Ok(res) => match res {
            Some((meta, data)) => (meta, data),
            None => {
                error!("exec: The file \"{}\" does not exist", file_name);
                return Ok(());
            }
        },
        Err(e) => {
            return Err(e);
        }
    };

    let elf64 = match Elf64::new(&elf_data) {
        Ok(e) => e,
        Err(_) => {
            error!("exec: The file \"{}\" is not an executable file", file_name);
            return Ok(());
        }
    };

    let header = elf64.header();

    if header.elf_type() != elf::Type::Executable {
        error!("exec: The file \"{}\" is not an executable file", file_name);
        return Ok(());
    }

    if header.machine() != elf::Machine::X8664 {
        error!("exec: Unsupported ISA");
        return Ok(());
    }

    let text_section_header = match elf64.section_header_by_name(".text") {
        Some(sh) => sh,
        None => {
            error!("exec: \".text\" section was not found");
            return Ok(());
        }
    };

    let text_section_data = match elf64.data_by_section_header(text_section_header) {
        Some(data) => data,
        None => {
            error!("exec: Failed to get \".text\" section data");
            return Ok(());
        }
    };

    // copy .text data to user frame
    let user_mem_frame_info =
        bitmap::alloc_mem_frame((text_section_data.len() / PAGE_SIZE).max(1))?;
    user_mem_frame_info
        .frame_start_virt_addr
        .copy_from_nonoverlapping(text_section_data.as_ptr(), text_section_data.len());
    user_mem_frame_info.set_permissions_to_user()?;
    let entry_addr = user_mem_frame_info.frame_start_virt_addr.get() + header.entry_point
        - text_section_header.addr;

    info!("entry: 0x{:x}", entry_addr);
    let entry: extern "sysv64" fn() = unsafe { mem::transmute(entry_addr as *const ()) };
    let exit_code = task::exec_user_task(entry, file_name, args)?;

    user_mem_frame_info.set_permissions_to_supervisor()?;
    bitmap::dealloc_mem_frame(user_mem_frame_info)?;

    info!("exec: Exited (code: {})", exit_code);

    Ok(())
}
