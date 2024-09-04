#[derive(Debug, Clone, Copy)]
pub struct ADSR {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    sample_rate: f32,
    current_sample: usize,
    trigger_sample: usize,
    note_off_sample: Option<usize>,
    current_amplitude: f32,
    velocity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct ADSRUpdate {
    pub attack: Option<f32>,
    pub decay: Option<f32>,
    pub sustain: Option<f32>,
    pub release: Option<f32>,
}

impl ADSR {
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

    pub fn on(&mut self, velocity: f32) {
        self.trigger_sample = self.current_sample;
        self.note_off_sample = None;
        self.velocity = velocity.clamp(0.0, 1.0);
    }

    pub fn off(&mut self) {
        if self.note_off_sample.is_none() {
            self.note_off_sample = Some(self.current_sample);
        }
    }

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

    pub fn is_active(&self) -> bool {
        self.current_amplitude > 0.0
    }
}

impl Default for ADSR {
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
