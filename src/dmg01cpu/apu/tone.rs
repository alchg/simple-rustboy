use super::super::super::Common;

const SWEEP_TIME: [f64; 7] = [0.0078, 0.0156, 0.0234, 0.0313, 0.0391, 0.0469, 0.0547];

pub struct Tone {
    pub frequency: f64,
    pub limit: f64,
    time: f64,
    amplitude: f64,
    duration: i32, // duration in sample
    pub is_on: bool,

    // Envelope
    envelope_time: i32,
    pub envelope_steps: i32,
    pub envelope_steps_init: i32,
    pub envelope_samples: i32,
    pub envelope_increasing: bool,

    // Sweep
    sweep_time: f64,
    sweep_steps: u8,
    pub sweep_step_len: u8,
    pub sweep_step: u8,
    pub sweep_increase: bool,
}

impl Tone {
    pub fn new() -> Tone {
        Tone {
            frequency: 0.0,
            limit: 0.0,
            time: 0.0,
            amplitude: 0.0,
            duration: 0,
            is_on: false,
            envelope_time: 0,
            envelope_steps: 0,
            envelope_steps_init: 0,
            envelope_samples: 0,
            envelope_increasing: false,
            sweep_time: 0.0,
            sweep_steps: 0x00,
            sweep_step_len: 0x00,
            sweep_step: 0x00,
            sweep_increase: false,
        }
    }

    fn sqware_wave(&self, time: f64) -> i16 {
        if time.sin() <= self.limit {
            1000
        } else {
            -1000
        }
    }

    pub fn sample(&mut self) -> i16 {
        let mut output: i16 = 0;
        let step: f64 = self.frequency * (std::f64::consts::PI * 2.0) / Common::SAMPLE_RATE as f64;
        self.time += step;

        if self.is_playing() && self.is_on {
            output = ((self.sqware_wave(self.time) as f64) * self.amplitude) as i16;
            if self.duration > 0 {
                self.duration -= 1;
            }
        }
        self.update_envelope();
        self.update_sweep();
        output
    }

    pub fn reset(&mut self, duration: i32) {
        self.amplitude = 1.0;
        self.envelope_time = 0;
        self.sweep_time = 0.0;
        self.sweep_step = 0;
        self.duration = duration;
    }

    fn is_playing(&mut self) -> bool {
        let mut result = false;

        if self.duration == -1 || self.duration > 0 {
            if self.envelope_steps_init > 0 {
                result = true;
            }
        }

        result
    }

    fn update_envelope(&mut self) {
        if self.envelope_samples > 0 {
            self.envelope_time += 1;

            if self.envelope_steps > 0 && self.envelope_time >= self.envelope_samples {
                self.envelope_time -= self.envelope_samples;
                self.envelope_steps -= 1;
                if self.envelope_steps == 0 {
                    self.amplitude = 0.0;
                } else if self.envelope_increasing {
                    self.amplitude =
                        1.0 - (self.envelope_steps as f64) / (self.envelope_steps_init as f64);
                } else {
                    self.amplitude =
                        (self.envelope_steps as f64) / (self.envelope_steps_init as f64);
                }
            }
        }
    }

    fn update_sweep(&mut self) {
        if self.sweep_step < self.sweep_steps {
            let time = SWEEP_TIME[(self.sweep_step_len - 1) as usize];
            self.sweep_time += Common::SAMPLE_RATE as f64 / Common::FPS as f64;
            if self.sweep_time > time {
                self.sweep_time -= time;
                self.sweep_step += 1;

                if self.sweep_increase {
                    self.frequency += self.frequency / 2.0f64.powf(self.sweep_step as f64);
                } else {
                    self.frequency -= self.frequency / 2.0f64.powf(self.sweep_step as f64);
                }
            }
        }
    }
}
