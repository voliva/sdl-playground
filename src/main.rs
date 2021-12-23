use sdl2::{
    event::Event, keyboard::Keycode, pixels::Color, rect::Point, render::Canvas, video::Window,
};
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
    let mut draw_context = DrawContext::new();
    canvas.set_draw_color(PALETTE[6]);
    'running: loop {
        let render_start = Instant::now();

        // canvas.set_draw_color(Color::RGB(0, 0, 0));
        // canvas.clear();
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
        // canvas.set_draw_color(PALETTE[6]);
        draw_context.cursor = (0, 6);
        draw_context.print(&mut canvas, "THE QUICK BROWN FOX JUMPS OVER", &vec![11])?;
        draw_context.print(&mut canvas, "THE LAZY DOG", &vec![2])?;
        draw_context.print(&mut canvas, "the quick brown fox jumps over", &vec![3])?;
        draw_context.print(&mut canvas, "the lazy dog", &vec![4])?;
        draw_context.print(&mut canvas, "\\|@#~[]{}!\"$%&/()=?^*;:_'`+,.-", &vec![10])?;
        draw_context.print(&mut canvas, "…ˇ∧⌂█░▒▤▥◆●☉♥♪✽❎", &vec![8])?;
        draw_context.print(&mut canvas, "➡️⧗⬅️⬆️⬇️🐱😐🅾️웃", &vec![9])?;

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

lazy_static::lazy_static! {
    static ref PALETTE: [Color; 16] = [
        Color::RGB(0x00, 0x00, 0x00),
        Color::RGB(0x1d, 0x2b, 0x53),
        Color::RGB(0x7e, 0x25, 0x53),
        Color::RGB(0x00, 0x87, 0x51),
        Color::RGB(0xab, 0x52, 0x36),
        Color::RGB(0x5f, 0x57, 0x4f),
        Color::RGB(0xc2, 0xc3, 0xc7),
        Color::RGB(0xff, 0xf1, 0xe8),
        Color::RGB(0xff, 0x00, 0x4d),
        Color::RGB(0xff, 0xa3, 0x00),
        Color::RGB(0xff, 0xec, 0x27),
        Color::RGB(0x00, 0xe4, 0x36),
        Color::RGB(0x29, 0xad, 0xff),
        Color::RGB(0x83, 0x76, 0x9c),
        Color::RGB(0xff, 0x77, 0xa8),
        Color::RGB(0xff, 0xcc, 0xaa),
    ];
}

struct DrawContext {
    cursor: (i32, i32),
}

impl DrawContext {
    fn new() -> Self {
        DrawContext { cursor: (0, 0) }
    }
    // // TODO variadic arguments
    // fn cursor(&mut self, x: i32, y: i32) {

    // }
    fn print(
        &mut self,
        canvas: &mut Canvas<Window>,
        text: &str,
        optional_args: &Vec<i32>,
    ) -> Result<(), String> {
        let color = if optional_args.len() == 1 {
            Some(optional_args[0])
        } else if optional_args.len() == 3 {
            Some(optional_args[2])
        } else {
            None
        };
        let cursor = if optional_args.len() >= 2 {
            (optional_args[0], optional_args[1])
        } else {
            self.cursor
        };
        self.cursor = cursor.clone();

        if let Some(c_idx) = color {
            canvas.set_draw_color(PALETTE[c_idx as usize]);
        }

        // TODO Multiline
        font::print(canvas, cursor.0, cursor.1, text)?;

        self.cursor.1 = self.cursor.1 + 6;
        if self.cursor.1 > (HEIGHT - 5) as i32 {
            self.cursor.1 = self.cursor.1 - 6;
            // TODO scroll viewport
        }

        Ok(())
    }
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
