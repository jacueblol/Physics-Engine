#pragma once
#include <cmath>

struct Vec2 {
  double x, y;
  Vec2(double x = 0, double y = 0) : x(x), y(y) {}

  Vec2 operator+(const Vec2 &o) const { return {x + o.x, y + o.y}; }
  Vec2 operator-(const Vec2 &o) const { return {x - o.x, y - o.y}; }
  Vec2 operator*(double s) const { return {x * s, y * s}; }
  Vec2 &operator+=(const Vec2 &o) {
    x += o.x;
    y += o.y;
    return *this;
  }

  double length() const { return std::sqrt(x * x + y * y); }
  Vec2 normalized() const {
    double l = length();
    return {x / l, y / l};
  }
};