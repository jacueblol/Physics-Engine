pub enum ControlMode {
    Voltage(f32),
    Pwm(f32),
    Position(f32),
    Velocity(f32),
}

pub struct MotorInputs {
    mode: ControlMode,
    pub coast_mode: bool,
}

pub struct MotorState {
    current: f32,
    position: f32,
    velocity: f32,
    temperature: f32,
}

pub struct MotorMeasurements {
    pub position: f32,
    pub velocity: f32,
    pub current: f32,
}

pub struct MotorParams {
    pub max_voltage: f32,
    pub max_current: f32,

    pub gear_ratio: f32,

    pub rotor_inertia: f32,
    pub friction_coefficient: f32,

    pub torque_constant: f32,
    pub back_emf_constant: f32,

    pub resistance: f32,
    pub inductance: f32,
}

pub struct PidParams {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub integral_limit: f32,
}

struct PidState {
    integral: f32,
    prev_error: f32,
}

pub struct Motor {
    pub inputs: MotorInputs,
    state: MotorState,
    pub params: MotorParams,
    pub pid: PidParams,
    pid_state: PidState,
    pid_voltage: f32,

    external_torque: f32,
}

impl Motor {
    pub fn new(params: MotorParams) -> Self {
        Self {
            inputs: MotorInputs {
                mode: ControlMode::Voltage(0.0),
                coast_mode: false,
            },

            state: MotorState {
                current: 0.0,
                position: 0.0,
                velocity: 0.0,
                temperature: 25.0,
            },

            params,

            pid: PidParams {
                kp: 2.0,
                ki: 0.2,
                kd: 0.1,
                integral_limit: 10.0,
            },
            pid_state: PidState {
                integral: 0.0,
                prev_error: 0.0,
            },
            pid_voltage: 0.0,

            external_torque: 0.0,
        }
    }

    pub fn applied_voltage(&self) -> f32 {
        match self.inputs.mode {
            ControlMode::Voltage(v) => v.clamp(-self.params.max_voltage, self.params.max_voltage),

            ControlMode::Pwm(duty) => duty.clamp(-1.0, 1.0) * self.params.max_voltage,

            ControlMode::Position(_) | ControlMode::Velocity(_) => {
                self.pid_voltage.clamp(-self.params.max_voltage, self.params.max_voltage)
            }
        }
    }

    fn compute_pid(&mut self, dt: f32) -> f32 {
        let error = match self.inputs.mode {
            ControlMode::Position(target) => target - self.state.position,
            ControlMode::Velocity(target) => target - self.state.velocity,
            _ => return 0.0,
        };

        let p = self.pid.kp * error;

        self.pid_state.integral = (self.pid_state.integral + error * dt)
            .clamp(-self.pid.integral_limit, self.pid.integral_limit);
        let i = self.pid.ki * self.pid_state.integral;

        let d = if dt > 0.0 {
            self.pid.kd * (error - self.pid_state.prev_error) / dt
        } else {
            0.0
        };
        self.pid_state.prev_error = error;

        p + i + d
    }

    pub fn update(&mut self, dt: f32) {
        self.pid_voltage = self.compute_pid(dt);

        let voltage = self.applied_voltage();

        // Rotor angular velocity = output shaft velocity × gear ratio
        let rotor_velocity = self.state.velocity * self.params.gear_ratio;

        // Back-EMF based on rotor speed
        let back_emf = self.params.back_emf_constant * rotor_velocity;

        // Coast mode = open circuit: no current flows
        let current = if self.inputs.coast_mode {
            0.0
        } else {
            ((voltage - back_emf) / self.params.resistance)
                .clamp(-self.params.max_current, self.params.max_current)
        };

        // Motor torque reflected to output shaft: T_out = Kt × I × N
        let motor_torque = self.params.torque_constant * current * self.params.gear_ratio;

        // Viscous friction at output shaft
        let friction_torque = self.params.friction_coefficient * self.state.velocity;

        // Net torque; effective inertia at output shaft = J_rotor × N²
        let net_torque = motor_torque - friction_torque - self.external_torque;
        let effective_inertia = self.params.rotor_inertia * self.params.gear_ratio.powi(2);

        let angular_acceleration = net_torque / effective_inertia;

        // Semi-implicit Euler integration
        self.state.velocity += angular_acceleration * dt;
        self.state.position += self.state.velocity * dt;

        self.state.current = current;

        // Joule heating (I²R) with Newton's law of cooling toward 25°C ambient
        const THERMAL_FACTOR: f32 = 0.01;
        const COOLING_COEFF: f32 = 0.04;
        let heat_in = current * current * self.params.resistance;
        let heat_out = (self.state.temperature - 25.0) * COOLING_COEFF;
        self.state.temperature += (heat_in - heat_out) * THERMAL_FACTOR * dt;
    }

    // -------------------------
    // Control setters
    // -------------------------

    pub fn set_voltage(&mut self, voltage: f32) {
        self.inputs.mode = ControlMode::Voltage(voltage);
    }

    pub fn set_pwm(&mut self, duty: f32) {
        self.inputs.mode = ControlMode::Pwm(duty);
    }

    pub fn set_position_target(&mut self, target_position: f32) {
        self.inputs.mode = ControlMode::Position(target_position);
    }

    pub fn set_velocity_target(&mut self, target_velocity: f32) {
        self.inputs.mode = ControlMode::Velocity(target_velocity);
    }

    pub fn set_coast_mode(&mut self, coast_mode: bool) {
        self.inputs.coast_mode = coast_mode;
    }

    pub fn set_external_torque(&mut self, torque: f32) {
        self.external_torque = torque;
    }

    // -------------------------
    // Measurements / getters
    // -------------------------

    pub fn measurements(&self) -> MotorMeasurements {
        MotorMeasurements {
            position: self.state.position,
            velocity: self.state.velocity,
            current: self.state.current,
        }
    }

    pub fn position(&self) -> f32 {
        self.state.position
    }

    pub fn velocity(&self) -> f32 {
        self.state.velocity
    }

    pub fn current(&self) -> f32 {
        self.state.current
    }

    pub fn temperature(&self) -> f32 {
        self.state.temperature
    }

    pub fn reset_pid(&mut self) {
        self.pid_state = PidState {
            integral: 0.0,
            prev_error: 0.0,
        };
        self.pid_voltage = 0.0;
    }

    pub fn reset_state(&mut self) {
        self.state = MotorState {
            current: 0.0,
            position: 0.0,
            velocity: 0.0,
            temperature: 25.0,
        };
        self.reset_pid();
    }
}
