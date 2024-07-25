use crate::{
    addr::IoPortAddress,
    bus::pci::{self, conf_space::BaseAddress, vendor_id},
    device::{
        virtio::{virt_queue, DeviceStatus, NetworkDeviceFeature},
        DeviceDriverFunction, DeviceDriverInfo,
    },
    error::{Error, Result},
    mem::{bitmap, paging::PAGE_SIZE},
    println,
    util::mutex::Mutex,
};
use core::mem::size_of;

static mut VIRTIO_NET_DRIVER: Mutex<VirtioNetDriver> = Mutex::new(VirtioNetDriver::new());

// reference: https://docs.oasis-open.org/virtio/virtio/v1.2/csd01/virtio-v1.2-csd01.html
// 5.1.4 Device configuration layout
#[allow(dead_code)]
#[derive(Debug)]
struct ConfigurationField {
    /* +0x00 */ mac: [u8; 6],
    /* +0x06 */ status: u16,
    /* +0x08 */ max_virtqueue_pairs: u16,
    /* +0x0a */ mtu: u16,
    /* +0x0c */ speed: u32,
    /* +0x10 */ duplex: u8,
    /* +0x11 */ rss_max_key_size: u8,
    /* +0x12 */ supported_hash_types: u32,
}

impl ConfigurationField {
    fn read(io_port_base: &IoPortAddress) -> Self {
        let mac = [
            io_port_base.offset(0x00).in8(),
            io_port_base.offset(0x01).in8(),
            io_port_base.offset(0x02).in8(),
            io_port_base.offset(0x03).in8(),
            io_port_base.offset(0x04).in8(),
            io_port_base.offset(0x05).in8(),
        ];
        let status = io_port_base.offset(0x06).in16();
        let max_virtqueue_pairs = io_port_base.offset(0x08).in16();
        let mtu = io_port_base.offset(0x0a).in16();
        let speed = io_port_base.offset(0x0c).in32();
        let duplex = io_port_base.offset(0x10).in8();
        let rss_max_key_size = io_port_base.offset(0x11).in8();
        let supported_hash_types = io_port_base.offset(0x12).in32();
        Self {
            mac,
            status,
            max_virtqueue_pairs,
            mtu,
            speed,
            duplex,
            rss_max_key_size,
            supported_hash_types,
        }
    }
}

// 5.1.6 Device Operation
struct PacketHeader {
    flags: u8,
    gso_type: u8,
    hdr_len: u16,
    gso_size: u16,
    csum_offset: u16,
    num_buffers: u16,
    // hash_value: u32,       // VIRTIO_NET_F_HASH_REPORT
    // hash_report: u32,      // VIRTIO_NET_F_HASH_REPORT
    // padding_reserved: u16, // VIRTIO_NET_F_HASH_REPORT
}

impl PacketHeader {
    const FLAG_NEEDS_CSUM: u8 = 1;
    const FLAG_DATA_VALID: u8 = 2;
    const FLAG_RSC_INFO: u8 = 4;

    const GSO_NONE: u8 = 0;
    const GSO_TCPV4: u8 = 1;
    const GSO_UDP: u8 = 3;
    const GSO_TCPV6: u8 = 4;
    const GSO_UDP_L4: u8 = 5;
    const GSO_ECN: u8 = 0x80;
}

struct VirtioNetDriver {
    device_driver_info: DeviceDriverInfo,
    pci_device_bdf: Option<(usize, usize, usize)>,

    rx_queue: Option<virt_queue::Queue>,
    tx_queue: Option<virt_queue::Queue>,
}
impl VirtioNetDriver {
    const RX_QUEUE_INDEX: u16 = 0;
    const TX_QUEUE_INDEX: u16 = 1;

    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("vtnet"),
            pci_device_bdf: None,
            rx_queue: None,
            tx_queue: None,
        }
    }

    fn send_packet(&mut self, payload: &[u8]) -> Result<()> {
        let tx_queue = match self.tx_queue.as_mut() {
            Some(q) => q,
            None => return Err(Error::Failed("TX queue is not initialized")),
        };
        tx_queue.send_packet(payload)?;
        Ok(())
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
            fn read_device_status(io_port_base: &IoPortAddress) -> u8 {
                io_port_base.offset(0x12).in8()
            }

            fn write_device_status(io_port_base: &IoPortAddress, status: u8) {
                io_port_base.offset(0x12).out8(status)
            }

            fn write_queue_notify(io_port_base: &IoPortAddress, queue_index: u16) {
                io_port_base.offset(0x10).out16(queue_index)
            }

            fn register_virt_queue(
                io_port_base: &IoPortAddress,
                queue_size: usize,
                queue_index: u16,
            ) -> Result<virt_queue::Queue> {
                if queue_size == 0 {
                    return Err(Error::Failed("Queue size is 0"));
                }

                // allocate descs
                let bytes_of_descs = size_of::<virt_queue::QueueDescriptor>() * queue_size;
                let bytes_of_queue_available =
                    size_of::<virt_queue::QueueAvailableHeader>() + size_of::<u16>() * queue_size;
                let bytes_of_queue_used = size_of::<virt_queue::QueueUsedHeader>()
                    + size_of::<virt_queue::QueueUsedElement>() * queue_size;
                let queue_page_cnt = ((bytes_of_descs + bytes_of_queue_available) / PAGE_SIZE)
                    .max(1)
                    + (bytes_of_queue_used / PAGE_SIZE).max(1);

                let mem_frame_info = bitmap::alloc_mem_frame(queue_page_cnt)?;
                let queue = match virt_queue::Queue::init(mem_frame_info, queue_size) {
                    Ok(info) => info,
                    Err(err) => {
                        bitmap::dealloc_mem_frame(mem_frame_info)?;
                        return Err(err);
                    }
                };

                // queue_select
                io_port_base.offset(0x0e).out16(queue_index);
                // queue_address
                io_port_base.offset(0x08).out32(
                    (mem_frame_info.frame_start_phys_addr.get() as usize / PAGE_SIZE) as u32,
                );
                Ok(queue)
            }

            let conf_space = d.read_conf_space_non_bridge_field()?;
            let bars = conf_space.get_bars()?;
            let (_, mmio_bar) = bars
                .get(0)
                .ok_or(Error::Failed("Failed to read MMIO base address register"))?;
            let io_port_base = match mmio_bar {
                BaseAddress::MmioAddressSpace(addr) => *addr,
                _ => return Err(Error::Failed("Invalid base address register")),
            }
            .into();

            // enable device dirver
            // http://www.dumais.io/index.php?article=aca38a9a2b065b24dfa1dee728062a12
            write_device_status(&io_port_base, DeviceStatus::Acknowledge as u8);
            write_device_status(
                &io_port_base,
                read_device_status(&io_port_base) | DeviceStatus::Driver as u8,
            );

            // enable device supported features + VIRTIO_NET_F_MAC
            let device_features = io_port_base.in32();
            // driver_features
            io_port_base
                .offset(0x04)
                .out32(device_features | NetworkDeviceFeature::Mac as u32);

            write_device_status(
                &io_port_base,
                read_device_status(&io_port_base) | DeviceStatus::FeaturesOk as u8,
            );

            // register virtqueues
            let queue_size = io_port_base.offset(0x0c).in16() as usize;
            self.rx_queue = Some(register_virt_queue(
                &io_port_base,
                queue_size,
                Self::RX_QUEUE_INDEX,
            )?);
            self.tx_queue = Some(register_virt_queue(
                &io_port_base,
                queue_size,
                Self::TX_QUEUE_INDEX,
            )?);

            write_device_status(
                &io_port_base,
                read_device_status(&io_port_base) | DeviceStatus::DriverOk as u8,
            );

            // configure rx virtqueue
            let rx_queue = self.rx_queue.as_mut().unwrap();
            rx_queue.available_header_mut().index = rx_queue.queue_size() as u16;

            for desc in rx_queue.descs_mut() {
                let mem_frame_info = bitmap::alloc_mem_frame(1)?;
                desc.addr = mem_frame_info.frame_start_virt_addr()?.get();
                desc.len = mem_frame_info.frame_size as u32;
                desc.flags = 2; // device write only
                desc.next = 0;
            }

            for (i, elem) in rx_queue.available_elements_mut().iter_mut().enumerate() {
                *elem = i as u16;
            }
            write_queue_notify(&io_port_base, Self::RX_QUEUE_INDEX);

            // configure tx virtqueue
            let tx_queue = self.tx_queue.as_mut().unwrap();

            for desc in tx_queue.descs_mut() {
                let mem_frame_info = bitmap::alloc_mem_frame(1)?;
                desc.addr = mem_frame_info.frame_start_virt_addr()?.get();
                desc.len = mem_frame_info.frame_size as u32;
                desc.flags = 0; // device read only
                desc.next = 0;
            }

            let conf_field = ConfigurationField::read(&io_port_base.offset(0x14));
            println!("{:?}", conf_field);

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
