/*
 * PROJECT:         ReactOS ARM64 Test GUI Application
 * LICENSE:         GPL-2.0-or-later
 * FILE:            base/applications/testgui/testgui.c
 * PURPOSE:         Minimal Win32 GUI app to validate KeUserModeCallback,
 *                  desktop creation, and framebuffer painting.
 *
 * Creates a window, receives WM_CREATE/WM_PAINT via user-mode callbacks,
 * and paints colored rectangles to the framebuffer.
 */

#include <windows.h>
#include <wine/debug.h>
WINE_DEFAULT_DEBUG_CHANNEL(testgui);

/* Helper: use DbgPrint via ntdll for UART trace output */
__declspec(dllimport) ULONG __cdecl DbgPrint(const char *Format, ...);

#define TESTGUI_CLASS L"TestGuiClass"

static LRESULT CALLBACK
TestWndProc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    switch (uMsg)
    {
        case WM_CREATE:
            DbgPrint("[TESTGUI] WM_CREATE received\n");
            return 0;

        case WM_PAINT:
        {
            PAINTSTRUCT ps;
            HDC hdc;
            RECT rc;
            HBRUSH hBrush;

            DbgPrint("[TESTGUI] WM_PAINT received\n");

            hdc = BeginPaint(hWnd, &ps);
            if (hdc)
            {
                GetClientRect(hWnd, &rc);

                /* Paint a bright blue rectangle as window background */
                hBrush = CreateSolidBrush(RGB(0, 100, 255));
                if (hBrush)
                {
                    FillRect(hdc, &rc, hBrush);
                    DeleteObject(hBrush);
                }

                /* Paint a smaller green rectangle in the center */
                {
                    RECT inner;
                    inner.left   = rc.right / 4;
                    inner.top    = rc.bottom / 4;
                    inner.right  = rc.right * 3 / 4;
                    inner.bottom = rc.bottom * 3 / 4;
                    hBrush = CreateSolidBrush(RGB(0, 200, 0));
                    if (hBrush)
                    {
                        FillRect(hdc, &inner, hBrush);
                        DeleteObject(hBrush);
                    }
                }

                /* Draw text to verify font rendering in client area */
                SetBkMode(hdc, TRANSPARENT);
                SetTextColor(hdc, RGB(255, 255, 255));
                {
                    RECT textRect;
                    textRect.left   = 20;
                    textRect.top    = 20;
                    textRect.right  = rc.right - 20;
                    textRect.bottom = 60;
                    DrawTextW(hdc, L"ReactOS ARM64 - Hello from client area!", -1,
                              &textRect, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
                }

                SetTextColor(hdc, RGB(255, 255, 0));
                TextOutW(hdc, rc.right / 4 + 10, rc.bottom / 4 + 10,
                         L"TextOut works!", 14);

                EndPaint(hWnd, &ps);
            }
            return 0;
        }

        case WM_DESTROY:
            DbgPrint("[TESTGUI] WM_DESTROY received\n");
            PostQuitMessage(0);
            return 0;
    }

    return DefWindowProcW(hWnd, uMsg, wParam, lParam);
}

int WINAPI
wWinMain(HINSTANCE hInstance,
         HINSTANCE hPrevInstance,
         LPWSTR lpCmdLine,
         int nShowCmd)
{
    WNDCLASSEXW wcex;
    HWND hWnd;
    MSG msg;

    UNREFERENCED_PARAMETER(hPrevInstance);
    UNREFERENCED_PARAMETER(lpCmdLine);

    DbgPrint("[TESTGUI] wWinMain entered (hInstance=%p)\n", hInstance);

    /* Register window class */
    ZeroMemory(&wcex, sizeof(wcex));
    wcex.cbSize        = sizeof(wcex);
    wcex.style         = CS_HREDRAW | CS_VREDRAW;
    wcex.lpfnWndProc   = TestWndProc;
    wcex.hInstance     = hInstance;
    wcex.hCursor       = NULL;  /* No cursor resources on ARM64 yet */
    wcex.hbrBackground = (HBRUSH)(COLOR_WINDOW + 1);
    wcex.lpszClassName = TESTGUI_CLASS;

    if (!RegisterClassExW(&wcex))
    {
        DbgPrint("[TESTGUI] RegisterClassExW FAILED (error %lu)\n", GetLastError());
        return 1;
    }
    DbgPrint("[TESTGUI] RegisterClassExW succeeded\n");

    /* Create the window */
    hWnd = CreateWindowExW(
        0,
        TESTGUI_CLASS,
        L"ReactOS ARM64 Test",
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        0, 0,                       /* Position: top-left */
        400, 300,                   /* Size */
        NULL,                       /* No parent */
        NULL,                       /* No menu */
        hInstance,
        NULL);

    if (!hWnd)
    {
        DbgPrint("[TESTGUI] CreateWindowExW FAILED (error %lu)\n", GetLastError());
        return 1;
    }
    DbgPrint("[TESTGUI] CreateWindowExW succeeded (hWnd=%p)\n", hWnd);

    ShowWindow(hWnd, nShowCmd);
    InvalidateRect(hWnd, NULL, TRUE);
    UpdateWindow(hWnd);

    /* Message loop */
    while (GetMessageW(&msg, NULL, 0, 0) > 0)
    {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    DbgPrint("[TESTGUI] Message loop exited (wParam=%lu)\n", (ULONG)msg.wParam);
    return (int)msg.wParam;
}
