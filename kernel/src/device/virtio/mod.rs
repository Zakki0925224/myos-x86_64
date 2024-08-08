use crate::addr::IoPortAddress;

pub mod net;
mod virt_queue;

// reference: https://docs.oasis-open.org/virtio/virtio/v1.2/csd01/virtio-v1.2-csd01.html
// 2.1 Device Status Field
#[derive(Debug)]
#[repr(u8)]
enum DeviceStatus {
    Acknowledge = 1,
    Driver = 2,
    Failed = 128,
    FeaturesOk = 8,
    DriverOk = 4,
    DeviceNeedsReset = 64,
}

// 5.1.3 Feature bits
#[derive(Debug)]
#[repr(u32)]
enum NetworkDeviceFeature {
    Mac = 5,
}

// 4.1.4.10 Legacy Interfaces: A Note on PCI Device Layout
// offset   bits  rw    desc
// +0x00    32    r     device_features
// +0x04    32    r/w   driver_features
// +0x08    32    r/w   queue_address
// +0x0c    16    r     queue_size
// +0x0e    16    r/w   queue_select
// +0x10    16    r/w   queue_notify
// +0x12    8     r/w   device_status
// +0x13    8     r     isr_status
#[derive(Debug)]
enum InterruptType {
    Queue,
    DeviceConfiguration,
}

struct IoRegister(IoPortAddress);

impl IoRegister {
    fn new(io_port_base: IoPortAddress) -> Self {
        Self(io_port_base)
    }

    fn io_port_base(&self) -> &IoPortAddress {
        &self.0
    }

    fn read_device_features(&self) -> u32 {
        self.io_port_base().in32()
    }

    fn read_driver_features(&self) -> u32 {
        self.io_port_base().offset(0x04).in32()
    }

    fn write_driver_features(&self, features: u32) {
        self.io_port_base().offset(0x04).out32(features)
    }

    fn read_queue_address(&self) -> u32 {
        self.io_port_base().offset(0x08).in32()
    }

    fn write_queue_address(&self, address: u32) {
        self.io_port_base().offset(0x08).out32(address)
    }

    fn read_queue_size(&self) -> u16 {
        self.io_port_base().offset(0x0c).in16()
    }

    fn read_queue_select(&self) -> u16 {
        self.io_port_base().offset(0x0e).in16()
    }

    fn write_queue_select(&self, queue_index: u16) {
        self.io_port_base().offset(0x0e).out16(queue_index)
    }

    fn read_queue_notify(&self) -> u16 {
        self.io_port_base().offset(0x10).in16()
    }

    fn write_queue_notify(&self, queue_index: u16) {
        self.io_port_base().offset(0x10).out16(queue_index)
    }

    fn read_device_status(&self) -> u8 {
        self.io_port_base().offset(0x12).in8()
    }

    fn write_device_status(&self, status: u8) {
        self.io_port_base().offset(0x12).out8(status)
    }

    fn read_isr_status(&self) -> u8 {
        self.io_port_base().offset(0x13).in8()
    }

    fn interrupt_type(&self) -> Option<InterruptType> {
        let status = self.read_isr_status();

        if status & 0x1 != 0 {
            Some(InterruptType::Queue)
        } else if status & 0x2 != 0 {
            Some(InterruptType::DeviceConfiguration)
        } else {
            None
        }
    }
}
