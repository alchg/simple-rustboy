use super::Log;

/* MBC1 or None */
pub struct MBC1 {
    log_mode: u8,
    bank_mode: u8, // rom or ram
    rom_bank: u8,
    shared_bank: u8, // ram bank or rom bank of upper 2bits
    enable_ram: u8,
}

impl MBC1 {
    pub fn new(log_mode: u8) -> Self {
        MBC1 {
            log_mode,
            rom_bank: 0x00,
            shared_bank: 0x00,
            bank_mode: 0x00,
            enable_ram: 0x00,
        }
    }

    fn get_rom_bank(&self) -> u8 {
        let bank_number: u8 = match self.bank_mode {
            0x01 => self.rom_bank,                        // ram banking mode
            _ => self.shared_bank & 0x60 | self.rom_bank, // 0x00:rom banking mode
        };

        match bank_number {
            0x00 | 0x20 | 0x40 | 0x60 => bank_number + 1, // specification
            _ => bank_number,
        }
    }

    fn is_ram_enabled(&self) -> bool {
        if self.enable_ram & 0x0f == 0x0a {
            true
        } else {
            false
        }
    }

    fn get_ram_bank(&self) -> u8 {
        match self.bank_mode {
            0x01 => self.shared_bank & 0x03, // ram banking mode
            _ => 0x00,                       // 0x00:rom banking mode
        }
    }

    pub fn write(&mut self, address: u16, value: u8, ram: &mut Vec<u8>) {
        match address {
            0x0000..=0x1fff => self.enable_ram = value,
            0x2000..=0x3fff => self.rom_bank = value & 0x1f,
            0x4000..=0x5fff => self.shared_bank = value,
            0x6000..=0x7fff => self.bank_mode = value & 0x01,
            0xa000..=0xbfff => {
                if self.is_ram_enabled() {
                    let offset: usize = (8 * 1024) * self.get_ram_bank() as usize;
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
                    let offset: usize = (8 * 1024) * self.get_ram_bank() as usize;
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
