use super::*;

pub struct TestDevice {
    base: DeviceDriverBase,
    counter: usize,
}

impl TestDevice {
    pub fn new() -> Self {
        Self {
            base: DeviceDriverBase::new(
                DeviceClass::Test,
                "test deivce",
                None,
                DevicePollMode::Sync,
            ),
            counter: 0,
        }
    }
}

impl DeviceFunction for TestDevice {
    fn get_info(&self) -> DeviceDriverInfo {
        self.base.info.clone()
    }

    fn is_attached(&self) -> bool {
        self.base.attached
    }

    fn poll_mode(&self) -> DevicePollMode {
        self.base.poll_mode
    }

    fn attach(&mut self) -> Result<()> {
        self.base.attached = true;
        Ok(())
    }

    fn detach(&mut self) -> Result<()> {
        self.base.attached = false;
        Ok(())
    }

    fn update_poll(&mut self) -> Result<()> {
        self.counter += 1;
        //info!("TestDevice: counter = {}", self.counter);
        Ok(())
    }
}
