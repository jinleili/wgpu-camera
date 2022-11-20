#!/usr/bin/env bash

# build to Android target
cd wgpu-camera
RUST_BACKTRACE=full RUST_LOG=wgpu_hal=debug cargo so b --lib --target aarch64-linux-android
# RUST_LOG=wgpu_hal=debug cargo so b --features angle --lib --target armv7-linux-androideabi
# RUST_BACKTRACE=full RUST_LOG=wgpu_hal=debug cargo so b --lib --target aarch64-linux-android 
# RUST_BACKTRACE=full RUST_LOG=wgpu_hal=debug cargo so b --lib --target armv7-linux-androideabi

# copy .so files to jniLibs folder
cd ../
ARM64="Android/app/libs/arm64-v8a"
ARMv7a="Android/app/libs/armeabi-v7a"

if [ ! -d "$ARM64" ]; then
    mkdir "$ARM64"
fi
if [ ! -d "$ARMv7a" ]; then
    mkdir "$ARMv7a"
fi

cp target/aarch64-linux-android/debug/libwgpu_camera.so "${ARM64}/libwgpu_camera.so"
# cp target/armv7-linux-androideabi/debug/libwgpu_camera.so "${ARMv7a}/libwgpu_camera.so"
