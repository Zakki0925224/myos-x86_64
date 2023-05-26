use super::{endpoint::EndpointContext, slot::SlotContext};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct DeviceContext
{
    pub slot_context: SlotContext,
    pub endpoint_contexts: [EndpointContext; 31],
}

impl DeviceContext
{
    pub fn new() -> Self
    {
        return Self {
            slot_context: SlotContext::new(),
            endpoint_contexts: [EndpointContext::new(); 31],
        };
    }
}
