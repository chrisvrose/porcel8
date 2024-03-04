use std::sync::MutexGuard;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{TextureAccess, WindowCanvas};
use crate::device::Device;

#[derive(Debug)]
pub struct SdlGraphicsAdapter {
    rgb_frame_buffer: Vec<u8>,
}

impl SdlGraphicsAdapter {
    pub const RGB_COMPONENTS: usize = 3;
    pub const RGB_FRAMEBUFFER_SIZE: usize = Self::RGB_COMPONENTS * Device::FRAME_BUFFER_SIZE;
    pub fn new() -> SdlGraphicsAdapter {
        let rgb_frame_buffer = vec![0; Self::RGB_FRAMEBUFFER_SIZE];
        SdlGraphicsAdapter {
            rgb_frame_buffer
        }
    }
    pub fn draw_screen(&mut self, frame_buffer: MutexGuard<Box<[bool; Device::FRAME_BUFFER_SIZE]>>, window_canvas: &mut WindowCanvas) {
        for (i, pixel) in frame_buffer.iter().enumerate() {
            let col_component = if *pixel { 0xff } else { 0 };
            self.rgb_frame_buffer[3 * i] = col_component;
            self.rgb_frame_buffer[3 * i + 1] = col_component;
            self.rgb_frame_buffer[3 * i + 2] = col_component;
        }
        drop(frame_buffer);

        let tex_creator = window_canvas.texture_creator();
        let mut tex = tex_creator.create_texture(PixelFormatEnum::RGB24, TextureAccess::Streaming, Device::FRAME_BUFFER_WIDTH as u32, Device::FRAME_BUFFER_HEIGHT as u32).expect("Failed to create tex");
        tex.with_lock(None, |u, i| {
            u.copy_from_slice(self.rgb_frame_buffer.as_slice());
        }).expect("Unwrap tex");
        window_canvas.copy(&tex, None, None).expect("Failed to set texture");
    }
}