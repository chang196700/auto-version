cmake_minimum_required(VERSION 3.15)
project(my-c-app C)

# ── auto-version integration ──────────────────────────────────────────────────
# Assumes auto-version is a git submodule at vendor/auto-version
include(vendor/auto-version/cmake/AutoVersion.cmake)

auto_version_setup()     # Compiles or locates the binary
auto_version_generate()  # Runs version generation (writes include/version.h etc.)
auto_version_get_vars()  # Pulls variables into CMake scope

message(STATUS "Building ${PROJECT_NAME} version ${AUTO_VERSION_SEM_VER}")

# ── Main target ───────────────────────────────────────────────────────────────
add_executable(my-c-app src/main.c)
target_include_directories(my-c-app PRIVATE include)
auto_version_target(my-c-app)  # Ensure version.h exists before building
