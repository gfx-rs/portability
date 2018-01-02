## gfx-portability
[![Build Status](https://travis-ci.org/gfx-rs/portability.svg?branch=master)](https://travis-ci.org/gfx-rs/portability)

This is a prototype static library implementing [Vulkan Portability Initiative](https://www.khronos.org/blog/khronos-announces-the-vulkan-portability-initiative) using gfx-rs [low-level core](http://gfx-rs.github.io/2017/07/24/low-level.html). See gfx-rs [meta issue](https://github.com/gfx-rs/gfx/issues/1354) for backend limitations and further details.

## Build

### Makefile (Unix)
```
make
```

### CMake (Window)
Build the Rust library (portability implementation):

```
cargo build --manifest-path libportability/Cargo.toml --features <vulkan|dx12>
```

Build the native example:

```
mkdir build
cd build
cmake ..
cmake --build . --target native_test
```
