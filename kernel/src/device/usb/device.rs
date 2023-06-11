use core::mem::size_of;

use alloc::vec::Vec;

use crate::{arch::addr::{PhysicalAddress, VirtualAddress}, device::usb::xhc::context::input::InputControlContext, mem::bitmap::*};

use super::{descriptor::{config::ConfigurationDescriptor, device::DeviceDescriptor, endpoint::EndpointDescriptor, hid::HumanInterfaceDeviceDescriptor, interface::InterfaceDescriptor, Descriptor, DescriptorHeader, DescriptorType}, setup_trb::*, xhc::{context::endpoint::*, ring_buffer::*, trb::*, XHC_DRIVER}};

const RING_BUF_LEN: usize = 8;
const DEFAULT_CONTROL_PIPE_ID: u8 = 1;

#[derive(Debug)]
pub enum UsbDeviceError
{
    RingBufferError(RingBufferError),
    BitmapMemoryManagerError(BitmapMemoryManagerError),
    XhcPortNotFoundError,
    XhcDriverWasNotInitializedError,
    InterfaceDescriptorWasNotFoundError(usize),
    InvalidTransferRequestBlockTypeError,
}

#[derive(Debug)]
pub struct UsbDevice
{
    slot_id: usize,
    transfer_ring_bufs: [Option<RingBuffer>; 32],
    dev_desc_buf_mem_info: MemoryFrameInfo,
    conf_desc_buf_mem_info: MemoryFrameInfo,

    max_packet_size: u16,

    configured_endpoint_dci: Vec<(usize, VirtualAddress)>,
    using_interface_desc_index: usize,
    current_conf_index: usize,
    dev_desc: DeviceDescriptor,
    conf_descs: Vec<Descriptor>,
}

impl UsbDevice
{
    pub fn new(
        slot_id: usize,
        transfer_ring_mem_info: MemoryFrameInfo,
        max_packet_size: u16,
    ) -> Result<Self, UsbDeviceError>
    {
        let mut dcp_ring_buf = match RingBuffer::new(
            transfer_ring_mem_info,
            RING_BUF_LEN,
            RingBufferType::TransferRing,
            true,
        )
        {
            Ok(transfer_ring_buf) => transfer_ring_buf,
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        };

        dcp_ring_buf.init();

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

        let mut transfer_ring_bufs = [None; 32];
        transfer_ring_bufs[1] = Some(dcp_ring_buf);

        let dev = Self {
            slot_id,
            transfer_ring_bufs,
            dev_desc_buf_mem_info,
            conf_desc_buf_mem_info,
            max_packet_size,
            configured_endpoint_dci: Vec::new(),
            using_interface_desc_index: 0,
            current_conf_index: 0,
            dev_desc: DeviceDescriptor::new(),
            conf_descs: Vec::new(),
        };

        return Ok(dev);
    }

    pub fn init(&mut self) -> Result<(), UsbDeviceError>
    {
        match self.request_to_get_desc(DescriptorType::Device, 0)
        {
            Ok(_) => (),
            Err(err) => return Err(err),
        }

        return Ok(());
    }

    pub fn slot_id(&self) -> usize { return self.slot_id; }

    pub fn read_dev_desc(&mut self)
    {
        self.dev_desc = self.dev_desc_buf_mem_info.get_frame_start_virt_addr().read_volatile();
    }

    pub fn read_conf_descs(&mut self)
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

        self.conf_descs = descs;
    }

    pub fn get_dev_desc(&self) -> &DeviceDescriptor { return &self.dev_desc; }

    pub fn get_conf_descs(&self) -> &Vec<Descriptor> { return &self.conf_descs; }

    pub fn request_to_get_desc(
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

        let setup_value = match desc_type
        {
            DescriptorType::Device => 0x100,
            DescriptorType::Configration => (desc_type as u16) << 8 | desc_index as u16,
            _ => unimplemented!(),
        };

        match desc_type
        {
            DescriptorType::Configration => self.current_conf_index = desc_index,
            _ => (),
        }

        let buf_size = buf_mem_info.get_frame_size();

        return self.ctrl_in(
            RequestType::Standard,
            RequestTypeRecipient::Device,
            SetupRequest::GetDescriptor,
            setup_value,
            0,
            buf_size as u16,
            buf_mem_info.get_frame_start_virt_addr().get_phys_addr(),
            buf_size as u32,
        );
    }

    pub fn request_to_set_conf(&mut self, conf_desc_value: u8) -> Result<(), UsbDeviceError>
    {
        let mut setup_stage_trb = TransferRequestBlock::new();
        setup_stage_trb.set_trb_type(TransferRequestBlockType::SetupStage);

        let mut setup_req_type = SetupRequestType::new();
        setup_req_type.set_direction(RequestTypeDirection::Out);
        setup_req_type.set_ty(RequestType::Standard);
        setup_req_type.set_recipient(RequestTypeRecipient::Device);

        setup_stage_trb.set_setup_request_type(setup_req_type);
        setup_stage_trb.set_setup_request(SetupRequest::SetConfiguration);
        setup_stage_trb.set_setup_index(0);
        setup_stage_trb.set_setup_value(conf_desc_value as u16);
        setup_stage_trb.set_setup_length(0);
        setup_stage_trb.set_status(8); // TRB transfer length
        setup_stage_trb.set_ctrl_regs(0); // TRT bits
        setup_stage_trb.set_other_flags(1 << 4); // IDT bit

        let mut status_stage_trb = TransferRequestBlock::new();
        status_stage_trb.set_trb_type(TransferRequestBlockType::StatusStage);
        status_stage_trb.set_other_flags(1 << 4); // IOC bit
        status_stage_trb.set_ctrl_regs(1); // DIR bit

        return self.send_to_transfer_ring(
            setup_stage_trb,
            None,
            Some(status_stage_trb),
            DEFAULT_CONTROL_PIPE_ID,
        );
    }

    pub fn get_num_confs(&self) -> usize { return self.get_dev_desc().num_configs() as usize; }

    pub fn get_interface_descs(&self) -> Vec<&InterfaceDescriptor>
    {
        return self
            .conf_descs
            .iter()
            .filter(|d| matches!(**d, Descriptor::Interface(_)))
            .map(|d| match d
            {
                Descriptor::Interface(desc) => desc,
                _ => unreachable!(),
            })
            .collect();
    }

    pub fn get_endpoint_descs(&self) -> Vec<&EndpointDescriptor>
    {
        return self
            .conf_descs
            .iter()
            .filter(|d| matches!(**d, Descriptor::Endpoint(_)))
            .map(|d| match d
            {
                Descriptor::Endpoint(desc) => desc,
                _ => unreachable!(),
            })
            .collect();
    }

    pub fn configure_endpoint(&mut self) -> Result<(), UsbDeviceError>
    {
        if let Some(xhc_driver) = XHC_DRIVER.lock().as_mut()
        {
            let port = match xhc_driver.find_port_by_slot_id(self.slot_id)
            {
                Some(port) => port,
                None => return Err(UsbDeviceError::XhcPortNotFoundError),
            };

            let mut transfer_ring_bufs = self.transfer_ring_bufs;
            let mut configured_endpoint_dci = self.configured_endpoint_dci.clone();

            let device_context = xhc_driver.read_device_context(self.slot_id).unwrap();
            let mut input_context = port.read_input_context();
            input_context.device_context.slot_context = device_context.slot_context;
            let mut input_ctrl_context = InputControlContext::new();
            input_ctrl_context.set_add_context_flag(0, true).unwrap();

            for endpoint_desc in self.get_endpoint_descs()
            {
                let transfer_ring_buf_mem_info =
                    match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
                    {
                        Ok(mem_info) => mem_info,
                        Err(err) => return Err(UsbDeviceError::BitmapMemoryManagerError(err)),
                    };

                let mut transfer_ring_buf = match RingBuffer::new(
                    transfer_ring_buf_mem_info,
                    RING_BUF_LEN,
                    RingBufferType::TransferRing,
                    true,
                )
                {
                    Ok(ring_buf) => ring_buf,
                    Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
                };

                transfer_ring_buf.init();
                match transfer_ring_buf.fill()
                {
                    Ok(_) => (),
                    Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
                }

                let endpoint_addr = endpoint_desc.endpoint_addr();
                let dci = endpoint_desc.dci();

                let mut endpoint_context = EndpointContext::new();
                endpoint_context.set_endpoint_type(EndpointType::new(
                    endpoint_addr,
                    endpoint_desc.bitmap_attrs(),
                ));
                endpoint_context.set_max_packet_size(self.max_packet_size);
                endpoint_context
                    .set_max_endpoint_service_interval_payload_low(self.max_packet_size);
                endpoint_context.set_max_burst_size(0);
                endpoint_context.set_dequeue_cycle_state(true);
                endpoint_context.set_tr_dequeue_ptr(
                    transfer_ring_buf_mem_info.get_frame_start_virt_addr().get_phys_addr().get()
                        >> 1,
                );
                endpoint_context.set_interval(endpoint_desc.interval());
                endpoint_context.set_max_primary_streams(0);
                endpoint_context.set_mult(0);
                endpoint_context.set_error_cnt(3);
                endpoint_context.set_average_trb_len(1);

                input_context.device_context.endpoint_contexts[dci - 1] = endpoint_context;
                input_ctrl_context.set_add_context_flag(dci, true).unwrap();

                transfer_ring_bufs[dci] = Some(transfer_ring_buf);
                configured_endpoint_dci
                    .push((dci, transfer_ring_buf_mem_info.get_frame_start_virt_addr()));
            }

            input_context.input_ctrl_context = input_ctrl_context;
            port.write_input_context(input_context);

            self.transfer_ring_bufs = transfer_ring_bufs;
            self.configured_endpoint_dci = configured_endpoint_dci;

            let mut config_endpoint_trb = TransferRequestBlock::new();
            config_endpoint_trb.set_trb_type(TransferRequestBlockType::ConfigureEndpointCommnad);
            config_endpoint_trb.set_ctrl_regs((self.slot_id as u16) << 8);
            config_endpoint_trb.set_param(port.input_context_base_virt_addr.get_phys_addr().get());

            return match xhc_driver.push_cmd_ring(config_endpoint_trb)
            {
                Ok(_) => Ok(()),
                Err(_) => unimplemented!(),
            };
        }
        else
        {
            return Err(UsbDeviceError::XhcDriverWasNotInitializedError);
        }
    }

    fn request_to_set_interface(&mut self) -> Result<(), UsbDeviceError>
    {
        let interface_desc = match self.get_using_interface_desc()
        {
            Some(desc) => desc,
            None =>
            {
                return Err(UsbDeviceError::InterfaceDescriptorWasNotFoundError(
                    self.using_interface_desc_index,
                ))
            }
        };

        return self.ctrl_out(
            RequestType::Standard,
            RequestTypeRecipient::Interface,
            SetupRequest::SetInterface,
            interface_desc.alternate_setting() as u16,
            interface_desc.interface_num() as u16,
            0,
        );
    }

    pub fn request_to_use_boot_protocol(&mut self) -> Result<(), UsbDeviceError>
    {
        let interface_desc = match self.get_using_interface_desc()
        {
            Some(desc) => desc,
            None =>
            {
                return Err(UsbDeviceError::InterfaceDescriptorWasNotFoundError(
                    self.using_interface_desc_index,
                ))
            }
        };

        return self.ctrl_out(
            RequestType::Class,
            RequestTypeRecipient::Interface,
            SetupRequest::SET_PROTOCOL,
            0, // boot protocol
            interface_desc.interface_num() as u16,
            0,
        );
    }

    pub fn update(&mut self, endpoint_id: usize)
    {
        if let Some(ring_buf) = self.transfer_ring_bufs[endpoint_id].as_mut()
        {
            let data_buf_phys_addr = BITMAP_MEM_MAN
                .lock()
                .alloc_single_mem_frame()
                .unwrap()
                .get_frame_start_virt_addr()
                .get_phys_addr();

            let mut trb = TransferRequestBlock::new();

            trb.set_trb_type(TransferRequestBlockType::Normal);
            trb.set_param(data_buf_phys_addr.get());
            trb.set_status(8); // TRB Transfer Lenght
            trb.set_other_flags(0x112); // BEI, IOC, ISP bit
            ring_buf.push(trb).unwrap();

            ring_buf.debug();
        }
    }

    fn ctrl_out(
        &mut self,
        req_type: RequestType,
        req_type_recipient: RequestTypeRecipient,
        setup_req: SetupRequest,
        setup_value: u16,
        setup_index: u16,
        setup_length: u16,
    ) -> Result<(), UsbDeviceError>
    {
        let mut setup_stage_trb = TransferRequestBlock::new();
        setup_stage_trb.set_trb_type(TransferRequestBlockType::SetupStage);

        let mut setup_req_type = SetupRequestType::new();
        setup_req_type.set_direction(RequestTypeDirection::Out);
        setup_req_type.set_ty(req_type);
        setup_req_type.set_recipient(req_type_recipient);

        setup_stage_trb.set_setup_request_type(setup_req_type);
        setup_stage_trb.set_setup_request(setup_req);
        setup_stage_trb.set_setup_index(setup_index);
        setup_stage_trb.set_setup_value(setup_value);
        setup_stage_trb.set_setup_length(setup_length);
        setup_stage_trb.set_status(8); // TRB transfer length
        setup_stage_trb.set_ctrl_regs(0); // TRT bits
        setup_stage_trb.set_other_flags(3 << 4); // IOC and IDT bit

        let mut data_stage_trb = TransferRequestBlock::new();
        data_stage_trb.set_trb_type(TransferRequestBlockType::DataStage);
        data_stage_trb.set_other_flags(1 << 4); // IOC bit
        data_stage_trb.set_ctrl_regs(0); // DIR bit

        let mut status_stage_trb = TransferRequestBlock::new();
        status_stage_trb.set_trb_type(TransferRequestBlockType::StatusStage);
        status_stage_trb.set_ctrl_regs(1); // DIR bit

        return self.send_to_transfer_ring(
            setup_stage_trb,
            Some(data_stage_trb),
            Some(status_stage_trb),
            DEFAULT_CONTROL_PIPE_ID,
        );
    }

    fn ctrl_in(
        &mut self,
        req_type: RequestType,
        req_type_recipient: RequestTypeRecipient,
        setup_req: SetupRequest,
        setup_value: u16,
        setup_index: u16,
        setup_length: u16,
        data_buf_phys_addr: PhysicalAddress,
        buf_size: u32,
    ) -> Result<(), UsbDeviceError>
    {
        let mut setup_stage_trb = TransferRequestBlock::new();
        setup_stage_trb.set_trb_type(TransferRequestBlockType::SetupStage);

        let mut request_type = SetupRequestType::new();
        request_type.set_direction(RequestTypeDirection::In);
        request_type.set_ty(req_type);
        request_type.set_recipient(req_type_recipient);

        setup_stage_trb.set_setup_request_type(request_type);
        setup_stage_trb.set_setup_request(setup_req);
        setup_stage_trb.set_setup_index(setup_index);
        setup_stage_trb.set_setup_value(setup_value);
        setup_stage_trb.set_setup_length(setup_length);
        setup_stage_trb.set_status(8); // TRB transfer length
        setup_stage_trb.set_ctrl_regs(3); // TRT bits
        setup_stage_trb.set_other_flags(3 << 4); // IOC and IDT bit

        let mut data_stage_trb = TransferRequestBlock::new();
        data_stage_trb.set_trb_type(TransferRequestBlockType::DataStage);
        data_stage_trb.set_param(data_buf_phys_addr.get());
        data_stage_trb.set_status(buf_size);
        data_stage_trb.set_other_flags(1 << 4); // IOC bit
        data_stage_trb.set_ctrl_regs(1); // DIR bit

        let mut status_stage_trb = TransferRequestBlock::new();
        status_stage_trb.set_trb_type(TransferRequestBlockType::StatusStage);
        status_stage_trb.set_ctrl_regs(0); // DIR bit

        return self.send_to_transfer_ring(
            setup_stage_trb,
            Some(data_stage_trb),
            Some(status_stage_trb),
            DEFAULT_CONTROL_PIPE_ID,
        );
    }

    fn send_to_transfer_ring(
        &mut self,
        setup_stage_trb: TransferRequestBlock,
        data_stage_trb: Option<TransferRequestBlock>,
        status_stage_trb: Option<TransferRequestBlock>,
        doorbell_target: u8,
    ) -> Result<(), UsbDeviceError>
    {
        if setup_stage_trb.trb_type() != TransferRequestBlockType::SetupStage
            || (data_stage_trb.is_some()
                && data_stage_trb.unwrap().trb_type() != TransferRequestBlockType::DataStage)
            || (status_stage_trb.is_some()
                && status_stage_trb.unwrap().trb_type() != TransferRequestBlockType::StatusStage)
        {
            return Err(UsbDeviceError::InvalidTransferRequestBlockTypeError);
        }

        let dcp_transfer_ring = self.transfer_ring_bufs[1].as_mut().unwrap();

        match dcp_transfer_ring.push(setup_stage_trb)
        {
            Ok(_) => (),
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        }

        if data_stage_trb.is_none()
        {
            return Ok(());
        }

        match dcp_transfer_ring.push(data_stage_trb.unwrap())
        {
            Ok(_) => (),
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        }

        if status_stage_trb.is_none()
        {
            return Ok(());
        }

        match dcp_transfer_ring.push(status_stage_trb.unwrap())
        {
            Ok(_) => (),
            Err(err) => return Err(UsbDeviceError::RingBufferError(err)),
        }

        match XHC_DRIVER.lock().as_ref()
        {
            Some(xhc_driver) => xhc_driver.ring_doorbell(self.slot_id, doorbell_target),
            None => return Err(UsbDeviceError::XhcDriverWasNotInitializedError),
        }

        return Ok(());
    }

    fn get_using_interface_desc(&self) -> Option<&InterfaceDescriptor>
    {
        return self
            .conf_descs
            .iter()
            .filter(|d| matches!(**d, Descriptor::Interface(_)))
            .map(|d| match d
            {
                Descriptor::Interface(desc) => desc,
                _ => unreachable!(),
            })
            .find(|d| d.interface_index() == self.using_interface_desc_index as u8);
    }
}
