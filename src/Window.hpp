#pragma once
#include <cstdio>
#include <GLFW/glfw3.h>

struct Window {
    GLFWwindow* handle = nullptr;

    static void errorCallback(int code, const char* desc) {
        std::fprintf(stderr, "GLFW error %d: %s\n", code, desc);
    }

    static void framebufferSizeCallback(GLFWwindow*, int w, int h) {
        // When you add OpenGL calls, set viewport here:
        // glViewport(0, 0, w, h);
        (void)w; (void)h;
    }

    int init(const char* title, int w = 800, int h = 800) {
        glfwSetErrorCallback(errorCallback);

        // Only set this hint if you specifically want X11 (fine on your setup)
        glfwInitHint(GLFW_PLATFORM, GLFW_PLATFORM_X11);

        if (!glfwInit()) return -1;

        glfwWindowHint(GLFW_FLOATING, GLFW_TRUE);
        // When you move to real rendering, also request a core profile version:
        // glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR, 3);
        // glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR, 3);
        // glfwWindowHint(GLFW_OPENGL_PROFILE, GLFW_OPENGL_CORE_PROFILE);

        handle = glfwCreateWindow(w, h, title, nullptr, nullptr);
        if (!handle) {
            glfwTerminate();
            return -1;
        }

        glfwMakeContextCurrent(handle);
        glfwSwapInterval(1); // vsync

        glfwSetFramebufferSizeCallback(handle, framebufferSizeCallback);
        return 0;
    }

    void update() {
        glfwSwapBuffers(handle);
        glfwPollEvents();
    }

    bool shouldClose() const {
        return glfwWindowShouldClose(handle);
    }

    void terminate() {
        if (handle) glfwDestroyWindow(handle);
        handle = nullptr;
        glfwTerminate();
    }
};