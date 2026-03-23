#include "Renderer.h"
#include "Simulation.h"

int main() {
  Renderer renderer;
  Simulation sim;

  // Create Bodies 

  Body sun;
  sun.mass = 1.989e30;
  sun.radius = 20.0;
  sun.color = sf::Color(255, 220, 50);
  sun.pos = {0, 0};
  sun.vel = {0, 0};

  Body earth;
  earth.mass = 5.972e24;
  earth.radius = 8.0;
  earth.color = sf::Color(70, 130, 200);
  earth.pos = {Renderer::AU, 0};
  earth.vel = {0, std::sqrt(Simulation::G * sun.mass / Renderer::AU)};

  Body mars;
  mars.mass = 6.39e23;
  mars.radius = 6.0;
  mars.color = sf::Color(200, 80, 40);
  mars.pos = {1.524 * Renderer::AU, 0};
  mars.vel = {0, std::sqrt(Simulation::G * sun.mass / (1.524 * Renderer::AU))};

  sim.bodies.push_back(sun);
  sim.bodies.push_back(earth);
  sim.bodies.push_back(mars);


  while (renderer.window.isOpen()) {
    while (const auto event = renderer.window.pollEvent()) {
      if (event->is<sf::Event::Closed>())
        renderer.window.close();

      const sf::Event::Resized *isResized = event->getIf<sf::Event::Resized>();
      renderer.handleResized(isResized);
    }

    sim.update();

    renderer.beginFrame();
    for (const auto &b : sim.bodies)
      renderer.drawBody(b);
    renderer.endFrame();
  }
}