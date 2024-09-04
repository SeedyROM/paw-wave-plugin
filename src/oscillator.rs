//! This module implements a PolyBLEP (Polynomial Bandlimited Step Function) oscillator.
//! PolyBLEP is a technique used to reduce aliasing in digital oscillators, particularly
//! for non-sinusoidal waveforms like square, saw, and triangle waves.

use std::f32::consts::PI;

use nih_plug::prelude::Enum;

/// Represents different types of oscillator waveforms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Enum)]
pub enum OscillatorType {
    /// Sine wave oscillator
    Sine,
    /// Square wave oscillator
    Square,
    /// Sawtooth wave oscillator
    Saw,
    /// Triangle wave oscillator
    Triangle,
}

/// A PolyBLEP oscillator capable of generating various waveforms with reduced aliasing.
pub struct PolyBlepOscillator {
    /// The sample rate of the audio system.
    sample_rate: f32,
    /// The frequency of the oscillator.
    frequency: f32,
    /// The current phase of the oscillator (0.0 to 1.0).
    phase: f32,
    /// The amount to increment the phase each sample.
    phase_increment: f32,
}

impl PolyBlepOscillator {
    /// Creates a new PolyBLEP oscillator.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - The sample rate of the audio system
    /// * `frequency` - The initial frequency of the oscillator
    ///
    /// # Returns
    ///
    /// A new [`PolyBlepOscillator`] instance
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

    /// Sets the frequency of the oscillator.
    ///
    /// # Arguments
    ///
    /// * `frequency` - The new frequency in Hz
    #[inline(always)]
    pub fn set_frequency(&mut self, frequency: f32) {
        self.frequency = frequency;
        self.phase_increment = self.frequency / self.sample_rate;
    }

    /// Applies the PolyBLEP correction to reduce aliasing at discontinuities.
    ///
    /// # Arguments
    ///
    /// * `t` - The phase at which to apply the correction
    ///
    /// # Returns
    ///
    /// The PolyBLEP correction value
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

    /// Generates the next sample of the oscillator.
    ///
    /// # Arguments
    ///
    /// * `osc_type` - The type of oscillator waveform to generate
    ///
    /// # Returns
    ///
    /// The next sample value in the range [-1.0, 1.0]
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
