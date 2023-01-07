use core::{mem::size_of, ptr::{read_volatile, write_volatile}};
use log::{error, info};
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use crate::arch::{asm, register::cr2::Cr2};

const IDT_LEN: usize = 256;
const VEC_DIVIDE_ERR: usize = 0;
const VEC_DEBUG: usize = 1;
const VEC_NMI_INT: usize = 2;
const VEC_BREAKPOINT: usize = 3;
const VEC_OVERFLOW: usize = 4;
const VEC_BOUND_RANGE_EXCEEDED: usize = 5;
const VEC_INVALID_OPCODE: usize = 6;
const VEC_DEVICE_NOT_AVAILABLE: usize = 7;
const VEC_DOUBLE_FAULT: usize = 8;
const VEC_INVALID_TSS: usize = 10;
const VEC_SEG_NOT_PRESENT: usize = 11;
const VEC_STACK_SEG_FAULT: usize = 12;
const VEC_GENERAL_PROTECTION: usize = 13;
const VEC_PAGE_FAULT: usize = 14;
const VEC_FLOATING_POINT_ERR: usize = 16;
const VEC_ALIGN_CHECK: usize = 17;
const VEC_MACHINE_CHECK: usize = 18;
const VEC_SIMD_FLOATING_POINT_EX: usize = 19;
const VEC_VIRT_EX: usize = 20;
const VEC_CTRL_PROTECTION_EX: usize = 21;
const VEC_MASKABLE_INT_0: usize = 32;

type Handler = extern "x86-interrupt" fn();

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 4]
pub enum GateType
{
    Interrupt = 0xe,
    Trap = 0xf,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct GateDescriptor
{
    handler_offset_low_low: B16,
    selector: B16,
    ist: B3,
    #[skip]
    reserved: B5,
    #[bits = 4]
    gate_type: GateType,
    #[skip(getters)]
    zero: B1,
    dpl: B2,
    p: bool,
    handler_offset_low_hi: B16,
    handler_offset_hi: B32,
    #[skip]
    reserved: B32,
}

impl GateDescriptor
{
    pub fn set_handler(&mut self, handler: Handler, cs: u16)
    {
        // TODO: virt addr
        let handler_addr = handler as *const () as u64;
        self.set_handler_offset_low_low(handler_addr as u16);
        self.set_handler_offset_low_hi((handler_addr >> 16) as u16);
        self.set_handler_offset_hi((handler_addr >> 32) as u32);
        self.set_selector(cs);
    }
}

fn read_desc(vec_num: usize) -> Option<GateDescriptor>
{
    if vec_num >= IDT_LEN
    {
        return None;
    }

    let ptr = (asm::sidt().base + (size_of::<GateDescriptor>() * vec_num) as u64)
        as *const GateDescriptor;
    return Some(unsafe { read_volatile(ptr) });
}

fn write_desc(vec_num: usize, desc: GateDescriptor)
{
    if vec_num >= IDT_LEN
    {
        return;
    }

    let ptr =
        (asm::sidt().base + (size_of::<GateDescriptor>() * vec_num) as u64) as *mut GateDescriptor;
    unsafe { write_volatile(ptr, desc) };
}

fn set_handler(vec_num: usize, handler: Handler)
{
    if let Some(mut desc) = read_desc(vec_num)
    {
        desc.set_handler(handler, asm::read_cs());
        write_desc(vec_num, desc);
    }
    else
    {
        error!("Failed to set IDT handler");
    }
}

extern "x86-interrupt" fn breakpint_handler()
{
    panic!("Exception: BREAKPOINT");
}

extern "x86-interrupt" fn page_fault_handler()
{
    panic!("Exception: PAGE FAULT, Accessed virtual address: 0x{:x}", Cr2::read().get());
}

extern "x86-interrupt" fn double_fault_handler()
{
    panic!("Exception: DOUBLE FAULT");
}

pub fn init()
{
    // TODO: support IDT updates via LIDT (use IDT struct)
    set_handler(VEC_BREAKPOINT, breakpint_handler);
    set_handler(VEC_PAGE_FAULT, page_fault_handler);
    set_handler(VEC_DOUBLE_FAULT, double_fault_handler);
    info!("Initialized IDT");
}
