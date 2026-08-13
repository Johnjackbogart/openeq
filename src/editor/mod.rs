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
const KNOB_CONTROL_HEIGHT: f32 = 76.0;
const LOW_COLOR: egui::Color32 = egui::Color32::from_rgb(93, 123, 239);
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
    #[cfg(debug_assertions)]
    let show_layout_debug = ui.ctx().debug_on_hover();
    #[cfg(not(debug_assertions))]
    let show_layout_debug = false;

    ui.vertical_centered(|ui| {
        ui.add_space(12.0);
        ui.heading("OpenEQ");
        ui.label("Three-band stereo equalizer");
        ui.add_space(12.0);
        ui.add(ResponseCurve::new(settings, RESPONSE_SAMPLE_RATE));
        ui.add_space(18.0);

        ui.columns(3, |columns| {
            band_column(
                &mut columns[0],
                "LOW SHELF",
                LOW_COLOR,
                show_layout_debug,
                |ui| {
                    knob_row(ui, 2, |ui, slot_width| {
                        knob_control(
                            ui,
                            "Frequency",
                            low_frequency.with_accent(LOW_COLOR),
                            slot_width,
                            show_layout_debug,
                        );
                        knob_control(
                            ui,
                            "Gain",
                            low_gain.with_accent(LOW_COLOR),
                            slot_width,
                            show_layout_debug,
                        );
                    });
                },
            );
            band_column(
                &mut columns[1],
                "MID BELL",
                MID_COLOR,
                show_layout_debug,
                |ui| {
                    knob_row(ui, 3, |ui, slot_width| {
                        knob_control(
                            ui,
                            "Frequency",
                            mid_frequency.with_accent(MID_COLOR),
                            slot_width,
                            show_layout_debug,
                        );
                        knob_control(
                            ui,
                            "Gain",
                            mid_gain.with_accent(MID_COLOR),
                            slot_width,
                            show_layout_debug,
                        );
                        knob_control(
                            ui,
                            "Q",
                            mid_q.with_accent(MID_COLOR),
                            slot_width,
                            show_layout_debug,
                        );
                    });
                },
            );
            band_column(
                &mut columns[2],
                "HIGH SHELF",
                HIGH_COLOR,
                show_layout_debug,
                |ui| {
                    knob_row(ui, 2, |ui, slot_width| {
                        knob_control(
                            ui,
                            "Frequency",
                            high_frequency.with_accent(HIGH_COLOR),
                            slot_width,
                            show_layout_debug,
                        );
                        knob_control(
                            ui,
                            "Gain",
                            high_gain.with_accent(HIGH_COLOR),
                            slot_width,
                            show_layout_debug,
                        );
                    });
                },
            );
        });
    });
}

fn band_column(
    ui: &mut egui::Ui,
    title: &str,
    color: egui::Color32,
    show_layout_debug: bool,
    add_controls: impl FnOnce(&mut egui::Ui),
) {
    let response = ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new(title).small().strong().color(color));
        ui.add_space(4.0);
        add_controls(ui);
    });

    if show_layout_debug {
        response.response.paint_debug_info();
    }
}

fn knob_row(ui: &mut egui::Ui, knob_count: usize, add_controls: impl FnOnce(&mut egui::Ui, f32)) {
    let row_width = ui.available_width();
    let spacing = ui.spacing().item_spacing.x;
    let slot_width =
        (row_width - spacing * (knob_count.saturating_sub(1) as f32)) / knob_count as f32;

    ui.allocate_ui_with_layout(
        egui::vec2(row_width, KNOB_CONTROL_HEIGHT),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| add_controls(ui, slot_width),
    );
}

fn knob_control(
    ui: &mut egui::Ui,
    label: &str,
    knob: Knob<impl KnobValue>,
    slot_width: f32,
    show_layout_debug: bool,
) {
    let response = ui.allocate_ui_with_layout(
        egui::vec2(slot_width, KNOB_CONTROL_HEIGHT),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            let knob_response = ui.add(knob);
            if show_layout_debug {
                knob_response.paint_debug_info();
            }
            ui.label(egui::RichText::new(label).small());
        },
    );

    if show_layout_debug {
        response.response.paint_debug_info();
    }
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
                Knob::for_param(&params.low_frequency, setter),
                Knob::for_param(&params.low_gain, setter),
                Knob::for_param(&params.mid_frequency, setter),
                Knob::for_param(&params.mid_gain, setter),
                Knob::for_param(&params.mid_q, setter),
                Knob::for_param(&params.high_frequency, setter),
                Knob::for_param(&params.high_gain, setter),
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
