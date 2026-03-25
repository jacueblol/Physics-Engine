// Simulation.h
#pragma once
#include "Body.h"
#include <cstddef>
#include <vector>

class Simulation {
public:
  std::vector<Body> bodies;

  static constexpr double G = 6.674e-11;
  static constexpr double dt = 3600.0;     // 1 sim-hour per step
  size_t maxTrail = 500; 

  Simulation();
  void update(int stepsPerFrame);

private:
  void applyGravity(Body &a, const Body &b);
  void integrate(Body &b);
};