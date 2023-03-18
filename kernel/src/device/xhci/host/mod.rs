use core::mem::{size_of, transmute};

use crate::{arch::{addr::*, apic::local::read_local_apic_id, idt::VEC_MASKABLE_INT_0, register::msi::*}, bus::pci::{conf_space::BaseAddress, device_id::*, msi::*, PCI_DEVICE_MAN}, device::xhci::host::{register::*, ring_buffer::RingBufferType}, mem::bitmap::BITMAP_MEM_MAN, println};
use alloc::vec::Vec;
use log::{info, warn};

use self::ring_buffer::RingBuffer;

pub mod register;
pub mod ring_buffer;

const PORT_REG_SETS_START_VIRT_ADDR_OFFSET: usize = 1024;
const RING_BUF_LEN: usize = 16;

#[derive(Debug)]
pub struct XhciHostDriver
{
    is_init: bool,
    controller_pci_bus: usize,
    controller_pci_device: usize,
    controller_pci_func: usize,
    cap_reg_virt_addr: VirtualAddress,
    ope_reg_virt_addr: VirtualAddress,
    runtime_reg_virt_addr: VirtualAddress,
    int_reg_sets_virt_addr: VirtualAddress,
    port_reg_sets_virt_addr: VirtualAddress,
    doorbell_reg_virt_addr: VirtualAddress,
    cmd_ring_virt_addr: VirtualAddress,
    primary_event_ring_virt_addr: VirtualAddress,
    root_hub_port_cnt: usize,
    transfer_ring_buf: Option<RingBuffer>,
    primary_event_ring_buf: Option<RingBuffer>,
    cmd_ring_buf: Option<RingBuffer>,
}

impl XhciHostDriver
{
    pub fn new() -> Option<Self>
    {
        let (class_code, subclass_code, prog_if) = PCI_USB_XHCI_ID;

        if let Some(device) =
            PCI_DEVICE_MAN.lock().find_by_class(class_code, subclass_code, prog_if)
        {
            let usb = XhciHostDriver {
                is_init: false,
                controller_pci_bus: device.bus,
                controller_pci_device: device.device,
                controller_pci_func: device.func,
                cap_reg_virt_addr: VirtualAddress::new(0),
                ope_reg_virt_addr: VirtualAddress::new(0),
                runtime_reg_virt_addr: VirtualAddress::new(0),
                int_reg_sets_virt_addr: VirtualAddress::new(0),
                port_reg_sets_virt_addr: VirtualAddress::new(0),
                doorbell_reg_virt_addr: VirtualAddress::new(0),
                cmd_ring_virt_addr: VirtualAddress::new(0),
                primary_event_ring_virt_addr: VirtualAddress::new(0),
                root_hub_port_cnt: 0,
                transfer_ring_buf: None,
                primary_event_ring_buf: None,
                cmd_ring_buf: None,
            };

            info!(
                "xHCI host controller: {}.{}:{} - {}",
                device.bus,
                device.device,
                device.func,
                device.conf_space_header.get_device_name().unwrap()
            );

            return Some(usb);
        }

        warn!("xHCI host controller was not found");
        return None;
    }

    pub fn init(&mut self)
    {
        fn failed_init_msg()
        {
            warn!("Failed to initialize xHCI host driver");
        }

        if let Some(controller) = PCI_DEVICE_MAN.lock().find_by_bdf(
            self.controller_pci_bus,
            self.controller_pci_device,
            self.controller_pci_func,
        )
        {
            let pcs = true;
            if let Some(conf_space_non_bridge_field) = controller.read_conf_space_non_bridge_field()
            {
                let bars = conf_space_non_bridge_field.get_bars();
                self.cap_reg_virt_addr = match bars[0].1
                {
                    BaseAddress::MemoryAddress64BitSpace(addr, _) => addr,
                    BaseAddress::MemoryAddress32BitSpace(addr, _) => addr,
                    _ =>
                    {
                        warn!("Invalid base address registers");
                        failed_init_msg();
                        return;
                    }
                }
                .get_virt_addr();
            }
            else
            {
                warn!("ConfigurationSpaceNonBridgeField was not found");
                failed_init_msg();
                return;
            }

            let cap_reg = self.read_cap_reg().unwrap();

            self.ope_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.cap_reg_length() as usize);

            self.runtime_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.runtime_reg_space_offset() as usize);

            self.int_reg_sets_virt_addr = self.cap_reg_virt_addr.offset(
                cap_reg.runtime_reg_space_offset() as usize + size_of::<RuntimeRegitsers>(),
            );

            self.port_reg_sets_virt_addr =
                self.ope_reg_virt_addr.offset(PORT_REG_SETS_START_VIRT_ADDR_OFFSET);

            self.doorbell_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.doorbell_offset() as usize);

            if self.ope_reg_virt_addr.get() == 0
                || self.runtime_reg_virt_addr.get() == 0
                || self.int_reg_sets_virt_addr.get() == 0
                || self.port_reg_sets_virt_addr.get() == 0
                || self.doorbell_reg_virt_addr.get() == 0
            {
                warn!("Some registers virtual address is 0");
                failed_init_msg();
                return;
            }

            // stop controller
            let mut ope_reg = self.read_ope_reg().unwrap();
            let mut usb_cmd = ope_reg.usb_cmd();
            usb_cmd.set_run_stop(false);
            ope_reg.set_usb_cmd(usb_cmd);
            self.write_ope_reg(ope_reg);

            loop
            {
                info!("Waiting xHCI host controller...");
                let ope_reg = self.read_ope_reg().unwrap();
                if ope_reg.usb_status().hchalted()
                {
                    break;
                }
            }
            info!("Stopped xHCI host controller");

            // reset controller
            let mut ope_reg = self.read_ope_reg().unwrap();
            let mut usb_cmd = ope_reg.usb_cmd();
            usb_cmd.set_host_controller_reset(true);
            ope_reg.set_usb_cmd(usb_cmd);
            self.write_ope_reg(ope_reg);

            loop
            {
                info!("Waiting xHCI host controller...");
                let ope_reg = self.read_ope_reg().unwrap();
                if !ope_reg.usb_cmd().host_controller_reset()
                    && !ope_reg.usb_status().controller_not_ready()
                {
                    break;
                }
            }
            info!("Reset xHCI host controller");

            // set max device slots
            let max_slots = cap_reg.structural_params1().max_slots();
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
                        warn!("Failed to allocate memory frame for scratchpad buffer(#{})", i);
                        failed_init_msg();
                        return;
                    }
                }

                virt_addr.write_volatile(arr);
                scratchpad_buf_arr_virt_addr = virt_addr;
            }
            else
            {
                warn!("Failed to allocate memory frame for scratchpad buffer array");
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
                info!("Initialized device context");
            }
            else
            {
                warn!("Failed to allocate memory frame for device context");
                failed_init_msg();
                return;
            }

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

                if let Some(cmd_ring_buf) = &self.cmd_ring_buf
                {
                    cmd_ring_buf.init();
                }
                else
                {
                    warn!("Failed to create command ring buffer");
                    failed_init_msg();
                    return;
                }

                let mut crcr = CommandRingControlRegister::new();
                crcr.set_cmd_ring_ptr(
                    cmd_ring_mem.get_frame_start_virt_addr().get_phys_addr().get() >> 6,
                );
                crcr.set_ring_cycle_state(pcs);
                let mut ope_reg = self.read_ope_reg().unwrap();
                ope_reg.set_cmd_ring_ctrl(crcr);
                self.write_ope_reg(ope_reg);

                info!("Initialized command ring");
            }
            else
            {
                warn!("Failed to allocate memory frame for command ring");
                failed_init_msg();
                return;
            }

            // register event ring (primary)
            let event_ring_seg_table_mem = BITMAP_MEM_MAN.lock().alloc_single_mem_frame();
            let event_ring_seg_mem = BITMAP_MEM_MAN.lock().alloc_single_mem_frame();
            let int_reg_sets = self.read_int_reg_sets();
            if let (
                Some(event_ring_seg_table_mem),
                Some(event_ring_seg_mem),
                Some(mut int_reg_sets),
            ) = (event_ring_seg_table_mem, event_ring_seg_mem, int_reg_sets)
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

                if let Some(primary_event_ring_buf) = &self.primary_event_ring_buf
                {
                    primary_event_ring_buf.init();
                }
                else
                {
                    warn!("Failed to create primary event ring buffer");
                    failed_init_msg();
                    return;
                }

                event_ring_seg_mem.get_frame_start_virt_addr().write_volatile(event_ring_seg_table);

                // init first interrupter register sets entry
                let mut int_reg_set_0 = int_reg_sets.registers[0];
                int_reg_set_0.set_event_ring_seg_table_base_addr(
                    event_ring_seg_table_mem.get_frame_start_virt_addr().get_phys_addr().get(),
                );
                int_reg_set_0.set_event_ring_seg_table_size(1);
                int_reg_set_0.set_event_ring_dequeue_ptr(
                    int_reg_set_0.event_ring_seg_table_base_addr()
                        + size_of::<TransferRequestBlock>() as u64,
                );
                int_reg_sets.registers[0] = int_reg_set_0;
                self.write_int_reg_sets(int_reg_sets);

                info!("Initialized event ring");
            }
            else
            {
                warn!("Failed to allocate memory frame for event ring or InterruptRegisterSets was not found");
                failed_init_msg();
                return;
            }

            // setting up msi
            let mut caps_list = Vec::new();

            let mut cap = MsiCapabilityField::new();
            cap.set_cap_id(5);

            let mut msg_ctrl = MsiMessageControlField::new();
            msg_ctrl.set_is_enable(true);
            msg_ctrl.set_multiple_msg_capable(0);
            cap.set_msg_ctrl(msg_ctrl);

            let mut msg_addr = MsiMessageAddressField::new();
            msg_addr.set_destination_id(read_local_apic_id());
            msg_addr.set_redirection_hint_indication(0);
            msg_addr.set_destination_mode(0);
            cap.set_msg_addr_low(msg_addr);

            let mut msg_data = MsiMessageDataField::new();
            msg_data.set_trigger_mode(TriggerMode::Level);
            msg_data.set_level(Level::Assert);
            msg_data.set_delivery_mode(DeliveryMode::Fixed);
            msg_data.set_vector(VEC_MASKABLE_INT_0 as u8);
            cap.set_msg_data(msg_data);

            caps_list.push(cap);
            controller.write_caps_list(caps_list);

            if let Some(mut reg_sets) = self.read_int_reg_sets()
            {
                let mut reg_0 = reg_sets.registers[0];
                reg_0.set_int_mod_interval(4000);
                reg_0.set_int_pending(true);
                reg_0.set_int_enable(true);
                reg_sets.registers[0] = reg_0;
                self.write_int_reg_sets(reg_sets);
                info!("Initialized MSI interrupt");
            }
            else
            {
                warn!("Failed to read Interrupter Register Sets");
                failed_init_msg();
                return;
            }

            // start controller
            info!("Starting xHCI host controller...");
            let mut ope_reg = self.read_ope_reg().unwrap();
            let mut usb_cmd = ope_reg.usb_cmd();
            usb_cmd.set_intr_enable(true);
            usb_cmd.set_run_stop(true);
            ope_reg.set_usb_cmd(usb_cmd);
            self.write_ope_reg(ope_reg);

            loop
            {
                info!("Waiting xHCI host controller...");
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
                warn!("Host controller is halted");
                failed_init_msg();
                return;
            }

            if usb_status.host_system_err()
            {
                warn!("An error occured on the host system");
                failed_init_msg();
                return;
            }

            if usb_status.host_controller_err()
            {
                warn!("An error occured on the xHC");
                failed_init_msg();
                return;
            }

            let cap_leg = self.read_cap_reg().unwrap();
            self.root_hub_port_cnt = cap_leg.structural_params1().max_ports() as usize;

            self.is_init = true;
            info!("Initialized xHCI host driver");

            let mut noop_trb = TransferRequestBlock::new();
            noop_trb.set_trb_type(TransferRequestBlockType::NoOpCommand);
            self.cmd_ring_buf.as_ref().unwrap().write(0, noop_trb);

            // TODO: enqueue and dequeue
            // loop
            // {
            //     println!("{:?}", self.primary_event_ring_buf.as_ref().unwrap().read(0));
            // }
        }
        else
        {
            failed_init_msg();
        }
    }

    pub fn is_init(&self) -> bool { return self.is_init; }

    pub fn read_cap_reg(&self) -> Option<CapabilityRegisters>
    {
        if self.cap_reg_virt_addr.get() == 0
        {
            return None;
        }

        return Some(self.cap_reg_virt_addr.read_volatile());
    }

    pub fn read_ope_reg(&self) -> Option<OperationalRegisters>
    {
        if self.ope_reg_virt_addr.get() == 0
        {
            return None;
        }

        let mut data: [u32; 15] = [0; 15];
        for (i, elem) in data.iter_mut().enumerate()
        {
            *elem = self.ope_reg_virt_addr.offset(i * 4).read_volatile::<u32>();
        }

        return Some(unsafe { transmute::<[u32; 15], OperationalRegisters>(data) });
    }

    pub fn write_ope_reg(&self, ope_reg: OperationalRegisters)
    {
        if self.ope_reg_virt_addr.get() == 0
        {
            return;
        }

        let data = unsafe { transmute::<OperationalRegisters, [u32; 15]>(ope_reg) };
        for (i, elem) in data.iter().enumerate()
        {
            self.ope_reg_virt_addr.offset(i * 4).write_volatile(*elem);
        }
    }

    pub fn read_runtime_reg(&self) -> Option<RuntimeRegitsers>
    {
        if self.runtime_reg_virt_addr.get() == 0
        {
            return None;
        }

        let mut data: [u32; 8] = [0; 8];
        for (i, elem) in data.iter_mut().enumerate()
        {
            *elem = self.runtime_reg_virt_addr.offset(i * 4).read_volatile::<u32>();
        }

        return Some(unsafe { transmute::<[u32; 8], RuntimeRegitsers>(data) });
    }

    pub fn write_runtime_reg(&self, runtime_reg: RuntimeRegitsers)
    {
        if self.runtime_reg_virt_addr.get() == 0
        {
            return;
        }

        let data = unsafe { transmute::<RuntimeRegitsers, [u32; 8]>(runtime_reg) };
        for (i, elem) in data.iter().enumerate()
        {
            self.runtime_reg_virt_addr.offset(i * 4).write_volatile(*elem);
        }
    }

    pub fn read_int_reg_sets(&self) -> Option<InterrupterRegisterSets>
    {
        if self.int_reg_sets_virt_addr.get() == 0
        {
            return None;
        }

        let mut data: [u32; 8192] = [0; 8192];
        for (i, elem) in data.iter_mut().enumerate()
        {
            *elem = self.int_reg_sets_virt_addr.offset(i * 4).read_volatile::<u32>();
        }

        return Some(unsafe { transmute::<[u32; 8192], InterrupterRegisterSets>(data) });
    }

    pub fn write_int_reg_sets(&self, int_reg_sets: InterrupterRegisterSets)
    {
        if self.int_reg_sets_virt_addr.get() == 0
        {
            return;
        }

        let data = unsafe { transmute::<InterrupterRegisterSets, [u32; 8192]>(int_reg_sets) };
        for (i, elem) in data.iter().enumerate()
        {
            self.int_reg_sets_virt_addr.offset(i * 4).write_volatile(*elem);
        }
    }

    pub fn read_port_reg_set(&self, index: usize) -> Option<PortRegisterSet>
    {
        if index == 0 || index > self.root_hub_port_cnt || self.port_reg_sets_virt_addr.get() == 0
        {
            return None;
        }

        let reg_set_virt_addr =
            self.port_reg_sets_virt_addr.offset((index - 1) * size_of::<PortRegisterSet>());
        return Some(reg_set_virt_addr.read_volatile());
    }

    pub fn write_port_reg_set(&self, index: usize, port_reg_set: PortRegisterSet)
    {
        if index == 0 || index > self.root_hub_port_cnt || self.port_reg_sets_virt_addr.get() == 0
        {
            return;
        }

        let reg_set_virt_addr =
            self.port_reg_sets_virt_addr.offset((index - 1) * size_of::<PortRegisterSet>());
        reg_set_virt_addr.write_volatile(port_reg_set);
    }

    pub fn read_doorbell_reg(&self, index: usize) -> Option<DoorbellRegister>
    {
        if index > DOORBELL_REG_MAX_LEN || self.doorbell_reg_virt_addr.get() == 0
        {
            return None;
        }

        let reg_virt_addr =
            self.doorbell_reg_virt_addr.offset(index * size_of::<DoorbellRegister>());
        return Some(reg_virt_addr.read_volatile());
    }

    pub fn write_doorbell_reg(&self, index: usize, doorbell_reg: DoorbellRegister)
    {
        if index > DOORBELL_REG_MAX_LEN || self.doorbell_reg_virt_addr.get() == 0
        {
            return;
        }

        let reg_virt_addr =
            self.doorbell_reg_virt_addr.offset(index * size_of::<DoorbellRegister>());
        reg_virt_addr.write_volatile(doorbell_reg);
    }
}
