cmake_minimum_required (VERSION 2.6)
project (portability)

include_directories("modules/vulkan-docs/src")
add_executable(native_test native/test.cpp native/window.cpp)

find_library(PORTABILITY_LIB portability "target/debug")
target_link_libraries(native_test ${PORTABILITY_LIB})

if (WIN32)
    target_link_libraries(native_test Dwmapi Userenv ws2_32)
else (WIN32)
    target_link_libraries(native_test pthread dl m X11 xcb)
endif (WIN32)
