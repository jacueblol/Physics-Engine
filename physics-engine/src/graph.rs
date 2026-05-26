use std::collections::VecDeque;

use crate::link::Link;
use crate::motor::{DriveMode, Motor};
use crate::scene::SceneWindow;
use eframe::egui;
use egui_plot::{HLine, Line, LineStyle, Plot, PlotPoints};

const LOAD_MODES: &[&str] = &["None", "Constant", "Spring", "Fan/Pump", "Pendulum"];

#[derive(PartialEq, Clone, Copy, Default)]
pub enum Mode {
    #[default]
    Voltage,
    Pwm,
    Position,
    Velocity,
}

#[derive(PartialEq, Clone, Copy)]
enum LoadMode {
    None,
    Constant,
    Spring,
    FanPump,
    Pendulum,
}

struct StepState {
    time: f64,
    initial: f64,
    target: f64,
}

struct StepMetrics {
    steady_state: f64,
    error: f64,
    overshoot_pct: f64,
    rise_time: Option<f64>,
    settling_time: Option<f64>,
}

pub struct GraphApp {
    time: f64,
    dt: f64,
    paused: bool,
    sim_speed: f64,
    time_window: f64,

    motor: Motor,

    control_mode: Mode,
    prev_control_mode: Mode,
    voltage_input: f32,
    pwm_input: f32,
    pos_target: f32,
    vel_target: f32,

    step_state: Option<StepState>,
    prev_target: f64,

    scene: SceneWindow,

    link_enabled: bool,
    link_mass: f32,
    link_length: f32,
    link_width: f32,

    load_mode: LoadMode,
    load_torque_val: f32,
    spring_k: f32,
    drag_k: f32,
    pendulum_k: f32,

    position_history: VecDeque<[f64; 2]>,
    velocity_history: VecDeque<[f64; 2]>,
    current_history: VecDeque<[f64; 2]>,
    temperature_history: VecDeque<[f64; 2]>,
    torque_history: VecDeque<[f64; 2]>,
    back_emf_history: VecDeque<[f64; 2]>,
    power_history: VecDeque<[f64; 2]>,
}

fn trim(deque: &mut VecDeque<[f64; 2]>, min_t: f64) {
    while deque.front().is_some_and(|p| p[0] < min_t) {
        deque.pop_front();
    }
}

fn to_vec(deque: &VecDeque<[f64; 2]>) -> Vec<[f64; 2]> {
    deque.iter().cloned().collect()
}

impl GraphApp {
    pub fn new(motor: Motor) -> Self {
        let voltage_input = motor.applied_voltage();
        Self {
            time: 0.0,
            dt: 0.001,
            paused: false,
            sim_speed: 1.0,
            time_window: 10.0,

            motor,

            control_mode: Mode::Voltage,
            prev_control_mode: Mode::Voltage,
            voltage_input,
            pwm_input: 0.0,
            pos_target: 0.0,
            vel_target: 0.0,

            step_state: None,
            prev_target: 0.0,

            scene: SceneWindow::new(),

            link_enabled: true,
            link_mass: 0.5,
            link_length: 1.0,
            link_width: 0.1,

            load_mode: LoadMode::None,
            load_torque_val: 0.0,
            spring_k: 1.0,
            drag_k: 0.01,
            pendulum_k: 5.0,

            position_history: VecDeque::new(),
            velocity_history: VecDeque::new(),
            current_history: VecDeque::new(),
            temperature_history: VecDeque::new(),
            torque_history: VecDeque::new(),
            back_emf_history: VecDeque::new(),
            power_history: VecDeque::new(),
        }
    }

    fn load_torque(&self) -> f32 {
        let pos = self.motor.position();
        let vel = self.motor.velocity();
        match self.load_mode {
            LoadMode::None => 0.0,
            LoadMode::Constant => self.load_torque_val,
            LoadMode::Spring => self.spring_k * pos,
            LoadMode::FanPump => self.drag_k * vel * vel.abs(),
            LoadMode::Pendulum => self.pendulum_k * pos.sin(),
        }
    }

    fn step_metrics(&self) -> Option<StepMetrics> {
        let step = self.step_state.as_ref()?;
        let history = match self.control_mode {
            Mode::Position => &self.position_history,
            Mode::Velocity => &self.velocity_history,
            _ => return None,
        };

        let step_size = step.target - step.initial;
        if step_size.abs() < 1e-6 {
            return None;
        }

        let samples: Vec<[f64; 2]> = history
            .iter()
            .filter(|p| p[0] >= step.time)
            .cloned()
            .collect();

        if samples.len() < 2 {
            return None;
        }

        // Steady state: mean of last 100 samples
        let n_avg = 100.min(samples.len());
        let steady_state = samples[samples.len() - n_avg..]
            .iter()
            .map(|p| p[1])
            .sum::<f64>()
            / n_avg as f64;
        let error = step.target - steady_state;

        // Overshoot
        let peak = if step_size > 0.0 {
            samples.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max)
        } else {
            samples.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min)
        };
        let overshoot_pct = if step_size > 0.0 {
            ((peak - step.target) / step_size.abs() * 100.0).max(0.0)
        } else {
            ((step.target - peak) / step_size.abs() * 100.0).max(0.0)
        };

        // Rise time: t10 → t90
        let t10_val = step.initial + 0.1 * step_size;
        let t90_val = step.initial + 0.9 * step_size;
        let cross10 = samples.iter().find(|p| {
            if step_size > 0.0 { p[1] >= t10_val } else { p[1] <= t10_val }
        });
        let rise_time = cross10.and_then(|c10| {
            let after: Vec<_> = samples.iter().filter(|p| p[0] >= c10[0]).collect();
            after
                .iter()
                .find(|p| if step_size > 0.0 { p[1] >= t90_val } else { p[1] <= t90_val })
                .map(|c90| c90[0] - c10[0])
        });

        // Settling time: last time outside ±2% band, once settled
        let band = step_size.abs() * 0.02;
        let n_check = 100.min(samples.len());
        let currently_settled = samples[samples.len() - n_check..]
            .iter()
            .all(|p| (p[1] - step.target).abs() <= band);
        let settling_time = if currently_settled {
            let last_outside = samples
                .iter()
                .rposition(|p| (p[1] - step.target).abs() > band);
            Some(last_outside.map_or(0.0, |i| samples[i][0] - step.time))
        } else {
            None
        };

        Some(StepMetrics {
            steady_state,
            error,
            overshoot_pct,
            rise_time,
            settling_time,
        })
    }

    fn apply_control(&mut self) {
        if self.control_mode != self.prev_control_mode {
            self.motor.reset_pid();
            self.prev_control_mode = self.control_mode;
            self.step_state = None;
            self.prev_target = 0.0;
        }

        match self.control_mode {
            Mode::Voltage => self.motor.set_voltage(self.voltage_input),
            Mode::Pwm => self.motor.set_pwm(self.pwm_input),
            Mode::Position => self.motor.set_position_target(self.pos_target.to_radians()),
            Mode::Velocity => self.motor.set_velocity_target(self.vel_target),
        }

        if matches!(self.control_mode, Mode::Position | Mode::Velocity) {
            let target = if self.control_mode == Mode::Position {
                self.pos_target as f64
            } else {
                self.vel_target as f64
            };
            if (target - self.prev_target).abs() > 3.0 {
                let current_val = if self.control_mode == Mode::Position {
                    self.motor.position().to_degrees() as f64
                } else {
                    self.motor.velocity() as f64
                };
                self.step_state = Some(StepState {
                    time: self.time,
                    initial: current_val,
                    target,
                });
                self.prev_target = target;
            }
        }
        self.motor.set_external_torque(self.load_torque());

        self.motor.link = if self.link_enabled {
            Some(Link {
                mass: self.link_mass,
                length: self.link_length,
                width: self.link_width,
            })
        } else {
            None
        };
    }

    fn simulate_step(&mut self) {
        self.motor.update(self.dt as f32);
        self.time += self.dt;

        let t = self.time;
        let current = self.motor.current() as f64;
        let torque = self.motor.torque() as f64;
        let back_emf = self.motor.back_emf() as f64;
        let power = self.motor.power() as f64;

        self.position_history.push_back([t, self.motor.position().to_degrees() as f64]);
        self.velocity_history.push_back([t, self.motor.velocity() as f64]);
        self.current_history.push_back([t, current]);
        self.temperature_history.push_back([t, self.motor.temperature() as f64]);
        self.torque_history.push_back([t, torque]);
        self.back_emf_history.push_back([t, back_emf]);
        self.power_history.push_back([t, power]);

        let min_t = t - self.time_window;
        trim(&mut self.position_history, min_t);
        trim(&mut self.velocity_history, min_t);
        trim(&mut self.current_history, min_t);
        trim(&mut self.temperature_history, min_t);
        trim(&mut self.torque_history, min_t);
        trim(&mut self.back_emf_history, min_t);
        trim(&mut self.power_history, min_t);
    }

    fn reset(&mut self) {
        self.time = 0.0;
        self.motor.reset_state();
        self.position_history.clear();
        self.velocity_history.clear();
        self.current_history.clear();
        self.temperature_history.clear();
        self.torque_history.clear();
        self.back_emf_history.clear();
        self.power_history.clear();
    }
}

impl eframe::App for GraphApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.apply_control();

        if !self.paused {
            let frame_dt = ctx.input(|i| i.unstable_dt) as f64;
            let steps = ((frame_dt * self.sim_speed / self.dt) as usize).clamp(1, 5000);
            for _ in 0..steps {
                self.simulate_step();
            }
        }

        egui::SidePanel::left("controls").min_width(240.0).show(ctx, |ui| {
            ui.heading("Motor Simulator");
            ui.separator();

            // --- Simulation ---
            ui.label(egui::RichText::new("Simulation").strong());
            ui.horizontal(|ui| {
                let play_label = if self.paused { "▶ Play" } else { "⏸ Pause" };
                if ui.button(play_label).clicked() {
                    self.paused = !self.paused;
                }
                if ui.button("⟲ Reset").clicked() {
                    self.reset();
                }
                if ui.button("⬡ Scene").clicked() {
                    self.scene.open = !self.scene.open;
                }
            });
            ui.add(
                egui::Slider::new(&mut self.sim_speed, 0.1..=10.0)
                    .text("Speed ×")
                    .logarithmic(true),
            );
            ui.add(
                egui::Slider::new(&mut self.time_window, 1.0..=60.0)
                    .text("Window (s)"),
            );
            ui.separator();

            // --- Control ---
            ui.label(egui::RichText::new("Control").strong());
            egui::ComboBox::from_label("Mode")
                .selected_text(match self.control_mode {
                    Mode::Voltage => "Voltage",
                    Mode::Pwm => "PWM",
                    Mode::Position => "Position",
                    Mode::Velocity => "Velocity",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.control_mode, Mode::Voltage, "Voltage");
                    ui.selectable_value(&mut self.control_mode, Mode::Pwm, "PWM");
                    ui.selectable_value(&mut self.control_mode, Mode::Position, "Position");
                    ui.selectable_value(&mut self.control_mode, Mode::Velocity, "Velocity");
                });

            match self.control_mode {
                Mode::Voltage => {
                    ui.add(
                        egui::Slider::new(
                            &mut self.voltage_input,
                            -self.motor.params.max_voltage..=self.motor.params.max_voltage,
                        )
                        .text("Voltage (V)"),
                    );
                }
                Mode::Pwm => {
                    ui.add(
                        egui::Slider::new(&mut self.pwm_input, -1.0..=1.0)
                            .text("PWM Duty"),
                    );
                }
                Mode::Position => {
                    ui.add(
                        egui::Slider::new(&mut self.pos_target, -720.0..=720.0)
                            .text("Target (°)"),
                    );
                }
                Mode::Velocity => {
                    ui.add(
                        egui::Slider::new(&mut self.vel_target, -50.0..=50.0)
                            .text("Target (rad/s)"),
                    );
                }
            }

            ui.horizontal(|ui| {
                ui.radio_value(&mut self.motor.inputs.drive_mode, DriveMode::Normal, "Normal");
                ui.radio_value(&mut self.motor.inputs.drive_mode, DriveMode::Coast, "Coast");
                ui.radio_value(&mut self.motor.inputs.drive_mode, DriveMode::Brake, "Brake");
            });

            // --- PID Gains + Step Response (only in closed-loop modes) ---
            if matches!(self.control_mode, Mode::Position | Mode::Velocity) {
                ui.separator();
                ui.label(egui::RichText::new("PID Gains").strong());
                ui.add(egui::Slider::new(&mut self.motor.pid.kp, 0.0..=20.0).text("Kp"));
                ui.add(egui::Slider::new(&mut self.motor.pid.ki, 0.0..=5.0).text("Ki"));
                ui.add(egui::Slider::new(&mut self.motor.pid.kd, 0.0..=5.0).text("Kd"));
                ui.add(
                    egui::Slider::new(&mut self.motor.pid.integral_limit, 0.1..=50.0)
                        .text("I limit"),
                );

                ui.separator();
                ui.label(egui::RichText::new("Step Response").strong());
                let metrics = self.step_metrics();
                let fmt_opt = |v: Option<f64>, unit: &str| -> String {
                    v.map_or_else(|| "--".to_string(), |x| format!("{x:.3} {unit}"))
                };
                match &metrics {
                    None => {
                        ui.label("Move the target to start analysis.");
                    }
                    Some(m) => {
                        ui.label(format!("Rise time:    {}", fmt_opt(m.rise_time, "s")));
                        ui.label(format!("Settle time:  {}", fmt_opt(m.settling_time, "s")));
                        ui.label(format!("Overshoot:    {:.1} %", m.overshoot_pct));
                        ui.label(format!("Steady-state: {:.4}", m.steady_state));
                        ui.label(format!("Error:        {:.4}", m.error));
                    }
                }
            }

            ui.separator();

            // --- Load ---
            ui.label(egui::RichText::new("Load").strong());
            let load_idx = self.load_mode as usize;
            egui::ComboBox::from_label("Type")
                .selected_text(LOAD_MODES[load_idx])
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.load_mode, LoadMode::None, "None");
                    ui.selectable_value(&mut self.load_mode, LoadMode::Constant, "Constant");
                    ui.selectable_value(&mut self.load_mode, LoadMode::Spring, "Spring");
                    ui.selectable_value(&mut self.load_mode, LoadMode::FanPump, "Fan/Pump");
                    ui.selectable_value(&mut self.load_mode, LoadMode::Pendulum, "Pendulum");
                });

            match self.load_mode {
                LoadMode::None => {}
                LoadMode::Constant => {
                    ui.add(
                        egui::Slider::new(&mut self.load_torque_val, -10.0..=10.0)
                            .text("Torque (Nm)"),
                    );
                }
                LoadMode::Spring => {
                    ui.add(
                        egui::Slider::new(&mut self.spring_k, 0.0..=20.0)
                            .text("Spring k (Nm/rad)"),
                    );
                }
                LoadMode::FanPump => {
                    ui.add(
                        egui::Slider::new(&mut self.drag_k, 0.0..=1.0)
                            .text("Drag k (Nm/(rad/s)²)"),
                    );
                }
                LoadMode::Pendulum => {
                    ui.add(
                        egui::Slider::new(&mut self.pendulum_k, 0.0..=20.0)
                            .text("Gravity k (Nm)"),
                    );
                }
            }

            ui.separator();

            // --- Live Readings ---
            ui.label(egui::RichText::new("Live Readings").strong());
            ui.label(format!("t            = {:.3} s", self.time));
            ui.label(format!("Position     = {:.2} °", self.motor.position().to_degrees()));
            ui.label(format!("Velocity     = {:.4} rad/s", self.motor.velocity()));
            ui.label(format!("Current      = {:.4} A", self.motor.current()));
            ui.label(format!("Temperature  = {:.2} °C", self.motor.temperature()));
            ui.label(format!("Torque       = {:.4} Nm", self.motor.torque()));
            ui.label(format!("Back-EMF     = {:.4} V", self.motor.back_emf()));
            ui.label(format!("Power        = {:.4} W", self.motor.power()));
            ui.label(format!("Load Torque  = {:.4} Nm", self.load_torque()));
            ui.separator();

            // --- Link ---
            ui.label(egui::RichText::new("Link").strong());
            ui.checkbox(&mut self.link_enabled, "Enable Link");
            if self.link_enabled {
                ui.add(
                    egui::Slider::new(&mut self.link_mass, 0.01..=10.0)
                        .text("Mass (kg)")
                        .logarithmic(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.link_length, 0.1..=3.0)
                        .text("Length (m)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.link_width, 0.02..=0.5)
                        .text("Width (m)"),
                );
            }
            ui.separator();

            // --- Motor Parameters ---
            ui.label(egui::RichText::new("Parameters").strong());
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.motor.params.max_voltage, 1.0..=48.0)
                        .text("Max Voltage (V)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.motor.params.max_current, 1.0..=500.0)
                        .text("Max Current (A)"),
                );
                ui.add(
                    egui::Slider::new(&mut self.motor.params.gear_ratio, 1.0..=1000.0)
                        .text("Gear Ratio")
                        .logarithmic(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.motor.params.rotor_inertia, 0.0001..=1.0)
                        .text("Rotor Inertia")
                        .logarithmic(true),
                );
                ui.add(
                    egui::Slider::new(
                        &mut self.motor.params.friction_coefficient,
                        0.0..=2.0,
                    )
                    .text("Friction Coeff"),
                );
                ui.add(
                    egui::Slider::new(&mut self.motor.params.torque_constant, 0.001..=1.0)
                        .text("Kt (Nm/A)")
                        .logarithmic(true),
                );
                ui.add(
                    egui::Slider::new(
                        &mut self.motor.params.back_emf_constant,
                        0.001..=10.0,
                    )
                    .text("Ke (V·s/rad)")
                    .logarithmic(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.motor.params.resistance, 0.01..=10.0)
                        .text("Resistance (Ω)")
                        .logarithmic(true),
                );
                ui.add(
                    egui::Slider::new(&mut self.motor.params.inductance, 0.0001..=0.1)
                        .text("Inductance (H)")
                        .logarithmic(true),
                );
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let plot_height = (ui.available_height() / 4.0) - 6.0;

            let control_mode = self.control_mode;
            let pos_target = self.pos_target as f64;
            let vel_target = self.vel_target as f64;

            let plot_data: [(&str, Vec<[f64; 2]>); 7] = [
                ("Position (°)", to_vec(&self.position_history)),
                ("Velocity (rad/s)", to_vec(&self.velocity_history)),
                ("Current (A)", to_vec(&self.current_history)),
                ("Temperature (°C)", to_vec(&self.temperature_history)),
                ("Torque (Nm)", to_vec(&self.torque_history)),
                ("Back-EMF (V)", to_vec(&self.back_emf_history)),
                ("Power (W)", to_vec(&self.power_history)),
            ];

            let setpoint_style = LineStyle::Dashed { length: 10.0 };

            for row in 0..4usize {
                ui.columns(2, |cols| {
                    for col in 0..2usize {
                        let idx = row * 2 + col;
                        if idx >= plot_data.len() {
                            return;
                        }
                        let (name, ref data) = plot_data[idx];
                        Plot::new(format!("plot_{idx}"))
                            .height(plot_height)
                            .y_axis_label(name)
                            .show(&mut cols[col], |plot_ui| {
                                plot_ui.line(
                                    Line::new(PlotPoints::from(data.clone())).name(name),
                                );
                                if idx == 0 && control_mode == Mode::Position {
                                    plot_ui.hline(
                                        HLine::new(pos_target)
                                            .name("Setpoint")
                                            .color(egui::Color32::from_rgb(220, 80, 80))
                                            .style(setpoint_style),
                                    );
                                } else if idx == 1 && control_mode == Mode::Velocity {
                                    plot_ui.hline(
                                        HLine::new(vel_target)
                                            .name("Setpoint")
                                            .color(egui::Color32::from_rgb(220, 80, 80))
                                            .style(setpoint_style),
                                    );
                                }
                            });
                    }
                });
            }
        });

        self.scene.show(ctx, &self.motor, self.control_mode, self.pos_target.to_radians());

        ctx.request_repaint();
    }
}
