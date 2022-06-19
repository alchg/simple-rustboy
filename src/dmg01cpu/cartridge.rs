use super::Log;
use std::fs::File;
use std::io::Read;

/* MBC1 or None */
pub struct Cartridge {
    log_mode: u8,
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    bank_mode: u8, // rom or ram
    rom_bank: u8,
    shared_bank: u8, // ram bank or rom bank of upper 2bits
    enable_ram: u8,
}

impl Cartridge {
    pub fn new(log_mode: u8, romfile: String) -> Self {
        let mut data = Vec::new();

        let mut file: File = match File::open(romfile) {
            Ok(result) => result,
            Err(error) => panic!("file open error:{}", error),
        };

        match file.read_to_end(&mut data) {
            Ok(result) => result,
            Err(error) => panic!("file read error:{}", error),
        };
        Log::info(format!("{: <5}:{} byte", "Size", data.len()), log_mode);

        let cartridge_type: u8 = data[0x0147];
        match cartridge_type {
            0x00 => Log::info(format!("{: <5}:{}", "Type", "NONE"), log_mode),
            0x01..=0x03 => Log::info(format!("{: <5}:{}", "Type", "MBC1"), log_mode),
            _ => {
                Log::info(
                    format!("{: <5}:{:#04x} byte", "Type", cartridge_type),
                    log_mode,
                );
                panic!("unsupported type {:#04x}", cartridge_type);
            }
        }

        let ram_size: usize = match data[0x0149] {
            0 => 0,
            1 => 2 * 1024,
            2 => 8 * 1024,
            3 => 32 * 1024, // 4 banks
            _ => panic!("unsupported ram size"),
        };
        Log::info(format!("{: <5}:{} byte", "RAM", ram_size), log_mode);

        Cartridge {
            log_mode: log_mode,
            rom: data,
            ram: vec![0; ram_size],
            rom_bank: 0x00,
            shared_bank: 0x00,
            bank_mode: 0x00,
            enable_ram: 0x00,
        }
    }

    fn get_rom_bank(&self) -> u8 {
        let bank_number: u8 = match self.bank_mode {
            0x01 => self.rom_bank,                      // ram banking mode
            _ => self.shared_bank << 5 | self.rom_bank, // 0x00:rom banking mode
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
            0x01 => self.shared_bank, // ram banking mode
            _ => 0x00,                // 0x00:rom banking mode
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        Log::rom(
            format!("{: <15}:{:#04x}", "write address", address),
            self.log_mode,
        );
        Log::rom(format!("{: <15}:{:#04x}", "value", value), self.log_mode);

        match address {
            0x0000..=0x1fff => self.enable_ram = value,
            0x2000..=0x3fff => self.rom_bank = value & 0x1f,
            0x4000..=0x5fff => self.shared_bank = value & 0x03,
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
                    self.ram[ram_address] = value;
                } else {
                    panic!("unexpected address {:#08x}.need return?", address)
                }
            }
            _ => {
                panic!("write address error:{:#08x}", address)
            }
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        Log::rom(
            format!("{: <15}:{:#04x}", "read address", address),
            self.log_mode,
        );

        let result: u8 = match address {
            0x0000..=0x3fff => self.rom[address as usize],
            0x4000..=0x7fff => {
                let offset: usize = (16 * 1024) * self.get_rom_bank() as usize;
                self.rom[(address & 0x3fff) as usize + offset]
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
                    self.ram[ram_address]
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
        };

        Log::rom(format!("{: <15}:{:#04x}", "result", result), self.log_mode);
        result
    }
}
