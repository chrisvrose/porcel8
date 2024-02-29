use log::LevelFilter;
use simple_logger::SimpleLogger;
use crate::device::{Device};

mod args;
mod device;

fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).env().init().unwrap();
    log::info!("Started emulator");
    let device = Device::new();
}
