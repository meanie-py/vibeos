// rustconv-assembly: whole-tu
use crate::prelude::*;
use core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn InpLocalePageProc(
    hwndDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {}
        _ => {}
    }
    FALSE as INT_PTR
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS
//  *  Copyright (C) 2004 ReactOS Team
//  *
//  *  This program is free software; you can redistribute it and/or modify
//  *  it under the terms of the GNU General Public License as published by
//  *  the Free Software Foundation; either version 2 of the License, or
//  *  (at your option) any later version.
//  *
//  *  This program is distributed in the hope that it will be useful,
//  *  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  *  GNU General Public License for more details.
//  *
//  *  You should have received a copy of the GNU General Public License along
//  *  with this program; if not, write to the Free Software Foundation, Inc.,
//  *  51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.
//  */
// /*
//  * PROJECT:         ReactOS International Control Panel
//  * FILE:            dll/cpl/intl/inplocale.c
//  * PURPOSE:         Input Locale property page
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "intl.h"
//
// /* Property page dialog callback */
// INT_PTR __stdcall
// InpLocalePageProc(HWND hwndDlg,
//                   UINT uMsg,
//                   WPARAM wParam,
//                   LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//             break;
//     }
//     return FALSE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
