use alloc::vec::Vec;

use crate::{
    drivers::fonts::color::{self, colors},
    serial_print, serial_println,
};

pub mod ansii_builder;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnsiiParserState {
    Anywhere = 0,
    CsiEntry = 1,
    CsiIgnore = 2,
    CsiIntermediate = 3,
    CsiParam = 4,
    DcsEntry = 5,
    DcsIgnore = 6,
    DcsIntermediate = 7,
    DcsParam = 8,
    DcsPassthrough = 9,
    Escape = 10,
    EscapeIntermediate = 11,
    Ground = 12,
    OscString = 13,
    SosPmApcString = 14,
    Utf8 = 15,
}

pub trait Performer {
    fn print(&mut self, c: char);
    fn set_bg_color(&mut self, color: u32);
    fn set_fg_color(&mut self, color: u32);
    fn backspace(&mut self);
    fn clear_screen(&mut self);
    fn move_cursor(&mut self, row: u64, col: u64);
    fn save_cursor(&mut self);
    fn print_cursor_position(&mut self);
    fn move_cursor_to_start(&mut self);
    fn get_cursor_position(&self) -> (u64, u64);
    fn get_saved_cursor_position(&self) -> Option<(u64, u64)>;
    fn set_bold(&mut self, bold: bool);
}

pub struct AnsiiParser<'a, T: Performer> {
    state: AnsiiParserState,
    params: Vec<u16>,
    intermediate: Vec<u8>,
    performer: &'a mut T,
}

impl<'a, T: Performer> AnsiiParser<'a, T> {
    pub fn new(performer: &'a mut T) -> Self {
        AnsiiParser {
            state: AnsiiParserState::Ground,
            params: Vec::new(),
            intermediate: Vec::new(),
            performer,
        }
    }

    pub fn parse(&mut self, byte: u8) {
        // serial_println!(
        //     "ANSII Parser: Parsing byte: {} in state: {:?}",
        //     byte,
        //     self.state
        // );
        match self.state {
            AnsiiParserState::Ground => {
                // asd
                match byte {
                    b'\x1B' => {
                        self.state = AnsiiParserState::Escape;
                    }
                    b'\x08' => {
                        self.performer.backspace();
                    }
                    b'\x07' => {
                        // save cursor
                        self.performer.save_cursor();
                    }
                    b'\x0D' => {
                        // carriage return
                        self.performer.move_cursor_to_start();
                    }
                    _ => {
                        // Normal character, print it
                        self.performer.print(byte as char);
                    }
                }
            }
            AnsiiParserState::Escape => {
                // asd
                match byte {
                    b'\x5B' => {
                        self.state = AnsiiParserState::CsiEntry;
                        self.params.clear();
                        self.intermediate.clear();
                    }
                    _ => {
                        self.state = AnsiiParserState::Ground;
                        serial_println!("Escape sequence not implemented for byte: {}", byte);
                    }
                }
            }
            AnsiiParserState::CsiEntry => {
                // asd
                match byte {
                    b'\x30'..=b'\x3F' => {
                        if byte <= b'\x3B' {
                            // parameters
                            self.params.push((byte - b'\x30') as u16);
                            self.state = AnsiiParserState::CsiParam;
                        } else {
                            // collect
                            self.state = AnsiiParserState::Ground;
                            serial_println!("Csi Param not implemented for byte: {}", byte);
                        }
                    }
                    b'\x20'..=b'\x2F' => {
                        // intermediate bytes
                        self.intermediate.push(byte);
                        self.state = AnsiiParserState::CsiIntermediate;
                    }
                    b'\x40'..=b'\x7E' => {
                        // dispatch
                        self.csi_dispatch(byte);
                        self.state = AnsiiParserState::Ground;
                    }
                    _ => {
                        self.state = AnsiiParserState::Ground;
                        serial_println!("CSI entry not implemented for byte: {}", byte)
                    }
                }
                {}
            }
            AnsiiParserState::CsiParam => {
                //asd
                match byte {
                    b'\x40'..=b'\x7E' => {
                        self.params.push(b'\x0B' as u16);
                        self.csi_dispatch(byte);
                        self.state = AnsiiParserState::Ground;
                    }
                    b'\x3A' => {
                        // ignore
                        self.state = AnsiiParserState::CsiIgnore;
                    }
                    b'\x30'..=b'\x3B' => {
                        // parameters
                        self.params.push((byte - b'\x30') as u16);
                        self.state = AnsiiParserState::CsiParam;
                    }
                    _ => {
                        self.state = AnsiiParserState::Ground;
                        serial_println!("Csi Param not implemented for byte: {}", byte);
                    }
                }
            }
            AnsiiParserState::CsiIntermediate => {
                //asd
                match byte {
                    b'\x40'..=b'\x7E' => {
                        self.csi_dispatch(byte);
                        self.state = AnsiiParserState::Ground;
                    }
                    _ => {
                        self.state = AnsiiParserState::Ground;
                        serial_println!("Csi Intermediate not implemented for byte: {}", byte);
                    }
                }
            }
            _ => {
                serial_println!(
                    "ANSII parser state not implemented for state: {:?}",
                    self.state
                )
            }
        }
    }

    pub fn csi_dispatch(&mut self, code: u8) {
        match code {
            b'm' => {
                // SGR - Select Graphic Rendition
                use super::color::{colors::ColorType, Color};
                let mut b_param = 0;
                let mut color_counter = 0;
                let mut color = Color::default();
                let mut color_type = ColorType::None;
                #[derive(PartialEq, Eq, Debug)]
                enum ColorSet {
                    RGB,
                    Indexed,
                    None,
                }
                let mut color_set = ColorSet::None;
                for param in self.params.iter() {
                    if *param == b'\x0B' as u16 {
                        if color_type == ColorType::None || color_set == ColorSet::None {
                            match b_param {
                                0 => {
                                    // reset all styles
                                    self.performer
                                        .set_fg_color(Color::new(255, 255, 255, 255).to_u32());
                                    self.performer
                                        .set_bg_color(Color::new(0, 0, 0, 255).to_u32());
                                    self.performer.set_bold(false);
                                }
                                1 => {
                                    // bold
                                    self.performer.set_bold(true);
                                    self.state = AnsiiParserState::Ground;
                                }
                                2 => {
                                    if color_type == ColorType::None {
                                        // dim
                                        self.state = AnsiiParserState::Ground;
                                        todo!("Dim not implemented");
                                    } else {
                                        color_set = ColorSet::RGB;
                                    }
                                }
                                3..=29 => {
                                    self.state = AnsiiParserState::Ground;
                                    todo!("Style not implemented for code: {}", b_param);
                                }
                                30..=37 | 90..=97 => {
                                    color = {
                                        if b_param >= 90 {
                                            colors::get_color(b_param - 60, true)
                                        } else {
                                            colors::get_color(b_param, false)
                                        }
                                    };
                                    self.performer.set_fg_color(color.to_u32());
                                }
                                40..=47 | 100..=107 => {
                                    color = {
                                        if b_param >= 100 {
                                            colors::get_color(b_param - 60, true)
                                        } else {
                                            colors::get_color(b_param, false)
                                        }
                                    };
                                    self.performer.set_bg_color(color.to_u32());
                                }
                                38 => {
                                    color_type = ColorType::Foreground;
                                }
                                48 => {
                                    color_type = ColorType::Background;
                                }
                                _ => {
                                    self.state = AnsiiParserState::Ground;
                                    panic!("Unsupported SGR code: {}", b_param);
                                }
                            }
                        } else {
                            match color_set {
                                ColorSet::Indexed => {
                                    self.state = AnsiiParserState::Ground;
                                    panic!("Indexed color not implemented yet");
                                }
                                ColorSet::RGB => {
                                    //
                                    match color_counter {
                                        0 => {
                                            color.r = b_param as u8;
                                            color_counter += 1;
                                        }
                                        1 => {
                                            color.g = b_param as u8;
                                            color_counter += 1;
                                        }
                                        2 => {
                                            color.b = b_param as u8;
                                            color_counter = 0;
                                            match color_type {
                                                ColorType::Foreground => {
                                                    self.performer.set_fg_color(color.to_u32())
                                                }
                                                ColorType::Background => {
                                                    self.performer.set_bg_color(color.to_u32())
                                                }
                                                ColorType::None => {
                                                    panic!("Ilegal color type state");
                                                }
                                            }
                                            color_type = ColorType::None;
                                            color_set = ColorSet::None;
                                        }
                                        _ => {
                                            self.state = AnsiiParserState::Ground;
                                            panic!(
                                                "Invalid color counter state: {}",
                                                color_counter
                                            );
                                        }
                                    }
                                }
                                ColorSet::None => {
                                    self.state = AnsiiParserState::Ground;
                                    panic!("Ilegal color set state");
                                }
                            }
                        }
                        b_param = 0;
                    } else {
                        b_param = b_param * 10 + *param;
                    }
                }
                self.params.clear();
                self.state = AnsiiParserState::Ground;
            }
            b'J' => {
                // ED - Erase in Display
                match self.params.get(0).map(|p| *p as u8) {
                    Some(2) | None => {
                        // Clear entire screen
                        self.performer.clear_screen();
                    }
                    _ => {
                        self.state = AnsiiParserState::Ground;
                        serial_println!("ED code not implemented: {:?}", self.params);
                    }
                }
                self.intermediate.clear();
                self.state = AnsiiParserState::Ground;
            }
            b'H' => {
                // CUP - Cursor Position
                if self.params.len() == 4 {
                    let row = self.params[0] as u64;
                    let col = self.params[2] as u64;
                    self.performer.move_cursor(row, col);
                } else if self.params.len() == 0 {
                } else {
                    self.state = AnsiiParserState::Ground;
                    serial_println!("CUP code not implemented with params: {:?}", self.params);
                }
                self.state = AnsiiParserState::Ground;
            }
            b'n' => {
                // DSR - Device Status Report
                match self.params.get(0).map(|p| *p as u8) {
                    Some(6) => {
                        // Report cursor position
                        self.performer.print_cursor_position();
                    }
                    _ => {
                        self.state = AnsiiParserState::Ground;
                        serial_println!("DSR code not implemented: {:?}", self.params);
                    }
                }
                self.state = AnsiiParserState::Ground;
            }
            b's' => {
                // Save Cursor
                self.performer.save_cursor();
                let saved_pos = self.performer.get_saved_cursor_position().unwrap();
                serial_println!("Cursor saved at position: {:?}", saved_pos);
                self.state = AnsiiParserState::Ground;
            }
            b'u' => {
                // Restore Cursor
                let (col, row) = self
                    .performer
                    .get_saved_cursor_position()
                    .unwrap_or(self.performer.get_cursor_position());
                self.performer.move_cursor(row, col);
                self.state = AnsiiParserState::Ground;
            }
            b'D' => {
                let (col, row) = self.performer.get_cursor_position();
                if self.params.len() == 0 {
                    self.state = AnsiiParserState::Ground;
                    self.performer.move_cursor(row, col - 1);
                } else if self.params.len() == 2 {
                    let n = self.params[0] as u64;
                    if n == 0 {
                        self.performer.move_cursor(row, col - 1);
                    } else {
                        self.performer.move_cursor(row, col - n);
                    }
                } else {
                    self.state = AnsiiParserState::Ground;
                    serial_println!("Unimplemented CSI code: D with params: {:?}", self.params);
                }
            }
            _ => {
                // Handle other CSI codes
                self.state = AnsiiParserState::Ground;
                serial_println!("CSI code not implemented: {}", code);
            }
        }
    }
}
