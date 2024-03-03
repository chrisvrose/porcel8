use std::fs::File;
use std::io::Read;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Duration;
use log::LevelFilter;
use sdl2::audio::{AudioQueue, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::EventPump;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::render::{BlendMode, TextureAccess, WindowCanvas};
use simple_logger::SimpleLogger;
use device::timer::Timer;
use crate::device::Device;
use crate::kb_map::get_key_index;

mod args;
mod device;
mod kb_map;

fn main() {
    SimpleLogger::new().with_level(LevelFilter::Info).env().init().unwrap();
    log::info!("Started emulator");

    let mut timer = Timer::new();
    timer.start();

    let frame_buffer = get_frame_buffer();
    let frame_buffer_for_display = Arc::clone(&frame_buffer);

    let (sender,receiver) = std::sync::mpsc::channel();

    let compute_handle = thread::spawn(move ||{

    let mut device = Device::new(timer, frame_buffer);
        device.set_default_font();
        {
            let rom = load_rom();
            device.load_rom(&rom);
        }

        loop {
            let val = receiver.try_recv();
            if let Ok(()) = val{
                break;
            }else if let Err(std::sync::mpsc::TryRecvError::Disconnected) = val{
                panic!("Disconnected");
            }
            device.cycle();
        }
    });

    let (mut canvas, mut event_pump) = initiate_sdl(8f32);
    let mut fb_sdl = vec![0;3*Device::FRAME_BUFFER_SIZE];

    canvas.set_draw_color(Color::BLACK);
    canvas.clear();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    sender.send(()).expect("Could not send");
                    break 'running;
                }
                Event::KeyDown { keycode: Some(x), repeat: false, .. } => {
                    if let Some(key_val) = get_key_index(x) {
                        log::info!("Key+ {}",key_val)
                    }
                }
                Event::KeyUp { keycode: Some(x), repeat: false, .. } => {
                    if let Some(key_val) = get_key_index(x) {
                        log::info!("Key- {}",key_val)
                    }
                }
                _ => {}
            }
        }


        // The rest of the game loop goes here...
        {
            let lock =  frame_buffer_for_display.lock().expect("Failed to get Display");
            draw_screen(lock,&mut canvas,&mut fb_sdl);
            // log::info!("Framebuffer status: {:?}",lock);
        }
        canvas.present();


        // 60fps - small offset to consider for cpu cycle time
        std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60 - 2000_000));
    }


    compute_handle.join().unwrap();

}

fn draw_screen(frame_buffer: MutexGuard<Box<[u8; 2048]>>, window_canvas: &mut WindowCanvas, x1: &mut Vec<u8>) {
    for (i,pixel) in frame_buffer.iter().enumerate(){
        x1[3*i] = *pixel;
        x1[3*i+1] = *pixel;
        x1[3*i+2] = *pixel;
    }
    drop(frame_buffer);

    let tex_creator = window_canvas.texture_creator();
    let mut tex = tex_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, Device::FRAME_BUFFER_WIDTH as u32, Device::FRAME_BUFFER_HEIGHT as u32).expect("Failed to create tex");
    tex.with_lock(None,|u,i|{
        u.copy_from_slice(x1);
    }).expect("Unwrap tex");
    window_canvas.copy(&tex,None,None);
}

fn get_frame_buffer() -> Arc<Mutex<Box<[u8; 2048]>>> {
    Arc::new(Mutex::new(vec![0u8; Device::FRAME_BUFFER_SIZE].into_boxed_slice().try_into().unwrap()))
}

const ROM_SIZE:usize = 4096 - 0x200;
fn load_rom()->[u8;ROM_SIZE]{
    let mut rom_slice = [0u8;ROM_SIZE];
    let mut file = File::open("roms/ibm_logo.ch8").expect("could not open");
    file.read(&mut rom_slice).expect("Unwrap");
    rom_slice

}

fn initiate_sdl(draw_scale:f32) -> (WindowCanvas, EventPump) {
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

    let window = video_subsystem.window("byte-pusher-emu", window_width,window_height)
        .position_centered()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().build().unwrap();

    canvas.set_scale(draw_scale, draw_scale).expect("Setting scale");

    canvas.set_blend_mode(BlendMode::None);
    canvas.clear();
    canvas.present();
    let event_pump = sdl_context.event_pump().unwrap();
    (canvas, event_pump
     // , audio_queue
    )
}



