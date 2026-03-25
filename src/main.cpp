#include "Gui.h"
#include "Renderer.h"
#include "Simulation.h"
#include <SFML/System/Clock.hpp>
#include <SFML/System/Time.hpp>
#include <imgui.h>

int main() {
  Renderer renderer = Renderer();
  Simulation sim;
  Gui gui(renderer.window, sim, renderer);

  // Create Bodies

  auto makePlanet = [&](std::string name, double massKg, double radiusM, double orbitAU,
                        sf::Color color, double centralMass) -> Body {
    Body b;
    b.name = name; 
    b.mass = massKg;
    b.radius = radiusM;
    b.color = color;
    b.pos = {orbitAU * Renderer::AU, 0};
    b.vel = {0,
             std::sqrt(Simulation::G * centralMass / (orbitAU * Renderer::AU))};
    return b;
  };

  Body sun;
  sun.name = "Sun";
  sun.mass = 1.989e30;  // kg
  sun.radius = 6.957e8; // m
  sun.color = sf::Color(255, 220, 50);
  sun.pos = {0, 0};
  sun.vel = {0, 0};

  auto mercury =
      makePlanet("mercury", 3.285e23, 2.440e6, 0.387, sf::Color(169, 169, 169), sun.mass);
  auto venus =
      makePlanet("venus", 4.867e24, 6.051e6, 0.723, sf::Color(230, 185, 100), sun.mass);
  auto earth =
      makePlanet("earth", 5.972e24, 6.371e6, 1.000, sf::Color(70, 130, 200), sun.mass);
  auto mars =
      makePlanet("mars", 6.390e23, 3.389e6, 1.524, sf::Color(200, 80, 40), sun.mass);

  sim.bodies = {sun, mercury, venus, earth, mars};

  sf::Clock clock;

  while (renderer.window.isOpen()) {
    while (const auto event = renderer.window.pollEvent()) {
      gui.processEvent(renderer.window, *event);

      if (event->is<sf::Event::Closed>())
        renderer.window.close();

      if (!ImGui::GetIO().WantCaptureMouse) {
        renderer.handleResized(event->getIf<sf::Event::Resized>());
        renderer.handleScroll(event->getIf<sf::Event::MouseWheelScrolled>());
        renderer.handleMousePressed(
            event->getIf<sf::Event::MouseButtonPressed>());
        renderer.handleMouseReleased(
            event->getIf<sf::Event::MouseButtonReleased>());
        renderer.handleMouseMoved(event->getIf<sf::Event::MouseMoved>());
      }
    }

    sf::Time dt = clock.restart();
    gui.update(renderer.window, dt);

    if (!gui.paused)
      sim.update(gui.stepsPerFrame);
    
    renderer.beginFrame();
    for (const auto &b : sim.bodies)
      renderer.drawBody(b);
    gui.render(sim, renderer);
    renderer.endFrame();
  }
}