use super::{
    bus::{device::UsbDevice, find_device_by_slot_id, update_device},
    trb::*,
};
use crate::{
    addr::{PhysicalAddress, VirtualAddress},
    apic,
    device::{self, pci_bus::conf_space::BaseAddress, DeviceDriverFunction, DeviceDriverInfo},
    error::{Error, Result},
    idt,
    mem::bitmap,
    register::msi::*,
    util::mutex::Mutex,
};
use alloc::vec::Vec;
use context::{
    device::DeviceContext,
    endpoint::{EndpointContext, EndpointType},
    input::InputContext,
    slot::SlotContext,
};
use core::mem::size_of;
use log::{debug, error, info, warn};
use port::{ConfigState, Port};
use register::*;
use ringbuf::*;

pub mod context;
pub mod port;
pub mod register;
pub mod ringbuf;

static mut XHC_DRIVER: Mutex<XhcDriver> = Mutex::new(XhcDriver::new());

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
    NotInitialized,
    NotRunning,
    PortWasNotFoundError(usize),
    PortIsNotEnabledError(usize),
}

struct XhcDriver {
    device_driver_info: DeviceDriverInfo,
    pci_device_bdf: Option<(usize, usize, usize)>,

    cap_reg_virt_addr: Option<VirtualAddress>,
    ope_reg_virt_addr: Option<VirtualAddress>,
    runtime_reg_virt_addr: Option<VirtualAddress>,
    intr_reg_sets_virt_addr: Option<VirtualAddress>,
    port_reg_sets_virt_addr: Option<VirtualAddress>,
    doorbell_reg_virt_addr: Option<VirtualAddress>,
    device_context_arr_virt_addr: Option<VirtualAddress>,
    num_of_ports: Option<usize>,
    num_of_slots: Option<usize>,
    primary_event_ring_buf: Option<RingBuffer<RING_BUF_LEN>>,
    cmd_ring_buf: Option<RingBuffer<RING_BUF_LEN>>,

    ports: Vec<Port>,

    configuring_port_id: Option<usize>,
    root_hub_port_id: Option<usize>,
}

impl XhcDriver {
    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("xhc"),
            pci_device_bdf: None,
            cap_reg_virt_addr: None,
            ope_reg_virt_addr: None,
            runtime_reg_virt_addr: None,
            intr_reg_sets_virt_addr: None,
            port_reg_sets_virt_addr: None,
            doorbell_reg_virt_addr: None,
            device_context_arr_virt_addr: None,
            num_of_ports: None,
            num_of_slots: None,
            primary_event_ring_buf: None,
            cmd_ring_buf: None,
            ports: Vec::new(),
            configuring_port_id: None,
            root_hub_port_id: None,
        }
    }

    fn start(&mut self) -> Result<()> {
        let DeviceDriverInfo { name, attached } = self.device_driver_info;

        if !attached {
            return Err(XhcDriverError::NotInitialized.into());
        }

        // start controller
        info!("{}: Starting xHC...", name);
        let mut ope_reg = self.read_ope_reg();
        ope_reg.usb_cmd.set_run_stop(true);
        self.write_ope_reg(ope_reg);

        loop {
            info!("{}: Waiting xHC...", name);
            if !self.read_ope_reg().usb_status.hchalted() {
                break;
            }
        }

        // check status
        let usb_status = self.read_ope_reg().usb_status;
        if usb_status.hchalted() {
            return Err(Error::Failed("Failed to start xHC"));
        }

        if usb_status.host_system_err() {
            return Err(Error::Failed("An error occured on the host system"));
        }

        if usb_status.host_controller_err() {
            return Err(Error::Failed("An error occured on xHC"));
        }

        self.ring_doorbell(0, 0);

        Ok(())
    }

    fn scan_ports(&mut self) -> Result<Vec<usize>> {
        let DeviceDriverInfo { name, attached } = self.device_driver_info;

        if !attached {
            return Err(XhcDriverError::NotInitialized.into());
        }

        if !self.is_running() {
            return Err(XhcDriverError::NotRunning.into());
        }

        self.ports = Vec::new();
        let mut port_ids = Vec::new();

        for i in 1..=self.num_of_ports.unwrap() {
            let port_reg_set = self.read_port_reg_set(i).unwrap();
            let sc_reg = port_reg_set.port_status_and_ctrl;
            if sc_reg.connect_status_change() && sc_reg.current_connect_status() {
                self.ports.push(Port::new(i));
                port_ids.push(i);
                info!("{}: Found connected port (port id: {})", name, i);
            }
        }

        Ok(port_ids)
    }

    fn reset_port(&mut self, port_id: usize) -> Result<()> {
        let DeviceDriverInfo { name, attached } = self.device_driver_info;

        if !attached {
            return Err(XhcDriverError::NotInitialized.into());
        }

        if !self.is_running() {
            return Err(XhcDriverError::NotRunning.into());
        }

        let port = self
            .read_port(port_id)
            .ok_or(XhcDriverError::PortWasNotFoundError(port_id))?;

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

        info!("{}: Reset port (port id: {})", name, port_id);

        self.configuring_port_id = Some(port_id);

        Ok(())
    }

    fn alloc_address_to_device(&mut self, port_id: usize) -> Result<UsbDevice> {
        if !self.device_driver_info.attached {
            return Err(XhcDriverError::NotInitialized.into());
        }

        if !self.is_running() {
            return Err(XhcDriverError::NotRunning.into());
        }

        let port = self
            .read_port(port_id)
            .ok_or(XhcDriverError::PortWasNotFoundError(port_id))?;

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

    fn is_running(&self) -> bool {
        !self.read_ope_reg().usb_status.hchalted()
    }

    fn find_port_by_slot_id(&self, slot_id: usize) -> Option<Port> {
        self.ports
            .iter()
            .find(|p| p.slot_id == Some(slot_id))
            .map(|p| p.clone())
    }

    fn alloc_slot(&mut self, port_id: usize, slot_id: usize) -> Result<()> {
        let port = self
            .read_port(port_id)
            .ok_or(XhcDriverError::PortWasNotFoundError(port_id))?;

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
        info!(
            "{}: Allocated slot: {} (port id: {})",
            self.device_driver_info.name, slot_id, port_id
        );

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
        CapabilityRegisters::read(self.cap_reg_virt_addr.unwrap())
    }

    fn read_ope_reg(&self) -> OperationalRegisters {
        OperationalRegisters::read(self.ope_reg_virt_addr.unwrap())
    }

    fn write_ope_reg(&self, mut ope_reg: OperationalRegisters) {
        ope_reg.write(self.ope_reg_virt_addr.unwrap());
    }

    fn read_runtime_reg(&self) -> RuntimeRegitsers {
        RuntimeRegitsers::read(self.runtime_reg_virt_addr.unwrap())
    }

    fn write_runtime_reg(&self, runtime_reg: RuntimeRegitsers) {
        runtime_reg.write(self.runtime_reg_virt_addr.unwrap());
    }

    fn read_intr_reg_sets(&self, index: usize) -> Option<InterrupterRegisterSet> {
        if index > INTR_REG_SET_MAX_LEN {
            return None;
        }

        let base_addr = self
            .intr_reg_sets_virt_addr
            .unwrap()
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
            .unwrap()
            .offset(index * size_of::<InterrupterRegisterSet>());

        intr_reg_set.write(base_addr, update_seg_table);

        Ok(())
    }

    fn read_port_reg_set(&self, index: usize) -> Option<PortRegisterSet> {
        if index == 0 || index > self.num_of_ports? {
            return None;
        }

        let base_addr = self
            .port_reg_sets_virt_addr?
            .offset((index - 1) * size_of::<PortRegisterSet>());
        Some(PortRegisterSet::read(base_addr))
    }

    fn write_port_reg_set(&self, index: usize, port_reg_set: PortRegisterSet) -> Result<()> {
        if index == 0 || index > self.num_of_ports.unwrap() {
            return Err(XhcDriverError::InvalidPortRegisterSetIndexError(index).into());
        }

        let mut port_reg_set = port_reg_set;

        let base_addr = self
            .port_reg_sets_virt_addr
            .unwrap()
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
            .unwrap()
            .offset(index * size_of::<DoorbellRegister>());
        doorbell_reg.write(base_addr);

        Ok(())
    }

    fn read_device_context_base_addr(&self, index: usize) -> Option<VirtualAddress> {
        if index > self.num_of_slots? + 1 {
            return None;
        }

        let entry: u64 = self
            .device_context_arr_virt_addr?
            .offset(index * size_of::<u64>())
            .read_volatile();

        Some(entry.into())
    }

    fn write_device_context_base_addr(
        &self,
        index: usize,
        phys_addr: PhysicalAddress,
    ) -> Result<()> {
        if index > self.num_of_slots.unwrap() + 1 {
            return Err(XhcDriverError::InvalidDeviceContextArrayIndexError(index).into());
        }

        self.device_context_arr_virt_addr
            .unwrap()
            .offset(index * size_of::<u64>())
            .write_volatile(phys_addr.get());

        Ok(())
    }

    fn read_device_context(&self, slot_id: usize) -> Option<DeviceContext> {
        if let Some(base_addr) = self.read_device_context_base_addr(slot_id) {
            return Some(base_addr.read_volatile());
        }

        None
    }

    fn ring_doorbell(&self, index: usize, value: u8) {
        let mut doorbell_reg = DoorbellRegister::default();
        doorbell_reg.db_target = value;
        self.write_doorbell_reg(index, doorbell_reg).unwrap();
    }

    fn push_cmd_ring(&mut self, trb: TransferRequestBlock) -> Result<()> {
        self.cmd_ring_buf.as_mut().unwrap().push(trb)?;
        self.ring_doorbell(0, 0);
        Ok(())
    }

    fn pop_primary_event_ring(&mut self) -> Option<TransferRequestBlock> {
        let mut intr_reg_sets_0 = self.read_intr_reg_sets(0)?;
        match self
            .primary_event_ring_buf
            .as_mut()?
            .pop(&mut intr_reg_sets_0)
        {
            Ok(trb) => {
                self.write_intr_reg_sets(0, intr_reg_sets_0).unwrap();
                Some(trb)
            }
            Err(err) => {
                warn!("{}: {:?}", self.device_driver_info.name, err);
                None
            }
        }
    }
}

impl DeviceDriverFunction for XhcDriver {
    type AttachInput = ();
    type PollNormalOutput = ();
    type PollInterruptOutput = ();

    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(self.device_driver_info.clone())
    }

    fn probe(&mut self) -> Result<()> {
        device::pci_bus::find_devices(0x0c, 0x03, 0x30, |d| {
            let device_name = d.conf_space_header().get_device_name().unwrap();

            if device_name.contains("xHCI") || device_name.contains("3.") {
                self.pci_device_bdf = Some(d.bdf());
            }

            Ok(())
        })?;

        Ok(())
    }

    fn attach(&mut self, _arg: Self::AttachInput) -> Result<()> {
        if self.pci_device_bdf.is_none() {
            return Err(Error::Failed("Device driver is not probed"));
        }

        let driver_name = self.device_driver_info.name;

        let (bus, device, func) = self.pci_device_bdf.unwrap();
        device::pci_bus::configure_device(bus, device, func, |d| {
            // read base address registers
            let conf_space_non_bridge_field = d.read_conf_space_non_bridge_field()?;
            let bars = conf_space_non_bridge_field.get_bars()?;
            if bars.len() == 0 {
                return Err(XhcDriverError::XhcDeviceWasNotFoundError.into());
            }

            self.cap_reg_virt_addr = match bars[0].1 {
                BaseAddress::MemoryAddress32BitSpace(addr, _) => Some(addr.get_virt_addr()?),
                BaseAddress::MemoryAddress64BitSpace(addr, _) => Some(addr.get_virt_addr()?),
                _ => return Err(XhcDriverError::InvalidRegisterAddressError.into()),
            };

            if self.cap_reg_virt_addr.unwrap().get() == 0 {
                return Err(XhcDriverError::InvalidRegisterAddressError.into());
            }

            // set registers address
            let cap_reg = self.read_cap_reg();

            self.ope_reg_virt_addr = Some(
                self.cap_reg_virt_addr
                    .unwrap()
                    .offset(cap_reg.cap_reg_length as usize),
            );
            self.runtime_reg_virt_addr = Some(
                self.cap_reg_virt_addr
                    .unwrap()
                    .offset(cap_reg.runtime_reg_space_offset as usize),
            );
            self.intr_reg_sets_virt_addr = Some(
                self.runtime_reg_virt_addr
                    .unwrap()
                    .offset(size_of::<RuntimeRegitsers>()),
            );
            self.port_reg_sets_virt_addr = Some(
                self.ope_reg_virt_addr
                    .unwrap()
                    .offset(PORT_REG_SETS_START_VIRT_ADDR_OFFSET),
            );
            self.doorbell_reg_virt_addr = Some(
                self.cap_reg_virt_addr
                    .unwrap()
                    .offset(cap_reg.doorbell_offset as usize),
            );

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
                info!("{}: Waiting xHC...", driver_name);
                let ope_reg = self.read_ope_reg();
                if !ope_reg.usb_cmd.host_controller_reset()
                    && !ope_reg.usb_status.controller_not_ready()
                {
                    break;
                }
            }
            info!("{}: Reset xHC", driver_name);

            // set max device slots
            let cap_reg = self.read_cap_reg();
            self.num_of_ports = Some(cap_reg.structural_params1.num_of_ports as usize);
            self.num_of_slots = Some(cap_reg.structural_params1.num_of_device_slots as usize);
            let mut ope_reg = self.read_ope_reg();
            ope_reg
                .configure
                .set_max_device_slots_enabled(self.num_of_slots.unwrap() as u8);
            self.write_ope_reg(ope_reg);
            debug!(
                "{}: Max ports: {}, Max slots: {}",
                driver_name,
                self.num_of_ports.unwrap(),
                self.num_of_slots.unwrap()
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
                Some(device_context_arr_mem_frame_info.frame_start_virt_addr()?);

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
            ope_reg.device_context_base_addr_array_ptr = self
                .device_context_arr_virt_addr
                .unwrap()
                .get_phys_addr()?
                .get();
            self.write_ope_reg(ope_reg);
            info!("{}: Device context initialized", driver_name);

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

            info!("{}: Command ring initialized", driver_name);

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

            info!("{}: Event ring initialized", driver_name);

            // setting up msi
            let vec_num = idt::set_handler_dyn_vec(
                idt::InterruptHandler::Normal(poll_int_xhc_driver),
                idt::GateType::Interrupt,
            )?;

            let msg_addr = MsiMessageAddressField::new(false, false, apic::local_apic_id());
            let msg_data = MsiMessageDataField::new(
                vec_num,
                DeliveryMode::Fixed,
                Level::Assert,
                TriggerMode::Level,
            );

            match d.set_msi_cap(msg_addr, msg_data) {
                Ok(_) => info!("{}: MSI interrupt initialized", driver_name),
                Err(err) => warn!("{}: {:?}", driver_name, err),
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

            Ok(())
        })?;

        self.device_driver_info.attached = true;
        Ok(())
    }

    fn poll_normal(&mut self) -> Result<Self::PollNormalOutput> {
        unimplemented!()
    }

    fn poll_int(&mut self) -> Result<Self::PollInterruptOutput> {
        let DeviceDriverInfo { name, attached } = self.device_driver_info;

        if !attached {
            return Ok(());
        }

        let trb = match self.pop_primary_event_ring() {
            Some(trb) => trb,
            None => return Ok(()),
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
                        "{}: Failed to process command (completion code: {:?})",
                        name, comp_code
                    );

                    // TODO
                    if comp_code == CompletionCode::TrbError {
                        self.device_driver_info.attached = false;
                    }

                    return Ok(());
                }

                if let (Some(port_id), Some(slot_id)) = (self.configuring_port_id, trb.slot_id()) {
                    match self.read_port(port_id).unwrap().config_state {
                        ConfigState::Reset => {
                            if let Err(err) = self.alloc_slot(port_id, slot_id) {
                                warn!("{}: {:?}", name, err);
                                return Ok(());
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
                        "{}: Might have been failed to process command (completion code: {:?})",
                        name, comp_code
                    );

                    // TODO
                    if comp_code == CompletionCode::TrbError {
                        self.device_driver_info.attached = false;
                    }

                    return Ok(());
                }

                let slot_id = trb.slot_id().unwrap();
                let endpoint_id = trb.endpoint_id().unwrap();

                //info!("slot id: {}, endpoint id: {}", slot_id, endpoint_id);

                if let Some(mut device) = find_device_by_slot_id(slot_id).unwrap_or(None) {
                    if !device.is_configured {
                        return Ok(());
                    }

                    device.update(endpoint_id, trb);

                    if update_device(device).is_ok() {
                        self.ring_doorbell(slot_id, endpoint_id as u8);
                    } else {
                        error!("{}: Failed to update USB device", name);
                    }
                }
            }
            TransferRequestBlockType::HostControllerEvent => {
                let comp_code = trb.completion_code().unwrap();
                if comp_code != CompletionCode::Success {
                    warn!(
                        "{}: Might have been failed to process command (completion code: {:?})",
                        name, comp_code
                    );

                    // TODO
                    // if comp_code == CompletionCode::TrbError {
                    //     self.is_init = false;
                    // }

                    return Ok(());
                }
            }
            _ => (),
        }

        Ok(())
    }

    fn read(&mut self) -> Result<Vec<u8>> {
        unimplemented!()
    }

    fn write(&mut self, _data: &[u8]) -> Result<()> {
        unimplemented!()
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    unsafe { XHC_DRIVER.try_lock() }?.get_device_driver_info()
}

pub fn probe_and_attach() -> Result<()> {
    let mut driver = unsafe { XHC_DRIVER.try_lock() }?;
    let driver_name = driver.get_device_driver_info()?.name;

    driver.probe()?;
    driver.attach(())?;
    info!("{}: Attached!", driver_name);
    driver.start()?;

    Ok(())
}

pub fn find_port_by_slot_id(slot_id: usize) -> Result<Option<Port>> {
    Ok(unsafe { XHC_DRIVER.try_lock() }?.find_port_by_slot_id(slot_id))
}

pub fn read_device_context(slot_id: usize) -> Result<Option<DeviceContext>> {
    Ok(unsafe { XHC_DRIVER.try_lock() }?.read_device_context(slot_id))
}

pub fn push_cmd_ring(trb: TransferRequestBlock) -> Result<()> {
    unsafe { XHC_DRIVER.try_lock() }?.push_cmd_ring(trb)
}

pub fn ring_doorbell(index: usize, value: u8) -> Result<()> {
    unsafe { XHC_DRIVER.try_lock() }?.ring_doorbell(index, value);
    Ok(())
}

pub fn scan_ports() -> Result<Vec<usize>> {
    unsafe { XHC_DRIVER.try_lock() }?.scan_ports()
}

pub fn reset_port(port_id: usize) -> Result<()> {
    unsafe { XHC_DRIVER.try_lock() }?.reset_port(port_id)
}

pub fn alloc_address_to_device(port_id: usize) -> Result<UsbDevice> {
    unsafe { XHC_DRIVER.try_lock() }?.alloc_address_to_device(port_id)
}

extern "x86-interrupt" fn poll_int_xhc_driver() {
    if let Ok(mut driver) = unsafe { XHC_DRIVER.try_lock() } {
        let _ = driver.poll_int();
    }

    idt::notify_end_of_int();
}
