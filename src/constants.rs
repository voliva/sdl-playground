use sdl2::pixels::Color;

pub const SCALE: f32 = 4.0;
pub const WIDTH: u32 = 128;
pub const HEIGHT: u32 = 128;
pub const FPS: u32 = 30;
pub const NANOS_IN_SEC: u32 = 1_000_000_000;
pub const TARGET_NANOS: u32 = NANOS_IN_SEC / FPS;

lazy_static::lazy_static! {
  pub static ref PALETTE: [Color; 16] = [
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