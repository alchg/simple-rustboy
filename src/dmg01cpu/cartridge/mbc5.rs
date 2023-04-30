use super::Log;

pub struct MBC5 {
    log_mode: u8,
    rom_bank_low: u8,
    rom_bank_high: u8,
    ram_bank: u8,
    enable_ram: u8,
}

impl MBC5 {
    pub fn new(log_mode: u8) -> Self {
        MBC5 {
            log_mode,
            rom_bank_low: 0x00,
            rom_bank_high: 0x00,
            ram_bank: 0x00,
            enable_ram: 0x00,
        }
    }

    fn get_rom_bank(&self) -> u16 {
        (self.rom_bank_high as u16) << 8 | self.rom_bank_low as u16
    }

    fn is_ram_enabled(&self) -> bool {
        if self.enable_ram & 0x0f == 0x0a {
            true
        } else {
            false
        }
    }

    pub fn write(&mut self, address: u16, value: u8, ram: &mut Vec<u8>) {
        match address {
            0x0000..=0x1fff => self.enable_ram = value,
            0x2000..=0x2fff => self.rom_bank_low = value,
            0x3000..=0x3fff => self.rom_bank_high = value & 0x01,
            0x4000..=0x5fff => self.ram_bank = value & 0x0f,
            0xa000..=0xbfff => {
                if self.is_ram_enabled() {
                    let offset: usize = (8 * 1024) * self.ram_bank as usize;
                    Log::rom(format!("{: <15}:{:#04x}", "offset", offset), self.log_mode);
                    let ram_address = (address & 0x1fff) as usize + offset;
                    Log::rom(
                        format!("{: <15}:{:#04x}", "ram address", ram_address),
                        self.log_mode,
                    );
                    ram[ram_address] = value;
                } else {
                    panic!("unexpected address {:#08x}.need return?", address)
                }
            }
            _ => {
                panic!("write address error:{:#08x}", address)
            }
        }
    }

    pub fn read(&self, address: u16, rom: &Vec<u8>, ram: &Vec<u8>) -> u8 {
        match address {
            0x0000..=0x3fff => rom[address as usize],
            0x4000..=0x7fff => {
                let offset: usize = (16 * 1024) * self.get_rom_bank() as usize;
                rom[(address & 0x3fff) as usize + offset]
            }

            0xa000..=0xbfff => {
                if self.is_ram_enabled() {
                    let offset: usize = (8 * 1024) * self.ram_bank as usize;
                    Log::rom(format!("{: <15}:{:#04x}", "offset", offset), self.log_mode);
                    let ram_address = (address & 0x1fff) as usize + offset;
                    Log::rom(
                        format!("{: <15}:{:#04x}", "ram address", ram_address),
                        self.log_mode,
                    );
                    ram[ram_address]
                } else {
                    panic!(
                        "unexpected address {:#08x}.ram is disabled.need 0xff?",
                        address
                    )
                }
            }

            _ => {
                panic!("unexpected address:{:#08x}", address)
            }
        }
    }
}
