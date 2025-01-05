use super::{endpoint::EndpointContext, slot::SlotContext};

#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct DeviceContext {
    pub slot_context: SlotContext,
    pub endpoint_contexts: [EndpointContext; 31],
}
