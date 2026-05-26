# DC Motor Simulator

A real-time DC motor simulator written in Rust with an interactive GUI.

## Features

### Motor Model
- DC motor physics: back-EMF, winding resistance, RL inductance dynamics (exact first-order solution), gear ratio
- Current limiting and thermal simulation (Joule heating + Newton cooling)
- Drive modes: Normal, Coast (open circuit), Brake (shorted terminals — back-EMF braking)
- Semi-implicit Euler integration

### Control Modes
- **Voltage** — direct open-loop voltage command
- **PWM** — duty cycle (-1.0 to 1.0) mapped to voltage
- **Position** — closed-loop position control via PID
- **Velocity** — closed-loop velocity control via PID

### Rigid Link
- Physics-coupled rigid link on the motor output shaft (first step toward a two-link arm)
- Moment of inertia (`m·l²/3`) adds directly to effective inertia at the output shaft
- Gravitational torque (`-m·g·(l/2)·cos(θ)`) applied automatically — motor fights gravity to hold position
- Tunable mass, length, and width; toggle on/off at runtime

### Load Modes
- None
- Constant torque
- Spring (linear restoring torque)
- Fan/Pump (quadratic drag)
- Pendulum (gravitational restoring torque)

### Visualizer
- Live plots: position, velocity, current, torque, back-EMF, power, temperature
- Scrolling time window with adjustable simulation speed
- Step response metrics: rise time, settling time, overshoot, steady-state error
- 2D scene window: link rendered as a filled rotating rectangle with pivot joint
- Tunable PID gains, motor parameters, and link properties in the sidebar

## Building

```
cargo run --release
```

## Dependencies

- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) — native GUI framework
- [egui_plot](https://github.com/emilk/egui_plot) — real-time plotting
