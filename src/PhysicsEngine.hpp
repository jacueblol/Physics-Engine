#pragma once
#include "Window.hpp"

class PhysicsEngine {
private:
    Window win;

public:
    bool createWindow(const char* title) {
        return win.init(title) == 0;
    }

    void gameLoop() {
        while (!win.shouldClose()) {

            if (glfwGetKey(win.handle, GLFW_KEY_ESCAPE) == GLFW_PRESS) {
                glfwSetWindowShouldClose(win.handle, GLFW_TRUE);
            }

            win.update();
        }
    }

    void closeWindow() {
        win.terminate();
    }
};