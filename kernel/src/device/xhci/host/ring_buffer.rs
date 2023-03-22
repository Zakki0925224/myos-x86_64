use crate::{arch::addr::VirtualAddress, mem::bitmap::MemoryFrameInfo};
use alloc::vec::Vec;
use core::mem::size_of;

use super::register::{EventRingSegmentTableEntry, TransferRequestBlock, TransferRequestBlockType};

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
        });
    }

    pub fn init(&self)
    {
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

            self.write(i, trb);
        }
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

    fn read(&self, index: usize) -> Option<TransferRequestBlock>
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

    fn write(&self, index: usize, trb: TransferRequestBlock)
    {
        if index >= self.buf_len
        {
            return;
        }

        let event_ring_seg_index = self.find_event_ring_seg_index(index);
        if self.buf_type == RingBufferType::EventRing && event_ring_seg_index.is_none()
        {
            return;
        }

        if let Some((i, offset)) = event_ring_seg_index
        {
            let virt_addr = VirtualAddress::new(
                self.event_ring_seg_table_entries.as_ref().unwrap()[i].ring_seg_base_addr(),
            )
            .offset(offset * size_of::<TransferRequestBlock>());
            virt_addr.write_volatile(trb);
            return;
        }

        let virt_addr = self.get_buf_virt_addr().offset(index * size_of::<TransferRequestBlock>());
        virt_addr.write_volatile(trb);
    }
}
