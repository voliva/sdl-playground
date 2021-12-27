use std::time::Instant;

use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

use crate::{
    constants::{NANOS_IN_SEC, WIDTH},
    font,
};

pub struct FpsMeter {
    frames: usize,
    start: Instant,
    last_fps: String,
}

impl FpsMeter {
    pub fn new() -> Self {
        FpsMeter {
            frames: 0,
            start: Instant::now(),
            last_fps: "".to_string(),
        }
    }
    pub fn register_frame(&mut self) {
        self.frames += 1;
        if self.frames == 10 {
            let fps = (self.frames as f64) * (NANOS_IN_SEC as f64)
                / (self.start.elapsed().as_nanos() as f64);
            self.frames = 0;
            self.start = Instant::now();
            self.last_fps = format!("{}", fps.round());
        }
    }
    pub fn render(&self, canvas: &mut Canvas<Window>) {
        let text_width = (self.last_fps.len() * 4) as i32;
        let original_color = canvas.draw_color();
        canvas.set_draw_color(Color::BLACK);
        canvas
            .fill_rect(Rect::new(
                (WIDTH as i32) - text_width - 1,
                0,
                text_width as u32 + 1,
                6,
            ))
            .unwrap();
        canvas.set_draw_color(Color::WHITE);
        font::print(canvas, (WIDTH as i32) - text_width, 0, &self.last_fps).unwrap();
        canvas.set_draw_color(original_color);
    }
}
