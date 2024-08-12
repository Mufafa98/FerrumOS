#![no_std]
#![no_main]
#![feature(str_from_raw_parts)]

use ferrum_os::*;

use limine::framebuffer::Framebuffer;
use limine::request::FramebufferRequest;
use x86_64::structures::paging::frame;

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

const FONT_DATA_BYTES: &[u8] = include_bytes!("./Agafari-16.psfu");

#[no_mangle]
unsafe extern "C" fn _start() -> ! {
    //test_framebuffer();
    serial_println!("<-------------------->\n FerrumOs has started\n<-------------------->");
    let temp = PsfFont::from(FONT_DATA_BYTES);
    //temp.print_glyphs_debug();
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            //for i in 0..temp.numglyph {
            let mut counter = 0;
            for i in 0..framebuffer.height() / 16 {
                for j in 0..framebuffer.width() / 8 {
                    let coords: (u64, u64) = (j * 8, i * 16);
                    // serial_print!(
                    //     "#{:?}/{:?} {:?} coords: {:?} unicode {:?}\n",
                    //     counter,
                    //     temp.numglyph,
                    //     temp.bytesperglyph,
                    //     coords,
                    //     temp.get_unicode(counter)
                    // );
                    // serial_println!("{:?}", temp.get_glyph(counter));

                    // display_glyphs(temp.get_glyph(counter), &framebuffer, coords);
                    // counter += 1;
                    // if counter >= temp.numglyph {
                    //     hlt_loop();
                    // }
                    let position = (j * 8, i * 16);
                    let string = "Hello world!";
                    display_glyphs(
                        temp.find_glyph(string.chars().nth(counter as usize).unwrap() as u32)
                            .unwrap_or([0; 16]),
                        &framebuffer,
                        position,
                    );
                    counter += 1;
                }
            }
        }
    }
    //serial_println!("{:?}", temp);
    hlt_loop();
}

unsafe fn test_framebuffer() {
    // Ensure we got a framebuffer.
    if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
        if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
            for i in 0..100_u64 {
                // Calculate the pixel offset using the framebuffer information we obtained above.
                // We skip `i` scanlines (pitch is provided in bytes) and add `i * 4` to skip `i` pixels forward.
                let pixel_offset = i * framebuffer.pitch() + i * 4;

                // Write 0xFFFFFFFF to the provided pixel offset to fill it white.
                *(framebuffer.addr().add(pixel_offset as usize) as *mut u32) = 0x00FFFFFF;
            }
        }
    }
}

#[derive(Debug)]
#[repr(C)]
struct PsfFont {
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
#[derive(Debug)]
#[repr(C)]
struct Glyphs {
    unicode: [u32; 512],
    glyph: [[u8; 16]; 512],
}

impl Glyphs {
    fn new(data: &[u8], glyphs_number: u32, glyphs_size: u32) -> Self {
        let mut result = Glyphs {
            glyph: [[0_u8; 16]; 512],
            unicode: [0_u32; 512],
        };
        let mut counter = 0;
        let glyph_data = &data[0..(glyphs_number * glyphs_size) as usize];
        for element in glyph_data.iter() {
            //serial_println!("{:0wd$b}", *element, wd = 8);
            let index = counter / 16;
            let offset = counter % 16;
            result.glyph[index as usize][offset as usize] = *element;
            counter += 1;
        }
        let unicode_data = &data[(glyphs_number * glyphs_size) as usize..];
        let mut hex_buffer: [u8; 4] = [0; 4];
        let mut hex_buffer_index = 0;
        counter = 0;
        for element in unicode_data.iter() {
            if *element == 0xff {
                #[cfg(feature = "debug")]
                serial_print!("\n");
                if let Some(unicode) = Self::get_utf8(&hex_buffer, hex_buffer_index) {
                    result.unicode[counter] = unicode;
                    counter += 1;
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
        result
    }

    fn get_glyph(&self, index: usize) -> [u8; 16] {
        self.glyph[index]
    }

    fn get_utf8(data: &[u8; 4], size: usize) -> Option<u32> {
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

impl PsfFont {
    fn from(data: &[u8]) -> Self {
        let magic = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        let headersize = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
        let flags = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let numglyph = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let bytesperglyph = u32::from_le_bytes([data[20], data[21], data[22], data[23]]);
        let height = u32::from_le_bytes([data[24], data[25], data[26], data[27]]);
        let width = u32::from_le_bytes([data[28], data[29], data[30], data[31]]);

        let result = PsfFont {
            magic,
            version,
            headersize,
            flags,
            numglyph,
            bytesperglyph,
            height,
            width,
            glyphs: Glyphs::new(&data[32..], numglyph, bytesperglyph),
        };

        result
    }

    fn print_glyphs_debug(&self) {
        for j in 0..self.numglyph {
            let start = (j * 16) as usize;
            for i in start..(start + 16) {
                serial_println!("{:0wd$b}", self.glyphs.get_glyph(j as usize)[i], wd = 8);
            }
        }
    }

    fn get_glyph(&self, index: u32) -> [u8; 16] {
        let mut glyph = [0; 16];
        for i in 0..16 {
            glyph[i] = self.glyphs.get_glyph(index as usize)[i];
        }
        glyph
    }
    fn find_glyph(&self, unicode: u32) -> Option<[u8; 16]> {
        for i in 0..self.numglyph {
            if self.glyphs.unicode[i as usize] == unicode {
                return Some(self.get_glyph(i));
            }
        }
        None
    }
}

fn display_glyphs(glymph: [u8; 16], framebuffer: &Framebuffer, position: (u64, u64)) {
    let mut position = position;
    for row in 0..16_u64 {
        for col in 0..8_u64 {
            let bit = glymph[row as usize] & (1 << (7 - col));
            if bit != 0 {
                let pixel_coord = (position.0 + col, position.1 + row);
                let pixel_offset = pixel_coord.1 * framebuffer.pitch() + pixel_coord.0 * 4;
                unsafe {
                    *(framebuffer.addr().add(pixel_offset as usize) as *mut u32) = 0x00FFFFFF;
                }
            }
        }
    }
}
