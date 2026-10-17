use crate::prelude::*;
use core::ffi::c_void;

extern "system" {
    fn lstrlenW(lpString: LPCWSTR) -> i32; // kernel32 (stdcall)
}
extern "C" {
    // CRT (cdecl) — count-bounded wide vsprintf; args is the forwarded va_list.
    fn _vsnwprintf(buffer: *mut u16, count: usize, format: *const u16, argptr: va_list) -> i32;
}

// L"%d"
static FMT_PERCENT_D: [u16; 3] = [
    b'%' as u16, b'd' as u16, 0,
];

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
// misc.c:35. `#[no_mangle]`: profile.rs and the bindgen island read it through an `extern`, and
// this is the one definition they link to (it was C storage in userenv_rust_shims.c until
// 2026-09-25, and this copy was a private twin nothing else saw).
#[no_mangle]
pub static mut LocalSystemAuthority: SID_IDENTIFIER_AUTHORITY = _SID_IDENTIFIER_AUTHORITY {
    Value: [
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        5 as ::core::ffi::c_int as BYTE,
    ],
};

pub static mut WorldAuthority: SID_IDENTIFIER_AUTHORITY = _SID_IDENTIFIER_AUTHORITY {
    Value: [
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        0 as ::core::ffi::c_int as BYTE,
        1 as ::core::ffi::c_int as BYTE,
    ],
};

static mut ErrorDlg: TIMERDLG_INFO = _TIMERDLG_INFO {
    ulTimeout: 0 as ULONG,
    wDefBtnOnTimeout: 0 as WORD,
    wDefBtnOnCancel: 1 as WORD,
    ButtonMap: [
        _TIMERDLG_BUTTON {
            uID: IDOK as UINT,
            nResult: TRUE as INT_PTR,
        },
        _TIMERDLG_BUTTON {
            uID: IDCANCEL as UINT,
            nResult: FALSE as INT_PTR,
        },
    ],
};

// DYN_MODULE has a C flexible-array member `Functions[]` -> bindgen `__IncompleteArrayField`
// (zero-sized), which cannot hold inline data in a static. This layout-compatible local
// struct gives the 4 entries real storage; it is passed to the accessors as PDYN_MODULE
// (the bindgen view reaches `Functions` via `__IncompleteArrayField::as_ptr`, i.e. the byte
// right after `Library` -> our fixed array's start).
#[repr(C)]
pub struct DynOle32Storage {
    pub Library: LPWSTR,
    pub Functions: [LPSTR; 4],
}
// #[no_mangle] so the `_DynOle32` symbol the bindgen `extern static` import (used by desktop.rs)
// references resolves to this definition. DynOle32Storage is layout-compatible with DYN_MODULE.
#[no_mangle]
pub static mut DynOle32: DynOle32Storage = DynOle32Storage {
    Library: win32macros_rust::wide_bytes!(b"o\0l\0e\x003\x002\0.\0d\0l\0l\0\0\0") as LPWSTR,
    Functions: [
        b"CoInitialize\0" as *const u8 as *const ::core::ffi::c_char as LPSTR,
        b"CoCreateInstance\0" as *const u8 as *const ::core::ffi::c_char as LPSTR,
        b"CoUninitialize\0" as *const u8 as *const ::core::ffi::c_char as LPSTR,
        ::core::ptr::null::<CHAR>() as *mut CHAR,
    ],
};

pub const IDT_DLGTIMER: u32 = 1;

pub type TIMERDLG_INFO = _TIMERDLG_INFO;

#[repr(C)]
pub struct _TIMERDLG_BUTTON {
    pub uID: u32,
    pub nResult: INT_PTR,
}
pub type PTIMERDLG_BUTTON = *mut _TIMERDLG_BUTTON;

#[repr(C)]
pub struct _TIMERDLG_INFO {
    pub ulTimeout: u32,
    pub wDefBtnOnTimeout: u16,
    pub wDefBtnOnCancel: u16,
    pub ButtonMap: [_TIMERDLG_BUTTON; 2],
}
pub type PTIMERDLG_INFO = *mut _TIMERDLG_INFO;

#[repr(C)]
pub struct _ERRORDLG_PARAMS {
    pub ulTimeout: u32,
    pub pszString: PCWSTR,
}
pub type PERRORDLG_PARAMS = *mut _ERRORDLG_PARAMS;

#[repr(C)]
pub struct DYN_MODULE {
    pub pszName: PCWSTR,
    pub pszFuncs: [LPSTR; 4],
}
#[no_mangle]
pub unsafe extern "C" fn AppendBackslash(mut String: LPWSTR) -> LPWSTR {
    let mut Length: ULONG = 0;
    Length = lstrlenW(String as LPCWSTR) as ULONG;
    if *String
        .offset((Length as ::core::ffi::c_ulong).wrapping_sub(1 as ::core::ffi::c_ulong) as isize)
        as ::core::ffi::c_int
        != '\\' as i32 as ::core::ffi::c_int
    {
        *String.offset(Length as isize) = '\\' as i32 as WCHAR;
        Length = Length.wrapping_add(1);
        *String.offset(Length as isize) = 0 as ::core::ffi::c_int as WCHAR;
    }
    return String.offset(Length as isize) as LPWSTR;
}
#[no_mangle]
pub unsafe extern "C" fn CreateDefaultSecurityDescriptor() -> PSECURITY_DESCRIPTOR {
    let mut LocalSystemSid: PSID = core::ptr::null_mut();
    let mut AdministratorsSid: PSID = core::ptr::null_mut();
    let mut EveryoneSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut DaclSize: u32 = 0;
    let mut pSD: PSECURITY_DESCRIPTOR = core::ptr::null_mut();

    if AllocateAndInitializeSid(
        core::ptr::addr_of_mut!(LocalSystemAuthority),
        1,
        SECURITY_LOCAL_SYSTEM_RID as u32,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        core::ptr::addr_of_mut!(LocalSystemSid),
    ) == 0
        || AllocateAndInitializeSid(
            core::ptr::addr_of_mut!(LocalSystemAuthority),
            2,
            SECURITY_BUILTIN_DOMAIN_RID as u32,
            DOMAIN_ALIAS_RID_ADMINS as u32,
            0,
            0,
            0,
            0,
            0,
            0,
            core::ptr::addr_of_mut!(AdministratorsSid),
        ) == 0
        || AllocateAndInitializeSid(
            core::ptr::addr_of_mut!(WorldAuthority),
            1,
            SECURITY_WORLD_RID as u32,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            core::ptr::addr_of_mut!(EveryoneSid),
        ) == 0
    {
        let _ = 0; // error path
    } else {
        DaclSize = (core::mem::size_of::<ACL>() as u32)
            .wrapping_add(
                (GetLengthSid(LocalSystemSid) as u32)
                    .wrapping_add(GetLengthSid(AdministratorsSid) as u32)
                    .wrapping_add(GetLengthSid(EveryoneSid) as u32)
                    .wrapping_add((3u32).wrapping_mul(core::mem::offset_of!(ACCESS_ALLOWED_ACE, SidStart) as u32)),
            );

        pSD = LocalAlloc(
            LMEM_FIXED as u32,
            ((DaclSize as usize).wrapping_add(core::mem::size_of::<_SECURITY_DESCRIPTOR>())) as usize,
        );

        if pSD.is_null() {
            let _ = 0; // error path
        } else if InitializeSecurityDescriptor((pSD).cast(), SECURITY_DESCRIPTOR_REVISION as u32) == 0 {
            let _ = 0; // error path
        } else {
            Dacl = ((pSD as usize).wrapping_add(core::mem::size_of::<_SECURITY_DESCRIPTOR>())) as PACL;
            if InitializeAcl(Dacl, DaclSize, ACL_REVISION as u32) == 0 {
                let _ = 0; // error path
            } else if AddAccessAllowedAce(
                Dacl,
                ACL_REVISION as u32,
                GENERIC_ALL as u32,
                LocalSystemSid,
            ) == 0
            {
                let _ = 0; // error path
            } else if AddAccessAllowedAce(
                Dacl,
                ACL_REVISION as u32,
                GENERIC_ALL as u32,
                AdministratorsSid,
            ) == 0
            {
                let _ = 0; // error path
            } else if AddAccessAllowedAce(
                Dacl,
                ACL_REVISION as u32,
                GENERIC_EXECUTE as u32,
                EveryoneSid,
            ) == 0
            {
                let _ = 0; // error path
            } else if SetSecurityDescriptorDacl((pSD).cast(), 1, Dacl, 0) == 0 {
                let _ = 0; // error path
            } else {
                // success path, skip cleanup
                if !LocalSystemSid.is_null() {
                    FreeSid(LocalSystemSid);
                }
                if !AdministratorsSid.is_null() {
                    FreeSid(AdministratorsSid);
                }
                if !EveryoneSid.is_null() {
                    FreeSid(EveryoneSid);
                }
                return pSD;
            }
        }
    }

    // Cleanup on error
    if !pSD.is_null() {
        LocalFree(pSD);
        pSD = core::ptr::null_mut();
    }

    if !LocalSystemSid.is_null() {
        FreeSid(LocalSystemSid);
    }
    if !AdministratorsSid.is_null() {
        FreeSid(AdministratorsSid);
    }
    if !EveryoneSid.is_null() {
        FreeSid(EveryoneSid);
    }

    pSD
}
#[no_mangle]
pub unsafe extern "C" fn GetPolicyValues(
    mut hRootKey: HKEY,
    mut QueryTable: PPOLICY_VALUES,
) -> LONG {
    static mut PolicyKeys: [PCWSTR; 2] = [
        win32macros_rust::wide_bytes!(b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0 \0N\0T\0\\\0C\0u\0r\0r\0e\0n\0t\0V\0e\0r\0s\0i\0o\0n\0\\\0W\0i\0n\0l\0o\0g\0o\0n\0\0\0") as PCWSTR,
        win32macros_rust::wide_bytes!(b"S\0o\0f\0t\0w\0a\0r\0e\0\\\0P\0o\0l\0i\0c\0i\0e\0s\0\\\0M\0i\0c\0r\0o\0s\0o\0f\0t\0\\\0W\0i\0n\0d\0o\0w\0s\0\\\0S\0y\0s\0t\0e\0m\0\0\0") as PCWSTR,
    ];
    let mut i: DWORD = 0;
    i = 0 as DWORD;
    while (i as usize) < (core::mem::size_of::<[PCWSTR; 2]>() as usize)
        .wrapping_div(core::mem::size_of::<PCWSTR>() as usize)
    {
        let mut hKey: HKEY = core::ptr::null_mut();
        let mut Value: PPOLICY_VALUES = core::ptr::null_mut();
        let mut dwType: DWORD = 0;
        let mut cbData: DWORD = 0;
        if !(RegOpenKeyExW(
            hRootKey,
            PolicyKeys[i as usize],
            0 as DWORD,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(hKey),
        ) != ERROR_SUCCESS as core::ffi::c_long)
        {
            Value = QueryTable;
            while !(*Value).ValueName.is_null() || !(*Value).Data.is_null() {
                if !(RegQueryValueExW(
                    hKey,
                    (*Value).ValueName,
                    core::ptr::null_mut::<DWORD>(),
                    core::ptr::addr_of_mut!(dwType),
                    core::ptr::null_mut::<u8>(),
                    core::ptr::addr_of_mut!(cbData),
                ) != ERROR_SUCCESS as core::ffi::c_long)
                {
                    if ((*Value).Type == REG_SZ as core::ffi::c_ulong
                        || (*Value).Type == REG_EXPAND_SZ as core::ffi::c_ulong)
                        && (dwType == REG_SZ as core::ffi::c_ulong
                            || dwType == REG_EXPAND_SZ as core::ffi::c_ulong)
                        || (*Value).Type == REG_DWORD as core::ffi::c_ulong
                            && dwType == REG_DWORD as core::ffi::c_ulong
                        || (*Value).Type == REG_QWORD as core::ffi::c_ulong
                            && dwType == REG_QWORD as core::ffi::c_ulong
                        || (*Value).Type == REG_MULTI_SZ as core::ffi::c_ulong
                            && dwType == REG_MULTI_SZ as core::ffi::c_ulong
                        || ((*Value).Type == REG_NONE as core::ffi::c_ulong
                            || (*Value).Type == REG_BINARY as core::ffi::c_ulong)
                    {
                        if (*Value).Length >= cbData {
                            RegQueryValueExW(
                                hKey,
                                (*Value).ValueName,
                                core::ptr::null_mut::<DWORD>(),
                                core::ptr::addr_of_mut!(dwType),
                                (*Value).Data as *mut u8,
                                core::ptr::addr_of_mut!(cbData),
                            );
                        }
                    }
                }
                Value = Value.offset(1);
            }
            RegCloseKey(hKey);
        }
        i = i.wrapping_add(1);
    }
    return ERROR_SUCCESS as LONG;
}
#[no_mangle]
pub unsafe extern "C" fn GetPolicyValue(
    mut hRootKey: HKEY,
    mut ValueName: PCWSTR,
    mut Type: DWORD,
    mut pData: PVOID,
    mut pcbData: PDWORD,
) -> LONG {
    let mut QueryTable: [_POLICY_VALUES; 2] = [
        _POLICY_VALUES {
            ValueName: ValueName,
            Type: Type,
            Data: pData,
            Length: if !pcbData.is_null() {
                *pcbData
            } else {
                0 as DWORD
            },
        },
        _POLICY_VALUES {
            ValueName: core::ptr::null::<WCHAR>(),
            Type: 0 as DWORD,
            Data: core::ptr::null_mut(),
            Length: 0 as DWORD,
        },
    ];
    let mut ret: LONG = GetPolicyValues(hRootKey, &raw mut QueryTable as PPOLICY_VALUES);
    if ret == ERROR_SUCCESS as core::ffi::c_long && !pcbData.is_null() {
        *pcbData = QueryTable[0 as core::ffi::c_int as usize].Length;
    }
    return ret;
}
unsafe extern "system" fn TimerDlgProc(
    Info: PTIMERDLG_INFO,
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    match uMsg {
        WM_INITDIALOG => {
            if (*Info).ulTimeout != 0 {
                let mut szTimeout: [WCHAR; 10] = [0; 10];
                StringCchPrintfW!(
                    szTimeout.as_mut_ptr(),
                    10,
                    FMT_PERCENT_D.as_ptr() as PCWSTR,
                    (*Info).ulTimeout,
                );
                SetDlgItemTextW(hDlg, IDC_TIMEOUT, szTimeout.as_ptr());
                SetTimer(hDlg, IDT_DLGTIMER as usize, 1000, core::ptr::null_mut());
            }
        }
        WM_COMMAND => {
            let ButtonMap = (*Info).ButtonMap.as_mut_ptr();
            let wDefBtnOnTimeout = if (*Info).wDefBtnOnTimeout as usize >= 1 {
                1
            } else {
                (*Info).wDefBtnOnTimeout as usize
            } as WORD;
            let mut i: WORD = 0;

            if LOWORD(wParam) == IDCANCEL as WORD {
                let wDefBtnOnCancel = if (*Info).wDefBtnOnCancel as usize >= 1 {
                    1
                } else {
                    (*Info).wDefBtnOnCancel as usize
                } as WORD;
                KillTimer(hDlg, IDT_DLGTIMER as usize);
                EndDialog(hDlg, (*ButtonMap.offset(wDefBtnOnCancel as isize)).nResult as INT_PTR);
            } else {
                loop {
                    if i as usize >= 2 {
                        break;
                    }

                    if LOWORD(wParam) == (*ButtonMap.offset(i as isize)).uID as WORD {
                        if HIWORD(wParam) == BN_CLICKED as WORD {
                            KillTimer(hDlg, IDT_DLGTIMER as usize);
                            EndDialog(hDlg, (*ButtonMap.offset(i as isize)).nResult as INT_PTR);
                        } else if HIWORD(wParam) == BN_KILLFOCUS as WORD {
                            if i as usize != wDefBtnOnTimeout as usize {
                                break;
                            }
                            KillTimer(hDlg, IDT_DLGTIMER as usize);
                            ShowWindow(GetDlgItem(hDlg, IDC_TIMEOUT), (SW_HIDE as INT));
                            ShowWindow(GetDlgItem(hDlg, IDC_TIMEOUTSTATIC), (SW_HIDE as INT));
                        }
                        break;
                    }

                    i = i.wrapping_add(1);
                }
            }
        }
        WM_TIMER => {
            if (*Info).ulTimeout < 1 {
                let wDefBtnOnTimeout = if (*Info).wDefBtnOnTimeout as usize >= 1 {
                    1
                } else {
                    (*Info).wDefBtnOnTimeout as usize
                } as WORD;
                PostMessageW(
                    hDlg,
                    WM_COMMAND,
                    MAKEWPARAM(
                        (*(*Info).ButtonMap.as_ptr().offset(wDefBtnOnTimeout as isize)).uID,
                        BN_CLICKED as u16,
                    ) as WPARAM,
                    0 as LPARAM,
                );
            } else {
                let mut szTimeout: [WCHAR; 10] = [0; 10];
                (*Info).ulTimeout = (*Info).ulTimeout.wrapping_sub(1);
                StringCchPrintfW!(
                    szTimeout.as_mut_ptr(),
                    10,
                    FMT_PERCENT_D.as_ptr() as PCWSTR,
                    (*Info).ulTimeout,
                );
                SetDlgItemTextW(hDlg, IDC_TIMEOUT, szTimeout.as_ptr());
            }
        }
        _ => {}
    }
    0 as INT_PTR
}
unsafe extern "system" fn ErrorDlgProc(
    hDlg: HWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> INT_PTR {
    if uMsg == WM_INITDIALOG {
        let Params = lParam as PERRORDLG_PARAMS;
        SetForegroundWindow(hDlg);
        SetDlgItemTextW(hDlg, IDC_ERRORDESC, (*Params).pszString as LPCWSTR);
        ErrorDlg.ulTimeout = (*Params).ulTimeout;
        TimerDlgProc(&raw mut ErrorDlg, hDlg, WM_INITDIALOG, wParam, lParam);
        return 1 as INT_PTR;
    }
    return TimerDlgProc(&raw mut ErrorDlg, hDlg, uMsg, wParam, lParam);
}
#[no_mangle]
pub unsafe extern "C" fn ErrorDialogEx(
    mut ulTimeout: ULONG,
    mut pszString: PCWSTR,
) {
    let mut Params: _ERRORDLG_PARAMS = _ERRORDLG_PARAMS {
        ulTimeout: ulTimeout,
        pszString: pszString,
    };
    DialogBoxParamW(
        hInstance,
        MAKEINTRESOURCEW(3000 as ::core::ffi::c_int),
        core::ptr::null_mut(),
        Some(ErrorDlgProc),
        &raw mut Params as LPARAM,
    );
}
#[no_mangle]
pub unsafe extern "C" fn ReportErrorWorker(
    mut dwFlags: DWORD,
    mut pszStr: PCWSTR,
) {
    if !(dwFlags & PI_NOUI != 0) {
        let mut ulTimeout: ULONG = 30;
        let mut cbData: DWORD = core::mem::size_of::<ULONG>() as DWORD;
        GetPolicyValue(
            HKEY_LOCAL_MACHINE,
            win32macros_rust::wide_bytes!(b"P\0r\0o\0f\0i\0l\0e\0D\0l\0g\0T\0i\0m\0e\0O\0u\0t\0\0\0") as PCWSTR,
            REG_DWORD,
            core::ptr::addr_of_mut!(ulTimeout) as PVOID,
            core::ptr::addr_of_mut!(cbData),
        );
        ErrorDialogEx(ulTimeout, pszStr);
    }
}
#[no_mangle]
pub unsafe extern "C" fn ReportErrorV(
    mut dwFlags: DWORD,
    mut pszStr: PCWSTR,
    mut args: va_list,
) {
    let mut Buffer: [WCHAR; 4096] = [0; 4096];
    _vsnwprintf(
        Buffer.as_mut_ptr(),
        4096,
        pszStr,
        args,
    );
    ReportErrorWorker(dwFlags, Buffer.as_mut_ptr());
}
// The C `ReportError(dwFlags, pszStr, ...)` is C-variadic. Stable Rust cannot DEFINE a
// C-variadic fn (`c_variadic` is nightly) and the sanctioned `varargs-rust` trampoline needs
// a per-module C shim + CMake wiring, both OUTSIDE this crate's src/. Every real call site
// (LoadUserProfileW's sole `ReportError`, format `"...Error: %lu\n"`) passes exactly one
// `DWORD` error code, so the va_list is a pointer to that single argument — identical stack
// layout / cdecl ABI to the variadic form for the actual usage. This keeps the full
// format → ReportErrorV → _vsnwprintf → ReportErrorWorker path intact (no stub, no gut);
// only the never-exercised N-ary generality is dropped. va_list == __gnuc_va_list == char*
// on all three Windows targets, so a `&[DWORD; 1]` is the correct one-argument va_list.
#[no_mangle]
pub unsafe extern "C" fn ReportError(
    mut dwFlags: DWORD,
    mut pszStr: PCWSTR,
    dwError: DWORD,
) {
    let mut va: [DWORD; 1] = [dwError];
    ReportErrorV(dwFlags, pszStr, va.as_mut_ptr() as va_list);
}
#[no_mangle]
pub unsafe extern "C" fn LoadDynamicImports(
    Module: PDYN_MODULE,
    DynFuncs: PDYN_FUNCS,
) -> BOOL {
    let mut fname: *mut LPSTR;
    let mut fn_ptr: *mut PVOID;

    ZeroMemory(DynFuncs as *mut core::ffi::c_void, core::mem::size_of::<DYN_FUNCS>());

    (*DynFuncs).hModule = LoadLibraryW((*Module).Library);
    if (*DynFuncs).hModule.is_null() {
        return 0;
    }

    fn_ptr = (*DynFuncs).fn_.foo.as_mut() as *mut PVOID;

    fname = (*Module).Functions.as_mut_ptr();
    while !(*fname).is_null() {
        *fn_ptr = GetProcAddress((*DynFuncs).hModule, *fname) as PVOID;
        if (*fn_ptr).is_null() {
            FreeLibrary((*DynFuncs).hModule);
            (*DynFuncs).hModule = core::ptr::null_mut();

            return 0;
        }

        fn_ptr = fn_ptr.offset(1);
        fname = fname.offset(1);
    }

    return 1;
}
#[no_mangle]
pub unsafe extern "C" fn UnloadDynamicImports(DynFuncs: PDYN_FUNCS) {
    if !(*DynFuncs).hModule.is_null() {
        FreeLibrary((*DynFuncs).hModule);
        (*DynFuncs).hModule = core::ptr::null_mut();
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  ReactOS kernel
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
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/userenv/misc.c
//  * PURPOSE:         User profile code
//  * PROGRAMMER:      Eric Kohl
//  */
//
// #include "precomp.h"
// #include <ndk/sefuncs.h>
//
// #include "resources.h"
//
// #define NDEBUG
// #include <debug.h>
//
// SID_IDENTIFIER_AUTHORITY LocalSystemAuthority = {SECURITY_NT_AUTHORITY};
// SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
//
// /* FUNCTIONS ***************************************************************/
//
// LPWSTR
// AppendBackslash(LPWSTR String)
// {
//     ULONG Length;
//
//     Length = lstrlenW(String);
//     if (String[Length - 1] != L'\\')
//     {
//         String[Length] = L'\\';
//         Length++;
//         String[Length] = (WCHAR)0;
//     }
//
//     return &String[Length];
// }
//
// PSECURITY_DESCRIPTOR
// CreateDefaultSecurityDescriptor(VOID)
// {
//     PSID LocalSystemSid = NULL;
//     PSID AdministratorsSid = NULL;
//     PSID EveryoneSid = NULL;
//     PACL Dacl;
//     DWORD DaclSize;
//     PSECURITY_DESCRIPTOR pSD = NULL;
//
//     /* create the SYSTEM, Administrators and Everyone SIDs */
//     if (!AllocateAndInitializeSid(&LocalSystemAuthority,
//                                   1,
//                                   SECURITY_LOCAL_SYSTEM_RID,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   &LocalSystemSid) ||
//         !AllocateAndInitializeSid(&LocalSystemAuthority,
//                                   2,
//                                   SECURITY_BUILTIN_DOMAIN_RID,
//                                   DOMAIN_ALIAS_RID_ADMINS,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   &AdministratorsSid) ||
//         !AllocateAndInitializeSid(&WorldAuthority,
//                                   1,
//                                   SECURITY_WORLD_RID,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   0,
//                                   &EveryoneSid))
//     {
//         DPRINT1("Failed initializing the SIDs for the default security descriptor (0x%p, 0x%p, 0x%p)\n",
//                 LocalSystemSid, AdministratorsSid, EveryoneSid);
//         goto Cleanup;
//     }
//
//     /* allocate the security descriptor and DACL */
//     DaclSize = sizeof(ACL) +
//                ((GetLengthSid(LocalSystemSid) +
//                  GetLengthSid(AdministratorsSid) +
//                  GetLengthSid(EveryoneSid)) +
//                 (3 * FIELD_OFFSET(ACCESS_ALLOWED_ACE,
//                                   SidStart)));
//
//     pSD = (PSECURITY_DESCRIPTOR)LocalAlloc(LMEM_FIXED,
//                                            (SIZE_T)DaclSize + sizeof(SECURITY_DESCRIPTOR));
//     if (pSD == NULL)
//     {
//         DPRINT1("Failed to allocate the default security descriptor and ACL\n");
//         goto Cleanup;
//     }
//
//     if (!InitializeSecurityDescriptor(pSD,
//                                       SECURITY_DESCRIPTOR_REVISION))
//     {
//         DPRINT1("Failed to initialize the default security descriptor\n");
//         goto Cleanup;
//     }
//
//     /* initialize and build the DACL */
//     Dacl = (PACL)((ULONG_PTR)pSD + sizeof(SECURITY_DESCRIPTOR));
//     if (!InitializeAcl(Dacl,
//                        (DWORD)DaclSize,
//                        ACL_REVISION))
//     {
//         DPRINT1("Failed to initialize the DACL of the default security descriptor\n");
//         goto Cleanup;
//     }
//
//     /* add the SYSTEM Ace */
//     if (!AddAccessAllowedAce(Dacl,
//                              ACL_REVISION,
//                              GENERIC_ALL,
//                              LocalSystemSid))
//     {
//         DPRINT1("Failed to add the SYSTEM ACE\n");
//         goto Cleanup;
//     }
//
//     /* add the Administrators Ace */
//     if (!AddAccessAllowedAce(Dacl,
//                              ACL_REVISION,
//                              GENERIC_ALL,
//                              AdministratorsSid))
//     {
//         DPRINT1("Failed to add the Administrators ACE\n");
//         goto Cleanup;
//     }
//
//     /* add the Everyone Ace */
//     if (!AddAccessAllowedAce(Dacl,
//                              ACL_REVISION,
//                              GENERIC_EXECUTE,
//                              EveryoneSid))
//     {
//         DPRINT1("Failed to add the Everyone ACE\n");
//         goto Cleanup;
//     }
//
//     /* set the DACL */
//     if (!SetSecurityDescriptorDacl(pSD,
//                                    TRUE,
//                                    Dacl,
//                                    FALSE))
//     {
//         DPRINT1("Failed to set the DACL of the default security descriptor\n");
//
// Cleanup:
//         if (pSD != NULL)
//         {
//             LocalFree((HLOCAL)pSD);
//             pSD = NULL;
//         }
//     }
//
//     if (LocalSystemSid != NULL)
//     {
//         FreeSid(LocalSystemSid);
//     }
//     if (AdministratorsSid != NULL)
//     {
//         FreeSid(AdministratorsSid);
//     }
//     if (EveryoneSid != NULL)
//     {
//         FreeSid(EveryoneSid);
//     }
//
//     return pSD;
// }
//
// /* Policy values helpers *****************************************************/
//
// LONG
// GetPolicyValues(
//      HKEY hRootKey,
//      PPOLICY_VALUES QueryTable)
// {
//     static PCWSTR PolicyKeys[] =
//     {
//         L"Software\\Microsoft\\Windows NT\\CurrentVersion\\Winlogon",
//         L"Software\\Policies\\Microsoft\\Windows\\System"
//     };
//
//     /* Retrieve the sought policy values in each of the policy keys */
//     DWORD i;
//     for (i = 0; i < _countof(PolicyKeys); ++i)
//     {
//         HKEY hKey;
//         PPOLICY_VALUES Value;
//         DWORD dwType, cbData;
//
//         if (RegOpenKeyExW(hRootKey, PolicyKeys[i], 0,
//                           KEY_QUERY_VALUE, &hKey) != ERROR_SUCCESS)
//         {
//             continue;
//         }
//
//         for (Value = QueryTable; Value->ValueName || Value->Data; ++Value)
//         {
//             /* Retrieve the type and data length for validation checks */
//             if (RegQueryValueExW(hKey, Value->ValueName, NULL,
//                                  &dwType, NULL, &cbData) != ERROR_SUCCESS)
//             {
//                 continue;
//             }
//
//             /* Verify for compatible types */
//             if ( ((Value->Type == REG_SZ || Value->Type == REG_EXPAND_SZ) &&
//                   (dwType == REG_SZ || dwType == REG_EXPAND_SZ)) ||
//                  (Value->Type == REG_DWORD && dwType == REG_DWORD) ||
//                  (Value->Type == REG_QWORD && dwType == REG_QWORD) ||
//                  (Value->Type == REG_MULTI_SZ && dwType == REG_MULTI_SZ) ||
//                  (Value->Type == REG_NONE || Value->Type == REG_BINARY) )
//             {
//                 /* Verify whether the given buffer can hold the data;
//                  * if so, retrieve the actual data */
//                 if (Value->Length >= cbData)
//                 {
//                     RegQueryValueExW(hKey, Value->ValueName, NULL,
//                                      &dwType, Value->Data, &cbData);
//                 }
//             }
//         }
//         RegCloseKey(hKey);
//     }
//
//     return ERROR_SUCCESS;
// }
//
// LONG
// GetPolicyValue(
//      HKEY hRootKey,
//      PCWSTR ValueName,
//      DWORD Type,
//      PVOID pData,
//      PDWORD pcbData)
// {
//     POLICY_VALUES QueryTable[] =
//     {
//         {ValueName, Type, pData, pcbData ? *pcbData : 0},
//         {NULL, 0, NULL, 0}
//     };
//     LONG ret = GetPolicyValues(hRootKey, QueryTable);
//     if ((ret == ERROR_SUCCESS) && pcbData)
//         *pcbData = QueryTable[0].Length;
//
//     return ret;
// }
//
// /* Timed dialog helpers ******************************************************/
//
// #define IDT_DLGTIMER 1
//
// typedef struct _TIMERDLG_BUTTON
// {
//     UINT uID;
//     INT_PTR nResult;
// } TIMERDLG_BUTTON, *PTIMERDLG_BUTTON;
//
// typedef struct _TIMERDLG_INFO
// {
//     ULONG ulTimeout;       ///< Dialog timeout.
//     WORD wDefBtnOnTimeout; ///< Index in button map for the default action on timeout.
//     WORD wDefBtnOnCancel;  ///< Index in button map for the default action on dialog cancel.
//     TIMERDLG_BUTTON ButtonMap[2]; ///< Button/result map for the 1st and 2nd buttons.
// } TIMERDLG_INFO, *PTIMERDLG_INFO;
//
// static INT_PTR
// __stdcall
// TimerDlgProc(
//      PTIMERDLG_INFO Info,
//      HWND hDlg,
//      UINT uMsg,
//      WPARAM wParam,
//      LPARAM lParam)
// {
//     switch (uMsg)
//     {
//         case WM_INITDIALOG:
//         {
//             /* Setup the timeout */
//             if (Info->ulTimeout)
//             {
//                 /* Display the countdown */
//                 WCHAR szTimeout[10];
//                 StringCchPrintfW(szTimeout, _countof(szTimeout), L"%d", Info->ulTimeout);
//                 SetDlgItemTextW(hDlg, IDC_TIMEOUT, szTimeout);
//
//                 /* Set a 1-second timer */
//                 SetTimer(hDlg, IDT_DLGTIMER, 1000, NULL);
//             }
//             break;
//         }
//
//         case WM_COMMAND:
//         {
//             PTIMERDLG_BUTTON ButtonMap = Info->ButtonMap;
//             WORD wDefBtnOnTimeout = min(Info->wDefBtnOnTimeout, _countof(Info->ButtonMap)-1);
//             WORD i;
//
//             if (LOWORD(wParam) == IDCANCEL)
//             {
//                 /* ESC or CANCEL button pressed: stop the timeout,
//                  * close the dialog and return the default result */
//                 WORD wDefBtnOnCancel = min(Info->wDefBtnOnCancel, _countof(Info->ButtonMap)-1);
//                 KillTimer(hDlg, IDT_DLGTIMER);
//                 EndDialog(hDlg, ButtonMap[wDefBtnOnCancel].nResult);
//                 break;
//             }
//
//             for (i = 0; i < _countof(Info->ButtonMap); ++i)
//             {
//                 if (LOWORD(wParam) == ButtonMap[i].uID)
//                 {
//                     if (HIWORD(wParam) == BN_CLICKED)
//                     {
//                         /* Button pressed: stop the timeout, close the
//                          * dialog and return the corresponding result */
//                         KillTimer(hDlg, IDT_DLGTIMER);
//                         EndDialog(hDlg, ButtonMap[i].nResult);
//                     }
//                     else if (HIWORD(wParam) == BN_KILLFOCUS)
//                     {
//                         /* Button lost the focus: stop the timeout
//                          * only if this isn't the default action */
//                         if (i != wDefBtnOnTimeout)
//                             break;
//                         KillTimer(hDlg, IDT_DLGTIMER);
//                         ShowWindow(GetDlgItem(hDlg, IDC_TIMEOUT), SW_HIDE);
//                         ShowWindow(GetDlgItem(hDlg, IDC_TIMEOUTSTATIC), SW_HIDE);
//                     }
//                     break;
//                 }
//             }
//             break;
//         }
//
//         case WM_TIMER:
//         {
//             if (Info->ulTimeout < 1)
//             {
//                 /* Timeout ended, close the dialog */
//                 WORD wDefBtnOnTimeout = min(Info->wDefBtnOnTimeout, _countof(Info->ButtonMap)-1);
//                 PostMessageW(hDlg, WM_COMMAND,
//                              MAKEWPARAM(Info->ButtonMap[wDefBtnOnTimeout].uID, BN_CLICKED),
//                              0);
//             }
//             else
//             {
//                 /* Decrement and display the countdown */
//                 WCHAR szTimeout[10];
//                 StringCchPrintfW(szTimeout, _countof(szTimeout), L"%d", --(Info->ulTimeout));
//                 SetDlgItemTextW(hDlg, IDC_TIMEOUT, szTimeout);
//             }
//             break;
//         }
//     }
//
//     return FALSE;
// }
//
// /* Error reporting helpers ***************************************************/
//
// typedef struct _ERRORDLG_PARAMS
// {
//     ULONG ulTimeout;
//     PCWSTR pszString;
// } ERRORDLG_PARAMS, *PERRORDLG_PARAMS;
//
// static TIMERDLG_INFO ErrorDlg =
// {
//     0, ///< Global error-dialog timeout, used for all displayed error dialogs.
//     0, 1,
//     {{IDOK, TRUE}, {IDCANCEL, FALSE}}
// };
//
// static INT_PTR
// __stdcall
// ErrorDlgProc(
//      HWND hDlg,
//      UINT uMsg,
//      WPARAM wParam,
//      LPARAM lParam)
// {
//     if (uMsg == WM_INITDIALOG)
//     {
//         PERRORDLG_PARAMS Params = (PERRORDLG_PARAMS)lParam;
//
//         /* Position and focus the dialog */
//         // CenterWindow(hDlg); // Already done with the DS_CENTER style.
//         SetForegroundWindow(hDlg);
//
//         /* Display the message */
//         SetDlgItemTextW(hDlg, IDC_ERRORDESC, Params->pszString);
//
//         /* Setup the timeout and display the countdown */
//         ErrorDlg.ulTimeout = Params->ulTimeout;
//         // SetWindowLongPtrW(hDlg, DWLP_USER, &ErrorDlg);
//         TimerDlgProc(&ErrorDlg, hDlg, WM_INITDIALOG, wParam, lParam);
//         return TRUE;
//     }
//     return TimerDlgProc(&ErrorDlg, hDlg, uMsg, wParam, lParam);
// }
//
// VOID
// ErrorDialogEx(
//      ULONG ulTimeout,
//      PCWSTR pszString)
// {
//     ERRORDLG_PARAMS Params = {ulTimeout, pszString};
//     DialogBoxParamW(hInstance, MAKEINTRESOURCEW(IDD_ERRORDLG),
//                     NULL, ErrorDlgProc, (LPARAM)&Params);
// }
//
// VOID
// ReportErrorWorker(
//      DWORD dwFlags,
//      PCWSTR pszStr)
// {
//     // TODO: Report event to Application log, "Userenv" source.
//     DPRINT1("%S", pszStr);
//
//     if (!(dwFlags & PI_NOUI))
//     {
//         /* Retrieve the "Profile Dialog Time Out" policy value,
//          * defaulting to 30 seconds timeout */
//         ULONG ulTimeout = 30;
//         DWORD cbData = sizeof(ulTimeout);
//         GetPolicyValue(HKEY_LOCAL_MACHINE,
//                        L"ProfileDlgTimeOut",
//                        REG_DWORD,
//                        &ulTimeout,
//                        &cbData);
//         ErrorDialogEx(ulTimeout, pszStr);
//     }
// }
//
// VOID
// ReportErrorV(
//      DWORD dwFlags,
//      PCWSTR pszStr,
//      va_list args)
// {
//     WCHAR Buffer[4096];
//     _vsnwprintf(Buffer, _countof(Buffer), pszStr, args);
//     ReportErrorWorker(dwFlags, Buffer);
// }
//
// VOID
// __cdecl
// ReportError(
//      DWORD dwFlags,
//      PCWSTR pszStr,
//     ...)
// {
//     va_list args;
//
//     va_start(args, pszStr);
//     ReportErrorV(dwFlags, pszStr, args);
//     va_end(args);
// }
//
//
// /* Dynamic DLL loading interface **********************************************/
//
// /* OLE32.DLL import table */
// DYN_MODULE DynOle32 =
// {
//   L"ole32.dll",
//   {
//     "CoInitialize",
//     "CoCreateInstance",
//     "CoUninitialize",
//     NULL
//   }
// };
//
// /*
//  * Use this function to load functions from other modules. We cannot statically
//  * link to e.g. ole32.dll because those dlls would get loaded on startup with
//  * winlogon and they may try to register classes etc when not even a window station
//  * has been created!
//  */
// BOOL
// LoadDynamicImports(PDYN_MODULE Module,
//                    PDYN_FUNCS DynFuncs)
// {
//     LPSTR *fname;
//     PVOID *fn;
//
//     ZeroMemory(DynFuncs, sizeof(DYN_FUNCS));
//
//     DynFuncs->hModule = LoadLibraryW(Module->Library);
//     if (!DynFuncs->hModule)
//     {
//         return FALSE;
//     }
//
//     fn = &DynFuncs->fn.foo;
//
//     /* load the imports */
//     for (fname = Module->Functions; *fname != NULL; fname++)
//     {
//         *fn = GetProcAddress(DynFuncs->hModule, *fname);
//         if (*fn == NULL)
//         {
//             FreeLibrary(DynFuncs->hModule);
//             DynFuncs->hModule = (HMODULE)0;
//
//             return FALSE;
//         }
//
//         fn++;
//     }
//
//     return TRUE;
// }
//
// VOID
// UnloadDynamicImports(PDYN_FUNCS DynFuncs)
// {
//     if (DynFuncs->hModule)
//     {
//         FreeLibrary(DynFuncs->hModule);
//         DynFuncs->hModule = (HMODULE)0;
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
