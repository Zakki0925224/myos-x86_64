#[derive(Debug)]
pub struct Slot
{
    slot_id: usize,
}

impl Slot
{
    pub fn new(slot_id: usize) -> Self { return Self { slot_id }; }

    pub fn slot_id(&self) -> usize { return self.slot_id; }
}
