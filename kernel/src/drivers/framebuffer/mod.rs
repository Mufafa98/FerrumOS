use crate::utils::custom_types::mut_u8_ptr::MutU8Ptr;
use lazy_static::lazy_static;
use limine::request::FramebufferRequest;
#[used]
#[link_section = ".requests"]
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

lazy_static! {
    pub static ref FRAMEBUFFER: FrameBuffer = FrameBuffer::init().unwrap();
}
#[derive(Debug)]
pub enum FrameBufferError {
    NoFrameBufferFound,
    ResponseNotReady,
}
pub struct FrameBuffer {
    buffer: MutU8Ptr,
    pitch: u64,
    width: u64,
    height: u64,
}
impl FrameBuffer {
    pub fn init() -> Result<Self, FrameBufferError> {
        if let Some(framebuffer_response) = FRAMEBUFFER_REQUEST.get_response() {
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

    pub fn put_pixel(&self, x: u64, y: u64, color: u32) {
        let pixel_offset = y * self.pitch + x * 4;
        unsafe {
            *(self.buffer.add(pixel_offset as usize) as *mut u32) = color;
        }
    }
    pub fn put_pixel_on_coords(&self, x: u64, y: u64, color: u32) {
        unsafe {
            *(self.buffer.add((x + y) as usize) as *mut u32) = color;
        }
    }

    pub fn get_pixel(&self, x: u64, y: u64) -> u32 {
        let pixel_offset = y * self.pitch + x * 4;
        unsafe { *(self.buffer.add(pixel_offset as usize) as *mut u32) }
    }
    pub fn get_pixel_on_coords(&self, x: u64, y: u64) -> u32 {
        unsafe { *(self.buffer.add((x + y) as usize) as *mut u32) }
    }
    pub fn put_pixel_on_square(&self, x: u64, y: u64, color: u32, size: u64) {
        for i in 0..size {
            for j in 0..size {
                self.put_pixel(x + i, y + j, color);
            }
        }
    }

    pub const fn get_width(&self) -> u64 {
        self.width
    }
    pub const fn get_height(&self) -> u64 {
        self.height
    }
    pub const fn get_pitch(&self) -> u64 {
        self.pitch
    }
}
