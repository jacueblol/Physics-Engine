use std::f32::consts::TAU;

use eframe::egui::{self, Color32, Pos2, Stroke};

use crate::graph::Mode;
use crate::motor::Motor;

pub struct SceneWindow {
    pub open: bool,
    scale: f32,
    arm_length: f32,
}

fn w2s(wx: f32, wy: f32, center: Pos2, scale: f32) -> Pos2 {
    egui::pos2(center.x + wx * scale, center.y - wy * scale)
}

impl SceneWindow {
    pub fn new() -> Self {
        Self {
            open: false,
            scale: 120.0,
            arm_length: 1.0,
        }
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        motor: &Motor,
        control_mode: Mode,
        pos_target: f32,
    ) {
        egui::Window::new("Scene Viewer")
            .open(&mut self.open)
            .default_size([380.0, 400.0])
            .resizable(true)
            .show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.scale, 40.0..=300.0)
                        .text("Zoom")
                        .step_by(10.0),
                );

                let (rect, _) =
                    ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());

                if rect.width() < 10.0 || rect.height() < 10.0 {
                    return;
                }

                let painter = ui.painter_at(rect);
                let center = rect.center();
                let scale = self.scale;
                let arm = self.arm_length;

                // Background
                painter.rect_filled(rect, 0.0, Color32::from_gray(25));

                // Reference circles at 0.5 and 1.0 world units
                for r in [0.5_f32, 1.0] {
                    painter.circle_stroke(
                        center,
                        r * scale,
                        Stroke::new(0.5, Color32::from_gray(55)),
                    );
                }

                // Axis crosshairs
                let axis_stroke = Stroke::new(0.5, Color32::from_gray(50));
                painter.line_segment(
                    [
                        w2s(-arm * 1.4, 0.0, center, scale),
                        w2s(arm * 1.4, 0.0, center, scale),
                    ],
                    axis_stroke,
                );
                painter.line_segment(
                    [
                        w2s(0.0, -arm * 1.4, center, scale),
                        w2s(0.0, arm * 1.4, center, scale),
                    ],
                    axis_stroke,
                );

                // Setpoint line (Position mode only)
                if control_mode == Mode::Position {
                    let sp = pos_target;
                    let sp_tip = w2s(sp.cos() * arm, sp.sin() * arm, center, scale);
                    painter.line_segment(
                        [center, sp_tip],
                        Stroke::new(1.5, Color32::from_rgba_premultiplied(220, 70, 70, 140)),
                    );
                    painter.circle_filled(
                        sp_tip,
                        5.0,
                        Color32::from_rgba_premultiplied(220, 70, 70, 180),
                    );
                }

                // Motor housing
                painter.circle_filled(center, 14.0, Color32::from_gray(75));
                painter.circle_stroke(center, 14.0, Stroke::new(1.5, Color32::from_gray(140)));

                // Rotating shaft arm
                let angle = motor.position().rem_euclid(TAU);
                let tip = w2s(angle.cos() * arm, angle.sin() * arm, center, scale);

                painter.line_segment(
                    [center, tip],
                    Stroke::new(3.5, Color32::from_rgb(90, 160, 230)),
                );

                // Tip indicator
                painter.circle_filled(tip, 7.0, Color32::from_rgb(90, 160, 230));
                painter.circle_stroke(tip, 7.0, Stroke::new(1.5, Color32::WHITE));

                // Angle label offset from tip
                let label_pos = w2s(
                    angle.cos() * (arm + 0.18),
                    angle.sin() * (arm + 0.18),
                    center,
                    scale,
                );
                let deg = motor.position().to_degrees().rem_euclid(360.0);
                painter.text(
                    label_pos,
                    egui::Align2::CENTER_CENTER,
                    format!("{deg:.1}°"),
                    egui::FontId::proportional(12.0),
                    Color32::WHITE,
                );
            });
    }
}
