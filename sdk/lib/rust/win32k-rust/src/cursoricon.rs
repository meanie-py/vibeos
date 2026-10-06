#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag, PsGetCurrentProcessWin32Process, RtlAddAtomToAtomTable, RtlCompareUnicodeString, RtlDeleteAtomFromAtomTable, RtlLookupAtomInAtomTable, RtlQueryAtomInAtomTable};
use ntkrnl_sys::rtl::{RtlCopyMemory, RtlZeroMemory};
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS};
use core::ffi::c_void;
const STATUS_ACCESS_VIOLATION: i32 = 0xC0000005_u32 as i32;

// Debug-channel macros — no kernel symbol, no-op.
#[allow(unused_macros)]
#[allow(unused_macros)]
#[allow(unused_macros)]
#[allow(unused_macros)]

// Pool tags (tags.h): USERTAG_CURSOR = 'ucsU', TAG_STRING = ' RTS'.
// C multi-char constants are big-endian; reverse for little-endian bytes.
const USERTAG_CURSOR: u32 = u32::from_le_bytes(*b"Uscu");
const TAG_STRING: u32 = u32::from_le_bytes(*b"STR "); // nolint: const-contradiction (' RTS' -- win32ss/user/ntuser/tags.h:3; freeldr's TAG_STRING is ' rtS')

// RT_ICON resource type (winuser.h MAKEINTRESOURCE(3)).
const RT_ICON: usize = 3;

extern "C" {
    fn GetW32ThreadInfo() -> *mut core::ffi::c_void;
    // IntLPtoDP is `static` in coord.c; reach it via the path_rust_shims cdecl shim.
    fn w32r_IntLPtoDP(pdc: PDC, ppt: LPPOINT, count: INT) -> BOOL;
}

// co_MsqInsertMouseMessage — FASTCALL (`@co_MsqInsertMouseMessage@16`) on i386.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn co_MsqInsertMouseMessage(Msg: *mut MSG, flags: DWORD, dwExtraInfo: ULONG_PTR, Hook: BOOL,);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn co_MsqInsertMouseMessage(Msg: *mut MSG, flags: DWORD, dwExtraInfo: ULONG_PTR, Hook: BOOL,);
}

// RtlInitEmptyUnicodeString (ntdef.h FORCEINLINE) — no exported symbol.
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(dst: *mut _UNICODE_STRING, buf: *mut u16, len: USHORT) {
    (*dst).Length = 0;
    (*dst).MaximumLength = len;
    (*dst).Buffer = buf;
}

// SetLastNtError — FASTCALL (`@SetLastNtError@4`) on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn SetLastNtError(Status: NTSTATUS);
}

// CheckWinstaAttributeAccess — FASTCALL (`@CheckWinstaAttributeAccess@4`) on i386.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn CheckWinstaAttributeAccess( amDesired: ACCESS_MASK) -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn CheckWinstaAttributeAccess(amDesired: ACCESS_MASK) -> BOOL;
}

// FreeProcMarkObject — CDECL (`_FreeProcMarkObject`).
extern "C" {
    fn FreeProcMarkObject(Object: *mut c_void);
}

// GDI engine helpers — STDCALL (`_Name@N`) → extern "system".
extern "system" {
    fn EXLATEOBJ_vInitialize(pexlo: PEXLATEOBJ, ppalSrc: *mut PALETTE, ppalDst: *mut PALETTE, c1: COLORREF, c2: COLORREF, c3: COLORREF);
    fn EXLATEOBJ_vInitSrcMonoXlate(pexlo: PEXLATEOBJ, ppalDst: PPALETTE, crBackgroundClr: COLORREF, crForegroundClr: COLORREF);
    fn EXLATEOBJ_vCleanup(exlo: *mut EXLATEOBJ);
    fn EBRUSHOBJ_vInit(pebo: *mut EBRUSHOBJ, pbrush: PBRUSH, psurf: *mut c_void, crFore: COLORREF, crBack: COLORREF, ppal: *mut c_void);
    fn EBRUSHOBJ_vCleanup(pebo: *mut EBRUSHOBJ);
    fn GreSetBitmapOwner(hbmp: HBITMAP, ulOwner: ULONG) -> BOOL;
}

// MmCopyToCaller(x,y,z) macro → _MmCopyToCaller (mmcopy.h).
#[inline(always)]
unsafe fn MmCopyToCaller(target: *mut c_void, source: *const c_void, bytes: ULONG) -> NTSTATUS {
    _MmCopyToCaller(target, source as *mut c_void, bytes as UINT)
}

// SafeProbeForRead/SafeProbeForWrite — safe_user_io.h, NTAPI (`_Safe...@12`).
extern "system" {
    fn SafeProbeForRead(Address: *const c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: *mut c_void, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
}

// ProbeAndCaptureUnicodeStringOrAtom (class.c) — NTAPI (`_...@8`), real export.
extern "system" {
    fn ProbeAndCaptureUnicodeStringOrAtom(pustrOut: PUNICODE_STRING, pustrUnsafe: PUNICODE_STRING) -> NTSTATUS;
}

// ProbeAndCaptureUnicodeString (probe.h static __inline) — probe + deep-copy.
// Delegates to the one SEH-faithful implementation: the old local copy used the
// RAISING ProbeForRead (unhandled here = BUGCHECK 0x1E) and an unprotected copy.
#[inline(always)]
unsafe fn ProbeAndCaptureUnicodeString(
    Dest: PUNICODE_STRING,
    CurrentMode: KPROCESSOR_MODE,
    UnsafeSrc: *const _UNICODE_STRING,
) -> NTSTATUS {
    crate::safe_user_io::probe_and_capture_unicode_string(Dest, CurrentMode as i8, UnsafeSrc)
}

// ReleaseCapturedUnicodeString (probe.h) — free a user-mode captured buffer.
#[inline(always)]
unsafe fn ReleaseCapturedUnicodeString(CapturedString: PUNICODE_STRING, CurrentMode: KPROCESSOR_MODE) {
    if CurrentMode != 0 && !(*CapturedString).Buffer.is_null() {
        ExFreePoolWithTag((*CapturedString).Buffer as *mut c_void, u32::from_le_bytes(*b"RTSU"));
    }
}

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of HEAD). FORCEINLINE.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    *(obj as *const HANDLE)
}

// NT_ASSERT — debug-only, no-op in release.
#[inline(always)]
fn NT_ASSERT(_cond: bool) {}

// IS_INTRESOURCE(ptr) = ((ULONG_PTR)ptr >> 16) == 0. (winuser.h macro)
#[inline(always)]
fn IS_INTRESOURCE<T>(ptr: *const T) -> BOOL {
    ((ptr as usize) >> 16 == 0) as BOOL
}

// MAKELPARAM(l, h) = (WORD)l | ((WORD)h << 16). (winuser.h macro)
#[inline(always)]
fn MAKELPARAM(l: u16, h: u16) -> DWORD {
    (l as u32) | ((h as u32) << 16)
}

// min/max integer helpers (C macros).
#[inline(always)]
fn max(a: LONG, b: LONG) -> LONG { if a > b { a } else { b } }
#[inline(always)]
fn min(a: LONG, b: LONG) -> LONG { if a < b { a } else { b } }

// GDI object-handle type field. GDI_HANDLE_GET_TYPE(h) = h & 0x007f0000.
#[inline(always)]
fn GDI_HANDLE_GET_TYPE<T>(h: *mut T) -> ULONG { ((h as usize) & 0x007f0000) as ULONG }

// SURFACE_/BRUSH_ share lock helpers — GDI-object macros (eng/<type>.h), not real
// fns. Inline-expand via GDIOBJ_* primitives (matches bitmaps.rs/draw.rs).
#[inline(always)]
unsafe fn SURFACE_ShareLockSurface(h: HGDIOBJ) -> PSURFACE {
    GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BITMAP as DWORD) as PSURFACE
}
#[inline(always)]
unsafe fn SURFACE_ShareUnlockSurface(p: PSURFACE) { GDIOBJ_vDereferenceObject(p as POBJ); }
#[inline(always)]
unsafe fn BRUSH_ShareLockBrush(h: HGDIOBJ) -> PBRUSH {
    GDIOBJ_ShareLockObj(h, GDI_OBJECT_TYPE_BRUSH as DWORD) as PBRUSH
}
#[inline(always)]
unsafe fn BRUSH_ShareUnlockBrush(pbr: PBRUSH) { GDIOBJ_vDereferenceObject(pbr as POBJ); }

// gpsi->ptCursor lives in the anonymous PERUSERSERVERINFO tail of tagSERVERINFO
// (bindgen flattens it to __bindgen_padding_0). Matches input.rs/ntmouse.rs.
#[inline(always)]
unsafe fn gpsi_per_user() -> *mut _PERUSERSERVERINFO {
    (gpsi as *mut u8)
        .add(core::mem::offset_of!(tagSERVERINFO, __bindgen_padding_0)) as *mut _PERUSERSERVERINFO
}

// PROCESSINFO->W32PF_flags lives on the flattened W32PROCESS prefix (peProcess
// ptr + RefCount DWORD) → offset size_of::<usize>()+size_of::<u32>(). Matches misc.rs.
#[inline(always)]
unsafe fn ppi_w32pf_flags(ppi: PPROCESSINFO) -> *mut DWORD {
    let off = core::mem::size_of::<usize>() + core::mem::size_of::<u32>();
    (ppi as *mut u8).add(off) as *mut DWORD
}

// RECTL_vSetRect / RECTL_vOffsetRect (rect.h FORCEINLINE).
#[inline(always)]
unsafe fn RECTL_vSetRect(prcl: *mut RECTL, left: i32, top: i32, right: i32, bottom: i32) {
    (*prcl).left = left;
    (*prcl).top = top;
    (*prcl).right = right;
    (*prcl).bottom = bottom;
}
#[inline(always)]
unsafe fn RECTL_vOffsetRect(prcl: *mut RECTL, cx: i32, cy: i32) {
    (*prcl).left += cx;
    (*prcl).top += cy;
    (*prcl).right += cx;
    (*prcl).bottom += cy;
}

// ---- file-scope preamble (translated) ----
pub static mut gSysCursorInfo: SYSTEM_CURSORINFO = unsafe { core::mem::zeroed() };
pub static mut gcurFirst: PCURICON_OBJECT = core::ptr::null_mut();
#[no_mangle]
pub static mut gasyscur: [SYSTEMCURICO; 16] = [
    SYSTEMCURICO { type_: OCR_NORMAL as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_IBEAM as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_WAIT as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_CROSS as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_UP as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_ICON as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_SIZE as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_SIZENWSE as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_SIZENESW as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_SIZEWE as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_SIZENS as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_SIZEALL as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_NO as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_HAND as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_APPSTARTING as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OCR_HELP as u32, handle: core::ptr::null_mut() },
];
pub static mut gasysico: [SYSTEMCURICO; 6] = [
    SYSTEMCURICO { type_: OIC_SAMPLE as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OIC_HAND as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OIC_QUES as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OIC_BANG as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OIC_NOTE as u32, handle: core::ptr::null_mut() },
    SYSTEMCURICO { type_: OIC_WINLOGO as u32, handle: core::ptr::null_mut() },
];

#[no_mangle]
pub unsafe extern "C" fn InitCursorImpl() -> BOOL {
    gSysCursorInfo.Enabled = 0u8 as _;
    gSysCursorInfo.ButtonsDown = 0;
    gSysCursorInfo.bClipped = 0u8 as _;
    gSysCursorInfo.LastBtnDown = 0;
    gSysCursorInfo.CurrentCursorObject = core::ptr::null_mut();
    gSysCursorInfo.ShowingCursor = -1;
    gSysCursorInfo.ClickLockActive = 0u8 as _;
    gSysCursorInfo.ClickLockTime = 0;

    1
}

unsafe fn IntInsertCursorIntoList(pcur: PCURICON_OBJECT) {
    let ppi: PPROCESSINFO = (*pcur).head.ppi;
    let ppcurHead: *mut PCURICON_OBJECT;

    ppcurHead = if ((*pcur).CURSORF_flags & CURSORF_GLOBAL) != 0 {
        core::ptr::addr_of_mut!(gcurFirst)
    } else {
        core::ptr::addr_of_mut!((*ppi).pCursorCache)
    };

    UserReferenceObject(pcur as _);
    (*pcur).pcurNext = *ppcurHead;
    *ppcurHead = pcur;
    (*pcur).CURSORF_flags |= CURSORF_LINKED;
}

unsafe fn IntRemoveCursorFromList(pcur: PCURICON_OBJECT) {
    let ppi: PPROCESSINFO = (*pcur).head.ppi;
    let ppcurHead: *mut PCURICON_OBJECT;
    let mut ppcur: *mut PCURICON_OBJECT;

    /* Get the right list head */
    ppcurHead = if ((*pcur).CURSORF_flags & CURSORF_GLOBAL) != 0 {
        core::ptr::addr_of_mut!(gcurFirst)
    } else {
        core::ptr::addr_of_mut!((*ppi).pCursorCache)
    };

    /* Loop all cursors in the cache */
    ppcur = ppcurHead;
    while !(*ppcur).is_null() {
        /* Check if this is the one we are looking for */
        if (*ppcur) == pcur {
            /* Remove it from the list */
            (*ppcur) = (*pcur).pcurNext;

            /* Dereference it */
            UserDereferenceObject(pcur as *mut core::ffi::c_void);
            (*pcur).CURSORF_flags &= !CURSORF_LINKED;
            return;
        }

        ppcur = core::ptr::addr_of_mut!((**ppcur).pcurNext);
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntLoadSystenIcons(hcur: HICON, id: DWORD) {
    let pcur: PCURICON_OBJECT;
    let mut i: i32;
    let ppi: PPROCESSINFO;

    if !hcur.is_null() {
        pcur = UserGetCurIconObject(hcur as _);
        if pcur.is_null() {
            EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
            return;
        }

        ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

        if ((*ppi_w32pf_flags(ppi)) & W32PF_CREATEDWINORDC) == 0 {
            return;
        }

        if id == OIC_INTERNAL_WINSMALL as DWORD {
            (*pcur).CURSORF_flags |= CURSORF_GLOBAL;
            UserReferenceObject(pcur as _);
            (*pcur).head.ppi = core::ptr::null_mut();
            return;
        }

        i = 0;
        while i < 6 {
            if gasysico[i as usize].type_ as DWORD == id {
                gasysico[i as usize].handle = pcur;
                (*pcur).CURSORF_flags |= CURSORF_GLOBAL;

                (*pcur).head.ppi = core::ptr::null_mut();

                IntInsertCursorIntoList(pcur);
                return;
            }
            i += 1;
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn IntGetSysCursorInfo() -> PSYSTEM_CURSORINFO {
    core::ptr::addr_of_mut!(gSysCursorInfo)
}

unsafe fn is_icon(object: PCURICON_OBJECT) -> BOOL {
    if (*object).rt as usize == RT_ICON as usize {
        1
    } else {
        0
    }
}

#[no_mangle]
#[cfg(target_arch = "x86")]
pub unsafe extern "fastcall" fn UserGetCurIconObject(hCurIcon: HCURSOR) -> PCURICON_OBJECT {
    UserGetCurIconObject_impl(hCurIcon)
}

#[no_mangle]
#[cfg(not(target_arch = "x86"))]
pub unsafe extern "C" fn UserGetCurIconObject(hCurIcon: HCURSOR) -> PCURICON_OBJECT {
    UserGetCurIconObject_impl(hCurIcon)
}

unsafe fn UserGetCurIconObject_impl(hCurIcon: HCURSOR) -> PCURICON_OBJECT {
    let CurIcon: PCURICON_OBJECT;

    if hCurIcon.is_null() {
        EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
        return core::ptr::null_mut();
    }

    if UserObjectInDestroy(hCurIcon as _) != 0 {
        EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
        return core::ptr::null_mut();
    }

    CurIcon = UserReferenceObjectByHandle(hCurIcon as _, TYPE_CURSOR as _) as PCURICON_OBJECT;
    if CurIcon.is_null() {
        EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
        return core::ptr::null_mut();
    }

    CurIcon
}

#[no_mangle]
pub unsafe extern "C" fn IntSystemSetCursor(pcurNew: PCURICON_OBJECT) -> PCURICON_OBJECT {
    let pcurOld = UserSetCursor(pcurNew, 0);
    if !pcurNew.is_null() {
        UserReferenceObject(pcurNew as *mut _);
    }
    if !pcurOld.is_null() {
        UserDereferenceObject(pcurOld as *mut _);
    }
    pcurOld
}

#[no_mangle]
pub unsafe extern "C" fn UserSetCursorPos(
    mut x: INT,
    mut y: INT,
    flags: DWORD,
    dwExtraInfo: ULONG_PTR,
    Hook: BOOL,
) -> BOOL {
    let DesktopWindow: PWND;
    let CurInfo: PSYSTEM_CURSORINFO;
    let mut Msg: MSG = core::mem::zeroed();
    let rcClip: RECTL;
    let mut pt: POINT = core::mem::zeroed();

    DesktopWindow = UserGetDesktopWindow();
    if DesktopWindow.is_null() {
        return 0;
    }

    CurInfo = IntGetSysCursorInfo();

    if (*CurInfo).bClipped == 0 {
        rcClip = (*DesktopWindow).rcClient;
    } else {
        rcClip = (*CurInfo).rcClip;
    }

    if x >= rcClip.right {
        x = rcClip.right - 1;
    }
    if x < rcClip.left {
        x = rcClip.left;
    }
    if y >= rcClip.bottom {
        y = rcClip.bottom - 1;
    }
    if y < rcClip.top {
        y = rcClip.top;
    }

    if x == (*gpsi_per_user()).ptCursor.x && y == (*gpsi_per_user()).ptCursor.y {
        return 1;
    }

    pt.x = x;
    pt.y = y;

    Msg.message = WM_MOUSEMOVE;
    Msg.wParam = UserGetMouseButtonsState() as _;
    Msg.lParam = MAKELPARAM(x as u16, y as u16) as _;
    Msg.pt = core::mem::transmute_copy(&pt);
    co_MsqInsertMouseMessage(core::ptr::addr_of_mut!(Msg), flags, dwExtraInfo, Hook);

    (*gpsi_per_user()).ptCursor = core::mem::transmute_copy(&pt);

    1
}

#[no_mangle]
pub unsafe extern "C" fn IntCreateCurIconHandle(Animated: BOOLEAN) -> HANDLE {
    let CurIcon: PCURICON_OBJECT;
    let mut hCurIcon: HANDLE = core::ptr::null_mut();

    CurIcon = UserCreateObject(
        gHandleTable,
        core::ptr::null_mut(),
        GetW32ThreadInfo() as _,
        &mut hCurIcon,
        TYPE_CURSOR,
        if Animated != 0 {
            core::mem::size_of::<ACON>() as _
        } else {
            core::mem::size_of::<CURICON_OBJECT>() as _
        },
    ) as PCURICON_OBJECT;

    if CurIcon.is_null() {
        EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return core::ptr::null_mut();
    }

    if Animated != 0 {
        (*CurIcon).CURSORF_flags |= CURSORF_ACON;
    }

    NT_ASSERT((*CurIcon).pcurNext.is_null());
    UserDereferenceObject(CurIcon as _);

    hCurIcon
}

#[no_mangle]
pub unsafe extern "C" fn IntDestroyCurIconObject(Object: PVOID) -> BOOLEAN {
    let CurIcon: PCURICON_OBJECT = Object as PCURICON_OBJECT;

    /* Check if the cursor is in a list */
    if ((*CurIcon).CURSORF_flags & CURSORF_LINKED) != 0 {
        /* Remove the cursor from it's list */
        IntRemoveCursorFromList(CurIcon);
    }

    /* We just mark the handle as being destroyed.
     * Deleting all the stuff will be deferred to the actual struct free. */
    UserDeleteObject(UserHMGetHandle(CurIcon), TYPE_CURSOR);
    1u8
}

#[no_mangle]
pub unsafe extern "C" fn FreeCurIconObject(Object: PVOID) {
    let CurIcon: PCURICON_OBJECT = Object as PCURICON_OBJECT;

    if ((*CurIcon).CURSORF_flags & CURSORF_ACON) == 0 {
        let bmpMask: HBITMAP = (*CurIcon).hbmMask;
        let bmpColor: HBITMAP = (*CurIcon).hbmColor;
        let bmpAlpha: HBITMAP = (*CurIcon).hbmAlpha;

        /* Delete bitmaps */
        if !bmpMask.is_null() {
            GreSetObjectOwner(bmpMask as _, GDI_OBJ_HMGR_POWNED);
            GreDeleteObject(bmpMask as _);
            (*CurIcon).hbmMask = core::ptr::null_mut();
        }
        if !bmpColor.is_null() {
            GreSetObjectOwner(bmpColor as _, GDI_OBJ_HMGR_POWNED);
            GreDeleteObject(bmpColor as _);
            (*CurIcon).hbmColor = core::ptr::null_mut();
        }
        if !bmpAlpha.is_null() {
            GreSetObjectOwner(bmpAlpha as _, GDI_OBJ_HMGR_POWNED);
            GreDeleteObject(bmpAlpha as _);
            (*CurIcon).hbmAlpha = core::ptr::null_mut();
        }
    } else {
        let AniCurIcon: PACON = CurIcon as PACON;
        let mut i: UINT = 0;

        while i < (*AniCurIcon).cpcur {
            UserDereferenceObject(*(*AniCurIcon).aspcur.offset(i as isize) as PVOID);
            IntDestroyCurIconObject(*(*AniCurIcon).aspcur.offset(i as isize) as PVOID);
            i += 1;
        }
        ExFreePoolWithTag((*AniCurIcon).aspcur as PVOID, USERTAG_CURSOR);
    }

    if ((*CurIcon).CURSORF_flags & CURSORF_LRSHARED) != 0 {
        if IS_INTRESOURCE((*CurIcon).strName.Buffer as PVOID) == 0 {
            ExFreePoolWithTag((*CurIcon).strName.Buffer as PVOID, TAG_STRING);
        }
        if (*CurIcon).atomModName != 0 {
            RtlDeleteAtomFromAtomTable(gAtomTable as _, (*CurIcon).atomModName);
        }
        (*CurIcon).strName.Buffer = core::ptr::null_mut();
        (*CurIcon).atomModName = 0;
    }

    /* Finally free the thing */
    FreeProcMarkObject(CurIcon as *mut c_void);
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCleanupCurIconCache(Win32Process: PPROCESSINFO) {
    IntCleanupCurIconCache_impl(Win32Process)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCleanupCurIconCache(Win32Process: PPROCESSINFO) {
    IntCleanupCurIconCache_impl(Win32Process)
}

unsafe fn IntCleanupCurIconCache_impl(Win32Process: PPROCESSINFO) {
    let mut CurIcon: PCURICON_OBJECT;

    while !(*Win32Process).pCursorCache.is_null() {
        CurIcon = (*Win32Process).pCursorCache;
        (*Win32Process).pCursorCache = (*CurIcon).pcurNext;
        UserDereferenceObject(CurIcon as _);
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetIconInfo(
    hCurIcon: HANDLE,
    IconInfo: PICONINFO,
    lpModule: PUNICODE_STRING,
    lpResName: PUNICODE_STRING,
    pbpp: LPDWORD,
    bInternal: BOOL,
) -> BOOL {
    let mut ii: ICONINFO = core::mem::zeroed();
    let CurIcon: PCURICON_OBJECT;
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut Ret: BOOL = 0;
    let mut colorBpp: DWORD = 0;

    /* Check if something was actually asked */
    if IconInfo.is_null() && lpModule.is_null() && lpResName.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    UserEnterExclusive();

    CurIcon = UserGetCurIconObject(hCurIcon as HCURSOR);
    if CurIcon.is_null() {
        UserLeave();
        return 0;
    }

    'leave: loop {
        /* Give back the icon information */
        if !IconInfo.is_null() {
            let mut FrameCurIcon: PCURICON_OBJECT = CurIcon;
            if (*CurIcon).CURSORF_flags & CURSORF_ACON != 0 {
                /* Get information from first frame. */
                FrameCurIcon = *(*(CurIcon as PACON)).aspcur.add(0);
            }

            /* Fill data */
            ii.fIcon = is_icon(FrameCurIcon);
            ii.xHotspot = (*FrameCurIcon).xHotspot as _;
            ii.yHotspot = (*FrameCurIcon).yHotspot as _;

            /* Copy bitmaps */
            ii.hbmMask = BITMAP_CopyBitmap((*FrameCurIcon).hbmMask);
            GreSetObjectOwner(ii.hbmMask as _, GDI_OBJ_HMGR_POWNED);
            ii.hbmColor = BITMAP_CopyBitmap((*FrameCurIcon).hbmColor);
            GreSetObjectOwner(ii.hbmColor as _, GDI_OBJ_HMGR_POWNED);
            colorBpp = (*FrameCurIcon).bpp;

            /* Copy fields */
            Status = SafeProbeForWrite(
                IconInfo as *mut core::ffi::c_void,
                core::mem::size_of::<ICONINFO>() as ULONG_PTR,
                1,
            );
            if NT_SUCCESS(Status) {
                Status = SafeProbeForWrite(
                    pbpp as *mut core::ffi::c_void,
                    core::mem::size_of::<DWORD>() as ULONG_PTR,
                    1,
                );
            }
            if NT_SUCCESS(Status) {
                // Probes are validate-only (USE_DUMMY_PSEH); the C had probe AND
                // store in one _SEH2_TRY, so the stores must be the fault-tolerant
                // copy. RtlCopyMemory into a user buffer is a plain memcpy.
                if !crate::copy::publish_to_user(
                    IconInfo as *mut core::ffi::c_void,
                    core::ptr::addr_of!(ii) as *const core::ffi::c_void,
                    core::mem::size_of::<ICONINFO>(),
                ) {
                    Status = STATUS_ACCESS_VIOLATION;
                }
                if NT_SUCCESS(Status)
                    && !pbpp.is_null()
                    && !crate::copy::publish_to_user(
                        pbpp as *mut core::ffi::c_void,
                        core::ptr::addr_of!(colorBpp) as *const core::ffi::c_void,
                        core::mem::size_of_val(&colorBpp),
                    )
                {
                    Status = STATUS_ACCESS_VIOLATION;
                }
            }

            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                break 'leave;
            }
        }

        /* Give back the module name */
        if !lpModule.is_null() {
            let mut BufLen: ULONG = 0;
            if (*CurIcon).atomModName == 0 {
                break 'leave;
            }

            RtlQueryAtomInAtomTable(
                gAtomTable as _,
                (*CurIcon).atomModName,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut BufLen,
            );
            /* Get the module name from the atom table */
            // C (cursoricon.c:497-522, NtUserGetIconInfo), one _SEH2_TRY whose
            // handler is `Status = _SEH2_GetExceptionCode();`, falling through to
            // the `if (!NT_SUCCESS(Status))` check below:
            //   BufLen += sizeof(WCHAR);
            //   if (BufLen > lpModule->MaximumLength) { lpModule->Length = 0;
            //       lpModule->MaximumLength = BufLen; }
            //   else { ProbeForWrite(lpModule->Buffer, lpModule->MaximumLength, 1);
            //       BufLen = lpModule->MaximumLength;
            //       RtlQueryAtomInAtomTable(.., lpModule->Buffer, &BufLen);
            //       lpModule->Length = BufLen; }
            // Every touch of lpModule and its Buffer is fault-tolerant here: the
            // header is captured ONCE (MaximumLength sizes the copy, so it must
            // not be re-fetched), the atom name is queried into a pool buffer and
            // published with copy_to_user, and the header fields are written back.
            //
            // Divergence: the C never probes lpModule itself before writing its
            // fields; a kernel-range pointer would be written through. Probe it.
            Status = (|| -> Result<(), NTSTATUS> {
                crate::safe_user_io::probe_for_write(
                    lpModule as *mut c_void,
                    core::mem::size_of::<_UNICODE_STRING>(),
                    1,
                )?;
                let Module: _UNICODE_STRING = ntkrnl_sys::probe::read_back(lpModule as *const _UNICODE_STRING)?;
                BufLen += core::mem::size_of::<WCHAR>() as ULONG;
                if BufLen > Module.MaximumLength as ULONG {
                    ntkrnl_sys::probe::write_back(core::ptr::addr_of_mut!((*lpModule).Length), 0 as USHORT)?;
                    ntkrnl_sys::probe::write_back(
                        core::ptr::addr_of_mut!((*lpModule).MaximumLength),
                        BufLen as USHORT,
                    )?;
                    return Ok(());
                }
                crate::safe_user_io::probe_for_write(
                    Module.Buffer as *mut c_void,
                    Module.MaximumLength as usize,
                    1,
                )?;
                // BufLen > 0 here, so MaximumLength >= 2 and the pool size is non-zero.
                let cb = Module.MaximumLength as usize;
                let tag = u32::from_le_bytes(*b"Usgm");
                let pool = ExAllocatePoolWithTag(PagedPool, cb as _, tag) as *mut WCHAR;
                if pool.is_null() {
                    return Err(0xC000009A_u32 as NTSTATUS); // STATUS_INSUFFICIENT_RESOURCES
                }
                let _pool_guard = ntkrnl_sys::probe::PoolAllocGuard::new(pool as _, tag);
                RtlZeroMemory(pool as *mut c_void, cb as _);
                BufLen = Module.MaximumLength as ULONG;
                RtlQueryAtomInAtomTable(
                    gAtomTable as _,
                    (*CurIcon).atomModName,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    pool as _,
                    &mut BufLen,
                );
                // The name plus its NUL when it fits: exactly the bytes the C's
                // in-place query would have stored.
                let publish = core::cmp::min(BufLen as usize + core::mem::size_of::<WCHAR>(), cb);
                ntkrnl_sys::probe::safe_copy_raw(Module.Buffer as *mut c_void, pool as *const c_void, publish)?;
                ntkrnl_sys::probe::write_back(core::ptr::addr_of_mut!((*lpModule).Length), BufLen as USHORT)?;
                Ok(())
            })()
            .err()
            .unwrap_or(Status);

            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                break 'leave;
            }
        }

        if !lpResName.is_null() {
            if (*CurIcon).strName.Buffer.is_null() {
                break 'leave;
            }

            /* Copy it */
            Status = SafeProbeForWrite(
                lpResName as *mut core::ffi::c_void,
                core::mem::size_of::<_UNICODE_STRING>() as ULONG_PTR,
                1,
            );
            // Probe is validate-only (USE_DUMMY_PSEH); the C had the probes AND every
            // one of these loads and stores inside one _SEH2_TRY. Capture the
            // UNICODE_STRING into kernel memory, mutate it there, publish once —
            // note the Buffer probe below needs the CAPTURED length, since reading it
            // back out of user memory is the very access being protected.
            let mut SafeName: _UNICODE_STRING = core::mem::zeroed();
            if NT_SUCCESS(Status)
                && !crate::copy::capture_from_user(
                    core::ptr::addr_of_mut!(SafeName) as *mut core::ffi::c_void,
                    lpResName as *const core::ffi::c_void,
                    core::mem::size_of::<_UNICODE_STRING>(),
                )
            {
                Status = STATUS_ACCESS_VIOLATION;
            }
            if NT_SUCCESS(Status) {
                Status = SafeProbeForWrite(
                    SafeName.Buffer as *mut core::ffi::c_void,
                    SafeName.MaximumLength as ULONG_PTR,
                    1,
                );
            }
            if NT_SUCCESS(Status) {
                let mut ok = true;
                if IS_INTRESOURCE((*CurIcon).strName.Buffer) != 0 {
                    SafeName.Buffer = (*CurIcon).strName.Buffer;
                    SafeName.Length = 0;
                    SafeName.MaximumLength = 0;
                } else if SafeName.MaximumLength < (*CurIcon).strName.MaximumLength {
                    SafeName.Length = 0;
                    SafeName.MaximumLength = (*CurIcon).strName.MaximumLength;
                } else {
                    ok = crate::copy::publish_to_user(
                        SafeName.Buffer as *mut core::ffi::c_void,
                        (*CurIcon).strName.Buffer as *const core::ffi::c_void,
                        (*CurIcon).strName.Length as usize,
                    );
                    SafeName.Length = (*CurIcon).strName.Length;
                }
                if !ok
                    || !crate::copy::publish_to_user(
                        lpResName as *mut core::ffi::c_void,
                        core::ptr::addr_of!(SafeName) as *const core::ffi::c_void,
                        core::mem::size_of::<_UNICODE_STRING>(),
                    )
                {
                    Status = STATUS_ACCESS_VIOLATION;
                }
            }
        }

        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'leave;
        }

        Ret = 1;
        break 'leave;
    }

    UserDereferenceObject(CurIcon as *mut core::ffi::c_void);

    UserLeave();

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetIconSize(
    hCurIcon: HANDLE,
    istepIfAniCur: UINT,
    plcx: PLONG,
    plcy: PLONG,
) -> BOOL {
    let mut CurIcon: PCURICON_OBJECT;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut bRet: BOOL = 0;

    UserEnterShared();

    'cleanup: {
        CurIcon = UserGetCurIconObject(hCurIcon as _);
        if CurIcon.is_null() {
            break 'cleanup;
        }

        if (*CurIcon).CURSORF_flags & CURSORF_ACON != 0 {
            let AniCurIcon: PACON = CurIcon as PACON;
            CurIcon = *(*AniCurIcon).aspcur.add(0);
            UserDereferenceObject(AniCurIcon as *mut _);
            UserReferenceObject(CurIcon as *mut _);
        }

        Status = SafeProbeForWrite(plcx as *mut c_void, core::mem::size_of::<LONG>() as _, 1);
        if NT_SUCCESS(Status) {
            Status = SafeProbeForWrite(plcy as *mut c_void, core::mem::size_of::<LONG>() as _, 1);
        }
            // The probes VALIDATE only; these STORES are the access. A raw write into the
        // caller's page faults with no handler under DUMMY_PSEH, so publish both
        // through the protected copy and record a fault in Status, which is checked
        // just below.
        if NT_SUCCESS(Status) {
            let cx: LONG = (*CurIcon).cx as LONG;
            let cy: LONG = (*CurIcon).cy as LONG;
            if !crate::copy::publish_to_user(
                plcx as *mut c_void,
                core::ptr::addr_of!(cx) as *const c_void,
                core::mem::size_of::<LONG>(),
            ) || !crate::copy::publish_to_user(
                plcy as *mut c_void,
                core::ptr::addr_of!(cy) as *const c_void,
                core::mem::size_of::<LONG>(),
            ) {
                Status = STATUS_ACCESS_VIOLATION;
            }
        }

        if NT_SUCCESS(Status) {
            bRet = 1;
        } else {
            SetLastNtError(Status);
        }

        UserDereferenceObject(CurIcon as *mut _);
    }

    UserLeave();
    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetCursorInfo(pci: PCURSORINFO) -> BOOL {
    let mut SafeCi: CURSORINFO = core::mem::zeroed();
    let CurInfo: PSYSTEM_CURSORINFO;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let CurIcon: PCURICON_OBJECT;
    let mut Ret: BOOL = 0;

    UserEnterShared();

    CurInfo = IntGetSysCursorInfo();
    CurIcon = (*CurInfo).CurrentCursorObject as PCURICON_OBJECT;

    SafeCi.cbSize = core::mem::size_of::<CURSORINFO>() as u32;
    SafeCi.flags = if !CurIcon.is_null() && (*CurInfo).ShowingCursor >= 0 {
        CURSOR_SHOWING
    } else {
        0
    };
    SafeCi.hCursor = if !CurIcon.is_null() {
        UserHMGetHandle(CurIcon) as _
    } else {
        core::ptr::null_mut()
    };

    SafeCi.ptScreenPos = (*gpsi_per_user()).ptCursor;

    // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
    // C (cursoricon.c:658-681, NtUserGetCursorInfo): handler is
    // `Status = _SEH2_GetExceptionCode();`, no return/goto -- falls through
    // to the `if (!NT_SUCCESS(Status))` check that already exists right
    // below. The probe itself, `ProbeForWrite(pci, sizeof(CURSORINFO), 1)`,
    // was dropped along with the handler (not just the handler) -- the raw
    // `(*pci).cbSize` read and the RtlCopyMemory write below were left
    // completely unprotected. Restored via SafeProbeForWrite, alignment 1
    // matching the C's literal argument.
    Status = SafeProbeForWrite(
        pci as *mut core::ffi::c_void,
        core::mem::size_of::<CURSORINFO>() as SIZE_T,
        1,
    );
    if NT_SUCCESS(Status) {
        // Probe is validate-only (USE_DUMMY_PSEH); the C had probe, load and store
        // in one _SEH2_TRY. Both directions go through the fault-tolerant copy.
        let mut cbSize: u32 = 0;
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(cbSize) as *mut core::ffi::c_void,
            core::ptr::addr_of!((*pci).cbSize) as *const core::ffi::c_void,
            core::mem::size_of::<u32>(),
        ) {
            Status = STATUS_ACCESS_VIOLATION;
        } else if cbSize == core::mem::size_of::<CURSORINFO>() as u32 {
            if crate::copy::publish_to_user(
                pci as *mut core::ffi::c_void,
                core::ptr::addr_of!(SafeCi) as *const core::ffi::c_void,
                core::mem::size_of::<CURSORINFO>(),
            ) {
                Ret = 1;
            } else {
                Status = STATUS_ACCESS_VIOLATION;
            }
        } else {
            EngSetLastError(ERROR_INVALID_PARAMETER);
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
    }

    UserLeave();

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn UserClipCursor(prcl: *mut RECTL) -> BOOL {
    let CurInfo: PSYSTEM_CURSORINFO;
    let DesktopWindow: PWND;

    if CheckWinstaAttributeAccess(WINSTA_WRITEATTRIBUTES as _) == 0 {
        return 0;
    }

    CurInfo = IntGetSysCursorInfo();

    DesktopWindow = UserGetDesktopWindow();

    if !prcl.is_null() && !DesktopWindow.is_null() {
        if (*prcl).right < (*prcl).left || (*prcl).bottom < (*prcl).top {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        (*CurInfo).bClipped = 1;

        (*CurInfo).rcClip.left = max((*prcl).left, (*DesktopWindow).rcWindow.left);
        (*CurInfo).rcClip.right = min((*prcl).right, (*DesktopWindow).rcWindow.right);
        if (*CurInfo).rcClip.right < (*CurInfo).rcClip.left {
            (*CurInfo).rcClip.right = (*CurInfo).rcClip.left;
        }

        (*CurInfo).rcClip.top = max((*prcl).top, (*DesktopWindow).rcWindow.top);
        (*CurInfo).rcClip.bottom = min((*prcl).bottom, (*DesktopWindow).rcWindow.bottom);
        if (*CurInfo).rcClip.bottom < (*CurInfo).rcClip.top {
            (*CurInfo).rcClip.bottom = (*CurInfo).rcClip.top;
        }

        UserSetCursorPos((*gpsi_per_user()).ptCursor.x, (*gpsi_per_user()).ptCursor.y, 0, 0, 0);
    } else {
        (*CurInfo).bClipped = 0;
    }

    1
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn NtUserClipCursor(mut prcl: *mut RECTL) -> BOOL {
    let mut rclLocal: RECTL = core::mem::zeroed();
    let bResult: BOOL;

    if !prcl.is_null() {
        let __SehStatus = SafeProbeForRead(prcl as *mut _, core::mem::size_of::<RECTL>() as u32, 1);
        if !NT_SUCCESS(__SehStatus) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(rclLocal) as *mut core::ffi::c_void,
            prcl as *const core::ffi::c_void,
            core::mem::size_of::<RECTL>(),
        ) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        prcl = core::ptr::addr_of_mut!(rclLocal);
    }

    UserEnterExclusive();

    bResult = UserClipCursor(prcl);

    UserLeave();

    bResult
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn NtUserClipCursor(mut prcl: *mut RECTL) -> BOOL {
    let mut rclLocal: RECTL = core::mem::zeroed();
    let bResult: BOOL;

    if !prcl.is_null() {
        let __SehStatus = SafeProbeForRead(prcl as *mut _, core::mem::size_of::<RECTL>() as _, 1);
        if !NT_SUCCESS(__SehStatus) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        // Probe is validate-only; the C had probe AND load in one _SEH2_TRY.
        if !crate::copy::capture_from_user(
            core::ptr::addr_of_mut!(rclLocal) as *mut core::ffi::c_void,
            prcl as *const core::ffi::c_void,
            core::mem::size_of::<RECTL>(),
        ) {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            return 0;
        }

        prcl = core::ptr::addr_of_mut!(rclLocal);
    }

    UserEnterExclusive();

    bResult = UserClipCursor(prcl);

    UserLeave();

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDestroyCursor(hCurIcon: HANDLE, bForce: BOOL) -> BOOL {
    let mut ret: BOOL = 0;
    let mut CurIcon: PCURICON_OBJECT = core::ptr::null_mut();

    UserEnterExclusive();

    'leave: {
        CurIcon = UserGetCurIconObject(hCurIcon as HCURSOR);
        if CurIcon.is_null() {
            ret = 0;
            break 'leave;
        }

        if bForce == 0 {
            /* Can not destroy global objects */
            if (*CurIcon).head.ppi.is_null() {
                ret = 1;
                break 'leave;
            }

            /* Maybe we have good reasons not to destroy this object */
            if (*CurIcon).head.ppi != PsGetCurrentProcessWin32Process() as PPROCESSINFO {
                ret = 0;
                break 'leave;
            }

            if ((*CurIcon).CURSORF_flags & CURSORF_CURRENT) != 0 {
                ret = 0;
                break 'leave;
            }

            if ((*CurIcon).CURSORF_flags & CURSORF_LRSHARED) != 0 {
                /* This one is not an error */
                ret = 1;
                break 'leave;
            }
        }

        /* Destroy the handle */
        ret = IntDestroyCurIconObject(CurIcon as PVOID) as BOOL;
    }

    if !CurIcon.is_null() {
        UserDereferenceObject(CurIcon as PVOID);
    }
    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserFindExistingCursorIcon(
    pustrModule: PUNICODE_STRING,
    pustrRsrc: PUNICODE_STRING,
    param: *mut FINDEXISTINGCURICONPARAM,
) -> HICON {
    let mut CurIcon: PCURICON_OBJECT;
    let mut Ret: HICON = core::ptr::null_mut();
    let mut ustrModuleSafe: _UNICODE_STRING = core::mem::zeroed();
    let mut ustrRsrcSafe: _UNICODE_STRING = core::mem::zeroed();
    let mut paramSafe: FINDEXISTINGCURICONPARAM = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let pProcInfo: PPROCESSINFO = PsGetCurrentProcessWin32Process() as PPROCESSINFO;
    let mut atomModName: RTL_ATOM = 0;

    Status = SafeProbeForRead(
        param as *mut core::ffi::c_void,
        core::mem::size_of::<FINDEXISTINGCURICONPARAM>() as _,
        1,
    );
    // Two defects at this one site. The probe is validate-only, so the copy had to
    // become the protected one -- and the probe's verdict was being THROWN AWAY: the
    // next statement overwrites Status with ProbeAndCaptureUnicodeStringOrAtom's
    // result, so a failed probe here fell through and the function carried on with a
    // zeroed paramSafe instead of failing. Both faults now return.
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }
    if !crate::copy::capture_from_user(
        core::ptr::addr_of_mut!(paramSafe) as *mut c_void,
        param as *const c_void,
        core::mem::size_of::<FINDEXISTINGCURICONPARAM>(),
    ) {
        SetLastNtError(STATUS_ACCESS_VIOLATION);
        return core::ptr::null_mut();
    }

    Status = ProbeAndCaptureUnicodeStringOrAtom(core::ptr::addr_of_mut!(ustrRsrcSafe), pustrRsrc);
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    'done: {
        // The translation had a second capture here, of pustrModule INTO ustrRsrcSafe. It
        // overwrote the resource name with the module path, so no LR_SHARED icon ever matched:
        // every LoadIcon/LoadImage(LR_SHARED) minted a new icon and two bitmaps, and the real
        // resource capture leaked its pool buffer. A repainting app hit the 10K GDI quota, and
        // win32k then fell over on the NULL regions (REGION_Delete(NULL)) or pool exhaustion.
        Status = ProbeAndCaptureUnicodeString(
            core::ptr::addr_of_mut!(ustrModuleSafe),
            1i32 as i8,
            pustrModule,
        );
        if !NT_SUCCESS(Status) {
            break 'done;
        }
        Status = RtlLookupAtomInAtomTable(
            gAtomTable as _,
            ustrModuleSafe.Buffer,
            core::ptr::addr_of_mut!(atomModName),
        );
        ReleaseCapturedUnicodeString(core::ptr::addr_of_mut!(ustrModuleSafe), 1i32 as i8);
        if !NT_SUCCESS(Status) {
            break 'done;
        }

        UserEnterShared();
        CurIcon = (*pProcInfo).pCursorCache;
        while !CurIcon.is_null() {
            if paramSafe.bIcon != is_icon(CurIcon) {
                CurIcon = (*CurIcon).pcurNext;
                continue;
            }
            if atomModName == (*CurIcon).atomModName {
                if IS_INTRESOURCE((*CurIcon).strName.Buffer as *mut core::ffi::c_void)
                    != IS_INTRESOURCE(ustrRsrcSafe.Buffer as *mut core::ffi::c_void)
                {
                    CurIcon = (*CurIcon).pcurNext;
                    continue;
                }

                if IS_INTRESOURCE((*CurIcon).strName.Buffer as *mut core::ffi::c_void) != 0 {
                    if (*CurIcon).strName.Buffer == ustrRsrcSafe.Buffer {
                        break;
                    }
                } else if RtlCompareUnicodeString(
                    core::ptr::addr_of_mut!(ustrRsrcSafe) as _,
                    core::ptr::addr_of_mut!((*CurIcon).strName) as _,
                    1u8,
                ) == 0
                {
                    break;
                }
            }
            CurIcon = (*CurIcon).pcurNext;
        }

        if CurIcon.is_null() {
            CurIcon = gcurFirst;
            while !CurIcon.is_null() {
                if paramSafe.bIcon != is_icon(CurIcon) {
                    CurIcon = (*CurIcon).pcurNext;
                    continue;
                }
                if atomModName == (*CurIcon).atomModName {
                    if IS_INTRESOURCE((*CurIcon).strName.Buffer as *mut core::ffi::c_void)
                        != IS_INTRESOURCE(ustrRsrcSafe.Buffer as *mut core::ffi::c_void)
                    {
                        CurIcon = (*CurIcon).pcurNext;
                        continue;
                    }
                    if IS_INTRESOURCE((*CurIcon).strName.Buffer as *mut core::ffi::c_void) != 0 {
                        if (*CurIcon).strName.Buffer == ustrRsrcSafe.Buffer {
                            break;
                        }
                    } else if RtlCompareUnicodeString(
                        core::ptr::addr_of_mut!(ustrRsrcSafe) as _,
                        core::ptr::addr_of_mut!((*CurIcon).strName) as _,
                        1u8,
                    ) == 0
                    {
                        break;
                    }
                }
                CurIcon = (*CurIcon).pcurNext;
            }
        }
        if !CurIcon.is_null() {
            Ret = UserHMGetHandle(CurIcon) as _;
        }
        UserLeave();
    }

    if IS_INTRESOURCE(ustrRsrcSafe.Buffer as *mut core::ffi::c_void) == 0 {
        ExFreePoolWithTag(ustrRsrcSafe.Buffer as *mut core::ffi::c_void, TAG_STRING);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetClipCursor(lpRect: *mut RECTL) -> BOOL {
    let CurInfo: PSYSTEM_CURSORINFO;
    let mut Rect: RECTL = core::mem::zeroed();
    let Status: NTSTATUS;
    let mut Ret: BOOL = 0;

    UserEnterShared();

    'exit: {
        if CheckWinstaAttributeAccess(WINSTA_READATTRIBUTES) == 0 {
            break 'exit;
        }

        if lpRect.is_null() {
            break 'exit;
        }

        CurInfo = IntGetSysCursorInfo();
        if (*CurInfo).bClipped != 0 {
            Rect = (*CurInfo).rcClip;
        } else {
            Rect.left = 0;
            Rect.top = 0;
            Rect.right = UserGetSystemMetrics(SM_CXSCREEN as ULONG);
            Rect.bottom = UserGetSystemMetrics(SM_CYSCREEN as ULONG);
        }

        Status = MmCopyToCaller(
            lpRect as *mut core::ffi::c_void,
            core::ptr::addr_of!(Rect) as *const core::ffi::c_void,
            core::mem::size_of::<RECT>() as ULONG,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'exit;
        }

        Ret = 1;
    }

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetCursor(hCursor: HCURSOR) -> HCURSOR {
    let pcurOld: PCURICON_OBJECT;
    let pcurNew: PCURICON_OBJECT;
    let mut hOldCursor: HCURSOR = core::ptr::null_mut();

    UserEnterExclusive();

    'leave: {
        if !hCursor.is_null() {
            pcurNew = UserGetCurIconObject(hCursor);
            if pcurNew.is_null() {
                EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
                break 'leave;
            }
            (*pcurNew).CURSORF_flags |= CURSORF_CURRENT;
        } else {
            pcurNew = core::ptr::null_mut();
        }

        let mut pcurOld_var = UserSetCursor(pcurNew, 0);

        if !pcurOld_var.is_null() {
            pcurOld_var = UserGetObjectNoErr(
                gHandleTable,
                UserHMGetHandle(pcurOld_var),
                TYPE_CURSOR,
            ) as PCURICON_OBJECT;

            if !pcurOld_var.is_null() {
                pcurOld = pcurOld_var;
                hOldCursor = UserHMGetHandle(pcurOld) as _;

                if ((*pcurOld).CURSORF_flags & CURSORF_GLOBAL) != 0 {
                    TRACE!(
                        b"Returning Global Cursor hcur %p\n\0".as_ptr() as *const i8,
                        hOldCursor,
                    );
                }

                if UserObjectInDestroy(hOldCursor as _) != 0 {
                    hOldCursor = core::ptr::null_mut();
                }
                (*pcurOld).CURSORF_flags &= !CURSORF_CURRENT;
                UserDereferenceObject(pcurOld as *mut core::ffi::c_void);
            }
        }
    }

    UserLeave();
    hOldCursor
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetCursorContents(
    hCurIcon: HANDLE,
    UnsafeIconInfo: PICONINFO,
) -> BOOL {
    0
}

unsafe fn IntSetCursorData(
    pcur: PCURICON_OBJECT,
    pustrName: PUNICODE_STRING,
    atomModName: ATOM,
    pcursordata: *const CURSORDATA,
) -> u32 {
    if (*pcursordata).CURSORF_flags & CURSORF_ACON != 0 {
        return 0;
    }

    if !(*pcur).hbmMask.is_null() {
        return 0;
    }

    if (*pcursordata).hbmMask.is_null() {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if GreSetBitmapOwner((*pcursordata).hbmMask, GDI_OBJ_HMGR_PUBLIC as ULONG) == 0 {
        return 0;
    }

    if !(*pcursordata).hbmColor.is_null() {
        if GreSetBitmapOwner((*pcursordata).hbmColor, GDI_OBJ_HMGR_PUBLIC as ULONG) == 0 {
            GreSetBitmapOwner((*pcursordata).hbmMask, GDI_OBJ_HMGR_POWNED as ULONG);
            return 0;
        }
    }

    if !(*pcursordata).hbmAlpha.is_null() {
        if GreSetBitmapOwner((*pcursordata).hbmAlpha, GDI_OBJ_HMGR_PUBLIC as ULONG) == 0 {
            GreSetBitmapOwner((*pcursordata).hbmMask, GDI_OBJ_HMGR_POWNED as ULONG);
            if !(*pcursordata).hbmColor.is_null() {
                GreSetBitmapOwner((*pcursordata).hbmColor, GDI_OBJ_HMGR_POWNED as ULONG);
            }
            return 0;
        }
    }

    if !(*pcur).strName.Buffer.is_null() {
        RtlInitEmptyUnicodeString(
            core::ptr::addr_of_mut!((*pcur).strName) as _,
            core::ptr::null_mut(),
            0,
        );
    }

    if (*pcur).atomModName != 0 {
        RtlDeleteAtomFromAtomTable(gAtomTable as _, (*pcur).atomModName);
    }

    (*pcur).atomModName = atomModName;
    (*pcur).rt = (*pcursordata).rt;
    (*pcur).CURSORF_flags = (*pcursordata).CURSORF_flags & CURSORF_USER_MASK;
    (*pcur).xHotspot = (*pcursordata).xHotspot;
    (*pcur).yHotspot = (*pcursordata).yHotspot;
    (*pcur).hbmMask = (*pcursordata).hbmMask;
    (*pcur).hbmColor = (*pcursordata).hbmColor;
    (*pcur).hbmAlpha = (*pcursordata).hbmAlpha;
    (*pcur).rcBounds.left = 0;
    (*pcur).rcBounds.top = 0;
    (*pcur).rcBounds.right = (*pcursordata).cx as LONG;
    (*pcur).rcBounds.bottom = (*pcursordata).cy as LONG;
    (*pcur).hbmUserAlpha = (*pcursordata).hbmUserAlpha;
    (*pcur).bpp = (*pcursordata).bpp;
    (*pcur).cx = (*pcursordata).cx;
    (*pcur).cy = (*pcursordata).cy;
    if !pustrName.is_null() {
        (*pcur).strName = *pustrName;
    }

    1
}

unsafe fn IntSetAconData(
    pacon: PACON,
    pustrName: PUNICODE_STRING,
    atomModName: ATOM,
    pcursordata: *const CURSORDATA,
) -> BOOL {
    let aspcur: *mut PCURICON_OBJECT;
    let aicur: *mut DWORD;
    let ajifRate: *mut INT;
    let pcdFrame: PCURSORDATA;
    let hcurFrame: HCURSOR;
    let cjSize: UINT;
    let mut i: UINT;

    /* Check if the CURSORF_ACON is also set in the cursor data */
    if (*pcursordata).CURSORF_flags & CURSORF_ACON == 0 {
        return 0;
    }

    /* Check if this acon was already set */
    if !(*pacon).aspcur.is_null() {
        return 0;
    }

    /* Loop all frames indexes */
    i = 0;
    while i < (*pcursordata).cicur as UINT {
        /* Check if the index is within the range of the frames */
        if *(*pcursordata).aicur.add(i as usize) >= (*pcursordata).cpcur as DWORD {
            return 0;
        }
        i += 1;
    }

    /* Calculate size */
    cjSize = ((*pcursordata).cpcur as usize * core::mem::size_of::<*mut CURICON_OBJECT>()
        + (*pcursordata).cicur as usize * core::mem::size_of::<DWORD>()
        + (*pcursordata).cicur as usize * core::mem::size_of::<INT>()) as UINT;

    /* Allocate a buffer */
    aspcur = ExAllocatePoolWithTag(PagedPool, cjSize as _, USERTAG_CURSOR) as *mut PCURICON_OBJECT;
    if aspcur.is_null() {
        return 0;
    }

    /* Set the pointers */
    aicur = aspcur.add((*pcursordata).cpcur as usize) as *mut DWORD;
    ajifRate = aicur.add((*pcursordata).cicur as usize) as *mut INT;

    /* Copy the values */
    RtlCopyMemory(
        aicur as PVOID,
        (*pcursordata).aicur as PVOID,
        (*pcursordata).cicur as usize * core::mem::size_of::<DWORD>(),
    );
    RtlCopyMemory(
        ajifRate as PVOID,
        (*pcursordata).ajifRate as PVOID,
        (*pcursordata).cicur as usize * core::mem::size_of::<INT>(),
    );

    /* Zero out the array */
    RtlZeroMemory(
        aspcur as PVOID,
        (*pcursordata).cpcur as usize * core::mem::size_of::<PCURICON_OBJECT>(),
    );

    /* Get a pointer to the cursor data for each frame */
    pcdFrame = (*pcursordata).aspcur;

    let success = 'cleanup: {
        /* Create the cursors */
        i = 0;
        while i < (*pcursordata).cpcur as UINT {
            /* Create a cursor for this frame */
            let hcurFrame = IntCreateCurIconHandle(0u8) as HCURSOR;
            if hcurFrame.is_null() {
                break 'cleanup false;
            }

            /* Get a pointer to the frame cursor */
            *aspcur.add(i as usize) = UserGetCurIconObject(hcurFrame);

            /* Check if the flags are valid */
            if (*pcdFrame).CURSORF_flags & !(CURSORF_USER_MASK | CURSORF_ACONFRAME) != 0 {
                break 'cleanup false;
            }

            /* Set the cursor data for this frame */
            if IntSetCursorData(
                *aspcur.add(i as usize),
                core::ptr::null_mut(),
                0,
                pcdFrame.add(i as usize),
            ) == 0
            {
                break 'cleanup false;
            }

            /* Mark this cursor as an acon frame */
            (**aspcur.add(i as usize)).CURSORF_flags |= CURSORF_ACONFRAME;

            i += 1;
        }

        /* Free the old name (Must be NULL atm.) */
        if !(*pacon).strName.Buffer.is_null() {
            if IS_INTRESOURCE((*pacon).strName.Buffer as PVOID) == 0 {
                ExFreePoolWithTag((*pacon).strName.Buffer as PVOID, TAG_STRING);
            }
            RtlInitEmptyUnicodeString(core::ptr::addr_of_mut!((*pacon).strName) as _, core::ptr::null_mut(), 0);
        }

        /* Free the module atom */
        if (*pacon).atomModName != 0 {
            RtlDeleteAtomFromAtomTable(gAtomTable as _, (*pacon).atomModName);
        }

        /* Free the previous frames */
        if !(*pacon).aspcur.is_null() {
            i = 0;
            while i < (*pacon).cpcur as UINT {
                UserDereferenceObject(*(*pacon).aspcur.add(i as usize) as PVOID);
                IntDestroyCurIconObject(*(*pacon).aspcur.add(i as usize) as PVOID);
                i += 1;
            }
            ExFreePoolWithTag((*pacon).aspcur as PVOID, USERTAG_CURSOR);
        }

        /* Finally set the data in the acon */
        (*pacon).atomModName = atomModName;
        (*pacon).rt = (*pcursordata).rt;
        (*pacon).CURSORF_flags = (*pcursordata).CURSORF_flags & CURSORF_USER_MASK;
        (*pacon).cpcur = (*pcursordata).cpcur;
        (*pacon).cicur = (*pcursordata).cicur;
        (*pacon).aspcur = aspcur;
        (*pacon).aicur = aicur;
        (*pacon).ajifRate = ajifRate;
        (*pacon).iicur = 0;
        if !pustrName.is_null() {
            (*pacon).strName = *pustrName;
        }

        return 1;
    };

    let _ = success;

    /* Cleanup */
    i = 0;
    while i < (*pcursordata).cpcur as UINT {
        if (*aspcur.add(i as usize)).is_null() {
            break;
        }

        /* Destroy this cursor */
        UserDereferenceObject(*aspcur.add(i as usize) as PVOID);
        IntDestroyCurIconObject(*aspcur.add(i as usize) as PVOID);

        i += 1;
    }

    /* Delete the allocated structure */
    ExFreePoolWithTag(aspcur as PVOID, USERTAG_CURSOR);

    0
}

#[no_mangle]
pub unsafe extern "system" fn UserSetCursorIconData(
    hcursor: HCURSOR,
    pustrModule: PUNICODE_STRING,
    pustrRsrc: PUNICODE_STRING,
    pcursordata: PCURSORDATA,
) -> BOOL {
    let pcur: PCURICON_OBJECT;
    let mut atomModName: ATOM;
    let mut status: NTSTATUS;
    let mut bResult: BOOL;

    /* Do we have a module name? */
    if pustrModule != core::ptr::null_mut() {
        /* Create an atom for the module name */
        atomModName = 0;
        status = RtlAddAtomToAtomTable(
            gAtomTable as _,
            (*pustrModule).Buffer,
            core::ptr::addr_of_mut!(atomModName),
        );
        if !NT_SUCCESS(status) {
            return 0;
        }
    } else {
        /* No module name atom */
        atomModName = 0;
    }

    /* Reference the cursor */
    pcur = UserGetCurIconObject(hcursor);
    'exit: {
        if pcur == core::ptr::null_mut() {
            bResult = 0;
            break 'exit;
        }

        /* Check if this is an acon */
        if (*pcur).CURSORF_flags & CURSORF_ACON != 0 {
            bResult = IntSetAconData(
                pcur as PACON,
                pustrRsrc,
                atomModName,
                pcursordata,
            );
        } else {
            bResult = IntSetCursorData(
                pcur,
                pustrRsrc,
                atomModName,
                pcursordata,
            ) as BOOL;
        }
    }

    /* Check if we had success */
    if bResult != 0 {
        /* Check if this is an LRSHARED cursor now */
        if !pcur.is_null() && (*pcur).CURSORF_flags & CURSORF_LRSHARED != 0 {
            /* Insert the cursor into the list. */
            IntInsertCursorIntoList(pcur);
        }
    } else {
        /* Cleanup on failure */
        if atomModName != 0 {
            RtlDeleteAtomFromAtomTable(gAtomTable as _, atomModName);
        }
    }

    /* Dereference the cursor and return the result */
    if !pcur.is_null() {
        UserDereferenceObject(pcur as _);
    }

    bResult
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetCursorIconData(
    hcursor: HCURSOR,
    pustrModule: PUNICODE_STRING,
    pustrRsrc: PUNICODE_STRING,
    pCursorData: *const CURSORDATA,
) -> BOOL {
    let mut cursordata: CURSORDATA = core::mem::zeroed();
    let mut ustrModule: _UNICODE_STRING = core::mem::zeroed();
    let mut ustrRsrc: _UNICODE_STRING = core::mem::zeroed();
    let mut pvBuffer: *mut core::ffi::c_void;
    let aspcur: *mut CURSORDATA;
    let aicur: *mut DWORD;
    let ajifRate: PINT;
    let cjSize: UINT;
    let mut status: NTSTATUS;
    let mut bResult: BOOL = 0;

    /* Initialize buffer, so we can handle cleanup */
    ustrRsrc.Buffer = core::ptr::null_mut();
    ustrModule.Buffer = core::ptr::null_mut();
    pvBuffer = core::ptr::null_mut();

    'exit: loop {
        // The C runs this whole capture inside one _SEH2_TRY whose handler is
        // `SetLastNtError(code); goto Exit;`. Each read below is probe +
        // fault-tolerant copy, failing the same way.
        /* Probe and capture the cursor data structure */
        cursordata = match crate::safe_user_io::capture_user(pCursorData) {
            Ok(v) => v,
            Err(st) => {
                SetLastNtError(st);
                break 'exit;
            }
        };

        /* Check if this is an animated cursor */
        if (cursordata.CURSORF_flags & CURSORF_ACON) != 0 {
            /* Check of the range is ok */
            if (cursordata.cpcur == 0) || (cursordata.cicur == 0)
                || (cursordata.cpcur > 1000) || (cursordata.cicur > 1000)
            {
                break 'exit;
            }

            /* Calculate size */
            cjSize = (cursordata.cpcur as UINT * core::mem::size_of::<CURSORDATA>() as UINT)
                + (cursordata.cicur as UINT * core::mem::size_of::<DWORD>() as UINT)
                + (cursordata.cicur as UINT * core::mem::size_of::<INT>() as UINT);

            /* Allocate a buffer */
            pvBuffer = ExAllocatePoolWithTag(PagedPool, cjSize as _, USERTAG_CURSOR);
            if pvBuffer == core::ptr::null_mut() {
                break 'exit;
            }

            /* Calculate the kernel mode pointers */
            aspcur = pvBuffer as *mut CURSORDATA;
            aicur = aspcur.offset(cursordata.cpcur as isize) as *mut DWORD;
            ajifRate = aicur.offset(cursordata.cicur as isize) as PINT;

            /* Probe and copy aspcur */
            if let Err(st) = crate::safe_user_io::copy_from_user(
                aspcur as *mut core::ffi::c_void,
                cursordata.aspcur as *const core::ffi::c_void,
                cursordata.cpcur as usize * core::mem::size_of::<CURSORDATA>(),
                1,
            ) {
                SetLastNtError(st);
                break 'exit;
            }

            /* Probe and copy aicur */
            if let Err(st) = crate::safe_user_io::copy_from_user(
                aicur as *mut core::ffi::c_void,
                cursordata.aicur as *const core::ffi::c_void,
                cursordata.cicur as usize * core::mem::size_of::<DWORD>(),
                1,
            ) {
                SetLastNtError(st);
                break 'exit;
            }

            /* Probe and copy ajifRate */
            if let Err(st) = crate::safe_user_io::copy_from_user(
                ajifRate as *mut core::ffi::c_void,
                cursordata.ajifRate as *const core::ffi::c_void,
                cursordata.cicur as usize * core::mem::size_of::<INT>(),
                1,
            ) {
                SetLastNtError(st);
                break 'exit;
            }

            /* Set the new pointers */
            cursordata.aspcur = aspcur;
            cursordata.aicur = aicur;
            cursordata.ajifRate = ajifRate;
        } else {
            /* This is a standard cursor, we don't use the pointers */
            cursordata.aspcur = core::ptr::null_mut();
            cursordata.aicur = core::ptr::null_mut();
            cursordata.ajifRate = core::ptr::null_mut();
        }

        /* Check if we got a module name */
        if pustrModule != core::ptr::null_mut() {
            status = ProbeAndCaptureUnicodeString(&mut ustrModule, 1i32 as KPROCESSOR_MODE, pustrModule);
            if !NT_SUCCESS(status) {
                break 'exit;
            }
        }

        /* Check if we got a resource name */
        if pustrRsrc != core::ptr::null_mut() {
            status = ProbeAndCaptureUnicodeStringOrAtom(&mut ustrRsrc, pustrRsrc);
            if !NT_SUCCESS(status) {
                break 'exit;
            }
        }

        /* Make sure the caller doesn't give us invalid flags */
        if (cursordata.CURSORF_flags & !CURSORF_USER_MASK) != 0 {
            break 'exit;
        }

        /* Acquire the global user lock */
        UserEnterExclusive();

        /* Call the internal function */
        bResult = UserSetCursorIconData(
            hcursor,
            if pustrModule != core::ptr::null_mut() { &mut ustrModule } else { core::ptr::null_mut() },
            if pustrRsrc != core::ptr::null_mut() { &mut ustrRsrc } else { core::ptr::null_mut() },
            &mut cursordata,
        );

        /* Release the global user lock */
        UserLeave();

        break 'exit;
    }

    /* Free the captured module name */
    if (ustrModule.Buffer != core::ptr::null_mut()) && IS_INTRESOURCE(ustrModule.Buffer as *mut core::ffi::c_void) == 0 {
        ReleaseCapturedUnicodeString(&mut ustrModule, 1i32 as KPROCESSOR_MODE);
    }

    if pvBuffer != core::ptr::null_mut() {
        ExFreePoolWithTag(pvBuffer, USERTAG_CURSOR);
    }

    /* Additional cleanup on failure */
    if bResult == 0 {
        if (ustrRsrc.Buffer != core::ptr::null_mut())
            && IS_INTRESOURCE(ustrRsrc.Buffer as *mut core::ffi::c_void) == 0
        {
            ExFreePoolWithTag(ustrRsrc.Buffer as *mut core::ffi::c_void, TAG_STRING);
        }
    }

    bResult
}

#[no_mangle]
pub unsafe extern "C" fn UserDrawIconEx(
    hDc: HDC,
    xLeft: INT,
    yTop: INT,
    mut pIcon: PCURICON_OBJECT,
    mut cxWidth: INT,
    mut cyHeight: INT,
    istepIfAniCur: UINT,
    hbrFlickerFreeDraw: HBRUSH,
    diFlags: UINT,
) -> BOOL {
    let psurfDest: PSURFACE;
    let psurfMask: PSURFACE;
    let psurfColor: PSURFACE;
    let mut pdc: PDC = core::ptr::null_mut();
    let mut Ret: BOOL = 0;
    let hbmMask: HBITMAP;
    let hbmColor: HBITMAP;
    let hbmAlpha: HBITMAP;
    let bOffScreen: BOOL;
    let mut rcDest: RECTL = core::mem::zeroed();
    let mut rcSrc: RECTL = core::mem::zeroed();
    let mut pdcClipObj: *mut CLIPOBJ = core::ptr::null_mut();
    let mut exlo: EXLATEOBJ = core::mem::zeroed();

    /* Stupid case */
    if (diFlags & DI_NORMAL) == 0 {
        return 0;
    }

    if ((*pIcon).CURSORF_flags & CURSORF_ACON) != 0 {
        let pAcon = pIcon as *mut ACON;
        if istepIfAniCur >= (*pAcon).cicur {
            return 0;
        }
        pIcon = *(*pAcon).aspcur.add(*(*pAcon).aicur.add(istepIfAniCur as usize) as usize);
    }

    hbmMask = (*pIcon).hbmMask;
    hbmColor = (*pIcon).hbmColor;
    hbmAlpha = (*pIcon).hbmAlpha;

    psurfMask = SURFACE_ShareLockSurface(hbmMask as _);
    if psurfMask.is_null() {
        return 0;
    }

    if hbmColor.is_null() {
        psurfColor = core::ptr::null_mut();
    } else {
        psurfColor = SURFACE_ShareLockSurface(hbmColor as _);
        if psurfColor.is_null() {
            SURFACE_ShareUnlockSurface(psurfMask);
            return 0;
        }
    }

    pdc = DC_LockDc(hDc);
    if pdc.is_null() {
        SURFACE_ShareUnlockSurface(psurfMask);
        if !psurfColor.is_null() {
            SURFACE_ShareUnlockSurface(psurfColor);
        }
        return 0;
    }

    if cxWidth == 0 {
        if (diFlags & DI_DEFAULTSIZE) != 0 {
            cxWidth = if is_icon(pIcon) != 0 {
                UserGetSystemMetrics(SM_CXICON as ULONG)
            } else {
                UserGetSystemMetrics(SM_CXCURSOR as ULONG)
            };
        } else {
            cxWidth = (*pIcon).cx as INT;
        }
    }

    if cyHeight == 0 {
        if (diFlags & DI_DEFAULTSIZE) != 0 {
            cyHeight = if is_icon(pIcon) != 0 {
                UserGetSystemMetrics(SM_CYICON as ULONG)
            } else {
                UserGetSystemMetrics(SM_CYCURSOR as ULONG)
            };
        } else {
            cyHeight = (*pIcon).cy as INT;
        }
    }

    RECTL_vSetRect(&mut rcDest, xLeft, yTop, xLeft + cxWidth, yTop + cyHeight);
    w32r_IntLPtoDP(pdc, &mut rcDest as *mut RECTL as LPPOINT, 2);
    RECTL_vOffsetRect(&mut rcDest, (*pdc).ptlDCOrig.x, (*pdc).ptlDCOrig.y);

    DC_vPrepareDCsForBlit(pdc, &mut rcDest, core::ptr::null_mut(), core::ptr::null_mut());

    psurfDest = (*pdc).dclevel.pSurface;

    if psurfDest.is_null() {
        DC_vFinishBlit(pdc, core::ptr::null_mut());
        DC_UnlockDc(pdc);
        SURFACE_ShareUnlockSurface(psurfMask);
        if !psurfColor.is_null() {
            SURFACE_ShareUnlockSurface(psurfColor);
        }
        return 0;
    }

    RECTL_vSetRect(&mut rcSrc, 0, 0, (*pIcon).cx as INT, (*pIcon).cy as INT);

    bOffScreen = (!hbrFlickerFreeDraw.is_null()
        && (GDI_HANDLE_GET_TYPE(hbrFlickerFreeDraw) == GDI_OBJECT_TYPE_BRUSH)) as BOOL;

    'cleanup: {
        if bOffScreen != 0 {
            let mut eboFill: EBRUSHOBJ = core::mem::zeroed();
            let pbrush: PBRUSH = BRUSH_ShareLockBrush(hbrFlickerFreeDraw as _);

            if pbrush.is_null() {
                break 'cleanup;
            }

            pdcClipObj = core::ptr::addr_of_mut!((*pdc).co) as *mut CLIPOBJ;
            EBRUSHOBJ_vInit(&mut eboFill, pbrush, psurfDest as _, 0x00FFFFFF, 0, core::ptr::null_mut());

            Ret = IntEngBitBlt(
                core::ptr::addr_of_mut!((*psurfDest).SurfObj),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                pdcClipObj,
                core::ptr::null_mut(),
                &mut rcDest,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(eboFill.BrushObject),
                core::ptr::addr_of_mut!((*crate::brushbody::pbrushbody(pbrush)).ptOrigin) as *mut POINTL,
                ROP4_PATCOPY,
            );

            EBRUSHOBJ_vCleanup(&mut eboFill);
            BRUSH_ShareUnlockBrush(pbrush);

            if Ret == 0 {
                break 'cleanup;
            }
        } else {
            pdcClipObj = core::ptr::addr_of_mut!((*pdc).co) as *mut CLIPOBJ;
        }

        'done: {
            if !hbmAlpha.is_null() && ((diFlags & DI_NORMAL) == DI_NORMAL) {
                let mut blendobj: BLENDOBJ = core::mem::zeroed();
                blendobj.BlendFunction.BlendOp = AC_SRC_OVER as u8;
                blendobj.BlendFunction.BlendFlags = 0;
                blendobj.BlendFunction.SourceConstantAlpha = 255;
                blendobj.BlendFunction.AlphaFormat = AC_SRC_ALPHA as u8;

                let psurf: PSURFACE = SURFACE_ShareLockSurface(hbmAlpha as _);
                if !psurf.is_null() {
                    EXLATEOBJ_vInitialize(
                        &mut exlo,
                        (*psurf).ppal,
                        (*psurfDest).ppal,
                        0xFFFFFFFF,
                        0xFFFFFFFF,
                        0,
                    );

                    Ret = IntEngAlphaBlend(
                        core::ptr::addr_of_mut!((*psurfDest).SurfObj),
                        core::ptr::addr_of_mut!((*psurf).SurfObj),
                        pdcClipObj,
                        core::ptr::addr_of_mut!(exlo.xlo),
                        &mut rcDest,
                        &mut rcSrc,
                        &mut blendobj,
                    );

                    EXLATEOBJ_vCleanup(&mut exlo);
                    SURFACE_ShareUnlockSurface(psurf);
                    if Ret != 0 {
                        break 'done;
                    }
                }
            }

            // NoAlpha
            if (diFlags & DI_MASK) != 0 {
                let rop4: DWORD = if (diFlags & DI_IMAGE) != 0 {
                    ROP4_SRCAND
                } else {
                    ROP4_SRCCOPY
                };

                EXLATEOBJ_vInitSrcMonoXlate(&mut exlo, (*psurfDest).ppal, 0x00FFFFFF, 0);

                Ret = IntEngStretchBlt(
                    core::ptr::addr_of_mut!((*psurfDest).SurfObj),
                    core::ptr::addr_of_mut!((*psurfMask).SurfObj),
                    core::ptr::null_mut(),
                    pdcClipObj,
                    core::ptr::addr_of_mut!(exlo.xlo),
                    core::ptr::null_mut(),
                    &mut rcDest,
                    &mut rcSrc,
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    core::ptr::null_mut(),
                    rop4,
                );

                EXLATEOBJ_vCleanup(&mut exlo);

                if Ret == 0 {
                    break 'cleanup;
                }
            }

            if (diFlags & DI_IMAGE) != 0 {
                if !psurfColor.is_null() {
                    let rop4: DWORD = if (diFlags & DI_MASK) != 0 {
                        ROP4_SRCINVERT
                    } else {
                        ROP4_SRCCOPY
                    };

                    EXLATEOBJ_vInitialize(
                        &mut exlo,
                        (*psurfColor).ppal,
                        (*psurfDest).ppal,
                        0x00FFFFFF,
                        0x00FFFFFF,
                        0,
                    );

                    Ret = IntEngStretchBlt(
                        core::ptr::addr_of_mut!((*psurfDest).SurfObj),
                        core::ptr::addr_of_mut!((*psurfColor).SurfObj),
                        core::ptr::null_mut(),
                        pdcClipObj,
                        core::ptr::addr_of_mut!(exlo.xlo),
                        core::ptr::null_mut(),
                        &mut rcDest,
                        &mut rcSrc,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        rop4,
                    );

                    EXLATEOBJ_vCleanup(&mut exlo);

                    if Ret == 0 {
                        break 'cleanup;
                    }
                } else {
                    let rop4: DWORD = if (diFlags & DI_MASK) != 0 {
                        ROP4_SRCINVERT
                    } else {
                        ROP4_SRCCOPY
                    };
                    RECTL_vOffsetRect(&mut rcSrc, 0, (*pIcon).cy as LONG);

                    EXLATEOBJ_vInitSrcMonoXlate(&mut exlo, (*psurfDest).ppal, 0x00FFFFFF, 0);

                    Ret = IntEngStretchBlt(
                        core::ptr::addr_of_mut!((*psurfDest).SurfObj),
                        core::ptr::addr_of_mut!((*psurfMask).SurfObj),
                        core::ptr::null_mut(),
                        pdcClipObj,
                        core::ptr::addr_of_mut!(exlo.xlo),
                        core::ptr::null_mut(),
                        &mut rcDest,
                        &mut rcSrc,
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        core::ptr::null_mut(),
                        rop4,
                    );

                    EXLATEOBJ_vCleanup(&mut exlo);

                    if Ret == 0 {
                        break 'cleanup;
                    }
                }
            }
        }
    }

    if !pdc.is_null() {
        DC_vFinishBlit(pdc, core::ptr::null_mut());
        DC_UnlockDc(pdc);
    }

    SURFACE_ShareUnlockSurface(psurfMask);
    if !psurfColor.is_null() {
        SURFACE_ShareUnlockSurface(psurfColor);
    }

    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDrawIconEx(
    hdc: HDC,
    xLeft: INT,
    yTop: INT,
    hIcon: HICON,
    cxWidth: INT,
    cyHeight: INT,
    istepIfAniCur: UINT,
    hbrFlickerFreeDraw: HBRUSH,
    diFlags: UINT,
    bMetaHDC: BOOL,
    pDIXData: PVOID,
) -> BOOL {
    let pIcon: PCURICON_OBJECT;
    let Ret: BOOL;

    UserEnterExclusive();

    pIcon = UserGetCurIconObject(hIcon);
    if pIcon.is_null() {
        UserLeave();
        return 0;
    }

    Ret = UserDrawIconEx(
        hdc,
        xLeft,
        yTop,
        pIcon,
        cxWidth,
        cyHeight,
        istepIfAniCur,
        hbrFlickerFreeDraw,
        diFlags,
    );

    UserDereferenceObject(pIcon as PVOID);

    UserLeave();
    Ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetCursorFrameInfo(
    hCursor: HCURSOR,
    istep: DWORD,
    rate_jiffies: *mut INT,
    num_steps: *mut DWORD,
) -> HCURSOR {
    let CurIcon: PCURICON_OBJECT;
    let mut ret: HCURSOR;
    let mut jiffies: INT = 0;
    let mut steps: DWORD = 1;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    UserEnterShared();

    CurIcon = UserGetCurIconObject(hCursor);
    if CurIcon.is_null() {
        UserLeave();
        return core::ptr::null_mut();
    }

    ret = UserHMGetHandle(CurIcon) as _;

    if (*CurIcon).CURSORF_flags & CURSORF_ACON != 0 {
        let AniCurIcon: PACON = CurIcon as PACON;
        if istep >= (*AniCurIcon).cicur {
            UserDereferenceObject(CurIcon as _);
            UserLeave();
            return core::ptr::null_mut();
        }
        jiffies = *(*AniCurIcon).ajifRate.offset(istep as isize);
        steps = (*AniCurIcon).cicur;
        ret = UserHMGetHandle(
            *(*AniCurIcon)
                .aspcur
                .offset(*(*AniCurIcon).aicur.offset(istep as isize) as isize),
        ) as _;
    }

    Status = SafeProbeForWrite(
        rate_jiffies as *mut _,
        core::mem::size_of::<INT>() as _,
        1,
    );
    if NT_SUCCESS(Status) {
        Status = SafeProbeForWrite(
            num_steps as *mut _,
            core::mem::size_of::<DWORD>() as _,
            1,
        );
    }
    // The probes VALIDATE only; these STORES are the access.
    if NT_SUCCESS(Status) {
        if !crate::copy::publish_to_user(
            rate_jiffies as *mut c_void,
            core::ptr::addr_of!(jiffies) as *const c_void,
            core::mem::size_of_val(&jiffies),
        ) || !crate::copy::publish_to_user(
            num_steps as *mut c_void,
            core::ptr::addr_of!(steps) as *const c_void,
            core::mem::size_of_val(&steps),
        ) {
            Status = STATUS_ACCESS_VIOLATION;
        }
    }

    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        ret = core::ptr::null_mut();
    }

    UserDereferenceObject(CurIcon as _);
    UserLeave();

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetSystemCursor(hcur: HCURSOR, id: DWORD) -> BOOL {
    let pcur: PCURICON_OBJECT;
    let mut pcurOrig: PCURICON_OBJECT = core::ptr::null_mut();
    let mut i: i32;
    let ppi: PPROCESSINFO;
    let mut Ret: BOOL = 0;

    UserEnterExclusive();

    'block: {
        if CheckWinstaAttributeAccess(WINSTA_WRITEATTRIBUTES) == 0 {
            break 'block;
        }

        if !hcur.is_null() {
            pcur = UserGetCurIconObject(hcur);
            if pcur.is_null() {
                EngSetLastError(ERROR_INVALID_CURSOR_HANDLE);
                break 'block;
            }

            ppi = PsGetCurrentProcessWin32Process() as PPROCESSINFO;

            i = 0;
            while i < 16 {
                if gasyscur[i as usize].type_ == id {
                    pcurOrig = gasyscur[i as usize].handle;

                    if !pcurOrig.is_null() {
                        break;
                    }

                    if ((*ppi_w32pf_flags(ppi)) & W32PF_CREATEDWINORDC) != 0 {
                        gasyscur[i as usize].handle = pcur;
                        (*pcur).CURSORF_flags |= CURSORF_GLOBAL;
                        (*pcur).head.ppi = core::ptr::null_mut();
                        IntInsertCursorIntoList(pcur);
                        Ret = 1;
                    }
                    break;
                }
                i += 1;
            }

            if !pcurOrig.is_null() {
                FIXME!(
                    b"Need to copy cursor data or do something! pcurOrig %p new pcur %p\n\0".as_ptr() as PCHAR,
                    pcurOrig,
                    pcur,
                );
            }
        }
    }

    UserLeave();
    Ret
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "C" {
    fn _MmCopyToCaller(Target: *mut c_void, Source: *mut c_void, Bytes: UINT) -> NTSTATUS;
}