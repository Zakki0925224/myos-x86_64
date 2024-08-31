use super::{addr::VirtualAddress, task};
use crate::{
    arch::{
        gdt::*,
        register::{model_specific::*, Register},
    },
    device, env,
    error::*,
    fs::vfs::{self, file_desc::FileDescriptorNumber},
    mem::{bitmap, paging::PAGE_SIZE},
    print, util,
};
use alloc::{ffi::CString, string::*};
use common::libm::{Stat, Utsname};
use core::{arch::asm, slice};
use log::*;

#[naked]
extern "sysv64" fn asm_syscall_handler() {
    unsafe {
        asm!(
            "push rbp",
            "push rcx",
            "push r11",
            "mov rcx, r10", // rcx was updated by syscall instruction
            "mov rbp, rsp",
            "and rsp, -16",
            "call syscall_handler",
            "mov rsp, rbp",
            "pop r11",
            "pop rcx",
            "pop rbp",
            "sysretq",
            options(noreturn)
        );
    }
}

#[no_mangle]
extern "sysv64" fn syscall_handler(
    arg0: u64, // (sysv abi) rdi
    arg1: u64, // (sysv abi) rsi
    arg2: u64, // (sysv abi) rdx
    arg3: u64, // (sysv abi) rcx from r10
    arg4: u64, // (sysv abi) r8
    arg5: u64, // (sysv abi) r9
) -> i64 /* rax */ {
    //let args = [arg0, arg1, arg2, arg3, arg4, arg5];
    //info!("syscall: Called!(args: {:?})", args);

    match arg0 {
        // read syscall
        0 => {
            let fd = match FileDescriptorNumber::new_val(arg1 as i64) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: read: {:?}", err);
                    return -1;
                }
            };
            let buf_addr = arg2.into();
            let buf_len = arg3 as usize;
            if let Err(err) = sys_read(fd, buf_addr, buf_len) {
                error!("syscall: read: {:?}", err);
                return -1;
            }
        }
        // write syscall
        1 => {
            let fd = match FileDescriptorNumber::new_val(arg1 as i64) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: write: {:?}", err);
                    return -1;
                }
            };
            let s_ptr = arg2 as *const u8;
            let s_len = arg3 as usize;
            if let Err(err) = sys_write(fd, s_ptr, s_len) {
                error!("syscall: write: {:?}", err);
                return -1;
            }
        }
        // open syscall
        2 => {
            let filename_ptr = arg1 as *const u8;
            let fd = match sys_open(filename_ptr) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: open: {:?}", err);
                    return -1;
                }
            };
            return fd.get() as i64;
        }
        // close syscall
        3 => {
            let fd = match FileDescriptorNumber::new_val(arg1 as i64) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: close: {:?}", err);
                    return -1;
                }
            };
            if let Err(err) = sys_close(fd) {
                error!("syscall: close: {:?}", err);
                return -1;
            }
        }
        // exit syscall
        4 => {
            let status = arg1;
            sys_exit(status);
            unreachable!();
        }
        // sbrk syscall
        5 => {
            let len = arg1 as usize;
            let addr = match sys_sbrk(len) {
                Ok(addr) => addr.get(),
                Err(err) => {
                    error!("syscall: sbrk: {:?}", err);
                    return 0; // return null address
                }
            };
            return addr as i64;
        }
        // uname syscall
        6 => {
            if let Err(err) = sys_uname(arg1.into()) {
                error!("syscall: uname: {:?}", err);
                return -1;
            }
        }
        // break syscall
        7 => {
            sys_break();
            unreachable!();
        }
        // stat syscall
        8 => {
            let fd = match FileDescriptorNumber::new_val(arg1 as i64) {
                Ok(fd) => fd,
                Err(err) => {
                    error!("syscall: read: {:?}", err);
                    return -1;
                }
            };

            if let Err(err) = sys_stat(fd, arg2.into()) {
                error!("syscall: stat: {:?}", err);
                return -1;
            }
        }
        num => {
            error!("syscall: Syscall number 0x{:x} is not defined", num);
            return -1;
        }
    }

    0
}

fn sys_read(fd: FileDescriptorNumber, buf_addr: VirtualAddress, buf_len: usize) -> Result<()> {
    match fd {
        FileDescriptorNumber::STDOUT | FileDescriptorNumber::STDERR => {
            return Err(Error::Failed("fd is not defined"));
        }
        FileDescriptorNumber::STDIN => {
            // wait input enter
            // TODO: not occured ps2-kbd interrupt
            let mut input_s = None;
            while input_s.is_none() {
                if let Ok(s) = device::ps2_keyboard::poll_normal() {
                    input_s = s;
                }
                if let Ok(s) = device::uart::poll_normal() {
                    input_s = s;
                }
            }

            let c_s = CString::new(input_s.unwrap()).unwrap().into_bytes_with_nul();
            buf_addr.copy_from_nonoverlapping(c_s.as_ptr(), buf_len);
        }
        fd => {
            let data = vfs::read_file(&fd)?;

            if buf_len < data.len() {
                return Err(Error::Failed("buffer is too small"));
            }

            buf_addr.copy_from_nonoverlapping(data.as_ptr(), data.len());
        }
    }

    Ok(())
}

fn sys_write(fd: FileDescriptorNumber, s_ptr: *const u8, s_len: usize) -> Result<()> {
    let s_slice = unsafe { slice::from_raw_parts(s_ptr, s_len) };
    let s = String::from_utf8_lossy(s_slice).to_string();

    match fd {
        FileDescriptorNumber::STDOUT => {
            print!("{}", s);
        }
        _ => return Err(Error::Failed("fd is not defined")),
    }

    Ok(())
}

fn sys_open(filename_ptr: *const u8) -> Result<FileDescriptorNumber> {
    let filename = unsafe { util::cstring::from_cstring_ptr(filename_ptr) };
    vfs::open_file(&filename)
}

fn sys_close(fd: FileDescriptorNumber) -> Result<()> {
    vfs::close_file(&fd)
}

fn sys_exit(status: u64) {
    task::return_to_kernel_task(status);
}

fn sys_sbrk(len: usize) -> Result<VirtualAddress> {
    let mem_frame_info = bitmap::alloc_mem_frame((len / PAGE_SIZE).max(1))?;
    mem_frame_info.set_permissions_to_user()?;
    let virt_addr = mem_frame_info.frame_start_virt_addr()?;
    info!(
        "syscall: sbrk: allocated {} bytes at 0x{:x}",
        mem_frame_info.frame_size,
        virt_addr.get()
    );
    task::push_allocated_mem_frame_info_for_user_task(mem_frame_info)?;
    Ok(virt_addr)
}

fn sys_uname(buf_addr: VirtualAddress) -> Result<()> {
    let sysname = env::OS_NAME.as_bytes();
    let nodename = "nodename".as_bytes();
    let release = "release".as_bytes();
    let version = env::ENV_VERSION.as_bytes();
    let machine = "x86_64".as_bytes();
    let domainname = "domainname".as_bytes();

    let mut utsname = Utsname::default();
    utsname.sysname[..sysname.len()].copy_from_slice(sysname);
    utsname.nodename[..nodename.len()].copy_from_slice(nodename);
    utsname.release[..release.len()].copy_from_slice(release);
    utsname.version[..version.len()].copy_from_slice(version);
    utsname.machine[..machine.len()].copy_from_slice(machine);
    utsname.domainname[..domainname.len()].copy_from_slice(domainname);
    buf_addr.copy_from_nonoverlapping(&utsname as *const Utsname, 1);
    Ok(())
}

fn sys_break() {
    task::debug_user_task();
    super::int3();
}

fn sys_stat(fd: FileDescriptorNumber, buf_addr: VirtualAddress) -> Result<()> {
    let size = match fd {
        FileDescriptorNumber::STDIN
        | FileDescriptorNumber::STDOUT
        | FileDescriptorNumber::STDERR => 0,
        fd => vfs::read_file(&fd)?.len() as u64, // FIXME
    };

    let stat = Stat { size };
    buf_addr.copy_from_nonoverlapping(&stat as *const Stat, 1);
    Ok(())
}

pub fn enable() {
    let mut efer = ExtendedFeatureEnableRegister::read();
    efer.set_syscall_enable(true);
    efer.write();
    assert_eq!(ExtendedFeatureEnableRegister::read().syscall_enable(), true);

    let asm_syscall_handler_addr = asm_syscall_handler as *const () as u64;
    let mut lstar = LongModeSystemCallTargetAddressRegister::read();
    lstar.set_target_addr(asm_syscall_handler_addr);
    lstar.write();
    assert_eq!(
        LongModeSystemCallTargetAddressRegister::read().target_addr(),
        asm_syscall_handler_addr
    );

    let target_addr =
        ((KERNEL_MODE_CS_VALUE as u64) << 32) | ((KERNEL_MODE_SS_VALUE as u64 | 3) << 48);
    let mut star = SystemCallTargetAddressRegister::read();
    star.set_target_addr(target_addr); // set CS and SS to kernel segment
    star.write();
    assert_eq!(
        SystemCallTargetAddressRegister::read().target_addr(),
        target_addr
    );

    let mut fmask = SystemCallFlagMaskRegister::read();
    fmask.set_value(0);
    fmask.write();
    assert_eq!(SystemCallFlagMaskRegister::read().value(), 0);

    info!("syscall: Enabled syscall");
}
