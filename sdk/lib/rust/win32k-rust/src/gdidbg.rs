// win32ss/gdi/ntgdi/gdidbg.c — GDI debug machinery
//
// Phase 2c wave 25 hand-translation.
//
// NESTED #if ANALYSIS (see win32ss/win32kp.h):
//   DBG = 1 (whole file is live)
//   DBG_ENABLE_GDIOBJ_BACKTRACES = 0 → CompareBacktraces, DbgDumpGdiHandleTableWithBT DEAD
//   DBG_ENABLE_EVENT_LOGGING     = 0 → DbgLogEvent, DbgPrintEvent, DbgDumpEventList,
//                                       DbgCleanupEventList DEAD
//   #if 1 || DBG_ENABLE_SERVICE_HOOKS → ALWAYS TRUE →
//                                       DbgDumpLockedGdiHandles, GdiDbgPreServiceHook,
//                                       GdiDbgPostServiceHook LIVE
//
// nm oracle (i386):
//   T _DbgCaptureStackBackTace@12   stdcall/NTAPI
//   D _DbgChannels                  exported global data (75 entries)
//   T _DbgDumpLockedGdiHandles@0    stdcall/NTAPI
//   T _DbgGdiHTIntegrityCheck@0     stdcall/NTAPI
//   T _DbgInitDebugChannels         cdecl (no @N in nm, no NTAPI in C decl)
//   T _GdiDbgPostServiceHook@8      stdcall/NTAPI
//   T _GdiDbgPreServiceHook@8       stdcall/NTAPI
//   T _QueryEnvironmentVariable@8   stdcall/NTAPI
//   B _gulLogUnique                 BSS, defined here
//   t _DbgAddDebugChannel           local text (static fn)
//   t _DbgCompareChannels           local text (static fn)
//   t _DbgParseDebugChannels        local text (static fn)
//
// Private struct reconstructed:
//   DBG_CHANNEL — win32ss/user/ntuser/win32kdebug.h: { PWCHAR Name; ULONG Id }
//   (only present when DBG=1; not in any bindgen)

use win32k_sys::win32k::*;

extern "C" {
    static mut gulFirstFree: ULONG;
    static mut gulFirstUnused: ULONG;
    static mut gpentHmgr: PENTRY;
    static mut GdiHandleTable: PGDI_HANDLE_TABLE;
}

extern "C" {
    fn DbgPrint(fmt: *const u8, ...) -> u32;
}

extern "system" {
    fn ExAllocatePool(PoolType: POOL_TYPE, NumberOfBytes: SIZE_T) -> PVOID;
    fn ExFreePool(P: PVOID);
}

use ntkrnl_sys::ddk::{KeEnterCriticalRegion, KeLeaveCriticalRegion};
use ntkrnl_sys::inline::ke::NtCurrentTeb;

extern "system" {
    fn RtlWalkFrameChain(Callers: *mut PVOID, Count: ULONG, Flags: ULONG) -> ULONG;
    fn RtlEqualUnicodeString(
        String1: *const UNICODE_STRING,
        String2: *const UNICODE_STRING,
        CaseInSensitive: BOOLEAN,
    ) -> BOOLEAN;
    fn PsGetCurrentThreadWin32Thread() -> PTHREADINFO;
    fn PsGetCurrentProcessWin32Process() -> PPROCESSINFO;
}

extern "C" {
    fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize)
        -> *mut core::ffi::c_void;
    fn memmove(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, n: usize)
        -> *mut core::ffi::c_void;
    fn memset(dst: *mut core::ffi::c_void, c: i32, n: usize) -> *mut core::ffi::c_void;
    fn wcslen(s: *const u16) -> usize;
    fn wcschr(s: *const u16, c: u16) -> *mut u16;
    fn wcscmp(s1: *const u16, s2: *const u16) -> i32;
    fn wcsncmp(s1: *const u16, s2: *const u16, n: usize) -> i32;
    fn bsearch(
        key: *const core::ffi::c_void,
        base: *const core::ffi::c_void,
        num: usize,
        size: usize,
        compar: unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> i32,
    ) -> *mut core::ffi::c_void;
}

#[repr(C)]
#[derive(Copy, Clone)]
struct DBG_CHANNEL {
    Name: PWCHAR,
    Id: ULONG,
}

const ERR_LEVEL: u8   = 0x1;
const FIXME_LEVEL: u8 = 0x2;
const WARN_LEVEL: u8  = 0x4;
const TRACE_LEVEL: u8 = 0x8;
const MAX_LEVEL: u8 = ERR_LEVEL | FIXME_LEVEL | WARN_LEVEL | TRACE_LEVEL;

#[allow(non_upper_case_globals)]
const PagedPool: POOL_TYPE = 1;

const STATUS_SUCCESS: NTSTATUS = 0i32;
const STATUS_VARIABLE_NOT_FOUND: NTSTATUS = 0xC0000100u32 as NTSTATUS;
const STATUS_BUFFER_TOO_SMALL: NTSTATUS   = 0xC0000023u32 as NTSTATUS;

#[inline(always)] fn NT_SUCCESS(s: NTSTATUS) -> bool { s >= 0 }
#[inline(always)] fn min_u(a: ULONG, b: ULONG) -> ULONG { if a < b { a } else { b } }

#[inline(always)] unsafe fn RtlZeroMemory(dst: PVOID, n: usize) { memset(dst, 0, n); }
#[inline(always)] unsafe fn RtlCopyMemory(dst: PVOID, src: *const core::ffi::c_void, n: usize) { memcpy(dst, src, n); }
#[inline(always)] unsafe fn RtlFillMemory(dst: PVOID, n: usize, fill: u8) { memset(dst, fill as i32, n); }

#[no_mangle]
pub static mut gulLogUnique: ULONG = 0;

const DbgChEngBlt:           ULONG = 0;
const DbgChEngBrush:         ULONG = 1;
const DbgChEngClip:          ULONG = 2;
const DbgChEngCursor:        ULONG = 3;
const DbgChEngDev:           ULONG = 4;
const DbgChEngErr:           ULONG = 5;
const DbgChEngEvent:         ULONG = 6;
const DbgChEngGrad:          ULONG = 7;
const DbgChEngLDev:          ULONG = 8;
const DbgChEngLine:          ULONG = 9;
const DbgChEngMapping:       ULONG = 10;
const DbgChEngMDev:          ULONG = 11;
const DbgChEngPDev:          ULONG = 12;
const DbgChEngSurface:       ULONG = 13;
const DbgChEngWnd:           ULONG = 14;
const DbgChEngXlate:         ULONG = 15;
const DbgChGdiBitmap:        ULONG = 16;
const DbgChGdiBlt:           ULONG = 17;
const DbgChGdiBrush:         ULONG = 18;
const DbgChGdiClipRgn:       ULONG = 19;
const DbgChGdiCoord:         ULONG = 20;
const DbgChGdiDC:            ULONG = 21;
const DbgChGdiDCAttr:        ULONG = 22;
const DbgChGdiDCState:       ULONG = 23;
const DbgChGdiDev:           ULONG = 24;
const DbgChGdiDib:           ULONG = 25;
const DbgChGdiFont:          ULONG = 26;
const DbgChGdiLine:          ULONG = 27;
const DbgChGdiObj:           ULONG = 28;
const DbgChGdiPalette:       ULONG = 29;
const DbgChGdiPath:          ULONG = 30;
const DbgChGdiPen:           ULONG = 31;
const DbgChGdiPool:          ULONG = 32;
const DbgChGdiRgn:           ULONG = 33;
const DbgChGdiText:          ULONG = 34;
const DbgChGdiXFormObj:      ULONG = 35;
const DbgChUserAccel:        ULONG = 36;
const DbgChUserCallback:     ULONG = 37;
const DbgChUserCallProc:     ULONG = 38;
const DbgChUserCaret:        ULONG = 39;
const DbgChUserClass:        ULONG = 40;
const DbgChUserClipbrd:      ULONG = 41;
const DbgChUserCsr:          ULONG = 42;
const DbgChUserDce:          ULONG = 43;
const DbgChUserDefwnd:       ULONG = 44;
const DbgChUserDesktop:      ULONG = 45;
const DbgChUserDisplay:      ULONG = 46;
const DbgChUserEvent:        ULONG = 47;
const DbgChUserFocus:        ULONG = 48;
const DbgChUserHook:         ULONG = 49;
const DbgChUserHotkey:       ULONG = 50;
const DbgChUserIcon:         ULONG = 51;
const DbgChUserInput:        ULONG = 52;
const DbgChUserKbd:          ULONG = 53;
const DbgChUserKbdLayout:    ULONG = 54;
const DbgChUserMenu:         ULONG = 55;
const DbgChUserMetric:       ULONG = 56;
const DbgChUserMisc:         ULONG = 57;
const DbgChUserMonitor:      ULONG = 58;
const DbgChUserMsg:          ULONG = 59;
const DbgChUserMsgQ:         ULONG = 60;
const DbgChUserObj:          ULONG = 61;
const DbgChUserPainting:     ULONG = 62;
const DbgChUserProcess:      ULONG = 63;
const DbgChUserPowerManager: ULONG = 64;
const DbgChUserProp:         ULONG = 65;
const DbgChUserScrollbar:    ULONG = 66;
const DbgChUserSecurity:     ULONG = 67;
const DbgChUserShutdown:     ULONG = 68;
const DbgChUserSysparams:    ULONG = 69;
const DbgChUserThread:       ULONG = 70;
const DbgChUserTimer:        ULONG = 71;
const DbgChUserWinsta:       ULONG = 72;
const DbgChUserWnd:          ULONG = 73;
const DbgChUserWinpos:       ULONG = 74;
const DbgChCount:            ULONG = 75;

// Wide channel name strings — null-terminated UTF-16. All ASCII so UTF-16 = zero-extend.
// Sizes verified: array size = strlen(name) + 1 (null).
static W_EngBlt:          [u16; 7]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'B' as u16, b'l' as u16, b't' as u16, 0];
static W_EngBrush:        [u16; 9]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'B' as u16, b'r' as u16, b'u' as u16, b's' as u16, b'h' as u16, 0];
static W_EngClip:         [u16; 8]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'C' as u16, b'l' as u16, b'i' as u16, b'p' as u16, 0];
static W_EngCursor:       [u16; 10] = [b'E' as u16, b'n' as u16, b'g' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b's' as u16, b'o' as u16, b'r' as u16, 0];
static W_EngDev:          [u16; 7]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'D' as u16, b'e' as u16, b'v' as u16, 0];
static W_EngErr:          [u16; 7]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'E' as u16, b'r' as u16, b'r' as u16, 0];
static W_EngEvent:        [u16; 9]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'E' as u16, b'v' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0];
static W_EngGrad:         [u16; 8]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'G' as u16, b'r' as u16, b'a' as u16, b'd' as u16, 0];
static W_EngLDev:         [u16; 8]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'L' as u16, b'D' as u16, b'e' as u16, b'v' as u16, 0];
static W_EngLine:         [u16; 8]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'L' as u16, b'i' as u16, b'n' as u16, b'e' as u16, 0];
static W_EngMapping:      [u16; 11] = [b'E' as u16, b'n' as u16, b'g' as u16, b'M' as u16, b'a' as u16, b'p' as u16, b'p' as u16, b'i' as u16, b'n' as u16, b'g' as u16, 0];
static W_EngMDev:         [u16; 8]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'M' as u16, b'D' as u16, b'e' as u16, b'v' as u16, 0];
static W_EngPDev:         [u16; 8]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'P' as u16, b'D' as u16, b'e' as u16, b'v' as u16, 0];
static W_EngSurface:      [u16; 11] = [b'E' as u16, b'n' as u16, b'g' as u16, b'S' as u16, b'u' as u16, b'r' as u16, b'f' as u16, b'a' as u16, b'c' as u16, b'e' as u16, 0];
static W_EngWnd:          [u16; 7]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'W' as u16, b'n' as u16, b'd' as u16, 0];
static W_EngXlate:        [u16; 9]  = [b'E' as u16, b'n' as u16, b'g' as u16, b'X' as u16, b'l' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0];
static W_GdiBitmap:       [u16; 10] = [b'G' as u16, b'd' as u16, b'i' as u16, b'B' as u16, b'i' as u16, b't' as u16, b'm' as u16, b'a' as u16, b'p' as u16, 0];
static W_GdiBlt:          [u16; 7]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'B' as u16, b'l' as u16, b't' as u16, 0];
static W_GdiBrush:        [u16; 9]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'B' as u16, b'r' as u16, b'u' as u16, b's' as u16, b'h' as u16, 0];
static W_GdiClipRgn:      [u16; 11] = [b'G' as u16, b'd' as u16, b'i' as u16, b'C' as u16, b'l' as u16, b'i' as u16, b'p' as u16, b'R' as u16, b'g' as u16, b'n' as u16, 0];
static W_GdiCoord:        [u16; 9]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'C' as u16, b'o' as u16, b'o' as u16, b'r' as u16, b'd' as u16, 0];
static W_GdiDC:           [u16; 6]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'D' as u16, b'C' as u16, 0];
static W_GdiDCAttr:       [u16; 10] = [b'G' as u16, b'd' as u16, b'i' as u16, b'D' as u16, b'C' as u16, b'A' as u16, b't' as u16, b't' as u16, b'r' as u16, 0];
static W_GdiDCState:      [u16; 11] = [b'G' as u16, b'd' as u16, b'i' as u16, b'D' as u16, b'C' as u16, b'S' as u16, b't' as u16, b'a' as u16, b't' as u16, b'e' as u16, 0];
static W_GdiDev:          [u16; 7]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'D' as u16, b'e' as u16, b'v' as u16, 0];
static W_GdiDib:          [u16; 7]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'D' as u16, b'i' as u16, b'b' as u16, 0];
static W_GdiFont:         [u16; 8]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'F' as u16, b'o' as u16, b'n' as u16, b't' as u16, 0];
static W_GdiLine:         [u16; 8]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'L' as u16, b'i' as u16, b'n' as u16, b'e' as u16, 0];
static W_GdiObj:          [u16; 7]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'O' as u16, b'b' as u16, b'j' as u16, 0];
static W_GdiPalette:      [u16; 11] = [b'G' as u16, b'd' as u16, b'i' as u16, b'P' as u16, b'a' as u16, b'l' as u16, b'e' as u16, b't' as u16, b't' as u16, b'e' as u16, 0];
static W_GdiPath:         [u16; 8]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'P' as u16, b'a' as u16, b't' as u16, b'h' as u16, 0];
static W_GdiPen:          [u16; 7]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'P' as u16, b'e' as u16, b'n' as u16, 0];
static W_GdiPool:         [u16; 8]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'P' as u16, b'o' as u16, b'o' as u16, b'l' as u16, 0];
static W_GdiRgn:          [u16; 7]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'R' as u16, b'g' as u16, b'n' as u16, 0];
static W_GdiText:         [u16; 8]  = [b'G' as u16, b'd' as u16, b'i' as u16, b'T' as u16, b'e' as u16, b'x' as u16, b't' as u16, 0];
static W_GdiXFormObj:     [u16; 12] = [b'G' as u16, b'd' as u16, b'i' as u16, b'X' as u16, b'F' as u16, b'o' as u16, b'r' as u16, b'm' as u16, b'O' as u16, b'b' as u16, b'j' as u16, 0];
static W_UserAccel:       [u16; 10] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'A' as u16, b'c' as u16, b'c' as u16, b'e' as u16, b'l' as u16, 0];
static W_UserCallback:    [u16; 13] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'C' as u16, b'a' as u16, b'l' as u16, b'l' as u16, b'b' as u16, b'a' as u16, b'c' as u16, b'k' as u16, 0];
static W_UserCallProc:    [u16; 13] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'C' as u16, b'a' as u16, b'l' as u16, b'l' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'c' as u16, 0];
static W_UserCaret:       [u16; 10] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'C' as u16, b'a' as u16, b'r' as u16, b'e' as u16, b't' as u16, 0];
static W_UserClass:       [u16; 10] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'C' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, 0];
static W_UserClipbrd:     [u16; 12] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'C' as u16, b'l' as u16, b'i' as u16, b'p' as u16, b'b' as u16, b'r' as u16, b'd' as u16, 0];
static W_UserCsr:         [u16; 8]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'C' as u16, b's' as u16, b'r' as u16, 0];
static W_UserDce:         [u16; 8]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'D' as u16, b'c' as u16, b'e' as u16, 0];
static W_UserDefwnd:      [u16; 11] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'D' as u16, b'e' as u16, b'f' as u16, b'w' as u16, b'n' as u16, b'd' as u16, 0];
static W_UserDesktop:     [u16; 12] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'D' as u16, b'e' as u16, b's' as u16, b'k' as u16, b't' as u16, b'o' as u16, b'p' as u16, 0];
static W_UserDisplay:     [u16; 12] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'D' as u16, b'i' as u16, b's' as u16, b'p' as u16, b'l' as u16, b'a' as u16, b'y' as u16, 0];
static W_UserEvent:       [u16; 10] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'E' as u16, b'v' as u16, b'e' as u16, b'n' as u16, b't' as u16, 0];
static W_UserFocus:       [u16; 10] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'F' as u16, b'o' as u16, b'c' as u16, b'u' as u16, b's' as u16, 0];
static W_UserHook:        [u16; 9]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'H' as u16, b'o' as u16, b'o' as u16, b'k' as u16, 0];
static W_UserHotkey:      [u16; 11] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'H' as u16, b'o' as u16, b't' as u16, b'k' as u16, b'e' as u16, b'y' as u16, 0];
static W_UserIcon:        [u16; 9]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'I' as u16, b'c' as u16, b'o' as u16, b'n' as u16, 0];
static W_UserInput:       [u16; 10] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'I' as u16, b'n' as u16, b'p' as u16, b'u' as u16, b't' as u16, 0];
static W_UserKbd:         [u16; 8]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'K' as u16, b'b' as u16, b'd' as u16, 0];
static W_UserKbdLayout:   [u16; 14] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'K' as u16, b'b' as u16, b'd' as u16, b'L' as u16, b'a' as u16, b'y' as u16, b'o' as u16, b'u' as u16, b't' as u16, 0];
static W_UserMenu:        [u16; 9]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'M' as u16, b'e' as u16, b'n' as u16, b'u' as u16, 0];
static W_UserMetric:      [u16; 11] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'M' as u16, b'e' as u16, b't' as u16, b'r' as u16, b'i' as u16, b'c' as u16, 0];
static W_UserMisc:        [u16; 9]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'M' as u16, b'i' as u16, b's' as u16, b'c' as u16, 0];
static W_UserMonitor:     [u16; 12] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'M' as u16, b'o' as u16, b'n' as u16, b'i' as u16, b't' as u16, b'o' as u16, b'r' as u16, 0];
static W_UserMsg:         [u16; 8]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'M' as u16, b's' as u16, b'g' as u16, 0];
static W_UserMsgQ:        [u16; 9]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'M' as u16, b's' as u16, b'g' as u16, b'Q' as u16, 0];
static W_UserObj:         [u16; 8]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'O' as u16, b'b' as u16, b'j' as u16, 0];
static W_UserPainting:    [u16; 13] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'P' as u16, b'a' as u16, b'i' as u16, b'n' as u16, b't' as u16, b'i' as u16, b'n' as u16, b'g' as u16, 0];
static W_UserProcess:     [u16; 12] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'c' as u16, b'e' as u16, b's' as u16, b's' as u16, 0];
static W_UserPowerManager:[u16; 17] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'P' as u16, b'o' as u16, b'w' as u16, b'e' as u16, b'r' as u16, b'M' as u16, b'a' as u16, b'n' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, 0];
static W_UserProp:        [u16; 9]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'p' as u16, 0];
static W_UserScrollbar:   [u16; 14] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'S' as u16, b'c' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'l' as u16, b'b' as u16, b'a' as u16, b'r' as u16, 0];
static W_UserSecurity:    [u16; 13] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'S' as u16, b'e' as u16, b'c' as u16, b'u' as u16, b'r' as u16, b'i' as u16, b't' as u16, b'y' as u16, 0];
static W_UserShutdown:    [u16; 13] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'S' as u16, b'h' as u16, b'u' as u16, b't' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'n' as u16, 0];
static W_UserSysparams:   [u16; 14] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'S' as u16, b'y' as u16, b's' as u16, b'p' as u16, b'a' as u16, b'r' as u16, b'a' as u16, b'm' as u16, b's' as u16, 0];
static W_UserThread:      [u16; 11] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'T' as u16, b'h' as u16, b'r' as u16, b'e' as u16, b'a' as u16, b'd' as u16, 0];
static W_UserTimer:       [u16; 10] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16, b'r' as u16, 0];
static W_UserWinpos:      [u16; 11] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'p' as u16, b'o' as u16, b's' as u16, 0];
static W_UserWinsta:      [u16; 11] = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b's' as u16, b't' as u16, b'a' as u16, 0];
static W_UserWnd:         [u16; 8]  = [b'U' as u16, b's' as u16, b'e' as u16, b'r' as u16, b'W' as u16, b'n' as u16, b'd' as u16, 0];
static W_all:             [u16; 4]  = [b'a' as u16, b'l' as u16, b'l' as u16, 0];

#[no_mangle]
pub static mut DbgChannels: [DBG_CHANNEL; 75] = [
    DBG_CHANNEL { Name: W_EngBlt.as_ptr()          as PWCHAR, Id: DbgChEngBlt },
    DBG_CHANNEL { Name: W_EngBrush.as_ptr()        as PWCHAR, Id: DbgChEngBrush },
    DBG_CHANNEL { Name: W_EngClip.as_ptr()         as PWCHAR, Id: DbgChEngClip },
    DBG_CHANNEL { Name: W_EngCursor.as_ptr()       as PWCHAR, Id: DbgChEngCursor },
    DBG_CHANNEL { Name: W_EngDev.as_ptr()          as PWCHAR, Id: DbgChEngDev },
    DBG_CHANNEL { Name: W_EngErr.as_ptr()          as PWCHAR, Id: DbgChEngErr },
    DBG_CHANNEL { Name: W_EngEvent.as_ptr()        as PWCHAR, Id: DbgChEngEvent },
    DBG_CHANNEL { Name: W_EngGrad.as_ptr()         as PWCHAR, Id: DbgChEngGrad },
    DBG_CHANNEL { Name: W_EngLDev.as_ptr()         as PWCHAR, Id: DbgChEngLDev },
    DBG_CHANNEL { Name: W_EngLine.as_ptr()         as PWCHAR, Id: DbgChEngLine },
    DBG_CHANNEL { Name: W_EngMapping.as_ptr()      as PWCHAR, Id: DbgChEngMapping },
    DBG_CHANNEL { Name: W_EngMDev.as_ptr()         as PWCHAR, Id: DbgChEngMDev },
    DBG_CHANNEL { Name: W_EngPDev.as_ptr()         as PWCHAR, Id: DbgChEngPDev },
    DBG_CHANNEL { Name: W_EngSurface.as_ptr()      as PWCHAR, Id: DbgChEngSurface },
    DBG_CHANNEL { Name: W_EngWnd.as_ptr()          as PWCHAR, Id: DbgChEngWnd },
    DBG_CHANNEL { Name: W_EngXlate.as_ptr()        as PWCHAR, Id: DbgChEngXlate },
    DBG_CHANNEL { Name: W_GdiBitmap.as_ptr()       as PWCHAR, Id: DbgChGdiBitmap },
    DBG_CHANNEL { Name: W_GdiBlt.as_ptr()          as PWCHAR, Id: DbgChGdiBlt },
    DBG_CHANNEL { Name: W_GdiBrush.as_ptr()        as PWCHAR, Id: DbgChGdiBrush },
    DBG_CHANNEL { Name: W_GdiClipRgn.as_ptr()      as PWCHAR, Id: DbgChGdiClipRgn },
    DBG_CHANNEL { Name: W_GdiCoord.as_ptr()        as PWCHAR, Id: DbgChGdiCoord },
    DBG_CHANNEL { Name: W_GdiDC.as_ptr()           as PWCHAR, Id: DbgChGdiDC },
    DBG_CHANNEL { Name: W_GdiDCAttr.as_ptr()       as PWCHAR, Id: DbgChGdiDCAttr },
    DBG_CHANNEL { Name: W_GdiDCState.as_ptr()      as PWCHAR, Id: DbgChGdiDCState },
    DBG_CHANNEL { Name: W_GdiDev.as_ptr()          as PWCHAR, Id: DbgChGdiDev },
    DBG_CHANNEL { Name: W_GdiDib.as_ptr()          as PWCHAR, Id: DbgChGdiDib },
    DBG_CHANNEL { Name: W_GdiFont.as_ptr()         as PWCHAR, Id: DbgChGdiFont },
    DBG_CHANNEL { Name: W_GdiLine.as_ptr()         as PWCHAR, Id: DbgChGdiLine },
    DBG_CHANNEL { Name: W_GdiObj.as_ptr()          as PWCHAR, Id: DbgChGdiObj },
    DBG_CHANNEL { Name: W_GdiPalette.as_ptr()      as PWCHAR, Id: DbgChGdiPalette },
    DBG_CHANNEL { Name: W_GdiPath.as_ptr()         as PWCHAR, Id: DbgChGdiPath },
    DBG_CHANNEL { Name: W_GdiPen.as_ptr()          as PWCHAR, Id: DbgChGdiPen },
    DBG_CHANNEL { Name: W_GdiPool.as_ptr()         as PWCHAR, Id: DbgChGdiPool },
    DBG_CHANNEL { Name: W_GdiRgn.as_ptr()          as PWCHAR, Id: DbgChGdiRgn },
    DBG_CHANNEL { Name: W_GdiText.as_ptr()         as PWCHAR, Id: DbgChGdiText },
    DBG_CHANNEL { Name: W_GdiXFormObj.as_ptr()     as PWCHAR, Id: DbgChGdiXFormObj },
    DBG_CHANNEL { Name: W_UserAccel.as_ptr()       as PWCHAR, Id: DbgChUserAccel },
    DBG_CHANNEL { Name: W_UserCallback.as_ptr()    as PWCHAR, Id: DbgChUserCallback },
    DBG_CHANNEL { Name: W_UserCallProc.as_ptr()    as PWCHAR, Id: DbgChUserCallProc },
    DBG_CHANNEL { Name: W_UserCaret.as_ptr()       as PWCHAR, Id: DbgChUserCaret },
    DBG_CHANNEL { Name: W_UserClass.as_ptr()       as PWCHAR, Id: DbgChUserClass },
    DBG_CHANNEL { Name: W_UserClipbrd.as_ptr()     as PWCHAR, Id: DbgChUserClipbrd },
    DBG_CHANNEL { Name: W_UserCsr.as_ptr()         as PWCHAR, Id: DbgChUserCsr },
    DBG_CHANNEL { Name: W_UserDce.as_ptr()         as PWCHAR, Id: DbgChUserDce },
    DBG_CHANNEL { Name: W_UserDefwnd.as_ptr()      as PWCHAR, Id: DbgChUserDefwnd },
    DBG_CHANNEL { Name: W_UserDesktop.as_ptr()     as PWCHAR, Id: DbgChUserDesktop },
    DBG_CHANNEL { Name: W_UserDisplay.as_ptr()     as PWCHAR, Id: DbgChUserDisplay },
    DBG_CHANNEL { Name: W_UserEvent.as_ptr()       as PWCHAR, Id: DbgChUserEvent },
    DBG_CHANNEL { Name: W_UserFocus.as_ptr()       as PWCHAR, Id: DbgChUserFocus },
    DBG_CHANNEL { Name: W_UserHook.as_ptr()        as PWCHAR, Id: DbgChUserHook },
    DBG_CHANNEL { Name: W_UserHotkey.as_ptr()      as PWCHAR, Id: DbgChUserHotkey },
    DBG_CHANNEL { Name: W_UserIcon.as_ptr()        as PWCHAR, Id: DbgChUserIcon },
    DBG_CHANNEL { Name: W_UserInput.as_ptr()       as PWCHAR, Id: DbgChUserInput },
    DBG_CHANNEL { Name: W_UserKbd.as_ptr()         as PWCHAR, Id: DbgChUserKbd },
    DBG_CHANNEL { Name: W_UserKbdLayout.as_ptr()   as PWCHAR, Id: DbgChUserKbdLayout },
    DBG_CHANNEL { Name: W_UserMenu.as_ptr()        as PWCHAR, Id: DbgChUserMenu },
    DBG_CHANNEL { Name: W_UserMetric.as_ptr()      as PWCHAR, Id: DbgChUserMetric },
    DBG_CHANNEL { Name: W_UserMisc.as_ptr()        as PWCHAR, Id: DbgChUserMisc },
    DBG_CHANNEL { Name: W_UserMonitor.as_ptr()     as PWCHAR, Id: DbgChUserMonitor },
    DBG_CHANNEL { Name: W_UserMsg.as_ptr()         as PWCHAR, Id: DbgChUserMsg },
    DBG_CHANNEL { Name: W_UserMsgQ.as_ptr()        as PWCHAR, Id: DbgChUserMsgQ },
    DBG_CHANNEL { Name: W_UserObj.as_ptr()         as PWCHAR, Id: DbgChUserObj },
    DBG_CHANNEL { Name: W_UserPainting.as_ptr()    as PWCHAR, Id: DbgChUserPainting },
    DBG_CHANNEL { Name: W_UserProcess.as_ptr()     as PWCHAR, Id: DbgChUserProcess },
    DBG_CHANNEL { Name: W_UserPowerManager.as_ptr() as PWCHAR, Id: DbgChUserPowerManager },
    DBG_CHANNEL { Name: W_UserProp.as_ptr()        as PWCHAR, Id: DbgChUserProp },
    DBG_CHANNEL { Name: W_UserScrollbar.as_ptr()   as PWCHAR, Id: DbgChUserScrollbar },
    DBG_CHANNEL { Name: W_UserSecurity.as_ptr()    as PWCHAR, Id: DbgChUserSecurity },
    DBG_CHANNEL { Name: W_UserShutdown.as_ptr()    as PWCHAR, Id: DbgChUserShutdown },
    DBG_CHANNEL { Name: W_UserSysparams.as_ptr()   as PWCHAR, Id: DbgChUserSysparams },
    DBG_CHANNEL { Name: W_UserTimer.as_ptr()       as PWCHAR, Id: DbgChUserTimer },
    DBG_CHANNEL { Name: W_UserThread.as_ptr()      as PWCHAR, Id: DbgChUserThread },
    DBG_CHANNEL { Name: W_UserWinpos.as_ptr()      as PWCHAR, Id: DbgChUserWinpos },
    DBG_CHANNEL { Name: W_UserWinsta.as_ptr()      as PWCHAR, Id: DbgChUserWinsta },
    DBG_CHANNEL { Name: W_UserWnd.as_ptr()         as PWCHAR, Id: DbgChUserWnd },
];

/// ULONG NTAPI DbgCaptureStackBackTace(PVOID* ppvFrames, ULONG cFramesToSkip,
///                                      ULONG cFramesToCapture)
/// i386: _DbgCaptureStackBackTace@12
#[no_mangle]
pub unsafe extern "system" fn DbgCaptureStackBackTace(
    ppvFrames: *mut PVOID,
    cFramesToSkip: ULONG,
    cFramesToCapture: ULONG,
) -> ULONG {
    let mut cFrameCount: ULONG;
    let mut apvTemp: [PVOID; 30] = [core::ptr::null_mut(); 30];

    RtlZeroMemory(ppvFrames as PVOID, cFramesToCapture as usize * core::mem::size_of::<PVOID>());

    cFrameCount = RtlWalkFrameChain(apvTemp.as_mut_ptr(), 30, 0);

    if cFramesToSkip > cFrameCount { return 0; }

    cFrameCount -= cFramesToSkip;
    cFrameCount = min_u(cFrameCount, cFramesToCapture);
    memcpy(
        ppvFrames as *mut core::ffi::c_void,
        apvTemp.as_ptr().add(cFramesToSkip as usize) as *const core::ffi::c_void,
        cFrameCount as usize * core::mem::size_of::<PVOID>(),
    );

    if cFrameCount < cFramesToCapture {
        cFrameCount += RtlWalkFrameChain(
            ppvFrames.add(cFrameCount as usize),
            cFramesToCapture - cFrameCount,
            1,
        );
    }

    cFrameCount
}


/// user_space_top_addr() — approximates MmHighestUserAddress without an import-table reference.
/// On i386: user space tops at 0x7FFFFFFF (2GB user / 2GB kernel split).
/// On amd64: user space tops at 0x00007FFFFFFFFFFF (128 TB user).
/// This is a conservative constant; the real MmHighestUserAddress is <= this value.
#[inline(always)]
fn user_space_top_addr() -> usize {
    #[cfg(target_arch = "x86")]
    { 0x7FFF_FFFFusize }
    #[cfg(not(target_arch = "x86"))]
    { 0x0000_7FFF_FFFF_FFFFusize }
}

/// BOOL NTAPI DbgGdiHTIntegrityCheck(VOID)
/// i386: _DbgGdiHTIntegrityCheck@0
#[no_mangle]
pub unsafe extern "system" fn DbgGdiHTIntegrityCheck() -> BOOL {
    let mut i: ULONG;
    let mut nDeleted: ULONG = 0;
    let mut nFree: ULONG = 0;
    let mut nUsed: ULONG = 0;
    let mut pEntry: PGDI_TABLE_ENTRY;
    let mut r: BOOL = 1;

    KeEnterCriticalRegion();

    i = gulFirstFree & 0xffff;
    while i != 0 {
        pEntry = &mut (*GdiHandleTable).Entries[i as usize];
        if i >= GDI_HANDLE_COUNT {
            DbgPrint(b"gdidbg: nDeleted=%lu\n\0".as_ptr(), nDeleted);
            break;
        }
        nDeleted += 1;
        let entry_type = (*pEntry).__bindgen_anon_1.Type as u32;
        if (entry_type & GDI_ENTRY_BASETYPE_MASK) != 0 { r = 0; DbgPrint(b"gdidbg: Deleted Entry has a type != 0\n\0".as_ptr()); }
        if (*pEntry).KernelData as usize >= GDI_HANDLE_COUNT as usize { r = 0; DbgPrint(b"gdidbg: Deleted entries KernelPointer too big\n\0".as_ptr()); }
        if !(*pEntry).UserData.is_null() { r = 0; DbgPrint(b"gdidbg: Deleted entry has UserData != 0\n\0".as_ptr()); }
        if (*pEntry).ProcessId != 0 { r = 0; DbgPrint(b"gdidbg: Deleted entry has ProcessId != 0\n\0".as_ptr()); }
        i = (*pEntry).KernelData as usize as ULONG & 0xffff;
    }

    i = gulFirstUnused;
    while i < GDI_HANDLE_COUNT {
        pEntry = &mut (*GdiHandleTable).Entries[i as usize];
        if (*pEntry).__bindgen_anon_1.Type != 0 { r = 0; DbgPrint(b"gdidbg: Free Entry has a type != 0\n\0".as_ptr()); }
        if (*pEntry).KernelData as usize != 0 { r = 0; DbgPrint(b"gdidbg: Free entries KernelPointer != 0\n\0".as_ptr()); }
        if !(*pEntry).UserData.is_null() { r = 0; DbgPrint(b"gdidbg: Free entry has UserData != 0\n\0".as_ptr()); }
        if (*pEntry).ProcessId != 0 { r = 0; DbgPrint(b"gdidbg: Free entry has ProcessId != 0\n\0".as_ptr()); }
        nFree += 1;
        i += 1;
    }

    i = RESERVE_ENTRIES_COUNT;
    while i < GDI_HANDLE_COUNT {
        pEntry = &mut (*GdiHandleTable).Entries[i as usize];
        let entry_type_long = (*pEntry).__bindgen_anon_1.Type;
        let entry_type = entry_type_long as u32;
        let hv = (entry_type_long << GDI_ENTRY_UPPER_SHIFT as i32) as u32;
        let Handle: HGDIOBJ = hv.wrapping_add(i) as usize as HGDIOBJ;

        if (entry_type & GDI_ENTRY_BASETYPE_MASK) != 0 {
            if (*pEntry).KernelData.is_null() {
                r = 0; DbgPrint(b"gdidbg: Used entry has KernelData == 0\n\0".as_ptr());
            } else if (*pEntry).KernelData as usize <= user_space_top_addr() {
                r = 0; DbgPrint(b"gdidbg: Used entry invalid KernelData\n\0".as_ptr());
            } else if (*((*pEntry).KernelData as POBJ)).hHmgr != Handle {
                r = 0;
                DbgPrint(b"gdidbg: Used entry %lu, has invalid hHmg %p (expected: %p)\n\0".as_ptr(),
                    i, (*((*pEntry).KernelData as POBJ)).hHmgr, Handle);
            }
            nUsed += 1;
        }
        i += 1;
    }

    if RESERVE_ENTRIES_COUNT + nDeleted + nFree + nUsed != GDI_HANDLE_COUNT {
        r = 0;
        DbgPrint(b"gdidbg: entries wrong: RESERVE=%lu nDeleted=%lu nFree=%lu nUsed=%lu\n\0".as_ptr(),
            RESERVE_ENTRIES_COUNT, nDeleted, nFree, nUsed);
    }

    KeLeaveCriticalRegion();
    r
}

// DBG_ENABLE_EVENT_LOGGING = 0 — dead region omitted

/// VOID NTAPI DbgDumpLockedGdiHandles(VOID)
/// i386: _DbgDumpLockedGdiHandles@0
#[no_mangle]
pub unsafe extern "system" fn DbgDumpLockedGdiHandles() {
    let mut i: ULONG = RESERVE_ENTRIES_COUNT;
    while i < GDI_HANDLE_COUNT {
        let pentry: PENTRY = gpentHmgr.add(i as usize);
        if (*pentry).Objt != 0 {
            let pobj: POBJ = (*pentry).einfo.pobj;
            if (*pobj).cExclusiveLock > 0 {
                DbgPrint(b"Locked object: %lx, type = %lx. allocated from:\n\0".as_ptr(),
                    i, (*pentry).Objt as ULONG);
                // DBG_DUMP_EVENT_LIST → no-op when EVENT_LOGGING=0
            }
        }
        i += 1;
    }
}

/// void NTAPI GdiDbgPreServiceHook(ULONG ulSyscallId, PULONG_PTR pulArguments)
/// i386: _GdiDbgPreServiceHook@8
#[no_mangle]
pub unsafe extern "system" fn GdiDbgPreServiceHook(
    ulSyscallId: ULONG,
    _pulArguments: PULONG_PTR,
) {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    if !pti.is_null() && (*pti).cExclusiveLocks != 0 {
        DbgPrint(b"FATAL: Win32DbgPreServiceHook(0x%lx): There are %lu exclusive locks!\n\0".as_ptr(),
            ulSyscallId, (*pti).cExclusiveLocks);
        DbgDumpLockedGdiHandles();
    }
}

/// ULONG_PTR NTAPI GdiDbgPostServiceHook(ULONG ulSyscallId, ULONG_PTR ulResult)
/// i386: _GdiDbgPostServiceHook@8
#[no_mangle]
pub unsafe extern "system" fn GdiDbgPostServiceHook(
    ulSyscallId: ULONG,
    ulResult: ULONG_PTR,
) -> ULONG_PTR {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread();
    if !pti.is_null() && (*pti).cExclusiveLocks != 0 {
        DbgPrint(b"FATAL: Win32DbgPostServiceHook(0x%lx): There are %lu exclusive locks!\n\0".as_ptr(),
            ulSyscallId, (*pti).cExclusiveLocks);
        DbgDumpLockedGdiHandles();
    }
    ulResult
}

/// NTSTATUS NTAPI QueryEnvironmentVariable(PUNICODE_STRING Name, PUNICODE_STRING Value)
/// i386: _QueryEnvironmentVariable@8
///
/// PEB.ProcessParameters offset: i386=0x10, amd64=0x20
/// RTL_USER_PROCESS_PARAMETERS.Environment offset: i386=0x48, amd64=0x80
#[no_mangle]
pub unsafe extern "system" fn QueryEnvironmentVariable(
    Name: *const UNICODE_STRING,
    Value: *mut UNICODE_STRING,
) -> NTSTATUS {
    // Ugly HACK for ReactOS system threads — system threads have no PEB.
    // Read TEB->ProcessEnvironmentBlock portably: NtCurrentTeb() resolves the
    // platform TEB self-pointer per arch (FS/GS on x86, x18 on arm64), then the
    // PEB sits at TEB+0x30 (i386) / TEB+0x60 (amd64 + arm64 share the 64-bit
    // layout). A null TEB/PEB (system thread) is handled below.
    #[cfg(target_arch = "x86")]
    const TEB_PEB_OFFSET: usize = 0x30;
    #[cfg(not(target_arch = "x86"))]
    const TEB_PEB_OFFSET: usize = 0x60;
    let peb: *mut core::ffi::c_void = {
        let teb = NtCurrentTeb() as usize;
        if teb == 0 {
            core::ptr::null_mut()
        } else {
            core::ptr::read((teb + TEB_PEB_OFFSET) as *const *mut core::ffi::c_void)
        }
    };
    if peb.is_null() {
        return STATUS_VARIABLE_NOT_FOUND;
    }

    #[cfg(target_arch = "x86")]
    let process_parameters_ptr: *mut core::ffi::c_void = {
        core::ptr::read(((peb as usize) + 0x10) as *const *mut core::ffi::c_void)
    };
    #[cfg(not(target_arch = "x86"))]
    let process_parameters_ptr: *mut core::ffi::c_void = {
        core::ptr::read(((peb as usize) + 0x20) as *const *mut core::ffi::c_void)
    };

    if process_parameters_ptr.is_null() {
        return STATUS_VARIABLE_NOT_FOUND;
    }

    #[cfg(target_arch = "x86")]
    let environment_ptr: *mut u16 = {
        core::ptr::read(((process_parameters_ptr as usize) + 0x48) as *const *mut u16)
    };
    #[cfg(not(target_arch = "x86"))]
    let environment_ptr: *mut u16 = {
        core::ptr::read(((process_parameters_ptr as usize) + 0x80) as *const *mut u16)
    };

    if environment_ptr.is_null() {
        return STATUS_VARIABLE_NOT_FOUND;
    }

    (*Value).Length = 0;

    let mut wcs: *mut u16 = environment_ptr;
    while *wcs != 0 {
        let var_buf: *mut u16 = wcs;
        wcs = wcs.add(1);

        let eq = wcschr(wcs, b'=' as u16);
        if !eq.is_null() {
            wcs = eq;
        } else {
            wcs = var_buf.add(wcslen(var_buf));
        }

        if *wcs != 0 {
            // Bytes, as the retired gdidbg.c meant: `(wcs - var.Buffer) * sizeof(WCHAR)`.
            let var_len = (wcs as usize - var_buf as usize) as u16; // nolint: ptr-byte-diff
            wcs = wcs.add(1);
            let val: *mut u16 = wcs;
            wcs = wcs.add(wcslen(wcs));

            let var_us = UNICODE_STRING {
                Length: var_len,
                MaximumLength: var_len,
                Buffer: var_buf as PWSTR,
            };

            if RtlEqualUnicodeString(&var_us, Name, 1) != 0 {
                let val_byte_len = (wcs as usize - val as usize) as u16; // nolint: ptr-byte-diff
                (*Value).Length = val_byte_len;
                if val_byte_len <= (*Value).MaximumLength {
                    let copy_len = min_u(
                        (val_byte_len as usize + core::mem::size_of::<u16>()) as ULONG,
                        (*Value).MaximumLength as ULONG,
                    ) as usize;
                    memcpy((*Value).Buffer as *mut core::ffi::c_void, val as *const core::ffi::c_void, copy_len);
                    return STATUS_SUCCESS;
                } else {
                    return STATUS_BUFFER_TOO_SMALL;
                }
            }
        }
        wcs = wcs.add(1);
    }

    STATUS_VARIABLE_NOT_FOUND
}

unsafe extern "C" fn DbgCompareChannels(
    a: *const core::ffi::c_void,
    b: *const core::ffi::c_void,
) -> i32 {
    wcscmp(a as *const u16, (*(b as *const DBG_CHANNEL)).Name as *const u16)
}

unsafe fn DbgAddDebugChannel(
    ppi: PPROCESSINFO,
    channel: *const u16,
    level: *const u16,
    op: u16,
) -> BOOL {
    if wcscmp(channel, W_all.as_ptr()) == 0 {
        let mut ich: ULONG = 0;
        while ich < DbgChCount {
            DbgAddDebugChannel(ppi, DbgChannels[ich as usize].Name as *const u16, level, op);
            ich += 1;
        }
        return 1;
    }

    let found = bsearch(
        channel as *const core::ffi::c_void,
        (*(&raw const DbgChannels)).as_ptr() as *const core::ffi::c_void,
        DbgChCount as usize,
        core::mem::size_of::<DBG_CHANNEL>(),
        DbgCompareChannels,
    );
    if found.is_null() { return 0; }

    let iChannel = (*(found as *const DBG_CHANNEL)).Id;

    let iLevel: u8;
    if level.is_null() || *level == 0 || wcslen(level) == 0 {
        iLevel = MAX_LEVEL;
    } else if wcsncmp(level, [b'e' as u16, b'r' as u16, b'r' as u16, 0u16].as_ptr(), 3) == 0 {
        iLevel = ERR_LEVEL;
    } else if wcsncmp(level, [b'f' as u16, b'i' as u16, b'x' as u16, b'm' as u16, b'e' as u16, 0u16].as_ptr(), 5) == 0 {
        iLevel = FIXME_LEVEL;
    } else if wcsncmp(level, [b'w' as u16, b'a' as u16, b'r' as u16, b'n' as u16, 0u16].as_ptr(), 4) == 0 {
        iLevel = WARN_LEVEL;
    } else if wcsncmp(level, [b't' as u16, b'r' as u16, b'a' as u16, b'c' as u16, b'e' as u16, 0u16].as_ptr(), 4) == 0 {
        iLevel = TRACE_LEVEL;
    } else {
        return 0;
    }

    if op == b'+' as u16 {
        (*ppi).DbgChannelLevel[iChannel as usize] |= iLevel;
    } else {
        (*ppi).DbgChannelLevel[iChannel as usize] &= !iLevel;
    }
    1
}

unsafe fn DbgParseDebugChannels(ppi: PPROCESSINFO, Value: *const UNICODE_STRING) -> BOOL {
    let mut str_ptr: *mut u16 = (*Value).Buffer;
    loop {
        let separator = wcschr(str_ptr, b',' as u16);
        if !separator.is_null() { *separator = 0; }

        let mut c = wcschr(str_ptr, b'+' as u16);
        if c.is_null() { c = wcschr(str_ptr, b'-' as u16); }

        if !c.is_null() {
            let op = *c;
            *c = 0;
            c = c.add(1);
            DbgAddDebugChannel(ppi, c, str_ptr, op);
        }

        if separator.is_null() { break; }
        str_ptr = separator.add(1);
    }
    1
}

/// BOOL DbgInitDebugChannels(VOID)
/// i386: _DbgInitDebugChannels  (cdecl, no NTAPI/no @N)
#[no_mangle]
pub unsafe extern "C" fn DbgInitDebugChannels() -> BOOL {
    let mut val_buffer: [u16; 100] = [0u16; 100];

    let ppi: PPROCESSINFO = PsGetCurrentProcessWin32Process();
    RtlFillMemory(
        (*ppi).DbgChannelLevel.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&(*ppi).DbgChannelLevel),
        ERR_LEVEL,
    );

    static W_DEBUGCHANNEL: [u16; 13] = [
        b'D' as u16, b'E' as u16, b'B' as u16, b'U' as u16,
        b'G' as u16, b'C' as u16, b'H' as u16, b'A' as u16,
        b'N' as u16, b'N' as u16, b'E' as u16, b'L' as u16, 0,
    ];
    let name = UNICODE_STRING {
        Length: (12 * core::mem::size_of::<u16>()) as u16,
        MaximumLength: (12 * core::mem::size_of::<u16>()) as u16,
        Buffer: W_DEBUGCHANNEL.as_ptr() as PWSTR,
    };

    let mut value = UNICODE_STRING {
        Length: 0,
        MaximumLength: (100 * core::mem::size_of::<u16>()) as u16,
        Buffer: val_buffer.as_mut_ptr(),
    };

    let mut status = QueryEnvironmentVariable(&name, &mut value);

    if status == STATUS_VARIABLE_NOT_FOUND { return 1; }

    if status == STATUS_BUFFER_TOO_SMALL {
        let buf_sz = value.MaximumLength as SIZE_T;
        value.Buffer = ExAllocatePool(PagedPool, buf_sz) as PWSTR;
        if value.Buffer.is_null() { return 0; }
        status = QueryEnvironmentVariable(&name, &mut value);
    }

    if !NT_SUCCESS(status) {
        if value.Buffer != val_buffer.as_mut_ptr() { ExFreePool(value.Buffer as PVOID); }
        return 0;
    }

    let ret = DbgParseDebugChannels(ppi, &value);

    if value.Buffer != val_buffer.as_mut_ptr() { ExFreePool(value.Buffer as PVOID); }

    ret
}
