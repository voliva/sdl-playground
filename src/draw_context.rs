use itertools::Itertools;
use sdl2::{
    render::{Canvas, Texture},
    video::Window,
};

use crate::{
    constants::{HEIGHT, PALETTE},
    font,
};

pub struct DrawContext<'a> {
    pub cursor: (i32, i32),
    pub spritesheet: Texture<'a>,
}

impl<'a> DrawContext<'a> {
    pub fn new(mut spritesheet: Texture<'a>, spritesheet_data: &[u8]) -> Self {
        spritesheet
            .update(None, &transform_spritesheet_data(spritesheet_data), 4 * 128)
            .unwrap();
        DrawContext {
            cursor: (0, 0),
            spritesheet,
        }
    }
    // // TODO variadic arguments
    // fn cursor(&mut self, x: i32, y: i32) {

    // }
    pub fn print(
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

fn transform_spritesheet_data(raw_spritesheet: &[u8]) -> Vec<u8> {
    // TODO mut through mem access
    // TODO palt to change transparency
    let transparent = vec![0, 0, 0, 0];

    raw_spritesheet
        .iter()
        .flat_map(|v| {
            let right = *v >> 4;
            let left = v & 0x0F;

            let left_d = if left == 0 {
                transparent.clone()
            } else {
                let color = PALETTE[left as usize];
                vec![color.r, color.g, color.b, 255]
            };
            let right_d = if right == 0 {
                transparent.clone()
            } else {
                let color = PALETTE[right as usize];
                vec![color.r, color.g, color.b, 255]
            };

            left_d.into_iter().chain(right_d.into_iter()).collect_vec()
        })
        .collect()
}
