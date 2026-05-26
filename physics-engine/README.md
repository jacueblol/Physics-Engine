# DC Motor Simulator

A real-time DC motor simulator written in Rust with an interactive GUI.

## Features

### Motor Model
- DC motor physics: back-EMF, winding resistance, inductance, gear ratio
- Current limiting and thermal simulation (Joule heating + Newton cooling)
- Coast and brake modes
- Semi-implicit Euler integration

### Control Modes
- **Voltage** — direct open-loop voltage command
- **PWM** — duty cycle (-1.0 to 1.0) mapped to voltage
- **Position** — closed-loop position control via PID
- **Velocity** — closed-loop velocity control via PID

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
- 2D scene window animating the load in real time
- Tunable PID gains and motor parameters in the sidebar

## Building

```
cargo run --release
```

## Dependencies

- [eframe](https://github.com/emilk/egui/tree/master/crates/eframe) — native GUI framework
- [egui_plot](https://github.com/emilk/egui_plot) — real-time plotting
