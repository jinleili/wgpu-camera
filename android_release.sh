# build to Android targets
cd gpu-image4
cargo so b --lib --target aarch64-linux-android --release
# cargo so b --lib --target armv7-linux-androideabi --release

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
cp target/aarch64-linux-android/release/libgpu_image4.so "${ARM64}/libgpu_image4.so"
# cp target/armv7-linux-androideabi/release/libgpu_image4.so "${ARMv7a}/libgpu_image4.so"
