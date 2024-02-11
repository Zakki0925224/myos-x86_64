use super::{register::*, trb::*};
use crate::{arch::addr::*, error::Result, mem::bitmap, println};
use alloc::boxed::Box;
use core::mem::size_of;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RingBufferError {
    UnsupportedRingBufferTypeError(RingBufferType),
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

#[derive(Debug, Clone)]
#[repr(C, align(4096))]
struct RingBufferInner<const N: usize>(pub [TransferRequestBlock; N]);

#[derive(Debug, Clone)]
pub struct RingBuffer<const N: usize> {
    buf: Box<RingBufferInner<N>>,
    buf_type: RingBufferType,
    cycle_state: bool,
    enqueue_index: usize,
}

impl<const N: usize> RingBuffer<N> {
    pub fn new(buf_type: RingBufferType, cycle_state_bit: bool) -> Result<Self> {
        if N < 2 {
            return Err(RingBufferError::RingBufferSizeIsTooSmallError(N).into());
        }

        Ok(Self {
            buf: Box::new(RingBufferInner([TransferRequestBlock::default(); N])),
            buf_type,
            cycle_state: cycle_state_bit,
            enqueue_index: 0,
        })
    }

    pub fn buf_ptr(&self) -> *const TransferRequestBlock {
        self.buf.as_ref().0.as_ptr()
    }

    pub fn set_link_trb(&mut self) -> Result<()> {
        if self.buf_type == RingBufferType::EventRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        let mut trb = TransferRequestBlock::default();
        trb.set_trb_type(TransferRequestBlockType::Link);
        trb.param = self.buf_ptr() as u64;

        let buf_len = self.buf_len();
        self.buf_mut()[buf_len - 1] = trb;

        Ok(())
    }

    pub fn buf_len(&self) -> usize {
        self.buf.as_ref().0.len()
    }

    pub fn enqueue(&mut self) -> Result<()> {
        if self.buf_type != RingBufferType::TransferRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        if self.enqueue_index == self.buf_len() - 1 {
            self.toggle_cycle()?;
            self.enqueue_index = 0;
        }

        let enqueue_index = self.enqueue_index;
        let trb = &mut self.buf_mut()[enqueue_index];
        trb.set_cycle_bit(!trb.cycle_bit());

        self.enqueue_index += 1;

        Ok(())
    }

    pub fn push(&mut self, trb: TransferRequestBlock) -> Result<()> {
        if self.buf_type == RingBufferType::EventRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        if self.enqueue_index == self.buf_len() - 1 {
            self.toggle_cycle()?;
            self.enqueue_index = 0;
        }

        let mut trb = trb;
        trb.set_cycle_bit(self.cycle_state);

        let enqueue_index = self.enqueue_index;
        self.buf_mut()[enqueue_index] = trb;
        self.enqueue_index += 1;

        Ok(())
    }

    pub fn pop(
        &mut self,
        int_reg_set: &mut InterrupterRegisterSet,
    ) -> Result<TransferRequestBlock> {
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
        let mut index = (dequeue_ptr.get() as usize - self.buf_ptr() as usize) / trb_size;
        let trb = self.buf_mut()[index];

        if trb.cycle_bit() != self.cycle_state {
            return Err(RingBufferError::InvalidTransferRequestBlockError(index).into());
        }

        index += 1;

        if index == self.buf_len() {
            index = 0;
            self.cycle_state = !self.cycle_state;
        }

        //println!("{:p}, index: {}", self.buf_ptr(), index);
        dequeue_ptr = VirtualAddress::new(self.buf_ptr() as u64).offset(index * trb_size);
        int_reg_set.set_event_ring_dequeue_ptr(dequeue_ptr.get_phys_addr().unwrap().get() >> 4);
        int_reg_set.set_event_handler_busy(false);

        Ok(trb)
    }

    pub fn fill_and_alloc_buf(&mut self, mut fill_trb: TransferRequestBlock) -> Result<()> {
        if self.buf_type != RingBufferType::TransferRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        if self.buf_len() < 3 {
            return Err(RingBufferError::RingBufferSizeIsTooSmallError(self.buf_len()).into());
        }

        for i in 0..self.buf_len() - 1 {
            let data_buf_mem_frame_info = bitmap::alloc_mem_frame(1)?;
            fill_trb.param = data_buf_mem_frame_info.get_frame_start_phys_addr().get();
            fill_trb.set_cycle_bit(if i < self.buf_len() - 3 {
                self.cycle_state
            } else {
                !self.cycle_state
            });

            self.buf_mut()[i] = fill_trb;
        }

        self.enqueue_index = self.buf_len() - 3;

        Ok(())
    }

    pub fn debug(&mut self) {
        println!(
            "{:?}:, current: {}, start: 0x{:x}",
            self.buf_type,
            self.enqueue_index,
            self.buf_ptr() as u64
        );
        for i in 0..self.buf_len() {
            let trb = self.buf_mut()[i];
            println!("{}: param: 0x{:x} cb: {:?}", i, trb.param, trb.cycle_bit());
        }
    }

    fn buf_mut(&mut self) -> &mut [TransferRequestBlock] {
        self.buf.as_mut().0.as_mut()
    }

    fn toggle_cycle(&mut self) -> Result<()> {
        if self.buf_type == RingBufferType::EventRing {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type).into());
        }

        let link_trb_index = self.buf_len() - 1;
        let cycle_state = self.cycle_state;
        let link_trb = &mut self.buf_mut()[link_trb_index];
        link_trb.set_cycle_bit(!link_trb.cycle_bit());
        // true -> toggle, false -> reset
        link_trb.set_toggle_cycle(cycle_state);

        self.cycle_state = !self.cycle_state;

        Ok(())
    }
}
