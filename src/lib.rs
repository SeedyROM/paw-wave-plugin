use nih_plug::prelude::*;
use std::sync::Arc;

mod osc;

use osc::{OscillatorType, PolyBlepOscillator};

struct PawWave {
    params: Arc<PawWaveParams>,
    osc: PolyBlepOscillator,
    gain: Smoother<f32>,
    sample_rate: f32,
}

#[derive(Params)]
struct PawWaveParams {
    #[id = "gain"]
    pub gain: FloatParam,

    #[id = "waveform"]
    pub waveform: EnumParam<OscillatorType>,
}

impl Default for PawWave {
    fn default() -> Self {
        Self {
            params: Arc::new(PawWaveParams::default()),
            osc: PolyBlepOscillator::new(44100.0, 440.0),
            gain: Smoother::new(SmoothingStyle::Linear(5.0)),
            sample_rate: 44100.0,
        }
    }
}

impl Default for PawWaveParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(-15.0),
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

            waveform: EnumParam::new("Waveform", OscillatorType::Sine),
        }
    }
}

impl Plugin for PawWave {
    const NAME: &'static str = "Paw Wave";
    const VENDOR: &'static str = "SeedyROM (Zack Kollar)";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "me@seedyrom.io";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            ..AudioIOLayout::const_default()
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(1),
            ..AudioIOLayout::const_default()
        },
    ];

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

        self.sample_rate = sample_rate;
        self.osc = PolyBlepOscillator::new(sample_rate, 440.0);

        true
    }

    fn reset(&mut self) {}

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let mut next_event = context.next_event();

        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() > sample_id as u32 {
                    break;
                }

                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        self.osc.set_frequency(util::midi_note_to_freq(note));
                        self.gain.set_target(self.sample_rate, velocity);
                    }
                    NoteEvent::NoteOff { .. } => {
                        self.gain.set_target(self.sample_rate, 0.0);
                    }
                    _ => (),
                }

                next_event = context.next_event();
            }

            let gain = self.params.gain.smoothed.next();

            for sample in channel_samples {
                *sample = self.osc.next_sample(self.params.waveform.value()) * self.gain.next();
                *sample *= gain;
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

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Synthesizer, ClapFeature::Stereo];
}

impl Vst3Plugin for PawWave {
    const VST3_CLASS_ID: [u8; 16] = *b"SeedyROM-PawWave";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument];
}

nih_export_clap!(PawWave);
nih_export_vst3!(PawWave);
