use crate::arch::addr::VirtualAddress;

use super::context::input::InputContext;

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
    pub input_context_base_virt_addr: VirtualAddress,
    pub output_context_base_virt_addr: VirtualAddress,
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
            output_context_base_virt_addr: VirtualAddress::new(0),
        };
    }

    pub fn port_id(&self) -> usize { return self.port_id; }

    pub fn read_input_context(&self) -> InputContext
    {
        return self.input_context_base_virt_addr.read_volatile();
    }

    pub fn write_input_context(&self, input_context: InputContext)
    {
        self.input_context_base_virt_addr.write_volatile(input_context);
    }
}
