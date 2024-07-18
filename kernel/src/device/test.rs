use super::*;
use alloc::string::ToString;

#[allow(dead_code)]
struct TestDevice {
    base: DeviceDriverBase,
    counter: usize,
}

impl DeviceFunction for TestDevice {
    fn new() -> Self {
        Self {
            base: DeviceDriverBase::new(
                DeviceClass::Test,
                "test deivce".to_string(),
                None,
                DevicePollMode::Sync,
            ),
            counter: 0,
        }
    }

    fn get_info(&self) -> DeviceDriverInfo {
        self.base.info.clone()
    }

    fn attach(&mut self) -> Result<()> {
        Ok(())
    }

    fn detach(&mut self) -> Result<()> {
        Ok(())
    }

    fn update_poll(&mut self) -> Result<()> {
        self.counter += 1;
        info!("TestDevice: counter = {}", self.counter);
        Ok(())
    }

    async fn update_poll_async(&mut self) -> Result<()> {
        Err(DeviceDriverError::InvalidPollMode(self.base.poll_mode).into())
    }
}
