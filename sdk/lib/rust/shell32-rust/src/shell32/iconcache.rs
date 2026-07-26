//! ORPHAN-BY-DESIGN: translated shell32 TU, parked pending substrate closure.
//! Unblocked by the 2026-07-25 seal fixes (macro authority, Rust-label masking,
//! goto replication, prototype/named-cast types); still needs SDK externs, Win32
//! constants and resource IDs resolved before it compiles. Wiring it in this state
//! would break the build for every lane rather than expose a regression.
//! This is a PAID artifact — do not delete it to tidy up.
// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_2C74F66573C9: [WCHAR; 22] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0053, 0x006D, 0x0061, 0x006C, 0x006C, 0x0020, 0x0049, 0x0063, 0x006F, 0x006E, 0x0020, 0x0053, 0x0069, 0x007A, 0x0065, 0x0000];
pub static CPP_COM_WSTR_47678CDC8AAF: [WCHAR; 15] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0049, 0x0063, 0x006F, 0x006E, 0x0020, 0x0042, 0x0050, 0x0050, 0x0000];
pub static CPP_COM_WSTR_50CEA9783DF2: [WCHAR; 36] = [0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0020, 0x0050, 0x0061, 0x006E, 0x0065, 0x006C, 0x005C, 0x0044, 0x0065, 0x0073, 0x006B, 0x0074, 0x006F, 0x0070, 0x005C, 0x0057, 0x0069, 0x006E, 0x0064, 0x006F, 0x0077, 0x004D, 0x0065, 0x0074, 0x0072, 0x0069, 0x0063, 0x0073, 0x0000];
pub static CPP_COM_WSTR_88409267423F: [WCHAR; 16] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0020, 0x0049, 0x0063, 0x006F, 0x006E, 0x0020, 0x0053, 0x0069, 0x007A, 0x0065, 0x0000];

use core::ffi::c_void;
use crate::shellcom::*;
use crate::types::*;
use crate::win32::*;

// ---- file-scope preamble (translated) ----
pub const INVALID_INDEX: i32 = -1;

#[repr(C)]
pub struct SIC_ENTRY {
    pub sSourceFile: LPWSTR,
    pub dwSourceIndex: DWORD,
    pub dwListIndex: DWORD,
    pub dwFlags: DWORD,
    pub dwAccessTime: DWORD,
}

pub type LPSIC_ENTRY = *mut SIC_ENTRY;

pub static mut sic_hdpa: HDPA = 0 as HDPA;
pub static mut ShellSmallIconList: HIMAGELIST = 0 as HIMAGELIST;
pub static mut ShellBigIconList: HIMAGELIST = 0 as HIMAGELIST;
pub static mut ShellSmallIconSize: INT = 0;
pub static mut ShellLargeIconSize: INT = 0;
pub static mut ShellIconBPP: INT = 0;

pub static mut SHELL32_SicCS: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: unsafe { core::ptr::addr_of_mut!(critsect_debug) },
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: 0 as HANDLE,
    LockSemaphore: 0 as HANDLE,
    SpinCount: 0,
};

pub static mut critsect_debug: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: unsafe { core::ptr::addr_of_mut!(SHELL32_SicCS) },
    ProcessLocksList: _LIST_ENTRY {
        Flink: unsafe { core::ptr::addr_of_mut!(critsect_debug.ProcessLocksList) },
        Blink: unsafe { core::ptr::addr_of_mut!(critsect_debug.ProcessLocksList) },
    },
    EntryCount: 0,
    ContentionCount: 0,
    Flags: 0,
    CreatorBackTraceIndexHigh: 0,
    SpareWORD: 0,
};
unsafe extern "C" fn SIC_GetMetricsValue(pszValueName: PCWSTR, nDefaultValue: i32) -> i32 {
    let mut szValue: [u16; 64] = [0u16; 64];
    let mut cbValue: u32 = core::mem::size_of_val(&szValue) as u32;
    let error = SHGetValueW(
        HKEY_CURRENT_USER,
        CPP_COM_WSTR_50CEA9783DF2.as_ptr(),
        pszValueName,
        core::ptr::null_mut(),
        szValue.as_mut_ptr() as *mut core::ffi::c_void,
        core::ptr::addr_of_mut!(cbValue),
    );
    if error != 0 {
        return nDefaultValue;
    }
    let idx = _countof(szValue) - 1;
    szValue[idx] = UNICODE_NULL; // Avoid buffer overrun
    _wtoi(szValue.as_ptr() as PCWSTR)
}
unsafe extern "C" fn SIC_GetLargeIconSize() -> i32 {
    let nDefaultSize: i32 = GetSystemMetrics(SM_CXICON);
    let nIconSize: i32 = SIC_GetMetricsValue(CPP_COM_WSTR_88409267423F.as_ptr(), nDefaultSize);
    if nIconSize > 0 { nIconSize } else { nDefaultSize }
}
unsafe extern "C" fn SIC_GetSmallIconSize() -> i32 {
    // NOTE: Shell icon size is always square
    let nDefaultSize = GetSystemMetrics(SM_CXSMICON);
    let nIconSize = SIC_GetMetricsValue(CPP_COM_WSTR_2C74F66573C9.as_ptr(), nDefaultSize);
    if nIconSize > 0 { nIconSize } else { nDefaultSize }
}
unsafe extern "C" fn SIC_GetIconBPP() -> i32 {
    let nDefaultBPP = SHGetCurColorRes();
    let nIconBPP = SIC_GetMetricsValue(CPP_COM_WSTR_47678CDC8AAF.as_ptr(), nDefaultBPP);
    if nIconBPP > 0 {
        nIconBPP
    } else {
        nDefaultBPP
    }
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn SIC_CompareEntries(
    p1: *mut core::ffi::c_void,
    p2: *mut core::ffi::c_void,
    _lparam: LPARAM,
) -> i32 {
    let e1: *mut SIC_ENTRY = p1 as *mut SIC_ENTRY;
    let e2: *mut SIC_ENTRY = p2 as *mut SIC_ENTRY;

    if (*e1).dwSourceIndex != (*e2).dwSourceIndex {
        return if (*e1).dwSourceIndex < (*e2).dwSourceIndex {
            -1
        } else {
            1
        };
    }

    if ((*e1).dwFlags & GIL_FORSHORTCUT) != ((*e2).dwFlags & GIL_FORSHORTCUT) {
        return if ((*e1).dwFlags & GIL_FORSHORTCUT) < ((*e2).dwFlags & GIL_FORSHORTCUT) {
            -1
        } else {
            1
        };
    }

    _wcsicmp((*e1).sSourceFile, (*e2).sSourceFile) as i32
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn SIC_CompareEntries(
    p1: *mut core::ffi::c_void,
    p2: *mut core::ffi::c_void,
    _lparam: LPARAM,
) -> i32 {
    let e1: *mut SIC_ENTRY = p1 as *mut SIC_ENTRY;
    let e2: *mut SIC_ENTRY = p2 as *mut SIC_ENTRY;

    if (*e1).dwSourceIndex != (*e2).dwSourceIndex {
        return if (*e1).dwSourceIndex < (*e2).dwSourceIndex {
            -1
        } else {
            1
        };
    }

    if ((*e1).dwFlags & GIL_FORSHORTCUT) != ((*e2).dwFlags & GIL_FORSHORTCUT) {
        return if ((*e1).dwFlags & GIL_FORSHORTCUT) < ((*e2).dwFlags & GIL_FORSHORTCUT) {
            -1
        } else {
            1
        };
    }

    _wcsicmp((*e1).sSourceFile, (*e2).sSourceFile) as i32
}
unsafe extern "C" fn SIC_OverlayShortcutImage(SourceIcon: HICON, large: BOOL) -> HICON {
    let mut ShortcutIcon: HICON = core::ptr::null_mut();
    let mut ShortcutIconInfo: ICONINFO = core::mem::zeroed();
    let mut TargetIconInfo: ICONINFO = core::mem::zeroed();
    ZeroMemory(core::ptr::addr_of_mut!(ShortcutIconInfo), core::mem::size_of::<ICONINFO>());
    ZeroMemory(core::ptr::addr_of_mut!(TargetIconInfo), core::mem::size_of::<ICONINFO>());

    let mut TargetBitmapInfo: BITMAP = core::mem::zeroed();
    let mut ShortcutBitmapInfo: BITMAP = core::mem::zeroed();
    let mut ShortcutDC: HDC = core::ptr::null_mut();
    let mut TargetDC: HDC = core::ptr::null_mut();
    let mut OldShortcutBitmap: HBITMAP = core::ptr::null_mut();
    let mut OldTargetBitmap: HBITMAP = core::ptr::null_mut();
    let mut TargetIcon: HICON = core::ptr::null_mut();

    if GetIconInfo(SourceIcon, core::ptr::addr_of_mut!(TargetIconInfo)) == 0 {
        return core::ptr::null_mut();
    }

    if TargetIconInfo.hbmColor.is_null() {
        goto fail;
    }

    if GetObjectW(TargetIconInfo.hbmColor, core::mem::size_of::<BITMAP>(), core::ptr::addr_of_mut!(TargetBitmapInfo)) == 0 {
        goto fail;
    }

    if s_imgListIdx == -1 {
        s_imgListIdx = SIC_LoadOverlayIcon(IDI_SHELL_SHORTCUT - 1);
    }

    if s_imgListIdx != -1 {
        let idx = s_imgListIdx as c_int;
        if large != 0 {
            ShortcutIcon = ImageList_GetIcon(ShellBigIconList, idx, ILD_TRANSPARENT);
        } else {
            ShortcutIcon = ImageList_GetIcon(ShellSmallIconList, idx, ILD_TRANSPARENT);
        }
    } else {
        ShortcutIcon = core::ptr::null_mut();
    }

    if ShortcutIcon.is_null() || GetIconInfo(ShortcutIcon, core::ptr::addr_of_mut!(ShortcutIconInfo)) == 0 {
        goto fail;
    }

    if ShortcutIconInfo.hbmColor.is_null() {
        goto fail;
    }

    if GetObjectW(ShortcutIconInfo.hbmColor, core::mem::size_of::<BITMAP>(), core::ptr::addr_of_mut!(ShortcutBitmapInfo)) == 0 {
        goto fail;
    }

    ShortcutDC = CreateCompatibleDC(core::ptr::null_mut());
    if ShortcutDC.is_null() {
        goto fail;
    }
    OldShortcutBitmap = SelectObject(ShortcutDC, ShortcutIconInfo.hbmMask) as HBITMAP;
    if OldShortcutBitmap.is_null() {
        goto fail;
    }

    TargetDC = CreateCompatibleDC(core::ptr::null_mut());
    if TargetDC.is_null() {
        goto fail;
    }
    OldTargetBitmap = SelectObject(TargetDC, TargetIconInfo.hbmMask) as HBITMAP;
    if OldTargetBitmap.is_null() {
        goto fail;
    }

    if BitBlt(TargetDC, 0, 0,
                               ShortcutBitmapInfo.bmWidth, ShortcutBitmapInfo.bmHeight,
                               ShortcutDC, 0, 0,
                               SRCAND) == 0 {
        goto fail;
    }

    let mut add_alpha: BOOL = 0;
    const BUFFER_SIZE: usize = core::mem::size_of::<BITMAPINFO>() + (256 as usize) * core::mem::size_of::<RGBQUAD>();
    let mut buffer: [u8; BUFFER_SIZE] = [0u8; BUFFER_SIZE];
    let lpbmi: *mut BITMAPINFO = buffer.as_mut_ptr() as *mut BITMAPINFO;

    if ShortcutBitmapInfo.bmBitsPixel == 32 {
        if TargetBitmapInfo.bmBitsPixel != 32 {
            add_alpha = FALSE;
        } else {
            ZeroMemory(buffer.as_mut_ptr() as *mut c_void, BUFFER_SIZE);
            (*lpbmi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
            (*lpbmi).bmiHeader.biWidth = TargetBitmapInfo.bmWidth;
            (*lpbmi).bmiHeader.biHeight = TargetBitmapInfo.bmHeight;
            (*lpbmi).bmiHeader.biPlanes = 1 as u16;
            (*lpbmi).bmiHeader.biBitCount = 32 as u16;

            let bits: *mut c_void = HeapAlloc(
                GetProcessHeap(),
                0,
                (TargetBitmapInfo.bmHeight * TargetBitmapInfo.bmWidthBytes) as usize,
            );
            if bits.is_null() {
                goto fail;
            }

            if GetDIBits(TargetDC, TargetIconInfo.hbmColor, 0,
                                        TargetBitmapInfo.bmHeight, bits, lpbmi,
                                        DIB_RGB_COLORS) == 0 {
                HeapFree(GetProcessHeap(), 0, bits);
                goto fail;
            }

            let mut pixel: *mut u32 = bits as *mut u32;
            let mut j: i32 = 0;
            let mut i: i32 = 0;
            while i < TargetBitmapInfo.bmHeight {
                j = 0;
                while j < TargetBitmapInfo.bmWidth {
                    add_alpha = ((*pixel & 0xFF000000) != 0) as i32;
                    pixel = pixel.add(1);
                    if add_alpha != 0 {
                        break;
                    }
                    j += 1;
                }
                if add_alpha != 0 {
                    break;
                }
                i += 1;
            }
            HeapFree(GetProcessHeap(), 0, bits);
        }

        let bits: *mut c_void = HeapAlloc(
            GetProcessHeap(),
            0,
            (ShortcutBitmapInfo.bmHeight * ShortcutBitmapInfo.bmWidthBytes) as usize,
        );
        if bits.is_null() {
            goto fail;
        }

        ZeroMemory(buffer.as_mut_ptr() as *mut c_void, BUFFER_SIZE);
        (*lpbmi).bmiHeader.biSize = core::mem::size_of::<BITMAPINFOHEADER>() as u32;
        (*lpbmi).bmiHeader.biWidth = ShortcutBitmapInfo.bmWidth;
        (*lpbmi).bmiHeader.biHeight = ShortcutBitmapInfo.bmHeight;
        (*lpbmi).bmiHeader.biPlanes = 1 as u16;
        (*lpbmi).bmiHeader.biBitCount = 32 as u16;

        if GetDIBits(TargetDC, ShortcutIconInfo.hbmColor, 0,
                                    ShortcutBitmapInfo.bmHeight, bits, lpbmi,
                                    DIB_RGB_COLORS) == 0 {
            HeapFree(GetProcessHeap(), 0, bits);
            goto fail;
        }

        let mut pixel: *mut u32 = bits as *mut u32;
        let mut i: i32 = 0;
        while i < ShortcutBitmapInfo.bmHeight {
            let mut j: i32 = 0;
            while j < ShortcutBitmapInfo.bmWidth {
                if add_alpha != 0 {
                    *pixel |= 0xFF000000;
                } else {
                    *pixel &= 0x00FFFFFF;
                }
                pixel = pixel.add(1);
                j += 1;
            }
            i += 1;
        }

        (*lpbmi).bmiHeader.biCompression = BI_RGB;

        if SetDIBits(TargetDC, ShortcutIconInfo.hbmColor, 0,
                                    ShortcutBitmapInfo.bmHeight, bits, lpbmi,
                                    DIB_RGB_COLORS) == 0 {
            HeapFree(GetProcessHeap(), 0, bits);
            goto fail;
        }
        HeapFree(GetProcessHeap(), 0, bits);
    }

    if SelectObject(ShortcutDC, ShortcutIconInfo.hbmColor).is_null()
        || SelectObject(TargetDC, TargetIconInfo.hbmColor).is_null() {
        goto fail;
    }
    if MaskBlt(TargetDC, 0, 0,
                               ShortcutBitmapInfo.bmWidth, ShortcutBitmapInfo.bmHeight,
                               ShortcutDC, 0, 0,
                               ShortcutIconInfo.hbmMask, 0, 0,
                               MAKEROP4(0xAA0000, SRCCOPY)) == 0 {
        goto fail;
    }

    SelectObject(TargetDC, OldTargetBitmap);
    DeleteDC(TargetDC);
    SelectObject(ShortcutDC, OldShortcutBitmap);
    DeleteDC(ShortcutDC);

    TargetIcon = CreateIconIndirect(&TargetIconInfo);

    DeleteObject(TargetIconInfo.hbmColor);
    DeleteObject(TargetIconInfo.hbmMask);
    DeleteObject(ShortcutIconInfo.hbmColor);
    DeleteObject(ShortcutIconInfo.hbmMask);
    DestroyIcon(ShortcutIcon);

    return TargetIcon;

fail:
    if !OldTargetBitmap.is_null() {
        SelectObject(TargetDC, OldTargetBitmap);
    }
    if !TargetDC.is_null() {
        DeleteDC(TargetDC);
    }
    if !OldShortcutBitmap.is_null() {
        SelectObject(ShortcutDC, OldShortcutBitmap);
    }
    if !ShortcutDC.is_null() {
        DeleteDC(ShortcutDC);
    }
    if !TargetIconInfo.hbmColor.is_null() {
        DeleteObject(TargetIconInfo.hbmColor);
    }
    if !TargetIconInfo.hbmMask.is_null() {
        DeleteObject(TargetIconInfo.hbmMask);
    }
    if !ShortcutIconInfo.hbmColor.is_null() {
        DeleteObject(ShortcutIconInfo.hbmColor);
    }
    if !ShortcutIconInfo.hbmMask.is_null() {
        DeleteObject(ShortcutIconInfo.hbmMask);
    }
    if !ShortcutIcon.is_null() {
        DestroyIcon(ShortcutIcon);
    }

    core::ptr::null_mut()
}
unsafe extern "C" fn SIC_IconAppend(
    sSourceFile: PCWSTR,
    dwSourceIndex: i32,
    hSmallIcon: HICON,
    hBigIcon: HICON,
    dwFlags: u32,
) -> i32 {
    let mut lpsice: *mut SIC_ENTRY;
    let mut ret: i32;
    let mut index: i32;
    let mut index1: i32;
    let mut indexDPA: i32;
    let mut path: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];

    lpsice = SHAlloc(core::mem::size_of::<SIC_ENTRY>()) as *mut SIC_ENTRY;

    GetFullPathNameW(
        sSourceFile,
        MAX_PATH,
        path.as_mut_ptr(),
        core::ptr::null_mut(),
    );
    (*lpsice).sSourceFile = HeapAlloc(
        GetProcessHeap(),
        0,
        (wcslen(path.as_ptr()) + 1) * core::mem::size_of::<u16>(),
    ) as *mut u16;
    wcscpy((*lpsice).sSourceFile, path.as_ptr());

    (*lpsice).dwSourceIndex = dwSourceIndex as u32;
    (*lpsice).dwFlags = dwFlags;

    EnterCriticalSection(core::ptr::addr_of_mut!(SHELL32_SicCS));

    indexDPA = DPA_Search(
        sic_hdpa,
        lpsice as *mut core::ffi::c_void,
        0,
        SIC_CompareEntries,
        0,
        DPAS_SORTED | DPAS_INSERTAFTER,
    );
    indexDPA = DPA_InsertPtr(sic_hdpa, indexDPA, lpsice as *mut core::ffi::c_void);
    if -1 == indexDPA {
        ret = INVALID_INDEX;
        'leave: loop {
            break 'leave;
        }
    } else {
        index = ImageList_AddIcon(ShellSmallIconList, hSmallIcon);
        index1 = ImageList_AddIcon(ShellBigIconList, hBigIcon);

        if (index == -1) || (index1 == -1) {
            if index != -1 {
                ImageList_Remove(ShellSmallIconList, index);
            }
            if index1 != -1 {
                ImageList_Remove(ShellBigIconList, index1);
            }
            ret = INVALID_INDEX;
            'leave: loop {
                break 'leave;
            }
        } else {
            if index != index1 {
            }
            (*lpsice).dwListIndex = index as u32;
            ret = (*lpsice).dwListIndex as i32;
            'leave: loop {
                break 'leave;
            }
        }
    }

    if ret == INVALID_INDEX {
        if indexDPA != -1 {
            DPA_DeletePtr(sic_hdpa, indexDPA);
        }
        HeapFree(GetProcessHeap(), 0, (*lpsice).sSourceFile as *mut core::ffi::c_void);
        SHFree(lpsice as *mut core::ffi::c_void);
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(SHELL32_SicCS));
    ret
}
unsafe extern "C" fn SIC_LoadIcon(sSourceFile: *const u16, dwSourceIndex: i32, dwFlags: u32) -> i32 {
    let mut hiconLarge: isize = core::ptr::null_mut();
    let mut hiconSmall: isize = core::ptr::null_mut();
    let ret: i32;

    PrivateExtractIconsW(
        sSourceFile,
        dwSourceIndex,
        ShellLargeIconSize,
        ShellLargeIconSize,
        &mut hiconLarge,
        core::ptr::null_mut(),
        1,
        LR_COPYFROMRESOURCE,
    );
    PrivateExtractIconsW(
        sSourceFile,
        dwSourceIndex,
        ShellSmallIconSize,
        ShellSmallIconSize,
        &mut hiconSmall,
        core::ptr::null_mut(),
        1,
        LR_COPYFROMRESOURCE,
    );

    if hiconLarge.is_null() || hiconSmall.is_null() {
        if !hiconLarge.is_null() {
            DestroyIcon(hiconLarge);
        }
        if !hiconSmall.is_null() {
            DestroyIcon(hiconSmall);
        }
        return INVALID_INDEX;
    }

    if 0 != (dwFlags & GIL_FORSHORTCUT) {
        let hiconLargeShortcut = SIC_OverlayShortcutImage(hiconLarge, TRUE);
        let hiconSmallShortcut = SIC_OverlayShortcutImage(hiconSmall, FALSE);
        if !hiconLargeShortcut.is_null() && !hiconSmallShortcut.is_null() {
            DestroyIcon(hiconLarge);
            DestroyIcon(hiconSmall);
            hiconLarge = hiconLargeShortcut;
            hiconSmall = hiconSmallShortcut;
        } else {
            if !hiconLargeShortcut.is_null() {
                DestroyIcon(hiconLargeShortcut);
            }
            if !hiconSmallShortcut.is_null() {
                DestroyIcon(hiconSmallShortcut);
            }
            dwFlags &= !GIL_FORSHORTCUT;
        }
    }

    ret = SIC_IconAppend(sSourceFile, dwSourceIndex, hiconSmall, hiconLarge, dwFlags);
    DestroyIcon(hiconLarge);
    DestroyIcon(hiconSmall);
    ret
}
#[no_mangle]
pub unsafe extern "C" fn SIC_GetIconIndex(
    sSourceFile: *const u16,
    dwSourceIndex: i32,
    dwFlags: u32,
) -> i32 {
    unsafe {
        let mut sice: SIC_ENTRY;
        let mut ret: i32;
        let mut index: i32 = INVALID_INDEX;
        let mut path: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

        GetFullPathNameW(
            sSourceFile,
            MAX_PATH,
            path.as_mut_ptr(),
            core::ptr::null_mut(),
        );
        sice.sSourceFile = path.as_mut_ptr();
        sice.dwSourceIndex = dwSourceIndex;
        sice.dwFlags = dwFlags;

        if sic_hdpa.is_null() {
            SIC_Initialize();
        }

        EnterCriticalSection(core::ptr::addr_of_mut!(SHELL32_SicCS));

        if !DPA_GetPtr(sic_hdpa, 0).is_null() {
            index = DPA_Search(
                sic_hdpa,
                &mut sice as *mut SIC_ENTRY as *mut core::ffi::c_void,
                0,
                SIC_CompareEntries as unsafe extern "C" fn(*mut core::ffi::c_void, *mut core::ffi::c_void, LPARAM) -> i32,
                0,
                DPAS_SORTED,
            );
        }

        if index == INVALID_INDEX {
            ret = SIC_LoadIcon(sSourceFile, dwSourceIndex, dwFlags);
        } else {
            ret = (*(DPA_GetPtr(sic_hdpa, index) as *mut SIC_ENTRY)).dwListIndex;
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!(SHELL32_SicCS));
        ret
    }
}
#[no_mangle]
pub unsafe extern "C" fn SIC_Initialize() -> i32 {
    let mut hSm: HICON = core::ptr::null_mut();
    let mut hLg: HICON = core::ptr::null_mut();
    let bpp: i32;
    let ilMask: u32;
    let mut result: i32 = FALSE;

    if !sic_hdpa.is_null() {
        return TRUE;
    }

    sic_hdpa = DPA_Create(16);
    if sic_hdpa.is_null() {
        return FALSE;
    }

    ShellSmallIconSize = SIC_GetSmallIconSize();
    ShellLargeIconSize = SIC_GetLargeIconSize();

    bpp = SIC_GetIconBPP();
    ShellIconBPP = bpp;

    ilMask = if bpp <= 4 {
        ILC_COLOR4
    } else if bpp <= 8 {
        ILC_COLOR8
    } else if bpp <= 16 {
        ILC_COLOR16
    } else if bpp <= 24 {
        ILC_COLOR24
    } else if bpp <= 32 {
        ILC_COLOR32
    } else {
        ILC_COLOR
    };

    let ilMask = ilMask | ILC_MASK;

    ShellSmallIconList = ImageList_Create(ShellSmallIconSize, ShellSmallIconSize, ilMask, 100, 100);
    if ShellSmallIconList.is_null() {
        result = result;
    } else {
        ShellBigIconList = ImageList_Create(ShellLargeIconSize, ShellLargeIconSize, ilMask, 100, 100);
        if ShellBigIconList.is_null() {
            result = result;
        } else {
            hSm = LoadImageW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_DOCUMENT),
                                IMAGE_ICON, ShellSmallIconSize, ShellSmallIconSize,
                                LR_SHARED | LR_DEFAULTCOLOR) as HICON;
            if hSm.is_null() {
                result = result;
            } else {
                hLg = LoadImageW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_DOCUMENT),
                                IMAGE_ICON, ShellLargeIconSize, ShellLargeIconSize,
                                LR_SHARED | LR_DEFAULTCOLOR) as HICON;
                if hLg.is_null() {
                    result = result;
                } else {
                    if SIC_IconAppend(swShell32Name, IDI_SHELL_DOCUMENT - 1, hSm, hLg, 0) == INVALID_INDEX {
                        result = result;
                    } else if SIC_IconAppend(swShell32Name, -IDI_SHELL_DOCUMENT, hSm, hLg, 0) == INVALID_INDEX {
                        result = result;
                    } else {
                        result = TRUE;
                    }
                }
            }
        }
    }

    if !hSm.is_null() {
        DestroyIcon(hSm);
    }
    if !hLg.is_null() {
        DestroyIcon(hLg);
    }

    if result == FALSE {
        if !sic_hdpa.is_null() {
            DPA_Destroy(sic_hdpa);
        }
        if !ShellSmallIconList.is_null() {
            ImageList_Destroy(ShellSmallIconList);
        }
        if !ShellBigIconList.is_null() {
            ImageList_Destroy(ShellSmallIconList);
        }
        sic_hdpa = core::ptr::null_mut();
        ShellSmallIconList = core::ptr::null_mut();
        ShellBigIconList = core::ptr::null_mut();
    }

    result
}
#[cfg(target_arch = "x86")]
unsafe extern "stdcall" fn sic_free(ptr: *mut core::ffi::c_void, lparam: *mut core::ffi::c_void) -> core::ffi::c_int {
    let _ = lparam;
    let p = ptr as *mut _SIC_ENTRY;
    HeapFree(GetProcessHeap(), 0, (*p).sSourceFile);
    SHFree(ptr);
    return TRUE;
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn sic_free(ptr: *mut core::ffi::c_void, lparam: *mut core::ffi::c_void) -> core::ffi::c_int {
    let _ = lparam;
    let p = ptr as *mut _SIC_ENTRY;
    HeapFree(GetProcessHeap(), 0, (*p).sSourceFile);
    SHFree(ptr);
    return TRUE;
}
#[no_mangle]
pub unsafe extern "C" fn SIC_Destroy() {
    unsafe {
        EnterCriticalSection(core::ptr::addr_of_mut!(SHELL32_SicCS));
        let hdpa = sic_hdpa;
        if !hdpa.is_null() {
            DPA_DestroyCallback(hdpa, sic_free, core::ptr::null_mut());
        }
        sic_hdpa = core::ptr::null_mut();
        ImageList_Destroy(ShellSmallIconList);
        ShellSmallIconList = 0;
        ImageList_Destroy(ShellBigIconList);
        ShellBigIconList = 0;
        LeaveCriticalSection(core::ptr::addr_of_mut!(SHELL32_SicCS));
    }
}
unsafe extern "C" fn SIC_LoadOverlayIcon(icon_idx: i32) -> i32 {
    let mut buffer: [u16; 1024] = [0u16; 1024];
    let iconPath: *const u16;
    let mut iconIdx: i32;

    iconPath = swShell32Name;
    iconIdx = icon_idx;

    if HLM_GetIconW(
        icon_idx,
        buffer.as_mut_ptr(),
        1024,
        &mut iconIdx,
    ) != 0
    {
        iconPath = buffer.as_ptr();
    } else {
        // empty
    }

    if sic_hdpa == 0 {
        SIC_Initialize();
    }

    SIC_LoadIcon(iconPath, iconIdx, 0)
}
#[no_mangle] #[no_mangle]
#[no_mangle] pub unsafe extern "system" fn Shell_GetImageLists(lpBigList: *mut HIMAGELIST, lpSmallList: *mut HIMAGELIST) -> BOOL {
    if sic_hdpa == 0 {
        SIC_Initialize();
    }
    if !lpBigList.is_null() {
        *lpBigList = ShellBigIconList;
    }
    if !lpSmallList.is_null() {
        *lpSmallList = ShellSmallIconList;
    }
    TRUE
}
#[no_mangle]
pub unsafe extern "C" fn PidlToSicIndex(
    sh: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    _bBigIcon: i32,
    uFlags: u32,
    pIndex: *mut i32,
) -> i32 {
    let mut ei: *mut IExtractIconW = core::ptr::null_mut();
    let mut szIconFile: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut iSourceIndex: i32 = 0;
    let mut ret: i32 = FALSE;
    let mut dwFlags: u32 = 0;
    let mut iShortcutDefaultIndex: i32 = INVALID_INDEX;

    if sic_hdpa.is_null() {
        SIC_Initialize();
    }

    let hr: i32 = (*(*sh).GetUIObjectOf)(
        sh,
        0,
        1,
        &pidl,
        IID_NULL_PPV_ARG(IExtractIconW, &mut ei),
    );
    if hr >= 0 {
        let hr2 = (*ei).GetIconLocation(
            uFlags & !GIL_FORSHORTCUT,
            szIconFile.as_mut_ptr(),
            MAX_PATH,
            &mut iSourceIndex,
            &mut dwFlags,
        );
        if hr2 >= 0 {
            *pIndex = SIC_GetIconIndex(szIconFile.as_ptr(), iSourceIndex, uFlags);
            ret = TRUE;
        }
    }

    if INVALID_INDEX == *pIndex {
        if 0 == (uFlags & GIL_FORSHORTCUT) {
            *pIndex = 0;
        } else {
            if INVALID_INDEX == iShortcutDefaultIndex {
                iShortcutDefaultIndex = SIC_LoadIcon(
                    swShell32Name,
                    0,
                    GIL_FORSHORTCUT,
                );
            }
            *pIndex = if INVALID_INDEX != iShortcutDefaultIndex {
                iShortcutDefaultIndex
            } else {
                0
            };
        }
    }

    if !ei.is_null() {
        (*ei).Release();
    }

    ret
}

#[no_mangle]
pub unsafe extern "system" fn SHMapPIDLToSystemImageListIndex(
    sh: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    pIndex: *mut i32,
) -> i32 {
    let mut Index: i32 = 0;
    let mut uGilFlags: u32 = 0;

    if SHELL_IsShortcut(pidl) != 0 {
        uGilFlags |= GIL_FORSHORTCUT;
    }

    if !pIndex.is_null() {
        if PidlToSicIndex(
            sh,
            pidl,
            1,
            uGilFlags,
            pIndex,
        ) == 0
        {
            *pIndex = -1;
        }
    }

    if PidlToSicIndex(
        sh,
        pidl,
        0,
        uGilFlags,
        core::ptr::addr_of_mut!(Index),
    ) == 0
    {
        return -1;
    }

    Index
}

#[no_mangle]
pub unsafe extern "system" fn SHMapIDListToImageListIndexAsync(
    pts: *mut IShellTaskScheduler,
    psf: *mut IShellFolder,
    pidl: LPCITEMIDLIST,
    flags: UINT,
    pfn: PFNASYNCICONTASKBALLBACK,
    pvData: *mut core::ffi::c_void,
    pvHint: *mut core::ffi::c_void,
    piIndex: *mut i32,
    piIndexSel: *mut i32,
) -> HRESULT {
    // FIXME("(%p, %p, %p, 0x%08x, %p, %p, %p, %p, %p)\n", pts, psf, pidl, flags, pfn, pvData, pvHint, piIndex, piIndexSel);
    E_FAIL
}

#[no_mangle]
pub unsafe extern "system" fn Shell_GetCachedImageIndexA(
    szPath: LPCSTR,
    nIndex: INT,
    bSimulateDoc: UINT,
) -> INT {
    let len = MultiByteToWideChar(
        CP_ACP,
        0,
        szPath,
        -1,
        core::ptr::null_mut::<u16>(),
        0,
    );
    let szTemp = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize) * core::mem::size_of::<u16>(),
    ) as LPWSTR;
    MultiByteToWideChar(
        CP_ACP,
        0,
        szPath,
        -1,
        szTemp,
        len,
    );
    let ret = SIC_GetIconIndex(szTemp, nIndex, 0);
    HeapFree(
        GetProcessHeap(),
        0,
        szTemp,
    );
    ret
}

#[no_mangle]
pub unsafe extern "system" fn Shell_GetCachedImageIndexW(
    szPath: PCWSTR,
    nIndex: i32,
    bSimulateDoc: u32,
) -> i32 {
    SIC_GetIconIndex(szPath, nIndex, 0)
}

#[no_mangle]
pub unsafe extern "system" fn Shell_GetCachedImageIndexAW(
    szPath: LPCVOID,
    nIndex: i32,
    bSimulateDoc: i32,
) -> i32 {
    if SHELL_OsIsUnicode() != 0 {
        return Shell_GetCachedImageIndexW(szPath as PCWSTR, nIndex, bSimulateDoc as u32);
    }
    Shell_GetCachedImageIndexA(szPath as LPCSTR, nIndex, bSimulateDoc as u32)
}

#[no_mangle]
pub unsafe extern "system" fn Shell_GetCachedImageIndex(
    szPath: LPCWSTR,
    nIndex: INT,
    bSimulateDoc: UINT,
) -> INT {
    Shell_GetCachedImageIndexAW(szPath, nIndex, bSimulateDoc)
}

#[no_mangle]
pub unsafe extern "system" fn ExtractIconExW(
    lpszFile: PCWSTR,
    nIconIndex: i32,
    phiconLarge: *mut HICON,
    phiconSmall: *mut HICON,
    nIcons: u32,
) -> u32 {
    let mut ret: u32 = 0;
    ret = PrivateExtractIconExW(lpszFile, nIconIndex, phiconLarge, phiconSmall, nIcons);
    if ret == 0xFFFFFFFF {
        ret = 0;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ExtractIconExA(
    lpszFile: *const i8,
    nIconIndex: i32,
    phiconLarge: *mut HICON,
    phiconSmall: *mut HICON,
    nIcons: u32,
) -> u32 {
    let mut ret: u32 = 0;
    let len = MultiByteToWideChar(
        CP_ACP,
        0,
        lpszFile,
        -1,
        core::ptr::null_mut::<u16>(),
        0,
    );
    let lpwstrFile = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize) * core::mem::size_of::<u16>(),
    ) as *mut u16;
    if !lpwstrFile.is_null() {
        MultiByteToWideChar(
            CP_ACP,
            0,
            lpszFile,
            -1,
            lpwstrFile,
            len,
        );
        ret = ExtractIconExW(
            lpwstrFile,
            nIconIndex,
            phiconLarge,
            phiconSmall,
            nIcons,
        );
        HeapFree(
            GetProcessHeap(),
            0,
            lpwstrFile,
        );
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn ExtractAssociatedIconA(
    hInst: HINSTANCE,
    lpIconPath: LPSTR,
    lpiIcon: LPWORD,
) -> HICON {
    let mut hIcon: HICON = core::ptr::null_mut();
    let len = MultiByteToWideChar(
        CP_ACP,
        0,
        lpIconPath,
        -1,
        core::ptr::null_mut(),
        0,
    );
    /* Note that we need to allocate MAX_PATH, since we are supposed to fill
     * the correct executable if there is no icon in lpIconPath directly.
     * lpIconPath itself is supposed to be large enough, so make sure lpIconPathW
     * is large enough too. Yes, I am puking too.
     */
    let lpIconPathW: LPWSTR = HeapAlloc(
        GetProcessHeap(),
        0,
        (MAX_PATH as usize) * core::mem::size_of::<u16>(),
    ) as LPWSTR;

    if !lpIconPathW.is_null() {
        MultiByteToWideChar(
            CP_ACP,
            0,
            lpIconPath,
            -1,
            lpIconPathW,
            len,
        );
        hIcon = ExtractAssociatedIconW(hInst, lpIconPathW, lpiIcon);
        WideCharToMultiByte(
            CP_ACP,
            0,
            lpIconPathW,
            -1,
            lpIconPath,
            MAX_PATH,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        HeapFree(GetProcessHeap(), 0, lpIconPathW);
    }
    hIcon
}

#[no_mangle]
pub unsafe extern "system" fn ExtractAssociatedIconW(hInst: HINSTANCE, lpIconPath: LPWSTR, lpiIcon: LPWORD) -> HICON {
    let mut h_icon: HICON = core::ptr::null_mut();
    let mut w_dummy_icon: u16 = 0;
    let mut lpi_icon = lpiIcon;

    if lpi_icon.is_null() {
        lpi_icon = &mut w_dummy_icon as *mut u16;
    }

    h_icon = ExtractIconW(hInst, lpIconPath, (*lpi_icon) as u32);

    if (h_icon as usize) < (2 as usize) {
        if (h_icon as usize) == (1 as usize) {
            let mut temp_path: [u16; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
            let u_ret: HINSTANCE = FindExecutableW(lpIconPath, core::ptr::null_mut(), temp_path.as_mut_ptr());

            if (u_ret as usize) > (32 as usize) && temp_path[0 as usize] != 0 {
                wcscpy(lpIconPath, temp_path.as_ptr());
                h_icon = ExtractIconW(hInst, lpIconPath, (*lpi_icon) as u32);
                if (h_icon as usize) > (2 as usize) {
                    return h_icon;
                }
            }
        }

        if (h_icon as usize) == (1 as usize) {
            *lpi_icon = 2;
        } else {
            *lpi_icon = 6;
        }

        if GetModuleFileNameW(hInst, lpIconPath, MAX_PATH) != 0 {
            h_icon = LoadIconW(hInst, MAKEINTRESOURCEW((*lpi_icon) as i32));
        }
    }

    h_icon
}

#[no_mangle]
pub unsafe extern "system" fn ExtractAssociatedIconExW(
    hInst: HINSTANCE,
    lpIconPath: LPWSTR,
    lpiIconIdx: LPWORD,
    lpiIconId: LPWORD,
) -> HICON {
    // FIXME("%p %s %p %p): stub\n", hInst, debugstr_w(lpIconPath), lpiIconIdx, lpiIconId);
    0
}

#[no_mangle]
pub unsafe extern "system" fn ExtractAssociatedIconExA(
    hInst: HINSTANCE,
    lpIconPath: LPSTR,
    lpiIconIdx: LPWORD,
    lpiIconId: LPWORD,
) -> HICON {
    let len: i32 = MultiByteToWideChar(
        CP_ACP,
        0,
        lpIconPath,
        -1,
        core::ptr::null_mut(),
        0,
    );
    let lpwstrFile: LPWSTR = HeapAlloc(
        GetProcessHeap(),
        0,
        (len as usize).wrapping_mul(core::mem::size_of::<u16>()),
    ) as LPWSTR;
    MultiByteToWideChar(
        CP_ACP,
        0,
        lpIconPath,
        -1,
        lpwstrFile,
        len,
    );
    let ret = ExtractAssociatedIconExW(hInst, lpwstrFile, lpiIconIdx, lpiIconId);
    HeapFree(GetProcessHeap(), 0, lpwstrFile);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SHDefExtractIconW(
    pszIconFile: PCWSTR,
    iIndex: i32,
    uFlags: u32,
    phiconLarge: *mut HICON,
    phiconSmall: *mut HICON,
    nIconSize: u32,
) -> HRESULT {
    let mut hIcons: [HICON; 2] = core::mem::zeroed();

    let nIconSize = if nIconSize == 0 {
        MAKELONG(
            GetSystemMetrics(SM_CXICON),
            GetSystemMetrics(SM_CXSMICON),
        )
    } else {
        nIconSize
    };

    let ret = PrivateExtractIconsW(
        pszIconFile,
        iIndex,
        nIconSize,
        nIconSize,
        hIcons.as_mut_ptr(),
        core::ptr::null_mut(),
        2,
        LR_DEFAULTCOLOR,
    );
    // FIXME: deal with uFlags parameter which contains GIL_ flags
    if ret == 0xFFFFFFFF {
        return E_FAIL;
    }
    if ret > 0 {
        if !phiconLarge.is_null() {
            *phiconLarge = hIcons[0 as usize];
        } else {
            DestroyIcon(hIcons[0 as usize]);
        }
        if !phiconSmall.is_null() {
            *phiconSmall = hIcons[1 as usize];
        } else {
            DestroyIcon(hIcons[1 as usize]);
        }
        return S_OK;
    }
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn SHDefExtractIconA(
    pszIconFile: PCSTR,
    iIndex: i32,
    uFlags: u32,
    phiconLarge: *mut HICON,
    phiconSmall: *mut HICON,
    nIconSize: u32,
) -> HRESULT {
    let len = MultiByteToWideChar(
        CP_ACP,
        0,
        pszIconFile,
        -1,
        core::ptr::null_mut(),
        0,
    );
    let lpwstrFile = HeapAlloc(
        GetProcessHeap(),
        0,
        len * core::mem::size_of::<u16>() as i32,
    ) as *mut u16;
    MultiByteToWideChar(
        CP_ACP,
        0,
        pszIconFile,
        -1,
        lpwstrFile,
        len,
    );
    let ret = SHDefExtractIconW(
        lpwstrFile as PCWSTR,
        iIndex,
        uFlags,
        phiconLarge,
        phiconSmall,
        nIconSize,
    );
    HeapFree(
        GetProcessHeap(),
        0,
        lpwstrFile,
    );
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SHGetIconOverlayIndexA(pszIconPath: *const i8, iIconIndex: i32) -> i32 {
    -1
}

#[no_mangle]
pub unsafe extern "system" fn SHGetIconOverlayIndexW(pszIconPath: PCWSTR, iIconIndex: i32) -> i32 {
    -1
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS shell32
//  * LICENSE:     LGPL-2.1-or-later (https://spdx.org/licenses/LGPL-2.1-or-later)
//  * PURPOSE:     Shell Icon Cache (SIC)
//  * COPYRIGHT:   Copyright 1998, 1999 Juergen Schmied
//  *              Copyright 2025 Katayama Hirofumi MZ (katayama.hirofumi.mz@gmail.com)
//  */
//
// #include "precomp.h"
//
// WINE_DEFAULT_DEBUG_CHANNEL(shell);
//
// /********************** THE ICON CACHE ********************************/
//
// #define INVALID_INDEX -1
//
// typedef struct
// {
//     LPWSTR sSourceFile;    /* file (not path!) containing the icon */
//     DWORD dwSourceIndex;    /* index within the file, if it is a resoure ID it will be negated */
//     DWORD dwListIndex;    /* index within the iconlist */
//     DWORD dwFlags;        /* GIL_* flags */
//     DWORD dwAccessTime;
// } SIC_ENTRY, * LPSIC_ENTRY;
//
// static HDPA        sic_hdpa = 0;
//
// static HIMAGELIST ShellSmallIconList;
// static HIMAGELIST ShellBigIconList;
// INT ShellSmallIconSize = 0;
// INT ShellLargeIconSize = 0;
// INT ShellIconBPP = 0; // Bits Per Pixel
//
// namespace
// {
// extern CRITICAL_SECTION SHELL32_SicCS;
// CRITICAL_SECTION_DEBUG critsect_debug =
// {
//     0, 0, &SHELL32_SicCS,
//     { &critsect_debug.ProcessLocksList, &critsect_debug.ProcessLocksList },
//       0, 0, { (DWORD_PTR)(__FILE__ ": SHELL32_SicCS") }
// };
// CRITICAL_SECTION SHELL32_SicCS = { &critsect_debug, -1, 0, 0, 0, 0 };
// }
//
// // Load metric value from registry
// static INT
// SIC_GetMetricsValue(
//      PCWSTR pszValueName,
//      INT nDefaultValue)
// {
//     WCHAR szValue[64];
//     DWORD cbValue = sizeof(szValue);
//     DWORD error = SHGetValueW(HKEY_CURRENT_USER, L"Control Panel\\Desktop\\WindowMetrics",
//                               pszValueName, NULL, szValue, &cbValue);
//     if (error)
//         return nDefaultValue;
//     szValue[_countof(szValue) - 1] = UNICODE_NULL; // Avoid buffer overrun
//     return _wtoi(szValue);
// }
//
// static INT
// SIC_GetLargeIconSize(VOID)
// {
//     // NOTE: Shell icon size is always square
//     INT nDefaultSize = GetSystemMetrics(SM_CXICON);
//     INT nIconSize = SIC_GetMetricsValue(L"Shell Icon Size", nDefaultSize);
//     return (nIconSize > 0) ? nIconSize : nDefaultSize;
// }
//
// static INT
// SIC_GetSmallIconSize(VOID)
// {
//     // NOTE: Shell icon size is always square
//     INT nDefaultSize = GetSystemMetrics(SM_CXSMICON);
//     INT nIconSize = SIC_GetMetricsValue(L"Shell Small Icon Size", nDefaultSize);
//     return (nIconSize > 0) ? nIconSize : nDefaultSize;
// }
//
// static INT
// SIC_GetIconBPP(VOID) // Bits Per Pixel
// {
//     INT nDefaultBPP = SHGetCurColorRes();
//     INT nIconBPP = SIC_GetMetricsValue(L"Shell Icon BPP", nDefaultBPP);
//     return (nIconBPP > 0) ? nIconBPP : nDefaultBPP;
// }
//
// /*****************************************************************************
//  * SIC_CompareEntries
//  *
//  * NOTES
//  *  Callback for DPA_Search
//  */
// static INT __stdcall SIC_CompareEntries( LPVOID p1, LPVOID p2, LPARAM lparam)
// {    LPSIC_ENTRY e1 = (LPSIC_ENTRY)p1, e2 = (LPSIC_ENTRY)p2;
//
//     TRACE("%p %p %8lx\n", p1, p2, lparam);
//
//     /* Icons in the cache are keyed by the name of the file they are
//      * loaded from, their resource index and the fact if they have a shortcut
//      * icon overlay or not.
//      */
//     /* first the faster one */
//     if (e1->dwSourceIndex != e2->dwSourceIndex)
//       return (e1->dwSourceIndex < e2->dwSourceIndex) ? -1 : 1;
//
//     if ((e1->dwFlags & GIL_FORSHORTCUT) != (e2->dwFlags & GIL_FORSHORTCUT))
//       return ((e1->dwFlags & GIL_FORSHORTCUT) < (e2->dwFlags & GIL_FORSHORTCUT)) ? -1 : 1;
//
//     return _wcsicmp(e1->sSourceFile,e2->sSourceFile);
// }
//
// /* declare SIC_LoadOverlayIcon() */
// static int SIC_LoadOverlayIcon(int icon_idx);
//
// /*****************************************************************************
//  * SIC_OverlayShortcutImage            [internal]
//  *
//  * NOTES
//  *  Creates a new icon as a copy of the passed-in icon, overlayed with a
//  *  shortcut image.
//  * FIXME: This should go to the ImageList implementation!
//  */
// static HICON SIC_OverlayShortcutImage(HICON SourceIcon, BOOL large)
// {
//     ICONINFO ShortcutIconInfo, TargetIconInfo;
//     HICON ShortcutIcon = NULL, TargetIcon;
//     BITMAP TargetBitmapInfo, ShortcutBitmapInfo;
//     HDC ShortcutDC = NULL,
//       TargetDC = NULL;
//     HBITMAP OldShortcutBitmap = NULL,
//       OldTargetBitmap = NULL;
//
//     static int s_imgListIdx = -1;
//     ZeroMemory(&ShortcutIconInfo, sizeof(ShortcutIconInfo));
//     ZeroMemory(&TargetIconInfo, sizeof(TargetIconInfo));
//
//     /* Get information about the source icon and shortcut overlay.
//      * We will write over the source bitmaps to get the final ones */
//     if (! GetIconInfo(SourceIcon, &TargetIconInfo))
//         return NULL;
//
//     /* Is it possible with the ImageList implementation? */
//     if(!TargetIconInfo.hbmColor)
//     {
//         /* Maybe we'll support this at some point */
//         FIXME("1bpp icon wants its overlay!\n");
//         goto fail;
//     }
//
//     if(!GetObjectW(TargetIconInfo.hbmColor, sizeof(BITMAP), &TargetBitmapInfo))
//     {
//         goto fail;
//     }
//
//     /* search for the shortcut icon only once */
//     if (s_imgListIdx == -1)
//         s_imgListIdx = SIC_LoadOverlayIcon(IDI_SHELL_SHORTCUT - 1);
//
//     if (s_imgListIdx != -1)
//     {
//         if (large)
//             ShortcutIcon = ImageList_GetIcon(ShellBigIconList, s_imgListIdx, ILD_TRANSPARENT);
//         else
//             ShortcutIcon = ImageList_GetIcon(ShellSmallIconList, s_imgListIdx, ILD_TRANSPARENT);
//     } else
//         ShortcutIcon = NULL;
//
//     if (!ShortcutIcon || !GetIconInfo(ShortcutIcon, &ShortcutIconInfo))
//     {
//         goto fail;
//     }
//
//     /* Is it possible with the ImageLists ? */
//     if(!ShortcutIconInfo.hbmColor)
//     {
//         /* Maybe we'll support this at some point */
//         FIXME("Should draw 1bpp overlay!\n");
//         goto fail;
//     }
//
//     if(!GetObjectW(ShortcutIconInfo.hbmColor, sizeof(BITMAP), &ShortcutBitmapInfo))
//     {
//         goto fail;
//     }
//
//     /* Setup the masks */
//     ShortcutDC = CreateCompatibleDC(NULL);
//     if (NULL == ShortcutDC) goto fail;
//     OldShortcutBitmap = (HBITMAP)SelectObject(ShortcutDC, ShortcutIconInfo.hbmMask);
//     if (NULL == OldShortcutBitmap) goto fail;
//
//     TargetDC = CreateCompatibleDC(NULL);
//     if (NULL == TargetDC) goto fail;
//     OldTargetBitmap = (HBITMAP)SelectObject(TargetDC, TargetIconInfo.hbmMask);
//     if (NULL == OldTargetBitmap) goto fail;
//
//     /* Create the complete mask by ANDing the source and shortcut masks.
//      * NOTE: in an ImageList, all icons have the same dimensions */
//     if (!BitBlt(TargetDC, 0, 0, ShortcutBitmapInfo.bmWidth, ShortcutBitmapInfo.bmHeight,
//                 ShortcutDC, 0, 0, SRCAND))
//     {
//       goto fail;
//     }
//
//     /*
//      * We must remove or add the alpha component to the shortcut overlay:
//      * If we don't, SRCCOPY will copy it to our resulting icon, resulting in a
//      * partially transparent icons where it shouldn't be, and to an invisible icon
//      * if the underlying icon don't have any alpha channel information. (16bpp only icon for instance).
//      * But if the underlying icon has alpha channel information, then we must mark the overlay information
//      * as opaque.
//      * NOTE: This code sucks(tm) and should belong to the ImageList implementation.
//      * NOTE2: there are better ways to do this.
//      */
//     if(ShortcutBitmapInfo.bmBitsPixel == 32)
//     {
//         BOOL add_alpha;
//         BYTE buffer[sizeof(BITMAPINFO) + 256 * sizeof(RGBQUAD)];
//         BITMAPINFO* lpbmi = (BITMAPINFO*)buffer;
//         PVOID bits;
//         PULONG pixel;
//         INT i, j;
//
//         /* Find if the source bitmap has an alpha channel */
//         if(TargetBitmapInfo.bmBitsPixel != 32) add_alpha = FALSE;
//         else
//         {
//             ZeroMemory(buffer, sizeof(buffer));
//             lpbmi->bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
//             lpbmi->bmiHeader.biWidth = TargetBitmapInfo.bmWidth;
//             lpbmi->bmiHeader.biHeight = TargetBitmapInfo.bmHeight;
//             lpbmi->bmiHeader.biPlanes = 1;
//             lpbmi->bmiHeader.biBitCount = 32;
//
//             bits = HeapAlloc(GetProcessHeap(), 0, TargetBitmapInfo.bmHeight * TargetBitmapInfo.bmWidthBytes);
//
//             if(!bits) goto fail;
//
//             if(!GetDIBits(TargetDC, TargetIconInfo.hbmColor, 0, TargetBitmapInfo.bmHeight, bits, lpbmi, DIB_RGB_COLORS))
//             {
//                 ERR("GetBIBits failed!\n");
//                 HeapFree(GetProcessHeap(), 0, bits);
//                 goto fail;
//             }
//
//             i = j = 0;
//             pixel = (PULONG)bits;
//
//             for(i=0; i<TargetBitmapInfo.bmHeight; i++)
//             {
//                 for(j=0; j<TargetBitmapInfo.bmWidth; j++)
//                 {
//                     add_alpha = (*pixel++ & 0xFF000000) != 0;
//                     if(add_alpha) break;
//                 }
//                 if(add_alpha) break;
//             }
//             HeapFree(GetProcessHeap(), 0, bits);
//         }
//
//         /* Allocate the bits */
//         bits = HeapAlloc(GetProcessHeap(), 0, ShortcutBitmapInfo.bmHeight*ShortcutBitmapInfo.bmWidthBytes);
//         if(!bits) goto fail;
//
//         ZeroMemory(buffer, sizeof(buffer));
//         lpbmi->bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
//         lpbmi->bmiHeader.biWidth = ShortcutBitmapInfo.bmWidth;
//         lpbmi->bmiHeader.biHeight = ShortcutBitmapInfo.bmHeight;
//         lpbmi->bmiHeader.biPlanes = 1;
//         lpbmi->bmiHeader.biBitCount = 32;
//
//         if(!GetDIBits(TargetDC, ShortcutIconInfo.hbmColor, 0, ShortcutBitmapInfo.bmHeight, bits, lpbmi, DIB_RGB_COLORS))
//         {
//             ERR("GetBIBits failed!\n");
//             HeapFree(GetProcessHeap(), 0, bits);
//             goto fail;
//         }
//
//         pixel = (PULONG)bits;
//         /* Remove alpha channel component or make it totally opaque */
//         for(i=0; i<ShortcutBitmapInfo.bmHeight; i++)
//         {
//             for(j=0; j<ShortcutBitmapInfo.bmWidth; j++)
//             {
//                 if(add_alpha) *pixel++ |= 0xFF000000;
//                 else *pixel++ &= 0x00FFFFFF;
//             }
//         }
//
//         /* GetDIBits return BI_BITFIELDS with masks set to 0, and SetDIBits fails when masks are 0. The irony... */
//         lpbmi->bmiHeader.biCompression = BI_RGB;
//
//         /* Set the bits again */
//         if(!SetDIBits(TargetDC, ShortcutIconInfo.hbmColor, 0, ShortcutBitmapInfo.bmHeight, bits, lpbmi, DIB_RGB_COLORS))
//         {
//             ERR("SetBIBits failed!, %lu\n", GetLastError());
//             HeapFree(GetProcessHeap(), 0, bits);
//             goto fail;
//         }
//         HeapFree(GetProcessHeap(), 0, bits);
//     }
//
//     /* Now do the copy. We overwrite the original icon data */
//     if (NULL == SelectObject(ShortcutDC, ShortcutIconInfo.hbmColor) ||
//         NULL == SelectObject(TargetDC, TargetIconInfo.hbmColor))
//         goto fail;
//     if (!MaskBlt(TargetDC, 0, 0, ShortcutBitmapInfo.bmWidth, ShortcutBitmapInfo.bmHeight,
//                  ShortcutDC, 0, 0, ShortcutIconInfo.hbmMask, 0, 0,
//                  MAKEROP4(0xAA0000, SRCCOPY)))
//     {
//         goto fail;
//     }
//
//     /* Clean up, we're not goto'ing to 'fail' after this so we can be lazy and not set
//        handles to NULL */
//     SelectObject(TargetDC, OldTargetBitmap);
//     DeleteDC(TargetDC);
//     SelectObject(ShortcutDC, OldShortcutBitmap);
//     DeleteDC(ShortcutDC);
//
//     /* Create the icon using the bitmaps prepared earlier */
//     TargetIcon = CreateIconIndirect(&TargetIconInfo);
//
//     /* CreateIconIndirect copies the bitmaps, so we can release our bitmaps now */
//     DeleteObject(TargetIconInfo.hbmColor);
//     DeleteObject(TargetIconInfo.hbmMask);
//     /* Delete what GetIconInfo gave us */
//     DeleteObject(ShortcutIconInfo.hbmColor);
//     DeleteObject(ShortcutIconInfo.hbmMask);
//     DestroyIcon(ShortcutIcon);
//
//     return TargetIcon;
//
// fail:
//     /* Clean up scratch resources we created */
//     if (NULL != OldTargetBitmap) SelectObject(TargetDC, OldTargetBitmap);
//     if (NULL != TargetDC) DeleteDC(TargetDC);
//     if (NULL != OldShortcutBitmap) SelectObject(ShortcutDC, OldShortcutBitmap);
//     if (NULL != ShortcutDC) DeleteDC(ShortcutDC);
//     if (NULL != TargetIconInfo.hbmColor) DeleteObject(TargetIconInfo.hbmColor);
//     if (NULL != TargetIconInfo.hbmMask) DeleteObject(TargetIconInfo.hbmMask);
//     if (NULL != ShortcutIconInfo.hbmColor) DeleteObject(ShortcutIconInfo.hbmColor);
//     if (NULL != ShortcutIconInfo.hbmMask) DeleteObject(ShortcutIconInfo.hbmMask);
//     if (NULL != ShortcutIcon) DestroyIcon(ShortcutIcon);
//
//     return NULL;
// }
//
// /*****************************************************************************
//  * SIC_IconAppend            [internal]
//  *
//  * NOTES
//  *  appends an icon pair to the end of the cache
//  */
// static INT SIC_IconAppend (LPCWSTR sSourceFile, INT dwSourceIndex, HICON hSmallIcon, HICON hBigIcon, DWORD dwFlags)
// {
//     LPSIC_ENTRY lpsice;
//     INT ret, index, index1, indexDPA;
//     WCHAR path[MAX_PATH];
//     TRACE("%s %i %p %p\n", debugstr_w(sSourceFile), dwSourceIndex, hSmallIcon ,hBigIcon);
//
//     lpsice = (LPSIC_ENTRY) SHAlloc (sizeof (SIC_ENTRY));
//
//     GetFullPathNameW(sSourceFile, MAX_PATH, path, NULL);
//     lpsice->sSourceFile = (LPWSTR)HeapAlloc( GetProcessHeap(), 0, (wcslen(path)+1)*sizeof(WCHAR) );
//     wcscpy( lpsice->sSourceFile, path );
//
//     lpsice->dwSourceIndex = dwSourceIndex;
//     lpsice->dwFlags = dwFlags;
//
//     EnterCriticalSection(&SHELL32_SicCS);
//
//     indexDPA = DPA_Search (sic_hdpa, lpsice, 0, SIC_CompareEntries, 0, DPAS_SORTED|DPAS_INSERTAFTER);
//     indexDPA = DPA_InsertPtr(sic_hdpa, indexDPA, lpsice);
//     if ( -1 == indexDPA )
//     {
//         ret = INVALID_INDEX;
//         goto leave;
//     }
//
//     index = ImageList_AddIcon (ShellSmallIconList, hSmallIcon);
//     index1= ImageList_AddIcon (ShellBigIconList, hBigIcon);
//
//     /* Something went wrong when allocating a new image in the list. Abort. */
//     if((index == -1) || (index1 == -1))
//     {
//         WARN("Something went wrong when adding the icon to the list: small - 0x%x, big - 0x%x.\n",
//             index, index1);
//         if(index != -1) ImageList_Remove(ShellSmallIconList, index);
//         if(index1 != -1) ImageList_Remove(ShellBigIconList, index1);
//         ret = INVALID_INDEX;
//         goto leave;
//     }
//
//     if (index!=index1)
//     {
//         FIXME("iconlists out of sync 0x%x 0x%x\n", index, index1);
//         /* What to do ???? */
//     }
//     lpsice->dwListIndex = index;
//     ret = lpsice->dwListIndex;
//
// leave:
//     if(ret == INVALID_INDEX)
//     {
//         if(indexDPA != -1) DPA_DeletePtr(sic_hdpa, indexDPA);
//         HeapFree(GetProcessHeap(), 0, lpsice->sSourceFile);
//         SHFree(lpsice);
//     }
//     LeaveCriticalSection(&SHELL32_SicCS);
//     return ret;
// }
// /****************************************************************************
//  * SIC_LoadIcon                [internal]
//  *
//  * NOTES
//  *  gets small/big icon by number from a file
//  */
// static INT SIC_LoadIcon (LPCWSTR sSourceFile, INT dwSourceIndex, DWORD dwFlags)
// {
//     HICON hiconLarge = NULL, hiconSmall = NULL;
//     UINT ret;
//
//     PrivateExtractIconsW(sSourceFile, dwSourceIndex, ShellLargeIconSize, ShellLargeIconSize,
//                          &hiconLarge, NULL, 1, LR_COPYFROMRESOURCE);
//     PrivateExtractIconsW(sSourceFile, dwSourceIndex, ShellSmallIconSize, ShellSmallIconSize,
//                          &hiconSmall, NULL, 1, LR_COPYFROMRESOURCE);
//
//     if (!hiconLarge || !hiconSmall)
//     {
//         WARN("failure loading icon %i from %s (%p %p)\n", dwSourceIndex, debugstr_w(sSourceFile), hiconLarge, hiconSmall);
//         if(hiconLarge) DestroyIcon(hiconLarge);
//         if(hiconSmall) DestroyIcon(hiconSmall);
//         return INVALID_INDEX;
//     }
//
//     if (0 != (dwFlags & GIL_FORSHORTCUT))
//     {
//         HICON hiconLargeShortcut = SIC_OverlayShortcutImage(hiconLarge, TRUE);
//         HICON hiconSmallShortcut = SIC_OverlayShortcutImage(hiconSmall, FALSE);
//         if (NULL != hiconLargeShortcut && NULL != hiconSmallShortcut)
//         {
//             DestroyIcon(hiconLarge);
//             DestroyIcon(hiconSmall);
//             hiconLarge = hiconLargeShortcut;
//             hiconSmall = hiconSmallShortcut;
//         }
//         else
//         {
//             WARN("Failed to create shortcut overlayed icons\n");
//             if (NULL != hiconLargeShortcut) DestroyIcon(hiconLargeShortcut);
//             if (NULL != hiconSmallShortcut) DestroyIcon(hiconSmallShortcut);
//             dwFlags &= ~ GIL_FORSHORTCUT;
//         }
//     }
//
//     ret = SIC_IconAppend (sSourceFile, dwSourceIndex, hiconSmall, hiconLarge, dwFlags);
//     DestroyIcon(hiconLarge);
//     DestroyIcon(hiconSmall);
//     return ret;
// }
// /*****************************************************************************
//  * SIC_GetIconIndex            [internal]
//  *
//  * Parameters
//  *    sSourceFile    []    filename of file containing the icon
//  *    index        []    index/resID (negated) in this file
//  *
//  * NOTES
//  *  look in the cache for a proper icon. if not available the icon is taken
//  *  from the file and cached
//  */
// INT SIC_GetIconIndex (LPCWSTR sSourceFile, INT dwSourceIndex, DWORD dwFlags )
// {
//     SIC_ENTRY sice;
//     INT ret, index = INVALID_INDEX;
//     WCHAR path[MAX_PATH];
//
//     TRACE("%s %i\n", debugstr_w(sSourceFile), dwSourceIndex);
//
//     GetFullPathNameW(sSourceFile, MAX_PATH, path, NULL);
//     sice.sSourceFile = path;
//     sice.dwSourceIndex = dwSourceIndex;
//     sice.dwFlags = dwFlags;
//
//     if (!sic_hdpa)
//         SIC_Initialize();
//
//     EnterCriticalSection(&SHELL32_SicCS);
//
//     if (NULL != DPA_GetPtr (sic_hdpa, 0))
//     {
//       /* search linear from position 0*/
//       index = DPA_Search (sic_hdpa, &sice, 0, SIC_CompareEntries, 0, DPAS_SORTED);
//     }
//
//     if ( INVALID_INDEX == index )
//     {
//           ret = SIC_LoadIcon (sSourceFile, dwSourceIndex, dwFlags);
//     }
//     else
//     {
//       TRACE("-- found\n");
//       ret = ((LPSIC_ENTRY)DPA_GetPtr(sic_hdpa, index))->dwListIndex;
//     }
//
//     LeaveCriticalSection(&SHELL32_SicCS);
//     return ret;
// }
//
// /*****************************************************************************
//  * SIC_Initialize            [internal]
//  */
// BOOL SIC_Initialize(void)
// {
//     HICON hSm = NULL, hLg = NULL;
//     INT bpp;
//     DWORD ilMask;
//     BOOL result = FALSE;
//
//     TRACE("Entered SIC_Initialize\n");
//
//     if (sic_hdpa)
//     {
//         TRACE("Icon cache already initialized\n");
//         return TRUE;
//     }
//
//     sic_hdpa = DPA_Create(16);
//     if (!sic_hdpa)
//     {
//         return FALSE;
//     }
//
//     ShellSmallIconSize = SIC_GetSmallIconSize();
//     ShellLargeIconSize = SIC_GetLargeIconSize();
//
//     bpp = ShellIconBPP = SIC_GetIconBPP(); // Bits Per Pixel
//     if (bpp <= 4)
//         ilMask = ILC_COLOR4;
//     else if (bpp <= 8)
//         ilMask = ILC_COLOR8;
//     else if (bpp <= 16)
//         ilMask = ILC_COLOR16;
//     else if (bpp <= 24)
//         ilMask = ILC_COLOR24;
//     else if (bpp <= 32)
//         ilMask = ILC_COLOR32;
//     else
//         ilMask = ILC_COLOR;
//
//     ilMask |= ILC_MASK;
//
//     ShellSmallIconList = ImageList_Create(ShellSmallIconSize, ShellSmallIconSize, ilMask, 100, 100);
//     if (!ShellSmallIconList)
//     {
//         ERR("Failed to create the small icon list.\n");
//         goto end;
//     }
//
//     ShellBigIconList = ImageList_Create(ShellLargeIconSize, ShellLargeIconSize, ilMask, 100, 100);
//     if (!ShellBigIconList)
//     {
//         ERR("Failed to create the big icon list.\n");
//         goto end;
//     }
//
//     /* Load the document icon, which is used as the default if an icon isn't found. */
//     hSm = (HICON)LoadImageW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_DOCUMENT),
//                             IMAGE_ICON, ShellSmallIconSize, ShellSmallIconSize,
//                             LR_SHARED | LR_DEFAULTCOLOR);
//     if (!hSm)
//     {
//         ERR("Failed to load small IDI_SHELL_DOCUMENT icon!\n");
//         goto end;
//     }
//
//     hLg = (HICON)LoadImageW(shell32_hInstance, MAKEINTRESOURCEW(IDI_SHELL_DOCUMENT),
//                             IMAGE_ICON, ShellLargeIconSize, ShellLargeIconSize,
//                             LR_SHARED | LR_DEFAULTCOLOR);
//     if (!hLg)
//     {
//         ERR("Failed to load large IDI_SHELL_DOCUMENT icon!\n");
//         goto end;
//     }
//
//     if(SIC_IconAppend(swShell32Name, IDI_SHELL_DOCUMENT-1, hSm, hLg, 0) == INVALID_INDEX)
//     {
//         ERR("Failed to add IDI_SHELL_DOCUMENT icon to cache.\n");
//         goto end;
//     }
//     if(SIC_IconAppend(swShell32Name, -IDI_SHELL_DOCUMENT, hSm, hLg, 0) == INVALID_INDEX)
//     {
//         ERR("Failed to add IDI_SHELL_DOCUMENT icon to cache.\n");
//         goto end;
//     }
//
//     /* Everything went fine */
//     result = TRUE;
//
// end:
//     /* The image list keeps a copy of the icons, we must destroy them */
//     if(hSm) DestroyIcon(hSm);
//     if(hLg) DestroyIcon(hLg);
//
//     /* Clean everything if something went wrong */
//     if(!result)
//     {
//         if(sic_hdpa) DPA_Destroy(sic_hdpa);
//         if(ShellSmallIconList) ImageList_Destroy(ShellSmallIconList);
//         if(ShellBigIconList) ImageList_Destroy(ShellSmallIconList);
//         sic_hdpa = NULL;
//         ShellSmallIconList = NULL;
//         ShellBigIconList = NULL;
//     }
//
//     TRACE("hIconSmall=%p hIconBig=%p\n",ShellSmallIconList, ShellBigIconList);
//
//     return result;
// }
//
// /*************************************************************************
//  * SIC_Destroy
//  *
//  * frees the cache
//  */
// static INT __stdcall sic_free( LPVOID ptr, LPVOID lparam )
// {
//     HeapFree(GetProcessHeap(), 0, ((LPSIC_ENTRY)ptr)->sSourceFile);
//     SHFree(ptr);
//     return TRUE;
// }
//
// void SIC_Destroy(void)
// {
//     TRACE("\n");
//
//     EnterCriticalSection(&SHELL32_SicCS);
//
//     if (sic_hdpa) DPA_DestroyCallback(sic_hdpa, sic_free, NULL );
//
//     sic_hdpa = NULL;
//     ImageList_Destroy(ShellSmallIconList);
//     ShellSmallIconList = 0;
//     ImageList_Destroy(ShellBigIconList);
//     ShellBigIconList = 0;
//
//     LeaveCriticalSection(&SHELL32_SicCS);
//     //DeleteCriticalSection(&SHELL32_SicCS); //static
// }
//
// /*****************************************************************************
//  * SIC_LoadOverlayIcon            [internal]
//  *
//  * Load a shell overlay icon and return its icon cache index.
//  */
// static int SIC_LoadOverlayIcon(int icon_idx)
// {
//     WCHAR buffer[1024];
//     LPWSTR iconPath;
//     int iconIdx;
//
//     iconPath = swShell32Name;    /* default: load icon from shell32.dll */
//     iconIdx = icon_idx;
//
//     if (HLM_GetIconW(icon_idx, buffer, _countof(buffer), &iconIdx))
//     {
//         iconPath = buffer;
//     }
//     else
//     {
//         WARN("Failed to load icon with index %d, using default one\n", icon_idx);
//     }
//
//     if (!sic_hdpa)
//         SIC_Initialize();
//
//     return SIC_LoadIcon(iconPath, iconIdx, 0);
// }
//
// /*************************************************************************
//  * Shell_GetImageLists            [SHELL32.71]
//  *
//  * PARAMETERS
//  *  imglist[1|2] [] pointer which receives imagelist handles
//  *
//  */
// BOOL WINAPI Shell_GetImageLists(HIMAGELIST * lpBigList, HIMAGELIST * lpSmallList)
// {
//     TRACE("(%p,%p)\n",lpBigList,lpSmallList);
//
//     if (!sic_hdpa)
//         SIC_Initialize();
//
//     if (lpBigList)
//         *lpBigList = ShellBigIconList;
//
//     if (lpSmallList)
//         *lpSmallList = ShellSmallIconList;
//
//     return TRUE;
// }
// /*************************************************************************
//  * PidlToSicIndex            [INTERNAL]
//  *
//  * PARAMETERS
//  *    sh    []    IShellFolder
//  *    pidl    []
//  *    bBigIcon []
//  *    uFlags    []    GIL_*
//  *    pIndex    []    index within the SIC
//  *
//  */
// BOOL PidlToSicIndex (
//     IShellFolder * sh,
//     LPCITEMIDLIST pidl,
//     BOOL bBigIcon,
//     UINT uFlags,
//     int * pIndex)
// {
//     CComPtr<IExtractIconW>        ei;
//     WCHAR        szIconFile[MAX_PATH];    /* file containing the icon */
//     INT        iSourceIndex;        /* index or resID(negated) in this file */
//     BOOL        ret = FALSE;
//     UINT        dwFlags = 0;
//     int        iShortcutDefaultIndex = INVALID_INDEX;
//
//     TRACE("sf=%p pidl=%p %s\n", sh, pidl, bBigIcon?"Big":"Small");
//
//     if (!sic_hdpa)
//         SIC_Initialize();
//
//     if (SUCCEEDED (sh->GetUIObjectOf(0, 1, &pidl, IID_NULL_PPV_ARG(IExtractIconW, &ei))))
//     {
//       if (SUCCEEDED(ei->GetIconLocation(uFlags &~ GIL_FORSHORTCUT, szIconFile, MAX_PATH, &iSourceIndex, &dwFlags)))
//       {
//         *pIndex = SIC_GetIconIndex(szIconFile, iSourceIndex, uFlags);
//         ret = TRUE;
//       }
//     }
//
//     if (INVALID_INDEX == *pIndex)    /* default icon when failed */
//     {
//       if (0 == (uFlags & GIL_FORSHORTCUT))
//       {
//         *pIndex = 0;
//       }
//       else
//       {
//         if (INVALID_INDEX == iShortcutDefaultIndex)
//         {
//           iShortcutDefaultIndex = SIC_LoadIcon(swShell32Name, 0, GIL_FORSHORTCUT);
//         }
//         *pIndex = (INVALID_INDEX != iShortcutDefaultIndex ? iShortcutDefaultIndex : 0);
//       }
//     }
//
//     return ret;
//
// }
//
// /*************************************************************************
//  * SHMapPIDLToSystemImageListIndex    [SHELL32.77]
//  *
//  * PARAMETERS
//  *    sh    []        pointer to an instance of IShellFolder
//  *    pidl    []
//  *    pIndex    [][]    SIC index for big icon
//  *
//  */
// int WINAPI SHMapPIDLToSystemImageListIndex(
//     IShellFolder *sh,
//     LPCITEMIDLIST pidl,
//     int *pIndex)
// {
//     int Index;
//     UINT uGilFlags = 0;
//
//     TRACE("(SF=%p,pidl=%p,%p)\n",sh,pidl,pIndex);
//     pdump(pidl);
//
//     if (SHELL_IsShortcut(pidl))
//         uGilFlags |= GIL_FORSHORTCUT;
//
//     if (pIndex)
//         if (!PidlToSicIndex ( sh, pidl, 1, uGilFlags, pIndex))
//             *pIndex = -1;
//
//     if (!PidlToSicIndex ( sh, pidl, 0, uGilFlags, &Index))
//         return -1;
//
//     return Index;
// }
//
// /*************************************************************************
//  * SHMapIDListToImageListIndexAsync  [SHELL32.148]
//  */
// extern "C" HRESULT WINAPI SHMapIDListToImageListIndexAsync(IShellTaskScheduler *pts, IShellFolder *psf,
//                                                 LPCITEMIDLIST pidl, UINT flags,
//                                                 PFNASYNCICONTASKBALLBACK pfn, void *pvData, void *pvHint,
//                                                 int *piIndex, int *piIndexSel)
// {
//     FIXME("(%p, %p, %p, 0x%08x, %p, %p, %p, %p, %p)\n",
//             pts, psf, pidl, flags, pfn, pvData, pvHint, piIndex, piIndexSel);
//     return E_FAIL;
// }
//
// /*************************************************************************
//  * Shell_GetCachedImageIndex        [SHELL32.72]
//  *
//  */
// INT WINAPI Shell_GetCachedImageIndexA(LPCSTR szPath, INT nIndex, UINT bSimulateDoc)
// {
//     INT ret, len;
//     LPWSTR szTemp;
//
//     WARN("(%s,%08x,%08x) semi-stub.\n",debugstr_a(szPath), nIndex, bSimulateDoc);
//
//     len = MultiByteToWideChar( CP_ACP, 0, szPath, -1, NULL, 0 );
//     szTemp = (LPWSTR)HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) );
//     MultiByteToWideChar( CP_ACP, 0, szPath, -1, szTemp, len );
//
//     ret = SIC_GetIconIndex( szTemp, nIndex, 0 );
//
//     HeapFree( GetProcessHeap(), 0, szTemp );
//
//     return ret;
// }
//
// INT WINAPI Shell_GetCachedImageIndexW(LPCWSTR szPath, INT nIndex, UINT bSimulateDoc)
// {
//     WARN("(%s,%08x,%08x) semi-stub.\n",debugstr_w(szPath), nIndex, bSimulateDoc);
//
//     return SIC_GetIconIndex(szPath, nIndex, 0);
// }
//
// extern "C" INT WINAPI Shell_GetCachedImageIndexAW(LPCVOID szPath, INT nIndex, BOOL bSimulateDoc)
// {    if( SHELL_OsIsUnicode())
//       return Shell_GetCachedImageIndexW((LPCWSTR)szPath, nIndex, bSimulateDoc);
//     return Shell_GetCachedImageIndexA((LPCSTR)szPath, nIndex, bSimulateDoc);
// }
//
// extern "C" INT WINAPI Shell_GetCachedImageIndex(LPCWSTR szPath, INT nIndex, UINT bSimulateDoc)
// {
//     return Shell_GetCachedImageIndexAW(szPath, nIndex, bSimulateDoc);
// }
//
// /*************************************************************************
//  * ExtractIconExW            [SHELL32.@]
//  * RETURNS
//  *  0 no icon found (or the file is not valid)
//  *  or number of icons extracted
//  */
// UINT WINAPI ExtractIconExW(LPCWSTR lpszFile, INT nIconIndex, HICON * phiconLarge, HICON * phiconSmall, UINT nIcons)
// {
//     UINT ret = 0;
//
//     /* get entry point of undocumented function PrivateExtractIconExW() in user32 */
//
//     TRACE("%s %i %p %p %i\n", debugstr_w(lpszFile), nIconIndex, phiconLarge, phiconSmall, nIcons);
//     ret = PrivateExtractIconExW(lpszFile, nIconIndex, phiconLarge, phiconSmall, nIcons);
//
//     /* PrivateExtractIconExW() may return -1 if the provided file is not a valid PE image file or the said
//      * file couldn't be found. The behaviour is correct although ExtractIconExW() only returns the successfully
//      * extracted icons from a file. In such scenario, simply return 0.
//     */
//     if (ret == 0xFFFFFFFF)
//     {
//         WARN("Invalid file or couldn't be found - %s\n", debugstr_w(lpszFile));
//         ret = 0;
//     }
//
//     return ret;
// }
//
// /*************************************************************************
//  * ExtractIconExA            [SHELL32.@]
//  */
// UINT WINAPI ExtractIconExA(LPCSTR lpszFile, INT nIconIndex, HICON * phiconLarge, HICON * phiconSmall, UINT nIcons)
// {
//     UINT ret = 0;
//     INT len = MultiByteToWideChar(CP_ACP, 0, lpszFile, -1, NULL, 0);
//     LPWSTR lpwstrFile = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//
//     TRACE("%s %i %p %p %i\n", lpszFile, nIconIndex, phiconLarge, phiconSmall, nIcons);
//
//     if (lpwstrFile)
//     {
//         MultiByteToWideChar(CP_ACP, 0, lpszFile, -1, lpwstrFile, len);
//         ret = ExtractIconExW(lpwstrFile, nIconIndex, phiconLarge, phiconSmall, nIcons);
//         HeapFree(GetProcessHeap(), 0, lpwstrFile);
//     }
//     return ret;
// }
//
// /*************************************************************************
//  *                ExtractAssociatedIconA (SHELL32.@)
//  *
//  * Return icon for given file (either from file itself or from associated
//  * executable) and patch parameters if needed.
//  */
// HICON WINAPI ExtractAssociatedIconA(HINSTANCE hInst, LPSTR lpIconPath, LPWORD lpiIcon)
// {
//     HICON hIcon = NULL;
//     INT len = MultiByteToWideChar(CP_ACP, 0, lpIconPath, -1, NULL, 0);
//     /* Note that we need to allocate MAX_PATH, since we are supposed to fill
//      * the correct executable if there is no icon in lpIconPath directly.
//      * lpIconPath itself is supposed to be large enough, so make sure lpIconPathW
//      * is large enough too. Yes, I am puking too.
//      */
//     LPWSTR lpIconPathW = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, MAX_PATH * sizeof(WCHAR));
//
//     TRACE("%p %s %p\n", hInst, debugstr_a(lpIconPath), lpiIcon);
//
//     if (lpIconPathW)
//     {
//         MultiByteToWideChar(CP_ACP, 0, lpIconPath, -1, lpIconPathW, len);
//         hIcon = ExtractAssociatedIconW(hInst, lpIconPathW, lpiIcon);
//         WideCharToMultiByte(CP_ACP, 0, lpIconPathW, -1, lpIconPath, MAX_PATH , NULL, NULL);
//         HeapFree(GetProcessHeap(), 0, lpIconPathW);
//     }
//     return hIcon;
// }
//
// /*************************************************************************
//  *                ExtractAssociatedIconW (SHELL32.@)
//  *
//  * Return icon for given file (either from file itself or from associated
//  * executable) and patch parameters if needed.
//  */
// HICON WINAPI ExtractAssociatedIconW(HINSTANCE hInst, LPWSTR lpIconPath, LPWORD lpiIcon)
// {
//     HICON hIcon = NULL;
//     WORD wDummyIcon = 0;
//
//     TRACE("%p %s %p\n", hInst, debugstr_w(lpIconPath), lpiIcon);
//
//     if(lpiIcon == NULL)
//         lpiIcon = &wDummyIcon;
//
//     hIcon = ExtractIconW(hInst, lpIconPath, *lpiIcon);
//
//     if( hIcon < (HICON)2 )
//     { if( hIcon == (HICON)1 ) /* no icons found in given file */
//       { WCHAR tempPath[MAX_PATH];
//         HINSTANCE uRet = FindExecutableW(lpIconPath,NULL,tempPath);
//
//         if( uRet > (HINSTANCE)32 && tempPath[0] )
//         { wcscpy(lpIconPath,tempPath);
//           hIcon = ExtractIconW(hInst, lpIconPath, *lpiIcon);
//           if( hIcon > (HICON)2 )
//             return hIcon;
//         }
//       }
//
//       if( hIcon == (HICON)1 )
//         *lpiIcon = 2;   /* MSDOS icon - we found .exe but no icons in it */
//       else
//         *lpiIcon = 6;   /* generic icon - found nothing */
//
//       if (GetModuleFileNameW(hInst, lpIconPath, MAX_PATH))
//         hIcon = LoadIconW(hInst, MAKEINTRESOURCEW(*lpiIcon));
//     }
//     return hIcon;
// }
//
// /*************************************************************************
//  *                ExtractAssociatedIconExW (SHELL32.@)
//  *
//  * Return icon for given file (either from file itself or from associated
//  * executable) and patch parameters if needed.
//  */
// extern "C" HICON WINAPI ExtractAssociatedIconExW(HINSTANCE hInst, LPWSTR lpIconPath, LPWORD lpiIconIdx, LPWORD lpiIconId)
// {
//   FIXME("%p %s %p %p): stub\n", hInst, debugstr_w(lpIconPath), lpiIconIdx, lpiIconId);
//   return 0;
// }
//
// /*************************************************************************
//  *                ExtractAssociatedIconExA (SHELL32.@)
//  *
//  * Return icon for given file (either from file itself or from associated
//  * executable) and patch parameters if needed.
//  */
// extern "C" HICON WINAPI ExtractAssociatedIconExA(HINSTANCE hInst, LPSTR lpIconPath, LPWORD lpiIconIdx, LPWORD lpiIconId)
// {
//   HICON ret;
//   INT len = MultiByteToWideChar( CP_ACP, 0, lpIconPath, -1, NULL, 0 );
//   LPWSTR lpwstrFile = (LPWSTR)HeapAlloc( GetProcessHeap(), 0, len * sizeof(WCHAR) );
//
//   TRACE("%p %s %p %p)\n", hInst, lpIconPath, lpiIconIdx, lpiIconId);
//
//   MultiByteToWideChar( CP_ACP, 0, lpIconPath, -1, lpwstrFile, len );
//   ret = ExtractAssociatedIconExW(hInst, lpwstrFile, lpiIconIdx, lpiIconId);
//   HeapFree(GetProcessHeap(), 0, lpwstrFile);
//   return ret;
// }
//
//
// /****************************************************************************
//  * SHDefExtractIconW        [SHELL32.@]
//  */
// HRESULT WINAPI SHDefExtractIconW(LPCWSTR pszIconFile, int iIndex, UINT uFlags,
//                                  HICON* phiconLarge, HICON* phiconSmall, UINT nIconSize)
// {
//     UINT ret;
//     HICON hIcons[2];
//     WARN("%s %d 0x%08x %p %p %d, semi-stub\n", debugstr_w(pszIconFile), iIndex, uFlags, phiconLarge, phiconSmall, nIconSize);
//
//     if (!nIconSize)
//         nIconSize = MAKELONG(GetSystemMetrics(SM_CXICON), GetSystemMetrics(SM_CXSMICON));
//
//     ret = PrivateExtractIconsW(pszIconFile, iIndex, nIconSize, nIconSize, hIcons, NULL, 2, LR_DEFAULTCOLOR);
//     /* FIXME: deal with uFlags parameter which contains GIL_ flags */
//     if (ret == 0xFFFFFFFF)
//       return E_FAIL;
//     if (ret > 0) {
//       if (phiconLarge)
//         *phiconLarge = hIcons[0];
//       else
//         DestroyIcon(hIcons[0]);
//       if (phiconSmall)
//         *phiconSmall = hIcons[1];
//       else
//         DestroyIcon(hIcons[1]);
//       return S_OK;
//     }
//     return S_FALSE;
// }
//
// /****************************************************************************
//  * SHDefExtractIconA        [SHELL32.@]
//  */
// HRESULT WINAPI SHDefExtractIconA(LPCSTR pszIconFile, int iIndex, UINT uFlags,
//                                  HICON* phiconLarge, HICON* phiconSmall, UINT nIconSize)
// {
//   HRESULT ret;
//   INT len = MultiByteToWideChar(CP_ACP, 0, pszIconFile, -1, NULL, 0);
//   LPWSTR lpwstrFile = (LPWSTR)HeapAlloc(GetProcessHeap(), 0, len * sizeof(WCHAR));
//
//   TRACE("%s %d 0x%08x %p %p %d\n", pszIconFile, iIndex, uFlags, phiconLarge, phiconSmall, nIconSize);
//
//   MultiByteToWideChar(CP_ACP, 0, pszIconFile, -1, lpwstrFile, len);
//   ret = SHDefExtractIconW(lpwstrFile, iIndex, uFlags, phiconLarge, phiconSmall, nIconSize);
//   HeapFree(GetProcessHeap(), 0, lpwstrFile);
//   return ret;
// }
//
// /****************************************************************************
//  * SHGetIconOverlayIndexA    [SHELL32.@]
//  *
//  * Returns the index of the overlay icon in the system image list.
//  */
// extern "C" INT WINAPI SHGetIconOverlayIndexA(LPCSTR pszIconPath, INT iIconIndex)
// {
//   FIXME("%s, %d\n", debugstr_a(pszIconPath), iIconIndex);
//
//   return -1;
// }
//
// /****************************************************************************
//  * SHGetIconOverlayIndexW    [SHELL32.@]
//  *
//  * Returns the index of the overlay icon in the system image list.
//  */
// extern "C" INT WINAPI SHGetIconOverlayIndexW(LPCWSTR pszIconPath, INT iIconIndex)
// {
//   FIXME("%s, %d\n", debugstr_w(pszIconPath), iIconIndex);
//
//   return -1;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
