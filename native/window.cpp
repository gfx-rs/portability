
#include <stdio.h>

#include "window.hpp"

#if defined(_WIN32)
const char *CLASS_NAME = "PortabilityClass";

auto WINAPI window_func(HWND hwnd, UINT u_msg, WPARAM w_param, LPARAM l_param) -> LRESULT {
    if (u_msg == WM_CLOSE) {
        PostQuitMessage(0);
    }
    return DefWindowProc(hwnd, u_msg, w_param, l_param);
}

auto register_class(HINSTANCE hinstance) -> bool {
    WNDCLASS wclass = {0};

    wclass.style = CS_HREDRAW | CS_VREDRAW;
    wclass.lpszClassName = CLASS_NAME;
    wclass.hInstance = hinstance;
    wclass.lpfnWndProc = window_func;
    wclass.hCursor = LoadCursor(NULL, IDC_ARROW);

     if(!RegisterClass(&wclass)) {
        printf("Couldn't register window class");
        return false;
     }

     return true;
}

auto new_window(Config config) -> Window {
    auto hinstance = GetModuleHandle(0);
    register_class(hinstance);

    RECT rect;

    rect.left = config.x; rect.right = config.x + config.width;
    rect.top = config.y;  rect.bottom = config.y + config.height;

    AdjustWindowRectEx(&rect, 0, false, 0);

    auto hwnd = ::CreateWindow(
        CLASS_NAME,
        "GfxPortability",
        WS_THICKFRAME | WS_SYSMENU,
        rect.left,
        rect.top,
        rect.right-rect.left,
        rect.bottom-rect.top,
        NULL,
        NULL,
        ::GetModuleHandle(0),
        NULL
    );

    if(!hwnd) {
        printf("Couldn't create window! error: %d", ::GetLastError());
    }

    ::ShowWindow(hwnd, SW_SHOWDEFAULT);
    ::UpdateWindow(hwnd);

    Window window = { hinstance, hwnd };
    return window;
}

auto poll_events() -> bool {
    MSG msg;
    while(PeekMessage(&msg, NULL, 0, 0, PM_REMOVE)) {
        if (msg.message == WM_QUIT) {
            return false;
        }
        TranslateMessage(&msg);
        DispatchMessage(&msg);
    }
    return true;
}

#endif
