use crate::{mem::bitmap::*, println};

use super::{descriptor::DescriptorType, setup_trb::*, xhc::{ring_buffer::*, trb::*, XHC_DRIVER}};

const RING_BUF_LEN: usize = 16;

#[derive(Debug)]
pub enum UsbDeviceError
{
    RingBufferError(RingBufferError),
    BitmapMemoryManagerError(BitmapMemoryManagerError),
}

#[derive(Debug)]
pub struct UsbDevice
{
    slot_id: usize,
    transfer_ring_buf: RingBuffer,
}

impl UsbDevice
{
    pub fn new(
        slot_id: usize,
        transfer_ring_mem_info: MemoryFrameInfo,
    ) -> Result<Self, UsbDeviceError>
    {
        match RingBuffer::new(
            transfer_ring_mem_info,
            RING_BUF_LEN,
            RingBufferType::TransferRing,
            true,
        )
        {
            Ok(transfer_ring_buf) => return Ok(Self { slot_id, transfer_ring_buf }),
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        }
    }

    pub fn init(&mut self)
    {
        self.transfer_ring_buf.init();

        self.get_desc(DescriptorType::Device);
    }

    pub fn slot_id(&self) -> usize { return self.slot_id; }

    fn get_desc(&mut self, desc_type: DescriptorType) -> Result<(), UsbDeviceError>
    {
        let desc_buf_mem_info = match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
        {
            Ok(mem_info) => mem_info,
            Err(err) => return Err(UsbDeviceError::BitmapMemoryManagerError(err)),
        };

        let mut setup_stage_trb = TransferRequestBlock::new();
        setup_stage_trb.set_trb_type(TransferRequestBlockType::SetupStage);

        let mut request_type = SetupRequestType::new();
        request_type.set_direction(RequestTypeDirection::In);
        request_type.set_ty(RequestType::Standard);
        request_type.set_recipient(RequestTypeRecipient::Device);

        setup_stage_trb.set_setup_request_type(request_type);
        setup_stage_trb.set_setup_request(SetupRequest::GetDescriptor);
        setup_stage_trb.set_setup_index(0);
        setup_stage_trb.set_setup_value((desc_type as u16) << 8);
        setup_stage_trb.set_setup_length(desc_buf_mem_info.get_frame_size() as u16);
        setup_stage_trb.set_status(8);
        setup_stage_trb.set_ctrl_regs(3);

        let mut data_stage_trb = TransferRequestBlock::new();
        data_stage_trb.set_trb_type(TransferRequestBlockType::BandwithRequestEvent); // Data Stage with DIR bit

        data_stage_trb
            .set_param(desc_buf_mem_info.get_frame_start_virt_addr().get_phys_addr().get());
        data_stage_trb.set_status(desc_buf_mem_info.get_frame_size() as u32);
        data_stage_trb.set_other_flags(1 << 4);

        let mut status_stage_trb = TransferRequestBlock::new();
        status_stage_trb.set_trb_type(TransferRequestBlockType::StatusStage);

        if let Err(err) = self.transfer_ring_buf.push(setup_stage_trb)
        {
            return Err(UsbDeviceError::RingBufferError(err));
        }

        if let Err(err) = self.transfer_ring_buf.push(data_stage_trb)
        {
            return Err(UsbDeviceError::RingBufferError(err));
        }

        if let Err(err) = self.transfer_ring_buf.push(status_stage_trb)
        {
            return Err(UsbDeviceError::RingBufferError(err));
        }

        if let Some(xhc_driver) = XHC_DRIVER.lock().as_ref()
        {
            xhc_driver.ring_doorbell(self.slot_id, 1);
        }

        return Ok(());
    }
}
