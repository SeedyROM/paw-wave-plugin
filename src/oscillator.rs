use std::f32::consts::PI;

use nih_plug::prelude::Enum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum OscillatorType {
    Sine,
    Square,
    Saw,
    Triangle,
}

pub struct PolyBlepOscillator {
    sample_rate: f32,
    frequency: f32,
    phase: f32,
    phase_increment: f32,
}

impl PolyBlepOscillator {
    #[inline(always)]
    pub fn new(sample_rate: f32, frequency: f32) -> Self {
        let mut osc = Self {
            sample_rate,
            frequency,
            phase: 0.0,
            phase_increment: 0.0,
        };
        osc.set_frequency(frequency);
        osc
    }

    #[inline(always)]
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.phase_increment = self.frequency / self.sample_rate;
    }

    #[inline(always)]
    fn poly_blep(&self, t: f32) -> f32 {
        if t < self.phase_increment {
            let t = t / self.phase_increment;
            2.0 * t - t * t - 1.0
        } else if t > 1.0 - self.phase_increment {
            let t = (t - 1.0) / self.phase_increment;
            t * t + 2.0 * t + 1.0
        } else {
            0.0
        }
    }

    #[inline(always)]
    pub fn next_sample(&mut self, osc_type: OscillatorType) -> f32 {
        let sample = match osc_type {
            OscillatorType::Sine => (2.0 * PI * self.phase).sin(),
            OscillatorType::Square => {
                let mut sample = if self.phase < 0.5 { 1.0 } else { -1.0 };
                sample += self.poly_blep(self.phase);
                sample -= self.poly_blep((self.phase + 0.5) % 1.0);
                sample
            }
            OscillatorType::Saw => {
                let mut sample = 2.0 * self.phase - 1.0;
                sample -= self.poly_blep(self.phase);
                sample
            }
            OscillatorType::Triangle => {
                let mut sample = if self.phase < 0.5 {
                    4.0 * self.phase - 1.0
                } else {
                    3.0 - 4.0 * self.phase
                };
                let dt = self.phase_increment;

                // Apply PolyBLEP at discontinuities
                sample += self.poly_blep(self.phase) * (4.0 * dt);
                sample -= self.poly_blep((self.phase + 0.5) % 1.0) * (4.0 * dt);

                sample
            }
        };

        self.phase += self.phase_increment;
        self.phase -= self.phase.floor();

        sample.clamp(-1.0, 1.0)
    }
}
