use core::mem::size_of;
use log::info;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};

use crate::arch::{asm::{self, DescriptorTableArgs}, register::control::Cr2};

use super::addr::VirtualAddress;

static mut IDT: [GateDescriptor; IDT_LEN] = [GateDescriptor::new(); IDT_LEN];

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
pub const VEC_XHCI_INT: usize = 64;

const END_OF_INT_REG_ADDR: u64 = 0xfee000b0;

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
    handler_offset_low: B16,
    selector: B16,
    // attributes
    int_stack_table: B3,
    #[skip]
    reserved0: B5,
    #[bits = 4]
    gate_type: GateType,
    #[skip]
    reserved1: B1,
    desc_privilege_level: B2,
    present: bool,

    handler_offset_middle: B16,
    handler_offset_high: B32,
    #[skip]
    reserved2: B32,
}

impl GateDescriptor
{
    pub fn set_handler(&mut self, handler: Handler, cs: u16, gate_type: GateType)
    {
        // TODO: virt addr
        let handler_addr = handler as *const () as u64;
        self.set_handler_offset_low(handler_addr as u16);
        self.set_handler_offset_middle((handler_addr >> 16) as u16);
        self.set_handler_offset_high((handler_addr >> 32) as u32);
        self.set_selector(cs);
        self.set_gate_type(gate_type);
        self.set_present(true);
    }
}

fn set_handler(vec_num: usize, handler: Handler, gate_type: GateType)
{
    let mut desc = GateDescriptor::new();
    desc.set_handler(handler, asm::read_cs(), gate_type);
    unsafe {
        IDT[vec_num] = desc;
    }
}

fn load_idt()
{
    let limit = (size_of::<[GateDescriptor; IDT_LEN]>() - 1) as u16;
    let base = &unsafe { IDT } as *const _ as u64;
    let args = DescriptorTableArgs { limit, base };
    asm::lidt(&args);
}

fn notify_end_of_int()
{
    let virt_addr = VirtualAddress::new(END_OF_INT_REG_ADDR);
    virt_addr.write_volatile(0);
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

extern "x86-interrupt" fn xhc_primary_event_ring_handler()
{
    info!("Interrupt: XHC PRIMARY EVENT RING");
    notify_end_of_int();
}

pub fn init()
{
    set_handler(VEC_BREAKPOINT, breakpint_handler, GateType::Interrupt);
    set_handler(VEC_PAGE_FAULT, page_fault_handler, GateType::Interrupt);
    set_handler(VEC_DOUBLE_FAULT, double_fault_handler, GateType::Interrupt);
    set_handler(VEC_XHCI_INT, xhc_primary_event_ring_handler, GateType::Interrupt);

    load_idt();
    info!("idt: Initialized IDT");
}
