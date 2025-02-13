use super::{addr::*, task};
use crate::{
    arch::{
        self,
        register::{control::Cr2, segment::Cs, Register},
    },
    device,
    error::{Error, Result},
    mem::paging,
    util::mutex::Mutex,
};
use alloc::string::String;
use core::{mem::size_of, panic};
use log::*;

static mut IDT: Mutex<InterruptDescriptorTable> = Mutex::new(InterruptDescriptorTable::new());

// https://github.com/rust-osdev/x86_64/blob/master/src/structures/idt.rs
#[repr(transparent)]
pub struct PageFaultErrorCode(u64);

impl PageFaultErrorCode {
    pub const PROTECTION_VIOLATION: u64 = 1;
    pub const CAUSED_BY_WRITE: u64 = 1 << 1;
    pub const USER_MODE: u64 = 1 << 2;
    pub const MALFORMED_TABLE: u64 = 1 << 3;
    pub const INSTRUCTION_FETCH: u64 = 1 << 4;
    pub const PROTECTION_KEY: u64 = 1 << 5;
    pub const SHADOW_STACK: u64 = 1 << 6;
    pub const SGX: u64 = 1 << 15;
    pub const RMP: u64 = 1 << 31;
}

impl core::fmt::Debug for PageFaultErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut fmt = String::from("PageFaultErrorCode");

        fmt = format!("{}(0x{:x}) {{ ", fmt, self.0);

        if (self.0 & Self::PROTECTION_VIOLATION) != 0 {
            fmt = format!("{}PROTECTION_VIOLATION, ", fmt);
        }

        if (self.0 & Self::CAUSED_BY_WRITE) != 0 {
            fmt = format!("{}CAUSED_BY_WRITE, ", fmt);
        }

        if (self.0 & Self::USER_MODE) != 0 {
            fmt = format!("{}USER_MODE, ", fmt);
        }

        if (self.0 & Self::MALFORMED_TABLE) != 0 {
            fmt = format!("{}MALFORMED_TABLE, ", fmt);
        }

        if (self.0 & Self::INSTRUCTION_FETCH) != 0 {
            fmt = format!("{}INSTRUCTION_FETCH, ", fmt);
        }

        if (self.0 & Self::PROTECTION_KEY) != 0 {
            fmt = format!("{}PROTECTION_KEY, ", fmt);
        }

        if (self.0 & Self::SHADOW_STACK) != 0 {
            fmt = format!("{}SHADOW_STACK, ", fmt);
        }

        if (self.0 & Self::SGX) != 0 {
            fmt = format!("{}SGX, ", fmt);
        }

        if (self.0 & Self::RMP) != 0 {
            fmt = format!("{}RMP, ", fmt);
        }

        fmt = format!("{}}}", fmt);

        write!(f, "{}", fmt)
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    pub ins_ptr: u64,
    pub code_seg: u16,
    reserved0: [u8; 6],
    pub cpu_flags: u64,
    pub stack_ptr: u64,
    pub stack_seg: u16,
    reserved1: [u8; 6],
}

// idt
const IDT_LEN: usize = 256;
const _VEC_DIVIDE_ERR: usize = 0x00;
const _VEC_DEBUG: usize = 0x01;
const _VEC_NMI_INT: usize = 0x02;
const VEC_BREAKPOINT: usize = 0x03;
const _VEC_OVERFLOW: usize = 0x04;
const _VEC_BOUND_RANGE_EXCEEDED: usize = 0x05;
const _VEC_INVALID_OPCODE: usize = 0x06;
const _VEC_DEVICE_NOT_AVAILABLE: usize = 0x07;
const VEC_DOUBLE_FAULT: usize = 0x08;
const _VEC_INVALID_TSS: usize = 0x0a;
const _VEC_SEG_NOT_PRESENT: usize = 0x0b;
const _VEC_STACK_SEG_FAULT: usize = 0x0c;
const VEC_GENERAL_PROTECTION: usize = 0x0d;
const VEC_PAGE_FAULT: usize = 0x0e;
const _VEC_FLOATING_POINT_ERR: usize = 0x10;
const _VEC_ALIGN_CHECK: usize = 0x11;
const _VEC_MACHINE_CHECK: usize = 0x12;
const _VEC_SIMD_FLOATING_POINT_EX: usize = 0x13;
const _VEC_VIRT_EX: usize = 0x14;
const _VEC_CTRL_PROTECTION_EX: usize = 0x15;

pub const VEC_PS2_KBD: usize = 0x21; // ps/2 keyboard
pub const VEC_PS2_MOUSE: usize = 0x2c; // ps/2 mouse

const MASTER_PIC_ADDR: IoPortAddress = IoPortAddress::new(0x20);
const SLAVE_PIC_ADDR: IoPortAddress = IoPortAddress::new(0xa0);
const PIC_END_OF_INT_CMD: u8 = 0x20;

pub enum InterruptHandler {
    Normal(extern "x86-interrupt" fn()),
    WithStackFrame(extern "x86-interrupt" fn(InterruptStackFrame)),
    PageFault(extern "x86-interrupt" fn(InterruptStackFrame, PageFaultErrorCode)),
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum GateType {
    Interrupt = 0xe,
    Trap = 0xf,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, align(16))]
pub struct GateDescriptor(u128);

impl GateDescriptor {
    const fn new() -> Self {
        Self(0)
    }

    fn set_handler(&mut self, handler: InterruptHandler, gate_type: GateType) {
        let handler_addr = match handler {
            InterruptHandler::Normal(handler) => handler as *const (),
            InterruptHandler::WithStackFrame(handler) => handler as *const (),
            InterruptHandler::PageFault(handler) => handler as *const (),
        } as u64;
        self.set_handler_offset(handler_addr);
        self.set_selector(Cs::read().raw());
        self.set_gate_type(gate_type);
        self.set_p(true);
    }

    fn set_handler_offset(&mut self, offset: u64) {
        let offset_low = offset as u16;
        let offset_middle = (offset >> 16) as u16;
        let offset_high = (offset >> 32) as u16;

        self.0 = (self.0 & !0xffff) | (offset_low as u128);
        self.0 = (self.0 & !0xffff_0000_0000_0000) | ((offset_middle as u128) << 48);
        self.0 = (self.0 & !0xffff_ffff_0000_0000_0000_0000) | ((offset_high as u128) << 64);
    }

    fn set_selector(&mut self, selector: u16) {
        self.0 = (self.0 & !0xffff_0000) | ((selector as u128) << 16);
    }

    fn set_gate_type(&mut self, gate_type: GateType) {
        let gate_type = gate_type as u8 & 0xf; // 4 bits
        self.0 = (self.0 & !0x0f00_0000_0000) | ((gate_type as u128) << 40);
    }

    fn set_p(&mut self, value: bool) {
        self.0 = (self.0 & !0x8000_0000_0000) | ((value as u128) << 47);
    }

    fn is_null(self) -> bool {
        self.0 == 0
    }
}

struct InterruptDescriptorTable {
    entries: [GateDescriptor; IDT_LEN],
}

impl InterruptDescriptorTable {
    const fn new() -> Self {
        Self {
            entries: [GateDescriptor::new(); IDT_LEN],
        }
    }

    fn set_handler(
        &mut self,
        vec_num: usize,
        handler: InterruptHandler,
        gate_type: GateType,
    ) -> Result<()> {
        if vec_num >= IDT_LEN {
            return Err(Error::Failed("Invalid interrupt vector number"));
        }

        let desc = &mut self.entries[vec_num];
        if !desc.is_null() {
            return Err(Error::Failed("Interrupt handler already set"));
        }
        desc.set_handler(handler, gate_type);

        Ok(())
    }

    fn set_handler_dyn_vec(
        &mut self,
        handler: InterruptHandler,
        gate_type: GateType,
    ) -> Result<u8> {
        for i in 32..IDT_LEN {
            let desc = &mut self.entries[i];
            if desc.is_null() {
                desc.set_handler(handler, gate_type);
                return Ok(i as u8);
            }
        }

        Err(Error::Failed("No available interrupt vector"))
    }

    fn load(&self) {
        let limit = (size_of::<GateDescriptor>() * IDT_LEN - 1) as u16;
        let base = self.entries.as_ptr() as u64;
        let args = arch::DescriptorTableArgs { limit, base };
        arch::disabled_int(|| arch::lidt(&args));
    }
}

pub fn notify_end_of_int() {
    MASTER_PIC_ADDR.out8(PIC_END_OF_INT_CMD);
    SLAVE_PIC_ADDR.out8(PIC_END_OF_INT_CMD);
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    panic!("int: BREAKPOINT, {:?}", stack_frame);
}

extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: InterruptStackFrame) {
    error!("int: GENERAL PROTECTION FAULT, {:?}", stack_frame);

    if task::is_running_user_task() {
        task::debug_user_task();
        task::return_task(122);
        unreachable!();
    }

    panic!();
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let accessed_virt_addr = Cr2::read().raw();
    let page_virt_addr = (accessed_virt_addr & !0xfff).into();
    let page_table_entry = paging::read_page_table_entry(page_virt_addr);

    error!(
        "int: PAGE FAULT, Accessed virtual address: 0x{:x}, {:?}, {:?}, Page table entry (at 0x{:x}): {:?}",
        accessed_virt_addr, error_code, stack_frame, page_virt_addr.get(), page_table_entry
    );

    if task::is_running_user_task() {
        task::debug_user_task();
        task::return_task(123);
        unreachable!();
    }

    panic!();
}

extern "x86-interrupt" fn double_fault_handler() {
    panic!("int: DOUBLE FAULT");
}

pub fn init_pic() {
    // disallow all interrupts
    MASTER_PIC_ADDR.offset(1).out8(0xff);
    SLAVE_PIC_ADDR.offset(1).out8(0xff);

    // mapping IRQ0 - 7 to IDT entries 0x20 - 0x27
    MASTER_PIC_ADDR.offset(0).out8(0x11);
    MASTER_PIC_ADDR.offset(1).out8(0x20);
    MASTER_PIC_ADDR.offset(1).out8(1 << 2);
    MASTER_PIC_ADDR.offset(1).out8(0x1); // none buffer mode

    // mapping IRQ8 - 15 to IDT entries 0x28 - 0x2f
    SLAVE_PIC_ADDR.offset(0).out8(0x11); // edge trigger mode
    SLAVE_PIC_ADDR.offset(1).out8(0x28);
    SLAVE_PIC_ADDR.offset(1).out8(2);
    SLAVE_PIC_ADDR.offset(1).out8(0x1); // none buffer mode

    // mask all
    MASTER_PIC_ADDR.offset(1).out8(0xfb);
    SLAVE_PIC_ADDR.offset(1).out8(0xff);

    // allow interrupts
    MASTER_PIC_ADDR.offset(1).out8(0xf9);
    SLAVE_PIC_ADDR.offset(1).out8(0xef);

    info!("idt: PIC initialized");
}

pub fn init_idt() -> Result<()> {
    let mut idt = unsafe { IDT.try_lock() }?;
    idt.set_handler(
        VEC_BREAKPOINT,
        InterruptHandler::WithStackFrame(breakpoint_handler),
        GateType::Trap,
    )?;
    idt.set_handler(
        VEC_GENERAL_PROTECTION,
        InterruptHandler::WithStackFrame(general_protection_fault_handler),
        GateType::Interrupt,
    )?;
    idt.set_handler(
        VEC_PAGE_FAULT,
        InterruptHandler::PageFault(page_fault_handler),
        GateType::Interrupt,
    )?;
    idt.set_handler(
        VEC_DOUBLE_FAULT,
        InterruptHandler::Normal(double_fault_handler),
        GateType::Interrupt,
    )?;
    idt.set_handler(
        VEC_PS2_KBD,
        InterruptHandler::Normal(device::ps2_keyboard::poll_int_ps2_kbd_driver),
        GateType::Interrupt,
    )?;
    idt.set_handler(
        VEC_PS2_MOUSE,
        InterruptHandler::Normal(device::ps2_mouse::poll_int_ps2_mouse_driver),
        GateType::Interrupt,
    )?;
    idt.load();

    info!("idt: Initialized");
    Ok(())
}

pub fn set_handler(vec_num: usize, handler: InterruptHandler, gate_type: GateType) -> Result<()> {
    let mut idt = unsafe { IDT.try_lock() }?;
    idt.set_handler(vec_num, handler, gate_type)?;
    idt.load();
    Ok(())
}

pub fn set_handler_dyn_vec(handler: InterruptHandler, gate_type: GateType) -> Result<u8> {
    let mut idt = unsafe { IDT.try_lock() }?;
    let vec_num = idt.set_handler_dyn_vec(handler, gate_type)?;
    idt.load();
    Ok(vec_num)
}
