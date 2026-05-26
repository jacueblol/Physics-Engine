use eframe::egui::{self, Color32, Pos2, Shape, Stroke};

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
                let display_arm = motor.link.as_ref().map_or(arm, |l| l.length);
                if control_mode == Mode::Position {
                    let sp = pos_target;
                    let sp_tip = w2s(sp.cos() * display_arm, sp.sin() * display_arm, center, scale);
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

                // Rotating arm / link
                let angle = motor.position();
                let cos_a = angle.cos();
                let sin_a = angle.sin();

                let tip_length = motor.link.as_ref().map_or(arm, |l| l.length);

                if let Some(link) = &motor.link {
                    // Draw filled rectangle for the rigid link block
                    let hw = link.width / 2.0;
                    let l = link.length;
                    // 4 corners in world space (pivot at origin)
                    let corners = vec![
                        w2s(-sin_a * hw,              cos_a * hw,              center, scale),
                        w2s( cos_a * l - sin_a * hw,  sin_a * l + cos_a * hw, center, scale),
                        w2s( cos_a * l + sin_a * hw,  sin_a * l - cos_a * hw, center, scale),
                        w2s( sin_a * hw,              -cos_a * hw,             center, scale),
                    ];
                    painter.add(Shape::convex_polygon(
                        corners,
                        Color32::from_rgb(70, 130, 200),
                        Stroke::new(1.5, Color32::from_rgb(140, 190, 255)),
                    ));
                } else {
                    // Fallback: simple line arm
                    let tip = w2s(cos_a * arm, sin_a * arm, center, scale);
                    painter.line_segment(
                        [center, tip],
                        Stroke::new(3.5, Color32::from_rgb(90, 160, 230)),
                    );
                    painter.circle_filled(tip, 7.0, Color32::from_rgb(90, 160, 230));
                    painter.circle_stroke(tip, 7.0, Stroke::new(1.5, Color32::WHITE));
                }

                // Pivot joint pin (always drawn on top)
                painter.circle_filled(center, 5.0, Color32::from_gray(200));
                painter.circle_stroke(center, 5.0, Stroke::new(1.5, Color32::WHITE));

                // Angle label at tip
                let label_pos = w2s(
                    cos_a * (tip_length + 0.18),
                    sin_a * (tip_length + 0.18),
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
