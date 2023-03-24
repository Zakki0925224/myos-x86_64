#[derive(Debug)]
pub struct Device
{
    port_num: usize,
}

impl Device
{
    pub fn new(port_num: usize) -> Self { return Self { port_num }; }
}
