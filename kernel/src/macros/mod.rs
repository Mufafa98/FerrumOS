#[macro_export]
macro_rules! ok {
    ($($arg:tt)*) => {{
        use $crate::drivers::fonts::ansii_parser::ansii_builder::AnsiiString;
        use $crate::drivers::fonts::color::colors::GREEN;
        use $crate::println;
        println!("[  {}  ]: {}", "ok".fg(GREEN), format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {{
        use $crate::drivers::fonts::ansii_parser::ansii_builder::AnsiiString;
        use $crate::drivers::fonts::color::colors::YELLOW;
        use $crate::println;
        println!("[ {} ]: {}", "warn".fg(YELLOW), format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! failed {
    ($($arg:tt)*) => {{
        use $crate::drivers::fonts::ansii_parser::ansii_builder::AnsiiString;
        use $crate::drivers::fonts::color::colors::RED;
        use $crate::println;
        println!("[{}]: {}", "failed".fg(RED), format_args!($($arg)*));
    }};
}
