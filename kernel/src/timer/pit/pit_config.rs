pub struct PITConfig(u8);
pub enum PITOperatingMode {
    InterruptOnTerminalCount = 0b000,
    HardwareRetriggerableOneShot = 0b001,
    RateGenerator = 0b010,
    SquareWaveGenerator = 0b011,
    SoftwareTriggeredStrobe = 0b100,
    HardwareTriggeredStrobe = 0b101,
}
pub enum PITAccessMode {
    AccessLowByte = 0b01,
    AccessHighByte = 0b10,
    AccessLowByteThenHighByte = 0b11,
}
pub enum PITChannel {
    Channel0 = 0b00,
    Channel1 = 0b01,
    Channel2 = 0b10,
}
pub enum PITEncoding {
    Binary = 0b0,
    BCD = 0b1,
}
impl PITConfig {
    pub fn new() -> Self {
        PITConfig(0b0)
    }
    pub fn build_from(
        encoding: PITEncoding,
        mode: PITOperatingMode,
        access_mode: PITAccessMode,
        channel: PITChannel,
    ) -> Self {
        PITConfig(
            (encoding as u8)
                | ((mode as u8) << 1)
                | ((access_mode as u8) << 4)
                | ((channel as u8) << 6),
        )
    }
    pub fn get_config(&self) -> u8 {
        self.0
    }
    pub fn set_encoding(&mut self, encoding: PITEncoding) {
        match encoding {
            PITEncoding::Binary => self.0 = self.0 & 0b1111_1110,
            PITEncoding::BCD => self.0 = self.0 | 0b1,
        }
    }

    pub fn set_mode(&mut self, mode: PITOperatingMode) {
        self.0 = self.0 | (mode as u8) << 1;
    }
    pub fn set_access_mode(&mut self, access_mode: PITAccessMode) {
        self.0 = self.0 | (access_mode as u8) << 4;
    }
    pub fn set_channel(&mut self, channel: PITChannel) {
        self.0 = self.0 | (channel as u8) << 6;
    }
}
