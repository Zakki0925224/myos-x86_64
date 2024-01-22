use crate::{
    arch::addr::*,
    error::Result,
    mem::bitmap::{self, MemoryFrameInfo},
    println,
};
use core::mem::size_of;

use super::{register::*, trb::*};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RingBufferError {
    NotInitialized,
    InvalidMemoryError {
        mem_frame_info: MemoryFrameInfo,
        buf_len: usize,
    },
    UnsupportedRingBufferTypeError(RingBufferType),
    InvalidRingBufferIndexError(usize),
    UnsupportedEventRingSegmentTableLengthError,
    InvalidTransferRequestBlockError(usize),
    RingBufferSizeIsTooSmallError(usize),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RingBufferType {
    TransferRing,
    EventRing,
    CommandRing,
}

#[derive(Debug, Clone, Copy)]
pub struct RingBuffer {
    is_init: bool,
    buf_base_virt_addr: VirtualAddress,
    buf_len: usize,
    buf_type: RingBufferType,
    cycle_state: bool,
    enqueue_index: usize,
}

impl RingBuffer {
    pub fn new(
        buf_base_mem_info: MemoryFrameInfo,
        buf_len: usize,
        buf_type: RingBufferType,
        cycle_state_bit: bool,
    ) -> Result<Self> {
        if !buf_base_mem_info.is_allocated()
            || (buf_base_mem_info.get_frame_size() / size_of::<TransferRequestBlock>()) < buf_len
        {
            return Err(RingBufferError::InvalidMemoryError {
                mem_frame_info: buf_base_mem_info,
                buf_len,
            }
            .into());
        }

        if buf_len < 2 {
            return Err(RingBufferError::RingBufferSizeIsTooSmallError(buf_len).into());
        }

        Ok(Self {
            buf_base_virt_addr: buf_base_mem_info.get_frame_start_virt_addr(),
            buf_len,
            buf_type,
            cycle_state: cycle_state_bit,
            is_init: false,
            enqueue_index: 0,
        })
    }

    pub fn init(&mut self) {
        self.is_init = true;

        if self.buf_type == RingBufferType::EventRing {
            return;
        }

        // set Link TRB
        let mut trb = TransferRequestBlock::new();
        trb.set_trb_type(TransferRequestBlockType::Link);
        trb.set_param(self.buf_base_virt_addr.get_phys_addr().unwrap().get());

        self.write(self.buf_len - 1, trb).unwrap();
    }

    pub fn is_init(&self) -> bool {
        self.is_init
    }

    pub fn buf_len(&self) -> usize {
        self.buf_len
    }

    pub fn enqueue_index(&self) -> usize {
        self.enqueue_index
    }

    pub fn cycle_state(&self) -> bool {
        self.cycle_state
    }

    pub fn enqueue(&mut self) -> Result<()> {
        if !self.is_init {
            return Err(RingBufferError::NotInitialized.into());
        }

        if self.buf_type != RingBufferType::TransferRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        if self.enqueue_index == self.buf_len - 1 {
            match self.toggle_cycle() {
                Err(err) => return Err(err),
                _ => (),
            }

            self.enqueue_index = 0;
        }

        let mut trb = match self.read(self.enqueue_index) {
            Some(trb) => trb,
            None => {
                return Err(RingBufferError::InvalidRingBufferIndexError(self.enqueue_index).into())
            }
        };

        trb.set_cycle_bit(!trb.cycle_bit());

        match self.write(self.enqueue_index, trb) {
            Err(err) => return Err(err),
            _ => (),
        }

        self.enqueue_index += 1;

        Ok(())
    }

    pub fn push(&mut self, trb: TransferRequestBlock) -> Result<()> {
        if !self.is_init {
            return Err(RingBufferError::NotInitialized.into());
        }

        if self.buf_type == RingBufferType::EventRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        if self.enqueue_index == self.buf_len - 1 {
            match self.toggle_cycle() {
                Err(err) => return Err(err),
                _ => (),
            }

            self.enqueue_index = 0;
        }

        let mut trb = trb;
        trb.set_cycle_bit(self.cycle_state);

        match self.write(self.enqueue_index, trb) {
            Err(err) => return Err(err),
            _ => (),
        }

        self.enqueue_index += 1;

        Ok(())
    }

    pub fn pop(
        &mut self,
        mut int_reg_set: InterrupterRegisterSet,
    ) -> Result<(TransferRequestBlock, InterrupterRegisterSet)> {
        if !self.is_init {
            return Err(RingBufferError::NotInitialized.into());
        }

        if self.buf_type != RingBufferType::EventRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        if int_reg_set.event_ring_seg_table_size() != 1 || int_reg_set.dequeue_erst_seg_index() != 0
        {
            return Err(RingBufferError::UnsupportedEventRingSegmentTableLengthError.into());
        }

        let trb_size = size_of::<TransferRequestBlock>();
        let mut dequeue_ptr =
            PhysicalAddress::new(int_reg_set.event_ring_dequeue_ptr() << 4).get_virt_addr();

        let mut index = (dequeue_ptr.get() - self.buf_base_virt_addr.get()) as usize / trb_size;

        let trb = match self.read(index) {
            Some(trb) => trb,
            None => return Err(RingBufferError::InvalidRingBufferIndexError(index).into()),
        };

        if trb.cycle_bit() != self.cycle_state {
            return Err(RingBufferError::InvalidTransferRequestBlockError(index).into());
        }

        index += 1;

        if index == self.buf_len {
            index = 0;
            self.cycle_state = !self.cycle_state;
        }

        dequeue_ptr = self.buf_base_virt_addr.offset(index * trb_size);
        int_reg_set.set_event_ring_dequeue_ptr(dequeue_ptr.get_phys_addr().unwrap().get() >> 4);
        int_reg_set.set_event_handler_busy(false);

        Ok((trb, int_reg_set))
    }

    pub fn fill(&mut self, fill_trb: TransferRequestBlock) -> Result<()> {
        if !self.is_init {
            return Err(RingBufferError::NotInitialized.into());
        }

        if self.buf_type != RingBufferType::TransferRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        if self.buf_len < 3 {
            return Err(RingBufferError::RingBufferSizeIsTooSmallError(self.buf_len).into());
        }

        for i in 0..self.buf_len - 1 {
            let data_buf_phys_addr = bitmap::alloc_mem_frame(1)?.get_frame_start_phys_addr();

            let mut trb = fill_trb;
            trb.set_param(data_buf_phys_addr.get());
            trb.set_cycle_bit(if i < self.buf_len - 3 {
                self.cycle_state
            } else {
                !self.cycle_state
            });

            if let Err(err) = self.write(i, trb) {
                return Err(err);
            }
        }

        self.enqueue_index = self.buf_len - 3;

        Ok(())
    }

    pub fn debug(&self) {
        println!(
            "{:?}:, current: {}, start: 0x{:x}",
            self.buf_type,
            self.enqueue_index,
            self.buf_base_virt_addr.get()
        );
        for i in 0..self.buf_len {
            let trb = self.read(i).unwrap();
            println!(
                "{}: param: 0x{:x} cb: {:?}",
                i,
                trb.param(),
                trb.cycle_bit()
            );
        }
    }

    fn toggle_cycle(&mut self) -> Result<()> {
        if !self.is_init {
            return Err(RingBufferError::NotInitialized.into());
        }

        if self.buf_type == RingBufferType::EventRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        let link_trb_index = self.buf_len - 1;
        let mut link_trb = match self.read(link_trb_index) {
            Some(trb) => trb,
            None => return Err(RingBufferError::InvalidRingBufferIndexError(link_trb_index).into()),
        };
        link_trb.set_cycle_bit(!link_trb.cycle_bit());
        // true -> toggle, false -> reset
        link_trb.set_toggle_cycle(self.cycle_state);
        match self.write(link_trb_index, link_trb) {
            Err(err) => return Err(err),
            _ => (),
        };

        self.cycle_state = !self.cycle_state;

        Ok(())
    }

    fn read(&self, index: usize) -> Option<TransferRequestBlock> {
        if index >= self.buf_len {
            return None;
        }

        let virt_addr = self
            .buf_base_virt_addr
            .offset(index * size_of::<TransferRequestBlock>());
        Some(virt_addr.read_volatile())
    }

    fn write(&self, index: usize, trb: TransferRequestBlock) -> Result<()> {
        if index >= self.buf_len {
            return Err(RingBufferError::InvalidRingBufferIndexError(index).into());
        }

        let virt_addr = self
            .buf_base_virt_addr
            .offset(index * size_of::<TransferRequestBlock>());
        virt_addr.write_volatile(trb);

        Ok(())
    }
}
