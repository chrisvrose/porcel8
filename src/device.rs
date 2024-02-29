

pub struct Device{
    pub registers:RegisterFile,
    pub memory: Box<[u8;Self::DEVICE_MEMORY_SIZE]>
}

impl Device{
    pub const DEVICE_MEMORY_SIZE:usize = 1024;
    pub fn new()->Device{
        let memory = vec![0u8;Self::DEVICE_MEMORY_SIZE].into_boxed_slice().try_into().unwrap();
        log::trace!("Successfully initiated device memory");
        Device{
            registers: RegisterFile{},
            memory
        }
    }
}

pub struct RegisterFile{

}