#pragma once
#include "Renderer.h"
#include "Simulation.h"
#include <SFML/System/Clock.hpp>
#include <imgui-SFML.h>
#include <imgui.h>
#include <string>

class Gui {
public:
  // Simulation controls — read these in main.cpp
  bool paused = false;
  int stepsPerFrame = 24;
  float visualBoost;
  int maxTrail;

  Gui(sf::RenderWindow &window, const Simulation &sim, Renderer &renderer)
      : visualBoost(renderer.visualBoost), maxTrail((int)sim.maxTrail) {
    (void)ImGui::SFML::Init(window);
    ImGui::GetIO().FontGlobalScale = 2.f;
  }

  ~Gui() { ImGui::SFML::Shutdown(); }

  void processEvent(sf::RenderWindow &window, const sf::Event &event) {
    ImGui::SFML::ProcessEvent(window, event);
  }

  void update(sf::RenderWindow &window, sf::Time dt) {
    ImGui::SFML::Update(window, dt);
  }

  // Call once per frame after sim.update() so values are current
  void render(Simulation &sim, Renderer &renderer) {
    // ---- SIMULATION ----------------------------------------
    ImGui::SetNextWindowPos({16, 16}, ImGuiCond_FirstUseEver);
    ImGui::SetNextWindowSize({300, 0}, ImGuiCond_FirstUseEver);
    ImGui::Begin("Simulation");

    
    if (ImGui::Button(paused ? "Resume" : "Pause"))
      paused = !paused;
    ImGui::SameLine();
    if (ImGui::Button("<< Reverse"))
      sim.timeDirection = -1;
    ImGui::SameLine();
    if (ImGui::Button(">> Forward"))
      sim.timeDirection = 1;
    if (sim.timeDirection == -1)
      ImGui::TextColored({1.f, 0.4f, 0.4f, 1.f}, "Rewinding...");
    
    ImGui::SameLine();
    if (ImGui::Button("Reset Camera")) {
      renderer.camera = {0, 0};
      renderer.zoom = 600.0 / Renderer::AU;
      renderer.followIndex = -1;
    }

    ImGui::Separator();

    // Speed
    ImGui::SliderInt("Steps / frame", &stepsPerFrame, 1, 240);
    float simHoursPerSec =
        (float)(stepsPerFrame * Simulation::dt / 3600.0 * 60);
    ImGui::Text("Sim speed: %.1f days / real-second", simHoursPerSec / 24.f);

    ImGui::Separator();

    // Visual
    ImGui::SliderFloat("Visual boost", &visualBoost, 1.f, 600.f);
    renderer.visualBoost = visualBoost;

    if (ImGui::SliderInt("Trail length", &maxTrail, 0, 2000)) {
      sim.maxTrail = (size_t)maxTrail;
      // Trim existing trails immediately if user dragged down
      for (auto &b : sim.bodies)
        while (b.trail.size() > sim.maxTrail)
          b.trail.pop_front();
    }

    double zoomAUpx = renderer.zoom * Renderer::AU;
    ImGui::Text("Zoom: %.1f px / AU", zoomAUpx);

    ImGui::End();

    // ---- BODIES -------------------------------------------
    ImGui::SetNextWindowPos({16, 220}, ImGuiCond_FirstUseEver);
    ImGui::SetNextWindowSize({300, 0}, ImGuiCond_FirstUseEver);
    ImGui::Begin("Bodies");

    // In the Bodies panel, replace the TreeNode block:
    for (size_t i = 0; i < sim.bodies.size(); ++i) {
      const auto &b = sim.bodies[i];
      ImGui::PushID((int)i);
      const char *label = b.name.empty() ? "(unnamed)" : b.name.c_str();
      if (ImGui::TreeNode(label)) {
        double distAU = b.pos.length() / Renderer::AU;
        double speedKms = b.vel.length() / 1000.0;
        ImGui::Text("Distance from origin: %.3f AU", distAU);
        ImGui::Text("Speed:  %.2f km/s", speedKms);
        ImGui::Text("Mass:   %.3e kg", b.mass);
        ImGui::Text("Radius: %.3e m", b.radius);

        // Follow button — highlights if currently following
        bool isFollowing = (renderer.followIndex == (int)i);
        if (isFollowing)
          ImGui::TextColored({0.4f, 1.f, 0.4f, 1.f}, "Following");
        else if (ImGui::Button("Follow"))
          renderer.followIndex = (int)i;

        if (isFollowing && ImGui::Button("Unfollow"))
          renderer.followIndex = -1;

        ImGui::TreePop();
      }
      ImGui::PopID();
    }
    ImGui::End();

    // ---- PERFORMANCE ------------------------------------------------
    ImGui::SetNextWindowPos({16, 420}, ImGuiCond_FirstUseEver);
    ImGui::SetNextWindowSize({300, 0}, ImGuiCond_FirstUseEver);
    ImGui::Begin("Performance");
    ImGui::Text("FPS: %.1f", ImGui::GetIO().Framerate);
    ImGui::Text("Bodies: %zu", sim.bodies.size());
    ImGui::Separator();
    ImGui::End();

    ImGui::SFML::Render(renderer.window);
  }
};