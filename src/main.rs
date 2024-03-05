use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use log::LevelFilter;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;

use sdl2::render::WindowCanvas;
use simple_logger::SimpleLogger;
use device::timer::Timer;
use crate::device::Device;
use crate::device::keyboard::Keyboard;
use crate::util::EmulatorResult;
use crate::kb_map::get_key_index;
use crate::sdl_graphics_adapter::SdlGraphicsAdapter;
use crate::sdl_keyboard_adapter::SdlKeyboardAdapter;

mod args;
mod device;
mod kb_map;
mod sdl_graphics_adapter;
mod util;
mod sdl_keyboard_adapter;

fn main() -> EmulatorResult<()> {
    SimpleLogger::new().with_level(LevelFilter::Info).env().init().unwrap();
    log::info!("Started emulator");

    let mut timer = Timer::new();
    timer.start();

    let (frame_buffer_for_display, frame_buffer_for_device) = get_frame_buffer_references();
    
    let (sdl_kb_adapter,device_keyboard) = SdlKeyboardAdapter::new_keyboard();

    let (termination_signal_sender, termination_signal_sender_receiver) = std::sync::mpsc::channel();
    
    let compute_handle = thread::Builder::new().name("Compute".to_string()).spawn(move || {
        do_device_loop(timer, frame_buffer_for_device, termination_signal_sender_receiver, device_keyboard);
    })?;

    let (mut canvas, mut event_pump) = try_initiate_sdl(8f32)?;

    let mut sdl_graphics_adapter = SdlGraphicsAdapter::new();

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    termination_signal_sender.send(()).expect("Could not send");
                    break 'running;
                }
                Event::KeyDown { keycode: Some(x), repeat: false, .. } => {
                    if let Some(key_val) = get_key_index(x) {
                        sdl_kb_adapter.send_key_down(key_val)?;
                        log::info!("Key+ {}",key_val)
                    }
                }
                Event::KeyUp { keycode: Some(x), repeat: false, .. } => {
                    if let Some(key_val) = get_key_index(x) {
                        sdl_kb_adapter.send_key_up(key_val)?;
                        log::info!("Key- {}",key_val)
                    }
                }
                _ => {}
            }
        }


        // The rest of the game loop goes here...
        {
            let lock = frame_buffer_for_display.lock()?;
            sdl_graphics_adapter.draw_screen(lock, &mut canvas)?;
        }
        canvas.present();

        // 60fps - small offset to consider for cpu cycle time
        thread::sleep(Duration::new(0, 1_000_000_000u32 / 60 ));
    }

    compute_handle.join().expect("Failed to close compute thread");
    Ok(())
}

fn do_device_loop(mut timer: Timer, frame_buffer: Arc<Mutex<Box<[bool; 2048]>>>, receiver: Receiver<()>, device_keyboard: Keyboard) {
    let mut device = Device::new(timer, frame_buffer, device_keyboard);
    device.set_default_font();
    {
        let rom = load_rom();
        device.load_rom(&rom);
    }

    loop {
        let val = receiver.try_recv();
        if let Ok(()) = val {
            break;
        } else if let Err(std::sync::mpsc::TryRecvError::Disconnected) = val {
            panic!("Disconnected");
        }
        device.cycle().expect("Failed to execute");
        // Put a bit of delay to slow down execution
        thread::sleep(Duration::from_nanos(500))
    }
}


fn get_frame_buffer_references() -> (Arc<Mutex<Box<[bool; 2048]>>>, Arc<Mutex<Box<[bool; 2048]>>>) {
    let arc = Arc::new(Mutex::new(vec![false; Device::FRAME_BUFFER_SIZE].into_boxed_slice().try_into().unwrap()));
    let arc2 = Arc::clone(&arc);
    (arc,arc2)
}

const ROM_SIZE: usize = 4096 - 0x200;

fn load_rom() -> [u8; ROM_SIZE] {
    let mut rom_slice = [0u8; ROM_SIZE];
    let mut file = File::open("roms/ibm_logo.ch8").expect("could not open");
    file.read(&mut rom_slice).expect("Unwrap");
    rom_slice
}

fn try_initiate_sdl(draw_scale: f32) -> EmulatorResult<(WindowCanvas, EventPump)> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    // let audio_subsystem = sdl_context.audio().unwrap();
    // let wanted_spec = AudioSpecDesired {
    //     channels: Some(1),
    //     samples: Some(256),
    //     freq: Some(15360),
    // };
    // let audio_queue = audio_subsystem.open_queue::<u8, _>(None, &wanted_spec).unwrap();
    // audio_queue.resume();
    let window_width = (Device::FRAME_BUFFER_WIDTH as f32 * draw_scale) as u32;
    let window_height = (Device::FRAME_BUFFER_HEIGHT as f32 * draw_scale) as u32;

    let window = video_subsystem.window("porcel8", window_width, window_height)
        .position_centered()
        .build()?;
    let mut canvas = window.into_canvas().build()?;

    canvas.set_scale(draw_scale, draw_scale)?;

    canvas.set_blend_mode(BlendMode::None);
    canvas.clear();
    canvas.present();
    let event_pump = sdl_context.event_pump()?;
    Ok((canvas, event_pump
     // , audio_queue
    ))
}



