pub mod hpet;
pub mod lapic;
pub mod pit;

pub enum Time {
    Femtoseconds(u64),
    Nanoseconds(u64),
    Microseconds(u64),
    Milliseconds(u64),
    Seconds(u64),
}
impl Time {
    fn to_nanoseconds(&self) -> u64 {
        match self {
            Time::Femtoseconds(n) => *n / 1_000_000,
            Time::Nanoseconds(n) => *n,
            Time::Microseconds(n) => n * 1000,
            Time::Milliseconds(n) => n * 1_000_000,
            Time::Seconds(n) => n * 1_000_000_000,
        }
    }
    fn to_femtoseconds(&self) -> u64 {
        match self {
            Time::Femtoseconds(n) => *n,
            Time::Nanoseconds(n) => n * 1_000_000,
            Time::Microseconds(n) => n * 1_000_000_000,
            Time::Milliseconds(n) => n * 1_000_000_000_000,
            Time::Seconds(n) => n * 1_000_000_000_000_000,
        }
    }
}
