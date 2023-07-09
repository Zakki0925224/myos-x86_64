use core::mem::size_of;
use lazy_static::lazy_static;
use log::info;
use modular_bitfield::{bitfield, specifiers::*, BitfieldSpecifier};
use spin::Mutex;

use crate::{
    arch::{
        asm::{self, DescriptorTableArgs},
        register::control::Cr2,
    },
    device::usb::xhc::XHC_DRIVER,
};

use self::info::{InterruptStackFrame, PageFaultErrorCode};

use super::addr::*;

pub mod info;

lazy_static! {
    static ref IDT: Mutex<InterruptDescriptorTable> = Mutex::new(InterruptDescriptorTable::new());
}

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

type NormalHandler = extern "x86-interrupt" fn();
type PageFaultHandler = extern "x86-interrupt" fn(InterruptStackFrame, PageFaultErrorCode);

enum InterruptHandler {
    Normal(NormalHandler),
    PageFault(PageFaultHandler),
}

#[derive(BitfieldSpecifier, Debug, Clone, Copy)]
#[bits = 4]
pub enum GateType {
    Interrupt = 0xe,
    Trap = 0xf,
}

#[bitfield]
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct GateDescriptor {
    handler_offset_low: B16,
    selector: B16,
    // attributes
    int_stack_table: B3,
    #[skip]
    reserved0: B5,
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

impl GateDescriptor {
    pub fn set_handler(&mut self, handler: InterruptHandler, cs: u16, gate_type: GateType) {
        let handler_addr = match handler {
            InterruptHandler::Normal(handler) => handler as *const () as u64,
            InterruptHandler::PageFault(handler) => handler as *const () as u64,
        };
        self.set_handler_offset_low(handler_addr as u16);
        self.set_handler_offset_middle((handler_addr >> 16) as u16);
        self.set_handler_offset_high((handler_addr >> 32) as u32);
        self.set_selector(cs);
        self.set_gate_type(gate_type);
        self.set_present(true);
    }
}

#[repr(C, align(16))]
struct InterruptDescriptorTable {
    entries: [GateDescriptor; IDT_LEN],
}

impl InterruptDescriptorTable {
    pub fn new() -> Self {
        return Self {
            entries: [GateDescriptor::new(); IDT_LEN],
        };
    }

    pub fn set_handler(&mut self, vec_num: usize, handler: InterruptHandler, gate_type: GateType) {
        if vec_num >= IDT_LEN {
            return;
        }

        let mut desc = GateDescriptor::new();
        desc.set_handler(handler, asm::read_cs(), gate_type);
        self.entries[vec_num] = desc;
    }

    pub fn load(&self) {
        let limit = (size_of::<[GateDescriptor; IDT_LEN]>() - 1) as u16;
        let base = self.entries.as_ptr() as u64;
        let args = DescriptorTableArgs { limit, base };
        asm::lidt(&args);

        //info!("idt: Loaded IDT: {:?}", args);
        asm::sti();
    }
}

fn notify_end_of_int() {
    let virt_addr = VirtualAddress::new(END_OF_INT_REG_ADDR);
    virt_addr.write_volatile(0);
}

extern "x86-interrupt" fn breakpoint_handler() {
    panic!("int: BREAKPOINT");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    panic!(
        "int: PAGE FAULT, Accessed virtual address: 0x{:x}, Error code: {:?}, Stack frame: {:?}",
        Cr2::read().get(),
        error_code,
        stack_frame
    );
}

extern "x86-interrupt" fn double_fault_handler() {
    panic!("int: DOUBLE FAULT");
}

extern "x86-interrupt" fn xhc_primary_event_ring_handler() {
    //info!("int: XHC PRIMARY EVENT RING");
    if XHC_DRIVER.is_locked() {
        panic!("int: XHC DRIVER is locked");
    }

    XHC_DRIVER.lock().as_mut().unwrap().on_updated_event_ring();
    notify_end_of_int();
}

pub fn init() {
    IDT.lock().set_handler(
        VEC_BREAKPOINT,
        InterruptHandler::Normal(breakpoint_handler),
        GateType::Interrupt,
    );
    IDT.lock().set_handler(
        VEC_PAGE_FAULT,
        InterruptHandler::PageFault(page_fault_handler),
        GateType::Interrupt,
    );
    IDT.lock().set_handler(
        VEC_DOUBLE_FAULT,
        InterruptHandler::Normal(double_fault_handler),
        GateType::Interrupt,
    );
    IDT.lock().set_handler(
        VEC_XHCI_INT,
        InterruptHandler::Normal(xhc_primary_event_ring_handler),
        GateType::Interrupt,
    );
    IDT.lock().load();
    info!("idt: Initialized IDT");
}
