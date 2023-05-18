use crate::{arch::addr::*, mem::bitmap::MemoryFrameInfo, println};
use core::mem::size_of;

use super::{register::*, trb::*};

#[derive(Debug)]
pub enum RingBufferError
{
    NotInitialized,
    InvalidMemoryFrameInfoError(MemoryFrameInfo),
    UnsupportedRingBufferTypeError(RingBufferType),
    InvalidRingBufferIndexError(usize),
    UnsupportedEventRingSegmentTableLengthError,
    InvalidTransferRequestBlockError(usize),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RingBufferType
{
    TransferRing,
    EventRing,
    CommandRing,
}

#[derive(Debug)]
pub struct RingBuffer
{
    is_init: bool,
    buf_base_virt_addr: VirtualAddress,
    buf_len: usize,
    buf_type: RingBufferType,
    cycle_state: bool,
}

impl RingBuffer
{
    pub fn new(
        buf_base_mem_info: MemoryFrameInfo,
        buf_len: usize,
        buf_type: RingBufferType,
        cycle_state_bit: bool,
    ) -> Result<Self, RingBufferError>
    {
        if !buf_base_mem_info.is_allocated()
            || (buf_base_mem_info.get_frame_size() / size_of::<TransferRequestBlock>()) < buf_len
        {
            return Err(RingBufferError::InvalidMemoryFrameInfoError(buf_base_mem_info));
        }

        return Ok(Self {
            buf_base_virt_addr: buf_base_mem_info.get_frame_start_virt_addr(),
            buf_len,
            buf_type,
            cycle_state: cycle_state_bit,
            is_init: false,
        });
    }

    pub fn init(&mut self)
    {
        self.is_init = true;

        if self.buf_type == RingBufferType::EventRing
        {
            return;
        }

        for i in 0..self.buf_len
        {
            let mut trb = TransferRequestBlock::new();
            trb.set_cycle_bit(!self.cycle_state);

            if i == self.buf_len - 1
            {
                trb.set_trb_type(TransferRequestBlockType::Link);
            }

            self.write(i, trb).unwrap();
        }
    }

    pub fn is_init(&self) -> bool { return self.is_init; }

    pub fn get_buf_len(&self) -> usize { return self.buf_len; }

    pub fn get_empty_buf_len(&self) -> usize
    {
        let mut cnt = 0;

        for i in 0..self.buf_len
        {
            let trb = match self.read(i)
            {
                Some(trb) => trb,
                None => break,
            };

            if trb.cycle_bit() != self.cycle_state
            {
                cnt += 1;
            }
        }

        return cnt - 1;
    }

    pub fn push(&mut self, trb: TransferRequestBlock) -> Result<(), RingBufferError>
    {
        if !self.is_init
        {
            return Err(RingBufferError::NotInitialized);
        }

        if self.buf_type == RingBufferType::EventRing
        {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type));
        }

        let mut trb = trb;
        trb.set_cycle_bit(self.cycle_state);

        let mut is_buf_end = false;
        for i in 0..self.buf_len
        {
            is_buf_end = i == self.buf_len - 2;

            let read_trb = match self.read(i)
            {
                Some(trb) => trb,
                None => return Err(RingBufferError::InvalidRingBufferIndexError(i)),
            };

            if read_trb.cycle_bit() == !self.cycle_state
            {
                match self.write(i, trb)
                {
                    Err(err) => return Err(err),
                    _ => (),
                };
                break;
            }
        }

        // reverse cycle bit
        if is_buf_end
        {
            let link_trb_index = self.buf_len - 1;
            let mut link_trb = match self.read(link_trb_index)
            {
                Some(trb) => trb,
                None => return Err(RingBufferError::InvalidRingBufferIndexError(link_trb_index)),
            };
            link_trb.set_cycle_bit(!link_trb.cycle_bit());
            // true -> toggle, false -> reset
            link_trb.set_toggle_cycle(self.cycle_state);
            match self.write(link_trb_index, link_trb)
            {
                Err(err) => return Err(err),
                _ => (),
            };

            self.cycle_state = !self.cycle_state;
        }

        return Ok(());
    }

    pub fn pop(
        &mut self,
        mut int_reg_set: InterrupterRegisterSet,
    ) -> Result<(TransferRequestBlock, InterrupterRegisterSet), RingBufferError>
    {
        if !self.is_init
        {
            return Err(RingBufferError::NotInitialized);
        }

        if self.buf_type != RingBufferType::EventRing
        {
            return Err(RingBufferError::UnsupportedRingBufferTypeError(self.buf_type));
        }

        if int_reg_set.event_ring_seg_table_size() != 1 || int_reg_set.dequeue_erst_seg_index() != 0
        {
            return Err(RingBufferError::UnsupportedEventRingSegmentTableLengthError);
        }

        println!("er: {:?}", int_reg_set);

        let trb_size = size_of::<TransferRequestBlock>();
        let mut dequeue_ptr =
            PhysicalAddress::new(int_reg_set.event_ring_dequeue_ptr() << 4).get_virt_addr();

        let mut index = (dequeue_ptr.get() - self.buf_base_virt_addr.get()) as usize / trb_size;

        //self.debug();
        println!("0x{:x} (index: {})", dequeue_ptr.get(), index);

        let trb = match self.read(index)
        {
            Some(trb) => trb,
            None => return Err(RingBufferError::InvalidRingBufferIndexError(index)),
        };

        if trb.cycle_bit() != self.cycle_state
        {
            return Err(RingBufferError::InvalidTransferRequestBlockError(index));
        }

        index += 1;

        if index == self.buf_len - 1
        {
            index = 0;
            self.cycle_state = !self.cycle_state;
        }

        // TODO: dequeue pointer is not updated by qemu
        dequeue_ptr = self.buf_base_virt_addr.offset(index * size_of::<TransferRequestBlock>());
        int_reg_set.set_event_ring_dequeue_ptr(dequeue_ptr.get_phys_addr().get() >> 4);
        int_reg_set.set_event_handler_busy(false);

        println!("er: {:?}", int_reg_set);

        return Ok((trb, int_reg_set));
    }

    pub fn debug(&self)
    {
        for i in 0..self.buf_len
        {
            let trb = self.read(i);
            println!("{}: {:?}", i, trb);
        }
    }

    pub fn read(&self, index: usize) -> Option<TransferRequestBlock>
    {
        if index >= self.buf_len
        {
            return None;
        }

        let virt_addr = self.buf_base_virt_addr.offset(index * size_of::<TransferRequestBlock>());
        return Some(virt_addr.read_volatile());
    }

    fn write(&self, index: usize, trb: TransferRequestBlock) -> Result<(), RingBufferError>
    {
        if index >= self.buf_len
        {
            return Err(RingBufferError::InvalidRingBufferIndexError(index));
        }

        let virt_addr = self.buf_base_virt_addr.offset(index * size_of::<TransferRequestBlock>());
        virt_addr.write_volatile(trb);

        return Ok(());
    }
}
