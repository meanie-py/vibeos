use core::ffi::c_void;
use explorer_sys::*;
use explorer_sys::com::*;
use explorer_sys::externs::*;
use explorer_sys::globals::*;
use winsrv_sys::consts::*;
use winsrv_sys::w;
use winsrv_sys::winsrv::*;

// Dialog procs are CALLBACK (stdcall) — extern "system". DialogBoxW is a C
// macro over DialogBoxParamW (user32 exports only the latter).
#[no_mangle]
pub unsafe extern "system" fn CustomizeNotifyIconsProc(
    hwnd: HWND,
    Message: u32,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match Message {
        WM_INITDIALOG => {
            return 1;
        }
        WM_COMMAND => {
            match (wParam & 0xFFFF) as u32 {
                IDOK => {
                    EndDialog(hwnd, IDOK as INT_PTR);
                }
                IDCANCEL => {
                    EndDialog(hwnd, IDCANCEL as INT_PTR);
                }
                _ => {}
            }
        }
        _ => {
            return 0;
        }
    }
    1
}
#[no_mangle]
pub unsafe extern "C" fn ShowCustomizeNotifyIcons(hInst: HINSTANCE, hExplorer: HWND) {
    const IDD_NOTIFICATIONS_CUSTOMIZE: u16 = 7; // resource.h (raw output stubbed 0)

    DialogBoxParamW(
        hInst,
        MAKEINTRESOURCEW(IDD_NOTIFICATIONS_CUSTOMIZE),
        hExplorer,
        Some(CustomizeNotifyIconsProc),
        0,
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * ReactOS Explorer
//  *
//  * Copyright 2015 Jared Smudde <computerwhiz02@hotmail.com>
//  *
//  * This library is free software; you can redistribute it and/or
//  * modify it under the terms of the GNU Lesser General Public
//  * License as published by the Free Software Foundation; either
//  * version 2.1 of the License, or (at your option) any later version.
//  *
//  * This library is distributed in the hope that it will be useful,
//  * but WITHOUT ANY WARRANTY; without even the implied warranty of
//  * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
//  * Lesser General Public License for more details.
//  *
//  * You should have received a copy of the GNU Lesser General Public
//  * License along with this library; if not, write to the Free Software
//  * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//  */
//
// #include "precomp.h"
//
// INT_PTR __stdcall CustomizeNotifyIconsProc(HWND hwnd, UINT Message, WPARAM wParam, LPARAM lParam)
// {
//     switch(Message)
//     {
//         case WM_INITDIALOG:
//
//         return TRUE;
//         case WM_COMMAND:
//            switch(LOWORD(wParam))
//            {
//                case IDOK:
//                    EndDialog(hwnd, IDOK);
//                break;
//                case IDCANCEL:
//                    EndDialog(hwnd, IDCANCEL);
//                break;
//            }
//        break;
//        default:
//            return FALSE;
//     }
//     return TRUE;
// }
//
// VOID ShowCustomizeNotifyIcons(HINSTANCE hInst, HWND hExplorer)
// {
//     DialogBoxW(hInst, MAKEINTRESOURCEW(IDD_NOTIFICATIONS_CUSTOMIZE), hExplorer, CustomizeNotifyIconsProc);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
