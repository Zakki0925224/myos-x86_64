use crate::{arch::addr::{PhysicalAddress, VirtualAddress}, mem::bitmap::MemoryFrameInfo, println};
use alloc::vec::Vec;
use core::mem::size_of;

use super::register::{EventRingSegmentTableEntry, InterrupterRegisterSet, TransferRequestBlock, TransferRequestBlockType};

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
    buf_base_mem_frame: MemoryFrameInfo,
    event_ring_seg_table_entries: Option<Vec<EventRingSegmentTableEntry>>,
    buf_len: usize,
    buf_type: RingBufferType,
    pcs: bool,
}

impl RingBuffer
{
    pub fn new(
        buf_base_mem_frame: MemoryFrameInfo,
        event_ring_seg_table_entries: Option<Vec<EventRingSegmentTableEntry>>,
        buf_len: usize,
        buf_type: RingBufferType,
        pcs: bool,
    ) -> Option<Self>
    {
        if !buf_base_mem_frame.is_allocated()
            || (buf_base_mem_frame.get_frame_size() / size_of::<TransferRequestBlock>()) < buf_len
        {
            return None;
        }

        if buf_type == RingBufferType::EventRing
        {
            if event_ring_seg_table_entries.is_none()
            {
                return None;
            }

            let ring_seg_size_sum: usize = event_ring_seg_table_entries
                .as_ref()
                .unwrap()
                .iter()
                .map(|e| e.ring_seg_size() as usize)
                .sum();

            if ring_seg_size_sum != buf_len
                || event_ring_seg_table_entries.as_ref().unwrap()[0].ring_seg_base_addr()
                    != buf_base_mem_frame.get_frame_start_virt_addr().get_phys_addr().get()
            {
                return None;
            }
        }

        return Some(Self {
            buf_base_mem_frame,
            event_ring_seg_table_entries,
            buf_len,
            buf_type,
            pcs,
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
            trb.set_cycle_bit(!self.pcs);

            if i == self.buf_len - 1
            {
                trb.set_trb_type(TransferRequestBlockType::Link);
            }

            self.write(i, trb).unwrap();
        }
    }

    pub fn is_init(&self) -> bool { return self.is_init; }

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
        trb.set_cycle_bit(self.pcs);

        let mut is_buf_end = false;
        for i in 0..self.buf_len
        {
            is_buf_end = i == self.buf_len - 2;

            let read_trb = self.read(i).unwrap();

            if read_trb.cycle_bit() == !self.pcs
            {
                self.write(i, trb).unwrap();
                break;
            }
        }

        if is_buf_end
        {
            let mut link_trb = self.read(self.buf_len - 1).unwrap();
            link_trb.set_cycle_bit(!link_trb.cycle_bit());
            self.write(self.buf_len - 1, link_trb).unwrap();
            self.pcs = !self.pcs;
        }

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

        let mut dequeue_addr =
            PhysicalAddress::new(int_reg_set.event_ring_dequeue_ptr() << 4).get_virt_addr();

        let mut index = 0;
        if let Some(event_ring_seg_table_entries) = &self.event_ring_seg_table_entries
        {
            for entry in event_ring_seg_table_entries
            {
                let addr = PhysicalAddress::new(entry.ring_seg_base_addr() << 6).get_virt_addr();
                let size = entry.ring_seg_size() as usize;

                if dequeue_addr.get() >= addr.get() && dequeue_addr.get() <= addr.offset(size).get()
                {
                    index += (dequeue_addr.get() - addr.get()) as usize
                        / size_of::<TransferRequestBlock>();
                    break;
                }

                index += size / size_of::<TransferRequestBlock>();
            }
        }
        else
        {
            return None;
        }

        dequeue_addr = dequeue_addr.offset(size_of::<TransferRequestBlock>());

        let trb = self.read(index).unwrap();
        int_reg_set.set_event_ring_dequeue_ptr(dequeue_addr.get_phys_addr().get() >> 4);

        return Some((trb, int_reg_set));
    }

    fn get_buf_virt_addr(&self) -> VirtualAddress
    {
        return self.buf_base_mem_frame.get_frame_start_virt_addr();
    }

    fn find_event_ring_seg_index(&self, index: usize) -> Option<(usize, usize)>
    {
        let mut offset = 0;

        if let Some(event_ring_seg_table_entries) = &self.event_ring_seg_table_entries
        {
            for (i, e) in event_ring_seg_table_entries.iter().enumerate()
            {
                let size = e.ring_seg_size() as usize;
                if index < offset + size
                {
                    return Some((i, index - offset));
                }

                offset += size;
            }

            return None;
        }

        return None;
    }

    pub fn read(&self, index: usize) -> Option<TransferRequestBlock>
    {
        if index >= self.buf_len
        {
            return None;
        }

        let event_ring_seg_index = self.find_event_ring_seg_index(index);
        if self.buf_type == RingBufferType::EventRing && event_ring_seg_index.is_none()
        {
            return None;
        }

        if let Some((i, offset)) = event_ring_seg_index
        {
            let virt_addr = VirtualAddress::new(
                self.event_ring_seg_table_entries.as_ref().unwrap()[i].ring_seg_base_addr(),
            )
            .offset(offset * size_of::<TransferRequestBlock>());
            return Some(virt_addr.read_volatile());
        }

        let virt_addr = self.get_buf_virt_addr().offset(index * size_of::<TransferRequestBlock>());
        return Some(virt_addr.read_volatile());
    }

    fn write(&self, index: usize, trb: TransferRequestBlock) -> Result<(), &'static str>
    {
        if index >= self.buf_len
        {
            return Err("Index out of bounds");
        }

        let event_ring_seg_index = self.find_event_ring_seg_index(index);
        if self.buf_type == RingBufferType::EventRing && event_ring_seg_index.is_none()
        {
            return Err("Index was not found in ring buffer");
        }

        if let Some((i, offset)) = event_ring_seg_index
        {
            let virt_addr = VirtualAddress::new(
                self.event_ring_seg_table_entries.as_ref().unwrap()[i].ring_seg_base_addr(),
            )
            .offset(offset * size_of::<TransferRequestBlock>());
            virt_addr.write_volatile(trb);
            return Ok(());
        }

        let virt_addr = self.get_buf_virt_addr().offset(index * size_of::<TransferRequestBlock>());
        virt_addr.write_volatile(trb);

        return Ok(());
    }
}
