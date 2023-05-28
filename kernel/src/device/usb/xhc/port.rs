use crate::arch::addr::VirtualAddress;

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
    pub slot_id: Option<usize>,
    pub config_state: ConfigState,
    // default control pipe
    pub input_context_base_virt_addr: VirtualAddress,
}

impl Port
{
    pub fn new(port_id: usize) -> Self
    {
        return Self {
            port_id,
            slot_id: None,
            config_state: ConfigState::NotConnected,
            input_context_base_virt_addr: VirtualAddress::new(0),
        };
    }

    pub fn port_id(&self) -> usize { return self.port_id; }
}
