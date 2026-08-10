//! OpenEQ: a real-time-safe, three-band stereo equalizer plugin.

use std::sync::Arc;

use nice_plug::prelude::*;

pub mod dsp;
pub mod editor;

const PARAMETER_SMOOTHING_MS: f32 = 20.0;

struct OpenEq {
    params: Arc<OpenEqParams>,
    equalizer: dsp::StereoEq,
    sample_rate: f32,
}

#[derive(Params)]
struct OpenEqParams {
    #[persist = "editor-state"]
    editor_state: Arc<nice_plug_egui::EguiState>,

    #[id = "low_frequency"]
    low_frequency: FloatParam,

    #[id = "low_gain"]
    low_gain: FloatParam,

    #[id = "mid_frequency"]
    mid_frequency: FloatParam,

    #[id = "mid_gain"]
    mid_gain: FloatParam,

    #[id = "mid_q"]
    mid_q: FloatParam,

    #[id = "high_frequency"]
    high_frequency: FloatParam,

    #[id = "high_gain"]
    high_gain: FloatParam,
}

impl Default for OpenEq {
    fn default() -> Self {
        Self {
            params: Arc::new(OpenEqParams::default()),
            equalizer: dsp::StereoEq::default(),
            sample_rate: 48_000.0,
        }
    }
}

impl Default for OpenEqParams {
    fn default() -> Self {
        Self {
            editor_state: editor::default_state(),
            low_frequency: frequency_param("Low Frequency", 120.0, 40.0, 400.0),
            low_gain: gain_param("Low Gain", 0.0),
            mid_frequency: frequency_param("Mid Frequency", 1_000.0, 200.0, 5_000.0),
            mid_gain: gain_param("Mid Gain", 0.0),
            mid_q: FloatParam::new("Mid Q", 1.0, FloatRange::Linear { min: 0.3, max: 6.0 })
                .with_smoother(SmoothingStyle::Linear(PARAMETER_SMOOTHING_MS))
                .with_value_to_string(formatters::v2s_f32_rounded(2)),
            high_frequency: frequency_param("High Frequency", 8_000.0, 2_000.0, 18_000.0),
            high_gain: gain_param("High Gain", 0.0),
        }
    }
}

impl Plugin for OpenEq {
    const NAME: &'static str = "OpenEQ";
    const VENDOR: &'static str = "OpenEQ Contributors";
    const URL: &'static str = "https://github.com/johnjackbogart/openeq";
    const EMAIL: &'static str = "opensource@openeq.dev";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),
        ..AudioIOLayout::const_default()
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone())
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        self.sample_rate = buffer_config.sample_rate;
        self.equalizer.reset();
        true
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _auxiliary_buffers: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for mut frame in buffer.iter_samples() {
            // Parameter values advance exactly once per frame, keeping the stereo channels
            // synchronized and avoiding per-channel smoother/format conversion work.
            self.equalizer.update(
                self.sample_rate,
                dsp::EqSettings {
                    low: dsp::BandSettings {
                        frequency_hz: self.params.low_frequency.smoothed.next(),
                        gain_db: self.params.low_gain.smoothed.next(),
                        q: 1.0,
                    },
                    mid: dsp::BandSettings {
                        frequency_hz: self.params.mid_frequency.smoothed.next(),
                        gain_db: self.params.mid_gain.smoothed.next(),
                        q: self.params.mid_q.smoothed.next(),
                    },
                    high: dsp::BandSettings {
                        frequency_hz: self.params.high_frequency.smoothed.next(),
                        gain_db: self.params.high_gain.smoothed.next(),
                        q: 1.0,
                    },
                },
            );

            let mut channels = frame.iter_mut();
            if let Some(left) = channels.next() {
                *left = self.equalizer.process_left(*left);
            }
            if let Some(right) = channels.next() {
                *right = self.equalizer.process_right(*right);
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for OpenEq {
    // Stable public identifier: do not change after a released build.
    const CLAP_ID: &'static str = "io.github.johnjackbogart.openeq";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A three-band stereo equalizer.");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_FEATURES: &'static [ClapFeature] = &[
        ClapFeature::AudioEffect,
        ClapFeature::Equalizer,
        ClapFeature::Stereo,
    ];
}

impl Vst3Plugin for OpenEq {
    // Stable public identifier: exactly 16 bytes, do not change after a released build.
    const VST3_CLASS_ID: [u8; 16] = *b"OpenEqThreeBand1";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Eq];
}

nice_export_clap!(OpenEq);
nice_export_vst3!(OpenEq);

fn gain_param(name: &str, default: f32) -> FloatParam {
    FloatParam::new(
        name,
        default,
        FloatRange::Linear {
            min: -24.0,
            max: 24.0,
        },
    )
    .with_smoother(SmoothingStyle::Linear(PARAMETER_SMOOTHING_MS))
    .with_unit(" dB")
    .with_value_to_string(formatters::v2s_f32_rounded(2))
}

fn frequency_param(name: &str, default: f32, min: f32, max: f32) -> FloatParam {
    FloatParam::new(name, default, FloatRange::Linear { min, max })
        .with_smoother(SmoothingStyle::Linear(PARAMETER_SMOOTHING_MS))
        .with_unit(" Hz")
        .with_value_to_string(formatters::v2s_f32_rounded(0))
}
