## gfx-portability
[![Build Status](https://travis-ci.org/gfx-rs/portability.svg?branch=master)](https://travis-ci.org/gfx-rs/portability)
[![Gitter](https://badges.gitter.im/gfx-rs/portability.svg)](https://gitter.im/gfx-rs/portability)

This is a prototype library implementing [Vulkan Portability Initiative](https://www.khronos.org/blog/khronos-announces-the-vulkan-portability-initiative) using gfx-rs [low-level core](http://gfx-rs.github.io/2017/07/24/low-level.html). See gfx-rs [meta issue](https://github.com/gfx-rs/gfx/issues/1354) for backend limitations and further details.

## Vulkan CTS coverage

| gfx-rs Backend | Total cases | Pass | Fail | Quality warning | Compatibility warning | Not supported | Resource error | Internal error | Timeout | Crash |
| -------- | ---- | ---- | --- | -- | - | ---- | - | - | - | - |
| *Vulkan* | 7723 | 2236 | 66  | 34 | 0 | 5387 | 0 | 0 | 0 | 0 |
| *DX12*   | 3563 | 1243 | 73  | 0  | 0 | 2247 | 0 | 0 | 0 | 0 |
| *Metal*  | 3710 | 1260 | 66  | 0  | 0 | 2384 | 0 | 0 | 0 | 0 |

DX12 and Metal measurement are currently out of date.
  
Currently stopping with:
> (RADV ARCH VULKAN) radv_BeginCommandBuffer: Assertion `pBeginInfo->pInheritanceInfo' failed.

Please visit [our wiki](https://github.com/gfx-rs/portability/wiki/Vulkan-CTS-status) for CTS hookup instructions. Once everything is set, you can generate the new results by calling `make cts` on Unix systems.

## Check out
```
git clone --recursive https://github.com/gfx-rs/portability && cd portability
```

## Build

### Makefile (Unix)
```
make
```

### CMake (Window)
Build the Rust library (portability implementation):

```
cargo build --manifest-path libportability/Cargo.toml --features <vulkan|dx12|metal>
```

Build the native example:

```
mkdir build
cd build
cmake ..
cmake --build . --target native_test
```

## Running Samples

### LunarG (API-Samples)
After building `portability` as shown above, grab a copy from https://github.com/LunarG/VulkanSamples.
Manually override the [`VULKAN_LOADER`](https://github.com/LunarG/VulkanSamples/blob/master/API-Samples/CMakeLists.txt#L189-L194) variable and set it to the portability library.
```
set (VULKAN_LOADER "path/to/portability/library")
```
Then proceed with the normal build instructions.
