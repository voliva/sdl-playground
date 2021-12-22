use std::time::{Duration, Instant};

use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Point};

const SCALE: f32 = 4.0;
const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const FPS: u32 = 2000;
const NANOS_IN_SEC: u32 = 1_000_000_000;
const TARGET_NANOS: u32 = NANOS_IN_SEC / FPS;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "rust-sdl2 demo",
            (WIDTH as f32 * SCALE) as u32,
            (HEIGHT as f32 * SCALE) as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_scale(SCALE, SCALE)?;
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut fps_meter = FpsMeter::new();
    'running: loop {
        let render_start = Instant::now();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }
        // The rest of the game loop goes here...

        canvas.set_draw_color(Color::RGB(255, 210, 0));
        for x in 0..WIDTH {
            canvas.draw_point(Point::new(x as i32, 0 as i32))?;
            canvas.draw_point(Point::new(x as i32, (HEIGHT - 1) as i32))?;
        }
        for y in 0..HEIGHT {
            canvas.draw_point(Point::new(0 as i32, y as i32))?;
            canvas.draw_point(Point::new((WIDTH - 1) as i32, y as i32))?;
        }

        canvas.present();

        let diff = render_start.elapsed().as_nanos().min(NANOS_IN_SEC as u128) as u32;
        if TARGET_NANOS > diff {
            ::std::thread::sleep(Duration::new(0, TARGET_NANOS - diff));
        }

        if let Some(_fps) = fps_meter.register_frame() {
            println!("{}", _fps);
        }
    }

    Ok(())
}

struct FpsMeter {
    frames: usize,
    start: Instant,
}

impl FpsMeter {
    fn new() -> Self {
        FpsMeter {
            frames: 0,
            start: Instant::now(),
        }
    }
    fn register_frame(&mut self) -> Option<usize> {
        self.frames += 1;
        if self.frames == 10 {
            let fps = (self.frames as f64) * (NANOS_IN_SEC as f64)
                / (self.start.elapsed().as_nanos() as f64);
            self.frames = 0;
            self.start = Instant::now();
            Some(fps.round() as usize)
        } else {
            None
        }
    }
}
