#pragma once
#include "Object.hpp"

class Disk : public Object {
    private:
        float radius = 1.0f;

    public:
        Disk() = default;

        Disk(const Vec3& pos, float r)
            : Object(pos), radius(r) {}
        
        float getRadius() const { return radius; };
        void setRadius(float r) { radius = r; }
};