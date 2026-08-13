//! A compact rotary control shared by the plugin editor and standalone preview.

use core::f32::consts::PI;

use nice_plug::context::gui::ParamSetter;
use nice_plug::params::Param;

const ANGLE_START: f32 = PI * 0.75;
const ANGLE_SWEEP: f32 = PI * 1.5;
const ARC_SEGMENTS: usize = 48;

/// The minimal interface a rotary control needs.
///
/// The real editor adapts a `nice_plug::Param` through [`ParamKnobValue`]. The standalone preview
/// supplies in-memory values, ensuring both surfaces execute the exact same layout and interaction
/// code.
pub trait KnobValue {
    fn name(&self) -> String;
    fn normalized(&self) -> f32;
    fn default_normalized(&self) -> f32;
    fn display(&self) -> String;

    fn begin_set(&mut self);
    fn set_normalized(&mut self, value: f32);
    fn end_set(&mut self);
}

/// Adapts a host parameter to [`KnobValue`] while preserving DAW automation gestures.
pub struct ParamKnobValue<'a, P: Param> {
    param: &'a P,
    setter: &'a ParamSetter<'a>,
}

impl<P: Param> KnobValue for ParamKnobValue<'_, P> {
    fn name(&self) -> String {
        self.param.name().to_string()
    }

    fn normalized(&self) -> f32 {
        self.param.unmodulated_normalized_value()
    }

    fn default_normalized(&self) -> f32 {
        self.param.default_normalized_value()
    }

    fn display(&self) -> String {
        self.param.to_string()
    }

    fn begin_set(&mut self) {
        self.setter.begin_set_parameter(self.param);
    }

    fn set_normalized(&mut self, value: f32) {
        self.setter.set_parameter_normalized(self.param, value);
    }

    fn end_set(&mut self) {
        self.setter.end_set_parameter(self.param);
    }
}

/// A rotary widget with vertical drag, shift-fine-adjust, and double-click reset behavior.
pub struct Knob<V> {
    value: V,
    diameter: f32,
    accent: egui::Color32,
}

impl<V: KnobValue> Knob<V> {
    pub fn new(value: V) -> Self {
        Self {
            value,
            diameter: 56.0,
            accent: egui::Color32::from_rgb(89, 198, 176),
        }
    }

    pub fn with_accent(mut self, accent: egui::Color32) -> Self {
        self.accent = accent;
        self
    }
}

impl<'a, P: Param> Knob<ParamKnobValue<'a, P>> {
    pub fn for_param(param: &'a P, setter: &'a ParamSetter<'a>) -> Self {
        Self::new(ParamKnobValue { param, setter })
    }
}

impl<V: KnobValue> egui::Widget for Knob<V> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let desired_size = egui::Vec2::splat(self.diameter);
        let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click_and_drag());

        if response.drag_started() {
            self.value.begin_set();
        }
        if response.dragged() {
            let speed = if ui.input(|input| input.modifiers.shift) {
                0.0015
            } else {
                0.005
            };
            let value = (self.value.normalized() - response.drag_delta().y * speed).clamp(0.0, 1.0);
            self.value.set_normalized(value);
        }
        if response.drag_stopped() {
            self.value.end_set();
        }
        if response.double_clicked() {
            self.value.begin_set();
            self.value.set_normalized(self.value.default_normalized());
            self.value.end_set();
        }

        if ui.is_rect_visible(rect) {
            let painter = ui.painter();
            let center = rect.center();
            let radius = rect.width() * 0.5;
            let normalized = self.value.normalized();
            let track = egui::Stroke::new(3.0, ui.visuals().widgets.inactive.bg_fill);
            // Keep the active arc tied to the band's accent in every interaction state. Using
            // egui's hover foreground here would replace this color while dragging a knob.
            let value_color = self.accent;

            painter.circle(
                center,
                radius * 0.72,
                ui.visuals().widgets.inactive.weak_bg_fill,
                egui::Stroke::new(1.0, ui.visuals().widgets.inactive.bg_fill),
            );
            painter.add(egui::Shape::line(
                arc_points(center, radius, ANGLE_START, ANGLE_START + ANGLE_SWEEP),
                track,
            ));
            painter.add(egui::Shape::line(
                arc_points(
                    center,
                    radius,
                    ANGLE_START,
                    ANGLE_START + normalized * ANGLE_SWEEP,
                ),
                egui::Stroke::new(3.0, value_color),
            ));

            let angle = ANGLE_START + normalized * ANGLE_SWEEP;
            painter.line_segment(
                [
                    center,
                    center + egui::Vec2::new(angle.cos(), angle.sin()) * (radius * 0.58),
                ],
                egui::Stroke::new(2.0, ui.visuals().text_color()),
            );
        }

        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::WidgetType::Slider, true, self.value.name())
        });
        response.on_hover_text(self.value.display())
    }
}

fn arc_points(
    center: egui::Pos2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
) -> Vec<egui::Pos2> {
    (0..=ARC_SEGMENTS)
        .map(|index| {
            let t = index as f32 / ARC_SEGMENTS as f32;
            let angle = start_angle + (end_angle - start_angle) * t;
            center + egui::Vec2::new(angle.cos(), angle.sin()) * radius
        })
        .collect()
}
