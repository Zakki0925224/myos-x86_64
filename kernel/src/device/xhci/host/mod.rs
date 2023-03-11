use core::mem::size_of;

use crate::{arch::{addr::VirtualAddress, apic::local::read_local_apic_id, idt::VEC_MASKABLE_INT_0, register::msi::*}, bus::pci::{conf_space::BaseAddress, device_id::*, msi::*, PCI_DEVICE_MAN}, device::xhci::host::register::*, mem::bitmap::BITMAP_MEM_MAN};
use alloc::vec::Vec;
use log::{info, warn};

pub mod register;

#[derive(Debug)]
pub struct XhciHostDriver
{
    controller_pci_bus: usize,
    controller_pci_device: usize,
    controller_pci_func: usize,
    cap_reg_virt_addr: VirtualAddress,
    ope_reg_virt_addr: VirtualAddress,
    runtime_reg_virt_addr: VirtualAddress,
    int_reg_sets_virt_addr: VirtualAddress,
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
                controller_pci_bus: device.bus,
                controller_pci_device: device.device,
                controller_pci_func: device.func,
                cap_reg_virt_addr: VirtualAddress::new(0),
                ope_reg_virt_addr: VirtualAddress::new(0),
                runtime_reg_virt_addr: VirtualAddress::new(0),
                int_reg_sets_virt_addr: VirtualAddress::new(0),
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
            let bars = controller.read_conf_space_non_bridge_field().unwrap().get_bars();
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

            let cap_reg = self.read_cap_reg().unwrap();

            self.ope_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.cap_reg_length() as usize);

            self.runtime_reg_virt_addr =
                self.cap_reg_virt_addr.offset(cap_reg.runtime_reg_space_offset() as usize);

            self.int_reg_sets_virt_addr = self.cap_reg_virt_addr.offset(
                cap_reg.runtime_reg_space_offset() as usize + size_of::<RuntimeRegitsers>(),
            );

            if self.ope_reg_virt_addr.get() == 0
                || self.runtime_reg_virt_addr.get() == 0
                || self.int_reg_sets_virt_addr.get() == 0
            {
                warn!("Some registers virtual address is 0");
                failed_init_msg();
                return;
            }

            // initialize host controller
            let mut ope_reg = self.read_ope_reg().unwrap();

            // TODO: what to do if computer have already completed initialization (currently forced to initialize)
            // if !ope_reg.usb_status().hchalted()
            // {
            //     warn!("USBSTS.HCH is not 1");
            //     failed_init_msg();
            //     return;
            // }

            let mut usb_cmd = ope_reg.usb_cmd();
            usb_cmd.set_host_controller_reset(true);
            ope_reg.set_usb_cmd(usb_cmd);
            self.write_ope_reg(ope_reg);

            let mut cnt = 0;
            loop
            {
                if cnt > 10
                {
                    warn!("Timed out");
                    failed_init_msg();
                    return;
                }

                info!("Waiting xHCI host controller...");
                let ope_reg = self.read_ope_reg().unwrap();
                if !ope_reg.usb_cmd().host_controller_reset()
                    && !ope_reg.usb_status().controller_not_ready()
                {
                    break;
                }

                cnt += 1;
            }

            // initialize device context
            let cap_reg = self.read_cap_reg().unwrap();
            let max_slots = cap_reg.structural_params1().max_slots();
            if let Some(dev_context_mem_frame) = BITMAP_MEM_MAN.lock().alloc_single_mem_frame()
            {
                let device_context_arr: &mut [u64] =
                    dev_context_mem_frame.get_frame_start_virt_addr().read_volatile();

                // init device context array
                for i in 0..(max_slots + 1) as usize
                {
                    if let Some(entry) = device_context_arr.get_mut(i)
                    {
                        *entry = 0;
                    }
                }

                dev_context_mem_frame
                    .get_frame_start_virt_addr()
                    .write_volatile(device_context_arr);

                let mut ope_reg = self.read_ope_reg().unwrap();
                ope_reg.set_device_context_base_addr_array_ptr(
                    dev_context_mem_frame.get_frame_start_virt_addr().get_phys_addr().get(),
                );
                self.write_ope_reg(ope_reg);
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
                let mut ope_reg = self.read_ope_reg().unwrap();
                let mut crcr = CommandRingControlRegister::new();
                crcr.set_cmd_ring_ptr(
                    cmd_ring_mem.get_frame_start_virt_addr().get_phys_addr().get() >> 6,
                );
                ope_reg.set_cmd_ring_ctrl(crcr);
                self.write_ope_reg(ope_reg);
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
                // init first TRB
                let mut trb = TransferRequestBlock::new();
                trb.set_trb_type(TransferRequestBlockType::Normal);
                trb.set_cycle_bit(1);
                event_ring_seg_mem.get_frame_start_virt_addr().write_volatile(trb);

                // init first event ring segment table entry
                let mut event_ring_seg_table: EventRingSegmentTableEntry =
                    event_ring_seg_table_mem.get_frame_start_virt_addr().read_volatile();
                event_ring_seg_table.set_ring_seg_base_addr(
                    event_ring_seg_mem.get_frame_start_virt_addr().get_phys_addr().get(),
                );
                event_ring_seg_table.set_ring_seg_size(1);
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
                let mut reg_0 = InterrupterRegisterSet::new();
                reg_0.set_int_mod_interval(4000);
                reg_0.set_int_pending(true);
                reg_0.set_int_enable(true);
                reg_sets.registers[0] = reg_0;
                self.write_int_reg_sets(reg_sets);
            }
            else
            {
                warn!("Failed to read Interrupter Register Sets");
                failed_init_msg();
                return;
            }

            // start controller
            let mut ope_reg = self.read_ope_reg().unwrap();
            let mut usb_cmd = ope_reg.usb_cmd();
            usb_cmd.set_intr_enable(true);
            usb_cmd.set_run_stop(1);
            ope_reg.set_usb_cmd(usb_cmd);
            self.write_ope_reg(ope_reg);

            let mut cnt = 0;
            loop
            {
                if cnt > 10
                {
                    warn!("Timed out");
                    failed_init_msg();
                    return;
                }

                info!("Waiting xHCI host controller...");
                let ope_reg = self.read_ope_reg().unwrap();
                if !ope_reg.usb_status().hchalted()
                {
                    break;
                }

                cnt += 1;
            }

            info!("Initialized xHCI host driver");
        }
        else
        {
            failed_init_msg();
        }
    }

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

        return Some(self.ope_reg_virt_addr.read_volatile());
    }

    pub fn write_ope_reg(&self, ope_reg: OperationalRegisters)
    {
        if self.ope_reg_virt_addr.get() != 0
        {
            self.ope_reg_virt_addr.write_volatile(ope_reg);
        }
    }

    pub fn read_runtime_reg(&self) -> Option<RuntimeRegitsers>
    {
        if self.runtime_reg_virt_addr.get() == 0
        {
            return None;
        }

        return Some(self.runtime_reg_virt_addr.read_volatile());
    }

    pub fn write_runtime_reg(&self, runtime_reg: RuntimeRegitsers)
    {
        if self.runtime_reg_virt_addr.get() != 0
        {
            self.runtime_reg_virt_addr.write_volatile(runtime_reg);
        }
    }

    pub fn read_int_reg_sets(&self) -> Option<InterrupterRegisterSets>
    {
        if self.int_reg_sets_virt_addr.get() == 0
        {
            return None;
        }

        return Some(self.int_reg_sets_virt_addr.read_volatile());
    }

    pub fn write_int_reg_sets(&self, int_reg_sets: InterrupterRegisterSets)
    {
        if self.int_reg_sets_virt_addr.get() != 0
        {
            self.int_reg_sets_virt_addr.write_volatile(int_reg_sets);
        }
    }
}
