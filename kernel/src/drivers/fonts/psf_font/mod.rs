use super::super::framebuffer::FrameBuffer;
// use crate::serial_println;
use glyph::Glyphs;
mod glyph;

#[derive(Debug)]
#[repr(C)]
pub struct PsfFont {
    magic: u32,
    version: u32,
    headersize: u32,
    flags: u32,
    numglyph: u32,
    bytesperglyph: u32,
    height: u32,
    width: u32,
    glyphs: Glyphs,
}

impl PsfFont {
    pub fn from(data: &[u8]) -> Self {
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let headersize = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let flags = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let numglyph = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let bytesperglyph = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let height = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let width = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);
        let glyphs = Glyphs::new(&data[32..], numglyph, bytesperglyph);
        let result = PsfFont {
            magic,
            version,
            headersize,
            flags,
            numglyph,
            bytesperglyph,
            height,
            width,
            glyphs,
        };

        result
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }
    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn display_char(
        &self,
        character: char,
        framebuffer: &FrameBuffer,
        position: (u64, u64),
        fg_color: u32,
        bg_color: u32,
        font_size_multiplier: u64,
    ) {
        let glyph = self.find_glyph(character as u32).unwrap_or([0; 16]);
        for row in 0..16_u64 {
            for col in 0..8_u64 {
                let bit = glyph[row as usize] & (1 << (7 - col));
                let pixel_coord = (
                    position.0 + col * font_size_multiplier,
                    position.1 + row * font_size_multiplier,
                );
                let color = if bit != 0 { fg_color } else { bg_color };
                framebuffer.put_pixel_on_square(
                    pixel_coord.0,
                    pixel_coord.1,
                    color,
                    font_size_multiplier,
                );
            }
        }
    }

    // fn print_glyphs_debug(&self) {
    //     for j in 0..self.numglyph {
    //         let start = (j * 16) as usize;
    //         for i in start..(start + 16) {
    //             serial_println!("{:0wd$b}", self.glyphs.get_glyph(j as usize)[i], wd = 8);
    //         }
    //     }
    // }

    fn get_glyph(&self, index: u32) -> [u8; 16] {
        let mut glyph = [0; 16];
        for i in 0..16 {
            glyph[i] = self.glyphs.get_glyph(index as usize)[i];
        }
        glyph
    }
    fn find_glyph(&self, unicode: u32) -> Option<[u8; 16]> {
        for i in 0..self.numglyph {
            if self.glyphs.get_unicode(i as usize) == unicode {
                return Some(self.get_glyph(i));
            }
        }
        None
    }
}
