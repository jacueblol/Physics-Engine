#pragma once
#include <cstdio>
#include "PhysicsEngine.hpp"

class PhysicsEngineLauncher {
private:
    PhysicsEngine engine;

public:
    void launch() {
        if (!engine.createWindow("Physics Engine")) {
            std::fprintf(stderr, "Failed to create window\n");
            return;
        }

        engine.gameLoop();
        engine.closeWindow();
    }
};