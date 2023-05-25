use core::mem::size_of;

use alloc::vec::Vec;

use crate::mem::bitmap::*;

use super::{descriptor::{config::ConfigurationDescriptor, device::DeviceDescriptor, hid::HumanInterfaceDeviceDescriptor, Descriptor, DescriptorHeader, DescriptorType}, setup_trb::*, xhc::{ring_buffer::*, trb::*, XHC_DRIVER}};

const RING_BUF_LEN: usize = 16;

#[derive(Debug)]
pub enum UsbDeviceError
{
    RingBufferError(RingBufferError),
    BitmapMemoryManagerError(BitmapMemoryManagerError),
    XhcPortNotFoundError,
    XhcDriverWasNotInitializedError,
}

#[derive(Debug)]
pub struct UsbDevice
{
    slot_id: usize,
    transfer_ring_buf: RingBuffer,
    dev_desc_buf_mem_info: MemoryFrameInfo,
    conf_desc_buf_mem_info: MemoryFrameInfo,
}

impl UsbDevice
{
    pub fn new(
        slot_id: usize,
        transfer_ring_mem_info: MemoryFrameInfo,
    ) -> Result<Self, UsbDeviceError>
    {
        let ring_buf = match RingBuffer::new(
            transfer_ring_mem_info,
            RING_BUF_LEN,
            RingBufferType::TransferRing,
            true,
        )
        {
            Ok(transfer_ring_buf) => transfer_ring_buf,
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        };

        let dev_desc_buf_mem_info = match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
        {
            Ok(mem_info) => mem_info,
            Err(err) => return Err(UsbDeviceError::BitmapMemoryManagerError(err)),
        };

        let conf_desc_buf_mem_info = match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
        {
            Ok(mem_info) => mem_info,
            Err(err) => return Err(UsbDeviceError::BitmapMemoryManagerError(err)),
        };

        let dev = Self {
            slot_id,
            transfer_ring_buf: ring_buf,
            dev_desc_buf_mem_info,
            conf_desc_buf_mem_info,
        };
        return Ok(dev);
    }

    pub fn init(&mut self) -> Result<(), UsbDeviceError>
    {
        self.transfer_ring_buf.init();

        match self.request_get_desc(DescriptorType::Device, 0)
        {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        return Ok(());
    }

    pub fn slot_id(&self) -> usize { return self.slot_id; }

    pub fn get_dev_desc(&self) -> DeviceDescriptor
    {
        return self.dev_desc_buf_mem_info.get_frame_start_virt_addr().read_volatile();
    }

    pub fn get_conf_descs(&self) -> Vec<Descriptor>
    {
        let conf_desc: ConfigurationDescriptor =
            self.conf_desc_buf_mem_info.get_frame_start_virt_addr().read_volatile();

        let mut descs = Vec::new();
        let mut offset = conf_desc.header().length() as usize;

        descs.push(Descriptor::Configuration(conf_desc));

        loop
        {
            let addr = self.conf_desc_buf_mem_info.get_frame_start_virt_addr().offset(offset);
            let desc_header = addr.read_volatile::<DescriptorHeader>();

            if desc_header.length() == 0
            {
                break;
            }

            offset += desc_header.length() as usize;

            let desc = match desc_header.ty()
            {
                DescriptorType::Device => Descriptor::Device(addr.read_volatile()),
                DescriptorType::Configration => Descriptor::Configuration(addr.read_volatile()),
                DescriptorType::Endpoint => Descriptor::Endpoint(addr.read_volatile()),
                DescriptorType::Interface => Descriptor::Interface(addr.read_volatile()),
                DescriptorType::HumanInterfaceDevice =>
                {
                    let hid_desc: HumanInterfaceDeviceDescriptor = addr.read_volatile();
                    let num_descs = hid_desc.num_descs() as usize;
                    let mut class_desc_headers = Vec::new();

                    for i in 0..num_descs
                    {
                        let addr = addr.offset(size_of::<DescriptorHeader>() * i);
                        class_desc_headers.push(addr.read_volatile());
                    }

                    Descriptor::HumanInterfaceDevice(hid_desc, class_desc_headers)
                }
                other => Descriptor::Unsupported(other),
            };

            descs.push(desc);
        }

        return descs;
    }

    pub fn request_get_desc(
        &mut self,
        desc_type: DescriptorType,
        desc_index: usize,
    ) -> Result<(), UsbDeviceError>
    {
        let buf_mem_info = match desc_type
        {
            DescriptorType::Device => self.dev_desc_buf_mem_info,
            DescriptorType::Configration => self.conf_desc_buf_mem_info,
            _ => unimplemented!(),
        };

        let buf_size = match desc_type
        {
            DescriptorType::Device => self.dev_desc_buf_mem_info.get_frame_size(),
            DescriptorType::Configration => self.dev_desc_buf_mem_info.get_frame_size(),
            _ => unimplemented!(),
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
        setup_stage_trb.set_setup_value((desc_type as u16) << 8 | desc_index as u16);
        setup_stage_trb.set_setup_length(buf_size as u16);
        setup_stage_trb.set_status(8); // TRB transfer length
        setup_stage_trb.set_ctrl_regs(3); // TRT bits
        setup_stage_trb.set_other_flags(3 << 4); // IOC and IDT bit

        let mut data_stage_trb = TransferRequestBlock::new();
        data_stage_trb.set_trb_type(TransferRequestBlockType::DataStage); // Data Stage

        data_stage_trb.set_param(buf_mem_info.get_frame_start_virt_addr().get_phys_addr().get());
        data_stage_trb.set_status(buf_size as u32);
        data_stage_trb.set_other_flags(1 << 4); // IOC bit
        data_stage_trb.set_ctrl_regs(1); // DIR bit

        let mut status_stage_trb = TransferRequestBlock::new();
        status_stage_trb.set_trb_type(TransferRequestBlockType::StatusStage);
        status_stage_trb.set_other_flags(1 << 4); // IOC bit

        match self.transfer_ring_buf.push(setup_stage_trb)
        {
            Ok(_) => (),
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        }

        match self.transfer_ring_buf.push(data_stage_trb)
        {
            Ok(_) => (),
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        }

        match self.transfer_ring_buf.push(status_stage_trb)
        {
            Ok(_) => (),
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        }

        match XHC_DRIVER.lock().as_ref()
        {
            Some(xhc_driver) => xhc_driver.ring_doorbell(self.slot_id, 1),
            None => return Err(UsbDeviceError::XhcDriverWasNotInitializedError),
        }

        return Ok(());
    }
}
