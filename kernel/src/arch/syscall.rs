use super::{addr::VirtualAddress, asm};
use crate::{
    arch::{
        gdt::{KERNEL_MODE_CS_VALUE, KERNEL_MODE_SS_VALUE},
        register::model_specific::*,
        task,
    },
    env,
    error::{Error, Result},
    fs::vfs::file_desc::FileDescriptorNumber,
    mem::{bitmap, paging::PAGE_SIZE},
    print,
};
use alloc::string::{String, ToString};
use common::libm::Utsname;
use core::{arch::asm, slice};
use log::{error, info};

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
    let args = [arg0, arg1, arg2, arg3, arg4, arg5];
    info!("syscall: Called!(args: {:?})", args);

    match arg0 {
        // write syscall
        1 => {
            let fd = FileDescriptorNumber::new_val(arg1);
            let s_ptr = arg2 as *const u8;
            let s_len = arg3 as usize;
            if let Err(err) = sys_write(fd, s_ptr, s_len) {
                error!("syscall: write: {:?}", err);
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
        num => {
            error!("syscall: Syscall number 0x{:x} is not defined", num);
            return -1;
        }
    }

    0
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
    asm::int3();
}

pub fn init() {
    let mut efer = ExtendedFeatureEnableRegister::read();
    efer.set_system_call_enable(true);
    efer.write();

    let mut lstar = LongModeSystemCallTargetAddressRegister::read();
    lstar.set_target_addr(asm_syscall_handler as *const () as u64);
    lstar.write();

    let mut star = SystemCallTargetAddressRegister::read();
    star.set_target_addr(
        ((KERNEL_MODE_CS_VALUE as u64) << 32) | ((KERNEL_MODE_SS_VALUE as u64 | 3) << 48),
    ); // set CS and SS to kernel segment
    star.write();

    let mut fmask = SystemCallFlagMaskRegister::read();
    fmask.set_value(0);
    fmask.write();

    info!("syscall: Initialized syscall");
}
