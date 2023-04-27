use super::super::super::Common;

pub struct Wave {
    pub frequency: f64,
    pub time: f64,
    pub amplitude: f64,
    duration: i32, // duration in sample
    pub is_on: bool,
    pub is_playback: bool,
    pub wave_form: [i16; 0x20],
}

impl Wave {
    pub fn new() -> Wave {
        let mut wave = Wave {
            frequency: 0.0,
            time: 0.0,
            amplitude: 0.0,
            duration: 0,
            is_on: false,
            is_playback: false,
            wave_form: [0; 32],
        };
        for i in 0..32 {
            if i % 2 == 1 {
                wave.wave_form[i] = 1000;
            } else {
                wave.wave_form[i] = -1000;
            }
        }
        wave
    }

    fn wave_form_index(&self, time: f64) -> usize {
        let index: usize = ((time / (std::f64::consts::PI * 2.0) * 32.0) as i32 % 0x20) as usize;
        return index;
    }

    pub fn sample(&mut self) -> i16 {
        let mut output: i16 = 0;
        let step: f64 = self.frequency * (std::f64::consts::PI * 2.0) / Common::SAMPLE_RATE as f64;
        self.time += step;

        if self.is_playing() && self.is_on {
            output =
                ((self.wave_form[self.wave_form_index(self.time)] as f64) * self.amplitude) as i16;
            if self.duration > 0 {
                self.duration -= 1;
            }
        }
        output
    }

    pub fn reset(&mut self, duration: i32) {
        self.amplitude = 1.0;
        self.duration = duration;
    }

    fn is_playing(&mut self) -> bool {
        let mut result = false;

        if self.duration == -1 || self.duration > 0 {
            if self.is_playback {
                result = true;
            }
        }

        result
    }
}
