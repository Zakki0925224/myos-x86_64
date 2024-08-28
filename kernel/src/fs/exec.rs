use super::vfs;
use crate::{
    arch::task,
    error::{Error, Result},
    mem::{
        bitmap::{self, MemoryFrameInfo},
        paging::{self, EntryMode, PageWriteThroughLevel, ReadWrite, PAGE_SIZE},
    },
};
use alloc::vec::Vec;
use common::elf::{self, Elf64, SegmentType};
use core::mem;
use log::info;

pub fn exec_elf(elf_path: &str, args: &[&str]) -> Result<()> {
    let fd_num = vfs::open_file(elf_path)?;

    let elf_data = vfs::read_file(&fd_num)?;
    let elf64 = match Elf64::new(&elf_data) {
        Ok(e) => e,
        Err(err) => return Err(err.into()),
    };

    vfs::close_file(&fd_num)?;

    let header = elf64.header();

    if header.elf_type() != elf::Type::Executable {
        return Err(Error::Failed("The file is not an executable file"));
    }

    if header.machine() != elf::Machine::X8664 {
        return Err(Error::Failed("Unsupported ISA"));
    }

    let mut allocated_mem_frames: Vec<MemoryFrameInfo> = Vec::new();
    let mut entry: Option<extern "sysv64" fn()> = None;

    for program_header in elf64.program_headers() {
        if program_header.segment_type() != SegmentType::Load {
            continue;
        }

        let p_virt_addr = program_header.virt_addr;
        let p_memsz = program_header.mem_size;
        let p_filesz = program_header.file_size;

        let pages_needed = ((p_virt_addr % PAGE_SIZE as u64 + p_memsz + PAGE_SIZE as u64 - 1)
            / PAGE_SIZE as u64) as usize;
        let user_mem_frame_info = bitmap::alloc_mem_frame(pages_needed)?;
        bitmap::mem_clear(&user_mem_frame_info)?;
        let user_mem_frame_start_virt_addr = user_mem_frame_info.frame_start_virt_addr()?;

        // copy data from file
        let program_data = elf64.data_by_program_header(program_header);
        if let Some(data) = program_data {
            user_mem_frame_start_virt_addr
                .offset((p_virt_addr % PAGE_SIZE as u64) as usize)
                .copy_from_nonoverlapping(data.as_ptr(), p_filesz as usize);
        }

        // update page mapping
        let start_virt_addr = (p_virt_addr / PAGE_SIZE as u64 * PAGE_SIZE as u64).into();
        paging::update_mapping(
            start_virt_addr,
            start_virt_addr.offset(user_mem_frame_info.frame_size),
            user_mem_frame_info.frame_start_phys_addr,
            ReadWrite::Write,
            EntryMode::User,
            PageWriteThroughLevel::WriteBack,
        )?;
        allocated_mem_frames.push(user_mem_frame_info);

        if header.entry_point >= p_virt_addr && header.entry_point < p_virt_addr + p_memsz {
            entry = Some(unsafe { mem::transmute(header.entry_point as *const ()) });
        }
    }

    if let Some(entry) = entry {
        let exit_code = task::exec_user_task(entry, elf_path, args)?;
        info!("exec: Exited (code: {})", exit_code);
    } else {
        return Err(Error::Failed("Entry point was not found"));
    }

    for mem_frame in allocated_mem_frames {
        // fix page mapping
        paging::update_mapping(
            mem_frame.frame_start_phys_addr.get().into(),
            (mem_frame.frame_start_phys_addr.get() + mem_frame.frame_size as u64).into(),
            mem_frame.frame_start_phys_addr,
            ReadWrite::Write,
            EntryMode::Supervisor,
            PageWriteThroughLevel::WriteBack,
        )?;
        bitmap::dealloc_mem_frame(mem_frame)?;
    }

    Ok(())
}
