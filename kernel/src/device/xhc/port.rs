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
    pub slot_id: usize,
    pub config_state: ConfigState,
}

impl Port
{
    pub fn new(port_id: usize) -> Self
    {
        return Self { port_id, slot_id: 0, config_state: ConfigState::NotConnected };
    }

    pub fn port_id(&self) -> usize { return self.port_id; }
}
