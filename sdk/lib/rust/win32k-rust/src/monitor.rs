#![allow(non_snake_case, non_upper_case_globals)]
use win32k_sys::win32k::*;
use ntkrnl_sys::base::{NT_SUCCESS, STATUS_SUCCESS, STATUS_INSUFFICIENT_RESOURCES};
use ntkrnl_sys::constants::{STATUS_INVALID_PARAMETER, STATUS_BUFFER_OVERFLOW};
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};

// Debug-channel macros — no kernel symbol, no-op.
macro_rules! ERR   { ($($t:tt)*) => {{}}; }
macro_rules! TRACE { ($($t:tt)*) => {{}}; }
macro_rules! WARN  { ($($t:tt)*) => {{}}; }

// USERTAG pool tag. C multi-char literal 'rmsU' is big-endian; the LE byte
// order is the literal reversed -> "Usmr".
const USERTAG_MONITORRECTS: u32 = u32::from_le_bytes(*b"Usmr");

// ---- cross-file win32k symbols (decoration verified via llvm-nm) ----
//
// FASTCALL (`@Name@N`) — extern "fastcall" on i386, cdecl elsewhere.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
    fn SetLastNtError(Status: NTSTATUS);
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RECTL_bIntersectRect(prclDst: *mut RECTL, prcl1: *const RECTL, prcl2: *const RECTL) -> BOOL;
    fn SetLastNtError(Status: NTSTATUS);
}

// cdecl (`_Name`):
extern "C" {
    fn wcslen(s: *const u16) -> usize;
    // MmCopyFromCaller(x,y,z) / MmCopyToCaller(x,y,z) are macros (mmcopy.h) for
    // `_MmCopyFromCaller((PCHAR)x,(PCHAR)y,(UINT)z)` / `_MmCopyToCaller(...)`.
    // The exported workers are cdecl.
    #[link_name = "_MmCopyFromCaller"]
    fn _MmCopyFromCaller(Target: PCHAR, Source: PCHAR, Bytes: UINT) -> NTSTATUS;
    #[link_name = "_MmCopyToCaller"]
    fn _MmCopyToCaller(Target: PCHAR, Source: PCHAR, Bytes: UINT) -> NTSTATUS;
}

// MmCopyFromCaller / MmCopyToCaller macro wrappers (mmcopy.h).
#[inline(always)]
unsafe fn MmCopyFromCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> NTSTATUS {
    _MmCopyFromCaller(dst as PCHAR, src as PCHAR, len as UINT)
}
#[inline(always)]
unsafe fn MmCopyToCaller(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, len: usize) -> NTSTATUS {
    _MmCopyToCaller(dst as PCHAR, src as PCHAR, len as UINT)
}

// UserHMGetHandle(obj) = obj->head.h (HANDLE at offset 0 of the object HEAD).
// FORCEINLINE macro — no exported symbol.
#[inline(always)]
unsafe fn UserHMGetHandle(pobj: *mut core::ffi::c_void) -> HANDLE {
    *(pobj as *const HANDLE)
}

// RECTL_bIsEmptyRect — FORCEINLINE in win32ss/gdi/ntgdi/rect.h, no symbol.
#[inline(always)]
unsafe fn RECTL_bIsEmptyRect(prcl: *const RECTL) -> BOOL {
    (((*prcl).left >= (*prcl).right) || ((*prcl).top >= (*prcl).bottom)) as BOOL
}

// min()/abs() macros over LONG (monitor.c uses the C msvcrt-style macros).
#[inline(always)]
fn min(a: LONG, b: LONG) -> LONG { if a < b { a } else { b } }
#[inline(always)]
fn abs(a: LONG) -> LONG { if a < 0 { -a } else { a } }

// gpsi embeds an anonymous PERUSERSERVERINFO (ntuser.h) that bindgen flattened
// into `tagSERVERINFO.__bindgen_padding_0`. Re-type that tail to reach
// rcScreenReal (same pattern as mouse.rs/stockobj.rs/metric.rs).
#[inline(always)]
unsafe fn peruser(psi: PSERVERINFO) -> *mut _PERUSERSERVERINFO {
    core::ptr::addr_of_mut!((*psi).__bindgen_padding_0) as *mut _PERUSERSERVERINFO
}

// MONITOR.IsPrimary lives in the anonymous union/bitfield (monitor.h); bindgen
// flattened it to __bindgen_anon_1.__bindgen_anon_1.IsPrimary (bit 2).
#[inline(always)]
unsafe fn mon_is_primary(p: PMONITOR) -> DWORD {
    (*p).__bindgen_anon_1.__bindgen_anon_1.IsPrimary()
}
#[inline(always)]
unsafe fn mon_set_primary(p: PMONITOR, val: DWORD) {
    (*p).__bindgen_anon_1.__bindgen_anon_1.set_IsPrimary(val);
}

// RtlStringCbCopyNExW — strsafe inline (ntstrsafe.h, no exported worker symbol).
// Faithful port of RtlStringCbCopyNExW -> RtlStringCopyNExWorkerW for the path
// used here (non-null dest/src, ppszDestEnd/pcbRemaining == NULL,
// STRSAFE_FILL_BEHIND_NULL). Bounded copy + null-terminate + fill behind.
const NTSTRSAFE_MAX_CCH: usize = 2147483647;
#[inline]
unsafe fn RtlStringCbCopyNExW(
    pszDest: *mut u16,
    cbDest: usize,
    pszSrc: *const u16,
    cbToCopy: usize,
    _ppszDestEnd: *mut *mut u16,
    _pcbRemaining: *mut usize,
    dwFlags: DWORD,
) -> NTSTATUS {
    let cchDest = cbDest / core::mem::size_of::<u16>();
    let mut cchToCopy = cbToCopy / core::mem::size_of::<u16>();

    if cchDest > NTSTRSAFE_MAX_CCH {
        if !pszDest.is_null() && cbDest > 0 {
            *pszDest = 0;
        }
        return STATUS_INVALID_PARAMETER;
    }

    if cchToCopy > NTSTRSAFE_MAX_CCH {
        return STATUS_INVALID_PARAMETER;
    }

    let mut status: NTSTATUS = STATUS_SUCCESS;
    if cchDest == 0 {
        if cchToCopy != 0 && *pszSrc != 0 {
            status = if pszDest.is_null() {
                STATUS_INVALID_PARAMETER
            } else {
                STATUS_BUFFER_OVERFLOW
            };
        }
    } else {
        let mut pszDestEnd = pszDest;
        let mut pszSrcCur = pszSrc;
        let mut cchRemaining = cchDest;

        while cchRemaining != 0 && cchToCopy != 0 && *pszSrcCur != 0 {
            *pszDestEnd = *pszSrcCur;
            pszDestEnd = pszDestEnd.add(1);
            pszSrcCur = pszSrcCur.add(1);
            cchRemaining -= 1;
            cchToCopy -= 1;
        }

        if cchRemaining > 0 {
            if (dwFlags & STRSAFE_FILL_BEHIND_NULL) != 0 {
                let fill = (dwFlags & 0x0000_00FF) as u8;
                let nbytes = ((cchRemaining - 1) * core::mem::size_of::<u16>())
                    + (cbDest % core::mem::size_of::<u16>());
                core::ptr::write_bytes(pszDestEnd.add(1) as *mut u8, fill, nbytes);
            }
        } else {
            pszDestEnd = pszDestEnd.sub(1);
            status = STATUS_BUFFER_OVERFLOW;
        }

        *pszDestEnd = 0;
    }

    status
}

// ---- file-scope preamble (translated) ----
static mut gMonitorList: PMONITOR = core::ptr::null_mut();

unsafe fn IntCreateMonitorObject() -> PMONITOR {
    UserCreateObject(
        gHandleTable,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        TYPE_MONITOR,
        core::mem::size_of::<MONITOR>() as _,
    ) as PMONITOR
}

unsafe fn IntDestroyMonitorObject(pMonitor: PMONITOR) {
    if !(*pMonitor).hrgnMonitor.is_null() {
        GreSetObjectOwner((*pMonitor).hrgnMonitor as _, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject((*pMonitor).hrgnMonitor as _);
    }

    UserDereferenceObject(pMonitor as *mut core::ffi::c_void);
    UserDeleteObject(UserHMGetHandle(pMonitor as *mut core::ffi::c_void), TYPE_MONITOR);
}
#[no_mangle]
pub unsafe extern "system" fn UserGetMonitorObject(hMonitor: HMONITOR) -> PMONITOR {
    let pMonitor: PMONITOR;

    if hMonitor.is_null() {
        EngSetLastError(ERROR_INVALID_MONITOR_HANDLE);
        return core::ptr::null_mut();
    }

    pMonitor = UserGetObject(gHandleTable, hMonitor as _, TYPE_MONITOR) as PMONITOR;
    if pMonitor.is_null() {
        EngSetLastError(ERROR_INVALID_MONITOR_HANDLE);
        return core::ptr::null_mut();
    }

    pMonitor
}
#[no_mangle]
pub unsafe extern "system" fn UserGetPrimaryMonitor() -> PMONITOR {
    let mut pMonitor: PMONITOR = gMonitorList;

    while pMonitor != core::ptr::null_mut() {
        if mon_is_primary(pMonitor) != 0 {
            break;
        }
        pMonitor = (*pMonitor).pMonitorNext;
    }

    pMonitor
}
#[no_mangle]
pub unsafe extern "system" fn UserAttachMonitor(hDev: HDEV) -> NTSTATUS {
    let pMonitor: PMONITOR;

    /* Create new monitor object */
    pMonitor = IntCreateMonitorObject();
    if pMonitor == core::ptr::null_mut() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*pMonitor).hDev = hDev;
    (*pMonitor).cWndStack = 0;

    if gMonitorList == core::ptr::null_mut() {
        mon_set_primary(pMonitor, 1);
        gMonitorList = pMonitor;
    } else {
        let mut pmonLast: PMONITOR = gMonitorList;
        while (*pmonLast).pMonitorNext != core::ptr::null_mut() {
            pmonLast = (*pmonLast).pMonitorNext;
        }

        (*pmonLast).pMonitorNext = pMonitor;
    }

    UserUpdateMonitorSize(hDev);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn UserDetachMonitor(hDev: HDEV) -> NTSTATUS {
    let mut pMonitor: PMONITOR = gMonitorList;
    let mut pLink: *mut PMONITOR = core::ptr::addr_of_mut!(gMonitorList);

    while !pMonitor.is_null() {
        if (*pMonitor).hDev == hDev {
            break;
        }

        pLink = core::ptr::addr_of_mut!((*pMonitor).pMonitorNext);
        pMonitor = (*pMonitor).pMonitorNext;
    }

    if pMonitor.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if mon_is_primary(pMonitor) != 0 && !(*pMonitor).pMonitorNext.is_null() {
        mon_set_primary((*pMonitor).pMonitorNext, 1);
    }

    *pLink = (*pMonitor).pMonitorNext;

    IntDestroyMonitorObject(pMonitor);

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn UserUpdateMonitorSize(hDev: HDEV) -> NTSTATUS {
    let mut pMonitor: PMONITOR;
    let mut DeviceSize: SIZEL = core::mem::zeroed();

    /* Find monitor attached to given device */
    pMonitor = gMonitorList;
    while pMonitor != core::ptr::null_mut() {
        if (*pMonitor).hDev == hDev {
            break;
        }
        pMonitor = (*pMonitor).pMonitorNext;
    }

    if pMonitor == core::ptr::null_mut() {
        /* No monitor has been found */
        return STATUS_INVALID_PARAMETER;
    }

    /* Get the size of the hdev */
    PDEVOBJ_sizl(hDev as PPDEVOBJ, &mut DeviceSize);

    /* Update monitor size */
    (*pMonitor).rcMonitor.left = 0;
    (*pMonitor).rcMonitor.top = 0;
    (*pMonitor).rcMonitor.right = (*pMonitor).rcMonitor.left + DeviceSize.cx;
    (*pMonitor).rcMonitor.bottom = (*pMonitor).rcMonitor.top + DeviceSize.cy;
    (*pMonitor).rcWork = (*pMonitor).rcMonitor;

    /* Destroy monitor region... */
    if (*pMonitor).hrgnMonitor != core::ptr::null_mut() {
        GreSetObjectOwner((*pMonitor).hrgnMonitor as _, GDI_OBJ_HMGR_POWNED);
        GreDeleteObject((*pMonitor).hrgnMonitor as _);
    }

    /* ...and create new one */
    (*pMonitor).hrgnMonitor = NtGdiCreateRectRgn(
        (*pMonitor).rcMonitor.left,
        (*pMonitor).rcMonitor.top,
        (*pMonitor).rcMonitor.right,
        (*pMonitor).rcMonitor.bottom,
    );
    if (*pMonitor).hrgnMonitor != core::ptr::null_mut() {
        IntGdiSetRegionOwner((*pMonitor).hrgnMonitor, GDI_OBJ_HMGR_PUBLIC);
    }

    //
    // Should be Virtual accumulation of all the available monitors.
    //
    (*peruser(gpsi)).rcScreenReal = (*pMonitor).rcMonitor;

    STATUS_SUCCESS
}

unsafe fn IntGetMonitorsFromRect(
    pRect: LPCRECTL,
    phMonitorList: *mut HMONITOR,
    prcMonitorList: PRECTL,
    dwListSize: DWORD,
    dwFlags: DWORD,
) -> UINT {
    let mut pMonitor: PMONITOR;
    let mut pNearestMonitor: PMONITOR = core::ptr::null_mut();
    let mut pPrimaryMonitor: PMONITOR = core::ptr::null_mut();
    let mut cMonitors: UINT = 0;
    let mut iNearestDistance: ULONG = 0xffffffff;

    pMonitor = gMonitorList;
    while pMonitor != core::ptr::null_mut() {
        let MonitorRect: RECTL;
        let mut IntersectionRect: RECTL = core::mem::zeroed();

        MonitorRect = (*pMonitor).rcMonitor;

        TRACE!(
            "MonitorRect: left = {}, top = {}, right = {}, bottom = {}",
            MonitorRect.left,
            MonitorRect.top,
            MonitorRect.right,
            MonitorRect.bottom,
        );

        if dwFlags == MONITOR_DEFAULTTOPRIMARY && mon_is_primary(pMonitor) != 0 {
            pPrimaryMonitor = pMonitor;
        }

        if pRect == core::ptr::null_mut() {
            IntersectionRect = MonitorRect;
        } else if RECTL_bIntersectRect(
            core::ptr::addr_of_mut!(IntersectionRect),
            core::ptr::addr_of!(MonitorRect),
            pRect,
        ) == 0
        {
            if dwFlags == MONITOR_DEFAULTTONEAREST {
                let cx: ULONG;
                let cy: ULONG;
                let iDistance: ULONG;

                cx = min(
                    abs(MonitorRect.left - (*pRect).right),
                    abs((*pRect).left - MonitorRect.right),
                ) as ULONG;
                cy = min(
                    abs(MonitorRect.top - (*pRect).bottom),
                    abs((*pRect).top - MonitorRect.bottom),
                ) as ULONG;

                iDistance = cx * cx + cy * cy;

                if iDistance < iNearestDistance {
                    iNearestDistance = iDistance;
                    pNearestMonitor = pMonitor;
                }
            }

            pMonitor = (*pMonitor).pMonitorNext;
            continue;
        }

        if cMonitors < dwListSize {
            if phMonitorList != core::ptr::null_mut() {
                *phMonitorList.add(cMonitors as usize) = UserHMGetHandle(pMonitor as _) as _;
            }
            if prcMonitorList != core::ptr::null_mut() {
                *prcMonitorList.add(cMonitors as usize) = IntersectionRect;
            }
        }

        cMonitors += 1;

        pMonitor = (*pMonitor).pMonitorNext;
    }

    if cMonitors == 0 {
        if dwFlags == MONITOR_DEFAULTTONEAREST && pNearestMonitor != core::ptr::null_mut() {
            if phMonitorList != core::ptr::null_mut() && dwListSize > 0 {
                *phMonitorList.add(cMonitors as usize) = UserHMGetHandle(pNearestMonitor as _) as _;
            }
            cMonitors += 1;
        } else if dwFlags == MONITOR_DEFAULTTOPRIMARY && pPrimaryMonitor != core::ptr::null_mut() {
            if phMonitorList != core::ptr::null_mut() && dwListSize > 0 {
                *phMonitorList.add(cMonitors as usize) = UserHMGetHandle(pPrimaryMonitor as _) as _;
            }
            cMonitors += 1;
        }
    }

    cMonitors
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetMonitorFromRect(pRect: LPCRECTL, dwFlags: DWORD) -> HMONITOR {
    IntGetMonitorFromRectImpl(pRect, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetMonitorFromRect(pRect: LPCRECTL, dwFlags: DWORD) -> HMONITOR {
    IntGetMonitorFromRectImpl(pRect, dwFlags)
}

unsafe fn IntGetMonitorFromRectImpl(pRect: LPCRECTL, dwFlags: DWORD) -> HMONITOR {
    let mut cMonitors: ULONG;
    let mut LargestArea: ULONG = 0;
    let mut i: ULONG;
    let mut prcMonitorList: PRECTL = core::ptr::null_mut();
    let mut phMonitorList: *mut HMONITOR = core::ptr::null_mut();
    let mut hMonitor: HMONITOR = core::ptr::null_mut();

    'cleanup: {
        cMonitors = IntGetMonitorsFromRect(pRect, &mut hMonitor, core::ptr::null_mut(), 1, dwFlags);
        if cMonitors <= 1 {
            break 'cleanup;
        }

        hMonitor = core::ptr::null_mut();

        phMonitorList = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<HMONITOR>() as ULONG * cMonitors) as _,
            USERTAG_MONITORRECTS,
        ) as *mut HMONITOR;
        if phMonitorList == core::ptr::null_mut() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }

        prcMonitorList = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<RECT>() as ULONG * cMonitors) as _,
            USERTAG_MONITORRECTS,
        ) as PRECTL;
        if prcMonitorList == core::ptr::null_mut() {
            EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
            break 'cleanup;
        }

        cMonitors = IntGetMonitorsFromRect(pRect, phMonitorList, prcMonitorList, cMonitors, 0);

        i = 0;
        while i < cMonitors {
            let pRectI = prcMonitorList.add(i as usize);
            let Area: ULONG = (((*pRectI).right - (*pRectI).left)
                * ((*pRectI).bottom - (*pRectI).top)) as ULONG;
            if Area >= LargestArea {
                hMonitor = *phMonitorList.add(i as usize);
                LargestArea = Area;
            }
            i += 1;
        }
    }

    if phMonitorList != core::ptr::null_mut() {
        ExFreePoolWithTag(phMonitorList as *mut core::ffi::c_void, USERTAG_MONITORRECTS);
    }
    if prcMonitorList != core::ptr::null_mut() {
        ExFreePoolWithTag(prcMonitorList as *mut core::ffi::c_void, USERTAG_MONITORRECTS);
    }
    hMonitor
}
#[no_mangle]
pub unsafe extern "system" fn UserMonitorFromRect(pRect: PRECTL, dwFlags: DWORD) -> PMONITOR {
    let hMonitor: HMONITOR;

    if dwFlags != MONITOR_DEFAULTTONULL
        && dwFlags != MONITOR_DEFAULTTOPRIMARY
        && dwFlags != MONITOR_DEFAULTTONEAREST
    {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return core::ptr::null_mut();
    }

    hMonitor = IntGetMonitorFromRect(pRect, dwFlags);
    UserGetMonitorObject(hMonitor)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserMonitorFromPoint(pt: POINT, dwFlags: DWORD) -> PMONITOR {
    UserMonitorFromPoint_impl(pt, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserMonitorFromPoint(pt: POINT, dwFlags: DWORD) -> PMONITOR {
    UserMonitorFromPoint_impl(pt, dwFlags)
}

unsafe fn UserMonitorFromPoint_impl(pt: POINT, dwFlags: DWORD) -> PMONITOR {
    let mut rc: RECTL = core::mem::zeroed();
    let mut hMonitor: HMONITOR = core::ptr::null_mut();

    if dwFlags != MONITOR_DEFAULTTONULL
        && dwFlags != MONITOR_DEFAULTTOPRIMARY
        && dwFlags != MONITOR_DEFAULTTONEAREST
    {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return core::ptr::null_mut();
    }

    rc.left = pt.x;
    rc.right = pt.x + 1;
    rc.top = pt.y;
    rc.bottom = pt.y + 1;

    IntGetMonitorsFromRect(
        &rc,
        &mut hMonitor,
        core::ptr::null_mut(),
        1,
        dwFlags,
    );

    UserGetMonitorObject(hMonitor)
}
#[no_mangle]
pub unsafe extern "system" fn NtUserEnumDisplayMonitors(
    hdc: HDC,
    pUnsafeRect: LPCRECTL,
    phUnsafeMonitorList: *mut HMONITOR,
    prcUnsafeMonitorList: PRECTL,
    dwListSize: DWORD,
) -> INT {
    let mut cMonitors: UINT;
    let mut i: UINT;
    let mut iRet: INT = -1;
    let mut phMonitorList: *mut HMONITOR = core::ptr::null_mut();
    let mut prcMonitorList: PRECTL = core::ptr::null_mut();
    let mut rc: RECTL = core::mem::zeroed();
    let pRect: PRECTL;
    let mut DcRect: RECTL = core::mem::zeroed();
    let mut Status: NTSTATUS;

    if pUnsafeRect != core::ptr::null_mut() {
        Status = MmCopyFromCaller(
            core::ptr::addr_of_mut!(rc) as *mut core::ffi::c_void,
            pUnsafeRect as *const core::ffi::c_void,
            core::mem::size_of::<RECT>(),
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            return -1;
        }
    }

    if hdc != core::ptr::null_mut() {
        let pDc: PDC;
        let iRgnType: INT;

        pDc = DC_LockDc(hdc);
        if pDc == core::ptr::null_mut() {
            return -1;
        }
        iRgnType = REGION_GetRgnBox((*pDc).prgnVis, core::ptr::addr_of_mut!(DcRect));
        DC_UnlockDc(pDc);

        if iRgnType == 0 {
            return -1;
        }
        if iRgnType == NULLREGION as INT {
            return 0;
        }
        if iRgnType == COMPLEXREGION as INT {
            /* TODO: Warning */
        }

        if pUnsafeRect != core::ptr::null_mut() {
            rc.left += DcRect.left;
            rc.right += DcRect.left;
            rc.top += DcRect.top;
            rc.bottom += DcRect.top;
        } else {
            rc = DcRect;
        }
    }

    if hdc == core::ptr::null_mut() && pUnsafeRect == core::ptr::null_mut() {
        pRect = core::ptr::null_mut();
    } else {
        pRect = core::ptr::addr_of_mut!(rc);
    }

    UserEnterShared();

    'cleanup: {
        cMonitors = IntGetMonitorsFromRect(
            pRect,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
            MONITOR_DEFAULTTONULL,
        );
        if cMonitors == 0
            || dwListSize == 0
            || (phUnsafeMonitorList == core::ptr::null_mut()
                && prcUnsafeMonitorList == core::ptr::null_mut())
        {
            iRet = cMonitors as INT;
            break 'cleanup;
        }

        if phUnsafeMonitorList != core::ptr::null_mut() && dwListSize != 0 {
            phMonitorList = ExAllocatePoolWithTag(
                PagedPool,
                (core::mem::size_of::<HMONITOR>() * dwListSize as usize) as _,
                USERTAG_MONITORRECTS,
            ) as *mut HMONITOR;
            if phMonitorList == core::ptr::null_mut() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
        }
        if prcUnsafeMonitorList != core::ptr::null_mut() && dwListSize != 0 {
            prcMonitorList = ExAllocatePoolWithTag(
                PagedPool,
                (core::mem::size_of::<RECT>() * dwListSize as usize) as _,
                USERTAG_MONITORRECTS,
            ) as PRECTL;
            if prcMonitorList == core::ptr::null_mut() {
                EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
                break 'cleanup;
            }
        }

        cMonitors = IntGetMonitorsFromRect(
            pRect,
            phMonitorList,
            prcMonitorList,
            dwListSize,
            MONITOR_DEFAULTTONULL,
        );

        if hdc != core::ptr::null_mut()
            && pRect != core::ptr::null_mut()
            && prcMonitorList != core::ptr::null_mut()
        {
            let limit = if cMonitors < dwListSize { cMonitors } else { dwListSize };
            i = 0;
            while i < limit {
                (*prcMonitorList.add(i as usize)).left -= DcRect.left;
                (*prcMonitorList.add(i as usize)).right -= DcRect.left;
                (*prcMonitorList.add(i as usize)).top -= DcRect.top;
                (*prcMonitorList.add(i as usize)).bottom -= DcRect.top;
                i += 1;
            }
        }

        if phUnsafeMonitorList != core::ptr::null_mut() && dwListSize != 0 {
            Status = MmCopyToCaller(
                phUnsafeMonitorList as *mut core::ffi::c_void,
                phMonitorList as *const core::ffi::c_void,
                (core::mem::size_of::<HMONITOR>() * dwListSize as usize) as _,
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                break 'cleanup;
            }
        }
        if prcUnsafeMonitorList != core::ptr::null_mut() && dwListSize != 0 {
            Status = MmCopyToCaller(
                prcUnsafeMonitorList as *mut core::ffi::c_void,
                prcMonitorList as *const core::ffi::c_void,
                (core::mem::size_of::<RECT>() * dwListSize as usize) as _,
            );
            if !NT_SUCCESS(Status) {
                SetLastNtError(Status);
                break 'cleanup;
            }
        }

        iRet = cMonitors as INT;
    }

    if phMonitorList != core::ptr::null_mut() {
        ExFreePoolWithTag(phMonitorList as *mut core::ffi::c_void, USERTAG_MONITORRECTS);
    }
    if prcMonitorList != core::ptr::null_mut() {
        ExFreePoolWithTag(prcMonitorList as *mut core::ffi::c_void, USERTAG_MONITORRECTS);
    }

    UserLeave();
    iRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetMonitorInfo(
    hMonitor: HMONITOR,
    pMonitorInfoUnsafe: LPMONITORINFO,
) -> BOOL {
    let pMonitor: PMONITOR;
    let mut MonitorInfo: MONITORINFOEXW = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut bRet: BOOL = 0;
    let pwstrDeviceName: PWCHAR;

    UserEnterShared();

    'cleanup: loop {
        /* Get monitor object */
        pMonitor = UserGetMonitorObject(hMonitor);
        if pMonitor.is_null() {
            break 'cleanup;
        }

        /* Check if pMonitorInfoUnsafe is valid */
        if pMonitorInfoUnsafe.is_null() {
            SetLastNtError(STATUS_INVALID_PARAMETER);
            break 'cleanup;
        }

        pwstrDeviceName = core::ptr::addr_of_mut!(
            (*(*((*pMonitor).hDev as PPDEVOBJ)).pGraphicsDevice).szWinDeviceName
        ) as PWCHAR;

        /* Get size of pMonitorInfoUnsafe */
        Status = MmCopyFromCaller(
            core::ptr::addr_of_mut!(MonitorInfo.cbSize) as _,
            core::ptr::addr_of_mut!((*pMonitorInfoUnsafe).cbSize) as _,
            core::mem::size_of_val(&MonitorInfo.cbSize),
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'cleanup;
        }

        /* Check if size of struct is valid */
        if MonitorInfo.cbSize != core::mem::size_of::<MONITORINFO>() as u32
            && MonitorInfo.cbSize != core::mem::size_of::<MONITORINFOEXW>() as u32
        {
            SetLastNtError(STATUS_INVALID_PARAMETER);
            break 'cleanup;
        }

        /* Fill monitor info */
        MonitorInfo.rcMonitor = (*pMonitor).rcMonitor;
        MonitorInfo.rcWork = (*pMonitor).rcWork;
        MonitorInfo.dwFlags = 0;
        if mon_is_primary(pMonitor) != 0 {
            MonitorInfo.dwFlags |= MONITORINFOF_PRIMARY;
        }

        /* Fill device name */
        if MonitorInfo.cbSize == core::mem::size_of::<MONITORINFOEXW>() as u32 {
            RtlStringCbCopyNExW(
                MonitorInfo.szDevice.as_mut_ptr(),
                core::mem::size_of_val(&MonitorInfo.szDevice),
                pwstrDeviceName,
                (wcslen(pwstrDeviceName) + 1) * core::mem::size_of::<u16>(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                STRSAFE_FILL_BEHIND_NULL,
            );
        }

        /* Output data */
        Status = MmCopyToCaller(
            pMonitorInfoUnsafe as _,
            core::ptr::addr_of_mut!(MonitorInfo) as _,
            MonitorInfo.cbSize as usize,
        );
        if !NT_SUCCESS(Status) {
            SetLastNtError(Status);
            break 'cleanup;
        }

        bRet = 1;
        break 'cleanup;
    }

    UserLeave();
    bRet
}
#[no_mangle]
pub unsafe extern "system" fn NtUserMonitorFromPoint(
    pt: POINT,
    dwFlags: DWORD,
) -> HMONITOR {
    let mut rc: RECTL = core::mem::zeroed();
    let mut hMonitor: HMONITOR = core::ptr::null_mut();

    if dwFlags != MONITOR_DEFAULTTONULL
        && dwFlags != MONITOR_DEFAULTTOPRIMARY
        && dwFlags != MONITOR_DEFAULTTONEAREST
    {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return core::ptr::null_mut();
    }

    rc.left = pt.x;
    rc.right = pt.x + 1;
    rc.top = pt.y;
    rc.bottom = pt.y + 1;

    UserEnterShared();

    IntGetMonitorsFromRect(
        &rc,
        &mut hMonitor,
        core::ptr::null_mut(),
        1,
        dwFlags,
    );

    UserLeave();
    hMonitor
}
#[no_mangle]
pub unsafe extern "system" fn NtUserMonitorFromRect(
    pRectUnsafe: LPCRECTL,
    dwFlags: DWORD,
) -> HMONITOR {
    let mut Rect: RECTL = core::mem::zeroed();
    let Status: NTSTATUS;
    let hMonitor: HMONITOR;

    if dwFlags != MONITOR_DEFAULTTONULL
        && dwFlags != MONITOR_DEFAULTTOPRIMARY
        && dwFlags != MONITOR_DEFAULTTONEAREST
    {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return core::ptr::null_mut();
    }

    Status = MmCopyFromCaller(
        core::ptr::addr_of_mut!(Rect) as *mut core::ffi::c_void,
        pRectUnsafe as *const core::ffi::c_void,
        core::mem::size_of::<RECT>(),
    );
    if !NT_SUCCESS(Status) {
        SetLastNtError(Status);
        return core::ptr::null_mut();
    }

    if RECTL_bIsEmptyRect(core::ptr::addr_of_mut!(Rect)) != 0 {
        Rect.right = Rect.left + 1;
        Rect.bottom = Rect.top + 1;
    }

    UserEnterShared();
    hMonitor = IntGetMonitorFromRect(core::ptr::addr_of_mut!(Rect) as LPCRECTL, dwFlags);
    UserLeave();

    hMonitor
}
#[no_mangle]
pub unsafe extern "system" fn NtUserMonitorFromWindow(hWnd: HWND, dwFlags: DWORD) -> HMONITOR {
    let pWnd: PWND;
    let mut hMonitor: HMONITOR = core::ptr::null_mut();
    let mut Rect: RECTL = RECTL { left: 0, top: 0, right: 0, bottom: 0 };

    if dwFlags != MONITOR_DEFAULTTONULL
        && dwFlags != MONITOR_DEFAULTTOPRIMARY
        && dwFlags != MONITOR_DEFAULTTONEAREST
    {
        EngSetLastError(ERROR_INVALID_FLAGS);
        return core::ptr::null_mut();
    }

    UserEnterShared();

    if !hWnd.is_null() {
        pWnd = UserGetWindowObject(hWnd);
        if !pWnd.is_null() {
            Rect = if ((*pWnd).style & WS_MINIMIZE as ULONG) != 0 {
                (*pWnd).InternalPos.NormalRect
            } else {
                (*pWnd).rcWindow
            };
            if RECTL_bIsEmptyRect(core::ptr::addr_of_mut!(Rect)) != 0 {
                Rect.right = Rect.left + 1;
                Rect.bottom = Rect.top + 1;
            }
        }
    }

    hMonitor = IntGetMonitorFromRect(core::ptr::addr_of_mut!(Rect), dwFlags);

    UserLeave();

    hMonitor
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  *  COPYRIGHT:        See COPYING in the top level directory
//  *  PROJECT:          ReactOS kernel
//  *  PURPOSE:          pMonitor support
//  *  FILE:             subsys/win32k/ntuser/monitor.c
//  *  PROGRAMERS:       Anich Gregor (blight@blight.eu.org)
//  *                    Rafal Harabien (rafalh@reactos.org)
//  */
//
// #include <win32k.h>
// DBG_DEFAULT_CHANNEL(UserMonitor);
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// /* List of monitors */
// static PMONITOR gMonitorList = NULL;
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /* IntCreateMonitorObject
//  *
//  * Creates a MONITOR
//  *
//  * Return value
//  *   If the function succeeds a pointer to a MONITOR is returned. On failure
//  *   NULL is returned.
//  */
// static
// PMONITOR
// IntCreateMonitorObject(VOID)
// {
//     return UserCreateObject(gHandleTable, NULL, NULL, NULL, TYPE_MONITOR, sizeof(MONITOR));
// }
//
// /* IntDestroyMonitorObject
//  *
//  * Destroys a MONITOR
//  * You have to be the owner of the monitors lock to safely destroy it.
//  *
//  * Arguments
//  *
//  *   pMonitor
//  *      Pointer to the MONITOR which shall be deleted
//  */
// static
// void
// IntDestroyMonitorObject(IN PMONITOR pMonitor)
// {
//     /* Remove monitor region */
//     if (pMonitor->hrgnMonitor)
//     {
//         GreSetObjectOwner(pMonitor->hrgnMonitor, GDI_OBJ_HMGR_POWNED);
//         GreDeleteObject(pMonitor->hrgnMonitor);
//     }
//
//     /* Destroy monitor object */
//     UserDereferenceObject(pMonitor);
//     UserDeleteObject(UserHMGetHandle(pMonitor), TYPE_MONITOR);
// }
//
// /* UserGetMonitorObject
//  *
//  * Returns monitor object from handle or sets last error if handle is invalid
//  *
//  * Arguments
//  *
//  *   hMonitor
//  *      Handle of MONITOR object
//  */
// PMONITOR NTAPI
// UserGetMonitorObject(IN HMONITOR hMonitor)
// {
//     PMONITOR pMonitor;
//
//     if (!hMonitor)
//     {
//         EngSetLastError(ERROR_INVALID_MONITOR_HANDLE);
//         return NULL;
//     }
//
//     pMonitor = (PMONITOR)UserGetObject(gHandleTable, hMonitor, TYPE_MONITOR);
//     if (!pMonitor)
//     {
//         EngSetLastError(ERROR_INVALID_MONITOR_HANDLE);
//         return NULL;
//     }
//
//     return pMonitor;
// }
//
// /* UserGetPrimaryMonitor
//  *
//  * Returns a PMONITOR for the primary monitor
//  *
//  * Return value
//  *   PMONITOR
//  */
// PMONITOR NTAPI
// UserGetPrimaryMonitor(VOID)
// {
//     PMONITOR pMonitor;
//
//     /* Find primary monitor */
//     for (pMonitor = gMonitorList; pMonitor != NULL; pMonitor = pMonitor->pMonitorNext)
//     {
//         if (pMonitor->IsPrimary)
//             break;
//     }
//
//     return pMonitor;
// }
//
// /* UserAttachMonitor
//  *
//  * Creates a new MONITOR and appends it to the list of monitors.
//  *
//  * Arguments
//  *
//  *   pGdiDevice     Pointer to the PDEVOBJ onto which the monitor was attached
//  *   DisplayNumber  Display Number (starting with 0)
//  *
//  * Return value
//  *   Returns a NTSTATUS
//  */
// NTSTATUS NTAPI
// UserAttachMonitor(IN HDEV hDev)
// {
//     PMONITOR pMonitor;
//
//     TRACE("Attaching monitor...\n");
//
//     /* Create new monitor object */
//     pMonitor = IntCreateMonitorObject();
//     if (pMonitor == NULL)
//     {
//         TRACE("Couldnt create monitor object\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     pMonitor->hDev = hDev;
//     pMonitor->cWndStack = 0;
//
//     if (gMonitorList == NULL)
//     {
//         TRACE("Primary monitor is beeing attached\n");
//         pMonitor->IsPrimary = TRUE;
//         gMonitorList = pMonitor;
//     }
//     else
//     {
//         PMONITOR pmonLast = gMonitorList;
//         TRACE("Additional monitor is beeing attached\n");
//         while (pmonLast->pMonitorNext != NULL)
//             pmonLast = pmonLast->pMonitorNext;
//
//         pmonLast->pMonitorNext = pMonitor;
//     }
//
//     UserUpdateMonitorSize(hDev);
//
//     return STATUS_SUCCESS;
// }
//
// /* UserDetachMonitor
//  *
//  * Deletes a MONITOR and removes it from the list of monitors.
//  *
//  * Arguments
//  *
//  *   pGdiDevice  Pointer to the PDEVOBJ from which the monitor was detached
//  *
//  * Return value
//  *   Returns a NTSTATUS
//  */
// NTSTATUS NTAPI
// UserDetachMonitor(IN HDEV hDev)
// {
//     PMONITOR pMonitor = gMonitorList, *pLink = &gMonitorList;
//
//     /* Find monitor attached to given device */
//     while (pMonitor != NULL)
//     {
//         if (pMonitor->hDev == hDev)
//             break;
//
//         pLink = &pMonitor->pMonitorNext;
//         pMonitor = pMonitor->pMonitorNext;
//     }
//
//     if (pMonitor == NULL)
//     {
//         /* No monitor has been found */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* We destroy primary monitor - set next as primary */
//     if (pMonitor->IsPrimary && pMonitor->pMonitorNext != NULL)
//         pMonitor->pMonitorNext->IsPrimary = TRUE;
//
//     /* Update Next ptr in previous monitor */
//     *pLink = pMonitor->pMonitorNext;
//
//     /* Finally destroy monitor */
//     IntDestroyMonitorObject(pMonitor);
//
//     return STATUS_SUCCESS;
// }
//
// /* UserUpdateMonitorSize
//  *
//  * Reset size of the monitor using atached device
//  *
//  * Arguments
//  *
//  *   PMONITOR
//  *      pGdiDevice  Pointer to the PDEVOBJ, which size has changed
//  *
//  * Return value
//  *   Returns a NTSTATUS
//  */
// NTSTATUS NTAPI
// UserUpdateMonitorSize(IN HDEV hDev)
// {
// 	PMONITOR pMonitor;
//     SIZEL DeviceSize;
//
//     /* Find monitor attached to given device */
//     for (pMonitor = gMonitorList; pMonitor != NULL; pMonitor = pMonitor->pMonitorNext)
//     {
//         if (pMonitor->hDev == hDev)
//             break;
//     }
//
//     if (pMonitor == NULL)
//     {
//         /* No monitor has been found */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Get the size of the hdev */
//     PDEVOBJ_sizl((PPDEVOBJ)hDev, &DeviceSize);
//
//     /* Update monitor size */
//     pMonitor->rcMonitor.left  = 0;
//     pMonitor->rcMonitor.top   = 0;
//     pMonitor->rcMonitor.right  = pMonitor->rcMonitor.left + DeviceSize.cx;
//     pMonitor->rcMonitor.bottom = pMonitor->rcMonitor.top + DeviceSize.cy;
//     pMonitor->rcWork = pMonitor->rcMonitor;
//
//     /* Destroy monitor region... */
//     if (pMonitor->hrgnMonitor)
//     {
//         GreSetObjectOwner(pMonitor->hrgnMonitor, GDI_OBJ_HMGR_POWNED);
//         GreDeleteObject(pMonitor->hrgnMonitor);
//     }
//
//     /* ...and create new one */
//     pMonitor->hrgnMonitor = NtGdiCreateRectRgn(
//         pMonitor->rcMonitor.left,
//         pMonitor->rcMonitor.top,
//         pMonitor->rcMonitor.right,
//         pMonitor->rcMonitor.bottom);
//     if (pMonitor->hrgnMonitor)
//         IntGdiSetRegionOwner(pMonitor->hrgnMonitor, GDI_OBJ_HMGR_PUBLIC);
//
//     //
//     // Should be Virtual accumulation of all the available monitors.
//     //
//     gpsi->rcScreenReal = pMonitor->rcMonitor;
//
//     return STATUS_SUCCESS;
// }
//
// /* IntGetMonitorsFromRect
//  *
//  * Returns a list of monitor handles/rectangles. The rectangles in the list are
//  * the areas of intersection with the monitors.
//  *
//  * Arguments
//  *
//  *   pRect
//  *      Rectangle in desktop coordinates. If this is NULL all monitors are
//  *      returned and the rect list is filled with the sizes of the monitors.
//  *
//  *   phMonitorList
//  *      Pointer to an array of HMONITOR which is filled with monitor handles.
//  *      Can be NULL
//  *
//  *   prcMonitorList
//  *      Pointer to an array of RECT which is filled with intersection rects in
//  *      desktop coordinates.
//  *      Can be NULL, will be ignored if no intersecting monitor is found and
//  *      flags is MONITOR_DEFAULTTONEAREST
//  *
//  *   dwListSize
//  *      Size of the phMonitorList and prcMonitorList arguments. If this is zero
//  *      phMonitorList and prcMonitorList are ignored.
//  *
//  *   dwFlags
//  *      Either 0 or MONITOR_DEFAULTTONEAREST (ignored if rect is NULL)
//  *
//  * Returns
//  *   The number of monitors which intersect the specified region.
//  */
// static
// UINT
// IntGetMonitorsFromRect(OPTIONAL IN LPCRECTL pRect,
//                        OPTIONAL OUT HMONITOR *phMonitorList,
//                        OPTIONAL OUT PRECTL prcMonitorList,
//                        OPTIONAL IN DWORD dwListSize,
//                        OPTIONAL IN DWORD dwFlags)
// {
//     PMONITOR pMonitor, pNearestMonitor = NULL, pPrimaryMonitor = NULL;
//     UINT cMonitors = 0;
//     ULONG iNearestDistance = 0xffffffff;
//
//     /* Find monitors which intersects the rectangle */
//     for (pMonitor = gMonitorList; pMonitor != NULL; pMonitor = pMonitor->pMonitorNext)
//     {
//         RECTL MonitorRect, IntersectionRect;
//
//         MonitorRect = pMonitor->rcMonitor;
//
//         TRACE("MonitorRect: left = %d, top = %d, right = %d, bottom = %d\n",
//                MonitorRect.left, MonitorRect.top, MonitorRect.right, MonitorRect.bottom);
//
//         /* Save primary monitor for later usage */
//         if (dwFlags == MONITOR_DEFAULTTOPRIMARY && pMonitor->IsPrimary)
//             pPrimaryMonitor = pMonitor;
//
//         /* Check if a rect is given */
//         if (pRect == NULL)
//         {
//             /* No rect given, so use the full monitor rect */
//             IntersectionRect = MonitorRect;
//         }
//         /* We have a rect, calculate intersection */
//         else if (!RECTL_bIntersectRect(&IntersectionRect, &MonitorRect, pRect))
//         {
//             /* Rects did not intersect */
//             if (dwFlags == MONITOR_DEFAULTTONEAREST)
//             {
//                 ULONG cx, cy, iDistance;
//
//                 /* Get x and y distance */
//                 cx = min(abs(MonitorRect.left - pRect->right),
//                          abs(pRect->left - MonitorRect.right));
//                 cy = min(abs(MonitorRect.top - pRect->bottom),
//                          abs(pRect->top - MonitorRect.bottom));
//
//                 /* Calculate distance square */
//                 iDistance = cx * cx + cy * cy;
//
//                 /* Check if this is the new nearest monitor */
//                 if (iDistance < iNearestDistance)
//                 {
//                     iNearestDistance = iDistance;
//                     pNearestMonitor = pMonitor;
//                 }
//             }
//
//             continue;
//         }
//
//         /* Check if there's space in the buffer */
//         if (cMonitors < dwListSize)
//         {
//             /* Save monitor data */
//             if (phMonitorList != NULL)
//                 phMonitorList[cMonitors] = UserHMGetHandle(pMonitor);
//             if (prcMonitorList != NULL)
//                 prcMonitorList[cMonitors] = IntersectionRect;
//         }
//
//         /* Increase count of found monitors */
//         cMonitors++;
//     }
//
//     /* Nothing has been found? */
//     if (cMonitors == 0)
//     {
//         /* Check if we shall default to the nearest monitor */
//         if (dwFlags == MONITOR_DEFAULTTONEAREST && pNearestMonitor)
//         {
//             if (phMonitorList && dwListSize > 0)
//                 phMonitorList[cMonitors] = UserHMGetHandle(pNearestMonitor);
//             cMonitors++;
//         }
//         /* Check if we shall default to the primary monitor */
//         else if (dwFlags == MONITOR_DEFAULTTOPRIMARY && pPrimaryMonitor)
//         {
//             if (phMonitorList != NULL && dwListSize > 0)
//                 phMonitorList[cMonitors] = UserHMGetHandle(pPrimaryMonitor);
//             cMonitors++;
//         }
//     }
//
//     return cMonitors;
// }
//
// HMONITOR FASTCALL
// IntGetMonitorFromRect(
//      LPCRECTL pRect,
//      DWORD dwFlags)
// {
//     ULONG cMonitors, LargestArea = 0, i;
//     PRECTL prcMonitorList = NULL;
//     HMONITOR *phMonitorList = NULL;
//     HMONITOR hMonitor = NULL;
//
//     /* Find intersecting monitors */
//     cMonitors = IntGetMonitorsFromRect(pRect, &hMonitor, NULL, 1, dwFlags);
//     if (cMonitors <= 1)
//     {
//         /* No or one monitor found. Just return handle. */
//         goto cleanup;
//     }
//
//     /* There is more than one monitor. Find monitor with largest intersection.
//        Temporary reset hMonitor */
//     hMonitor = NULL;
//
//     /* Allocate helper buffers */
//     phMonitorList = ExAllocatePoolWithTag(PagedPool,
//                                           sizeof(HMONITOR) * cMonitors,
//                                           USERTAG_MONITORRECTS);
//     if (phMonitorList == NULL)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto cleanup;
//     }
//
//     prcMonitorList = ExAllocatePoolWithTag(PagedPool,
//                                            sizeof(RECT) * cMonitors,
//                                            USERTAG_MONITORRECTS);
//     if (prcMonitorList == NULL)
//     {
//         EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//         goto cleanup;
//     }
//
//     /* Get intersecting monitors again but now with rectangle list */
//     cMonitors = IntGetMonitorsFromRect(pRect, phMonitorList, prcMonitorList, cMonitors, 0);
//
//     /* Find largest intersection */
//     for (i = 0; i < cMonitors; i++)
//     {
//         ULONG Area = (prcMonitorList[i].right - prcMonitorList[i].left) *
//                      (prcMonitorList[i].bottom - prcMonitorList[i].top);
//         if (Area >= LargestArea)
//         {
//             hMonitor = phMonitorList[i];
//             LargestArea = Area;
//         }
//     }
//
// cleanup:
//     if (phMonitorList)
//         ExFreePoolWithTag(phMonitorList, USERTAG_MONITORRECTS);
//     if (prcMonitorList)
//         ExFreePoolWithTag(prcMonitorList, USERTAG_MONITORRECTS);
//     return hMonitor;
// }
//
// PMONITOR NTAPI
// UserMonitorFromRect(
//     PRECTL pRect,
//     DWORD dwFlags)
// {
//     HMONITOR hMonitor;
//
//     /* Check if flags are valid */
//     if (dwFlags != MONITOR_DEFAULTTONULL &&
//         dwFlags != MONITOR_DEFAULTTOPRIMARY &&
//         dwFlags != MONITOR_DEFAULTTONEAREST)
//     {
//         EngSetLastError(ERROR_INVALID_FLAGS);
//         return NULL;
//     }
//
//     hMonitor = IntGetMonitorFromRect(pRect, dwFlags);
//     return UserGetMonitorObject(hMonitor);
// }
//
// PMONITOR
// FASTCALL
// UserMonitorFromPoint(
//     IN POINT pt,
//     IN DWORD dwFlags)
// {
//     RECTL rc;
//     HMONITOR hMonitor = NULL;
//
//     /* Check if flags are valid */
//     if (dwFlags != MONITOR_DEFAULTTONULL &&
//         dwFlags != MONITOR_DEFAULTTOPRIMARY &&
//         dwFlags != MONITOR_DEFAULTTONEAREST)
//     {
//         EngSetLastError(ERROR_INVALID_FLAGS);
//         return NULL;
//     }
//
//     /* Fill rect (bottom-right exclusive) */
//     rc.left = pt.x;
//     rc.right = pt.x + 1;
//     rc.top = pt.y;
//     rc.bottom = pt.y + 1;
//
//     /* Find intersecting monitor */
//     IntGetMonitorsFromRect(&rc, &hMonitor, NULL, 1, dwFlags);
//
//     return UserGetMonitorObject(hMonitor);
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /* NtUserEnumDisplayMonitors
//  *
//  * Enumerates display monitors which intersect the given HDC/cliprect
//  *
//  * Arguments
//  *
//  *   hdc
//  *      Handle to a DC for which to enum intersecting monitors. If this is NULL
//  *      it returns all monitors which are part of the current virtual screen.
//  *
//  *   pUnsafeRect
//  *      Clipping rectangle with coordinate system origin at the DCs origin if the
//  *      given HDC is not NULL or in virtual screen coordinated if it is NULL.
//  *      Can be NULL
//  *
//  *   phUnsafeMonitorList
//  *      Pointer to an array of HMONITOR which is filled with monitor handles.
//  *      Can be NULL
//  *
//  *   prcUnsafeMonitorList
//  *      Pointer to an array of RECT which is filled with intersection rectangles.
//  *      Can be NULL
//  *
//  *   dwListSize
//  *      Size of the hMonitorList and monitorRectList arguments. If this is zero
//  *      hMonitorList and monitorRectList are ignored.
//  *
//  * Returns
//  *   The number of monitors which intersect the specified region or -1 on failure.
//  */
// INT
// __stdcall
// NtUserEnumDisplayMonitors(
//     OPTIONAL IN HDC hdc,
//     OPTIONAL IN LPCRECTL pUnsafeRect,
//     OPTIONAL OUT HMONITOR *phUnsafeMonitorList,
//     OPTIONAL OUT PRECTL prcUnsafeMonitorList,
//     OPTIONAL IN DWORD dwListSize)
// {
//     UINT cMonitors, i;
//     INT iRet = -1;
//     HMONITOR *phMonitorList = NULL;
//     PRECTL prcMonitorList = NULL;
//     RECTL rc, *pRect;
//     RECTL DcRect = {0};
//     NTSTATUS Status;
//
//     /* Get rectangle */
//     if (pUnsafeRect != NULL)
//     {
//         Status = MmCopyFromCaller(&rc, pUnsafeRect, sizeof(RECT));
//         if (!NT_SUCCESS(Status))
//         {
//             TRACE("MmCopyFromCaller() failed!\n");
//             SetLastNtError(Status);
//             return -1;
//         }
//     }
//
//     if (hdc != NULL)
//     {
//         PDC pDc;
//         INT iRgnType;
//
//         /* Get visible region bounding rect */
//         pDc = DC_LockDc(hdc);
//         if (pDc == NULL)
//         {
//             TRACE("DC_LockDc() failed!\n");
//             /* FIXME: setlasterror? */
//             return -1;
//         }
//         iRgnType = REGION_GetRgnBox(pDc->prgnVis, &DcRect);
//         DC_UnlockDc(pDc);
//
//         if (iRgnType == 0)
//         {
//             TRACE("NtGdiGetRgnBox() failed!\n");
//             return -1;
//         }
//         if (iRgnType == NULLREGION)
//             return 0;
//         if (iRgnType == COMPLEXREGION)
//         {
//             /* TODO: Warning */
//         }
//
//         /* If hdc and pRect are given the area of interest is pRect with
//            coordinate origin at the DC position */
//         if (pUnsafeRect != NULL)
//         {
//             rc.left += DcRect.left;
//             rc.right += DcRect.left;
//             rc.top += DcRect.top;
//             rc.bottom += DcRect.top;
//         }
//         /* If hdc is given and pRect is not the area of interest is the
//            bounding rect of hdc */
//         else
//         {
//             rc = DcRect;
//         }
//     }
//
//     if (hdc == NULL && pUnsafeRect == NULL)
//         pRect = NULL;
//     else
//         pRect = &rc;
//
//     UserEnterShared();
//
//     /* Find intersecting monitors */
//     cMonitors = IntGetMonitorsFromRect(pRect, NULL, NULL, 0, MONITOR_DEFAULTTONULL);
//     if (cMonitors == 0 || dwListSize == 0 ||
//         (phUnsafeMonitorList == NULL && prcUnsafeMonitorList == NULL))
//     {
//         /* Simple case - just return monitors count */
//         TRACE("cMonitors = %u\n", cMonitors);
//         iRet = cMonitors;
//         goto cleanup;
//     }
//
//     /* Allocate safe buffers */
//     if (phUnsafeMonitorList != NULL && dwListSize != 0)
//     {
//         phMonitorList = ExAllocatePoolWithTag(PagedPool, sizeof (HMONITOR) * dwListSize, USERTAG_MONITORRECTS);
//         if (phMonitorList == NULL)
//         {
//             EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//     }
//     if (prcUnsafeMonitorList != NULL && dwListSize != 0)
//     {
//         prcMonitorList = ExAllocatePoolWithTag(PagedPool, sizeof(RECT) * dwListSize,USERTAG_MONITORRECTS);
//         if (prcMonitorList == NULL)
//         {
//             EngSetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             goto cleanup;
//         }
//     }
//
//     /* Get intersecting monitors */
//     cMonitors = IntGetMonitorsFromRect(pRect, phMonitorList, prcMonitorList,
//                                        dwListSize, MONITOR_DEFAULTTONULL);
//
//     if (hdc != NULL && pRect != NULL && prcMonitorList != NULL)
//     {
//         for (i = 0; i < min(cMonitors, dwListSize); i++)
//         {
//             _Analysis_assume_(i < dwListSize);
//             prcMonitorList[i].left -= DcRect.left;
//             prcMonitorList[i].right -= DcRect.left;
//             prcMonitorList[i].top -= DcRect.top;
//             prcMonitorList[i].bottom -= DcRect.top;
//         }
//     }
//
//     /* Output result */
//     if (phUnsafeMonitorList != NULL && dwListSize != 0)
//     {
//         Status = MmCopyToCaller(phUnsafeMonitorList, phMonitorList, sizeof(HMONITOR) * dwListSize);
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             goto cleanup;
//         }
//     }
//     if (prcUnsafeMonitorList != NULL && dwListSize != 0)
//     {
//         Status = MmCopyToCaller(prcUnsafeMonitorList, prcMonitorList, sizeof(RECT) * dwListSize);
//         if (!NT_SUCCESS(Status))
//         {
//             SetLastNtError(Status);
//             goto cleanup;
//         }
//     }
//
//     /* Return monitors count on success */
//     iRet = cMonitors;
//
// cleanup:
//     if (phMonitorList)
//         ExFreePoolWithTag(phMonitorList, USERTAG_MONITORRECTS);
//     if (prcMonitorList)
//         ExFreePoolWithTag(prcMonitorList, USERTAG_MONITORRECTS);
//
//     UserLeave();
//     return iRet;
// }
//
// /* NtUserGetMonitorInfo
//  *
//  * Retrieves information about a given monitor
//  *
//  * Arguments
//  *
//  *   hMonitor
//  *      Handle to a monitor for which to get information
//  *
//  *   pMonitorInfoUnsafe
//  *      Pointer to a MONITORINFO struct which is filled with the information.
//  *      The cbSize member must be set to sizeof(MONITORINFO) or
//  *      sizeof(MONITORINFOEX). Even if set to sizeof(MONITORINFOEX) only parts
//  *      from MONITORINFO will be filled.
//  *
//  *   pDevice
//  *      Pointer to a UNICODE_STRING which will receive the device's name. The
//  *      length should be CCHDEVICENAME
//  *      Can be NULL
//  *
//  * Return value
//  *   TRUE on success; FALSE on failure (calls SetLastNtError())
//  *
//  */
// BOOL
// __stdcall
// NtUserGetMonitorInfo(
//     IN HMONITOR hMonitor,
//     OUT LPMONITORINFO pMonitorInfoUnsafe)
// {
//     PMONITOR pMonitor;
//     MONITORINFOEXW MonitorInfo;
//     NTSTATUS Status;
//     BOOL bRet = FALSE;
//     PWCHAR pwstrDeviceName;
//
//     TRACE("Enter NtUserGetMonitorInfo\n");
//     UserEnterShared();
//
//     /* Get monitor object */
//     pMonitor = UserGetMonitorObject(hMonitor);
//     if (!pMonitor)
//     {
//         TRACE("Couldnt find monitor %p\n", hMonitor);
//         goto cleanup;
//     }
//
//     /* Check if pMonitorInfoUnsafe is valid */
//     if(pMonitorInfoUnsafe == NULL)
//     {
//         SetLastNtError(STATUS_INVALID_PARAMETER);
//         goto cleanup;
//     }
//
//     pwstrDeviceName = ((PPDEVOBJ)(pMonitor->hDev))->pGraphicsDevice->szWinDeviceName;
//
//     /* Get size of pMonitorInfoUnsafe */
//     Status = MmCopyFromCaller(&MonitorInfo.cbSize, &pMonitorInfoUnsafe->cbSize, sizeof(MonitorInfo.cbSize));
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         goto cleanup;
//     }
//
//     /* Check if size of struct is valid */
//     if (MonitorInfo.cbSize != sizeof(MONITORINFO) &&
//         MonitorInfo.cbSize != sizeof(MONITORINFOEXW))
//     {
//         SetLastNtError(STATUS_INVALID_PARAMETER);
//         goto cleanup;
//     }
//
//     /* Fill monitor info */
//     MonitorInfo.rcMonitor = pMonitor->rcMonitor;
//     MonitorInfo.rcWork = pMonitor->rcWork;
//     MonitorInfo.dwFlags = 0;
//     if (pMonitor->IsPrimary)
//         MonitorInfo.dwFlags |= MONITORINFOF_PRIMARY;
//
//     /* Fill device name */
//     if (MonitorInfo.cbSize == sizeof(MONITORINFOEXW))
//     {
//         RtlStringCbCopyNExW(MonitorInfo.szDevice,
//                           sizeof(MonitorInfo.szDevice),
//                           pwstrDeviceName,
//                           (wcslen(pwstrDeviceName)+1) * sizeof(WCHAR),
//                           NULL, NULL, STRSAFE_FILL_BEHIND_NULL);
//     }
//
//     /* Output data */
//     Status = MmCopyToCaller(pMonitorInfoUnsafe, &MonitorInfo, MonitorInfo.cbSize);
//     if (!NT_SUCCESS(Status))
//     {
//         TRACE("GetMonitorInfo: MmCopyToCaller failed\n");
//         SetLastNtError(Status);
//         goto cleanup;
//     }
//
//     TRACE("GetMonitorInfo: success\n");
//     bRet = TRUE;
//
// cleanup:
//     TRACE("Leave NtUserGetMonitorInfo, ret=%i\n", bRet);
//     UserLeave();
//     return bRet;
// }
//
// /* NtUserMonitorFromPoint
//  *
//  * Returns a handle to the monitor containing the given point.
//  *
//  * Arguments
//  *
//  *   pt
//  *     Point for which to find monitor
//  *
//  *   dwFlags
//  *     Specifies the behaviour if the point isn't on any of the monitors.
//  *
//  * Return value
//  *   If the point is found a handle to the monitor is returned; if not the
//  *   return value depends on dwFlags
//  */
// HMONITOR
// __stdcall
// NtUserMonitorFromPoint(
//      POINT pt,
//      DWORD dwFlags)
// {
//     RECTL rc;
//     HMONITOR hMonitor = NULL;
//
//     /* Check if flags are valid */
//     if (dwFlags != MONITOR_DEFAULTTONULL &&
//         dwFlags != MONITOR_DEFAULTTOPRIMARY &&
//         dwFlags != MONITOR_DEFAULTTONEAREST)
//     {
//         EngSetLastError(ERROR_INVALID_FLAGS);
//         return NULL;
//     }
//
//     /* Fill rect (bottom-right exclusive) */
//     rc.left = pt.x;
//     rc.right = pt.x + 1;
//     rc.top = pt.y;
//     rc.bottom = pt.y + 1;
//
//     UserEnterShared();
//
//     /* Find intersecting monitor */
//     IntGetMonitorsFromRect(&rc, &hMonitor, NULL, 1, dwFlags);
//
//     UserLeave();
//     return hMonitor;
// }
//
// /* NtUserMonitorFromRect
//  *
//  * Returns a handle to the monitor having the largest intersection with a
//  * given rectangle
//  *
//  * Arguments
//  *
//  *   pRectUnsafe
//  *     Pointer to a RECT for which to find monitor
//  *
//  *   dwFlags
//  *     Specifies the behaviour if no monitor intersects the given rect
//  *
//  * Return value
//  *   If a monitor intersects the rect a handle to it is returned; if not the
//  *   return value depends on dwFlags
//  */
// HMONITOR
// __stdcall
// NtUserMonitorFromRect(
//      LPCRECTL pRectUnsafe,
//      DWORD dwFlags)
// {
//     RECTL Rect;
//     NTSTATUS Status;
//     HMONITOR hMonitor;
//
//     /* Check if flags are valid */
//     if (dwFlags != MONITOR_DEFAULTTONULL &&
//         dwFlags != MONITOR_DEFAULTTOPRIMARY &&
//         dwFlags != MONITOR_DEFAULTTONEAREST)
//     {
//         EngSetLastError(ERROR_INVALID_FLAGS);
//         return NULL;
//     }
//
//     /* Copy rectangle to safe buffer */
//     Status = MmCopyFromCaller(&Rect, pRectUnsafe, sizeof (RECT));
//     if (!NT_SUCCESS(Status))
//     {
//         SetLastNtError(Status);
//         return NULL;
//     }
//
//     if (RECTL_bIsEmptyRect(&Rect))
//     {
//         Rect.right = Rect.left + 1;
//         Rect.bottom = Rect.top + 1;
//     }
//
//     UserEnterShared();
//     hMonitor = IntGetMonitorFromRect(&Rect, dwFlags);
//     UserLeave();
//
//     return hMonitor;
// }
//
//
// HMONITOR
// __stdcall
// NtUserMonitorFromWindow(
//      HWND hWnd,
//      DWORD dwFlags)
// {
//     PWND pWnd;
//     HMONITOR hMonitor = NULL;
//     RECTL Rect = {0, 0, 0, 0};
//
//     TRACE("Enter NtUserMonitorFromWindow\n");
//
//     /* Check if flags are valid */
//     if (dwFlags != MONITOR_DEFAULTTONULL &&
//         dwFlags != MONITOR_DEFAULTTOPRIMARY &&
//         dwFlags != MONITOR_DEFAULTTONEAREST)
//     {
//         EngSetLastError(ERROR_INVALID_FLAGS);
//         return NULL;
//     }
//
//     UserEnterShared();
//
//     if (hWnd)
//     {
//         /* Get window object */
//         pWnd = UserGetWindowObject(hWnd);
//         if (pWnd)
//         {
//             Rect = ((pWnd->style & WS_MINIMIZE) ? pWnd->InternalPos.NormalRect : pWnd->rcWindow);
//             if (RECTL_bIsEmptyRect(&Rect))
//             {
//                 Rect.right = Rect.left + 1;
//                 Rect.bottom = Rect.top + 1;
//             }
//         }
//     }
//
//     hMonitor = IntGetMonitorFromRect(&Rect, dwFlags);
//
//     UserLeave();
//
//     TRACE("Leave NtUserMonitorFromWindow, ret=%p\n", hMonitor);
//     return hMonitor;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
