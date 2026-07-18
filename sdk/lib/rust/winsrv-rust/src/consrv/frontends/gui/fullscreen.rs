use winsrv_sys::nt::*;
use winsrv_sys::winsrv::*;
use winsrv_sys::consts::*;

pub unsafe extern "system" fn EnterFullScreen(_GuiData: PGUI_CONSOLE_DATA) -> BOOLEAN {
    let mut DevMode: DEVMODEW = core::mem::zeroed();
    DevMode.dmSize = core::mem::size_of::<DEVMODEW>() as u16;

    DevMode.__bindgen_anon_1.__bindgen_anon_2.dmDisplayFixedOutput = DMDFO_CENTER;
    DevMode.dmPelsWidth = 640;
    DevMode.dmPelsHeight = 480;
    DevMode.dmFields = DM_DISPLAYFIXEDOUTPUT | DM_PELSWIDTH | DM_PELSHEIGHT;

    (ChangeDisplaySettingsW(
        core::ptr::addr_of_mut!(DevMode),
        CDS_FULLSCREEN,
    ) == DISP_CHANGE_SUCCESSFUL as i32) as BOOLEAN
}

pub unsafe extern "system" fn LeaveFullScreen(_GuiData: PGUI_CONSOLE_DATA) {
    ChangeDisplaySettingsW(core::ptr::null_mut(), CDS_RESET);
}

pub unsafe extern "system" fn SwitchFullScreen(GuiData: PGUI_CONSOLE_DATA, FullScreen: BOOL) {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;

    if FullScreen == (*GuiData).GuiInfo.FullScreen {
        return;
    }

    if (*GuiData).GuiInfo.FullScreen == 0 {
        (*GuiData).IsWndMax = IsZoomed((*GuiData).hWindow);
        if (*GuiData).IsWndMax != 0 {
            SendMessageW((*GuiData).hWindow, WM_SYSCOMMAND, SC_RESTORE as WPARAM, 0);
        }

        (*GuiData).WndPl.length = core::mem::size_of::<WINDOWPLACEMENT>() as u32;
        GetWindowPlacement(
            (*GuiData).hWindow,
            core::ptr::addr_of_mut!((*GuiData).WndPl),
        );

        (*GuiData).WndStyle = GetWindowLongPtrW((*GuiData).hWindow, GWL_STYLE);
        (*GuiData).WndStyleEx = GetWindowLongPtrW((*GuiData).hWindow, GWL_EXSTYLE);
    }

    if FullScreen != 0 {
        SendMessageW((*GuiData).hWindow, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0);

        if EnterFullScreen(GuiData) != 0 {
            (*Console).FixedSize = 1u8;
            (*GuiData).GuiInfo.FullScreen = 1;

            (*(*GuiData).ActiveBuffer).OldViewSize = (*(*GuiData).ActiveBuffer).ViewSize;

            SetWindowLongPtrW(
                (*GuiData).hWindow,
                GWL_STYLE,
                (WS_POPUP | WS_CLIPSIBLINGS | WS_CLIPCHILDREN) as LONG_PTR,
            );
            SetWindowLongPtrW(
                (*GuiData).hWindow,
                GWL_EXSTYLE,
                WS_EX_APPWINDOW as LONG_PTR,
            );

            SetWindowPos(
                (*GuiData).hWindow,
                -1isize as HWND,
                0,
                0,
                0,
                0,
                SWP_NOSIZE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );

            SetForegroundWindow((*GuiData).hWindow);

            PostMessageW((*GuiData).hWindow, WM_SYSCOMMAND, SC_MAXIMIZE as WPARAM, 0);
        }

        PostMessageW(
            (*GuiData).hWindow,
            WM_SYSCOMMAND,
            if (*GuiData).GuiInfo.FullScreen != 0 || (*GuiData).IsWndMax != 0 {
                SC_MAXIMIZE as WPARAM
            } else {
                SC_RESTORE as WPARAM
            },
            0,
        );
    } else {
        SendMessageW((*GuiData).hWindow, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0);

        LeaveFullScreen(GuiData);

        (*GuiData).GuiInfo.FullScreen = 0;
        (*Console).FixedSize = 0u8;

        (*(*GuiData).ActiveBuffer).ViewSize = (*(*GuiData).ActiveBuffer).OldViewSize;

        SetWindowLongPtrW((*GuiData).hWindow, GWL_STYLE, (*GuiData).WndStyle);
        SetWindowLongPtrW((*GuiData).hWindow, GWL_EXSTYLE, (*GuiData).WndStyleEx);
        SetWindowPos(
            (*GuiData).hWindow,
            core::ptr::null_mut(),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED,
        );

        SetWindowPlacement(
            (*GuiData).hWindow,
            core::ptr::addr_of_mut!((*GuiData).WndPl),
        );

        PostMessageW(
            (*GuiData).hWindow,
            WM_SYSCOMMAND,
            if (*GuiData).IsWndMax != 0 {
                SC_MAXIMIZE as WPARAM
            } else {
                SC_RESTORE as WPARAM
            },
            0,
        );

        SetForegroundWindow((*GuiData).hWindow);
    }
}

pub unsafe extern "C" fn GuiConsoleSwitchFullScreen(GuiData: PGUI_CONSOLE_DATA) {
    let Console: PCONSRV_CONSOLE = (*GuiData).Console;
    let FullScreen: BOOL;

    if ConDrvValidateConsoleUnsafe(Console as PCONSOLE, CONSOLE_RUNNING, 1u8) == 0 {
        return;
    }

    FullScreen = if (*GuiData).GuiInfo.FullScreen != 0 { 0 } else { 1 };

    SwitchFullScreen(GuiData, FullScreen);

    LeaveCriticalSection(core::ptr::addr_of_mut!((*Console).Lock));
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Console Server DLL
//  * FILE:            win32ss/user/winsrv/consrv/frontends/gui/fullscreen.c
//  * PURPOSE:         GUI Terminal Full-screen Mode
//  * PROGRAMMERS:     Hermes Belusca-Maito (hermes.belusca@sfr.fr)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <consrv.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #include "guiterm.h"
//
// /* FUNCTIONS ******************************************************************/
//
// BOOL
// EnterFullScreen(PGUI_CONSOLE_DATA GuiData)
// {
//     DEVMODEW DevMode;
//
//     ZeroMemory(&DevMode, sizeof(DevMode));
//     DevMode.dmSize = sizeof(DevMode);
//
//     DevMode.dmDisplayFixedOutput = DMDFO_CENTER; // DMDFO_STRETCH // DMDFO_DEFAULT
//     // DevMode.dmDisplayFlags = DMDISPLAYFLAGS_TEXTMODE;
//     DevMode.dmPelsWidth  = 640; // GuiData->ActiveBuffer->ViewSize.X * GuiData->CharWidth;
//     DevMode.dmPelsHeight = 480; // GuiData->ActiveBuffer->ViewSize.Y * GuiData->CharHeight;
//     // DevMode.dmBitsPerPel = 32;
//     DevMode.dmFields     = DM_DISPLAYFIXEDOUTPUT | /* DM_DISPLAYFLAGS | DM_BITSPERPEL | */ DM_PELSWIDTH | DM_PELSHEIGHT;
//
//     return (ChangeDisplaySettingsW(&DevMode, CDS_FULLSCREEN) == DISP_CHANGE_SUCCESSFUL);
// }
//
// VOID
// LeaveFullScreen(PGUI_CONSOLE_DATA GuiData)
// {
//     ChangeDisplaySettingsW(NULL, CDS_RESET);
// }
//
//
// // static VOID
// // GuiConsoleResize(PGUI_CONSOLE_DATA GuiData, WPARAM wParam, LPARAM lParam);
//
// VOID
// SwitchFullScreen(PGUI_CONSOLE_DATA GuiData, BOOL FullScreen)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//
//     /*
//      * See:
//      * http://stackoverflow.com/questions/2382464/win32-full-screen-and-hiding-taskbar
//      * http://stackoverflow.com/questions/3549148/fullscreen-management-with-winapi
//      * https://devblogs.microsoft.com/oldnewthing/20100412-00/?p=14353
//      * https://devblogs.microsoft.com/oldnewthing/20050505-04/?p=35703
//      * http://stackoverflow.com/questions/1400654/how-do-i-put-my-opengl-app-into-fullscreen-mode
//      * https://web.archive.org/web/20180210071518/http://nehe.gamedev.net/tutorial/creating_an_opengl_window_win32/13001/
//      * https://web.archive.org/web/20121001015230/http://www.reocities.com/pcgpe/dibs.html
//      */
//
//     /* If we are already in the given state, just bail out */
//     if (FullScreen == GuiData->GuiInfo.FullScreen) return;
//
//     /* Save the current window state if we are not already full-screen */
//     if (!GuiData->GuiInfo.FullScreen)
//     {
//         GuiData->IsWndMax = IsZoomed(GuiData->hWindow);
//         if (GuiData->IsWndMax)
//             SendMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_RESTORE as WPARAM, 0);
//
//         /* Save its old position and size and show state */
//         GuiData->WndPl.length = sizeof(WINDOWPLACEMENT);
//         GetWindowPlacement(GuiData->hWindow, &GuiData->WndPl);
//
//         /* Save the old window styles */
//         GuiData->WndStyle   = GetWindowLongPtr(GuiData->hWindow, GWL_STYLE  );
//         GuiData->WndStyleEx = GetWindowLongPtr(GuiData->hWindow, GWL_EXSTYLE);
//     }
//
//     if (FullScreen)
//     {
//         SendMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0);
//
//         /* Switch to full screen */
//         if (EnterFullScreen(GuiData))
//         {
//             /* Save the new state */
//             Console->FixedSize = TRUE;
//             GuiData->GuiInfo.FullScreen = TRUE;
//
//             GuiData->ActiveBuffer->OldViewSize = GuiData->ActiveBuffer->ViewSize;
//             // GuiData->ActiveBuffer->OldScreenBufferSize = GuiData->ActiveBuffer->ScreenBufferSize;
//
//             /* Change the window styles */
//             SetWindowLongPtr(GuiData->hWindow, GWL_STYLE,
//                              WS_POPUP | WS_CLIPSIBLINGS | WS_CLIPCHILDREN);
//             SetWindowLongPtr(GuiData->hWindow, GWL_EXSTYLE,
//                              WS_EX_APPWINDOW);
//             // SetWindowPos(GuiData->hWindow, NULL, 0, 0, 0, 0,
//                          // SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER |
//                          // SWP_FRAMECHANGED | SWP_SHOWWINDOW);
//
//             /* Reposition the window to the upper-left corner */
//             SetWindowPos(GuiData->hWindow, HWND_TOPMOST, 0, 0, 0, 0,
//                          SWP_NOSIZE | SWP_FRAMECHANGED | SWP_SHOWWINDOW);
//
//             /* Make it the foreground window */
//             SetForegroundWindow(GuiData->hWindow);
//
//             /* Resize it */
//             // // GuiConsoleResizeWindow(GuiData);
//             // GuiConsoleResize(GuiData, SIZE_RESTORED, MAKELPARAM(640, 480));
//
//             PostMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_MAXIMIZE as WPARAM, 0);
//             // SendMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_MAXIMIZE as WPARAM, 0);
//         }
//
//         PostMessageW(GuiData->hWindow, WM_SYSCOMMAND,
//                      GuiData->GuiInfo.FullScreen || GuiData->IsWndMax
//                         ? SC_MAXIMIZE : SC_RESTORE,
//                      0);
//         // ShowWindowAsync(GuiData->hWindow, SW_RESTORE);
//     }
//     else
//     {
//         SendMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_MINIMIZE as WPARAM, 0);
//
//         /* Restore windowing mode */
//         LeaveFullScreen(GuiData);
//
//         /* Save the new state */
//         GuiData->GuiInfo.FullScreen = FALSE;
//         Console->FixedSize = FALSE;
//
//         /*
//          * Restore possible saved dimensions
//          * of the active screen buffer view.
//          */
//         GuiData->ActiveBuffer->ViewSize = GuiData->ActiveBuffer->OldViewSize;
//         // GuiData->ActiveBuffer->ScreenBufferSize = GuiData->ActiveBuffer->OldScreenBufferSize;
//
//         /* Restore the window styles */
//         SetWindowLongPtr(GuiData->hWindow, GWL_STYLE,
//                          GuiData->WndStyle);
//         SetWindowLongPtr(GuiData->hWindow, GWL_EXSTYLE,
//                          GuiData->WndStyleEx);
//         SetWindowPos(GuiData->hWindow, NULL, 0, 0, 0, 0,
//                      SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER |
//                      SWP_FRAMECHANGED /*| SWP_SHOWWINDOW*/);
//
//
//         /* Restore the window to its original position */
//         SetWindowPlacement(GuiData->hWindow, &GuiData->WndPl);
//
//         PostMessageW(GuiData->hWindow, WM_SYSCOMMAND,
//                      GuiData->IsWndMax ? SC_MAXIMIZE : SC_RESTORE,
//                      0);
//         // ShowWindowAsync(GuiData->hWindow, SW_RESTORE);
//
//         /* Make it the foreground window */
//         SetForegroundWindow(GuiData->hWindow);
//
//         /* Resize it */
//         // GuiConsoleResizeWindow(GuiData);
//
//         // PostMessageW(GuiData->hWindow, WM_SYSCOMMAND, SC_RESTORE as WPARAM, 0);
//         // // ShowWindowAsync(GuiData->hWindow, SW_RESTORE);
//     }
// }
//
// VOID
// GuiConsoleSwitchFullScreen(PGUI_CONSOLE_DATA GuiData)
// {
//     PCONSRV_CONSOLE Console = GuiData->Console;
//     BOOL FullScreen;
//
//     if (!ConDrvValidateConsoleUnsafe((PCONSOLE)Console, CONSOLE_RUNNING, TRUE)) return;
//
//     /* Switch to full-screen or to windowed mode */
//     FullScreen = !GuiData->GuiInfo.FullScreen;
//     DPRINT("GuiConsoleSwitchFullScreen - Switch to %s ...\n",
//             (FullScreen ? "full-screen" : "windowed mode"));
//
//     SwitchFullScreen(GuiData, FullScreen);
//
//     LeaveCriticalSection(&Console->Lock);
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
