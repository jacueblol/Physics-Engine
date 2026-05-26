pub mod graph;
pub mod link;
pub mod motor;
pub mod scene;

use graph::GraphApp;
use link::Link;
use motor::{Motor, MotorParams};

fn main() -> eframe::Result<()> {
    let params = MotorParams {
        max_voltage: 12.0,
        max_current: 30.0,

        gear_ratio: 10.0,

        rotor_inertia: 0.1,
        friction_coefficient: 0.05,

        torque_constant: 0.1,
        back_emf_constant: 0.1,

        resistance: 0.5,
        inductance: 0.001,
    };

    let mut motor = Motor::new(params);

    motor.link = Some(Link { mass: 0.5, length: 1.0, width: 0.1 });
    motor.set_voltage(12.0);

    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "Motor Simulator",
        options,
        Box::new(|_cc| Box::new(GraphApp::new(motor))),
    )
}
