use crate::arch::addr::VirtualAddress;

use super::device::Device;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConfigState
{
    NotConnected,
    Reset,
    Enabled,
    AddressingDevice,
    InitializingDevice,
    ConfiguringEndpoints,
    Configured,
}

#[derive(Debug, Clone, Copy)]
pub struct Port
{
    port_id: usize,
    pub device: Option<Device>,
    pub config_state: ConfigState,
    pub input_context_base_virt_addr: VirtualAddress,
}

impl Port
{
    pub fn new(port_id: usize) -> Self
    {
        return Self {
            port_id,
            device: None,
            config_state: ConfigState::NotConnected,
            input_context_base_virt_addr: VirtualAddress::new(0),
        };
    }

    pub fn port_id(&self) -> usize { return self.port_id; }
}
