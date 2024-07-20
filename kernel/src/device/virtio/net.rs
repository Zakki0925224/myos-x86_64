use crate::{
    bus::pci::{self, vendor_id},
    device::DeviceDriverFunction,
    error::{Error, Result},
    println,
    util::mutex::Mutex,
};

static mut VIRTIO_NET_DRIVER: Mutex<VirtioNetDriver> = Mutex::new(VirtioNetDriver::new());

struct VirtioNetDriver {
    pci_device_bdf: Option<(usize, usize, usize)>,
}
impl VirtioNetDriver {
    const fn new() -> Self {
        Self {
            pci_device_bdf: None,
        }
    }
}

impl DeviceDriverFunction for VirtioNetDriver {
    fn probe(&mut self) -> Result<()> {
        pci::configure_devices(2, 0, 0, |d| {
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
            let device_name = d
                .conf_space_header()
                .get_device_name()
                .ok_or(Error::Failed("Failed to read PCI device name"))?;

            println!("{} ({}:{}:{})", device_name, bus, device, func);
            Ok(())
        })?;

        Ok(())
    }
}

pub fn probe_and_attach() -> Result<()> {
    let mut driver = unsafe { VIRTIO_NET_DRIVER.try_lock() }?;
    driver.probe()?;
    driver.attach()?;
    Ok(())
}
