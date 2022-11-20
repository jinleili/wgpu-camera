#!/usr/bin/env bash

cargo build --target aarch64-apple-ios
# cargo build --target <aarch64-apple-ios-sim or x86_64-apple-ios>

# Copy libwgpu_camera.a to iOS project
# 
# Why copy?
# On Xcode 14.1, when xxx..dylib file exists in the library search path, Xcode will try to reference it and report an error:
# Dylib (/Users/XXX/wgpu-camera/target/aarch64-apple-ios/debug/libwgpu_camera.dylib) was built for newer iOS version (16.1) than being linked (13.0)

cp target/aarch64-apple-ios/debug/libwgpu_camera.a iOS/libs/debug/libwgpu_camera.a