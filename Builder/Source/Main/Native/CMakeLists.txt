cmake_minimum_required(VERSION 3.22)
project(apk_builder LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

if(NOT DEFINED APK_BUILDER_RUST_ARTIFACT_DIR)
    message(FATAL_ERROR
        "APK_BUILDER_RUST_ARTIFACT_DIR was not supplied. Gradle passes it; invoking "
        "this CMake file directly is not a supported entry point.")
endif()

if(NOT DEFINED APK_BUILDER_RUST_PROFILE)
    set(APK_BUILDER_RUST_PROFILE "release")
endif()

if(ANDROID_ABI STREQUAL "arm64-v8a")
    set(APK_BUILDER_RUST_TARGET "aarch64-linux-android")
elseif(ANDROID_ABI STREQUAL "armeabi-v7a")
    set(APK_BUILDER_RUST_TARGET "armv7-linux-androideabi")
elseif(ANDROID_ABI STREQUAL "x86_64")
    set(APK_BUILDER_RUST_TARGET "x86_64-linux-android")
elseif(ANDROID_ABI STREQUAL "x86")
    set(APK_BUILDER_RUST_TARGET "i686-linux-android")
else()
    message(FATAL_ERROR
        "Unsupported ANDROID_ABI '${ANDROID_ABI}'. Supported: arm64-v8a, "
        "armeabi-v7a, x86_64, x86.")
endif()

set(APK_BUILDER_CORE_ARCHIVE
    "${APK_BUILDER_RUST_ARTIFACT_DIR}/${APK_BUILDER_RUST_TARGET}/${APK_BUILDER_RUST_PROFILE}/libapk_builder_core.a")

if(NOT EXISTS "${APK_BUILDER_CORE_ARCHIVE}")
    message(FATAL_ERROR
        "Apk Builder Core archive not found for ABI '${ANDROID_ABI}'.\n"
        "  Expected: ${APK_BUILDER_CORE_ARCHIVE}\n"
        "  Build it first, for example:\n"
        "    cargo build --release --target ${APK_BUILDER_RUST_TARGET}\n"
        "This build refuses to produce a library without the Core linked in.")
endif()

message(STATUS "Apk Builder Core archive for ${ANDROID_ABI}: ${APK_BUILDER_CORE_ARCHIVE}")

add_library(apk_builder SHARED Builder.cpp)

target_include_directories(apk_builder PRIVATE "${CMAKE_CURRENT_SOURCE_DIR}")

target_compile_options(apk_builder PRIVATE
    -Wall
    -Wextra
    -Werror
    -fvisibility=hidden
    -fstack-protector-strong)

target_link_libraries(apk_builder PRIVATE
    "${APK_BUILDER_CORE_ARCHIVE}"
    log
    dl
    m)

target_link_options(apk_builder PRIVATE
    "-Wl,--exclude-libs,ALL"
    "-Wl,-z,relro"
    "-Wl,-z,now"
    "-Wl,--build-id=sha1")
