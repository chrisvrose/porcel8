use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use clap::Parser;
use log::LevelFilter;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::BlendMode;

use sdl2::render::WindowCanvas;
use simple_logger::SimpleLogger;

use crate::args::Porcel8ProgramArgs;
use crate::device::Device;

use crate::util::EmulatorResult;
use crate::sdl_adapters::sdl_audio_adapter::SdlAudioAdapter;
use crate::sdl_adapters::sdl_graphics_adapter::SdlGraphicsAdapter;
use crate::sdl_adapters::sdl_keyboard_adapter::SdlKeyboardAdapter;

mod args;
mod device;
mod util;
mod sdl_adapters;
mod rom;

fn main() -> EmulatorResult<()> {
    SimpleLogger::new().with_level(LevelFilter::Info).env().init().unwrap();
    let Porcel8ProgramArgs { filename, new_chip8_behaviour, draw_scale, halt_on_invalid } = Porcel8ProgramArgs::parse();

    log::info!("Started emulator");

    let (mut canvas, mut event_pump, audio_queue) = try_initiate_sdl(draw_scale)?;

    let (mut timer, mut sdl_aud_adapter) = SdlAudioAdapter::new_timers(SdlAudioAdapter::AUDIO_FREQUENCY, 0.85, audio_queue);

    let (frame_buffer_for_display, frame_buffer_for_device) = get_frame_buffer_references();
    let (sdl_kb_adapter, device_keyboard) = SdlKeyboardAdapter::new_keyboard();

    timer.start();

    let device = Device::new(timer, frame_buffer_for_device, device_keyboard, new_chip8_behaviour, halt_on_invalid);

    let (device_termination_signal_sender, compute_handle) = start_compute_thread(filename, device)?;


    let mut sdl_graphics_adapter = SdlGraphicsAdapter::new();

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();

    // Compute a frame time offset
    // Thread will sleep for 60fps - (time spent computing)
    let mut frame_timer = std::time::Instant::now();
    'running: loop {
        let last_time = frame_timer.elapsed();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    device_termination_signal_sender.send(()).expect("Could not send");
                    break 'running;
                }
                Event::KeyDown { keycode: Some(keycode), repeat: false, .. } => {
                    sdl_kb_adapter.process_key_down(keycode)?;
                }
                Event::KeyUp { keycode: Some(keycode), repeat: false, .. } => {
                    sdl_kb_adapter.process_key_up(keycode)?;
                }
                _ => {}
            }
        }

        // lock and draw framebuffer
        {
            let lock = frame_buffer_for_display.lock()?;
            sdl_graphics_adapter.draw_screen(lock, &mut canvas)?;
        }
        canvas.present();
        sdl_aud_adapter.process_push_audio()?;
        let sleep_duration = SdlGraphicsAdapter::FRAME_RATE_TIMING - last_time;
        thread::sleep(sleep_duration);
        frame_timer = std::time::Instant::now();
    }

    compute_handle.join().expect("Failed to close compute thread");
    Ok(())
}

fn start_compute_thread(filename: Option<String>, mut device: Device) -> EmulatorResult<(Sender<()>, JoinHandle<()>)> {
    device.set_default_font();

    if let Some(rom_file_location) = filename {
        let rom = rom::load_rom(rom_file_location)?;
        device.load_rom(&rom);
    }

    let (device_termination_signal_sender, device_termination_signal_sender_receiver) = std::sync::mpsc::channel();
    let compute_handle = thread::Builder::new().name("Compute".to_string()).spawn(move || {

        loop {
            let val = device_termination_signal_sender_receiver.try_recv();
            if let Ok(()) = val {
                break;
            } else if let Err(std::sync::mpsc::TryRecvError::Disconnected) = val {
                panic!("Disconnected");
            }
            device.cycle().expect("Failed to execute");
            // Put a bit of delay to slow down execution
            thread::sleep(Duration::from_millis(2))
        }
    })?;
    Ok((device_termination_signal_sender, compute_handle))
}


fn get_frame_buffer_references() -> (Arc<Mutex<Box<[bool; 2048]>>>, Arc<Mutex<Box<[bool; 2048]>>>) {
    let arc = Arc::new(Mutex::new(vec![false; Device::FRAME_BUFFER_SIZE].into_boxed_slice().try_into().unwrap()));
    let arc2 = Arc::clone(&arc);
    (arc, arc2)
}

/// Initiate SDL resources:
/// 1. A window canvas for drawing
/// 2. An event pump for use as an event loop,
/// 3. An Audio queue for sound
fn try_initiate_sdl(draw_scale: f32) -> EmulatorResult<(WindowCanvas, EventPump, AudioQueue<f32>)> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let wanted_spec = AudioSpecDesired {
        channels: Some(1),
        samples: None,
        freq: Some(SdlAudioAdapter::SAMPLING_FREQ),
    };

    let audio_queue = audio_subsystem.open_queue::<f32, _>(None, &wanted_spec)?;

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
    Ok((canvas, event_pump, audio_queue))
}
