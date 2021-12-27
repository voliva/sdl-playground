use core::panic;
use std::fs::File;

use itertools::Itertools;
use png::ColorType;
use regex::Regex;

#[derive(Debug)]
pub struct Cartridge {
    pub sprite_map: Vec<u8>,
    pub sprite_flags: Vec<u8>,
    pub music: Vec<u8>,
    pub sfx: Vec<u8>,
    pub lua: String,
}

pub fn read_cartridge(filename: &str) -> Result<Cartridge, String> {
    let decoder = png::Decoder::new(File::open(filename).or(Err("Couldn't open file"))?);
    let mut reader = decoder.read_info().or(Err("Couldn't decode png"))?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).or(Err("Couldn't read png"))?;

    if info.color_type != ColorType::Rgba {
        return Err("Invalid cartridge format".to_string());
    }

    let bytes = &buf[..info.buffer_size()];
    let p8_bytes = bytes
        .into_iter()
        .chunks(4)
        .into_iter()
        .map(|v| {
            let (r, g, b, a) = v.collect_tuple().unwrap();
            // p8 bytes take the least 2 significant bits of argb
            let mut result: u8 = 0;
            result = result | (a & 0x03);
            result = (result << 2) | (r & 0x03);
            result = (result << 2) | (g & 0x03);
            result = (result << 2) | (b & 0x03);
            return result;
        })
        .collect_vec();

    let sprite_map = Vec::from(&p8_bytes[0x0..0x3000]);
    let sprite_flags = Vec::from(&p8_bytes[0x3000..0x3100]);
    let music = Vec::from(&p8_bytes[0x3100..0x3200]);
    let sfx = Vec::from(&p8_bytes[0x3200..0x4300]);

    let compressed_lua = &p8_bytes[0x4300..0x8000];
    let lua = decompress_lua(compressed_lua);

    Ok(Cartridge {
        sprite_map,
        sprite_flags,
        music,
        sfx,
        lua,
    })
}

fn decompress_lua(compressed_lua: &[u8]) -> String {
    let fake_lua = if compressed_lua[0..4] == [0, ascii('p'), ascii('x'), ascii('a')] {
        new_decompression(compressed_lua)
    } else if compressed_lua[0..4] == [ascii(':'), ascii('c'), ascii(':'), 0] {
        old_decompression(compressed_lua)
    } else {
        raw_value(compressed_lua)
    };

    fake_lua
        .split("\n")
        .into_iter()
        .map(|line| {
            // TODO improve this
            let mut result = line.trim().to_string();

            if result.starts_with("?") {
                result = format!("print({})", &result[1..])
            }

            if result.contains("!=") {
                result = result.replace("!=", "~=");
            }

            if result.contains("btnp") {
                let re = Regex::new(r"btnp\(([^)]+)\)").unwrap();
                result = re.replace_all(&result, r#"btnp("$1")"#).to_string();
            }

            if result.contains("btn") {
                let re = Regex::new(r"btn\(([^)]+)\)").unwrap();
                result = re.replace_all(&result, r#"btn("$1")"#).to_string();
            }

            if result.contains("\\-") {
                result = result.replace("\\-", "\\\\-");
            }

            result
        })
        .join("\n")
}

fn new_decompression(compressed_lua: &[u8]) -> String {
    let mut reader = BinaryReader::new(Vec::from(compressed_lua));
    // let reader_size = reader.remaining();

    let mut header = vec![];
    for _ in 0..8 {
        header.push(reader.next_u8(8));
    }

    let decompressed_length = (header[4] as u16) << 8 | (header[5] as u16);
    // let compressed_length = (header[6] as u16) << 8 | (header[7] as u16);
    // let max_t = reader_size - (compressed_length as usize) * 8;

    let mut decompressed = vec![];
    let mut mtf = MoveToFront::new();

    while decompressed.len() < (decompressed_length as usize) {
        let header = reader.next_bit();
        if header == 1 {
            let mut unary = 0;
            while reader.next_bit() == 1 {
                unary += 1;
            }
            let unary_mask = (1 << unary) - 1;
            let index = reader.next_u8(4 + unary) + (unary_mask << 4);

            let ascii = mtf.get_and_move(index as usize);
            decompressed.push(ascii);
        } else {
            let offset_bits = if reader.next_bit() == 1 {
                if reader.next_bit() == 1 {
                    5
                } else {
                    10
                }
            } else {
                15
            };

            let offset = reader.next_usize(offset_bits) + 1;
            let mut length = 3;
            loop {
                let part = reader.next_u8(3) as usize;
                length += part;
                if part != 7 {
                    break;
                }
            }
            let start = decompressed.len() - offset;
            while length > 0 {
                let end = (start + length).min(decompressed.len());
                length -= end - start;
                let copy = Vec::from(&decompressed[start..end]);
                decompressed.extend(copy.into_iter());
            }
        }
    }

    return map_emojis(&decompressed);
}

fn old_decompression(_compressed_lua: &[u8]) -> String {
    todo!()
}

fn raw_value(ascii: &[u8]) -> String {
    map_emojis(
        &ascii
            .into_iter()
            .take_while(|v| **v > 0)
            .map(|v| *v)
            .collect_vec(),
    )
}

fn map_emojis(ascii: &Vec<u8>) -> String {
    ascii
        .iter()
        .map(|char| {
            if char.is_ascii() {
                String::from(char::from_u32(*char as u32).unwrap())
            } else {
                // TODO https://www.lexaloffle.com/bbs/?tid=3739
                let str = match char {
                    148 => "⬆️",
                    131 => "⬇️",
                    142 => "🅾️",
                    139 => "⬅️",
                    145 => "➡️",
                    151 => "❎",
                    138 => "⌂",
                    _ => panic!("unmaped special char {}", char),
                };
                // println!("{}", str);
                str.to_string()
            }
        })
        .join("")
}

fn ascii(c: char) -> u8 {
    u32::from(c) as u8
}

struct MoveToFront {
    values: Vec<u8>,
}

impl MoveToFront {
    fn new() -> Self {
        let mut values = vec![];
        for i in 0..=255 {
            values.push(i);
        }
        MoveToFront { values }
    }

    fn get_and_move(&mut self, index: usize) -> u8 {
        if index == 0 {
            return self.values[0];
        }

        let value = self.values.remove(index);
        self.values.insert(0, value);
        return value;
    }
}

struct BinaryReader {
    data: Vec<u8>,
    pointer: usize,
    bit: u8,
}
impl BinaryReader {
    fn new(data: Vec<u8>) -> Self {
        BinaryReader {
            data,
            pointer: 0,
            bit: 7,
        }
    }
    fn _remaining(&self) -> usize {
        (self.data.len() - self.pointer) * 8 - (7 - self.bit as usize)
    }
    fn next_bit(&mut self) -> u8 {
        let v = self.data[self.pointer];
        let ret = (v >> (7 - self.bit)) & 0x01;
        if self.bit == 0 {
            self.bit = 7;
            self.pointer += 1;
        } else {
            self.bit -= 1;
        }

        ret
    }
    fn next_u8(&mut self, n: u8) -> u8 {
        let mut v = 0;
        /*for _ in 0..n {
            v = (v << 1) | self.next_bit()
        }*/
        for i in 0..n {
            if self.next_bit() == 1 {
                v |= 1 << i;
            }
        }
        v
    }
    fn next_usize(&mut self, n: usize) -> usize {
        let mut v = 0;
        for i in 0..n {
            if self.next_bit() == 1 {
                v |= 1 << i;
            }
        }
        // for _ in 0..n {
        //     v = (v << 1) | (self.next_bit() as usize)
        // }
        v
    }
}
