use std::sync::mpsc::{self, Receiver, Sender};

use rlua::Context;
use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

use crate::draw_context::DrawContext;

pub enum FnCall {
    Print(String),
    Cursor(i32, i32),
    Cls,
    Spr(i32, i32, i32),
}

pub struct LuaFnProxy {
    sender: Sender<FnCall>,
    receiver: Receiver<FnCall>,
}

impl LuaFnProxy {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        LuaFnProxy { sender, receiver }
    }
    pub fn register_fns(&self, lua_ctx: Context) {
        let globals = lua_ctx.globals();

        let s_cpy = self.sender.clone();
        let print = lua_ctx
            .create_function(move |_, str: String| {
                s_cpy.send(FnCall::Print(str)).unwrap();
                Ok(())
            })
            .unwrap();
        globals.set("print", print).unwrap();

        let s_cpy = self.sender.clone();
        let cursor = lua_ctx
            .create_function(move |_, (x, y): (i32, i32)| {
                s_cpy.send(FnCall::Cursor(x, y)).unwrap();
                Ok(())
            })
            .unwrap();
        globals.set("cursor", cursor).unwrap();

        let s_cpy = self.sender.clone();
        let cls = lua_ctx
            .create_function(move |_, _: ()| {
                s_cpy.send(FnCall::Cls).unwrap();
                Ok(())
            })
            .unwrap();
        globals.set("cls", cls).unwrap();

        let s_cpy = self.sender.clone();
        let spr = lua_ctx
            .create_function(move |_, (i, x, y): (i32, i32, i32)| {
                s_cpy.send(FnCall::Spr(i, x, y)).unwrap();
                Ok(())
            })
            .unwrap();
        globals.set("spr", spr).unwrap();

        globals
            .set(
                "rnd",
                lua_ctx
                    .create_function(|_, max: Option<i32>| {
                        Ok(rand::random::<f64>() * max.unwrap_or(1) as f64)
                    })
                    .unwrap(),
            )
            .unwrap();

        globals
            .set(
                "flr",
                lua_ctx
                    .create_function(|_, value: f64| Ok(value.floor()))
                    .unwrap(),
            )
            .unwrap();

        // TODO Fuck - how to keep a seed? It should be in the draw_context, but it's owned in another thread :(
        globals
            .set(
                "srand",
                lua_ctx.create_function(|_, _: f64| Ok(())).unwrap(),
            )
            .unwrap();
    }
    pub fn exec_fns(&self, draw_context: &mut DrawContext, canvas: &mut Canvas<Window>) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                FnCall::Print(str) => draw_context.print(canvas, &str, &vec![]).unwrap(),
                FnCall::Cursor(x, y) => draw_context.cursor = (x, y),
                FnCall::Cls => {
                    // Does it have a clear color?
                    let original_color = canvas.draw_color();
                    canvas.set_draw_color(Color::BLACK);
                    canvas.clear();
                    canvas.set_draw_color(original_color);
                }
                FnCall::Spr(sprite, x, y) => canvas
                    .copy(
                        &draw_context.spritesheet,
                        Some(Rect::new(8 * (sprite % 16), 8 * (sprite / 16), 8, 8)),
                        Some(Rect::new(x, y, 8, 8)),
                    )
                    .unwrap(),
            };
        }
    }
}
