## gfx-portability
[![Build Status](https://travis-ci.org/gfx-rs/portability.svg?branch=master)](https://travis-ci.org/gfx-rs/portability)
[![Gitter](https://badges.gitter.im/gfx-rs/portability.svg)](https://gitter.im/gfx-rs/portability)

This is a prototype library implementing [Vulkan Portability Initiative](https://www.khronos.org/blog/khronos-announces-the-vulkan-portability-initiative) using gfx-rs [low-level core](http://gfx-rs.github.io/2017/07/24/low-level.html). See gfx-rs [meta issue](https://github.com/gfx-rs/gfx/issues/1354) for backend limitations and further details.

## Vulkan CTS coverage

| gfx-rs Backend | Total cases | Pass | Fail | Quality warning | Compatibility warning | Not supported | Resource error | Internal error | Timeout | Crash |
| -------- | ---- | ---- | --- | -- | - | ---- | - | - | - | - |
| *Vulkan* | 7759 | 2155 | 131 | 34 | 0 | 5439 | 0 | 0 | 0 | 0 |
| *DX12*   | 3576 | 1258 | 70  | 0  | 0 | 2248 | 0 | 0 | 0 | 0 |
| *Metal*  | 7687 | 2072 | 112 | 39 | 0 | 5464 | 0 | 0 | 0 | 0 |

Current blockers:
- *Vulkan*: "api.command_buffers.render_pass_continue" (secondary render passes).
- *DX12*: lack of `VkBufferView` implementation.
- *Metal*: "api.buffer_view.access.suballocation.buffer_view_memory_test_complete" (missing R32Uint support).


Please visit [our wiki](https://github.com/gfx-rs/portability/wiki/Vulkan-CTS-status) for CTS hookup instructions. Once everything is set, you can generate the new results by calling `make cts` on Unix systems. When investigating a particular failure, it's handy to do `make cts debug=<test_name>`, which runs a single test under system debugger (gdb/lldb). For simply inspecting the log output, one can also do `make cts pick=<test_name>`.

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

### LunarG (API-Samples) (Non-Linux)
After building `portability` as shown above, grab a copy from https://github.com/LunarG/VulkanSamples.
Manually override the [`VULKAN_LOADER`](https://github.com/LunarG/VulkanSamples/blob/master/API-Samples/CMakeLists.txt#L189-L194) variable and set it to the portability library.
```
set (VULKAN_LOADER "path/to/portability/library")
```
Then proceed with the normal build instructions.

### LunarG (API-Samples) (and other applications) (Linux)
1. After building `portability` as shown above, grab a copy from https://github.com/LunarG/VulkanSamples.

2. Then proceed with the normal build instructions.

3. Before running the application(s), additional steps must be taken to aid:
    1. `portability` in selecting the correct Vulkan implementation; and
    2. the application in selecting `portability`.


4. After completing the above mentioned steps you may run the LunarG samples.

#### i. Aiding `portability` in selecting the correct Vulkan implementation (Linux)
`portability`, to avoid calling it self, uses `RTLD_NEXT` to find the 
`vkGetInstanceProcAddr` symbol. For more information relating to `RTLD_NEXT`,
please refer to `dlsym`'s man page.

The simplest way to get `portability` to select your system's (or any other) 
Vulkan implementation is to add your system's (or other's) `libvulkan.so.1` to
`LD_PRELOAD`. This places your selected `libvulkan.so.1` into the front of the 
default library search order. You can examine this order by using `ldd`.

Since we're using `RTLD_NEXT`, not `RTLD_DEFAULT`, our library needs to be 
earlier (later?) in the search order. The easiest way is to just preload our 
library after the systems. *

The easiest way to locate your system's library is to execute 
`whereis libvulkan.so.1`.

I, for example, execute this before running the LunarG examples:
```
export LD_PRELOAD=/usr/lib/libvulkan.so.1:/home/gentz/Documents/gfx/portability/target/release/libvulkan.so.1
```

Alternatively, you can prepend it before whatever application you wish to use 
`portability` with, for example:
```
LD_PRELOAD=/usr/lib/libvulkan.so.1:/home/gentz/Documents/gfx/portability/target/release/libvulkan.so.1 vulkaninfo
```

* Our library should always be earlier on the search order. We want applications
which dynamically link to `libvulkan.so.1` to always choose our library above 
theirs. Most games and applications, however, use `dlopen` and `dlsym`, so in 
most cases your applications would've worked had we used `RTLD_DEFAULT` & not 
preloaded `portability`.

#### ii. Aiding the application to in selecting `portability` (Linux)
To force your application to use our `libvulkan.so.1`, you need to add 
either `$(pwd)/target/release` or `$(pwd)/target/debug` to `LD_LIBRARY_PATH` 
(assuming `pwd` is `portability`'s root directory). This makes it so that when
your application attempt to call `dlopen` on `libvulkan.so.1` it opens 
`portability`'s `libvulkan.so.1`.

I, for example, execute this before running the LunarG examples:
```
export LD_LIBRARY_PATH=/home/gentz/Documents/gfx/portability/target/release
```

Alternatively, you can prepend it before whatever application you wish to use 
`portability` with, for example:
```
LD_LIBRARY_PATH=/home/gentz/Documents/gfx/portability/target/release vulkaninfo
```
