mod constants;
mod draw_context;
mod font;
mod fps_meter;
mod lua_fn_proxy;
mod png_reader;

use constants::{HEIGHT, NANOS_IN_SEC, SCALE, TARGET_NANOS, WIDTH};
use draw_context::DrawContext;
use fps_meter::FpsMeter;
use itertools::Itertools;
use lua_fn_proxy::LuaFnProxy;
use rlua::{Function, Lua};
use sdl2::{
    event::{Event, WindowEvent},
    keyboard::Keycode,
    pixels::{Color, PixelFormatEnum},
    render::TextureAccess,
};
use std::time::{Duration, Instant};

use crate::constants::PALETTE;

fn main() -> Result<(), String> {
    let cartridge = png_reader::read_cartridge("minewalker.p8.png")?;

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
    // canvas.set_logical_size(WIDTH, HEIGHT).unwrap(); // Works better than set_scale // Not needed anymore after frame texture
    let mut event_pump = sdl_context.event_pump().unwrap();

    let lua = Lua::new();

    // let mut prints = vec![];
    let lua_fn_proxy = LuaFnProxy::new();
    lua.context(|lua_ctx| lua_fn_proxy.register_fns(lua_ctx));

    // lua.context(|lua_ctx| {
    //     lua_ctx
    //         .load(
    //             r#"
    //     y = 0
    //     x = 0

    //     function _init()
    //         if 1 ~= 2 then
    //             y = 100
    //         end
    //         if true then y = 0 end
    //         -- y += 3
    //         x = 30
    //         boo = "\\-0\\-5🅾️ to mark obvious nearby mines"
    //     end

    //     function _update()
    //         y = (y + 1) % 128
    //     end

    //     function _draw()
    //         cls()
    //         cursor(x,y)
    //         print("hello!!")
    //         spr(1, 90, y)
    //     end
    // "#,
    //         )
    //         .exec()
    //         .unwrap()
    // });

    lua.context(|lua_ctx| {
        let result = lua_ctx.load(&cartridge.lua).exec();
        match result {
            Ok(_) => {}
            Err(err) => {
                let lines = cartridge.lua.split("\n").collect_vec();
                println!("{:#?}", &lines[339..343]);
                println!("{:#?}", err);

                panic!("couldn't parse lua code");
            }
        }
    });

    lua.context(|lua_ctx| {
        let globals = lua_ctx.globals();

        let init: Function = globals.get("_init").unwrap();
        init.call::<_, ()>(()).unwrap();
    });

    'initializing: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Window {
                    win_event: WindowEvent::Exposed,
                    ..
                } => {
                    println!("{:?}", event);
                    break 'initializing;
                }
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    return Err("Terminated".to_string());
                }
                _ => println!("{:?}", event),
            }
        }
        std::thread::sleep(Duration::new(0, 1_000_000));
    }

    let mut fps_meter = FpsMeter::new();
    let texture_creator = canvas.texture_creator();

    let spritesheet = texture_creator
        .create_texture(
            Some(PixelFormatEnum::RGBA8888),
            TextureAccess::Static,
            128,
            128,
        )
        .unwrap();
    let raw_spritesheet = &(cartridge.sprite_map[..0x2000]);
    let mut draw_context = DrawContext::new(spritesheet, raw_spritesheet);

    let mut frame = texture_creator
        .create_texture(None, TextureAccess::Target, WIDTH, HEIGHT)
        .unwrap();
    canvas
        .with_texture_canvas(&mut frame, |canvas| canvas.clear())
        .unwrap();

    canvas.set_draw_color(PALETTE[6]);
    'running: loop {
        let render_start = Instant::now();

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

        lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();

            let update: Function = globals.get("_update").unwrap();
            let draw: Function = globals.get("_draw").unwrap();

            // TODO separate?
            update.call::<_, ()>(()).unwrap();
            draw.call::<_, ()>(()).unwrap();
        });

        canvas
            .with_texture_canvas(&mut frame, |canvas| {
                lua_fn_proxy.exec_fns(&mut draw_context, canvas);

                fps_meter.render(canvas);
            })
            .unwrap();

        let original_color = canvas.draw_color();
        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        canvas.set_draw_color(original_color);
        canvas.copy(&frame, None, None)?;
        canvas.present();

        let diff = render_start.elapsed().as_nanos().min(NANOS_IN_SEC as u128) as u32;
        if TARGET_NANOS > diff {
            std::thread::sleep(Duration::new(0, TARGET_NANOS - diff));
        }

        fps_meter.register_frame();
    }

    Ok(())
}
