cmake_minimum_required(VERSION 3.16)
project(GravitySim)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

find_package(SFML 3 COMPONENTS Graphics Window System REQUIRED)

add_executable(GravitySim 
  src/main.cpp
  src/Simulation.cpp
)

target_link_libraries(GravitySim
    SFML::Graphics
    SFML::Window
    SFML::System
)

include(CTest)
enable_testing()

