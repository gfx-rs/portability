cmake_minimum_required (VERSION 2.6)
project (portability)

include_directories("modules/vulkan-docs/src")
add_executable(native_test native/test.cpp native/window.cpp)

# That's quite a mess, cleanup if possible..
if (WIN32)
    # TODO: can we use `find_library`? It seemed to search for `portability.lib` always..
    target_link_libraries(native_test "../target/debug/portability.dll")
    target_link_libraries(native_test Dwmapi Userenv ws2_32)

    # Copy dll over to build directory
    add_custom_command(TARGET native_test POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        "${PROJECT_SOURCE_DIR}/target/debug/portability.dll"
        $<TARGET_FILE_DIR:native_test>)

else (WIN32)
    find_library(PORTABILITY_LIB portability "target/debug")
    target_link_libraries(native_test ${PORTABILITY_LIB})
    target_link_libraries(native_test pthread dl m X11 xcb)

endif (WIN32)
