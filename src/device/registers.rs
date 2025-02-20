use super::Device;


#[derive(Debug)]
pub struct RegisterFile {
    pub v: [u8; 0x10],
    /// program counter - only u12 technically.
    pub pc: u16,
    /// stack pointer
    pub i: u16,
}

impl RegisterFile {
    pub const DEFAULT_PC_VALUE: u16 = Device::ROM_START as u16;
}

impl Default for RegisterFile{
    fn default() -> Self {
        Self { v: [0;0x10], pc: Self::DEFAULT_PC_VALUE, i: 0 }
    }
}

