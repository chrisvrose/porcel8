use std::ops::SubAssign;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;
use crate::device::timer::Timer;


pub struct Device {
    pub registers: RegisterFile,
    pub memory: Box<[u8; Self::DEVICE_MEMORY_SIZE]>,
    pub timer: Timer,
    pub stack: Vec<u16>
}

impl Device {
    pub const DEVICE_MEMORY_SIZE: usize = 2 << 12;
    pub fn new(timer: Timer) -> Device {
        let memory = vec![0u8; Self::DEVICE_MEMORY_SIZE].into_boxed_slice().try_into().unwrap();
        log::trace!("Successfully initiated device memory");
        Device {
            registers: RegisterFile::new(),
            memory,
            stack:Vec::with_capacity(16),
            timer,
        }
    }
}
impl Device{
    const DEFAULT_FONT:[u8;5*16] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80  // F
    ];
    const FONT_DEFAULT_MEM_LOCATION_START:usize = 0x50;
    const FONT_DEFAULT_MEM_LOCATION_END:usize = 0x9F;
    pub fn cycle(&mut self){

    }

    pub fn set_default_font(&mut self){
        self.memory[Self::FONT_DEFAULT_MEM_LOCATION_START..=Self::FONT_DEFAULT_MEM_LOCATION_END].copy_from_slice(&Self::DEFAULT_FONT);
    }

}
impl Drop for Device{
    fn drop(&mut self) {
        self.timer.send_stop_signal()
    }
}

pub struct RegisterFile {
    pub v: [u8; 0x10],
    // program counter - only u12 technically.
    pub pc: u16,
    /// stack pointer
    pub i: u16,
}

impl RegisterFile {
    pub fn new() -> RegisterFile {
        RegisterFile {
            v: [0; 0x10],
            pc: 0x200,
            i: 0,
        }
    }
}
