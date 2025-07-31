//! Fonts module.
pub mod ansii_parser;
pub mod color;
pub mod psf_font;
pub mod text_writer;

const DEFAULT_FONT_DATA_BYTES: &[u8] = include_bytes!("./font_files/Agafari-16.psfu");
