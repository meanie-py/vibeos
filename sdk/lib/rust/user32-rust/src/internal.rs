//! internal.rs — shared user32-internal substrate for the translated bodies.
//!
//! Three families, all typed against `win32k_sys::win32k::*` (so callers that
//! `use win32k_sys::win32k::*` pass compatible types — no cross-bindgen-crate
//! type collisions):
//!   1. ntdll Rtl*/Zw* exports (declared `extern "system"`, resolve via the
//!      ntdll importlib at link). NOT in win32k-sys's surface.
//!   2. TEB/PEB inlines (NtCurrentTeb/NtCurrentPeb/RtlGetProcessHeap) — these are
//!      FORCEINLINE macros, not exports; reproduced over the win32k-sys `_TEB`/`_PEB`.
//!   3. user32-internal C symbols that still live in the surviving C objects of
//!      user32.dll (cohab): the ValidateHwnd family + UserApiHook trio (FASTCALL)
//!      and the `guah`/`gfServerProcess` globals, plus `LoadUserApiHook` (a
//!      user_x.h static-inline, reproduced) and the common Win32 word macros.

use win32k_sys::win32k::*;

// ─── 1. ntdll Rtl*/Zw* exports (resolve via ntdll importlib) ──────────────────
extern "system" {
    pub fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    pub fn RtlFreeHeap(HeapHandle: HANDLE, Flags: ULONG, P: PVOID) -> BOOLEAN;
    pub fn RtlInitUnicodeString(DestinationString: PUNICODE_STRING, SourceString: PCWSTR);
    pub fn RtlInitAnsiString(DestinationString: PANSI_STRING, SourceString: PCSZ);
    pub fn RtlFreeUnicodeString(UnicodeString: PUNICODE_STRING);
    pub fn RtlFreeAnsiString(AnsiString: PANSI_STRING);
    pub fn RtlCreateUnicodeStringFromAsciiz(Destination: PUNICODE_STRING, Source: PCSZ) -> BOOLEAN;
    pub fn RtlAnsiStringToUnicodeString(
        DestinationString: PUNICODE_STRING,
        SourceString: PCANSI_STRING,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;
    pub fn RtlUnicodeStringToAnsiString(
        DestinationString: PANSI_STRING,
        SourceString: PCUNICODE_STRING,
        AllocateDestinationString: BOOLEAN,
    ) -> NTSTATUS;
    pub fn RtlMultiByteToUnicodeN(
        UnicodeString: PWCH,
        MaxBytesInUnicodeString: ULONG,
        BytesInUnicodeString: PULONG,
        MultiByteString: *const CHAR,
        BytesInMultiByteString: ULONG,
    ) -> NTSTATUS;
    pub fn RtlUnicodeToMultiByteN(
        MbString: PCHAR,
        MbSize: ULONG,
        ResultSize: PULONG,
        UnicodeString: PCWCH,
        UnicodeSize: ULONG,
    ) -> NTSTATUS;
    pub fn ZwYieldExecution() -> NTSTATUS;
    pub fn ZwCallbackReturn(Result: PVOID, ResultLength: ULONG, Status: NTSTATUS) -> NTSTATUS;
}

// ─── 2. TEB/PEB inlines (FORCEINLINE macros — not exports) ────────────────────

/// `NtCurrentTeb()` — per-thread TEB self pointer (NtTib.Self).
#[inline]
pub unsafe fn NtCurrentTeb() -> *mut _TEB {
    let teb: *mut _TEB;
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb,
        options(nostack, pure, readonly, preserves_flags));
    #[cfg(target_arch = "x86")]
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb,
        options(nostack, pure, readonly, preserves_flags));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("mov {}, x18", out(reg) teb,
        options(nostack, pure, nomem, preserves_flags));
    teb
}

/// `NtCurrentPeb()` — `NtCurrentTeb()->ProcessEnvironmentBlock`.
#[inline]
pub unsafe fn NtCurrentPeb() -> PPEB {
    (*NtCurrentTeb()).ProcessEnvironmentBlock
}

/// `RtlGetProcessHeap()` — `NtCurrentPeb()->ProcessHeap`.
#[inline]
pub unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

/// `RtlInitEmptyAnsiString()` — FORCEINLINE macro (rtlfuncs.h), not an export.
#[inline]
pub unsafe fn RtlInitEmptyAnsiString(
    AnsiString: PANSI_STRING,
    Buffer: PCHAR,
    BufferSize: USHORT,
) {
    (*AnsiString).Length = 0;
    (*AnsiString).MaximumLength = BufferSize;
    (*AnsiString).Buffer = Buffer;
}

/// `RtlInitEmptyUnicodeString()` — FORCEINLINE macro (rtlfuncs.h), not an export.
#[inline]
pub unsafe fn RtlInitEmptyUnicodeString(
    UnicodeString: PUNICODE_STRING,
    Buffer: PWCH,
    BufferSize: USHORT,
) {
    (*UnicodeString).Length = 0;
    (*UnicodeString).MaximumLength = BufferSize;
    (*UnicodeString).Buffer = Buffer;
}

// ─── 3. user32-internal cohab symbols (in the surviving C objects) ────────────
// FASTCALL → extern "fastcall" on i386 (`@Name@N`); matches user32p.h.
#[cfg(target_arch = "x86")]
extern "fastcall" {
    pub fn ValidateHwnd(hwnd: HWND) -> PWND;
    pub fn ValidateHwndOrDesk(hwnd: HWND) -> PWND;
    pub fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    pub fn BeginIfHookedUserApiHook() -> BOOL;
    pub fn EndUserApiHook() -> BOOL;
    pub fn IsInsideUserApiHook() -> BOOL;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    pub fn ValidateHwnd(hwnd: HWND) -> PWND;
    pub fn ValidateHwndOrDesk(hwnd: HWND) -> PWND;
    pub fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    pub fn BeginIfHookedUserApiHook() -> BOOL;
    pub fn EndUserApiHook() -> BOOL;
    pub fn IsInsideUserApiHook() -> BOOL;
}

extern "C" {
    pub static mut guah: USERAPIHOOK; // usrapihk.c
    pub static gfServerProcess: BOOLEAN; // dllmain.c
}

/// `LoadUserApiHook()` — user_x.h `static __inline`; no exported symbol, reproduced.
#[inline]
pub unsafe fn LoadUserApiHook() {
    if gfServerProcess == 0
        && IsInsideUserApiHook() == 0
        && !gpsi.is_null()
        && ((*gpsi).dwSRVIFlags & SRVINFO_APIHOOK) != 0
        && RtlIsThreadWithinLoaderCallout() == 0
    {
        NtUserCallNoParam(NOPARAM_ROUTINE_LOADUSERAPIHOOK as DWORD);
    }
}

extern "system" {
    pub fn RtlIsThreadWithinLoaderCallout() -> BOOLEAN; // ntdll
}


// ─── Common Win32 word/coordinate macros (winuser.h / windef.h) ───────────────
#[inline]
pub fn LOWORD(l: DWORD) -> WORD {
    (l & 0xffff) as WORD
}
#[inline]
pub fn HIWORD(l: DWORD) -> WORD {
    ((l >> 16) & 0xffff) as WORD
}
#[inline]
pub fn LOBYTE(w: WORD) -> BYTE {
    (w & 0xff) as BYTE
}
#[inline]
pub fn HIBYTE(w: WORD) -> BYTE {
    ((w >> 8) & 0xff) as BYTE
}
#[inline]
pub fn MAKEWORD(a: BYTE, b: BYTE) -> WORD {
    (a as WORD) | ((b as WORD) << 8)
}
#[inline]
pub fn MAKELONG(a: WORD, b: WORD) -> LONG {
    ((a as u32) | ((b as u32) << 16)) as LONG
}
#[inline]
pub fn MAKEWPARAM(l: WORD, h: WORD) -> WPARAM {
    MAKELONG(l, h) as u32 as WPARAM
}
#[inline]
pub fn MAKELPARAM(l: WORD, h: WORD) -> LPARAM {
    MAKELONG(l, h) as LPARAM
}
#[inline]
pub fn GET_X_LPARAM(lp: LPARAM) -> i32 {
    (lp & 0xffff) as i16 as i32
}
#[inline]
pub fn GET_Y_LPARAM(lp: LPARAM) -> i32 {
    ((lp >> 16) & 0xffff) as i16 as i32
}
