cmake_minimum_required(VERSION 3.16)
project(GravitySim)

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(SFML 3 COMPONENTS Graphics Window System REQUIRED)

include(FetchContent)

FetchContent_Declare(imgui
    GIT_REPOSITORY https://github.com/ocornut/imgui
    GIT_TAG        v1.91.5
)
FetchContent_Declare(imgui-sfml
    GIT_REPOSITORY https://github.com/SFML/imgui-sfml
    GIT_TAG        master
)

# Make imgui available first so imgui_SOURCE_DIR is populated
FetchContent_MakeAvailable(imgui)

# Now imgui_SOURCE_DIR exists — pass it to imgui-sfml
set(IMGUI_DIR ${imgui_SOURCE_DIR} CACHE PATH "" FORCE)
set(BUILD_SHARED_LIBS ON CACHE BOOL "" FORCE)

# Now make imgui-sfml available
FetchContent_MakeAvailable(imgui-sfml)

add_executable(GravitySim
    src/main.cpp
    src/Simulation.cpp
)

target_link_libraries(GravitySim
    SFML::Graphics
    SFML::Window
    SFML::System
    ImGui-SFML::ImGui-SFML
)