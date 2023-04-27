mod noise;
mod tone;
mod wave;

use super::super::Common;
use super::Log;
use noise::Noise;
use tone::Tone;
use wave::Wave;

pub struct APU {
    log_mode: u8,
    counter: u32, // FPS
    channel1: Tone,
    channel2: Tone,
    channel3: Wave,
    channel4: Noise,
    lvol: f64,
    rvol: f64,
    ram: [u8; 0x40],
}

impl APU {
    pub fn new(log_mode: u8) -> Self {
        let apu = APU {
            log_mode: log_mode,
            counter: 1,
            channel1: Tone::new(),
            channel2: Tone::new(),
            channel3: Wave::new(),
            channel4: Noise::new(),
            lvol: 0.0,
            rvol: 0.0,
            ram: [0; 0x40],
        };

        apu
    }

    pub fn execute(&mut self, modify: u32) -> Vec<i16> {
        let mut result = Vec::new();
        let max: u32 = Common::SAMPLE_RATE / Common::FPS as u32;
        let vol: f64 = (self.lvol + self.rvol) / 2.0;

        for _ in 0..max + modify {
            let value = self.channel1.sample()
                + self.channel2.sample()
                + self.channel3.sample()
                + self.channel4.sample();
            result.push(((value as f64) * vol) as i16);
        }

        if self.counter >= 3600 * Common::FPS as u32 - 1 {
            self.channel1.time = 0.0;
            self.channel2.time = 0.0;
            self.channel3.time = 0.0;
            self.channel4.time = 0.0;
            self.channel4.last_time = 0.0;
            self.counter = 0;
            Log::info(format!("RESET APU TIME"), self.log_mode);
        }

        self.counter += 1;
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

    fn start_channel4(&mut self) {
        let selection = (self.ram[0x23] & 0x40) >> 6; // 1:stop when nr41 expire
        let length = self.ram[0x20] & 0x3f;

        let envelope_vol = (self.ram[0x21] & 0xf0) >> 4;
        let envelope_dir = (self.ram[0x21] & 0x08) >> 3;
        let envelope_sweep = self.ram[0x21] & 0x07;

        let mut duration = -1;
        if selection == 1 {
            duration =
                ((((61 - length) as f32) * (1.0 / 256.0)) as i32) * Common::SAMPLE_RATE as i32;
        }

        self.channel4.reset(duration);
        self.channel4.envelope_steps = envelope_vol as i32;
        self.channel4.envelope_steps_init = envelope_vol as i32;
        self.channel4.envelope_samples =
            (envelope_sweep as i32) * (Common::SAMPLE_RATE as i32) / 64;
        self.channel4.envelope_increasing = envelope_dir == 1;
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
            // 0xff13 NR13
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
            // 0xff10 NR10
            // 0xff12 NR12

            // 0xff18 NR23
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
            // 0xff17 NR22

            //
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
            // 0xff1d NR33
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
            // 0xff1b NR31
            0xff30..=0xff3f => {
                // Wave Pattern RAM
                if self.ram[0x1a] & 0x80 == 0x80 {
                    panic!("unexpected address {:#08x}", address);
                }
                let index: usize = ((address - 0xff30) * 2) as usize;
                self.channel3.wave_form[index] = ((value >> 4) as i16 - 8) * 125;
                self.channel3.wave_form[index + 1] = ((value & 0x0f) as i16 - 8) * 125;
            }

            0xff22 => {
                // NR43
                let clock_shift: u16 = ((value & 0xf0) >> 4) as u16;
                let mut div_ratio: f64 = (value & 0x07) as f64;
                if div_ratio == 0.0 {
                    div_ratio = 0.5;
                }
                self.channel4.frequency = 524288.0 / div_ratio / (clock_shift + 1).pow(2) as f64;
            }
            // 0xff20 NR41
            0xff23 => {
                // NR44
                if value & 0x80 == 0x80 {
                    self.start_channel4(); // restart
                }
            }
            // 0xff21 NR42

            //
            0xff24 => {
                // NR50
                self.lvol = (((self.ram[ram_address] & 0x70) >> 4) as f64) / 7.0;
                self.rvol = ((self.ram[ram_address] & 0x07) as f64) / 7.0;
            }

            0xff25 => {
                // NR51
                let out1r = self.ram[ram_address] & 0x01 == 0x01;
                let out1l = self.ram[ram_address] & 0x10 == 0x10;
                let out2r = self.ram[ram_address] & 0x02 == 0x02;
                let out2l = self.ram[ram_address] & 0x20 == 0x20;
                let out3r = self.ram[ram_address] & 0x04 == 0x04;
                let out3l = self.ram[ram_address] & 0x40 == 0x40;
                let out4r = self.ram[ram_address] & 0x08 == 0x08;
                let out4l = self.ram[ram_address] & 0x80 == 0x80;

                self.channel1.is_on = out1r | out1l;
                self.channel2.is_on = out2r | out2l;
                self.channel3.is_on = out3r | out3l;
                self.channel4.is_on = out4r | out4l;
            }

            _ => (),
        }
    }
}
