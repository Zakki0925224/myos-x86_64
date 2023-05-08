#[derive(Debug, Clone, Copy)]
pub struct Device
{
    slot_id: usize,
}

impl Device
{
    pub fn new(slot_id: usize) -> Self { return Self { slot_id }; }

    pub fn slot_id(&self) -> usize { return self.slot_id; }
}
