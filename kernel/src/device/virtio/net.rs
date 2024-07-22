use crate::{
    bus::pci::{self, conf_space::BaseAddress, vendor_id},
    device::{virtio::MmioDeviceRegister, DeviceDriverFunction, DeviceDriverInfo},
    error::{Error, Result},
    println,
    util::mutex::Mutex,
};

static mut VIRTIO_NET_DRIVER: Mutex<VirtioNetDriver> = Mutex::new(VirtioNetDriver::new());

struct VirtioNetDriver {
    device_driver_info: DeviceDriverInfo,
    pci_device_bdf: Option<(usize, usize, usize)>,
}
impl VirtioNetDriver {
    const fn new() -> Self {
        Self {
            device_driver_info: DeviceDriverInfo::new("vtnet"),
            pci_device_bdf: None,
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
            let conf_space = d.read_conf_space_non_bridge_field()?;
            let bars = conf_space.get_bars()?;
            let (_, mmio_bar) = bars
                .get(1)
                .ok_or(Error::Failed("Failed to read MMIO base address register"))?;
            let mmio_addr = match mmio_bar {
                //BaseAddress::MmioAddressSpace(addr) => (*addr as u64).into(),
                BaseAddress::MemoryAddress32BitSpace(phy_addr, _) => phy_addr.get_virt_addr(),
                _ => return Err(Error::Failed("Invalid base address register")),
            }?;

            // TODO: magic number is 0
            let dev_reg = MmioDeviceRegister::read(mmio_addr);
            println!("{:?}", dev_reg);

            // let mut c_common = None;
            // let mut c_notify = None;
            // let mut c_isr = None;
            // let mut c_device = None;
            // let mut c_pci = None;
            // let mut c_memory = None;
            // let mut c_vendor = None;

            // let caps_list = d.read_msi_caps_list();
            // for c in caps_list {
            //     let ty = (c.msg_ctrl.raw() >> 8) as u8;

            //     if c.cap_id != 0x09 {
            //         continue;
            //     }

            //     match ty {
            //         0x01 => c_common = Some(c),
            //         0x02 => c_notify = Some(c),
            //         0x03 => c_isr = Some(c),
            //         0x04 => c_device = Some(c),
            //         0x05 => c_pci = Some(c),
            //         0x08 => c_memory = Some(c),
            //         0x09 => c_vendor = Some(c),
            //         _ => (),
            //     }
            // }

            // println!("common: {:?}", c_common);
            // println!("notify: {:?}", c_notify);
            // println!("isr: {:?}", c_isr);
            // println!("device: {:?}", c_device);
            // println!("pci: {:?}", c_pci);
            // println!("memory: {:?}", c_memory);
            // println!("vendor: {:?}", c_vendor);

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
