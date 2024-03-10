use std::fs::File;
use std::io::Read;
use crate::util::EmulatorResult;

pub const ROM_SIZE: usize = 4096 - 0x200;

pub fn load_rom(rom_file_location: String) -> EmulatorResult<[u8; ROM_SIZE]> {
    let mut rom_slice = [0u8; ROM_SIZE];
    let mut file = File::open(rom_file_location)?;
    file.read(&mut rom_slice)?;
    Ok(rom_slice)
}
