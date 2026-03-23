// Simulation.cpp
#include "Simulation.h"
#include "Body.h"

// ---- Constructor --------------------------------------------------------

Simulation::Simulation() {}

// ---- Public -------------------------------------------------------------

void Simulation::update() {
  for (int s = 0; s < stepsPerFrame; ++s) {
    // Accumulate gravity forces
    for (auto &a : bodies)
      for (auto &b : bodies)
        if (&a != &b)
          applyGravity(a, b);

    // Step all bodies forward
    for (auto &b : bodies) {
      integrate(b);
      b.trail.push_back(b.pos);
      if (b.trail.size() > MAX_TRAIL)
        b.trail.pop_front();
    }
  }
}

// ---- Private ------------------------------------------------------------

void Simulation::applyGravity(Body &a, const Body &b) {
  Vec2 delta = b.pos - a.pos;
  double dist = delta.length();
  if (dist < 1.0)
    return;
  double force = G * a.mass * b.mass / (dist * dist);
  a.acc += delta.normalized() * (force / a.mass);
}

void Simulation::integrate(Body &b) {
  b.vel += b.acc * dt;
  b.pos += b.vel * dt;
  b.acc = {0, 0};
}