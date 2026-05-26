pub struct Link {
    pub mass: f32,   // kg
    pub length: f32, // m, pivot to tip
    pub width: f32,  // m, for rendering only
}

impl Link {
    /// Moment of inertia of a uniform rod pivoting at one end: I = (1/3) m l²
    pub fn inertia(&self) -> f32 {
        self.mass * self.length * self.length / 3.0
    }

    /// Gravitational torque in the vertical plane, angle measured from horizontal.
    /// T = -m g (l/2) cos(θ)  →  opposes the motor when the arm is near horizontal.
    pub fn gravity_torque(&self, angle: f32) -> f32 {
        -self.mass * 9.81 * (self.length / 2.0) * angle.cos()
    }
}
