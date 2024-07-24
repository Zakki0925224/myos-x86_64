use crate::{
    arch,
    bus::pci::{self, conf_space::BaseAddress, vendor_id},
    device::{
        virtio::{virt_queue, DeviceStatus, NetworkDeviceFeature},
        DeviceDriverFunction, DeviceDriverInfo,
    },
    error::{Error, Result},
    mem::{bitmap, paging::PAGE_SIZE},
    util::mutex::Mutex,
};
use core::mem::size_of;

static mut VIRTIO_NET_DRIVER: Mutex<VirtioNetDriver> = Mutex::new(VirtioNetDriver::new());

struct VirtioNetDriver {
    device_driver_info: DeviceDriverInfo,
    pci_device_bdf: Option<(usize, usize, usize)>,

    queue_info: Option<virt_queue::Queue>,
}
impl VirtioNetDriver {
    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("vtnet"),
            pci_device_bdf: None,
            queue_info: None,
        }
    }
}

impl DeviceDriverFunction for VirtioNetDriver {
    fn get_device_driver_info(&self) -> Result<DeviceDriverInfo> {
        Ok(self.device_driver_info.clone())
    }

    fn probe(&mut self) -> Result<()> {
        pci::find_devices(2, 0, 0, |d| {
            let vendor_id = d.conf_space_header().vendor_id;
            let device_id = d.conf_space_header().device_id;

            // transitional virtio-net device
            if vendor_id == vendor_id::RED_HAT && device_id == 0x1000 {
                self.pci_device_bdf = Some(d.device_bdf());
            }
            Ok(())
        })?;

        Ok(())
    }

    fn attach(&mut self) -> Result<()> {
        if self.pci_device_bdf.is_none() {
            return Err(Error::Failed("Device driver is not probed"));
        }

        let (bus, device, func) = self.pci_device_bdf.unwrap();
        pci::configure_device(bus, device, func, |d| {
            fn read_device_status(io_port_base: u16) -> u8 {
                arch::in8(io_port_base + 0x12)
            }

            fn write_device_status(io_port_base: u16, status: u8) {
                arch::out8(io_port_base + 0x12, status)
            }

            let conf_space = d.read_conf_space_non_bridge_field()?;
            let bars = conf_space.get_bars()?;
            let (_, mmio_bar) = bars
                .get(0)
                .ok_or(Error::Failed("Failed to read MMIO base address register"))?;
            let io_port = match mmio_bar {
                BaseAddress::MmioAddressSpace(addr) => *addr,
                _ => return Err(Error::Failed("Invalid base address register")),
            };

            // enable device dirver
            // http://www.dumais.io/index.php?article=aca38a9a2b065b24dfa1dee728062a12
            write_device_status(io_port as u16, DeviceStatus::Acknowledge as u8);
            write_device_status(
                io_port as u16,
                read_device_status(io_port as u16) | DeviceStatus::Driver as u8,
            );

            // enable device supported features + VIRTIO_NET_F_MAC
            let device_features = arch::in32(io_port);
            // driver_features
            arch::out32(
                io_port + 0x04,
                device_features | NetworkDeviceFeature::Mac as u32,
            );

            write_device_status(
                io_port as u16,
                read_device_status(io_port as u16) | DeviceStatus::FeaturesOk as u8,
            );

            // config virtqueue
            // queue_select
            arch::out16(io_port as u16 + 0x0e, /* queue index */ 0);
            let queue_size = arch::in16(io_port as u16 + 0x0c);
            // allocate descs
            let bytes_of_descs = size_of::<virt_queue::QueueDescriptor>() * queue_size as usize;
            let bytes_of_queue_available = size_of::<virt_queue::QueueAvailableHeader>()
                + size_of::<u16>() * queue_size as usize;
            let bytes_of_queue_used = size_of::<virt_queue::QueueUsedHeader>()
                + size_of::<virt_queue::QueueUsedElement>() * queue_size as usize;
            let queue_page_cnt = ((bytes_of_descs + bytes_of_queue_available) / PAGE_SIZE).max(1)
                + (bytes_of_queue_used / PAGE_SIZE).max(1);

            let mem_frame_info = bitmap::alloc_mem_frame(queue_page_cnt)?;

            let queue_info = match virt_queue::Queue::init(mem_frame_info, queue_size as usize) {
                Ok(info) => info,
                Err(err) => {
                    bitmap::dealloc_mem_frame(mem_frame_info)?;
                    return Err(err);
                }
            };
            self.queue_info = Some(queue_info);

            Ok(())
        })?;

        self.device_driver_info.attached = true;
        Ok(())
    }
}

pub fn get_device_driver_info() -> Result<DeviceDriverInfo> {
    let driver = unsafe { VIRTIO_NET_DRIVER.try_lock() }?;
    driver.get_device_driver_info()
}

pub fn probe_and_attach() -> Result<()> {
    let mut driver = unsafe { VIRTIO_NET_DRIVER.try_lock() }?;
    driver.probe()?;
    driver.attach()?;
    Ok(())
}
