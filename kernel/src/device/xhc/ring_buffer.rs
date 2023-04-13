use log::{info, warn};

use crate::{arch::addr::*, mem::bitmap::MemoryFrameInfo, println};
use core::mem::size_of;

use super::{register::*, trb::{TransferRequestBlock, TransferRequestBlockType}};

#[derive(Debug, PartialEq, Eq)]
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
        buf_base_mem_frame: MemoryFrameInfo,
        buf_len: usize,
        buf_type: RingBufferType,
        cycle_state_bit: bool,
    ) -> Option<Self>
    {
        if !buf_base_mem_frame.is_allocated()
            || (buf_base_mem_frame.get_frame_size() / size_of::<TransferRequestBlock>()) < buf_len
        {
            return None;
        }

        return Some(Self {
            buf_base_virt_addr: buf_base_mem_frame.get_frame_start_virt_addr(),
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
            let trb = self.read(i).unwrap();
            if trb.cycle_bit() != self.cycle_state
            {
                cnt += 1;
            }
        }

        return cnt - 1;
    }

    pub fn push(&mut self, trb: TransferRequestBlock) -> Result<(), &'static str>
    {
        if !self.is_init
        {
            return Err("Ring buffer is not initialized");
        }

        if self.buf_type == RingBufferType::EventRing
        {
            return Err("Event ring is not support push");
        }

        let mut trb = trb;
        trb.set_cycle_bit(self.cycle_state);

        let mut is_buf_end = false;
        for i in 0..self.buf_len
        {
            is_buf_end = i == self.buf_len - 2;

            let read_trb = self.read(i).unwrap();

            if read_trb.cycle_bit() == !self.cycle_state
            {
                self.write(i, trb).unwrap();
                break;
            }
        }

        // reverse cycle bit
        if is_buf_end
        {
            let link_trb_index = self.buf_len - 1;
            let mut link_trb = self.read(link_trb_index).unwrap();
            link_trb.set_cycle_bit(!link_trb.cycle_bit());
            // true -> toggle, false -> reset
            link_trb.set_toggle_cycle(self.cycle_state).unwrap();
            self.write(link_trb_index, link_trb).unwrap();
            self.cycle_state = !self.cycle_state;
        }

        info!("xhci: Pushed to ring buffer: {:?} (empty len is {})", trb, self.get_empty_buf_len());

        return Ok(());
    }

    pub fn pop(
        &self,
        mut int_reg_set: InterrupterRegisterSet,
    ) -> Option<(TransferRequestBlock, InterrupterRegisterSet)>
    {
        if !self.is_init
        {
            return None;
        }

        if self.buf_type != RingBufferType::EventRing
        {
            return None;
        }

        if int_reg_set.event_ring_seg_table_size() != 1 || int_reg_set.dequeue_erst_seg_index() != 0
        {
            warn!("xhci: Unsupported event ring segment table length (is not 1)");
            return None;
        }

        let trb_size = size_of::<TransferRequestBlock>();
        let mut dequeue_addr =
            PhysicalAddress::new(int_reg_set.event_ring_dequeue_ptr() << 4).get_virt_addr();

        let mut index = (dequeue_addr.get() - self.buf_base_virt_addr.get()) as usize / trb_size;

        let mut trb = self.read(index).unwrap();

        let mut skip_cnt = 0;
        loop
        {
            if skip_cnt == self.buf_len
            {
                warn!("xhci: Valid TRB was not found");
                return None;
            }

            if index >= self.buf_len
            {
                index = 0;
            }

            trb = self.read(index).unwrap();
            if trb.cycle_bit() == self.cycle_state
            {
                break;
            }

            index += 1;
            skip_cnt += 1;
        }

        let tmp_trb = trb;
        trb.set_cycle_bit(!self.cycle_state);
        self.write(index, trb).unwrap();

        dequeue_addr = self.buf_base_virt_addr.offset(index * size_of::<TransferRequestBlock>());
        int_reg_set.set_event_ring_dequeue_ptr(dequeue_addr.get_phys_addr().get() >> 4);
        int_reg_set.set_event_handler_busy(false);
        info!("xhci: Poped from ring buffer: {:?} (index: {})", tmp_trb, index);

        return Some((tmp_trb, int_reg_set));
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

    fn write(&self, index: usize, trb: TransferRequestBlock) -> Result<(), &'static str>
    {
        if index >= self.buf_len
        {
            return Err("Index out of bounds");
        }

        let virt_addr = self.buf_base_virt_addr.offset(index * size_of::<TransferRequestBlock>());
        virt_addr.write_volatile(trb);

        return Ok(());
    }
}
