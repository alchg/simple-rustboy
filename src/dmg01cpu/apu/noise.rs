use std::time::Instant;

use super::super::super::Common;

pub struct Noise {
    pub frequency: f64,
    pub time: f64,
    pub last_time: f64,
    noise_value: i16,
    amplitude: f64,
    duration: i32, // duration in sample
    pub is_on: bool,

    // Envelope
    envelope_time: i32,
    pub envelope_steps: i32,
    pub envelope_steps_init: i32,
    pub envelope_samples: i32,
    pub envelope_increasing: bool,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            frequency: 0.0,
            time: 0.0,
            last_time: 0.0,
            noise_value: 0,
            amplitude: 0.0,
            duration: 0,
            is_on: false,
            envelope_time: 0,
            envelope_steps: 0,
            envelope_steps_init: 0,
            envelope_samples: 0,
            envelope_increasing: false,
        }
    }

    fn noise(&mut self, time: f64) -> i16 {
        if time - self.last_time > std::f64::consts::PI * 2.0 {
            self.last_time = time;
            let rand = Instant::now().elapsed().as_nanos() % 2;
            if rand == 1 {
                self.noise_value = 1000
            } else {
                self.noise_value = -1000
            }
        }
        self.noise_value
    }

    pub fn sample(&mut self) -> i16 {
        let mut output: i16 = 0;
        let step: f64 = self.frequency * (std::f64::consts::PI * 2.0) / Common::SAMPLE_RATE as f64;
        self.time += step;

        if self.is_playing() && self.is_on {
            output = ((self.noise(self.time) as f64) * self.amplitude) as i16;
            if self.duration > 0 {
                self.duration -= 1;
            }
        }
        self.update_envelope();
        output
    }

    pub fn reset(&mut self, duration: i32) {
        self.amplitude = 1.0;
        self.envelope_time = 0;
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
}
