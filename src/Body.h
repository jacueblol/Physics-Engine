#pragma once
#include "Vec2.h"
#include <SFML/Graphics/Color.hpp>
#include <deque>

struct Body {
  Vec2 pos, vel, acc;
  double mass, radius;
  sf::Color color;
  std::deque<Vec2> trail;
};