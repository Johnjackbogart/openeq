//! The native egui editor shared with `examples/editor_preview.rs`.

use std::sync::Arc;

use nice_plug::editor::Editor;
use nice_plug::editor::dpi::LogicalSize;
use nice_plug_egui::{EguiNiceSettings, EguiState, create_egui_editor};

use crate::OpenEqParams;
use crate::dsp::{BandSettings, EqSettings};

pub mod knob;
mod response_curve;

use knob::{Knob, KnobValue};
use response_curve::ResponseCurve;

const EDITOR_WIDTH: f32 = 640.0;
const EDITOR_HEIGHT: f32 = 430.0;
const RESPONSE_SAMPLE_RATE: f32 = 48_000.0;
const LOW_COLOR: egui::Color32 = egui::Color32::from_rgb(93, 167, 239);
const MID_COLOR: egui::Color32 = egui::Color32::from_rgb(89, 198, 176);
const HIGH_COLOR: egui::Color32 = egui::Color32::from_rgb(244, 174, 92);

pub(crate) fn default_state() -> Arc<EguiState> {
    EguiState::from_size(LogicalSize::new(EDITOR_WIDTH, EDITOR_HEIGHT))
}

/// Builds the editor layout used by both the plugin and the standalone preview.
#[allow(clippy::too_many_arguments)]
pub fn build_ui(
    ui: &mut egui::Ui,
    settings: EqSettings,
    low_frequency: Knob<impl KnobValue>,
    low_gain: Knob<impl KnobValue>,
    mid_frequency: Knob<impl KnobValue>,
    mid_gain: Knob<impl KnobValue>,
    mid_q: Knob<impl KnobValue>,
    high_frequency: Knob<impl KnobValue>,
    high_gain: Knob<impl KnobValue>,
) {
    ui.vertical_centered(|ui| {
        ui.add_space(12.0);
        ui.heading("OpenEQ");
        ui.label("Three-band stereo equalizer");
        ui.add_space(12.0);
        ui.add(ResponseCurve::new(settings, RESPONSE_SAMPLE_RATE));
        ui.add_space(18.0);

        ui.horizontal_top(|ui| {
            band_column(ui, "LOW SHELF", LOW_COLOR, |ui| {
                knob_control(ui, "Frequency", low_frequency);
                knob_control(ui, "Gain", low_gain);
            });
            ui.add_space(12.0);
            band_column(ui, "MID BELL", MID_COLOR, |ui| {
                knob_control(ui, "Frequency", mid_frequency);
                knob_control(ui, "Gain", mid_gain);
                knob_control(ui, "Q", mid_q);
            });
            ui.add_space(12.0);
            band_column(ui, "HIGH SHELF", HIGH_COLOR, |ui| {
                knob_control(ui, "Frequency", high_frequency);
                knob_control(ui, "Gain", high_gain);
            });
        });
    });
}

fn band_column(
    ui: &mut egui::Ui,
    title: &str,
    color: egui::Color32,
    add_controls: impl FnOnce(&mut egui::Ui),
) {
    ui.allocate_ui_with_layout(
        egui::vec2(190.0, 102.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            ui.label(egui::RichText::new(title).small().strong().color(color));
            ui.add_space(4.0);
            ui.horizontal_centered(add_controls);
        },
    );
}

fn knob_control(ui: &mut egui::Ui, label: &str, knob: Knob<impl KnobValue>) {
    ui.vertical_centered(|ui| {
        ui.add(knob);
        ui.label(egui::RichText::new(label).small());
    });
}

pub(crate) fn create(params: Arc<OpenEqParams>) -> Option<Box<dyn Editor>> {
    create_egui_editor(
        params.editor_state.clone(),
        (),
        EguiNiceSettings::new().with_tile("OpenEQ"),
        |_ctx, _commands, _state| {},
        move |ui, setter, _commands, _state| {
            let settings = params_to_settings(&params);
            build_ui(
                ui,
                settings,
                Knob::for_param(&params.low_frequency, setter).with_accent(LOW_COLOR),
                Knob::for_param(&params.low_gain, setter).with_accent(LOW_COLOR),
                Knob::for_param(&params.mid_frequency, setter).with_accent(MID_COLOR),
                Knob::for_param(&params.mid_gain, setter).with_accent(MID_COLOR),
                Knob::for_param(&params.mid_q, setter).with_accent(MID_COLOR),
                Knob::for_param(&params.high_frequency, setter).with_accent(HIGH_COLOR),
                Knob::for_param(&params.high_gain, setter).with_accent(HIGH_COLOR),
            );
        },
    )
}

fn params_to_settings(params: &OpenEqParams) -> EqSettings {
    EqSettings {
        low: BandSettings {
            frequency_hz: params.low_frequency.value(),
            gain_db: params.low_gain.value(),
            q: 1.0,
        },
        mid: BandSettings {
            frequency_hz: params.mid_frequency.value(),
            gain_db: params.mid_gain.value(),
            q: params.mid_q.value(),
        },
        high: BandSettings {
            frequency_hz: params.high_frequency.value(),
            gain_db: params.high_gain.value(),
            q: 1.0,
        },
    }
}
