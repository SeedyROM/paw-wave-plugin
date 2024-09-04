//! PawWave Synthesizer Plugin
//!
//! This module implements a simple yet versatile monophonic synthesizer plugin called PawWave.
//! It features:
//! - A PolyBLEP oscillator with multiple waveform options (Sine, Square, Saw, Triangle)
//! - An ADSR (Attack, Decay, Sustain, Release) envelope
//! - Volume control with dB scaling
//! - MIDI input for note events
//! - Support for both CLAP and VST3 plugin formats
//!
//! The synthesizer is built using the nih-plug framework, making it compatible with various
//! digital audio workstations (DAWs) that support CLAP or VST3 plugins.

use nih_plug::prelude::*;
use std::sync::Arc;

mod envelope;
mod oscillator;

use envelope::ADSR;
use oscillator::{OscillatorType, PolyBlepOscillator};

// Main struct for the PawWave synthesizer
struct PawWave {
    params: Arc<PawWaveParams>,
    sample_rate: f32,
    osc: PolyBlepOscillator,
    adsr: ADSR,
    gain: Smoother<f32>,
}

// Parameters for the PawWave synthesizer
#[derive(Params)]
struct PawWaveParams {
    #[id = "volume"]
    pub volume: FloatParam,

    #[id = "waveform"]
    pub waveform: EnumParam<OscillatorType>,
}

impl Default for PawWave {
    fn default() -> Self {
        Self {
            params: Arc::new(PawWaveParams::default()),
            sample_rate: 44100.0,
            osc: PolyBlepOscillator::new(44100.0, 440.0), // Default to 440 Hz (A4)
            adsr: ADSR::default(),
            gain: Smoother::new(SmoothingStyle::Linear(5.0)),
        }
    }
}

impl Default for PawWaveParams {
    fn default() -> Self {
        Self {
            // Volume parameter with logarithmic scaling
            volume: FloatParam::new(
                "Volume",
                util::db_to_gain(-15.0), // Default to -15 dB
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),

            // Waveform selection parameter
            waveform: EnumParam::new("Waveform", OscillatorType::Sine),
        }
    }
}

impl Plugin for PawWave {
    // Plugin metadata
    const NAME: &'static str = "Paw Wave";
    const VENDOR: &'static str = "SeedyROM (Zack Kollar)";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "me@seedyrom.io";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // Supported audio I/O layouts
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2), // Stereo output
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(1), // Mono output
            ..AudioIOLayout::const_default()
        },
    ];

    // MIDI configuration
    const MIDI_INPUT: MidiConfig = MidiConfig::Basic;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let sample_rate = buffer_config.sample_rate;

        // Initialize components with the correct sample rate
        self.sample_rate = sample_rate;
        self.osc = PolyBlepOscillator::new(sample_rate, 440.0);
        self.adsr = ADSR::new(0.02, 0.02, 0.5, 0.5, sample_rate);

        true
    }

    fn reset(&mut self) {
        // Reset is not needed for this simple synth
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = context.next_event();

        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            // Process all MIDI events for this sample
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }

                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        self.osc.set_frequency(util::midi_note_to_freq(note));
                        self.adsr.on(velocity);
                    }
                    NoteEvent::NoteOff { .. } => {
                        self.adsr.off();
                    }
                    _ => (),
                }

                next_event = context.next_event();
            }

            // Get the smoothed volume
            let volume = self.params.volume.smoothed.next();

            // Compute the next ADSR value
            self.gain
                .set_target(self.sample_rate, self.adsr.next_sample());

            // Generate and process audio for all channels
            for sample in channel_samples {
                *sample = self.osc.next_sample(self.params.waveform.value()) * self.gain.next();
                *sample *= volume;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for PawWave {
    const CLAP_ID: &'static str = "com.seedyrom.paw-wave";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple synth.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // CLAP-specific features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Synthesizer, ClapFeature::Stereo];
}

impl Vst3Plugin for PawWave {
    const VST3_CLASS_ID: [u8; 16] = *b"SeedyROM-PawWave";

    // VST3-specific categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument];
}

// Export the plugin for CLAP and VST3 formats
nih_export_clap!(PawWave);
nih_export_vst3!(PawWave);
