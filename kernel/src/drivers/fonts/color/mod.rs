/// Color struct containing the red, green, blue and alpha values
///
#[derive(Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub mod colors {

    use super::Color;

    pub const BLACK: Color = Color::new(0, 0, 0, 255);
    pub const RED: Color = Color::new(170, 0, 0, 255);
    pub const GREEN: Color = Color::new(0, 170, 0, 255);
    pub const BROWN: Color = Color::new(170, 85, 0, 255);
    pub const BLUE: Color = Color::new(0, 0, 170, 255);
    pub const MAGENTA: Color = Color::new(170, 0, 170, 255);
    pub const CYAN: Color = Color::new(0, 170, 170, 255);
    pub const LIGHT_GRAY: Color = Color::new(170, 170, 170, 255);
    pub const DARK_GRAY: Color = Color::new(85, 85, 85, 255);
    pub const LIGHT_RED: Color = Color::new(255, 85, 85, 255);
    pub const LIGHT_GREEN: Color = Color::new(85, 255, 85, 255);
    pub const YELLOW: Color = Color::new(255, 255, 85, 255);
    pub const LIGHT_BLUE: Color = Color::new(85, 85, 255, 255);
    pub const LIGHT_MAGENTA: Color = Color::new(255, 85, 255, 255);
    pub const LIGHT_CYAN: Color = Color::new(85, 255, 255, 255);
    pub const WHITE: Color = Color::new(255, 255, 255, 255);

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ColorType {
        Background,
        Foreground,
        None,
    }

    pub fn get_color(value: u16, bright: bool) -> Color {
        match value {
            30 | 40 => {
                if bright {
                    LIGHT_GRAY
                } else {
                    BLACK
                }
            }
            31 | 41 => {
                if bright {
                    LIGHT_RED
                } else {
                    RED
                }
            }
            32 | 42 => {
                if bright {
                    LIGHT_GREEN
                } else {
                    GREEN
                }
            }
            33 | 43 => {
                if bright {
                    YELLOW
                } else {
                    BROWN
                }
            }
            34 | 44 => {
                if bright {
                    LIGHT_BLUE
                } else {
                    BLUE
                }
            }
            35 | 45 => {
                if bright {
                    LIGHT_MAGENTA
                } else {
                    MAGENTA
                }
            }
            36 | 46 => {
                if bright {
                    LIGHT_CYAN
                } else {
                    CYAN
                }
            }
            37 | 47 => {
                if bright {
                    WHITE
                } else {
                    LIGHT_GRAY
                }
            }
            _ => BLACK, // Default to black for out of range values
        }
    }
}

impl Color {
    /// Creates a new color with the given red, green, blue and alpha values
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub const fn default() -> Self {
        Color::new(0, 0, 0, 255) // Default to black with full opacity
    }

    pub fn set_r(&mut self, r: u8) {
        self.r = r;
    }
    pub fn set_g(&mut self, g: u8) {
        self.g = g;
    }
    pub fn set_b(&mut self, b: u8) {
        self.b = b;
    }
    /// Converts the color to a u32 value
    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn from_u32(value: u32) -> Self {
        Color {
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: (value & 0xFF) as u8,
            a: ((value >> 24) & 0xFF) as u8,
        }
    }
}
