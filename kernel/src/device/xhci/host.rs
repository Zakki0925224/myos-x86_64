use core::mem::size_of;

use crate::{arch::{addr::*, apic::local::read_local_apic_id, idt::VEC_XHCI_INT, register::msi::*}, bus::pci::{conf_space::*, device_id::*, PCI_DEVICE_MAN}, device::xhci::{device::Device, register::*, ring_buffer::*}, mem::bitmap::BITMAP_MEM_MAN, println};
use alloc::vec::Vec;
use log::{info, warn};

const PORT_REG_SETS_START_VIRT_ADDR_OFFSET: usize = 1024;
const RING_BUF_LEN: usize = 16;

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
    num_of_ports: usize,
    transfer_ring_buf: Option<RingBuffer>,
    primary_event_ring_buf: Option<RingBuffer>,
    cmd_ring_buf: Option<RingBuffer>,

    devices: Vec<Device>,
}

impl XhcDriver
{
    pub fn new() -> Option<Self>
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
                    primary_event_ring_virt_addr: VirtualAddress::new(0),
                    num_of_ports: 0,
                    transfer_ring_buf: None,
                    primary_event_ring_buf: None,
                    cmd_ring_buf: None,
                    devices: Vec::new(),
                };

                info!(
                    "xhci: xHC device: {}.{}:{} - {}",
                    device.bus,
                    device.device,
                    device.func,
                    device.conf_space_header.get_device_name().unwrap()
                );

                return Some(usb);
            }
        }

        warn!("xhci: xHC device was not found");
        return None;
    }

    pub fn init(&mut self)
    {
        fn failed_init_msg()
        {
            warn!("xhci: Failed to initialize xHC driver");
        }

        if let Some(controller) = PCI_DEVICE_MAN.lock().find_by_bdf(
            self.controller_pci_bus,
            self.controller_pci_device,
            self.controller_pci_func,
        )
        {
            if let Some(conf_space_non_bridge_field) = controller.read_conf_space_non_bridge_field()
            {
                let bars = conf_space_non_bridge_field.get_bars();
                self.cap_reg_virt_addr = match bars[0].1
                {
                    BaseAddress::MemoryAddress64BitSpace(addr, _) => addr,
                    BaseAddress::MemoryAddress32BitSpace(addr, _) => addr,
                    _ =>
                    {
                        warn!("xhci: Invalid base address registers");
                        failed_init_msg();
                        return;
                    }
                }
                .get_virt_addr();
            }
            else
            {
                warn!("xhci: ConfigurationSpaceNonBridgeField was not found");
                failed_init_msg();
                return;
            }

            let cap_reg = self.read_cap_reg().unwrap();

            self.ope_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.cap_reg_length() as usize);

            self.runtime_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.runtime_reg_space_offset() as usize);

            self.intr_reg_sets_virt_addr = self.cap_reg_virt_addr.offset(
                cap_reg.runtime_reg_space_offset() as usize + size_of::<RuntimeRegitsers>(),
            );

            self.port_reg_sets_virt_addr =
                self.ope_reg_virt_addr.offset(PORT_REG_SETS_START_VIRT_ADDR_OFFSET);

            self.doorbell_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.doorbell_offset() as usize);

            if self.ope_reg_virt_addr.get() == 0
                || self.runtime_reg_virt_addr.get() == 0
                || self.intr_reg_sets_virt_addr.get() == 0
                || self.port_reg_sets_virt_addr.get() == 0
                || self.doorbell_reg_virt_addr.get() == 0
            {
                warn!("xhci: Some registers virtual address is 0");
                failed_init_msg();
                return;
            }

            // setting up msi
            let mut msg_addr = MsiMessageAddressField::new();
            msg_addr.set_destination_id(read_local_apic_id());
            msg_addr.set_redirection_hint_indication(0);
            msg_addr.set_destination_mode(0);

            let mut msg_data = MsiMessageDataField::new();
            msg_data.set_trigger_mode(TriggerMode::Level);
            msg_data.set_level(Level::Assert);
            msg_data.set_delivery_mode(DeliveryMode::Fixed);
            msg_data.set_vector(VEC_XHCI_INT as u8);

            if let Err(msg) = controller.set_msix_cap(msg_addr, msg_data)
            {
                warn!("xhci: {}", msg);
            }
            else
            {
                info!("xhci: Initialized MSI interrupt");
            }

            // TODO: request host controller ownership

            // stop controller
            let mut ope_reg = self.read_ope_reg().unwrap();
            let mut usb_cmd = ope_reg.usb_cmd();
            usb_cmd.set_intr_enable(false);
            usb_cmd.set_host_system_err_enable(false);
            usb_cmd.set_enable_wrap_event(false);

            if !ope_reg.usb_status().hchalted()
            {
                usb_cmd.set_run_stop(false);
            }

            ope_reg.set_usb_cmd(usb_cmd);
            self.write_ope_reg(ope_reg);

            loop
            {
                info!("xhci: Waiting xHC...");
                let ope_reg = self.read_ope_reg().unwrap();
                if ope_reg.usb_status().hchalted()
                {
                    break;
                }
            }
            info!("xhci: Stopped xHC");

            // reset controller
            let mut ope_reg = self.read_ope_reg().unwrap();
            let mut usb_cmd = ope_reg.usb_cmd();
            usb_cmd.set_host_controller_reset(true);
            ope_reg.set_usb_cmd(usb_cmd);
            self.write_ope_reg(ope_reg);

            loop
            {
                info!("xhci: Waiting xHC...");
                let ope_reg = self.read_ope_reg().unwrap();
                if !ope_reg.usb_cmd().host_controller_reset()
                    && !ope_reg.usb_status().controller_not_ready()
                {
                    break;
                }
            }
            info!("xhci: Reset xHC");

            // set max device slots
            self.num_of_ports = cap_reg.structural_params1().num_of_ports() as usize;
            let max_slots = cap_reg.structural_params1().num_of_device_slots();
            let mut ope_reg = self.read_ope_reg().unwrap();
            let mut conf_reg = ope_reg.configure();
            conf_reg.set_max_device_slots_enabled(max_slots);
            ope_reg.set_configure(conf_reg);
            self.write_ope_reg(ope_reg);

            // initialize scratchpad
            let cap_reg = self.read_cap_reg().unwrap();
            let sp2 = cap_reg.structural_params2();
            let num_of_bufs =
                (sp2.max_scratchpad_bufs_high() << 5 | sp2.max_scratchpad_bufs_low()) as usize;
            let mut scratchpad_buf_arr_virt_addr = VirtualAddress::new(0);
            if let Some(scratchpad_buf_arr_mem) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                let virt_addr = scratchpad_buf_arr_mem.get_frame_start_virt_addr();
                let arr: &mut [u64] = virt_addr.read_volatile();

                for i in 0..num_of_bufs
                {
                    if let Some(mem_frame_info) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
                    {
                        arr[i] = mem_frame_info.get_frame_start_virt_addr().get_phys_addr().get();
                    }
                    else
                    {
                        warn!(
                            "xhci: Failed to allocate memory frame for scratchpad buffer(#{})",
                            i
                        );
                        failed_init_msg();
                        return;
                    }
                }

                virt_addr.write_volatile(arr);
                scratchpad_buf_arr_virt_addr = virt_addr;
            }
            else
            {
                warn!("xhci: Failed to allocate memory frame for scratchpad buffer array");
                failed_init_msg();
                return;
            }

            // initialize device context
            if let Some(dev_context_mem_frame) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                let virt_addr = dev_context_mem_frame.get_frame_start_virt_addr();
                let device_context_arr: &mut [u64] = virt_addr.read_volatile();

                // init device context array
                for i in 0..(max_slots + 1) as usize
                {
                    if let Some(entry) = device_context_arr.get_mut(i)
                    {
                        if i == 0
                        {
                            *entry = scratchpad_buf_arr_virt_addr.get_phys_addr().get();
                            continue;
                        }

                        *entry = 0;
                    }
                }

                virt_addr.write_volatile(device_context_arr);

                let mut ope_reg = self.read_ope_reg().unwrap();
                ope_reg.set_device_context_base_addr_array_ptr(virt_addr.get_phys_addr().get());
                self.write_ope_reg(ope_reg);
                info!("xhci: Initialized device context");
            }
            else
            {
                warn!("xhci: Failed to allocate memory frame for device context");
                failed_init_msg();
                return;
            }

            let pcs = true;

            // register command ring
            if let Some(cmd_ring_mem) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                self.cmd_ring_virt_addr = cmd_ring_mem.get_frame_start_virt_addr();
                self.cmd_ring_buf = RingBuffer::new(
                    cmd_ring_mem,
                    None,
                    RING_BUF_LEN,
                    RingBufferType::CommandRing,
                    pcs,
                );

                if let Some(cmd_ring_buf) = self.cmd_ring_buf.as_mut()
                {
                    cmd_ring_buf.init();
                }
                else
                {
                    warn!("xhci: Failed to create command ring buffer");
                    failed_init_msg();
                    return;
                }

                let mut crcr = CommandRingControlRegister::new();
                crcr.set_cmd_ring_ptr(
                    cmd_ring_mem.get_frame_start_virt_addr().get_phys_addr().get() >> 6,
                );
                crcr.set_ring_cycle_state(pcs);
                crcr.set_cmd_stop(false);
                crcr.set_cmd_abort(false);
                let mut ope_reg = self.read_ope_reg().unwrap();
                ope_reg.set_cmd_ring_ctrl(crcr);
                self.write_ope_reg(ope_reg);

                info!("xhci: Initialized command ring");
            }
            else
            {
                warn!("xhci: Failed to allocate memory frame for command ring");
                failed_init_msg();
                return;
            }

            // register event ring (primary)
            let event_ring_seg_table_mem = BITMAP_MEM_MAN.lock().alloc_single_mem_frame();
            let event_ring_seg_mem = BITMAP_MEM_MAN.lock().alloc_single_mem_frame();
            if let (Some(event_ring_seg_table_mem), Some(event_ring_seg_mem)) =
                (event_ring_seg_table_mem, event_ring_seg_mem)
            {
                self.primary_event_ring_virt_addr = event_ring_seg_mem.get_frame_start_virt_addr();

                // init event ring segment table
                let mut event_ring_seg_table: EventRingSegmentTableEntry =
                    event_ring_seg_table_mem.get_frame_start_virt_addr().read_volatile();
                event_ring_seg_table.set_ring_seg_base_addr(
                    event_ring_seg_mem.get_frame_start_virt_addr().get_phys_addr().get(),
                );
                event_ring_seg_table.set_ring_seg_size(RING_BUF_LEN as u16);

                let mut entries = Vec::new();
                entries.push(event_ring_seg_table.clone());

                // init event ring buffer
                self.primary_event_ring_buf = RingBuffer::new(
                    event_ring_seg_mem,
                    Some(entries),
                    RING_BUF_LEN,
                    RingBufferType::EventRing,
                    pcs,
                );

                if let Some(primary_event_ring_buf) = self.primary_event_ring_buf.as_mut()
                {
                    primary_event_ring_buf.init();
                }
                else
                {
                    warn!("xhci: Failed to create primary event ring buffer");
                    failed_init_msg();
                    return;
                }

                event_ring_seg_mem.get_frame_start_virt_addr().write_volatile(event_ring_seg_table);

                // init first interrupter register sets entry
                let mut intr_reg_set_0 = self.read_intr_reg_sets(0).unwrap();
                //println!("before: {:?}", intr_reg_set_0);
                intr_reg_set_0.set_event_ring_seg_table_base_addr(
                    event_ring_seg_table_mem.get_frame_start_virt_addr().get_phys_addr().get(),
                );
                intr_reg_set_0.set_event_ring_seg_table_size(1);
                intr_reg_set_0.set_event_ring_dequeue_ptr(
                    (intr_reg_set_0.event_ring_seg_table_base_addr()
                        + size_of::<TransferRequestBlock>() as u64)
                        >> 4,
                );
                intr_reg_set_0.set_int_mod_interval(4000);
                intr_reg_set_0.set_int_pending(true);
                intr_reg_set_0.set_int_enable(true);
                //println!("after: {:?}", intr_reg_set_0);
                self.write_intr_reg_sets(0, intr_reg_set_0);
                //println!("read: {:?}", self.read_intr_reg_sets(0).unwrap());

                info!("xhci: Initialized event ring");
            }
            else
            {
                warn!("xhci: Failed to allocate memory frame for event ring or InterruptRegisterSets was not found");
                failed_init_msg();
                return;
            }

            // enable interrupt
            let mut ope_reg = self.read_ope_reg().unwrap();
            let mut usb_cmd = ope_reg.usb_cmd();
            usb_cmd.set_intr_enable(true);
            ope_reg.set_usb_cmd(usb_cmd);
            self.write_ope_reg(ope_reg);

            self.is_init = true;
            info!("xhci: Initialized xHC driver");
        }
        else
        {
            failed_init_msg();
        }
    }

    pub fn start(&mut self)
    {
        fn failed_init_msg()
        {
            warn!("xhci: Failed to start xHC driver");
        }

        if !self.is_init
        {
            warn!("xhci: xHC driver was not initialized");
            failed_init_msg();
            return;
        }

        // start controller
        info!("xhci: Starting xHC...");
        let mut ope_reg = self.read_ope_reg().unwrap();
        let mut usb_cmd = ope_reg.usb_cmd();
        usb_cmd.set_run_stop(true);
        ope_reg.set_usb_cmd(usb_cmd);
        self.write_ope_reg(ope_reg);

        loop
        {
            info!("xhci: Waiting xHC...");
            let ope_reg = self.read_ope_reg().unwrap();
            if !ope_reg.usb_status().hchalted()
            {
                break;
            }
        }

        // check status
        let usb_status = self.read_ope_reg().unwrap().usb_status();
        if usb_status.hchalted()
        {
            warn!("xhci: xHC is halted");
            failed_init_msg();
            return;
        }

        if usb_status.host_system_err()
        {
            warn!("xhci: An error occured on the host system");
            failed_init_msg();
            return;
        }

        if usb_status.host_controller_err()
        {
            warn!("xhci: An error occured on xHC");
            failed_init_msg();
            return;
        }

        info!("xhci: Available ports: {}", self.num_of_ports);
    }

    pub fn scan_ports(&mut self)
    {
        if !self.is_init || !self.is_running()
        {
            return;
        }

        self.devices = Vec::new();

        for i in 1..=self.num_of_ports
        {
            let mut port_reg_set = self.read_port_reg_set(i).unwrap();
            let mut sc_reg = port_reg_set.port_status_and_ctrl();
            if sc_reg.connect_status_change() && sc_reg.current_connect_status()
            {
                info!("xhci: Found connected port (port id: {})", i);

                // reset port
                sc_reg.set_port_reset(true);
                port_reg_set.set_port_status_and_ctrl(sc_reg);
                self.write_port_reg_set(i, port_reg_set);

                loop
                {
                    if self.read_port_reg_set(i).unwrap().port_status_and_ctrl().port_enabled()
                    {
                        break;
                    }
                }

                info!("xhci: Reset port (port id: {})", i);
            }
        }

        println!("{:?}", self.read_intr_reg_sets(0).unwrap());
    }

    pub fn is_init(&self) -> bool { return self.is_init; }

    pub fn is_running(&self) -> bool
    {
        if let Some(ope_reg) = self.read_ope_reg()
        {
            return !ope_reg.usb_status().hchalted();
        }
        else
        {
            return false;
        }
    }

    fn read_cap_reg(&self) -> Option<CapabilityRegisters>
    {
        return match self.cap_reg_virt_addr.get()
        {
            0 => None,
            _ => Some(CapabilityRegisters::read(self.cap_reg_virt_addr)),
        };
    }

    fn read_ope_reg(&self) -> Option<OperationalRegisters>
    {
        return match self.ope_reg_virt_addr.get()
        {
            0 => None,
            _ => Some(OperationalRegisters::read(self.ope_reg_virt_addr)),
        };
    }

    fn write_ope_reg(&self, ope_reg: OperationalRegisters)
    {
        match self.ope_reg_virt_addr.get()
        {
            0 => return,
            _ => ope_reg.write(self.ope_reg_virt_addr),
        }
    }

    fn read_runtime_reg(&self) -> Option<RuntimeRegitsers>
    {
        return match self.runtime_reg_virt_addr.get()
        {
            0 => None,
            _ => Some(RuntimeRegitsers::read(self.runtime_reg_virt_addr)),
        };
    }

    fn write_runtime_reg(&self, runtime_reg: RuntimeRegitsers)
    {
        match self.runtime_reg_virt_addr.get()
        {
            0 => return,
            _ => runtime_reg.write(self.runtime_reg_virt_addr),
        }
    }

    fn read_intr_reg_sets(&self, index: usize) -> Option<InterrupterRegisterSet>
    {
        if index > INTR_REG_SET_MAX_LEN || self.intr_reg_sets_virt_addr.get() == 0
        {
            return None;
        }

        let base_addr =
            self.intr_reg_sets_virt_addr.offset(index * size_of::<InterrupterRegisterSet>());
        return Some(InterrupterRegisterSet::read(base_addr));
    }

    fn write_intr_reg_sets(&self, index: usize, intr_reg_set: InterrupterRegisterSet)
    {
        if index > INTR_REG_SET_MAX_LEN || self.intr_reg_sets_virt_addr.get() == 0
        {
            return;
        }

        let base_addr =
            self.intr_reg_sets_virt_addr.offset(index * size_of::<InterrupterRegisterSet>());
        intr_reg_set.write(base_addr);
    }

    fn read_port_reg_set(&self, index: usize) -> Option<PortRegisterSet>
    {
        if index == 0 || index > self.num_of_ports || self.port_reg_sets_virt_addr.get() == 0
        {
            return None;
        }

        let base_addr =
            self.port_reg_sets_virt_addr.offset((index - 1) * size_of::<PortRegisterSet>());
        return Some(PortRegisterSet::read(base_addr));
    }

    fn write_port_reg_set(&self, index: usize, port_reg_set: PortRegisterSet)
    {
        if index == 0 || index > self.num_of_ports || self.port_reg_sets_virt_addr.get() == 0
        {
            return;
        }

        let base_addr =
            self.port_reg_sets_virt_addr.offset((index - 1) * size_of::<PortRegisterSet>());
        port_reg_set.write(base_addr);
    }

    fn read_doorbell_reg(&self, index: usize) -> Option<DoorbellRegister>
    {
        if index > DOORBELL_REG_MAX_LEN || self.doorbell_reg_virt_addr.get() == 0
        {
            return None;
        }

        let base_addr = self.doorbell_reg_virt_addr.offset(index * size_of::<DoorbellRegister>());
        return Some(DoorbellRegister::read(base_addr));
    }

    fn write_doorbell_reg(&self, index: usize, doorbell_reg: DoorbellRegister)
    {
        if index > DOORBELL_REG_MAX_LEN || self.doorbell_reg_virt_addr.get() == 0
        {
            return;
        }

        let base_addr = self.doorbell_reg_virt_addr.offset(index * size_of::<DoorbellRegister>());
        doorbell_reg.write(base_addr);
    }
}
