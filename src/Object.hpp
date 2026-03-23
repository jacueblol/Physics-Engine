#pragma once

struct Vec3 {
    float x;
    float y;
    float z;
};

class Object {
    protected:
        Vec3 position;

    public:
        Object() = default;
        explicit Object(const Vec3& pos) : position(pos) {}

        virtual ~Object() = default;

        Vec3 getPos() const { return position; }
        void setPos(const Vec3& pos) { position = pos; }
};