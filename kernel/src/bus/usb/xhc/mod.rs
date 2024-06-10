use self::{
    context::{device::DeviceContext, endpoint::*, input::InputContext, slot::SlotContext},
    port::Port,
    ring_buffer::*,
    trb::*,
};
use super::device::*;
use crate::{
    arch::{addr::*, apic::local_apic_id, idt::VEC_XHCI_INT, register::msi::*},
    bus::{
        pci::{self, conf_space::BaseAddress, device_id::PCI_USB_XHCI_ID},
        usb::xhc::{port::ConfigState, register::*},
    },
    error::Result,
    mem::bitmap,
    util::mutex::{Mutex, MutexError},
};
use alloc::vec::Vec;
use core::mem::size_of;
use log::{error, info, warn};

pub mod context;
pub mod port;
pub mod register;
pub mod ring_buffer;
pub mod trb;

static mut XHC_DRIVER: Mutex<Option<XhcDriver>> = Mutex::new(None);

const PORT_REG_SETS_START_VIRT_ADDR_OFFSET: usize = 1024;
const RING_BUF_LEN: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum XhcDriverError {
    XhcDeviceWasNotFoundError,
    InvalidRegisterAddressError,
    InvalidInterrupterRegisterSetIndexError(usize),
    InvalidPortRegisterSetIndexError(usize),
    InvalidDoorbellRegisterIndexError(usize),
    InvalidDeviceContextArrayIndexError(usize),
    HostControllerIsNotHaltedError,
    OtherError(&'static str),
    NotInitialized,
    NotRunning,
    PortWasNotFoundError(usize),
    PortIsNotEnabledError(usize),
}

#[derive(Debug)]
pub struct XhcDriver {
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
    device_context_arr_virt_addr: VirtualAddress,
    num_of_ports: usize,
    num_of_slots: usize,
    primary_event_ring_buf: Option<RingBuffer<RING_BUF_LEN>>,
    cmd_ring_buf: Option<RingBuffer<RING_BUF_LEN>>,

    ports: Vec<Port>,

    configuring_port_id: Option<usize>,
    root_hub_port_id: Option<usize>,
}

impl XhcDriver {
    pub fn new() -> Result<Self> {
        let (class_code, subclass_code, prog_if) = PCI_USB_XHCI_ID;
        let mut device_bdf = None;

        pci::configure_devices(class_code, subclass_code, prog_if, |d| {
            let device_name = d.conf_space_header().get_device_name().unwrap();

            // TODO
            if (device_name.contains("xHCI") || device_name.contains("3.")) && device_bdf.is_none()
            {
                device_bdf = Some(d.device_bdf());
            }

            Ok(())
        })?;

        if let Some((bus, device, func)) = device_bdf {
            let usb = XhcDriver {
                is_init: false,
                controller_pci_bus: bus,
                controller_pci_device: device,
                controller_pci_func: func,
                cap_reg_virt_addr: VirtualAddress::default(),
                ope_reg_virt_addr: VirtualAddress::default(),
                runtime_reg_virt_addr: VirtualAddress::default(),
                intr_reg_sets_virt_addr: VirtualAddress::default(),
                port_reg_sets_virt_addr: VirtualAddress::default(),
                doorbell_reg_virt_addr: VirtualAddress::default(),
                device_context_arr_virt_addr: VirtualAddress::default(),
                num_of_ports: 0,
                num_of_slots: 0,
                primary_event_ring_buf: None,
                cmd_ring_buf: None,
                ports: Vec::new(),
                configuring_port_id: None,
                root_hub_port_id: None,
            };

            pci::configure_device(bus, device, func, |d| {
                info!(
                    "xhc: xHC device: {:?} - {}",
                    d.device_class(),
                    d.conf_space_header().get_device_name().unwrap()
                );

                Ok(())
            })?;

            return Ok(usb);
        }

        Err(XhcDriverError::XhcDeviceWasNotFoundError.into())
    }

    pub fn init(&mut self) -> Result<()> {
        // check pci device
        let is_exit_controller = pci::is_exit_device(
            self.controller_pci_bus,
            self.controller_pci_device,
            self.controller_pci_func,
        )?;
        if !is_exit_controller {
            return Err(XhcDriverError::XhcDeviceWasNotFoundError.into());
        }

        pci::configure_device(
            self.controller_pci_bus,
            self.controller_pci_device,
            self.controller_pci_func,
            |d| {
                // read base address registers
                let conf_space_non_bridge_field = d.read_conf_space_non_bridge_field()?;
                let bars = conf_space_non_bridge_field.get_bars()?;
                if bars.len() == 0 {
                    return Err(XhcDriverError::XhcDeviceWasNotFoundError.into());
                }

                self.cap_reg_virt_addr = match bars[0].1 {
                    BaseAddress::MemoryAddress32BitSpace(addr, _) => addr.get().into(),
                    BaseAddress::MemoryAddress64BitSpace(addr, _) => addr.get().into(),
                    _ => return Err(XhcDriverError::XhcDeviceWasNotFoundError.into()),
                };

                if self.cap_reg_virt_addr.get() == 0 {
                    return Err(XhcDriverError::InvalidRegisterAddressError.into());
                }

                // set registers address
                let cap_reg = self.read_cap_reg();

                self.ope_reg_virt_addr = self
                    .cap_reg_virt_addr
                    .offset(cap_reg.cap_reg_length as usize);
                self.runtime_reg_virt_addr = self
                    .cap_reg_virt_addr
                    .offset(cap_reg.runtime_reg_space_offset as usize);
                self.intr_reg_sets_virt_addr = self
                    .runtime_reg_virt_addr
                    .offset(size_of::<RuntimeRegitsers>());
                self.port_reg_sets_virt_addr = self
                    .ope_reg_virt_addr
                    .offset(PORT_REG_SETS_START_VIRT_ADDR_OFFSET);
                self.doorbell_reg_virt_addr = self
                    .cap_reg_virt_addr
                    .offset(cap_reg.doorbell_offset as usize);

                // TODO: request host controller ownership

                // stop controller
                if !self.read_ope_reg().usb_status.hchalted() {
                    return Err(XhcDriverError::HostControllerIsNotHaltedError.into());
                }

                // reset controller
                let mut ope_reg = self.read_ope_reg();
                ope_reg.usb_cmd.set_host_controller_reset(true);
                self.write_ope_reg(ope_reg);

                loop {
                    info!("xhc: Waiting xHC...");
                    let ope_reg = self.read_ope_reg();
                    if !ope_reg.usb_cmd.host_controller_reset()
                        && !ope_reg.usb_status.controller_not_ready()
                    {
                        break;
                    }
                }
                info!("xhc: Reset xHC");

                // set max device slots
                let cap_reg = self.read_cap_reg();
                self.num_of_ports = cap_reg.structural_params1.num_of_ports as usize;
                self.num_of_slots = cap_reg.structural_params1.num_of_device_slots as usize;
                let mut ope_reg = self.read_ope_reg();
                ope_reg
                    .configure
                    .set_max_device_slots_enabled(self.num_of_slots as u8);
                self.write_ope_reg(ope_reg);
                info!(
                    "xhc: Max ports: {}, Max slots: {}",
                    self.num_of_ports, self.num_of_slots
                );

                // initialize scratchpad
                // let cap_reg = self.read_cap_reg();
                // let sp2 = cap_reg.structural_params2();
                // let num_of_bufs =
                //     (sp2.max_scratchpad_bufs_high() << 5 | sp2.max_scratchpad_bufs_low()) as usize;

                // let scratchpad_buf_arr_virt_addr = match BITMAP_MEM_MAN.try_lock().unwrap().alloc_single_mem_frame() {
                //     Ok(mem_info) => mem_info,
                //     Err(err) => return Err(XhcDriverError::BitmapMemoryManagerError(err)),
                // }
                // .get_frame_start_virt_addr();

                // let arr: &mut [u64] = scratchpad_buf_arr_virt_addr.read_volatile();

                // for i in 0..num_of_bufs {
                //     let mem_frame_info = match BITMAP_MEM_MAN.try_lock().unwrap().alloc_single_mem_frame() {
                //         Ok(mem_info) => mem_info,
                //         Err(err) => return Err(XhcDriverError::BitmapMemoryManagerError(err)),
                //     };

                //     arr[i] = mem_frame_info.get_frame_start_phys_addr().get();
                // }

                // scratchpad_buf_arr_virt_addr.write_volatile(arr);

                // initialize device context
                let device_context_arr_mem_frame_info = bitmap::alloc_mem_frame(1)?;
                bitmap::mem_clear(&device_context_arr_mem_frame_info)?;
                self.device_context_arr_virt_addr =
                    device_context_arr_mem_frame_info.frame_start_virt_addr()?;

                // initialize device context array
                // for i in 0..(self.num_of_slots + 1) {
                //     let entry = if i == 0 {
                //         //scratchpad_buf_arr_virt_addr
                //         VirtualAddress::default()
                //     } else {
                //         VirtualAddress::default()
                //     };
                //     self.write_device_context_base_addr(i, entry)?;
                // }

                let mut ope_reg = self.read_ope_reg();
                ope_reg.device_context_base_addr_array_ptr =
                    self.device_context_arr_virt_addr.get_phys_addr()?.get();
                self.write_ope_reg(ope_reg);
                info!("xhc: Initialized device context");

                // register command ring
                let pcs = true;

                let mut cmd_ring_buf = RingBuffer::new(RingBufferType::CommandRing, pcs)?;
                cmd_ring_buf.set_link_trb()?;
                self.cmd_ring_buf = Some(cmd_ring_buf);

                let mut crcr = CommandRingControlRegister::default();
                crcr.set_cmd_ring_ptr(self.cmd_ring_buf.as_ref().unwrap().buf_ptr() as u64);
                crcr.set_ring_cycle_state(pcs);
                crcr.set_cmd_stop(false);
                crcr.set_cmd_abort(false);
                let mut ope_reg = self.read_ope_reg();
                ope_reg.cmd_ring_ctrl = crcr;
                self.write_ope_reg(ope_reg);

                info!("xhc: Initialized command ring");

                // register event ring (primary)
                let primary_event_ring_seg_table_virt_addr =
                    bitmap::alloc_mem_frame(1)?.frame_start_virt_addr()?;

                // initialized event ring buffer (support only segment table length is 1)
                let primary_event_ring_buf = RingBuffer::new(RingBufferType::EventRing, pcs)?;
                self.primary_event_ring_buf = Some(primary_event_ring_buf);

                // initialize event ring segment table entry
                let mut seg_table_entry = EventRingSegmentTableEntry::default();
                seg_table_entry.ring_seg_base_addr =
                    self.primary_event_ring_buf.as_ref().unwrap().buf_ptr() as u64;
                seg_table_entry.ring_seg_size = RING_BUF_LEN as u16;
                primary_event_ring_seg_table_virt_addr.write_volatile(seg_table_entry);

                // initialize first interrupter register sets entry
                let mut intr_reg_sets_0 = self.read_intr_reg_sets(0).unwrap();
                intr_reg_sets_0.set_event_ring_seg_table_base_addr(
                    primary_event_ring_seg_table_virt_addr
                        .get_phys_addr()
                        .unwrap()
                        .get(),
                );
                intr_reg_sets_0.set_event_ring_seg_table_size(1);
                intr_reg_sets_0.set_dequeue_erst_seg_index(0);
                intr_reg_sets_0.set_event_ring_dequeue_ptr(
                    self.primary_event_ring_buf.as_ref().unwrap().buf_ptr() as u64,
                );
                self.write_intr_reg_sets(0, intr_reg_sets_0).unwrap();

                info!("xhc: Initialized event ring");

                // setting up msi
                let msg_addr = MsiMessageAddressField::new(false, false, local_apic_id());
                let msg_data = MsiMessageDataField::new(
                    VEC_XHCI_INT as u8,
                    DeliveryMode::Fixed,
                    Level::Assert,
                    TriggerMode::Level,
                );

                match d.set_msi_cap(msg_addr, msg_data) {
                    Ok(_) => info!("xhc: Initialized MSI interrupt"),
                    Err(err) => warn!("xhc: {:?}", err),
                }

                // enable interrupt
                let mut intr_reg_set_0 = self.read_intr_reg_sets(0).unwrap();
                intr_reg_set_0.set_int_mod_interval(4000);
                intr_reg_set_0.set_int_pending(false);
                intr_reg_set_0.set_int_enable(true);
                self.write_intr_reg_sets(0, intr_reg_set_0)?;

                let mut ope_reg = self.read_ope_reg();
                ope_reg.usb_cmd.set_intr_enable(true);
                self.write_ope_reg(ope_reg);

                self.is_init = true;
                info!("xhc: Initialized xHC driver");

                Ok(())
            },
        )?;

        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        if !self.is_init {
            return Err(XhcDriverError::NotInitialized.into());
        }

        // start controller
        info!("xhc: Starting xHC...");
        let mut ope_reg = self.read_ope_reg();
        ope_reg.usb_cmd.set_run_stop(true);
        self.write_ope_reg(ope_reg);

        loop {
            info!("xhc: Waiting xHC...");
            if !self.read_ope_reg().usb_status.hchalted() {
                break;
            }
        }

        // check status
        let usb_status = self.read_ope_reg().usb_status;
        if usb_status.hchalted() {
            return Err(XhcDriverError::OtherError("xHC is halted").into());
        }

        if usb_status.host_system_err() {
            return Err(XhcDriverError::OtherError("An error occured on the host system").into());
        }

        if usb_status.host_controller_err() {
            return Err(XhcDriverError::OtherError("An error occured on xHC").into());
        }

        self.ring_doorbell(0, 0);

        Ok(())
    }

    pub fn scan_ports(&mut self) -> Result<Vec<usize>> {
        if !self.is_init {
            return Err(XhcDriverError::NotInitialized.into());
        }

        if !self.is_running() {
            return Err(XhcDriverError::NotRunning.into());
        }

        self.ports = Vec::new();
        let mut port_ids = Vec::new();

        for i in 1..=self.num_of_ports {
            let port_reg_set = self.read_port_reg_set(i).unwrap();
            let sc_reg = port_reg_set.port_status_and_ctrl;
            if sc_reg.connect_status_change() && sc_reg.current_connect_status() {
                self.ports.push(Port::new(i));
                port_ids.push(i);
                info!("xhc: Found connected port (port id: {})", i);
            }
        }

        Ok(port_ids)
    }

    pub fn reset_port(&mut self, port_id: usize) -> Result<()> {
        if !self.is_init {
            return Err(XhcDriverError::NotInitialized.into());
        }

        if !self.is_running() {
            return Err(XhcDriverError::NotRunning.into());
        }

        let port = match self.read_port(port_id) {
            Some(port) => port,
            None => return Err(XhcDriverError::PortWasNotFoundError(port_id).into()),
        };

        let mut port = port.clone();
        let mut port_reg_set = self.read_port_reg_set(port_id).unwrap();
        port_reg_set.port_status_and_ctrl.set_port_reset(true);
        port_reg_set
            .port_status_and_ctrl
            .set_connect_status_change(false);
        self.write_port_reg_set(port_id, port_reg_set).unwrap();

        loop {
            let port_reg_set = self.read_port_reg_set(port_id).unwrap();
            if !port_reg_set.port_status_and_ctrl.port_reset() {
                break;
            }
        }

        port.config_state = ConfigState::Reset;
        self.write_port(port);

        info!("xhc: Reset port (port id: {})", port_id);

        self.configuring_port_id = Some(port_id);

        Ok(())
    }

    pub fn alloc_address_to_device(&mut self, port_id: usize) -> Result<UsbDevice> {
        if !self.is_init {
            return Err(XhcDriverError::NotInitialized.into());
        }

        if !self.is_running() {
            return Err(XhcDriverError::NotRunning.into());
        }

        let port = match self.read_port(port_id) {
            Some(port) => port,
            None => return Err(XhcDriverError::PortWasNotFoundError(port_id).into()),
        };

        if port.config_state != ConfigState::Enabled {
            return Err(XhcDriverError::PortIsNotEnabledError(port_id).into());
        }

        let slot_id = port.slot_id.unwrap();

        let input_context_mem_frame_info = bitmap::alloc_mem_frame(1)?;
        bitmap::mem_clear(&input_context_mem_frame_info)?;
        let input_context_base_virt_addr = input_context_mem_frame_info.frame_start_virt_addr()?;

        let mut port = port.clone();
        port.config_state = ConfigState::AddressingDevice;
        port.input_context_base_virt_addr = input_context_base_virt_addr;
        self.write_port(port);

        self.configuring_port_id = Some(port_id);

        // initialize input control context
        let mut input_context = InputContext::default();
        input_context
            .input_ctrl_context
            .set_add_context_flag(0, true)
            .unwrap();
        input_context
            .input_ctrl_context
            .set_add_context_flag(1, true)
            .unwrap();

        let port_speed = self
            .read_port_reg_set(self.root_hub_port_id.unwrap())
            .unwrap()
            .port_status_and_ctrl
            .port_speed();

        let max_packet_size = port_speed.get_max_packet_size();

        let mut slot_context = SlotContext::default();
        slot_context.set_speed(port_speed);
        slot_context.set_context_entries(1);
        slot_context.set_root_hub_port_num(self.root_hub_port_id.unwrap() as u8);

        input_context.device_context.slot_context = slot_context;

        let mut endpoint_context_0 = EndpointContext::default();
        endpoint_context_0.set_endpoint_type(EndpointType::ControlBidirectional);
        endpoint_context_0.set_max_packet_size(max_packet_size);
        endpoint_context_0.set_max_burst_size(0);
        endpoint_context_0.set_dequeue_cycle_state(true);
        endpoint_context_0.set_interval(0);
        endpoint_context_0.set_max_primary_streams(0);
        endpoint_context_0.set_mult(0);
        endpoint_context_0.set_error_cnt(3);

        let transfer_ring_buf = RingBuffer::new(RingBufferType::TransferRing, true)?;

        endpoint_context_0.set_tr_dequeue_ptr(transfer_ring_buf.buf_ptr() as u64);
        input_context.device_context.endpoint_contexts[0] = endpoint_context_0;
        input_context_base_virt_addr.write_volatile(input_context);

        let mut trb = TransferRequestBlock::default();
        trb.set_trb_type(TransferRequestBlockType::AddressDeviceCommand);
        trb.param = input_context_base_virt_addr.get_phys_addr().unwrap().get();
        trb.ctrl_regs = (slot_id as u16) << 8;
        self.push_cmd_ring(trb).unwrap();

        return UsbDevice::new(slot_id, max_packet_size, transfer_ring_buf);
    }

    pub fn on_updated_event_ring(&mut self) {
        if !self.is_init {
            return;
        }

        let trb = match self.pop_primary_event_ring() {
            Some(trb) => trb,
            None => return,
        };

        match trb.trb_type() {
            TransferRequestBlockType::PortStatusChangeEvent => {
                // get root hub port id
                self.root_hub_port_id = Some(trb.port_id().unwrap());

                if let Some(port_id) = self.configuring_port_id {
                    match self.read_port(port_id).unwrap().config_state {
                        ConfigState::Reset => {
                            let mut trb = TransferRequestBlock::default();
                            trb.set_trb_type(TransferRequestBlockType::EnableSlotCommand);
                            self.push_cmd_ring(trb).unwrap();
                        }
                        _ => (),
                    }
                }
            }
            TransferRequestBlockType::CommandCompletionEvent => {
                let comp_code = trb.completion_code().unwrap();
                if comp_code != CompletionCode::Success {
                    warn!(
                        "xhc: Failed to process command (completion code: {:?})",
                        comp_code
                    );

                    // TODO
                    if comp_code == CompletionCode::TrbError {
                        self.is_init = false;
                    }

                    return;
                }

                if let (Some(port_id), Some(slot_id)) = (self.configuring_port_id, trb.slot_id()) {
                    match self.read_port(port_id).unwrap().config_state {
                        ConfigState::Reset => {
                            if let Err(err) = self.alloc_slot(port_id, slot_id) {
                                warn!("xhc: {:?}", err);
                                return;
                            }
                            self.configuring_port_id = None;
                        }
                        ConfigState::AddressingDevice => {
                            let mut port = self.read_port(port_id).unwrap().clone();
                            port.config_state = ConfigState::InitializingDevice;
                            self.write_port(port);
                            self.configuring_port_id = None;
                        }
                        _ => (),
                    }
                }
            }
            TransferRequestBlockType::TransferEvent => {
                let comp_code = trb.completion_code().unwrap();
                if comp_code != CompletionCode::Success {
                    warn!(
                        "xhc: Might have been failed to process command (completion code: {:?})",
                        comp_code
                    );

                    // TODO
                    if comp_code == CompletionCode::TrbError {
                        self.is_init = false;
                    }

                    return;
                }

                let slot_id = trb.slot_id().unwrap();
                let endpoint_id = trb.endpoint_id().unwrap();

                //info!("slot id: {}, endpoint id: {}", slot_id, endpoint_id);

                if let Some(mut device) = super::find_device_by_slot_id(slot_id) {
                    if !device.is_configured {
                        return;
                    }

                    device.update(endpoint_id, trb);

                    if super::update_device(device).is_ok() {
                        self.ring_doorbell(slot_id, endpoint_id as u8);
                    } else {
                        error!("xhc: Failed to update USB device");
                    }
                }
            }
            TransferRequestBlockType::HostControllerEvent => {
                let comp_code = trb.completion_code().unwrap();
                if comp_code != CompletionCode::Success {
                    warn!(
                        "xhc: Might have been failed to process command (completion code: {:?})",
                        comp_code
                    );

                    // TODO
                    // if comp_code == CompletionCode::TrbError {
                    //     self.is_init = false;
                    // }

                    return;
                }
            }
            _ => (),
        }
    }

    pub fn is_running(&self) -> bool {
        !self.read_ope_reg().usb_status.hchalted()
    }

    pub fn find_port_by_slot_id(&self, slot_id: usize) -> Option<Port> {
        self.ports
            .iter()
            .find(|p| p.slot_id == Some(slot_id))
            .map(|p| p.clone())
    }

    fn alloc_slot(&mut self, port_id: usize, slot_id: usize) -> Result<()> {
        let port = match self.read_port(port_id) {
            Some(port) => port,
            None => return Err(XhcDriverError::PortWasNotFoundError(port_id).into()),
        };

        let device_context_mem_frame_info = bitmap::alloc_mem_frame(1)?;
        bitmap::mem_clear(&device_context_mem_frame_info)?;
        let device_context_base_virt_addr =
            device_context_mem_frame_info.frame_start_virt_addr()?;

        let mut port = port.clone();
        port.slot_id = Some(slot_id);
        port.config_state = ConfigState::Enabled;
        port.output_context_base_virt_addr = device_context_base_virt_addr;
        self.write_port(port);

        self.write_device_context_base_addr(
            slot_id,
            device_context_base_virt_addr.get_phys_addr()?,
        )?;
        info!("xhc: Allocated slot: {} (port id: {})", slot_id, port_id);

        Ok(())
    }

    fn read_port(&self, port_id: usize) -> Option<&Port> {
        self.ports.iter().find(|p| p.port_id() == port_id)
    }

    fn write_port(&mut self, port: Port) {
        if let Some(mut_port) = self
            .ports
            .iter_mut()
            .find(|p| p.port_id() == port.port_id())
        {
            *mut_port = port;
        }
    }

    fn read_cap_reg(&self) -> CapabilityRegisters {
        CapabilityRegisters::read(self.cap_reg_virt_addr)
    }

    fn read_ope_reg(&self) -> OperationalRegisters {
        OperationalRegisters::read(self.ope_reg_virt_addr)
    }

    fn write_ope_reg(&self, mut ope_reg: OperationalRegisters) {
        ope_reg.write(self.ope_reg_virt_addr);
    }

    fn read_runtime_reg(&self) -> RuntimeRegitsers {
        RuntimeRegitsers::read(self.runtime_reg_virt_addr)
    }

    fn write_runtime_reg(&self, runtime_reg: RuntimeRegitsers) {
        runtime_reg.write(self.runtime_reg_virt_addr);
    }

    fn read_intr_reg_sets(&self, index: usize) -> Option<InterrupterRegisterSet> {
        if index > INTR_REG_SET_MAX_LEN {
            return None;
        }

        let base_addr = self
            .intr_reg_sets_virt_addr
            .offset(index * size_of::<InterrupterRegisterSet>());
        Some(InterrupterRegisterSet::read(base_addr))
    }

    fn write_intr_reg_sets(
        &self,
        index: usize,
        intr_reg_set: InterrupterRegisterSet,
    ) -> Result<()> {
        if index > INTR_REG_SET_MAX_LEN {
            return Err(XhcDriverError::InvalidInterrupterRegisterSetIndexError(index).into());
        }

        let read = self.read_intr_reg_sets(index).unwrap();
        let update_seg_table =
            intr_reg_set.event_ring_seg_table_base_addr() != read.event_ring_seg_table_base_addr();

        let mut intr_reg_set = intr_reg_set;

        let base_addr = self
            .intr_reg_sets_virt_addr
            .offset(index * size_of::<InterrupterRegisterSet>());

        intr_reg_set.write(base_addr, update_seg_table);

        Ok(())
    }

    fn read_port_reg_set(&self, index: usize) -> Option<PortRegisterSet> {
        if index == 0 || index > self.num_of_ports {
            return None;
        }

        let base_addr = self
            .port_reg_sets_virt_addr
            .offset((index - 1) * size_of::<PortRegisterSet>());
        Some(PortRegisterSet::read(base_addr))
    }

    fn write_port_reg_set(&self, index: usize, port_reg_set: PortRegisterSet) -> Result<()> {
        if index == 0 || index > self.num_of_ports {
            return Err(XhcDriverError::InvalidPortRegisterSetIndexError(index).into());
        }

        let mut port_reg_set = port_reg_set;

        let base_addr = self
            .port_reg_sets_virt_addr
            .offset((index - 1) * size_of::<PortRegisterSet>());
        port_reg_set.write(base_addr);

        Ok(())
    }

    fn write_doorbell_reg(&self, index: usize, doorbell_reg: DoorbellRegister) -> Result<()> {
        if index > DOORBELL_REG_MAX_LEN {
            return Err(XhcDriverError::InvalidDoorbellRegisterIndexError(index).into());
        }

        let base_addr = self
            .doorbell_reg_virt_addr
            .offset(index * size_of::<DoorbellRegister>());
        doorbell_reg.write(base_addr);

        Ok(())
    }

    fn read_device_context_base_addr(&self, index: usize) -> Option<VirtualAddress> {
        if index > self.num_of_slots + 1 {
            return None;
        }

        let entry: u64 = self
            .device_context_arr_virt_addr
            .offset(index * size_of::<u64>())
            .read_volatile();

        Some(entry.into())
    }

    fn write_device_context_base_addr(
        &self,
        index: usize,
        phys_addr: PhysicalAddress,
    ) -> Result<()> {
        if index > self.num_of_slots + 1 {
            return Err(XhcDriverError::InvalidDeviceContextArrayIndexError(index).into());
        }

        self.device_context_arr_virt_addr
            .offset(index * size_of::<u64>())
            .write_volatile(phys_addr.get());

        Ok(())
    }

    pub fn read_device_context(&self, slot_id: usize) -> Option<DeviceContext> {
        if let Some(base_addr) = self.read_device_context_base_addr(slot_id) {
            return Some(base_addr.read_volatile());
        }

        None
    }

    pub fn ring_doorbell(&self, index: usize, value: u8) {
        let mut doorbell_reg = DoorbellRegister::default();
        doorbell_reg.db_target = value;
        self.write_doorbell_reg(index, doorbell_reg).unwrap();
    }

    pub fn push_cmd_ring(&mut self, trb: TransferRequestBlock) -> Result<()> {
        match self.cmd_ring_buf.as_mut().unwrap().push(trb) {
            Ok(_) => self.ring_doorbell(0, 0),
            Err(err) => return Err(err),
        }

        Ok(())
    }

    fn pop_primary_event_ring(&mut self) -> Option<TransferRequestBlock> {
        let mut intr_reg_sets_0 = self.read_intr_reg_sets(0)?;
        match self
            .primary_event_ring_buf
            .as_mut()
            .unwrap()
            .pop(&mut intr_reg_sets_0)
        {
            Ok(trb) => {
                self.write_intr_reg_sets(0, intr_reg_sets_0).unwrap();
                Some(trb)
            }
            Err(err) => {
                warn!("xhc: {:?}", err);
                None
            }
        }
    }
}

pub fn init() -> Result<()> {
    if let Ok(mut xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        *xhc_driver = match XhcDriver::new() {
            Ok(mut d) => {
                d.init()?;
                Some(d)
            }
            Err(e) => return Err(e),
        };

        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn start() -> Result<()> {
    if let Ok(mut xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        return xhc_driver
            .as_mut()
            .ok_or(XhcDriverError::NotInitialized)?
            .start();
    }

    Err(MutexError::Locked.into())
}
pub fn find_port_by_slot_id(slot_id: usize) -> Option<Port> {
    if let Ok(mut xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        return xhc_driver.as_mut()?.find_port_by_slot_id(slot_id);
    }

    None
}

pub fn read_device_context(slot_id: usize) -> Option<DeviceContext> {
    if let Ok(xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        return xhc_driver.as_ref()?.read_device_context(slot_id);
    }

    None
}

pub fn push_cmd_ring(trb: TransferRequestBlock) -> Result<()> {
    if let Ok(mut xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        return xhc_driver
            .as_mut()
            .ok_or(XhcDriverError::NotInitialized)?
            .push_cmd_ring(trb);
    }

    Err(MutexError::Locked.into())
}

pub fn ring_doorbell(index: usize, value: u8) -> Result<()> {
    if let Ok(xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        xhc_driver
            .as_ref()
            .ok_or(XhcDriverError::NotInitialized)?
            .ring_doorbell(index, value);
        return Ok(());
    }

    Err(MutexError::Locked.into())
}

pub fn scan_ports() -> Result<Vec<usize>> {
    if let Ok(mut xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        return xhc_driver
            .as_mut()
            .ok_or(XhcDriverError::NotInitialized)?
            .scan_ports();
    }

    Err(MutexError::Locked.into())
}

pub fn reset_port(port_id: usize) -> Result<()> {
    if let Ok(mut xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        return xhc_driver
            .as_mut()
            .ok_or(XhcDriverError::NotInitialized)?
            .reset_port(port_id);
    }

    Err(MutexError::Locked.into())
}

pub fn alloc_address_to_device(port_id: usize) -> Result<UsbDevice> {
    if let Ok(mut xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        return xhc_driver
            .as_mut()
            .ok_or(XhcDriverError::NotInitialized)?
            .alloc_address_to_device(port_id);
    }

    Err(MutexError::Locked.into())
}

pub fn on_updated_event_ring() -> Result<()> {
    if let Ok(mut xhc_driver) = unsafe { XHC_DRIVER.try_lock() } {
        xhc_driver
            .as_mut()
            .ok_or(XhcDriverError::NotInitialized)?
            .on_updated_event_ring();
        return Ok(());
    }

    Err(MutexError::Locked.into())
}
