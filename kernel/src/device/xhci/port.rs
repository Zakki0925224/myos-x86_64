#[derive(Debug)]
pub struct Port
{
    port_num: usize,
}

impl Port
{
    pub fn new(port_num: usize) -> Self { return Self { port_num }; }
}
