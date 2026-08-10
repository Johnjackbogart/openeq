//! Frequency-response display for the three-band EQ.

use crate::dsp::{self, EqSettings};

const MIN_FREQUENCY_HZ: f32 = 20.0;
const MAX_FREQUENCY_HZ: f32 = 20_000.0;
const DB_RANGE: f32 = 24.0;
const CURVE_SEGMENTS: usize = 128;

/// A logarithmic frequency-response plot. This widget is GUI-thread-only: it may allocate while
/// generating paint points, but it never participates in audio processing.
pub struct ResponseCurve {
    settings: EqSettings,
    sample_rate: f32,
    size: egui::Vec2,
}

impl ResponseCurve {
    pub fn new(settings: EqSettings, sample_rate: f32) -> Self {
        Self {
            settings,
            sample_rate,
            size: egui::vec2(600.0, 180.0),
        }
    }
}

impl egui::Widget for ResponseCurve {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) = ui.allocate_exact_size(self.size, egui::Sense::hover());

        if !ui.is_rect_visible(rect) {
            return response;
        }

        let painter = ui.painter_at(rect);
        let visuals = ui.visuals();
        let grid_color = visuals.widgets.noninteractive.bg_fill;
        painter.rect_filled(rect, 6.0, visuals.extreme_bg_color);
        painter.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, grid_color),
            egui::StrokeKind::Inside,
        );

        let x_for_frequency = |frequency_hz: f32| {
            let fraction =
                (frequency_hz / MIN_FREQUENCY_HZ).ln() / (MAX_FREQUENCY_HZ / MIN_FREQUENCY_HZ).ln();
            rect.left() + fraction * rect.width()
        };
        let y_for_db =
            |db: f32| rect.center().y - (db / DB_RANGE).clamp(-1.0, 1.0) * rect.height() * 0.5;

        for db in [-24.0, -12.0, 0.0, 12.0, 24.0] {
            let color = if db == 0.0 {
                visuals.widgets.active.bg_fill
            } else {
                grid_color
            };
            painter.hline(rect.x_range(), y_for_db(db), egui::Stroke::new(1.0, color));
        }
        for frequency_hz in [20.0, 100.0, 1_000.0, 10_000.0, 20_000.0] {
            painter.vline(
                x_for_frequency(frequency_hz),
                rect.y_range(),
                egui::Stroke::new(1.0, grid_color),
            );
        }

        let points = (0..=CURVE_SEGMENTS)
            .map(|index| {
                let fraction = index as f32 / CURVE_SEGMENTS as f32;
                let frequency_hz =
                    MIN_FREQUENCY_HZ * (MAX_FREQUENCY_HZ / MIN_FREQUENCY_HZ).powf(fraction);
                let magnitude =
                    dsp::response_magnitude(self.sample_rate, self.settings, frequency_hz);
                let db = 20.0 * magnitude.max(f32::MIN_POSITIVE).log10();
                egui::pos2(rect.left() + fraction * rect.width(), y_for_db(db))
            })
            .collect();
        painter.add(egui::Shape::line(
            points,
            egui::Stroke::new(2.5, egui::Color32::from_rgb(89, 198, 176)),
        ));

        response.on_hover_text("EQ frequency response: 20 Hz to 20 kHz, ±24 dB")
    }
}

#[cfg(test)]
mod tests {
    use super::{CURVE_SEGMENTS, ResponseCurve};
    use crate::dsp::{BandSettings, EqSettings};

    #[test]
    fn response_curve_paints_a_complete_line() {
        let settings = EqSettings {
            low: BandSettings {
                frequency_hz: 120.0,
                gain_db: 0.0,
                q: 1.0,
            },
            mid: BandSettings {
                frequency_hz: 1_000.0,
                gain_db: 6.0,
                q: 1.0,
            },
            high: BandSettings {
                frequency_hz: 8_000.0,
                gain_db: 0.0,
                q: 1.0,
            },
        };
        let context = egui::Context::default();
        let mut output = None;
        for _ in 0..2 {
            context.begin_pass(egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(640.0, 240.0),
                )),
                ..Default::default()
            });
            egui::Area::new(egui::Id::new("response-curve-test"))
                .fixed_pos(egui::Pos2::ZERO)
                .show(&context, |ui| {
                    ui.add(ResponseCurve::new(settings, 48_000.0));
                });
            output = Some(context.end_pass());
        }
        let output = output.expect("the two-pass loop always produces output");

        assert!(output.shapes.iter().any(|clipped| {
            matches!(&clipped.shape, egui::Shape::Path(path) if path.points.len() == CURVE_SEGMENTS + 1)
        }));
    }
}
