mod channel;
mod wave;

use super::super::Common;
use super::Log;
use channel::Channel;
use wave::Wave;

pub struct APU {
    log_mode: u8,
    channel1: Channel,
    channel2: Channel,
    channel3: Wave,
    //channel4: Channel,
    lvol: f64,
    rvol: f64,
    ram: [u8; 0x40],
}

impl APU {
    pub fn new(log_mode: u8) -> Self {
        let apu = APU {
            log_mode: log_mode,
            channel1: Channel::new(),
            channel2: Channel::new(),
            channel3: Wave::new(),
            //channel4: Channel::new(),
            lvol: 0.0,
            rvol: 0.0,
            ram: [0; 0x40],
        };

        apu
    }

    pub fn execute(&mut self, modify: u32) -> Vec<i16> {
        let mut result = Vec::new();
        let max: u32 = Common::SAMPLE_RATE / 60;
        let vol: f64 = (self.lvol + self.rvol) / 2.0;

        for _ in 0..max + modify {
            let value =
                (self.channel1.sample() + self.channel2.sample() + self.channel3.sample()) / 3;

            result.push(((value as f64) * vol) as i16);
        }

        result
    }

    fn start_channel1(&mut self) {
        let selection = (self.ram[0x14] & 0x40) >> 6; // 1:stop when nr11 expire
        let length = self.ram[0x11] & 0x3f;

        let envelope_vol = (self.ram[0x12] & 0xf0) >> 4;
        let envelope_dir = (self.ram[0x12] & 0x08) >> 3;
        let envelope_sweep = self.ram[0x12] & 0x07;

        let sweep_time = (self.ram[0x10] & 0x70) >> 4;
        let sweep_dir = self.ram[0x10] >> 3; // 1:decrease
        let sweep_num = self.ram[0x10] & 0x07;

        let mut duration = -1;
        if selection == 1 {
            duration = ((length as f32 * (1.0 / 64.0)) as i32) * Common::SAMPLE_RATE as i32;
        }

        self.channel1.reset(duration);
        self.channel1.envelope_steps = envelope_vol as i32;
        self.channel1.envelope_steps_init = envelope_vol as i32;
        self.channel1.envelope_samples =
            (envelope_sweep as i32) * (Common::SAMPLE_RATE as i32) / 64;
        self.channel1.envelope_increasing = envelope_dir == 1;

        self.channel1.sweep_step_len = sweep_time;
        self.channel1.sweep_step = sweep_num;
        self.channel1.sweep_increase = sweep_dir == 0;
    }

    fn start_channel2(&mut self) {
        let selection = (self.ram[0x19] & 0x40) >> 6; // 1:stop when nr21 expire
        let length = self.ram[0x16] & 0x3f;

        let envelope_vol = (self.ram[0x17] & 0xf0) >> 4;
        let envelope_dir = (self.ram[0x17] & 0x08) >> 3;
        let envelope_sweep = self.ram[0x17] & 0x07;

        let mut duration = -1;
        if selection == 1 {
            duration = ((length as f32 * (1.0 / 64.0)) as i32) * Common::SAMPLE_RATE as i32;
        }

        self.channel2.reset(duration);
        self.channel2.envelope_steps = envelope_vol as i32;
        self.channel2.envelope_steps_init = envelope_vol as i32;
        self.channel2.envelope_samples =
            (envelope_sweep as i32) * (Common::SAMPLE_RATE as i32) / 64;
        self.channel2.envelope_increasing = envelope_dir == 1;
    }

    fn start_channel3(&mut self) {
        let selection = (self.ram[0x1e] & 0x40) >> 6; // 1:stop when nr31 expire
        let length = self.ram[0x1b];

        let mut duration = -1;
        if selection == 1 {
            duration =
                (((256.0 - length as f32) * (1.0 / 256.0)) as i32) * Common::SAMPLE_RATE as i32;
        }

        self.channel3.reset(duration);
    }

    fn masked_read(&self, address: u16, value: u8) -> u8 {
        match address {
            0xff10 => value & 0xff,                   // sound channel 1 - sweep
            0xff11 | 0xff16 => value & 0xc0,          // sound channel 1,2 length/duty
            0xff12 | 0xff17 | 0xff21 => value & 0xff, // sound channel 1,2,4 - envelope
            0xff14 | 0xff19 | 0xff1e => value & 0x40, // sound channel 1,2,3 - frequency high
            0xff22 | 0xff24 | 0xff25 => value & 0xff, // sound channel 4 - polynomial counter,channel control,output selection
            0xff1a => value & 0x80,                   // sound channel 3 - on/off
            0xff26 => value & 0x80,                   // sound controller - on/off
            0xff20 => value & 0x3f,                   // sound channel 4 - length
            0xff23 => value & 0x40, // sound channel 4 - consecutive/initial counter
            0xff13 | 0xff15 | 0xff18 | 0xff1b | 0xff1d | 0xff1f => {
                panic!("unexcepted address {}", address)
            }

            _ => 0,
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        Log::apu(
            format!("{: <15}:{:#04x}", "read address", address),
            self.log_mode,
        );
        let ram_address = (address - 0xff00) as usize;

        let result = match address {
            0xff10..=0xff26 => self.masked_read(address, self.ram[ram_address]), // sound 1,2,3,4
            0xff30..=0xff3f => self.ram[ram_address],                            // waveram
            _ => panic!("unexcepted address {}", address),
        };

        Log::apu(format!("{: <15}:{:#04x}", "result", result), self.log_mode);
        result
    }

    fn get_squarelimit(&self, wave_pattern: u8) -> f64 {
        match wave_pattern {
            // duty
            0 => -0.25, // 12.5%
            1 => -0.5,  // 25%
            2 => 0.0,   // 50% normal
            3 => 0.5,   // 75%
            _ => panic!("unexcepted wave_pattern={}", wave_pattern),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        Log::apu(
            format!("{: <15}:{:#04x}", "write address", address),
            self.log_mode,
        );
        Log::apu(format!("{: <15}:{:#04x}", "value", value), self.log_mode);
        let ram_address = (address - 0xff00) as usize;
        self.ram[ram_address] = value;

        match address {
            0xff14 => {
                // NR14
                if value & 0x80 == 0x80 {
                    self.start_channel1(); // restart
                }
                let frequency_value: u16 = ((self.ram[ram_address] & 0x07) as u16) << 8
                    | (self.ram[ram_address - 1] as u16);
                self.channel1.frequency = (131072 / (2048 - frequency_value as u32)) as f64;
            }
            0xff11 => {
                // NR11
                let wave_pattern: u8 = (self.ram[ram_address] & 0xc0) >> 6;
                self.channel1.limit = self.get_squarelimit(wave_pattern);
            }

            0xff19 => {
                // NR24
                if value & 0x80 == 0x80 {
                    self.start_channel2(); // restart
                }
                let frequency_value: u16 = ((self.ram[ram_address] & 0x07) as u16) << 8
                    | (self.ram[ram_address - 1] as u16);
                self.channel2.frequency = (131072 / (2048 - frequency_value as u32)) as f64;
            }
            0xff16 => {
                // NR21
                let wave_pattern: u8 = (self.ram[ram_address] & 0xc0) >> 6;
                self.channel2.limit = self.get_squarelimit(wave_pattern);
            }

            0xff1a => {
                // NR30
                if self.ram[ram_address] & 0x80 == 0x80 {
                    self.channel3.is_playback = true;
                } else {
                    self.channel3.is_playback = false;
                }
            }
            0xff1e => {
                // NR34
                if value & 0x80 == 0x80 {
                    self.start_channel3(); // restart
                }
                let frequency_value: u16 =
                    ((self.ram[0x1e] & 0x07) as u16) << 8 | self.ram[0x1d] as u16;
                self.channel3.frequency = (65536 / (2048 - frequency_value as u32)) as f64;
            }
            0xff1c => {
                // NR32
                self.channel3.amplitude = match value & 0x60 >> 5 {
                    0x00 => 0.0,
                    0x01 => 1.0,
                    0x02 => 0.5,
                    0x03 => 0.25,
                    _ => panic!("unexpected value {:#08x}", value),
                }
            }
            0xff30..=0xff3f => {
                // Wave Pattern RAM
                if self.ram[0x1a] & 0x80 == 0x80 {
                    panic!("unexpected address {:#08x}", address);
                }
                let index: usize = ((address - 0xff30) * 2) as usize;
                self.channel3.wave_form[index] = ((value >> 4) as i16 - 8) * 125;
                self.channel3.wave_form[index + 1] = ((value & 0x0f) as i16 - 8) * 125;
            }

            0xff24 => {
                // NR50
                self.lvol = (((self.ram[ram_address] & 0x70) >> 4) as f64) / 7.0;
                self.rvol = ((self.ram[ram_address] & 0x07) as f64) / 7.0;
            }

            0xff25 => {
                // NR51
                let o1r = self.ram[ram_address] & 0x01 == 0x01;
                let o1l = self.ram[ram_address] & 0x10 == 0x10;

                self.channel1.is_on = o1r | o1l;
            }

            _ => (),
        }
    }
}
