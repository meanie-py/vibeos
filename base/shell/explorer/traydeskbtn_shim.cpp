/*
 * traydeskbtn_shim.cpp — C linkage bridge for the still-C++ CTrayShowDesktopButton.
 *
 * traywnd.cpp is now Rust (explorer-rust traywnd.rs). CTrayWindow holds a
 * CTrayShowDesktopButton* (obtained from CTrayNotifyWnd via the
 * TNWM_GETSHOWDESKTOPBUTTON message) and calls a handful of its methods + reads
 * two fields. Since traydeskbtn.cpp is still C++, the Rust side treats the
 * button as an opaque pointer and reaches it through these extern "C" wrappers
 * (the appbar-shim pattern in reverse). When traydeskbtn.cpp is translated these
 * shims retire.
 */

#include "precomp.h"

extern "C" BOOL
TrayShowDesktopButton_PtInButton(CTrayShowDesktopButton *p, const POINT *ppt)
{
    return p->PtInButton(const_cast<LPPOINT>(ppt));
}

extern "C" LRESULT
TrayShowDesktopButton_OnLButtonDown(CTrayShowDesktopButton *p, UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL *pbHandled)
{
    return p->OnLButtonDown(uMsg, wParam, lParam, *pbHandled);
}

extern "C" LRESULT
TrayShowDesktopButton_OnLButtonUp(CTrayShowDesktopButton *p, UINT uMsg, WPARAM wParam, LPARAM lParam, BOOL *pbHandled)
{
    return p->OnLButtonUp(uMsg, wParam, lParam, *pbHandled);
}

extern "C" void
TrayShowDesktopButton_Click(CTrayShowDesktopButton *p)
{
    p->Click();
}

extern "C" HWND
TrayShowDesktopButton_GetHWND(CTrayShowDesktopButton *p)
{
    return p->m_hWnd;
}

extern "C" BOOL
TrayShowDesktopButton_IsWindow(CTrayShowDesktopButton *p)
{
    return p->IsWindow();
}

extern "C" BOOL
TrayShowDesktopButton_GetPressed(CTrayShowDesktopButton *p)
{
    return p->m_bPressed;
}
