use core::mem::size_of;

use crate::{arch::{addr::*, apic::local::read_local_apic_id, idt::VEC_XHCI_INT, register::msi::*}, bus::pci::{conf_space::*, device_id::*, PCI_DEVICE_MAN}, device::usb::xhc::{register::*, ring_buffer::RingBufferType}, mem::bitmap::{BitmapMemoryManagerError, BITMAP_MEM_MAN}, println};
use alloc::vec::Vec;
use log::{info, warn};

use super::{context::{device::DeviceContext, endpoint::{EndpointContext, EndpointType}, input::InputContext, slot::SlotContext}, device::Device, port::{ConfigState, Port}, register::{CapabilityRegisters, DoorbellRegister, InterrupterRegisterSet, OperationalRegisters, PortRegisterSet, DOORBELL_REG_MAX_LEN, INTR_REG_SET_MAX_LEN}, ring_buffer::{RingBuffer, RingBufferError}, trb::{CompletionCode, TransferRequestBlock, TransferRequestBlockType}};

const PORT_REG_SETS_START_VIRT_ADDR_OFFSET: usize = 1024;
const RING_BUF_LEN: usize = 16;

#[derive(Debug)]
pub enum XhcDriverError
{
    XhcDeviceWasNotFoundError,
    InvalidRegisterAddressError,
    InvalidInterrupterRegisterSetIndexError(usize),
    InvalidPortRegisterSetIndexError(usize),
    InvalidDoorbellRegisterIndexError(usize),
    InvalidDeviceContextArrayIndexError(usize),
    HostControllerIsNotHaltedError,
    BitmapMemoryManagerError(BitmapMemoryManagerError),
    RingBufferError(RingBufferError),
    OtherError(&'static str),
    NotInitialized,
    NotRunning,
}

#[derive(Debug)]
pub struct XhcDriver
{
    is_init: bool,
    controller_pci_bus: usize,
    controller_pci_device: usize,
    controller_pci_func: usize,
    cap_reg_virt_addr: VirtualAddress,
    ope_reg_virt_addr: VirtualAddress,
    runtime_reg_virt_addr: VirtualAddress,
    intr_reg_sets_virt_addr: VirtualAddress,
    port_reg_sets_virt_addr: VirtualAddress,
    doorbell_reg_virt_addr: VirtualAddress,
    cmd_ring_virt_addr: VirtualAddress,
    primary_event_ring_virt_addr: VirtualAddress,
    device_context_arr_virt_addr: VirtualAddress,
    num_of_ports: usize,
    num_of_slots: usize,
    transfer_ring_buf: Option<RingBuffer>,
    primary_event_ring_buf: Option<RingBuffer>,
    cmd_ring_buf: Option<RingBuffer>,

    ports: Vec<Port>,
    devices: Vec<Device>,

    configuring_port_id: Option<usize>,
    root_hub_port_id: Option<usize>,
}

impl XhcDriver
{
    pub fn new() -> Result<Self, XhcDriverError>
    {
        let (class_code, subclass_code, prog_if) = PCI_USB_XHCI_ID;

        let pci_device_man = PCI_DEVICE_MAN.lock();
        let xhci_controllers = pci_device_man.find_by_class(class_code, subclass_code, prog_if);

        for device in xhci_controllers
        {
            if let Some(conf_space) = device.read_conf_space_non_bridge_field()
            {
                let bars = conf_space.get_bars();
                if bars.len() == 0
                {
                    continue;
                }

                let usb = XhcDriver {
                    is_init: false,
                    controller_pci_bus: device.bus,
                    controller_pci_device: device.device,
                    controller_pci_func: device.func,
                    cap_reg_virt_addr: VirtualAddress::new(0),
                    ope_reg_virt_addr: VirtualAddress::new(0),
                    runtime_reg_virt_addr: VirtualAddress::new(0),
                    intr_reg_sets_virt_addr: VirtualAddress::new(0),
                    port_reg_sets_virt_addr: VirtualAddress::new(0),
                    doorbell_reg_virt_addr: VirtualAddress::new(0),
                    cmd_ring_virt_addr: VirtualAddress::new(0),
                    device_context_arr_virt_addr: VirtualAddress::new(0),
                    primary_event_ring_virt_addr: VirtualAddress::new(0),
                    num_of_ports: 0,
                    num_of_slots: 0,
                    transfer_ring_buf: None,
                    primary_event_ring_buf: None,
                    cmd_ring_buf: None,
                    ports: Vec::new(),
                    devices: Vec::new(),
                    configuring_port_id: None,
                    root_hub_port_id: None,
                };

                info!(
                    "xhc: xHC device: {:?} - {}",
                    device.get_device_class(),
                    device.conf_space_header.get_device_name().unwrap()
                );

                return Ok(usb);
            }
        }

        return Err(XhcDriverError::XhcDeviceWasNotFoundError);
    }

    pub fn init(&mut self) -> Result<(), XhcDriverError>
    {
        let pci_device_man = PCI_DEVICE_MAN.lock();
        let controller = match pci_device_man.find_by_bdf(
            self.controller_pci_bus,
            self.controller_pci_device,
            self.controller_pci_func,
        )
        {
            Some(controller) => controller,
            None => return Err(XhcDriverError::XhcDeviceWasNotFoundError),
        };

        // read base address registers
        let conf_space_non_bridge_field = match controller.read_conf_space_non_bridge_field()
        {
            Some(field) => field,
            None => return Err(XhcDriverError::XhcDeviceWasNotFoundError),
        };

        let bars = conf_space_non_bridge_field.get_bars();

        if bars.len() == 0
        {
            return Err(XhcDriverError::XhcDeviceWasNotFoundError);
        }

        self.cap_reg_virt_addr = match bars[0].1
        {
            BaseAddress::MemoryAddress32BitSpace(addr, _) => addr,
            BaseAddress::MemoryAddress64BitSpace(addr, _) => addr,
            _ => return Err(XhcDriverError::XhcDeviceWasNotFoundError),
        }
        .get_virt_addr();

        if self.cap_reg_virt_addr.get() == 0
        {
            return Err(XhcDriverError::InvalidRegisterAddressError);
        }

        // set registers address
        let cap_reg = self.read_cap_reg();

        self.ope_reg_virt_addr = self.cap_reg_virt_addr.offset(cap_reg.cap_reg_length() as usize);
        self.runtime_reg_virt_addr =
            self.cap_reg_virt_addr.offset(cap_reg.runtime_reg_space_offset() as usize);
        self.intr_reg_sets_virt_addr =
            self.runtime_reg_virt_addr.offset(size_of::<RuntimeRegitsers>());
        self.port_reg_sets_virt_addr =
            self.ope_reg_virt_addr.offset(PORT_REG_SETS_START_VIRT_ADDR_OFFSET);
        self.doorbell_reg_virt_addr =
            self.cap_reg_virt_addr.offset(cap_reg.doorbell_offset() as usize);

        // TODO: request host controller ownership

        // stop controller
        let ope_reg = self.read_ope_reg();
        if !ope_reg.usb_status().hchalted()
        {
            return Err(XhcDriverError::HostControllerIsNotHaltedError);
        }

        // reset controller
        let mut ope_reg = self.read_ope_reg();
        let mut usb_cmd = ope_reg.usb_cmd();
        usb_cmd.set_host_controller_reset(true);
        ope_reg.set_usb_cmd(usb_cmd);
        self.write_ope_reg(ope_reg);

        loop
        {
            info!("xhc: Waiting xHC...");
            let ope_reg = self.read_ope_reg();
            if !ope_reg.usb_cmd().host_controller_reset()
                && !ope_reg.usb_status().controller_not_ready()
            {
                break;
            }
        }
        info!("xhc: Reset xHC");

        // set max device slots
        let cap_reg = self.read_cap_reg();
        self.num_of_ports = cap_reg.structural_params1().num_of_ports() as usize;
        self.num_of_slots = cap_reg.structural_params1().num_of_device_slots() as usize;
        let mut ope_reg = self.read_ope_reg();
        let mut conf_reg = ope_reg.configure();
        conf_reg.set_max_device_slots_enabled(self.num_of_slots as u8);
        ope_reg.set_configure(conf_reg);
        self.write_ope_reg(ope_reg);
        info!("xhc: Max ports: {}, Max slots: {}", self.num_of_ports, self.num_of_slots);

        // initialize scratchpad
        let cap_reg = self.read_cap_reg();
        let sp2 = cap_reg.structural_params2();
        let num_of_bufs =
            (sp2.max_scratchpad_bufs_high() << 5 | sp2.max_scratchpad_bufs_low()) as usize;
        let mut scratchpad_buf_arr_virt_addr = VirtualAddress::new(0);

        let scratchpad_buf_arr_mem_virt_addr = match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
        {
            Ok(mem_info) => mem_info,
            Err(err) => return Err(XhcDriverError::BitmapMemoryManagerError(err)),
        }
        .get_frame_start_virt_addr();

        let arr: &mut [u64] = scratchpad_buf_arr_mem_virt_addr.read_volatile();

        for i in 0..num_of_bufs
        {
            let mem_frame_info = match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                Ok(mem_info) => mem_info,
                Err(err) => return Err(XhcDriverError::BitmapMemoryManagerError(err)),
            };

            arr[i] = mem_frame_info.get_frame_start_virt_addr().get_phys_addr().get();
        }

        scratchpad_buf_arr_mem_virt_addr.write_volatile(arr);
        scratchpad_buf_arr_virt_addr = scratchpad_buf_arr_mem_virt_addr;

        // initialize device context
        self.device_context_arr_virt_addr = match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
        {
            Ok(mem_info) => mem_info,
            Err(err) => return Err(XhcDriverError::BitmapMemoryManagerError(err)),
        }
        .get_frame_start_virt_addr();

        // initialize device context array
        for i in 0..(self.num_of_slots + 1)
        {
            let entry = if i == 0 { scratchpad_buf_arr_virt_addr } else { VirtualAddress::new(0) };
            self.write_device_context_base_addr(i, entry).unwrap();
        }

        let mut ope_reg = self.read_ope_reg();
        ope_reg.set_device_context_base_addr_array_ptr(
            self.device_context_arr_virt_addr.get_phys_addr().get(),
        );
        self.write_ope_reg(ope_reg);
        info!("xhc: Initialized device context");

        // register command ring
        let pcs = true;

        let cmd_ring_mem_info = match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
        {
            Ok(mem_info) => mem_info,
            Err(err) => return Err(XhcDriverError::BitmapMemoryManagerError(err)),
        };

        self.cmd_ring_virt_addr = cmd_ring_mem_info.get_frame_start_virt_addr();
        self.cmd_ring_buf = match RingBuffer::new(
            cmd_ring_mem_info,
            RING_BUF_LEN,
            RingBufferType::CommandRing,
            pcs,
        )
        {
            Ok(ring_buf) => Some(ring_buf),
            Err(err) => return Err(XhcDriverError::RingBufferError(err)),
        };
        self.cmd_ring_buf.as_mut().unwrap().init();

        let mut crcr = CommandRingControlRegister::new();
        crcr.set_cmd_ring_ptr(
            cmd_ring_mem_info.get_frame_start_virt_addr().get_phys_addr().get() >> 6,
        );
        crcr.set_ring_cycle_state(pcs);
        crcr.set_cmd_stop(false);
        crcr.set_cmd_abort(false);
        let mut ope_reg = self.read_ope_reg();
        ope_reg.set_cmd_ring_ctrl(crcr);
        self.write_ope_reg(ope_reg);

        info!("xhc: Initialized command ring");

        // register event ring (primary)
        let primary_event_ring_seg_table_virt_addr =
            match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                Ok(mem_info) => mem_info,
                Err(err) => return Err(XhcDriverError::BitmapMemoryManagerError(err)),
            }
            .get_frame_start_virt_addr();

        let primary_event_ring_mem_info = match BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
        {
            Ok(mem_info) => mem_info,
            Err(err) => return Err(XhcDriverError::BitmapMemoryManagerError(err)),
        };

        self.primary_event_ring_virt_addr = primary_event_ring_mem_info.get_frame_start_virt_addr();

        // initialize event ring segment table entry
        let mut seg_table_entry =
            primary_event_ring_seg_table_virt_addr.read_volatile::<EventRingSegmentTableEntry>();
        seg_table_entry
            .set_ring_seg_base_addr(self.primary_event_ring_virt_addr.get_phys_addr().get());
        seg_table_entry.set_ring_seg_size(RING_BUF_LEN as u16);
        primary_event_ring_seg_table_virt_addr.write_volatile(seg_table_entry);

        // initialized event ring buffer (support only segment table length is 1)
        self.primary_event_ring_buf = match RingBuffer::new(
            primary_event_ring_mem_info,
            RING_BUF_LEN,
            RingBufferType::EventRing,
            pcs,
        )
        {
            Ok(ring_buf) => Some(ring_buf),
            Err(err) => return Err(XhcDriverError::RingBufferError(err)),
        };
        self.primary_event_ring_buf.as_mut().unwrap().init();

        // initialize first interrupter register sets entry
        let mut intr_reg_sets_0 = self.read_intr_reg_sets(0).unwrap();
        intr_reg_sets_0.set_event_ring_seg_table_base_addr(
            primary_event_ring_seg_table_virt_addr.get_phys_addr().get() >> 6,
        );
        intr_reg_sets_0.set_event_ring_seg_table_size(1);
        intr_reg_sets_0.set_dequeue_erst_seg_index(0);
        intr_reg_sets_0.set_event_ring_dequeue_ptr(
            self.primary_event_ring_virt_addr.get_phys_addr().get() >> 4,
        );
        self.write_intr_reg_sets(0, intr_reg_sets_0).unwrap();

        info!("xhc: Initialized event ring");

        // setting up msi
        let mut msg_addr = MsiMessageAddressField::new();
        msg_addr.set_destination_id(read_local_apic_id());
        msg_addr.set_redirection_hint_indication(0);
        msg_addr.set_destination_mode(0);
        msg_addr.set_const_0xfee(0xfee);

        let mut msg_data = MsiMessageDataField::new();
        msg_data.set_trigger_mode(TriggerMode::Level);
        msg_data.set_level(Level::Assert);
        msg_data.set_delivery_mode(DeliveryMode::Fixed);
        msg_data.set_vector(VEC_XHCI_INT as u8);

        match controller.set_msi_cap(msg_addr, msg_data)
        {
            Ok(_) => info!("xhc: Initialized MSI interrupt"),
            Err(err) => warn!("xhc: {}", err),
        }

        // enable interrupt
        let mut intr_reg_set_0 = self.read_intr_reg_sets(0).unwrap();
        intr_reg_set_0.set_int_mod_interval(4000);
        intr_reg_set_0.set_int_pending(true);
        intr_reg_set_0.set_int_enable(true);
        self.write_intr_reg_sets(0, intr_reg_set_0).unwrap();

        let mut ope_reg = self.read_ope_reg();
        let mut usb_cmd = ope_reg.usb_cmd();
        usb_cmd.set_intr_enable(true);
        ope_reg.set_usb_cmd(usb_cmd);
        self.write_ope_reg(ope_reg);

        self.is_init = true;
        info!("xhc: Initialized xHC driver");

        return Ok(());
    }

    pub fn start(&mut self) -> Result<(), XhcDriverError>
    {
        if !self.is_init
        {
            return Err(XhcDriverError::NotInitialized);
        }

        // start controller
        info!("xhc: Starting xHC...");
        let mut ope_reg = self.read_ope_reg();
        let mut usb_cmd = ope_reg.usb_cmd();
        usb_cmd.set_run_stop(true);
        ope_reg.set_usb_cmd(usb_cmd);
        self.write_ope_reg(ope_reg);

        loop
        {
            info!("xhc: Waiting xHC...");
            let ope_reg = self.read_ope_reg();
            if !ope_reg.usb_status().hchalted()
            {
                break;
            }
        }

        // check status
        let usb_status = self.read_ope_reg().usb_status();
        if usb_status.hchalted()
        {
            return Err(XhcDriverError::OtherError("xHC is halted"));
        }

        if usb_status.host_system_err()
        {
            return Err(XhcDriverError::OtherError("An error occured on the host system"));
        }

        if usb_status.host_controller_err()
        {
            return Err(XhcDriverError::OtherError("An error occured on xHC"));
        }

        self.ring_doorbell(0);

        return Ok(());
    }

    pub fn scan_ports(&mut self) -> Result<(), XhcDriverError>
    {
        if !self.is_init
        {
            return Err(XhcDriverError::NotInitialized);
        }

        if !self.is_running()
        {
            return Err(XhcDriverError::NotRunning);
        }

        self.ports = Vec::new();

        for i in 1..=self.num_of_ports
        {
            let port_reg_set = self.read_port_reg_set(i).unwrap();
            let sc_reg = port_reg_set.port_status_and_ctrl();
            if sc_reg.connect_status_change() && sc_reg.current_connect_status()
            {
                self.ports.push(Port::new(i));
                info!("xhc: Found connected port (port id: {})", i);
            }
        }

        return Ok(());
    }

    pub fn reset_port(&mut self, port_id: usize)
    {
        if let Some(port) = self.read_port(port_id)
        {
            let mut port = port.clone();
            let mut port_reg_set = self.read_port_reg_set(port_id).unwrap();
            let mut sc_reg = port_reg_set.port_status_and_ctrl();
            sc_reg.set_port_reset(true);
            sc_reg.set_connect_status_change(false);
            port_reg_set.set_port_status_and_ctrl(sc_reg);
            self.write_port_reg_set(port_id, port_reg_set).unwrap();

            loop
            {
                let port_reg_set = self.read_port_reg_set(port_id).unwrap();
                if !port_reg_set.port_status_and_ctrl().port_reset()
                {
                    break;
                }
            }

            port.config_state = ConfigState::Reset;
            self.write_port(port);

            info!("xhc: Reset port (port id: {})", port_id);

            self.configuring_port_id = Some(port_id);
        }
    }

    pub fn alloc_address_to_device(&mut self, port_id: usize)
    {
        if let Some(port) = self.read_port(port_id)
        {
            if port.config_state != ConfigState::Enabled
            {
                return;
            }

            // init input context
            if let Ok(input_context_mem_frame) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                let base_addr = input_context_mem_frame.get_frame_start_virt_addr();

                let mut port = port.clone();
                port.config_state = ConfigState::AddressingDevice;
                port.input_context_base_virt_addr = base_addr;
                self.write_port(port);

                self.configuring_port_id = Some(port_id);

                // initialize input control context
                let mut input_context = InputContext::new();
                input_context.input_ctrl_context.set_add_context_flag(0, true).unwrap();
                input_context.input_ctrl_context.set_add_context_flag(1, true).unwrap();

                let port_speed = self
                    .read_port_reg_set(self.root_hub_port_id.unwrap())
                    .unwrap()
                    .port_status_and_ctrl()
                    .port_speed();

                let max_packet_size = port_speed.get_max_packet_size();

                let mut slot_context = SlotContext::new();
                slot_context.set_speed(port_speed);
                slot_context.set_context_entries(1);
                slot_context.set_root_hub_port_num(self.root_hub_port_id.unwrap() as u8);

                input_context.device_context.slot_context = slot_context;

                let mut endpoint_context_0 = EndpointContext::new();
                endpoint_context_0.set_endpoint_type(EndpointType::ControlBidirectional);
                endpoint_context_0.set_max_packet_size(max_packet_size);
                endpoint_context_0.set_max_burst_size(0);
                endpoint_context_0.set_dequeue_cycle_state(true);
                endpoint_context_0.set_interval(0);
                endpoint_context_0.set_max_primary_streams(0);
                endpoint_context_0.set_mult(0);
                endpoint_context_0.set_error_cnt(3);

                input_context.device_context.endpoint_contexts[0] = endpoint_context_0;

                base_addr.write_volatile(input_context);

                let mut trb = TransferRequestBlock::new();
                trb.set_trb_type(TransferRequestBlockType::AddressDeviceCommand);
                trb.set_param(base_addr.get_phys_addr().get());
                trb.set_ctrl_regs((port.slot_id.unwrap() as u16) << 8);
                self.push_cmd_ring(trb).unwrap();
            }
            else
            {
                warn!("xhc: Failed to allocate memory frame for input context");
            }
        }
    }

    pub fn on_updated_event_ring(&mut self)
    {
        if let Some(trb) = self.pop_primary_event_ring()
        {
            match trb.trb_type()
            {
                TransferRequestBlockType::PortStatusChangeEvent =>
                {
                    // get root hub port id
                    self.root_hub_port_id = Some(trb.port_id().unwrap());

                    if let Some(port_id) = self.configuring_port_id
                    {
                        match self.read_port(port_id).unwrap().config_state
                        {
                            ConfigState::Reset =>
                            {
                                let mut trb = TransferRequestBlock::new();
                                trb.set_trb_type(TransferRequestBlockType::EnableSlotCommand);
                                self.push_cmd_ring(trb).unwrap();
                            }
                            _ => (),
                        }
                    }
                }
                TransferRequestBlockType::CommandCompletionEvent =>
                {
                    let comp_code = trb.completion_code().unwrap();
                    if comp_code != CompletionCode::Success
                    {
                        warn!("xhc: Failed to process command (completion code: {:?})", comp_code);
                        return;
                    }

                    if let (Some(port_id), Some(slot_id)) =
                        (self.configuring_port_id, trb.slot_id())
                    {
                        match self.read_port(port_id).unwrap().config_state
                        {
                            ConfigState::Reset =>
                            {
                                self.alloc_slot(port_id, slot_id);
                                self.configuring_port_id = None;
                            }
                            ConfigState::AddressingDevice =>
                            {
                                // TODO
                                self.devices.push(Device::new(slot_id));
                            }
                            _ => (),
                        }
                    }
                }
                _ => (),
            }
        }
    }

    pub fn is_init(&self) -> bool { return self.is_init; }

    pub fn is_running(&self) -> bool { return !self.read_ope_reg().usb_status().hchalted(); }

    fn alloc_slot(&mut self, port_id: usize, slot_id: usize)
    {
        if let Some(port) = self.read_port(port_id)
        {
            let mut port = port.clone();
            port.slot_id = Some(slot_id);
            port.config_state = ConfigState::Enabled;
            self.write_port(port);

            if let Ok(mem_frame_info) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                self.write_device_context_base_addr(
                    slot_id,
                    mem_frame_info.get_frame_start_virt_addr(),
                )
                .unwrap();
            }
            else
            {
                warn!("xhc: Failed to allocate memory frame for device context #{}", slot_id);
                return;
            }

            info!("xhc: Allocated slot: {} (port id: {})", slot_id, port_id);
        }
    }

    fn read_port(&self, port_id: usize) -> Option<&Port>
    {
        return self.ports.iter().find(|p| p.port_id() == port_id);
    }

    fn write_port(&mut self, port: Port)
    {
        if let Some(mut_port) = self.ports.iter_mut().find(|p| p.port_id() == port.port_id())
        {
            *mut_port = port;
        }
    }

    fn read_cap_reg(&self) -> CapabilityRegisters
    {
        return CapabilityRegisters::read(self.cap_reg_virt_addr);
    }

    fn read_ope_reg(&self) -> OperationalRegisters
    {
        return OperationalRegisters::read(self.ope_reg_virt_addr);
    }

    fn write_ope_reg(&self, mut ope_reg: OperationalRegisters)
    {
        ope_reg.write(self.ope_reg_virt_addr);
    }

    fn read_runtime_reg(&self) -> RuntimeRegitsers
    {
        return RuntimeRegitsers::read(self.runtime_reg_virt_addr);
    }

    fn write_runtime_reg(&self, mut runtime_reg: RuntimeRegitsers)
    {
        runtime_reg.write(self.runtime_reg_virt_addr);
    }

    fn read_intr_reg_sets(&self, index: usize) -> Option<InterrupterRegisterSet>
    {
        if index > INTR_REG_SET_MAX_LEN
        {
            return None;
        }

        let base_addr =
            self.intr_reg_sets_virt_addr.offset(index * size_of::<InterrupterRegisterSet>());
        return Some(InterrupterRegisterSet::read(base_addr));
    }

    fn write_intr_reg_sets(
        &self,
        index: usize,
        intr_reg_set: InterrupterRegisterSet,
    ) -> Result<(), XhcDriverError>
    {
        if index > INTR_REG_SET_MAX_LEN
        {
            return Err(XhcDriverError::InvalidInterrupterRegisterSetIndexError(index));
        }

        let mut intr_reg_set = intr_reg_set;

        let base_addr =
            self.intr_reg_sets_virt_addr.offset(index * size_of::<InterrupterRegisterSet>());
        intr_reg_set.write(base_addr);

        return Ok(());
    }

    fn read_port_reg_set(&self, index: usize) -> Option<PortRegisterSet>
    {
        if index == 0 || index > self.num_of_ports
        {
            return None;
        }

        let base_addr =
            self.port_reg_sets_virt_addr.offset((index - 1) * size_of::<PortRegisterSet>());
        return Some(PortRegisterSet::read(base_addr));
    }

    fn write_port_reg_set(
        &self,
        index: usize,
        port_reg_set: PortRegisterSet,
    ) -> Result<(), XhcDriverError>
    {
        if index == 0 || index > self.num_of_ports
        {
            return Err(XhcDriverError::InvalidPortRegisterSetIndexError(index));
        }

        let mut port_reg_set = port_reg_set;

        let base_addr =
            self.port_reg_sets_virt_addr.offset((index - 1) * size_of::<PortRegisterSet>());
        port_reg_set.write(base_addr);

        return Ok(());
    }

    fn read_doorbell_reg(&self, index: usize) -> Option<DoorbellRegister>
    {
        if index > DOORBELL_REG_MAX_LEN
        {
            return None;
        }

        let base_addr = self.doorbell_reg_virt_addr.offset(index * size_of::<DoorbellRegister>());
        return Some(DoorbellRegister::read(base_addr));
    }

    fn write_doorbell_reg(
        &self,
        index: usize,
        doorbell_reg: DoorbellRegister,
    ) -> Result<(), XhcDriverError>
    {
        if index > DOORBELL_REG_MAX_LEN
        {
            return Err(XhcDriverError::InvalidDoorbellRegisterIndexError(index));
        }

        let base_addr = self.doorbell_reg_virt_addr.offset(index * size_of::<DoorbellRegister>());
        doorbell_reg.write(base_addr);

        return Ok(());
    }

    fn read_device_context_base_addr(&self, index: usize) -> Option<VirtualAddress>
    {
        if index > self.num_of_slots + 1
        {
            return None;
        }

        let entry =
            self.device_context_arr_virt_addr.offset(index * size_of::<u64>()).read_volatile();
        let virt_addr = PhysicalAddress::new(entry).get_virt_addr();

        return Some(virt_addr);
    }

    fn write_device_context_base_addr(
        &self,
        index: usize,
        base_addr: VirtualAddress,
    ) -> Result<(), XhcDriverError>
    {
        if index > self.num_of_slots + 1
        {
            return Err(XhcDriverError::InvalidDeviceContextArrayIndexError(index));
        }

        self.device_context_arr_virt_addr
            .offset(index * size_of::<u64>())
            .write_volatile(base_addr.get_phys_addr().get());

        return Ok(());
    }

    fn read_device_context(&self, index: usize) -> Option<DeviceContext>
    {
        if let Some(base_addr) = self.read_device_context_base_addr(index)
        {
            return Some(base_addr.read_volatile());
        }

        return None;
    }

    fn ring_doorbell(&self, index: usize)
    {
        self.write_doorbell_reg(index, DoorbellRegister::new()).unwrap();
    }

    fn push_cmd_ring(&mut self, trb: TransferRequestBlock) -> Result<(), XhcDriverError>
    {
        match self.cmd_ring_buf.as_mut().unwrap().push(trb)
        {
            Ok(_) => self.ring_doorbell(0),
            Err(err) => return Err(XhcDriverError::RingBufferError(err)),
        }

        return Ok(());
    }

    fn pop_primary_event_ring(&self) -> Option<TransferRequestBlock>
    {
        let intr_reg_sets_0 = self.read_intr_reg_sets(0).unwrap();
        return match self.primary_event_ring_buf.as_ref().as_mut().unwrap().pop(intr_reg_sets_0)
        {
            Ok((trb, intr_reg_set)) =>
            {
                self.write_intr_reg_sets(0, intr_reg_set).unwrap();
                info!("xhc: Poped from event ring: {:?}", trb);
                Some(trb)
            }
            Err(err) =>
            {
                warn!("xhc: {:?}", err);
                None
            }
        };
    }
}
