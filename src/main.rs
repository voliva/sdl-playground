use sdl2::{event::Event, keyboard::Keycode, pixels::Color, rect::Point};
use std::time::{Duration, Instant};

mod font;

const SCALE: f32 = 4.0;
const WIDTH: u32 = 128;
const HEIGHT: u32 = 128;
const FPS: u32 = 30;
const NANOS_IN_SEC: u32 = 1_000_000_000;
const TARGET_NANOS: u32 = NANOS_IN_SEC / FPS;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "rust-sdl2 demo",
            (WIDTH as f32 * SCALE) as u32,
            (HEIGHT as f32 * SCALE) as u32 + 2, // MacOS eats the first line, visible when scale=1
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_logical_size(WIDTH, HEIGHT).unwrap(); // Works better than set_scale
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut fps_meter = FpsMeter::new();
    let mut last_fps = "".to_string();
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
        font::print(
            &mut canvas,
            (WIDTH as i32) - (last_fps.len() * 4) as i32,
            0,
            &last_fps,
        )?;

        // for x in 0..WIDTH {
        //     canvas.draw_point(Point::new(x as i32, 0 as i32))?;
        //     canvas.draw_point(Point::new(x as i32, (HEIGHT - 1) as i32))?;
        // }
        // for y in 0..HEIGHT {
        //     canvas.draw_point(Point::new(0 as i32, y as i32))?;
        //     canvas.draw_point(Point::new((WIDTH - 1) as i32, y as i32))?;
        // }
        // for line in 0..HEIGHT / 6 {
        //     font::print(
        //         &mut canvas,
        //         0,
        //         (line * 6) as i32,
        //         "the quick brown fox jumps over",
        //     )?;
        // }
        font::print(&mut canvas, 0, 10, "THE QUICK BROWN FOX JUMPS OVER")?;
        font::print(&mut canvas, 10, 16, "THE LAZY DOG")?;
        font::print(&mut canvas, 0, 22, "the quick brown fox jumps over")?;
        font::print(&mut canvas, 0, 28, "the lazy dog")?;
        font::print(&mut canvas, 0, 34, "\\|@#~[]{}!\"$%&/()=?^*;:_'`+,.-")?;
        font::print(&mut canvas, 0, 40, "…ˇ∧⌂█░▒▤▥◆●☉♥♪✽❎")?;
        font::print(&mut canvas, 0, 46, "➡️⧗⬅️⬆️⬇️🐱😐🅾️웃")?;

        canvas.present();

        let diff = render_start.elapsed().as_nanos().min(NANOS_IN_SEC as u128) as u32;
        if TARGET_NANOS > diff {
            ::std::thread::sleep(Duration::new(0, TARGET_NANOS - diff));
        }

        if let Some(_fps) = fps_meter.register_frame() {
            last_fps = format!("{}", _fps);
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
