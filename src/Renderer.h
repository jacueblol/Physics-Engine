#pragma once
#include "Body.h"
#include <SFML/Graphics/CircleShape.hpp>
#include <SFML/Graphics/Color.hpp>
#include <SFML/Graphics/Rect.hpp>
#include <SFML/Graphics/RenderWindow.hpp>
#include <SFML/Graphics/View.hpp>
#include <SFML/System/Vector2.hpp>
#include <SFML/Window/Event.hpp>
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
  double zoom = 600.0 / AU;

  // Multiplier to make bodies visible at solar-system scale.
  // Without this the Sun is ~2.8px and planets are sub-pixel.
  // Zoom in close and bodies grow using their real physical radius.
  double visualBoost = 50.0;

  sf::RenderWindow window;
  Vec2 camera = {0, 0};
  bool dragging = false;
  sf::Vector2i lastMousePos;

  Renderer()
      : window(sf::VideoMode({1280, 720}), "Gravity Sim", sf::Style::Default) {
    window.setFramerateLimit(60);
  }

  void beginFrame() { window.clear(sf::Color(8, 8, 20)); }
  void endFrame() { window.display(); }

  void handleResized(const sf::Event::Resized *e) {
    if (!e)
      return;
    const auto size = e->size;
    sf::FloatRect visibleArea({0.f, 0.f}, {(float)size.x, (float)size.y});
    window.setView(sf::View(visibleArea));
  }

  void handleScroll(const sf::Event::MouseWheelScrolled *e) {
    if (!e)
      return;
    auto winSize = window.getSize();
    float mx = (float)e->position.x;
    float my = (float)e->position.y;

    double worldX = (mx - winSize.x / 2.f) / zoom + camera.x;
    double worldY = (my - winSize.y / 2.f) / zoom + camera.y;

    double factor = (e->delta > 0) ? 1.15 : 1.0 / 1.15;
    zoom *= factor;

    // Shift camera so the point under the cursor stays fixed
    camera.x = worldX - (mx - winSize.x / 2.f) / zoom;
    camera.y = worldY - (my - winSize.y / 2.f) / zoom;
  }

  void handleMousePressed(const sf::Event::MouseButtonPressed *e) {
    if (!e)
      return;
    if (e->button == sf::Mouse::Button::Left) {
      dragging = true;
      lastMousePos = e->position;
    }
  }

  void handleMouseReleased(const sf::Event::MouseButtonReleased *e) {
    if (!e)
      return;
    if (e->button == sf::Mouse::Button::Left)
      dragging = false;
  }

  void handleMouseMoved(const sf::Event::MouseMoved *e) {
    if (!e || !dragging)
      return;
    sf::Vector2i delta = e->position - lastMousePos;
    // Convert pixel delta to world units
    camera.x -= delta.x / zoom;
    camera.y -= delta.y / zoom;
    lastMousePos = e->position;
  }

  void drawBody(const Body &b) {
    auto winSize = window.getSize();

    double boost = (b.name == "Sun") ? 30.f : visualBoost * 3.0;

    float screenRadius = std::max((float)(b.radius*zoom*boost), 2.f);

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
    sf::CircleShape circle(screenRadius);
    circle.setFillColor(b.color);
    circle.setOrigin({screenRadius, screenRadius});
    circle.setPosition(toScreen(b.pos, winSize));
    window.draw(circle);
  }

private:
  sf::Vector2f toScreen(Vec2 pos, sf::Vector2u winSize) const {
    float sx = (float)((pos.x - camera.x) * zoom) + winSize.x / 2.f;
    float sy = (float)((pos.y - camera.y) * zoom) + winSize.y / 2.f;
    return {sx, sy};
  }
};