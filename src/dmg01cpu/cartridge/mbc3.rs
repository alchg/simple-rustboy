use super::Log;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::time::SystemTime;

pub struct MBC3 {
    log_mode: u8,
    enable_ram_rtc: u8,
    ram_rtc_status: u8,
    rom_bank: u8,
    is_latch: bool,
    pre_latch: u8,
    latch_sec: u8,
    latch_min: u8,
    latch_hour: u8,
    latch_day_counter_low: u8,
    latch_day_counter_high: u8,
    /* RTC */
    sec: u8,
    min: u8,
    hour: u8,
    day_counter_low: u8,
    day_counter_high: u8,
}

impl MBC3 {
    pub fn new(log_mode: u8) -> Self {
        MBC3 {
            log_mode,
            enable_ram_rtc: 0x00,
            ram_rtc_status: 0x00,
            rom_bank: 0x00,
            is_latch: false,
            pre_latch: 0x01,
            latch_sec: 0x00,
            latch_min: 0x00,
            latch_hour: 0x00,
            latch_day_counter_low: 0x00,
            latch_day_counter_high: 0x00,
            /* RTC */
            sec: 0x00,
            min: 0x00,
            hour: 0x00,
            day_counter_low: 0x00,
            day_counter_high: 0x00,
        }
    }

    pub fn exec_rtc(&mut self) {
        // 0x40 halt
        if self.day_counter_high & 0x40 == 0x00 {
            // active
            self.sec += 1;
            let mut day_counter: u16 =
                (self.day_counter_high as u16) << 8 | self.day_counter_low as u16;

            if self.sec > 0x3b {
                self.sec = 0;
                self.min += 1;
            }
            if self.min > 0x3b {
                self.min = 0;
                self.hour += 1;
            }
            if self.hour > 0x17 {
                self.hour = 0;
                day_counter += 1;
            }
            if day_counter & 0x03ff > 0x01ff {
                self.day_counter_high = self.day_counter_high | 0x80; // counter overflow
                self.day_counter_high = self.day_counter_high & 0xc0;
                self.day_counter_low = 0x00;
            } else {
                self.day_counter_high = (day_counter >> 8) as u8 | self.day_counter_high;
                self.day_counter_low = (day_counter & 0x00ff) as u8
            }
        }
    }

    pub fn save_rtc(self, rtcfile: String) {
        let mut file: File = match File::create(rtcfile) {
            Ok(result) => result,
            Err(error) => panic!("file create error:{}", error),
        };

        let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(result) => result,
            Err(error) => panic!("current time error:{}", error),
        };

        let lines = vec![
            self.sec.to_string(),
            self.min.to_string(),
            self.hour.to_string(),
            self.day_counter_low.to_string(),
            self.day_counter_high.to_string(),
            current_time.as_secs().to_string(),
        ];

        for line in lines {
            match writeln!(file, "{}", line) {
                Ok(result) => result,
                Err(error) => panic!("file write error:{}", error),
            };
        }
    }

    pub fn load_rtc(&mut self, rtcfile: String) {
        let file: File = match File::open(rtcfile) {
            Ok(result) => result,
            Err(error) => panic!("file open error:{}", error),
        };

        let mut reader = BufReader::new(file);
        let mut buf = String::new();

        match reader.read_line(&mut buf) {
            Ok(_) => {
                self.sec = buf.trim().parse().unwrap();
                buf.clear();
            }
            Err(error) => panic!("file read error:{}", error),
        }
        match reader.read_line(&mut buf) {
            Ok(_) => {
                self.min = buf.trim().parse().unwrap();
                buf.clear();
            }
            Err(error) => panic!("file read error:{}", error),
        }
        match reader.read_line(&mut buf) {
            Ok(_) => {
                self.hour = buf.trim().parse().unwrap();
                buf.clear();
            }
            Err(error) => panic!("file read error:{}", error),
        }
        match reader.read_line(&mut buf) {
            Ok(_) => {
                self.day_counter_low = buf.trim().parse().unwrap();
                buf.clear();
            }
            Err(error) => panic!("file read error:{}", error),
        }
        match reader.read_line(&mut buf) {
            Ok(_) => {
                self.day_counter_high = buf.trim().parse().unwrap();
                buf.clear();
            }
            Err(error) => panic!("file read error:{}", error),
        }

        let last_time_sec: u64;
        match reader.read_line(&mut buf) {
            Ok(_) => {
                last_time_sec = buf.trim().parse().unwrap();
                buf.clear();
            }
            Err(error) => panic!("file read error:{}", error),
        }

        let current_time = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(result) => result,
            Err(error) => panic!("current time error:{}", error),
        };
        let current_time_sec: u64 = current_time.as_secs();
        if current_time_sec > last_time_sec {
            let counter = current_time_sec - last_time_sec;
            for _ in 0..counter {
                self.exec_rtc();
            }
        } else {
            panic!("last time error");
        }
    }

    fn get_rom_bank(&self) -> u8 {
        let bank = self.rom_bank;

        if self.rom_bank == 0x00 {
            //specification
            0x01
        } else {
            bank
        }
    }

    fn is_ram_rtc_enabled(&self) -> bool {
        match self.enable_ram_rtc {
            0x00 => false,
            0x0a => true,
            _ => {
                panic!(
                    "unexpected value to enable ram and rtc{:#04x}",
                    self.enable_ram_rtc
                )
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8, ram: &mut Vec<u8>) {
        match address {
            0x0000..=0x1fff => self.enable_ram_rtc = value,
            0x2000..=0x3fff => self.rom_bank = value & 0x7f,
            0x4000..=0x5fff => self.ram_rtc_status = value,
            0x6000..=0x7fff => {
                if self.pre_latch == 0x00 && value == 0x01 {
                    self.is_latch = !self.is_latch;
                    if self.is_latch {
                        self.latch_sec = self.sec;
                        self.latch_min = self.min;
                        self.latch_hour = self.hour;
                        self.latch_day_counter_low = self.day_counter_low;
                        self.latch_day_counter_high = self.day_counter_high;
                    }
                }
                self.pre_latch = value;
            }
            0xa000..=0xbfff => {
                if self.is_ram_rtc_enabled() {
                    match self.ram_rtc_status {
                        0x00..=0x03 => {
                            let offset: usize = (8 * 1024) * self.ram_rtc_status as usize;
                            Log::rom(format!("{: <15}:{:#04x}", "offset", offset), self.log_mode);
                            let ram_address = (address & 0x1fff) as usize + offset;
                            Log::rom(
                                format!("{: <15}:{:#04x}", "ram address", ram_address),
                                self.log_mode,
                            );
                            ram[ram_address] = value;
                        }
                        0x08..=0x0c => {
                            if self.is_latch == false {
                                match self.ram_rtc_status {
                                    0x08 => self.sec = value & 0x3b,
                                    0x09 => self.min = value & 0x3b,
                                    0x0a => self.hour = value & 0x17,
                                    0x0b => self.day_counter_low = value,
                                    _ => self.day_counter_high = value, // 0x0c
                                }
                            }
                        }
                        _ => {
                            panic!("unexpected ram rtc status {:#04x}.", self.ram_rtc_status)
                        }
                    }
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
                if self.is_ram_rtc_enabled() {
                    match self.ram_rtc_status {
                        0x00..=0x03 => {
                            let offset: usize = (8 * 1024) * self.ram_rtc_status as usize;
                            Log::rom(format!("{: <15}:{:#04x}", "offset", offset), self.log_mode);
                            let ram_address = (address & 0x1fff) as usize + offset;
                            Log::rom(
                                format!("{: <15}:{:#04x}", "ram address", ram_address),
                                self.log_mode,
                            );
                            ram[ram_address]
                        }
                        0x08..=0x0c => {
                            if self.is_latch {
                                match self.ram_rtc_status {
                                    0x08 => self.latch_sec,
                                    0x09 => self.latch_min,
                                    0x0a => self.latch_hour,
                                    0x0b => self.latch_day_counter_low,
                                    _ => self.latch_day_counter_high, //0x0c
                                }
                            } else {
                                match self.ram_rtc_status {
                                    0x08 => self.sec,
                                    0x09 => self.min,
                                    0x0a => self.hour,
                                    0x0b => self.day_counter_low,
                                    _ => self.day_counter_high, // 0x0c
                                }
                            }
                        }
                        _ => {
                            panic!("unexpected ram rtc status {:#04x}.", self.ram_rtc_status)
                        }
                    }
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
