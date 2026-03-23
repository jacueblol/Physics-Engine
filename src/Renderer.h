#pragma once
#include "Body.h"
#include <SFML/Graphics/CircleShape.hpp>
#include <SFML/Graphics/Color.hpp>
#include <SFML/Graphics/Rect.hpp>
#include <SFML/Graphics/RenderWindow.hpp>
#include <SFML/Graphics/View.hpp>
#include <SFML/System/Vector2.hpp>
#include <cstddef>

class Renderer {
public:
  /**
   * @brief One Astronomical Unit in meters — the real-world distance from
   *        Earth to the Sun (1.496e11 m). Used to set up initial body
   *        positions in main.cpp so coordinates match physical reality.
   */

  static constexpr double AU = 1.496e11;
  /**
   * @brief Pixels per meter. Converts simulation coordinates (meters) into
   *        screen coordinates (pixels) for rendering.
   *
   *        600.0 / AU means "1 AU fits in 600 pixels", so the Earth's orbit
   *        spans ~600px from the Sun. Increase to zoom in, decrease to zoom
   * out. Used in toScreen() as: screen_x = (world_x - camera.x) * zoom +
   * centerX
   */
  static constexpr double zoom = 600.0 / AU;

  sf::RenderWindow window;
  Vec2 camera = {0, 0};

  Renderer()
      : window(sf::VideoMode({1280, 720}), "Gravity Sim", sf::Style::Default) {
    window.setFramerateLimit(60);
  }

  void beginFrame() { window.clear(sf::Color(8, 8, 20)); }

  void endFrame() { window.display(); }

  void drawBody(const Body &b) {
    auto winSize = window.getSize();
    for (size_t i = 1; i < b.trail.size(); ++i) {
      auto p1 = toScreen(b.trail[i - 1], winSize);
      auto p2 = toScreen(b.trail[i], winSize);
      float alpha = (float)i / b.trail.size();
      sf::Color tc = b.color;
      tc.a = (uint8_t)(alpha * 120);
      sf::Vertex line[2] = {sf::Vertex{p1, tc}, sf::Vertex{p2, tc}};
      window.draw(line, 2, sf::PrimitiveType::Lines);
    }

    // circle
    sf::CircleShape circle(b.radius);
    circle.setFillColor(b.color);
    circle.setOrigin({(float)b.radius, (float)b.radius});
    circle.setPosition(toScreen(b.pos, winSize));
    window.draw(circle);
  }

  void handleResized(const sf::Event::Resized *isResized) {
    if (!isResized)
      return;
    const auto size = isResized->size;
    sf::FloatRect visibleArea({0.f, 0.f}, {(float)size.x, (float)size.y});
    window.setView(sf::View(visibleArea));
  }

private:
  sf::Vector2f toScreen(Vec2 pos, sf::Vector2u winSize) const {
    float sx = (float)((pos.x - camera.x) * zoom) + winSize.x / 2.f;
    float sy = (float)((pos.y - camera.y) * zoom) + winSize.y / 2.f;
    return {sx, sy};
  }
};