#pragma once

#include <stdint.h>

#ifdef _WIN32
#include <windows.h>
#elif __APPLE__
//TODO
#elif __unix__
#include <xcb/xcb.h>
#else
#error "unknown platform"
#endif

struct Window {
#ifdef _WIN32
    HINSTANCE instance;
    HWND window;
#elif __APPLE__
    void *layer;
#else
    xcb_connection_t *connection;
    xcb_drawable_t window;
#endif
};

struct Config {
    uint32_t x;
    uint32_t y;
    uint32_t width;
    uint32_t height;
};

auto new_window(Config config) -> Window;
auto poll_events() -> bool;
