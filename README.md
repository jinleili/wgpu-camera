# wgpu-image

```sh
RUSTFLAGS=--cfg=web_sys_unstable_apis cargo run --release --package run-wasm --
```

## Run on iOS
[中文版文档: 与 iOS App 集成](https://jinleili.github.io/learn-wgpu-zh/integration-and-debugging/ios/)

Build for iOS target:

```sh
# Add iOS build target
rustup target add aarch64-apple-ios 
# Add iOS simulator target
# Intel CPU Mac
rustup target add x86_64-apple-ios
# M1+ Mac
rustup target add aarch64-apple-ios-sim

# Build for iOS device and/or simulator
cargo build --target aarch64-apple-ios && cargo build --target <aarch64-apple-ios-sim or x86_64-apple-ios>

```

Then, Open `iOS/GPUImage4.xcodeproj` with Xcode and run on iOS device. 


## Run on Android
[中文版文档: 与 Android App 集成](https://jinleili.github.io/learn-wgpu-zh/integration-and-debugging/android/)

### Set up Android environment

Assuming your computer already has Android Studio installed, go to `Android Studio` > `Tools` > `SDK Manager` > `Android SDK` > `SDK Tools`. Check the following options for installation and click OK. 

- [x] Android SDK Build-Tools
- [x] Android SDK Command-line Tools
- [x] NDK(Side by side)

Then, set two following environment variables:

```sh
export ANDROID_SDK_ROOT=$HOME/Library/Android/sdk
# Replace the NDK version number with the version you installed 
export NDK_HOME=$ANDROID_SDK_ROOT/ndk/23.1.7779620
```

### Build for Android target
```sh
# Add build target
rustup target add aarch64-linux-android armv7-linux-androideabi

# Install cargo so subcommand
cargo install cargo-so

# Build
cargo so b --lib --target aarch64-linux-android --release
cargo so b --lib --target armv7-linux-androideabi --release

# copy .so files to android project jniLibs folder
cp target/aarch64-linux-android/release/libgpu_image4.so android/app/libs/arm64-v8a/libgpu_image4.so
cp target/armv7-linux-androideabi/release/libgpu_image4.so android/app/libs/armeabi-v7a/libgpu_image4.so
```
