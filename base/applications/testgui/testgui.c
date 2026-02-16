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

/* Buffer for typed characters */
static WCHAR g_TypedText[256];
static int g_TypedLen = 0;

/* Child EDIT control handle */
static HWND g_hEdit = NULL;

static LRESULT CALLBACK
TestWndProc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
    switch (uMsg)
    {
        case WM_CREATE:
            DbgPrint("[TESTGUI] WM_CREATE received\n");
            /* Create child EDIT control to test edit control rendering */
            g_hEdit = CreateWindowExW(
                0, L"EDIT", L"Edit control test text",
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_AUTOHSCROLL,
                10, 120,    /* x, y */
                380, 24,    /* width, height */
                hWnd, (HMENU)100,
                ((LPCREATESTRUCT)lParam)->hInstance, NULL);
            if (g_hEdit)
                DbgPrint("[TESTGUI] EDIT control created (hWnd=%p)\n", g_hEdit);
            else
                DbgPrint("[TESTGUI] EDIT control creation FAILED (error %lu)\n", GetLastError());
            return 0;

        case WM_KEYDOWN:
            DbgPrint("[TESTGUI] WM_KEYDOWN vk=0x%02X scan=0x%02X\n",
                     (UINT)wParam, (UINT)((lParam >> 16) & 0xFF));
            /* F1 = toggle focus to EDIT control */
            if (wParam == VK_F1 && g_hEdit)
            {
                SetFocus(g_hEdit);
                DbgPrint("[TESTGUI] Focus switched to EDIT control\n");
            }
            return 0;

        case WM_CHAR:
        {
            WCHAR ch = (WCHAR)wParam;
            DbgPrint("[TESTGUI] WM_CHAR char=0x%04X '%c'\n",
                     (UINT)ch, (ch >= 0x20 && ch < 0x7F) ? (char)ch : '?');
            if (ch == 8 && g_TypedLen > 0) /* Backspace */
                g_TypedLen--;
            else if (ch >= 0x20 && g_TypedLen < 255)
                g_TypedText[g_TypedLen++] = ch;
            g_TypedText[g_TypedLen] = 0;
            InvalidateRect(hWnd, NULL, TRUE);
            return 0;
        }

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

                /* Display typed text (keyboard input test) */
                if (g_TypedLen > 0)
                {
                    RECT typedRect;
                    typedRect.left   = 20;
                    typedRect.top    = 70;
                    typedRect.right  = rc.right - 20;
                    typedRect.bottom = 110;
                    SetTextColor(hdc, RGB(255, 200, 0));
                    DrawTextW(hdc, L"Typed: ", -1,
                              &typedRect, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
                    typedRect.left = 100;
                    DrawTextW(hdc, g_TypedText, g_TypedLen,
                              &typedRect, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
                }
                else
                {
                    RECT typedRect;
                    typedRect.left   = 20;
                    typedRect.top    = 70;
                    typedRect.right  = rc.right - 20;
                    typedRect.bottom = 110;
                    SetTextColor(hdc, RGB(180, 180, 180));
                    DrawTextW(hdc, L"(Type to test keyboard input)", -1,
                              &typedRect, DT_LEFT | DT_SINGLELINE | DT_VCENTER);
                }

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
    SetForegroundWindow(hWnd);
    SetFocus(hWnd);
    InvalidateRect(hWnd, NULL, TRUE);
    UpdateWindow(hWnd);
    DbgPrint("[TESTGUI] Window shown, focus set, entering message loop\n");

    /* Message loop */
    while (GetMessageW(&msg, NULL, 0, 0) > 0)
    {
        TranslateMessage(&msg);
        DispatchMessageW(&msg);
    }

    DbgPrint("[TESTGUI] Message loop exited (wParam=%lu)\n", (ULONG)msg.wParam);
    return (int)msg.wParam;
}
