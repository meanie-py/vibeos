use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{RtlCopyUnicodeString, ExAllocatePoolWithTag};
use ntkrnl_sys::base::{STATUS_SUCCESS, STATUS_NO_MEMORY};
use core::ffi::c_void;
use ntkrnl_sys::w;

extern "C" {
    fn wcslen(s: *const u16) -> usize;
}

// ---- helpers (FORCEINLINE list ops / macros / pool tag the C used) ----
// TAG_USTR = 'RTSU' (tags.h) — C multichar, big-endian.
const TAG_USTR: u32 = 0x5254_5355;

#[inline(always)]
unsafe fn InitializeListHead(lh: PLIST_ENTRY) {
    (*lh).Flink = lh;
    (*lh).Blink = lh;
}
#[inline(always)]
unsafe fn IsListEmpty(lh: PLIST_ENTRY) -> u8 {
    ((*lh).Flink == lh) as u8
}
#[inline(always)]
unsafe fn RemoveTailList(lh: PLIST_ENTRY) -> PLIST_ENTRY {
    let entry = (*lh).Blink;
    let new_tail = (*entry).Blink;
    (*new_tail).Flink = lh;
    (*lh).Blink = new_tail;
    entry
}
#[inline(always)]
unsafe fn InsertHeadList(lh: PLIST_ENTRY, entry: PLIST_ENTRY) {
    let first = (*lh).Flink;
    (*entry).Flink = first;
    (*entry).Blink = lh;
    (*first).Blink = entry;
    (*lh).Flink = entry;
}
#[inline(always)]
unsafe fn RtlCopyMemory(dst: *mut c_void, src: *const c_void, len: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dst as *mut u8, len);
}
// ASSERT_DC_PREPARED is a debug ASSERT in C — no-op (DUMMY_PSEH / NDEBUG).
#[inline(always)]
unsafe fn ASSERT_DC_PREPARED(_dc: PDC) {}
// LANGID bit macros (winnt.h)
#[inline(always)]
fn PRIMARYLANGID(lgid: u16) -> u32 { (lgid & 0x3ff) as u32 }
#[inline(always)]
fn SUBLANGID(lgid: u16) -> u32 { (lgid >> 10) as u32 }
// ROP4_FROM_INDEX(index) = (index | (index << 8))  [inteng.h]
#[inline(always)]
fn ROP4_FROM_INDEX(index: u32) -> u32 { index | (index << 8) }

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut g_PointZero: POINTL = POINTL { x: 0, y: 0 };

#[no_mangle]
pub unsafe extern "C" fn SZZ_GetSize(pszz: PCZZWSTR) -> usize {
    
    let mut ret: usize = 0;
    let mut pch = pszz;
    while *pch != 0 {
        let cch = wcslen(pch) + 1;
        ret += cch;
        pch = pch.add(cch);
    }
    ret += 1;
    ret * core::mem::size_of::<u16>()
}

#[no_mangle]
pub unsafe extern "C" fn IntRebaseList(
    pNewHead: PLIST_ENTRY,
    pOldHead: PLIST_ENTRY)
{
    let mut Entry: PLIST_ENTRY;

    InitializeListHead(pNewHead);
    while IsListEmpty(pOldHead) == 0 {
        Entry = RemoveTailList(pOldHead);
        InsertHeadList(pNewHead, Entry);
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntNormalizeAngle(nTenthsOfDegrees: LONG) -> LONG {
    let mut nTenthsOfDegrees = nTenthsOfDegrees % (360 * 10);
    if nTenthsOfDegrees >= 0 {
        return nTenthsOfDegrees;
    }
    return nTenthsOfDegrees + 360 * 10;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntNameFromCharSet(CharSet: u8) -> LPCWSTR {
    match CharSet as u32 {
        ANSI_CHARSET => w!("ANSI").as_ptr(),
        DEFAULT_CHARSET => w!("Default").as_ptr(),
        SYMBOL_CHARSET => w!("Symbol").as_ptr(),
        SHIFTJIS_CHARSET => w!("Shift_JIS").as_ptr(),
        HANGUL_CHARSET => w!("Hangul").as_ptr(),
        GB2312_CHARSET => w!("GB 2312").as_ptr(),
        CHINESEBIG5_CHARSET => w!("Chinese Big5").as_ptr(),
        OEM_CHARSET => w!("OEM").as_ptr(),
        JOHAB_CHARSET => w!("Johab").as_ptr(),
        HEBREW_CHARSET => w!("Hebrew").as_ptr(),
        ARABIC_CHARSET => w!("Arabic").as_ptr(),
        GREEK_CHARSET => w!("Greek").as_ptr(),
        TURKISH_CHARSET => w!("Turkish").as_ptr(),
        VIETNAMESE_CHARSET => w!("Vietnamese").as_ptr(),
        THAI_CHARSET => w!("Thai").as_ptr(),
        EASTEUROPE_CHARSET => w!("Eastern European").as_ptr(),
        RUSSIAN_CHARSET => w!("Russian").as_ptr(),
        MAC_CHARSET => w!("Mac").as_ptr(),
        BALTIC_CHARSET => w!("Baltic").as_ptr(),
        _ => w!("Unknown").as_ptr(),
    }
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntNameFromCharSet(CharSet: u8) -> LPCWSTR {
    match CharSet as u32 {
        ANSI_CHARSET => w!("ANSI").as_ptr(),
        DEFAULT_CHARSET => w!("Default").as_ptr(),
        SYMBOL_CHARSET => w!("Symbol").as_ptr(),
        SHIFTJIS_CHARSET => w!("Shift_JIS").as_ptr(),
        HANGUL_CHARSET => w!("Hangul").as_ptr(),
        GB2312_CHARSET => w!("GB 2312").as_ptr(),
        CHINESEBIG5_CHARSET => w!("Chinese Big5").as_ptr(),
        OEM_CHARSET => w!("OEM").as_ptr(),
        JOHAB_CHARSET => w!("Johab").as_ptr(),
        HEBREW_CHARSET => w!("Hebrew").as_ptr(),
        ARABIC_CHARSET => w!("Arabic").as_ptr(),
        GREEK_CHARSET => w!("Greek").as_ptr(),
        TURKISH_CHARSET => w!("Turkish").as_ptr(),
        VIETNAMESE_CHARSET => w!("Vietnamese").as_ptr(),
        THAI_CHARSET => w!("Thai").as_ptr(),
        EASTEUROPE_CHARSET => w!("Eastern European").as_ptr(),
        RUSSIAN_CHARSET => w!("Russian").as_ptr(),
        MAC_CHARSET => w!("Mac").as_ptr(),
        BALTIC_CHARSET => w!("Baltic").as_ptr(),
        _ => w!("Unknown").as_ptr(),
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntCharSetFromLangID(LangID: LANGID) -> u8 {
    match PRIMARYLANGID(LangID) {
        LANG_CHINESE => {
            match SUBLANGID(LangID) {
                SUBLANG_CHINESE_TRADITIONAL => return CHINESEBIG5_CHARSET as u8,
                SUBLANG_CHINESE_SIMPLIFIED | _ => {}
            }
            return GB2312_CHARSET as u8;
        }

        LANG_CZECH | LANG_HUNGARIAN | LANG_POLISH
        | LANG_SLOVAK | LANG_SLOVENIAN | LANG_ROMANIAN => {
            return EASTEUROPE_CHARSET as u8;
        }

        LANG_RUSSIAN | LANG_BULGARIAN | LANG_MACEDONIAN
        | LANG_SERBIAN | LANG_UKRAINIAN => {
            return RUSSIAN_CHARSET as u8;
        }

        LANG_ARABIC => return ARABIC_CHARSET as u8,
        LANG_GREEK => return GREEK_CHARSET as u8,
        LANG_HEBREW => return HEBREW_CHARSET as u8,
        LANG_JAPANESE => return SHIFTJIS_CHARSET as u8,
        LANG_KOREAN => return JOHAB_CHARSET as u8,
        LANG_TURKISH => return TURKISH_CHARSET as u8,
        LANG_THAI => return THAI_CHARSET as u8,
        LANG_LATVIAN => return BALTIC_CHARSET as u8,
        LANG_VIETNAMESE => return VIETNAMESE_CHARSET as u8,

        LANG_ENGLISH | LANG_BASQUE | LANG_CATALAN
        | LANG_DANISH | LANG_DUTCH | LANG_FINNISH
        | LANG_FRENCH | LANG_GERMAN | LANG_ITALIAN
        | LANG_NORWEGIAN | LANG_PORTUGUESE | LANG_SPANISH
        | LANG_SWEDISH | _ => {
            return ANSI_CHARSET as u8;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntSwapEndian(pvData: LPVOID, mut Size: u32) {
    let mut pb = pvData as *mut u8;
    Size /= 2;
    while Size > 0 {
        Size -= 1;
        let b = *pb.offset(0);
        *pb.offset(0) = *pb.offset(1);
        *pb.offset(1) = b;
        pb = pb.offset(1);
        pb = pb.offset(1);
    }
}

#[no_mangle]
pub unsafe extern "C" fn PathFindFileNameW(pszPath: PCWSTR) -> PWSTR {
    let mut lastSlash: PCWSTR = pszPath;
    let mut current: PCWSTR = pszPath;
    while *current != 0 {
        if (*current == '\\' as u16 || *current == '/' as u16 || *current == ':' as u16) &&
            *current.offset(1) != 0 && *current.offset(1) != '\\' as u16 && *current.offset(1) != '/' as u16 {
            lastSlash = current.offset(1);
        }
        current = current.offset(1);
    }
    lastSlash as PWSTR
}

#[no_mangle]
pub unsafe extern "C" fn PathIsRelativeW(lpszPath: LPCWSTR) -> i32 {
    if lpszPath.is_null() || *lpszPath == 0 {
        return 1;
    }
    if *lpszPath == b'\\' as u16 || (!(*lpszPath == 0) && *lpszPath.offset(1) == b':' as u16) {
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "C" fn IntUnicodeStringToBuffer(
    pszBuffer: LPWSTR,
    cbBuffer: usize,
    pString: *const _UNICODE_STRING,
) {
    let mut cbLength = (*pString).Length as usize;

    if cbBuffer < core::mem::size_of::<u16>() {
        return;
    }

    if cbLength > cbBuffer - core::mem::size_of::<u16>() {
        cbLength = cbBuffer - core::mem::size_of::<u16>();
    }

    RtlCopyMemory(
        pszBuffer as *mut core::ffi::c_void,
        (*pString).Buffer as *const core::ffi::c_void,
        cbLength,
    );
    *pszBuffer.add(cbLength / core::mem::size_of::<u16>()) = 0u16;
}

#[no_mangle]
pub unsafe extern "C" fn IntDuplicateUnicodeString(
    Source: PCUNICODE_STRING,
    Destination: PUNICODE_STRING) -> NTSTATUS
{
    let mut Status: NTSTATUS = STATUS_NO_MEMORY;
    let mut Tmp: _UNICODE_STRING = core::mem::zeroed();

    Tmp.Buffer = ExAllocatePoolWithTag(PagedPool, (*Source).MaximumLength as SIZE_T, TAG_USTR) as *mut u16;
    if !Tmp.Buffer.is_null()
    {
        Tmp.MaximumLength = (*Source).MaximumLength;
        Tmp.Length = 0;
        RtlCopyUnicodeString(core::ptr::addr_of_mut!(Tmp) as *mut _, Source as *const _);

        (*Destination).MaximumLength = Tmp.MaximumLength;
        (*Destination).Length = Tmp.Length;
        (*Destination).Buffer = Tmp.Buffer;

        Status = STATUS_SUCCESS;
    }

    return Status;
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntEngFillBox(
    dc: PDC,
    X: i32,
    Y: i32,
    Width: i32,
    Height: i32,
    BrushObj: *mut BRUSHOBJ,
) {
    let mut DestRect: RECTL = core::mem::zeroed();
    let psurf: *mut SURFACE = (*dc).dclevel.pSurface;

    ASSERT_DC_PREPARED(dc);
    ntkrnl_sys::ASSERT!(!psurf.is_null());

    let mut x = X;
    let mut width = Width;
    let mut y = Y;
    let mut height = Height;

    if width < 0 {
        x += width;
        width = -width;
    }

    if height < 0 {
        y += height;
        height = -height;
    }

    DestRect.left = x;
    DestRect.right = x + width;
    DestRect.top = y;
    DestRect.bottom = y + height;

    IntEngBitBlt(
        core::ptr::addr_of_mut!((*psurf).SurfObj),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(DestRect),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        BrushObj,
        core::ptr::addr_of_mut!(g_PointZero),
        ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY),
    );
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntEngFillBox(
    dc: PDC,
    X: i32,
    Y: i32,
    Width: i32,
    Height: i32,
    BrushObj: *mut BRUSHOBJ,
) {
    let mut DestRect: RECTL = core::mem::zeroed();
    let psurf: *mut SURFACE = (*dc).dclevel.pSurface;

    ASSERT_DC_PREPARED(dc);
    ntkrnl_sys::ASSERT!(!psurf.is_null());

    let mut x = X;
    let mut width = Width;
    let mut y = Y;
    let mut height = Height;

    if width < 0 {
        x += width;
        width = -width;
    }

    if height < 0 {
        y += height;
        height = -height;
    }

    DestRect.left = x;
    DestRect.right = x + width;
    DestRect.top = y;
    DestRect.bottom = y + height;

    IntEngBitBlt(
        core::ptr::addr_of_mut!((*psurf).SurfObj),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!((*dc).co) as *mut CLIPOBJ,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(DestRect),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        BrushObj,
        core::ptr::addr_of_mut!(g_PointZero),
        ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY),
    );
}

#[no_mangle]
pub unsafe extern "system" fn IntEngFillPolygon(
    dc: PDC,
    pPoints: *mut POINTL,
    cPoints: u32,
    BrushObj: *mut BRUSHOBJ,
) {
    let psurf: *mut SURFACE = (*dc).dclevel.pSurface;
    let mut Rect: RECTL = core::mem::zeroed();
    let mut i: u32;
    let mut x: i32;
    let mut y: i32;

    ASSERT_DC_PREPARED(dc);

    Rect.left = (*pPoints.offset(0)).x;
    Rect.right = (*pPoints.offset(0)).x;
    Rect.top = (*pPoints.offset(0)).y;
    Rect.bottom = (*pPoints.offset(0)).y;
    
    i = 1;
    while i < cPoints {
        x = (*pPoints.offset(i as isize)).x;
        if x < Rect.left {
            Rect.left = x;
        } else if Rect.right < x {
            Rect.right = x;
        }

        y = (*pPoints.offset(i as isize)).y;
        if y < Rect.top {
            Rect.top = y;
        } else if Rect.bottom < y {
            Rect.bottom = y;
        }
        i += 1;
    }

    IntFillPolygon(
        dc,
        (*dc).dclevel.pSurface,
        BrushObj,
        pPoints,
        cPoints as i32,
        Rect,
        core::ptr::addr_of_mut!(g_PointZero),
    );
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS win32 kernel mode subsystem
//  * LICENSE:     GPL-2.1-or-later (https://spdx.org/licenses/GPL-2.1-or-later)
//  * PURPOSE:     Utility functions
//  * COPYRIGHT:   Copyright 2026 Katayama Hirofumi MZ <katayama.hirofumi.mz@gmail.com>
//  */
//
// #include <win32k.h>
//
// extern UNICODE_STRING g_FontRegPath;
//
// POINTL g_PointZero = { 0, 0 };
//
// SIZE_T
// SZZ_GetSize(_In_ PCZZWSTR pszz)
// {
//     SIZE_T ret = 0, cch;
//     const WCHAR *pch = pszz;
//     while (*pch)
//     {
//         cch = wcslen(pch) + 1;
//         ret += cch;
//         pch += cch;
//     }
//     ++ret;
//     return ret * sizeof(WCHAR);
// }
//
// VOID
// IntRebaseList(
//     _Inout_ PLIST_ENTRY pNewHead,
//     _Inout_ PLIST_ENTRY pOldHead)
// {
//     PLIST_ENTRY Entry;
//
//     ASSERT(pNewHead != pOldHead);
//
//     InitializeListHead(pNewHead);
//     while (!IsListEmpty(pOldHead))
//     {
//         Entry = RemoveTailList(pOldHead);
//         InsertHeadList(pNewHead, Entry);
//     }
// }
//
// LONG IntNormalizeAngle(_In_ LONG nTenthsOfDegrees)
// {
//     nTenthsOfDegrees %= 360 * 10;
//     if (nTenthsOfDegrees >= 0)
//         return nTenthsOfDegrees;
//     return nTenthsOfDegrees + 360 * 10;
// }
//
// LPCWSTR FASTCALL IntNameFromCharSet(_In_ BYTE CharSet)
// {
//     switch (CharSet)
//     {
//         case ANSI_CHARSET: return L"ANSI";
//         case DEFAULT_CHARSET: return L"Default";
//         case SYMBOL_CHARSET: return L"Symbol";
//         case SHIFTJIS_CHARSET: return L"Shift_JIS";
//         case HANGUL_CHARSET: return L"Hangul";
//         case GB2312_CHARSET: return L"GB 2312";
//         case CHINESEBIG5_CHARSET: return L"Chinese Big5";
//         case OEM_CHARSET: return L"OEM";
//         case JOHAB_CHARSET: return L"Johab";
//         case HEBREW_CHARSET: return L"Hebrew";
//         case ARABIC_CHARSET: return L"Arabic";
//         case GREEK_CHARSET: return L"Greek";
//         case TURKISH_CHARSET: return L"Turkish";
//         case VIETNAMESE_CHARSET: return L"Vietnamese";
//         case THAI_CHARSET: return L"Thai";
//         case EASTEUROPE_CHARSET: return L"Eastern European";
//         case RUSSIAN_CHARSET: return L"Russian";
//         case MAC_CHARSET: return L"Mac";
//         case BALTIC_CHARSET: return L"Baltic";
//         default: return L"Unknown";
//     }
// }
//
// /* See https://learn.microsoft.com/en-us/previous-versions/visualstudio/visual-studio-2008/bb165625(v=vs.90) */
// BYTE IntCharSetFromLangID(_In_ LANGID LangID)
// {
//     /* FIXME: Add more and fix if wrong */
//     switch (PRIMARYLANGID(LangID))
//     {
//         case LANG_CHINESE:
//             switch (SUBLANGID(LangID))
//             {
//                 case SUBLANG_CHINESE_TRADITIONAL:
//                     return CHINESEBIG5_CHARSET;
//                 case SUBLANG_CHINESE_SIMPLIFIED:
//                 default:
//                     break;
//             }
//             return GB2312_CHARSET;
//
//         case LANG_CZECH: case LANG_HUNGARIAN: case LANG_POLISH:
//         case LANG_SLOVAK: case LANG_SLOVENIAN: case LANG_ROMANIAN:
//             return EASTEUROPE_CHARSET;
//
//         case LANG_RUSSIAN: case LANG_BULGARIAN: case LANG_MACEDONIAN:
//         case LANG_SERBIAN: case LANG_UKRAINIAN:
//             return RUSSIAN_CHARSET;
//
//         case LANG_ARABIC:       return ARABIC_CHARSET;
//         case LANG_GREEK:        return GREEK_CHARSET;
//         case LANG_HEBREW:       return HEBREW_CHARSET;
//         case LANG_JAPANESE:     return SHIFTJIS_CHARSET;
//         case LANG_KOREAN:       return JOHAB_CHARSET;
//         case LANG_TURKISH:      return TURKISH_CHARSET;
//         case LANG_THAI:         return THAI_CHARSET;
//         case LANG_LATVIAN:      return BALTIC_CHARSET;
//         case LANG_VIETNAMESE:   return VIETNAMESE_CHARSET;
//
//         case LANG_ENGLISH: case LANG_BASQUE: case LANG_CATALAN:
//         case LANG_DANISH: case LANG_DUTCH: case LANG_FINNISH:
//         case LANG_FRENCH: case LANG_GERMAN: case LANG_ITALIAN:
//         case LANG_NORWEGIAN: case LANG_PORTUGUESE: case LANG_SPANISH:
//         case LANG_SWEDISH: default:
//             return ANSI_CHARSET;
//     }
// }
//
// VOID IntSwapEndian(_Inout_ LPVOID pvData, _In_ DWORD Size)
// {
//     BYTE b, *pb = pvData;
//     Size /= 2;
//     while (Size-- > 0)
//     {
//         b = pb[0];
//         pb[0] = pb[1];
//         pb[1] = b;
//         ++pb; ++pb;
//     }
// }
//
// /* Borrowed from shlwapi!PathFindFileNameW */
// PWSTR PathFindFileNameW(_In_ PCWSTR pszPath)
// {
//     PCWSTR lastSlash = pszPath;
//     while (*pszPath)
//     {
//         if ((*pszPath == '\\' || *pszPath == '/' || *pszPath == ':') &&
//             pszPath[1] && pszPath[1] != '\\' && pszPath[1] != '/')
//         {
//             lastSlash = pszPath + 1;
//         }
//         pszPath++;
//     }
//     return (PWSTR)lastSlash;
// }
//
// /* Borrowed from shlwapi!PathIsRelativeW */
// BOOL PathIsRelativeW(_In_ LPCWSTR lpszPath)
// {
//     if (!lpszPath || !*lpszPath)
//         return TRUE;
//     if (*lpszPath == '\\' || (*lpszPath && lpszPath[1] == ':'))
//         return FALSE;
//     return TRUE;
// }
//
// VOID
// IntUnicodeStringToBuffer(
//     _Out_ LPWSTR pszBuffer,
//     _In_ SIZE_T cbBuffer,
//     _In_ const UNICODE_STRING *pString)
// {
//     SIZE_T cbLength = pString->Length;
//
//     if (cbBuffer < sizeof(UNICODE_NULL))
//         return;
//
//     if (cbLength > cbBuffer - sizeof(UNICODE_NULL))
//         cbLength = cbBuffer - sizeof(UNICODE_NULL);
//
//     RtlCopyMemory(pszBuffer, pString->Buffer, cbLength);
//     pszBuffer[cbLength / sizeof(WCHAR)] = UNICODE_NULL;
// }
//
// NTSTATUS
// IntDuplicateUnicodeString(
//     _In_ PCUNICODE_STRING Source,
//     _Out_ PUNICODE_STRING Destination)
// {
//     NTSTATUS Status = STATUS_NO_MEMORY;
//     UNICODE_STRING Tmp;
//
//     Tmp.Buffer = ExAllocatePoolWithTag(PagedPool, Source->MaximumLength, TAG_USTR);
//     if (Tmp.Buffer)
//     {
//         Tmp.MaximumLength = Source->MaximumLength;
//         Tmp.Length = 0;
//         RtlCopyUnicodeString(&Tmp, Source);
//
//         Destination->MaximumLength = Tmp.MaximumLength;
//         Destination->Length = Tmp.Length;
//         Destination->Buffer = Tmp.Buffer;
//
//         Status = STATUS_SUCCESS;
//     }
//
//     return Status;
// }
//
// VOID
// FASTCALL
// IntEngFillBox(
//     IN OUT PDC dc,
//     IN INT X,
//     IN INT Y,
//     IN INT Width,
//     IN INT Height,
//     IN BRUSHOBJ *BrushObj)
// {
//     RECTL DestRect;
//     SURFACE *psurf = dc->dclevel.pSurface;
//
//     ASSERT_DC_PREPARED(dc);
//     ASSERT(psurf != NULL);
//
//     if (Width < 0)
//     {
//         X += Width;
//         Width = -Width;
//     }
//
//     if (Height < 0)
//     {
//         Y += Height;
//         Height = -Height;
//     }
//
//     DestRect.left = X;
//     DestRect.right = X + Width;
//     DestRect.top = Y;
//     DestRect.bottom = Y + Height;
//
//     IntEngBitBlt(&psurf->SurfObj,
//                  NULL,
//                  NULL,
//                  (CLIPOBJ *)&dc->co,
//                  NULL,
//                  &DestRect,
//                  NULL,
//                  NULL,
//                  BrushObj,
//                  &g_PointZero,
//                  ROP4_FROM_INDEX(R3_OPINDEX_PATCOPY));
// }
//
// VOID APIENTRY
// IntEngFillPolygon(
//     IN OUT PDC dc,
//     IN POINTL *pPoints,
//     IN UINT cPoints,
//     IN BRUSHOBJ *BrushObj)
// {
//     SURFACE *psurf = dc->dclevel.pSurface;
//     RECT Rect;
//     UINT i;
//     INT x, y;
//
//     ASSERT_DC_PREPARED(dc);
//     ASSERT(psurf != NULL);
//
//     Rect.left = Rect.right = pPoints[0].x;
//     Rect.top = Rect.bottom = pPoints[0].y;
//     for (i = 1; i < cPoints; ++i)
//     {
//         x = pPoints[i].x;
//         if (x < Rect.left)
//             Rect.left = x;
//         else if (Rect.right < x)
//             Rect.right = x;
//
//         y = pPoints[i].y;
//         if (y < Rect.top)
//             Rect.top = y;
//         else if (Rect.bottom < y)
//             Rect.bottom = y;
//     }
//
//     IntFillPolygon(dc, dc->dclevel.pSurface, BrushObj, pPoints, cPoints, Rect, &g_PointZero);
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
