# AutoVersion.cmake
# ─────────────────────────────────────────────────────────────────────────────
# CMake integration for auto-version.
#
# Usage (after `git submodule add`):
#
#   include(auto-version/cmake/AutoVersion.cmake)
#   auto_version_setup()
#   auto_version_generate()
#   target_compile_definitions(my_app PRIVATE ${AUTO_VERSION_DEFINES})
#
# Requirements:
#   - Cargo/Rust toolchain OR pre-built binary on PATH
# ─────────────────────────────────────────────────────────────────────────────

cmake_minimum_required(VERSION 3.15)

set(AUTO_VERSION_SOURCE_DIR "${CMAKE_CURRENT_LIST_DIR}/.." CACHE PATH
    "Root of the auto-version source tree (submodule path)")

set(AUTO_VERSION_BINARY_DIR "${CMAKE_BINARY_DIR}/_auto-version-build" CACHE PATH
    "Build directory for auto-version binary")

set(AUTO_VERSION_CONFIG "" CACHE FILEPATH
    "Path to auto-version.toml config file (auto-discovered if empty)")

# ─── auto_version_setup ───────────────────────────────────────────────────────
# Compile auto-version from source using Cargo (once, via ExternalProject).
# Sets AUTO_VERSION_EXE to the path of the compiled binary.
function(auto_version_setup)
    find_program(CARGO_EXECUTABLE cargo)

    if(CARGO_EXECUTABLE)
        include(ExternalProject)
        ExternalProject_Add(auto_version_build
            SOURCE_DIR  "${AUTO_VERSION_SOURCE_DIR}"
            BINARY_DIR  "${AUTO_VERSION_BINARY_DIR}"
            CONFIGURE_COMMAND ""
            BUILD_COMMAND
                ${CARGO_EXECUTABLE} build --release
                --manifest-path "${AUTO_VERSION_SOURCE_DIR}/Cargo.toml"
                --target-dir    "${AUTO_VERSION_BINARY_DIR}"
            INSTALL_COMMAND ""
            BUILD_BYPRODUCTS
                "${AUTO_VERSION_BINARY_DIR}/release/auto-version${CMAKE_EXECUTABLE_SUFFIX}"
                "${AUTO_VERSION_BINARY_DIR}/release/auto-version.exe"
            LOG_BUILD ON
        )

        if(WIN32)
            set(AUTO_VERSION_EXE
                "${AUTO_VERSION_BINARY_DIR}/release/auto-version.exe" CACHE FILEPATH "" FORCE)
        else()
            set(AUTO_VERSION_EXE
                "${AUTO_VERSION_BINARY_DIR}/release/auto-version" CACHE FILEPATH "" FORCE)
        endif()
    else()
        # Fallback: look for pre-built binary on PATH
        find_program(AUTO_VERSION_EXE auto-version REQUIRED
            DOC "auto-version binary (install from GitHub Releases or via cargo install auto-version)")
        message(STATUS "auto-version: Cargo not found, using pre-built binary: ${AUTO_VERSION_EXE}")
    endif()

    message(STATUS "auto-version binary: ${AUTO_VERSION_EXE}")
endfunction()

# ─── auto_version_generate ────────────────────────────────────────────────────
# Run auto-version to generate all outputs declared in auto-version.toml.
function(auto_version_generate)
    if(NOT AUTO_VERSION_EXE)
        message(FATAL_ERROR "Call auto_version_setup() before auto_version_generate()")
    endif()

    set(config_arg "")
    if(AUTO_VERSION_CONFIG)
        set(config_arg "--config" "${AUTO_VERSION_CONFIG}")
    endif()

    execute_process(
        COMMAND "${AUTO_VERSION_EXE}" ${config_arg} generate
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
        RESULT_VARIABLE result
        OUTPUT_VARIABLE output
        ERROR_VARIABLE  error_output
    )

    if(NOT result EQUAL 0)
        message(FATAL_ERROR "auto-version generate failed:\n${error_output}")
    endif()
endfunction()

# ─── auto_version_get_vars ────────────────────────────────────────────────────
# Query version variables into CMake scope.
# Sets: AUTO_VERSION_SEMVER, AUTO_VERSION_FULL, AUTO_VERSION_MAJOR,
#       AUTO_VERSION_MINOR, AUTO_VERSION_PATCH, AUTO_VERSION_BRANCH,
#       AUTO_VERSION_SHA, AUTO_VERSION_BUILD_DATE
function(auto_version_get_vars)
    if(NOT AUTO_VERSION_EXE)
        message(FATAL_ERROR "Call auto_version_setup() before auto_version_get_vars()")
    endif()

    set(config_arg "")
    if(AUTO_VERSION_CONFIG)
        set(config_arg "--config" "${AUTO_VERSION_CONFIG}")
    endif()

    # Get kv output
    execute_process(
        COMMAND "${AUTO_VERSION_EXE}" ${config_arg} show --format kv
        WORKING_DIRECTORY "${CMAKE_SOURCE_DIR}"
        OUTPUT_VARIABLE kv_output
        RESULT_VARIABLE result
    )

    if(NOT result EQUAL 0)
        message(WARNING "auto-version show failed; version variables will be empty")
        return()
    endif()

    string(REPLACE "\n" ";" kv_lines "${kv_output}")
    foreach(line IN LISTS kv_lines)
        if(line MATCHES "^([A-Z_]+)=(.*)$")
            set(key   "${CMAKE_MATCH_1}")
            set(value "${CMAKE_MATCH_2}")
            set("AUTO_VERSION_${key}" "${value}" PARENT_SCOPE)
        endif()
    endforeach()
endfunction()

# ─── auto_version_target ──────────────────────────────────────────────────────
# Ensure version generation runs before the given target is built.
function(auto_version_target target)
    if(TARGET auto_version_build)
        add_dependencies("${target}" auto_version_build)
    endif()
endfunction()
