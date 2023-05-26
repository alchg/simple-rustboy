use super::Log;

pub struct MBC2 {
    log_mode: u8,
    rom_bank: u8,
    enable_ram: bool,
}

impl MBC2 {
    pub fn new(log_mode: u8) -> Self {
        MBC2 {
            log_mode,
            rom_bank: 0x01,
            enable_ram: true,
        }
    }

    fn get_rom_bank(&self) -> u8 {
        let bank: u8 = self.rom_bank & 0x0f;
        match bank {
            0x01..=0x0F => bank,
            _ => {
                panic!("rom bank error:{:#04x}", bank)
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8, ram: &mut Vec<u8>) {
        match address {
            0x0000..=0x1fff => {
                if (address & 0x0100) == 0 {
                    self.enable_ram = !(self.enable_ram);
                }
            }
            0x2000..=0x3fff => {
                if (address & 0x0100) == 0x0100 {
                    self.rom_bank = value;
                }
            }
            0xa000..=0xa1ff => {
                if self.enable_ram {
                    let ram_address = (address & 0x01ff) as usize;
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

            0xa000..=0xa1ff => {
                if self.enable_ram {
                    let ram_address = (address & 0x01ff) as usize;
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
