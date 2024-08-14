use alloc::vec::Vec;

use crate::{serial_print, serial_println};

#[derive(Debug)]
#[repr(C)]
pub struct Glyphs {
    unicode: Vec<u32>,
    glyph: Vec<Vec<u8>>,
}

impl Glyphs {
    pub fn new(data: &[u8], glyphs_number: u32, glyphs_size: u32) -> Self {
        let mut glyphs_result: Vec<Vec<u8>> = Vec::new();
        let glyph_data = &data[0..(glyphs_number * glyphs_size) as usize];
        let mut glyph_result: Vec<u8> = Vec::new();
        for element in glyph_data.iter() {
            if glyph_result.len() == 16 {
                glyphs_result.push(glyph_result);
                glyph_result = Vec::new();
            }
            glyph_result.push(*element);
        }

        let unicode_data = &data[(glyphs_number * glyphs_size) as usize..];
        let mut hex_buffer: [u8; 4] = [0; 4];
        let mut hex_buffer_index = 0;
        // counter = 0;
        let mut unicode_result: Vec<u32> = Vec::new();
        for element in unicode_data.iter() {
            if *element == 0xff {
                #[cfg(feature = "debug")]
                serial_print!("\n");
                if let Some(unicode) = Self::get_utf8(&hex_buffer, hex_buffer_index) {
                    unicode_result.push(unicode);
                    // result.unicode[counter] = unicode;
                    // counter += 1;
                    #[cfg(feature = "debug")]
                    serial_print!("unicode: {:?}\n", char::from_u32(unicode));
                }

                hex_buffer = [0; 4];
                hex_buffer_index = 0;
            } else {
                if hex_buffer_index < 4 {
                    hex_buffer[hex_buffer_index] = *element;
                    hex_buffer_index += 1;
                }
            }
        }
        // result
        Glyphs {
            unicode: unicode_result,
            glyph: glyphs_result,
        }
    }

    pub fn get_glyph(&self, index: usize) -> &Vec<u8> {
        &self.glyph[index]
    }

    pub fn get_unicode(&self, index: usize) -> u32 {
        self.unicode[index]
    }

    pub fn get_utf8(data: &[u8; 4], size: usize) -> Option<u32> {
        #[cfg(feature = "debug")]
        for i in 0..size {
            serial_print!("{:x} ", data[i]);
        }
        if data[0] & 0b11110000 == 0b11110000 {
            #[cfg(feature = "debug")]
            serial_print!("4 bytes entry ");
            let b1_data: u32 = (data[0] & 0b00000111).into();
            let b2_data: u32 = (data[1] & 0b00111111).into();
            let b3_data: u32 = (data[2] & 0b00111111).into();
            let b4_data: u32 = (data[3] & 0b00111111).into();
            let result = (b1_data << 18) | (b2_data << 12) | (b3_data << 6) | b4_data;
            Some(result)
        } else if data[0] & 0b11100000 == 0b11100000 {
            #[cfg(feature = "debug")]
            serial_print!("3 bytes entry ");
            let b1_data: u32 = (data[0] & 0b00001111).into();
            let b2_data: u32 = (data[1] & 0b00111111).into();
            let b3_data: u32 = (data[2] & 0b00111111).into();
            let result: u32 = (b1_data << 12) | (b2_data << 6) | b3_data;
            Some(result)
        } else if data[0] & 0b11000000 == 0b11000000 {
            #[cfg(feature = "debug")]
            serial_print!("2 bytes entry ");
            let b1_data: u32 = (data[0] & 0b00011111).into();
            let b2_data: u32 = (data[1] & 0b00111111).into();
            let result: u32 = (b1_data << 6) | b2_data;
            Some(result)
        } else if data[0] & 0b00000000 == 0b00000000 {
            #[cfg(feature = "debug")]
            serial_print!("1 byte entry ");
            let result: u32 = data[0] as u32;
            Some(result)
        } else if data[0] & 0b10000000 == 0b10000000 {
            #[cfg(feature = "debug")]
            serial_print!("continuation byte\n");
            None
        } else {
            #[cfg(feature = "debug")]
            serial_print!("error\n");
            None
        }
    }
}
