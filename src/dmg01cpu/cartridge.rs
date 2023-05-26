mod mbc1;
mod mbc2;
mod mbc5;

use super::Log;
use mbc1::MBC1;
use mbc2::MBC2;
use mbc5::MBC5;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub struct Cartridge {
    log_mode: u8,
    pub rom: Vec<u8>,
    pub ram: Vec<u8>,
    ramfile: String,
    cartridge_type: u8,
    mbc1: MBC1,
    mbc2: MBC2,
    mbc5: MBC5,
}

impl Cartridge {
    pub fn new(log_mode: u8, romfile: String) -> Self {
        let rom_data: Vec<u8>;
        let ram_data: Vec<u8>;
        let ramfile: String = romfile.clone() + ".sav";

        rom_data = Self::load_file(romfile);
        Log::info(format!("{: <5}:{} byte", "Size", rom_data.len()), log_mode);

        let cartridge_type: u8 = rom_data[0x0147];
        match cartridge_type {
            0x00 => Log::info(format!("{: <5}:{}", "Type", "NONE"), log_mode),
            0x01..=0x03 => Log::info(format!("{: <5}:{}", "Type", "MBC1"), log_mode),
            0x05..=0x06 => Log::info(format!("{: <5}:{}", "Type", "MBC2"), log_mode),
            0x19..=0x1e => Log::info(format!("{: <5}:{}", "Type", "MBC5"), log_mode),
            _ => {
                Log::info(format!("{: <5}:{:#04x}", "Type", cartridge_type), log_mode);
                panic!("unsupported type {:#04x}", cartridge_type);
            }
        }

        let ram_size: usize = match rom_data[0x0149] {
            0 => {
                match cartridge_type {
                    0x05..=0x06 => 512, // mbc2
                    _ => 0,
                }
            }
            1 => 2 * 1024, // unused
            2 => 8 * 1024,
            3 => 8 * 4 * 1024,  // 4 banks
            4 => 8 * 16 * 1024, // 16 banks
            5 => 8 * 8 * 1024,  // 8 banks
            _ => panic!("unsupported ram size"),
        };

        let path: &Path = Path::new(&ramfile);
        if path.exists() {
            Log::info(format!("{: <5}:{}", "RAM", ramfile), log_mode);
            ram_data = Self::load_file(ramfile.clone());
        } else {
            Log::info(format!("{: <5}:", "RAM"), log_mode);
            ram_data = vec![0; ram_size];
        }
        Log::info(format!("{: <5}:{} byte", "SIZE", ram_data.len()), log_mode);

        Cartridge {
            log_mode,
            rom: rom_data,
            ram: ram_data,
            ramfile,
            cartridge_type,
            mbc1: MBC1::new(log_mode),
            mbc2: MBC2::new(log_mode),
            mbc5: MBC5::new(log_mode),
        }
    }

    fn load_file(file: String) -> Vec<u8> {
        let mut data = Vec::new();

        let mut file: File = match File::open(file) {
            Ok(result) => result,
            Err(error) => panic!("file open error:{}", error),
        };

        match file.read_to_end(&mut data) {
            Ok(result) => result,
            Err(error) => panic!("file read error:{}", error),
        };
        data
    }

    pub fn save(self) {
        Log::info(format!("{: <5}:{}", "Save", self.ramfile), self.log_mode);

        let mut file: File = match File::create(self.ramfile) {
            Ok(result) => result,
            Err(error) => panic!("file create error:{}", error),
        };

        match file.write_all(&self.ram) {
            Ok(result) => result,
            Err(error) => panic!("file write error:{}", error),
        };
    }

    pub fn write(&mut self, address: u16, value: u8) {
        Log::rom(
            format!("{: <15}:{:#04x}", "write address", address),
            self.log_mode,
        );
        Log::rom(format!("{: <15}:{:#04x}", "value", value), self.log_mode);

        match self.cartridge_type {
            0x00..=0x03 => self.mbc1.write(address, value, &mut self.ram),
            0x05..=0x06 => self.mbc2.write(address, value, &mut self.ram),
            0x19..=0x1e => self.mbc5.write(address, value, &mut self.ram),
            _ => panic!("unsupported type {:#04x}", self.cartridge_type),
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        Log::rom(
            format!("{: <15}:{:#04x}", "read address", address),
            self.log_mode,
        );

        let result: u8 = match self.cartridge_type {
            0x00..=0x03 => self.mbc1.read(address, &self.rom, &self.ram),
            0x05..=0x06 => self.mbc2.read(address, &self.rom, &self.ram),
            0x19..=0x1e => self.mbc5.read(address, &self.rom, &self.ram),
            _ => panic!("unsupported type {:#04x}", self.cartridge_type),
        };

        Log::rom(format!("{: <15}:{:#04x}", "result", result), self.log_mode);
        result
    }
}
