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

// 5.1.3 Feature bits
#[derive(Debug)]
#[repr(u32)]
enum NetworkDeviceFeature {
    Mac = 5,
}
