use crate::drivers::fonts::color::{colors, Color};
use alloc::format;
use alloc::string::String;

pub trait AnsiiString {
    fn fg(self, color: Color) -> String;
    fn bg(self, color: Color) -> String;
    fn reset(self) -> String;
    fn green_fg(self) -> String;
    fn red_fg(self) -> String;
    fn bold(self) -> String;
}

macro_rules! impl_ansii_string {
    ($T:ty) => {
        impl AnsiiString for $T {
            fn fg(self, color: Color) -> String {
                format!(
                    "\x1b[38;2;{};{};{}m{}\x1b[0m",
                    color.r, color.g, color.b, self
                )
            }

            fn bg(self, color: Color) -> String {
                format!(
                    "\x1b[48;2;{};{};{}m{}\x1b[0m",
                    color.r, color.g, color.b, self
                )
            }

            fn reset(self) -> String {
                format!("\x1b[0m{}", self)
            }

            fn green_fg(self) -> String {
                // `self` here refers to the instance of $T, which implements AnsiiString
                self.fg(colors::GREEN)
            }

            fn red_fg(self) -> String {
                self.fg(colors::RED)
            }

            fn bold(self) -> String {
                format!("\x1b[1m{}\x1b[0m", self)
            }
        }
    };
}

impl_ansii_string!(&'static str);
impl_ansii_string!(String);
