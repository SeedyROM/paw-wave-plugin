//! This module implements an ADSR (Attack, Decay, Sustain, Release) envelope generator.
//! ADSR envelopes are commonly used in sound synthesis to control the amplitude of a sound over time.

/// Represents an ADSR envelope generator.
#[derive(Debug, Clone, Copy)]
pub struct ADSR {
    /// Duration of the attack phase in seconds.
    attack: f32,
    /// Duration of the decay phase in seconds.
    decay: f32,
    /// Level of the sustain phase (0.0 to 1.0).
    sustain: f32,
    /// Duration of the release phase in seconds.
    release: f32,
    /// Sample rate of the audio system.
    sample_rate: f32,
    /// Current sample number being processed.
    current_sample: usize,
    /// Sample number when the note was triggered.
    trigger_sample: usize,
    /// Sample number when the note was released (if applicable).
    note_off_sample: Option<usize>,
    /// Current amplitude of the envelope (0.0 to 1.0).
    current_amplitude: f32,
    /// Velocity of the note (0.0 to 1.0).
    velocity: f32,
}

/// Struct for updating ADSR parameters.
#[derive(Debug, Clone, Copy)]
pub struct ADSRUpdate {
    /// New attack time (if provided).
    pub attack: Option<f32>,
    /// New decay time (if provided).
    pub decay: Option<f32>,
    /// New sustain level (if provided).
    pub sustain: Option<f32>,
    /// New release time (if provided).
    pub release: Option<f32>,
}

impl ADSR {
    /// Creates a new ADSR envelope generator.
    ///
    /// # Arguments
    ///
    /// * `attack` - Attack time in seconds
    /// * `decay` - Decay time in seconds
    /// * `sustain` - Sustain level (0.0 to 1.0)
    /// * `release` - Release time in seconds
    /// * `sample_rate` - Sample rate of the audio system
    ///
    /// # Returns
    ///
    /// A new [`ADSR`] instance
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32, sample_rate: f32) -> Self {
        ADSR {
            attack: attack.max(0.0),
            decay: decay.max(0.0),
            sustain: sustain.clamp(0.0, 1.0),
            release: release.max(0.0),
            sample_rate,
            current_sample: 0,
            trigger_sample: 0,
            note_off_sample: None,
            current_amplitude: 0.0,
            velocity: 1.0,
        }
    }

    /// Updates the ADSR parameters.
    ///
    /// # Arguments
    ///
    /// * `params` - An [`ADSRUpdate`] struct containing the parameters to update
    pub fn update_params(&mut self, params: ADSRUpdate) {
        if let Some(attack) = params.attack {
            self.attack = attack.max(0.0);
        }
        if let Some(decay) = params.decay {
            self.decay = decay.max(0.0);
        }
        if let Some(sustain) = params.sustain {
            self.sustain = sustain.clamp(0.0, 1.0);
        }
        if let Some(release) = params.release {
            self.release = release.max(0.0);
        }
    }

    /// Triggers the envelope with a given velocity.
    ///
    /// # Arguments
    ///
    /// * `velocity` - The velocity of the note (0.0 to 1.0)
    pub fn on(&mut self, velocity: f32) {
        self.trigger_sample = self.current_sample;
        self.note_off_sample = None;
        self.velocity = velocity.clamp(0.0, 1.0);
    }

    /// Releases the envelope, starting the release phase.
    pub fn off(&mut self) {
        if self.note_off_sample.is_none() {
            self.note_off_sample = Some(self.current_sample);

            // If the note was released before the attack phase is complete, skip to the release phase
            if self.current_sample < self.trigger_sample + (self.attack * self.sample_rate) as usize
            {
                self.current_sample =
                    self.trigger_sample + (self.attack * self.sample_rate) as usize;
            }
        }
    }

    /// Generates the next sample of the envelope.
    ///
    /// # Returns
    ///
    /// The current amplitude of the envelope (0.0 to 1.0)
    pub fn next_sample(&mut self) -> f32 {
        let time = (self.current_sample - self.trigger_sample) as f32 / self.sample_rate as f32;
        let attack_end = self.attack;
        let decay_end = attack_end + self.decay;

        let envelope = match self.note_off_sample {
            Some(note_off) if self.current_sample >= note_off => {
                // Release phase
                let release_time =
                    (self.current_sample - note_off) as f32 / self.sample_rate as f32;
                if release_time >= self.release {
                    0.0
                } else {
                    self.sustain * (1.0 - release_time / self.release)
                }
            }
            _ => {
                // Attack, Decay, or Sustain phase
                if time < attack_end {
                    // Attack
                    time / attack_end
                } else if time < decay_end {
                    // Decay
                    let decay_progress = (time - attack_end) / self.decay;
                    1.0 - (1.0 - self.sustain) * decay_progress
                } else {
                    // Sustain
                    self.sustain
                }
            }
        };

        self.current_amplitude = envelope * self.velocity;
        self.current_sample += 1;
        self.current_amplitude
    }

    /// Checks if the envelope is still active (non-zero amplitude).
    ///
    /// # Returns
    ///
    /// `true` if the envelope is still producing non-zero amplitude, `false` otherwise
    pub fn is_active(&self) -> bool {
        self.current_amplitude > 0.0
    }
}

impl Default for ADSR {
    /// Creates a default ADSR envelope with instant attack and sustain, and no decay or release.
    fn default() -> Self {
        ADSR {
            attack: 0.0,
            decay: 0.0,
            sustain: 1.0,
            release: 0.0,
            sample_rate: 44100.0,
            current_sample: 0,
            trigger_sample: 0,
            note_off_sample: None,
            current_amplitude: 0.0,
            velocity: 1.0,
        }
    }
}
