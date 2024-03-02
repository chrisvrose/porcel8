use log::LevelFilter;
use simple_logger::SimpleLogger;
use device::timer::Timer;
use crate::device::Device;

mod args;
mod device;

fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).env().init().unwrap();
    log::info!("Started emulator");

    let mut timer = Timer::new();
    timer.start();

    let mut device = Device::new(timer);
    device.set_default_font();
    let mut i = 0;



}



