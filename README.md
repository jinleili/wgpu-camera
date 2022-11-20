# wgpu-camera

## Run on iOS

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
sh ./ios_debug.sh
```
Then, Open `iOS/WGPUCamera.xcodeproj` with Xcode and run on iOS device. 


## Run on Android

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
sh ./android_debug.sh
```
