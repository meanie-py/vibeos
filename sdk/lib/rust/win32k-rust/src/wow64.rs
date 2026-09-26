//! WoW64 support for win32k's user-mode-visible thread state.
//!
//! A 32-bit guest's user32 reads its CLIENTINFO out of the **32-bit** TEB
//! (`GetWin32ClientInfo()` = `NtCurrentTeb()->Win32ClientInfo`, and under WoW64
//! `NtCurrentTeb()` is the TEB32). win32k, being 64-bit, only ever writes the
//! 64-bit TEB's copy — so before this module a WoW64 guest saw an all-zero
//! CLIENTINFO: `ulClientDelta == 0` and `pDeskInfo == NULL`, which made
//! `DesktopPtrToUser()` a no-op and handed user32 a raw (truncated) *kernel*
//! desktop-heap pointer. That is the AV behind M9 (field manual §9.58b).
//!
//! So for every WoW64 thread we project the 64-bit CLIENTINFO into a
//! `CLIENTINFO32` written into TEB32. The projection is deliberately partial:
//!
//! * `CallbackWnd` is left **zero**. It is a client-side cache of the last
//!   callback's `PWND`, and a 64-bit `WND*` is unusable to the guest. Leaving it
//!   zero makes user32's `ValidateHwnd` fast-path miss and fall through to
//!   `ValidateHandle`, which routes through the guest's own WoW64 WND-snapshot
//!   layer (`user32/misc/wow64.c`). Correct, just not cached.
//! * `msgDbcsCB` / `Win32ClientInfo3` are zero (DBCS + private, unused here).
//!
//! Everything else the 32-bit user32 reads — `pDeskInfo`, `ulClientDelta`,
//! `dwTIFlags`, `fsHooks`, key-state caches, `hKL`, `ppi` — is projected.

#![allow(non_snake_case)]

use ntkrnl_sys::ddk::PEPROCESS;
// In kernel mode NtCurrentTeb() is an inline that reads the PCR — the ddk::
// declaration is an extern with no definition to link against.
use ntkrnl_sys::inline::ke::NtCurrentTeb;
use ntkrnl_sys::ps::PsGetCurrentProcess;
use ntkrnl_sys::rtl::RtlZeroMemory;
use win32k_sys::win32k::{PCLIENTINFO, PTHREADINFO};

/// Exact i386 `CLIENTINFO` layout (win32ss/include/ntuser.h) — 248 bytes, i.e.
/// the full `TEB32::Win32ClientInfo[62]`. Field order/offsets must track the C
/// struct; the `C_ASSERT` below is the guard.
#[cfg(not(target_arch = "x86"))]
#[repr(C)]
#[derive(Default)]
pub(crate) struct ClientInfo32 {
    pub CI_flags: u32,
    pub cSpins: u32,
    pub dwExpWinVer: u32,
    pub dwCompatFlags: u32,
    pub dwCompatFlags2: u32,
    pub dwTIFlags: u32,
    pub pDeskInfo: u32,
    pub ulClientDelta: u32,
    pub phkCurrent: u32,
    pub fsHooks: u32,
    /* CALLBACKWND32 { HWND hWnd; WND *pWnd; PVOID pActCtx } — kept zero, see above. */
    pub CallbackWnd_hWnd: u32,
    pub CallbackWnd_pWnd: u32,
    pub CallbackWnd_pActCtx: u32,
    pub dwHookCurrent: u32,
    pub cInDDEMLCallback: i32,
    pub pClientThreadInfo: u32,
    pub dwHookData: u32,
    pub dwKeyCache: u32,
    pub afKeyState: [u8; 8],
    pub dwAsyncKeyCache: u32,
    pub afAsyncKeyState: [u8; 8],
    pub afAsyncKeyStateRecentDow: [u8; 8],
    pub hKL: u32,
    pub CodePage: u16,
    pub achDbcsCF: [u8; 2],
    /* MSG32 msgDbcsCB: hwnd/message/wParam/lParam/time + POINT pt — 28 bytes. */
    pub msgDbcsCB: [u32; 7],
    pub lpdwRegisteredClasses: u32,
    pub Win32ClientInfo3: [u32; 26],
    pub ppi: u32,
}

#[cfg(not(target_arch = "x86"))]
const _: () = assert!(core::mem::size_of::<ClientInfo32>() == 62 * 4);

/// TEB32 sits at `WOW64_TEB32_OFFSET` from TEB64 in the combined region;
/// `ntkrnl-sys` owns the constant and `MmCreateTeb` is the allocator.
#[cfg(not(target_arch = "x86"))]
use ntkrnl_sys::wow64::WOW64_TEB32_OFFSET;

/// Offset of `Win32ClientInfo` within TEB32. Taken from the bindgen `_TEB32`, so
/// it cannot drift.
#[cfg(not(target_arch = "x86"))]
#[inline]
const fn teb32_clientinfo_offset() -> usize {
    core::mem::offset_of!(ntkrnl_sys::ddk::_TEB32, Win32ClientInfo)
}

/// The i386 TEB puts `Win32ClientInfo` at 0x6CC, and the 32-bit user32 reaches
/// its fields by that ABSOLUTE displacement -- `ValidateHandleNoErr` compiles
/// `pci->pDeskInfo` to a flat `mov 0x6e4(%eax)` off `fs:[0x18]`, with no
/// indirection this projection could redirect. So a bindgen `_TEB32` that put
/// the array anywhere else would not fail to build and would not fault here:
/// it would silently publish a correct CLIENTINFO32 at the wrong address, and
/// the guest would read whichever neighbouring field happened to line up.
/// Assert the number the guest has already hardcoded.
#[cfg(not(target_arch = "x86"))]
const _: () = assert!(teb32_clientinfo_offset() == 0x6CC);

/// `true` when the current process is a WoW64 (32-bit guest) process.
#[cfg(not(target_arch = "x86"))]
#[inline]
pub(crate) unsafe fn IsWow64Process() -> bool {
    let p: PEPROCESS = PsGetCurrentProcess() as _;
    !p.is_null() && !(*p).Wow64Process.is_null()
}

/// Project a thread's 64-bit CLIENTINFO into its TEB32 as a 32-bit CLIENTINFO.
///
/// No-op for native processes and for threads with no TEB (system threads).
/// Call this anywhere win32k mutates a field of CLIENTINFO that the 32-bit
/// user32 reads — today: thread init, desktop assignment (which is what sets
/// `pDeskInfo`/`ulClientDelta`), and the callback in/out points.
#[cfg(not(target_arch = "x86"))]
pub(crate) unsafe fn Wow64PublishClientInfo(pti: PTHREADINFO) {
    use ntkrnl_sys::ddk::_TEB32;

    if pti.is_null() || !IsWow64Process() {
        return;
    }

    /* Always called on the thread being published (init / desktop switch /
     * callback in-out), so the current TEB is that thread's TEB64. */
    let pci: PCLIENTINFO = (*pti).pClientInfo;
    let teb64 = NtCurrentTeb() as *mut u8;
    if pci.is_null() || teb64.is_null() {
        return;
    }

    let teb32 = teb64.add(WOW64_TEB32_OFFSET) as *mut _TEB32;
    let ci32 = (teb32 as *mut u8).add(teb32_clientinfo_offset()) as *mut ClientInfo32;

    /* TEB.Win32ThreadInfo — the 32-bit user32 reads this via GetW32ThreadInfo()
     * to test window ownership (IntCallMessageProc: wnd_pti(Wnd) == it). win32k
     * writes the full kernel THREADINFO into the 64-bit TEB (main.rs), but the
     * guest reads the 32-bit TEB, where it stayed 0 → every pump-dispatched
     * message (WM_KEYDOWN/mouse) was dropped ERROR_MESSAGE_SYNC_ONLY (M14 doom
     * input). The WND snapshot's head.pti is the SAME low-32 truncation of the
     * kernel THREADINFO VA (normalize_wnd), so the two sides compare equal. */
    (*teb32).Win32ThreadInfo = pti as usize as u32;

    RtlZeroMemory(ci32 as *mut core::ffi::c_void, core::mem::size_of::<ClientInfo32>());

    (*ci32).CI_flags = (*pci).CI_flags as u32;
    (*ci32).cSpins = (*pci).cSpins as u32;
    (*ci32).dwExpWinVer = (*pci).dwExpWinVer;
    (*ci32).dwCompatFlags = (*pci).dwCompatFlags;
    (*ci32).dwCompatFlags2 = (*pci).dwCompatFlags2;
    (*ci32).dwTIFlags = (*pci).dwTIFlags;

    /* pDeskInfo and pClientThreadInfo are already CLIENT-side pointers (win32k
     * subtracted ulClientDelta when it stored them), and the desktop heap is
     * mapped below 4GB for WoW64 processes (W32Wow64ZeroBits), so the truncation
     * is lossless. */
    (*ci32).pDeskInfo = (*pci).pDeskInfo as usize as u32;
    (*ci32).pClientThreadInfo = (*pci).pClientThreadInfo as usize as u32;

    /* ulClientDelta is a difference of two addresses that are congruent mod 2^32
     * in the guest's view, so the low 32 bits are the right delta for 32-bit
     * pointer arithmetic (which wraps mod 2^32 anyway). */
    (*ci32).ulClientDelta = (*pci).ulClientDelta as usize as u32;

    (*ci32).fsHooks = (*pci).fsHooks;
    (*ci32).dwHookCurrent = (*pci).dwHookCurrent;
    (*ci32).cInDDEMLCallback = (*pci).cInDDEMLCallback;
    (*ci32).dwHookData = (*pci).dwHookData as u32;

    (*ci32).dwKeyCache = (*pci).dwKeyCache;
    (*ci32).afKeyState = (*pci).afKeyState;
    (*ci32).dwAsyncKeyCache = (*pci).dwAsyncKeyCache;
    (*ci32).afAsyncKeyState = (*pci).afAsyncKeyState;
    (*ci32).afAsyncKeyStateRecentDow = (*pci).afAsyncKeyStateRecentDow;

    (*ci32).hKL = (*pci).hKL as usize as u32;
    (*ci32).CodePage = (*pci).CodePage;
    (*ci32).achDbcsCF = (*pci).achDbcsCF;

    (*ci32).lpdwRegisteredClasses = (*pci).lpdwRegisteredClasses as usize as u32;
    (*ci32).ppi = (*pci).ppi as usize as u32;
}

/// The current thread's TEB32, where a WoW64 guest's gdi32 keeps its GDI batch.
///
/// The batch has to be flushed from HERE, not from TEB64: gdi32 increments
/// `TEB32->GdiBatchCount`, and when the buffer is full it calls `NtGdiFlush()`
/// and then writes at `Buffer + Offset` WITHOUT re-checking, trusting the
/// kernel to have zeroed `Offset`. Before the trap path learned to read TEB32's
/// counter, nothing did: `Offset` climbed through `CLIENTINFO.pDeskInfo` at
/// 0x508 and user32 dereferenced a batch record byte as a pointer (field manual
/// §9.41, second occurrence).
#[cfg(not(target_arch = "x86"))]
pub(crate) unsafe fn Wow64CurrentTeb32() -> *mut ntkrnl_sys::ddk::_TEB32 {
    let teb64 = NtCurrentTeb() as *mut u8;
    if teb64.is_null() {
        return core::ptr::null_mut();
    }
    teb64.add(WOW64_TEB32_OFFSET) as *mut ntkrnl_sys::ddk::_TEB32
}

/// i386 host has no WoW64 layer.
#[cfg(target_arch = "x86")]
#[inline]
pub(crate) unsafe fn Wow64PublishClientInfo(_pti: PTHREADINFO) {}

/// A 32-bit host has no WoW64 layer, so nothing is ever a guest here and the
/// GDI batch path keeps its original behaviour unchanged.
#[cfg(target_arch = "x86")]
#[inline]
pub(crate) unsafe fn IsWow64Process() -> bool {
    false
}
