#pragma once

#include <stdint.h>

#if defined(_WIN32)
#include <windows.h>
#else
#include <xcb/xcb.h>
#endif

struct Window {
#if defined(_WIN32)
    HINSTANCE instance;
    HWND window;
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
