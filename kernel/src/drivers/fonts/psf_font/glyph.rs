//! This module contains the implementation of the Glyphs struct,
//! which is used to represent the glyphs in a font.
use alloc::vec::Vec;
/// Glyphs struct containing the unicode and glyph data
#[derive(Debug)]
#[repr(C)]
pub struct Glyphs {
    unicode: Vec<u32>,
    glyph: Vec<Vec<u8>>,
}

impl Glyphs {
    /// Creates a new Glyphs struct from the given data
    pub fn new(data: &[u8], glyphs_number: u32, glyphs_size: u32) -> Self {
        let mut glyphs_result: Vec<Vec<u8>> = Vec::new();
        // The bitmap for the character is stored in the data array
        // witch is located in the given data array from
        // 0 to (glyphs_number * glyphs_size)
        let glyph_data = &data[0..(glyphs_number * glyphs_size) as usize];
        let mut glyph_result: Vec<u8> = Vec::new();
        for element in glyph_data.iter() {
            // Each glyph is 16 bytes long
            // TO DO: Make this more general
            if glyph_result.len() == 16 {
                glyphs_result.push(glyph_result);
                glyph_result = Vec::new();
            }
            glyph_result.push(*element);
        }
        // The unicode data is stored in the data array
        // witch is located in the given data array from
        // (glyphs_number * glyphs_size) to the end
        let unicode_data = &data[(glyphs_number * glyphs_size) as usize..];
        // Parse the unicode data
        // TO DO: Improve this using vec
        let mut hex_buffer: [u8; 4] = [0; 4];
        let mut hex_buffer_index = 0;
        let mut unicode_result: Vec<u32> = Vec::new();
        for element in unicode_data.iter() {
            // If the element is 0xff, then the unicode data
            // for the current glyph is finished
            if *element == 0xff {
                if let Some(unicode) = Self::get_utf8(&hex_buffer) {
                    unicode_result.push(unicode);
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
    /// Gets the glyph data for the given index
    pub fn get_glyph(&self, index: usize) -> &Vec<u8> {
        &self.glyph[index]
    }
    /// Gets the unicode data for the given index
    pub fn get_unicode(&self, index: usize) -> u32 {
        self.unicode[index]
    }
    /// Converts a 4 byte array to a u32 utf8 value
    pub fn get_utf8(data: &[u8; 4]) -> Option<u32> {
        if data[0] & 0b11110000 == 0b11110000 {
            // 4 bytes entry
            let b1_data: u32 = (data[0] & 0b00000111).into();
            let b2_data: u32 = (data[1] & 0b00111111).into();
            let b3_data: u32 = (data[2] & 0b00111111).into();
            let b4_data: u32 = (data[3] & 0b00111111).into();
            let result = (b1_data << 18) | (b2_data << 12) | (b3_data << 6) | b4_data;
            Some(result)
        } else if data[0] & 0b11100000 == 0b11100000 {
            // 3 bytes entry
            let b1_data: u32 = (data[0] & 0b00001111).into();
            let b2_data: u32 = (data[1] & 0b00111111).into();
            let b3_data: u32 = (data[2] & 0b00111111).into();
            let result: u32 = (b1_data << 12) | (b2_data << 6) | b3_data;
            Some(result)
        } else if data[0] & 0b11000000 == 0b11000000 {
            // 2 bytes entry
            let b1_data: u32 = (data[0] & 0b00011111).into();
            let b2_data: u32 = (data[1] & 0b00111111).into();
            let result: u32 = (b1_data << 6) | b2_data;
            Some(result)
        } else if data[0] & 0b00000000 == 0b00000000 {
            // Single byte
            let result: u32 = data[0] as u32;
            Some(result)
        } else if data[0] & 0b10000000 == 0b10000000 {
            // Continuation byte
            None
        } else {
            // Error
            None
        }
    }
}
