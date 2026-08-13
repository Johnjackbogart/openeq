//! Standalone custom-editor preview. Run with `cargo run --example editor_preview`.

use baseview::dpi::{LogicalSize, Size};
use egui::{CentralPanel, Context, FullOutput, Ui, ViewportOutput};
use egui_baseview::{EguiWindow, EguiWindowSettings, ExtraOutputCommands};
use openeq::dsp::{BandSettings, EqSettings};
use openeq::editor::knob::{Knob, KnobValue};

const WIDTH: f32 = 640.0;
const HEIGHT: f32 = 430.0;

struct MockKnobValue<'a> {
    name: &'static str,
    normalized: &'a mut f32,
    default_normalized: f32,
    display: fn(f32) -> String,
}

impl KnobValue for MockKnobValue<'_> {
    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn normalized(&self) -> f32 {
        *self.normalized
    }

    fn default_normalized(&self) -> f32 {
        self.default_normalized
    }

    fn display(&self) -> String {
        (self.display)(*self.normalized)
    }

    fn begin_set(&mut self) {}

    fn set_normalized(&mut self, value: f32) {
        *self.normalized = value;
    }

    fn end_set(&mut self) {}
}

struct PreviewState {
    #[cfg(debug_assertions)]
    show_layout_debug: bool,
    low_frequency: f32,
    low_gain: f32,
    mid_frequency: f32,
    mid_gain: f32,
    mid_q: f32,
    high_frequency: f32,
    high_gain: f32,
}

impl Default for PreviewState {
    fn default() -> Self {
        Self {
            #[cfg(debug_assertions)]
            show_layout_debug: false,
            low_frequency: normalized(120.0, 40.0, 400.0),
            low_gain: normalized(0.0, -24.0, 24.0),
            mid_frequency: normalized(1_000.0, 200.0, 5_000.0),
            mid_gain: normalized(0.0, -24.0, 24.0),
            mid_q: normalized(1.0, 0.3, 6.0),
            high_frequency: normalized(8_000.0, 2_000.0, 18_000.0),
            high_gain: normalized(0.0, -24.0, 24.0),
        }
    }
}

impl PreviewState {
    fn settings(&self) -> EqSettings {
        EqSettings {
            low: BandSettings {
                frequency_hz: denormalized(self.low_frequency, 40.0, 400.0),
                gain_db: denormalized(self.low_gain, -24.0, 24.0),
                q: 1.0,
            },
            mid: BandSettings {
                frequency_hz: denormalized(self.mid_frequency, 200.0, 5_000.0),
                gain_db: denormalized(self.mid_gain, -24.0, 24.0),
                q: denormalized(self.mid_q, 0.3, 6.0),
            },
            high: BandSettings {
                frequency_hz: denormalized(self.high_frequency, 2_000.0, 18_000.0),
                gain_db: denormalized(self.high_gain, -24.0, 24.0),
                q: 1.0,
            },
        }
    }
}

fn main() {
    EguiWindow::open_blocking(
        EguiWindowSettings::new()
            .with_tile("OpenEQ editor preview")
            .with_size(Size::Logical(LogicalSize {
                width: f64::from(WIDTH),
                height: f64::from(HEIGHT),
            })),
        PreviewState::default(),
        |_ctx: &Context, _commands: &mut ExtraOutputCommands, _state: &mut PreviewState| {},
        |_output: &FullOutput, _viewport_output: &ViewportOutput, _state: &mut PreviewState| {},
        |ui: &mut Ui, _commands: &mut ExtraOutputCommands, state: &mut PreviewState| {
            #[cfg(debug_assertions)]
            ui.ctx().all_styles_mut(|style| {
                style.debug.debug_on_hover = state.show_layout_debug;
                style.debug.show_interactive_widgets = state.show_layout_debug;
                style.debug.show_widget_hits = state.show_layout_debug;
            });

            let settings = state.settings();
            CentralPanel::default().show(ui, |ui| {
                #[cfg(debug_assertions)]
                ui.checkbox(&mut state.show_layout_debug, "Layout debug (hover)");
                openeq::editor::build_ui(
                    ui,
                    settings,
                    Knob::new(MockKnobValue {
                        name: "Low Frequency",
                        normalized: &mut state.low_frequency,
                        default_normalized: normalized(120.0, 40.0, 400.0),
                        display: format_low_frequency,
                    }),
                    Knob::new(MockKnobValue {
                        name: "Low Gain",
                        normalized: &mut state.low_gain,
                        default_normalized: normalized(0.0, -24.0, 24.0),
                        display: format_gain,
                    }),
                    Knob::new(MockKnobValue {
                        name: "Mid Frequency",
                        normalized: &mut state.mid_frequency,
                        default_normalized: normalized(1_000.0, 200.0, 5_000.0),
                        display: format_mid_frequency,
                    }),
                    Knob::new(MockKnobValue {
                        name: "Mid Gain",
                        normalized: &mut state.mid_gain,
                        default_normalized: normalized(0.0, -24.0, 24.0),
                        display: format_gain,
                    }),
                    Knob::new(MockKnobValue {
                        name: "Mid Q",
                        normalized: &mut state.mid_q,
                        default_normalized: normalized(1.0, 0.3, 6.0),
                        display: format_q,
                    }),
                    Knob::new(MockKnobValue {
                        name: "High Frequency",
                        normalized: &mut state.high_frequency,
                        default_normalized: normalized(8_000.0, 2_000.0, 18_000.0),
                        display: format_high_frequency,
                    }),
                    Knob::new(MockKnobValue {
                        name: "High Gain",
                        normalized: &mut state.high_gain,
                        default_normalized: normalized(0.0, -24.0, 24.0),
                        display: format_gain,
                    }),
                );
            });
        },
    );
}

fn normalized(value: f32, min: f32, max: f32) -> f32 {
    (value - min) / (max - min)
}

fn denormalized(value: f32, min: f32, max: f32) -> f32 {
    min + value * (max - min)
}

fn format_low_frequency(value: f32) -> String {
    format!("{:.0} Hz", denormalized(value, 40.0, 400.0))
}

fn format_mid_frequency(value: f32) -> String {
    format!("{:.0} Hz", denormalized(value, 200.0, 5_000.0))
}

fn format_high_frequency(value: f32) -> String {
    format!("{:.0} Hz", denormalized(value, 2_000.0, 18_000.0))
}

fn format_gain(value: f32) -> String {
    format!("{:.2} dB", denormalized(value, -24.0, 24.0))
}

fn format_q(value: f32) -> String {
    format!("{:.2}", denormalized(value, 0.3, 6.0))
}
