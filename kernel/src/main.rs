#![no_std]
#![no_main]

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
                    serial_print!(
                        "counter: {:?} coords: {:?} unicode {:?}\n",
                        counter,
                        coords,
                        temp.get_unicode(counter)
                    );
                    display_glyphs(temp.get_glyph(counter), &framebuffer, coords);
                    counter += 1;
                }
            }
            //}
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
struct PsfFont<'a> {
    magic: u32,
    version: u32,
    headersize: u32,
    flags: u32,
    numglyph: u32,
    bytesperglyph: u32,
    height: u32,
    width: u32,
    data: &'a [u8],
}

impl PsfFont<'_> {
    fn from(data: &[u8]) -> Self {
        let data = include_bytes!("./Agafari-16.psfu");
        PsfFont {
            magic: u32::from_le_bytes([data[0], data[1], data[2], data[3]]),
            version: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            headersize: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            flags: u32::from_le_bytes([data[12], data[13], data[14], data[15]]),
            numglyph: u32::from_le_bytes([data[16], data[17], data[18], data[19]]),
            bytesperglyph: u32::from_le_bytes([data[20], data[21], data[22], data[23]]),
            height: u32::from_le_bytes([data[24], data[25], data[26], data[27]]),
            width: u32::from_le_bytes([data[28], data[29], data[30], data[31]]),
            data: &data[32..],
        }
    }

    fn print_glyphs_debug(&self) {
        for j in 0..self.numglyph {
            let start = (j * 16) as usize;
            for i in start..(start + 16) {
                serial_println!("{:0wd$b}", self.data[i], wd = 8);
            }
        }
    }

    fn get_glyph(&self, index: u32) -> [u8; 16] {
        let start = (index * 16) as usize;
        let mut glyph = [0; 16];
        for i in start..(start + 16) {
            glyph[i - start] = self.data[i];
        }
        glyph
    }

    fn get_unicode(&self, index: u32) -> Option<u32> {
        todo!()
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
