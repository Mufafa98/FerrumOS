//! Framebuffer driver for drawing on the screen.
use crate::{serial_println, utils::custom_types::mut_u8_ptr::MutU8Ptr};
use lazy_static::lazy_static;
use limine::request::FramebufferRequest;
#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

lazy_static! {
    /// The global framebuffer instance.
    pub static ref FRAMEBUFFER: FrameBuffer = FrameBuffer::init().unwrap();
}
/// Framebuffer error type.
#[derive(Debug)]
pub enum FrameBufferError {
    NoFrameBufferFound,
    ResponseNotReady,
}
/// Framebuffer struct.
pub struct FrameBuffer {
    buffer: MutU8Ptr,
    pitch: u64,
    width: u64,
    height: u64,
}
impl FrameBuffer {
    /// Initializes the framebuffer.
    pub fn init() -> Result<Self, FrameBufferError> {
        if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
            // Get the first framebuffer.
            // TO DO : Handle multiple framebuffers.
            // serial_println!(
            //     "Framebuffers found: {:?}",
            //     framebuffer_response.framebuffers().count()
            // );
            // serial_println!(
            //     "Framebuffer size: {} {}",
            //     framebuffer_response.framebuffers().next().unwrap().width(),
            //     framebuffer_response.framebuffers().next().unwrap().height()
            // );
            if let Some(framebuffer) = framebuffer_response.framebuffers().next() {
                return Ok(FrameBuffer {
                    buffer: MutU8Ptr::new(framebuffer.addr()),
                    pitch: framebuffer.pitch(),
                    width: framebuffer.width(),
                    height: framebuffer.height(),
                });
            }
            return Err(FrameBufferError::NoFrameBufferFound);
        }
        return Err(FrameBufferError::ResponseNotReady);
    }
    /// Puts a pixel on the screen at the given coordinates with the given color.
    pub fn put_pixel(&self, x: u64, y: u64, color: u32) {
        let pixel_offset = y * self.pitch + x * 4;
        unsafe {
            *(self.buffer.add(pixel_offset as usize) as *mut u32) = color;
        }
    }
    /// Puts a pixel on the screen at the given coordinates with the given color.
    /// This function uses already calculated offset.
    pub fn put_pixel_on_coords(&self, x: u64, y: u64, color: u32) {
        unsafe {
            *(self.buffer.add((x + y) as usize) as *mut u32) = color;
        }
    }
    /// Gets a pixel from the screen at the given coordinates.
    pub fn get_pixel(&self, x: u64, y: u64) -> u32 {
        let pixel_offset = y * self.pitch + x * 4;
        // unsafe { *(self.buffer.add((y * self.pitch + x * 4) as usize)) as *mut u32 }
        unsafe { *(self.buffer.add(pixel_offset as usize) as *mut u32) }
    }
    /// Gets a pixel from the screen at the given coordinates.
    /// This function uses already calculated offset.
    pub fn get_pixel_on_coords(&self, x: u64, y: u64) -> u32 {
        unsafe { *(self.buffer.add((x + y) as usize) as *mut u32) }
    }
    /// Draws a square on the screen at the given coordinates with the given color.
    pub fn put_pixel_on_square(&self, x: u64, y: u64, color: u32, size: u64) {
        for i in 0..size {
            for j in 0..size {
                self.put_pixel(x + i, y + j, color);
            }
        }
    }
    /// Gets the width of the framebuffer.
    pub const fn get_width(&self) -> u64 {
        self.width
    }
    /// Gets the height of the framebuffer.
    pub const fn get_height(&self) -> u64 {
        self.height
    }
    /// Gets the pitch of the framebuffer.
    /// The pitch is the number of bytes between two lines.
    pub const fn get_pitch(&self) -> u64 {
        self.pitch
    }
}
