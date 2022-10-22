# GPUImage4

## **iOS**
[中文版文档: 与 iOS App 集成](https://jinleili.github.io/learn-wgpu-zh/integration-and-debugging/ios/)

### Add build target

Since iPhone 5 and later are 64-bit devices, `armv7s-apple-ios` and `armv7-apple-ios` are not required.

```sh
# Add iOS device target
rustup target add aarch64-apple-ios 

# Add iOS simulator target
# Intel CPU Mac
rustup target add x86_64-apple-ios
# M1+ Mac
rustup target add aarch64-apple-ios-sim
```

### Build and run
Build for iOS device and simulator
```sh
# Use Metal backend
cargo build --target aarch64-apple-ios && cargo build --target <aarch64-apple-ios-sim or x86_64-apple-ios>
```
