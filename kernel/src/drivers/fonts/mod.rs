use lazy_static::lazy_static;
use psf_font::PsfFont;

pub mod psf_font;
pub mod text_writer;

const DEFAULT_FONT_DATA_BYTES: &[u8] = include_bytes!("./font_files/Agafari-16.psfu");
