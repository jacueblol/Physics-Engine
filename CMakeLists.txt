cmake_minimum_required(VERSION 3.16)
project(PhysicsEngine LANGUAGES CXX C)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

add_executable(engine
    src/main.cpp
    external/glad/src/glad.c
)

target_include_directories(engine PRIVATE
    external/glad/include
)

find_package(glfw3 REQUIRED)

target_link_libraries(engine PRIVATE glfw)

# Linux extra libs
if(UNIX AND NOT APPLE)
    target_link_libraries(engine PRIVATE dl pthread)
endif()