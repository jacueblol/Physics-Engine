// Simulation.h
#pragma once
#include "Body.h"
#include <vector>

class Simulation {
public:
  std::vector<Body> bodies;

  static constexpr double G = 6.674e-11;
  static constexpr double dt = 3600.0;     // 1 sim-hour per step
  static constexpr int stepsPerFrame = 24; // 24 sim-hours per frame
  static constexpr size_t MAX_TRAIL = 300;

  Simulation();
  void update();

private:
  void applyGravity(Body &a, const Body &b);
  void integrate(Body &b);
};