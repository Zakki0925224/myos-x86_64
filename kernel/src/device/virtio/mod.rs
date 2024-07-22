use crate::addr::VirtualAddress;
use core::mem::transmute;

pub mod net;

const MMIO_DEVICE_REG_MAGIC: u32 = 0x74726976;

// reference: https://docs.oasis-open.org/virtio/virtio/v1.2/csd01/virtio-v1.2-csd01.html

// 4.2.2 MMIO Device Register Layout
#[derive(Debug)]
#[repr(C)]
struct MmioDeviceRegister {
    /*r */ magic: u32,
    /*r */ version: u32,
    /*r */ device_id: u32,
    /*r */ vendor_id: u32,
    /*r */ device_features: u32,
    /*w */ device_features_sel: u32,
    /*w */ driver_features: u32,
    /*w */ driver_features_sel: u32,
    /*w */ queue_sel: u32,
    /*r */ queue_num_max: u32,
    /*w */ queue_num: u32,
    /*rw*/ queue_ready: u32,
    /*w */ queue_notify: u32,
    /*r */ int_status: u32,
    /*w */ int_ack: u32,
    /*rw*/ status: u32,
    /*w */ queue_desc_low: u32,
    /*w */ queue_desc_high: u32,
    /*w */ queue_device_low: u32,
    /*w */ queue_device_high: u32,
    /*w */ shm_sel: u32, // shared memory
    /*r */ shm_len_low: u32,
    /*r */ shm_len_high: u32,
    /*r */ shm_base_low: u32,
    /*r */ shm_base_high: u32,
    /*rw*/ queue_reset: u32,
    /*r */ conf_gen: u32,
    // /*rw*/ config: N
}

impl MmioDeviceRegister {
    pub fn read(base_addr: VirtualAddress) -> Self {
        let mut data = [0; 27];
        for (i, field) in data.iter_mut().enumerate() {
            *field = base_addr.offset(i * 4).read_volatile();
        }

        let reg = unsafe { transmute::<[u32; 27], Self>(data) };
        assert_eq!(reg.magic, MMIO_DEVICE_REG_MAGIC, "Invalid magic number");
        reg
    }

    #[allow(dead_code)]
    pub fn write(&self) {
        unimplemented!();
    }
}
