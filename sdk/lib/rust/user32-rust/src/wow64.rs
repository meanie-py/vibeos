//! WoW64: reading the 64-bit USER shared/desktop-heap structures from a 32-bit client.
//!
//! A WoW64 process runs a 32-bit user32 against a 64-bit win32k. Most of USER is
//! syscall-mediated and therefore marshalled by wow64win — but three things are
//! read *directly* out of shared memory, with no syscall at all:
//!
//!   1. the shared USER handle table (`gSharedInfo.aheList`) — what `ValidateHwnd`
//!      indexes to turn an HWND into a `PWND`,
//!   2. the desktop heap's `WND` / `CLS` objects, reached via `DesktopPtrToUser`,
//!   3. `DESKTOPINFO` (`pvDesktopBase`/`pvDesktopLimit`/`fsHooks`/`spwnd`).
//!
//! Those mappings are already placed below 4GB for WoW64 processes (win32k's
//! `W32Wow64ZeroBits`), so a 32-bit client can *address* them — it just cannot
//! *parse* them, because a 64-bit kernel laid them out with 8-byte pointers. The
//! symptom is M9: `ValidateHwnd` silently returns NULL for every HWND, so
//! `DispatchMessageA` drops WM_PAINT on the floor (field manual §9.58b).
//!
//! This module gives the client a normalizing view of each. `WND` and `CLS` get
//! *snapshots*: the 64-bit object is copied into a native-layout struct which is
//! handed back instead. That is sound because the client's desktop-heap mapping is
//! `PAGE_READONLY` (win32k `IntMapDesktopView`) — user32 never writes through a
//! `PWND`, so a copy cannot lose an update. Snapshots are cached per kernel object
//! so pointer identity stays stable across calls (user32 compares `PWND`s), and
//! refreshed on every lookup so reads see current state.
//!
//! Pointer-valued fields inside a snapshot (`spwnd*`, `pcls`, `strName.Buffer`, …)
//! keep the *kernel* value truncated to 32 bits, exactly as the native i386 path
//! stores them, so a caller's subsequent `DesktopPtrToUser`/`desktop_wnd_to_user`
//! on them works unchanged. The truncation is lossless (heap is below 4GB) and the
//! delta arithmetic wraps mod 2^32.

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use win32k_sys::win32k::*;

use crate::internal::{RtlAllocateHeap, RtlGetProcessHeap};
use crate::misc::dllmain::{gHandleEntries, gHandleTable, gSharedInfo};
use crate::ntwrapper::NtUserxGetDesktopMapping;

extern "C" {
    // ntoskrnl/ntdll cdecl variadic debug print.
    fn DbgPrint(Format: *const i8, ...) -> ULONG;
    // misc/misc.c: PTHREADINFO GetW32ThreadInfo(VOID) — plain cdecl, NOT stdcall
    // (matches windows/window.rs; an extern "system" decl here would be stdcall on
    // i386 and silently UB-fold the call).
    fn GetW32ThreadInfo() -> PTHREADINFO;
}

/// ntdef.h: `NT_SUCCESS(Status) := (Status >= 0)`.
#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

/// Snapshot kind tags (NOT the USER handle TYPE_* values — CLS has no handle type).
const SNAP_WND: UINT = 1;
const SNAP_CLS: UINT = 2;

/// `GetWin32ClientInfo()` = `NtCurrentTeb()->Win32ClientInfo` (ntuser.h macro).
#[inline]
pub unsafe fn get_client_info() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*crate::internal::NtCurrentTeb()).Win32ClientInfo) as PCLIENTINFO
}

/// `SharedPtrToUser()` (user_x.h) — the shared-heap delta, not the desktop one.
#[inline]
unsafe fn SharedPtrToUser(ptr: PVOID) -> PVOID {
    (ptr as ULONG_PTR).wrapping_sub(gSharedInfo.ulSharedDelta as ULONG_PTR) as PVOID
}

// ── the 64-bit shapes ────────────────────────────────────────────────────────
//
// Layout note: on i686-w64-mingw (MS ABI) a `u64` has 8-byte alignment inside a
// struct, exactly as on x86_64 — so declaring every pointer field as `u64` and
// letting the compiler lay the struct out reproduces the 64-bit layout verbatim.
// This is verified: every field of `Wnd64` lands at an identical offset on both
// targets, and the `const _` guards at the bottom of this file re-prove it against
// the *real* structs on every native 64-bit build.

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ThrDeskHead64 {
    pub h: u64,
    pub cLockObj: u32,
    pub Padding0: u32,
    pub pti: u64,
    pub rpdesk: u64,
    pub pSelf: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct LargeUnicodeString64 {
    pub Length: u32,
    /// `MaximumLength:31` + `bAnsi:1` — kept packed; callers use the accessors.
    pub MaxLenAndAnsi: u32,
    pub Buffer: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ListEntry64 {
    pub Flink: u64,
    pub Blink: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct InternalPos64 {
    pub NormalRect: RECTL,
    pub IconPos: POINTL,
    pub MaxPos: POINTL,
    pub flags: UINT,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Wnd64 {
    pub head: ThrDeskHead64,
    pub state: u32,
    pub state2: u32,
    pub ExStyle: u32,
    pub style: u32,
    pub hModule: u64,
    pub fnid: u32,
    pub Padding1: u32,
    pub spwndNext: u64,
    pub spwndPrev: u64,
    pub spwndParent: u64,
    pub spwndChild: u64,
    pub spwndOwner: u64,
    pub rcWindow: RECTL,
    pub rcClient: RECTL,
    pub lpfnWndProc: u64,
    pub pcls: u64,
    pub hrgnUpdate: u64,
    pub PropListHead: ListEntry64,
    pub PropListItems: u32,
    pub Padding2: u32,
    pub pSBInfo: u64,
    pub SystemMenu: u64,
    pub IDMenu: u64,
    pub hrgnClip: u64,
    pub hrgnNewFrame: u64,
    pub strName: LargeUnicodeString64,
    pub cbwndExtra: u32,
    pub Padding3: u32,
    pub spwndLastActive: u64,
    pub hImc: u64,
    pub dwUserData: u64,
    pub pActCtx: u64,
    pub spwndClipboardListener: u64,
    pub ExStyle2: u32,
    pub InternalPos: InternalPos64,
    /// `Unicode:1 | InternalPosInitialized:1 | HideFocus:1 | HideAccel:1`
    pub Bits: u32,
    pub pSBInfoex: u64,
    pub ThreadListEntry: ListEntry64,
    pub DialogPointer: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Cls64 {
    pub pclsNext: u64,
    pub atomClassName: u16,
    pub atomNVClassName: u16,
    pub fnid: u32,
    pub rpdeskParent: u64,
    pub pdce: u64,
    pub CSF_flags: u32,
    pub Padding1: u32,
    pub lpszClientAnsiMenuName: u64,
    pub lpszClientUnicodeMenuName: u64,
    pub spcpdFirst: u64,
    pub pclsBase: u64,
    pub pclsClone: u64,
    pub cWndReferenceCount: u32,
    pub style: u32,
    pub lpfnWndProc: u64,
    pub cbclsExtra: i32,
    pub cbwndExtra: i32,
    pub hModule: u64,
    pub spicn: u64,
    pub spcur: u64,
    pub hbrBackground: u64,
    pub lpszMenuName: u64,
    pub lpszAnsiClassName: u64,
    pub spicnSm: u64,
    /// `Unicode:1 | Global:1 | MenuNameIsString:1 | NotUsed:29`
    pub Bits: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct DesktopInfo64 {
    pub pvDesktopBase: u64,
    pub pvDesktopLimit: u64,
    pub spwnd: u64,
    pub fsHooks: u32,
    pub Padding0: u32,
    pub aphkStart: [ListEntry64; NB_HOOKS as usize],
    pub hTaskManWindow: u64,
    pub hProgmanWindow: u64,
    pub hShellWindow: u64,
    pub spwndShell: u64,
    pub spwndBkGnd: u64,
    pub ppiShellProcess: u64,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UserHandleEntry64 {
    pub ptr: u64,
    pub pi: u64,
    pub type_: u8,
    pub flags: u8,
    pub generation: u16,
    pub Padding0: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UserHandleTable64 {
    pub handles: u64,
    pub freelist: u64,
    pub nb_handles: i32,
    pub allocated_handles: i32,
}

// ── the tripwire ─────────────────────────────────────────────────────────────
//
// On a native 64-bit build these shapes must BE the real structs. Any drift in
// ntuser.h therefore breaks the amd64/arm64 build immediately, instead of
// silently corrupting WoW64 clients on arm64. Keep it that way.
#[cfg(not(target_arch = "x86"))]
mod layout_guard {
    use super::*;
    use core::mem::{offset_of, size_of};

    const _: () = assert!(size_of::<ThrDeskHead64>() == size_of::<THRDESKHEAD>());
    const _: () = assert!(size_of::<ListEntry64>() == size_of::<LIST_ENTRY>());

    const _: () = assert!(size_of::<Wnd64>() == size_of::<WND>());
    const _: () = assert!(offset_of!(Wnd64, state) == offset_of!(WND, state));
    const _: () = assert!(offset_of!(Wnd64, style) == offset_of!(WND, style));
    const _: () = assert!(offset_of!(Wnd64, fnid) == offset_of!(WND, fnid));
    const _: () = assert!(offset_of!(Wnd64, spwndParent) == offset_of!(WND, spwndParent));
    const _: () = assert!(offset_of!(Wnd64, rcWindow) == offset_of!(WND, rcWindow));
    const _: () = assert!(offset_of!(Wnd64, rcClient) == offset_of!(WND, rcClient));
    const _: () = assert!(offset_of!(Wnd64, lpfnWndProc) == offset_of!(WND, lpfnWndProc));
    const _: () = assert!(offset_of!(Wnd64, pcls) == offset_of!(WND, pcls));
    const _: () = assert!(offset_of!(Wnd64, strName) == offset_of!(WND, strName));
    const _: () = assert!(offset_of!(Wnd64, cbwndExtra) == offset_of!(WND, cbwndExtra));
    const _: () = assert!(offset_of!(Wnd64, ExStyle2) == offset_of!(WND, ExStyle2));
    const _: () = assert!(offset_of!(Wnd64, InternalPos) == offset_of!(WND, InternalPos));
    const _: () = assert!(offset_of!(Wnd64, pSBInfoex) == offset_of!(WND, pSBInfoex));
    const _: () = assert!(offset_of!(Wnd64, DialogPointer) == offset_of!(WND, DialogPointer));

    const _: () = assert!(size_of::<Cls64>() == size_of::<CLS>());
    const _: () = assert!(offset_of!(Cls64, fnid) == offset_of!(CLS, fnid));
    const _: () = assert!(offset_of!(Cls64, CSF_flags) == offset_of!(CLS, CSF_flags));
    const _: () = assert!(offset_of!(Cls64, style) == offset_of!(CLS, style));
    const _: () = assert!(offset_of!(Cls64, lpfnWndProc) == offset_of!(CLS, lpfnWndProc));
    const _: () = assert!(offset_of!(Cls64, cbclsExtra) == offset_of!(CLS, cbclsExtra));
    const _: () = assert!(offset_of!(Cls64, hModule) == offset_of!(CLS, hModule));
    const _: () = assert!(offset_of!(Cls64, spicnSm) == offset_of!(CLS, spicnSm));

    const _: () = assert!(offset_of!(DesktopInfo64, pvDesktopLimit) == offset_of!(DESKTOPINFO, pvDesktopLimit));
    const _: () = assert!(offset_of!(DesktopInfo64, spwnd) == offset_of!(DESKTOPINFO, spwnd));
    const _: () = assert!(offset_of!(DesktopInfo64, fsHooks) == offset_of!(DESKTOPINFO, fsHooks));
    const _: () = assert!(offset_of!(DesktopInfo64, hShellWindow) == offset_of!(DESKTOPINFO, hShellWindow));

    const _: () = assert!(size_of::<UserHandleEntry64>() == size_of::<USER_HANDLE_ENTRY>());
    const _: () = assert!(size_of::<UserHandleTable64>() == size_of::<USER_HANDLE_TABLE>());
    const _: () = assert!(offset_of!(UserHandleTable64, nb_handles) == offset_of!(USER_HANDLE_TABLE, nb_handles));
}

// ── detection ────────────────────────────────────────────────────────────────

/// `true` iff this 32-bit user32 is running as a WoW64 guest on a 64-bit kernel.
/// Compile-time `false` on the native 64-bit builds, so every branch on it folds
/// away there.
#[cfg(target_arch = "x86")]
pub static mut g_UserWow64: bool = false;

#[cfg(target_arch = "x86")]
#[inline]
pub unsafe fn IsWow64() -> bool {
    g_UserWow64
}

#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn IsWow64() -> bool {
    false
}

#[cfg(target_arch = "x86")]
extern "system" {
    fn NtQueryInformationProcess(
        ProcessHandle: HANDLE,
        ProcessInformationClass: u32,
        ProcessInformation: PVOID,
        ProcessInformationLength: ULONG,
        ReturnLength: PULONG,
    ) -> NTSTATUS;
}

/// Detect WoW64 once, at user32 connect time (mirrors gdi32's `GdiProcessSetup`).
#[cfg(target_arch = "x86")]
pub unsafe fn UserWow64Init() {
    const ProcessWow64Information: u32 = 26;
    let mut Wow64Info: ULONG_PTR = 0;

    let Status = NtQueryInformationProcess(
        -1isize as HANDLE,
        ProcessWow64Information,
        core::ptr::addr_of_mut!(Wow64Info) as PVOID,
        core::mem::size_of::<ULONG_PTR>() as ULONG,
        core::ptr::null_mut(),
    );

    if NT_SUCCESS(Status) && Wow64Info != 0 {
        g_UserWow64 = true;
    }

    DbgPrint(
        b"[user32] UserWow64Init: st=%lx info=%p -> g_UserWow64=%d gHandleTable=%p\n\0".as_ptr() as *const i8,
        Status,
        Wow64Info,
        g_UserWow64 as u32,
        gHandleTable,
    );
}

#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn UserWow64Init() {}

// ── snapshot cache ───────────────────────────────────────────────────────────

#[cfg(target_arch = "x86")]
mod snap {
    use super::*;

    pub(super) union SnapObj {
        pub wnd: WND,
        pub cls: CLS,
    }

    pub(super) struct Snap {
        pub next: *mut Snap,
        pub key: PVOID,
        pub type_: UINT,
        pub obj: SnapObj,
    }

    pub(super) static mut SNAP_LIST: *mut Snap = core::ptr::null_mut();

    /// Find (or create) the snapshot for a given kernel object. Returns null only
    /// on allocation failure.
    ///
    /// Single-threaded discipline: every caller runs on a GUI thread inside a
    /// user32 entry point, and the list is only ever *appended* to (entries are
    /// never freed or relinked), so a concurrent reader can never observe a
    /// torn/dangling `next`. A duplicate entry from a simultaneous first-touch of
    /// the same object on two threads is harmless — both are valid snapshots of
    /// the same object; the loser is simply never revisited.
    pub(super) unsafe fn find_or_add(key: PVOID, type_: UINT) -> *mut Snap {
        let mut s = SNAP_LIST;
        while !s.is_null() {
            if (*s).key == key && (*s).type_ == type_ {
                return s;
            }
            s = (*s).next;
        }

        let s = RtlAllocateHeap(RtlGetProcessHeap(), 8 /* HEAP_ZERO_MEMORY */, core::mem::size_of::<Snap>() as SIZE_T)
            as *mut Snap;
        if s.is_null() {
            return core::ptr::null_mut();
        }

        (*s).key = key;
        (*s).type_ = type_;
        (*s).next = SNAP_LIST;
        SNAP_LIST = s;
        s
    }
}

// ── normalizers ──────────────────────────────────────────────────────────────

/// Normalize the 64-bit `WND` at client address `p64` into a cached native `WND`.
#[cfg(target_arch = "x86")]
pub unsafe fn normalize_wnd(p64: PVOID) -> PWND {
    if p64.is_null() {
        return core::ptr::null_mut();
    }

    let s = snap::find_or_add(p64, SNAP_WND);
    if s.is_null() {
        return core::ptr::null_mut();
    }

    let src = &*(p64 as *const Wnd64);
    let d = core::ptr::addr_of_mut!((*s).obj.wnd);

    // `head` is a bindgen-flattened THRDESKHEAD: HEAD/THROBJHEAD collapse into
    // padding, so h/cLockObj/pti are reached by casting (same as windows/window.rs).
    (*(d as *mut _HEAD)).h = src.head.h as usize as HANDLE;
    (*(d as *mut _HEAD)).cLockObj = src.head.cLockObj;
    (*(d as *mut _THROBJHEAD)).pti = src.head.pti as usize as *mut _THREADINFO;
    (*d).head.rpdesk = src.head.rpdesk as usize as *mut _DESKTOP;
    (*d).head.pSelf = src.head.pSelf as usize as PVOID;

    (*d).state = src.state;
    (*d).state2 = src.state2;
    (*d).ExStyle = src.ExStyle;
    (*d).style = src.style;
    (*d).hModule = src.hModule as usize as HINSTANCE;
    (*d).fnid = src.fnid;

    (*d).spwndNext = src.spwndNext as usize as PWND;
    (*d).spwndPrev = src.spwndPrev as usize as PWND;
    (*d).spwndParent = src.spwndParent as usize as PWND;
    (*d).spwndChild = src.spwndChild as usize as PWND;
    (*d).spwndOwner = src.spwndOwner as usize as PWND;

    (*d).rcWindow = src.rcWindow;
    (*d).rcClient = src.rcClient;

    (*d).lpfnWndProc = core::mem::transmute::<usize, WNDPROC>(src.lpfnWndProc as usize);
    (*d).pcls = src.pcls as usize as PCLS;
    (*d).hrgnUpdate = src.hrgnUpdate as usize as HRGN;

    (*d).PropListHead.Flink = src.PropListHead.Flink as usize as _;
    (*d).PropListHead.Blink = src.PropListHead.Blink as usize as _;
    (*d).PropListItems = src.PropListItems;

    (*d).pSBInfo = src.pSBInfo as usize as _;
    (*d).SystemMenu = src.SystemMenu as usize as HMENU;
    (*d).IDMenu = src.IDMenu as usize as UINT_PTR;
    (*d).hrgnClip = src.hrgnClip as usize as HRGN;
    (*d).hrgnNewFrame = src.hrgnNewFrame as usize as HRGN;

    (*d).strName.Length = src.strName.Length;
    (*d).strName.set_MaximumLength(src.strName.MaxLenAndAnsi & 0x7FFF_FFFF);
    (*d).strName.set_bAnsi(src.strName.MaxLenAndAnsi >> 31);
    (*d).strName.Buffer = src.strName.Buffer as usize as PWSTR;

    (*d).cbwndExtra = src.cbwndExtra;
    (*d).spwndLastActive = src.spwndLastActive as usize as PWND;
    (*d).hImc = src.hImc as usize as HIMC;
    (*d).dwUserData = src.dwUserData as usize as LONG_PTR;
    (*d).pActCtx = src.pActCtx as usize as PVOID;
    (*d).spwndClipboardListener = src.spwndClipboardListener as usize as PWND;
    (*d).ExStyle2 = src.ExStyle2;

    (*d).InternalPos.NormalRect = src.InternalPos.NormalRect;
    (*d).InternalPos.IconPos = src.InternalPos.IconPos;
    (*d).InternalPos.MaxPos = src.InternalPos.MaxPos;
    (*d).InternalPos.flags = src.InternalPos.flags;

    (*d).set_Unicode(src.Bits & 1);
    (*d).set_InternalPosInitialized((src.Bits >> 1) & 1);
    (*d).set_HideFocus((src.Bits >> 2) & 1);
    (*d).set_HideAccel((src.Bits >> 3) & 1);

    (*d).pSBInfoex = src.pSBInfoex as usize as _;
    (*d).DialogPointer = src.DialogPointer as usize as PVOID;

    d
}

/// Normalize the 64-bit `CLS` at client address `p64` into a cached native `CLS`.
#[cfg(target_arch = "x86")]
pub unsafe fn normalize_cls(p64: PVOID) -> PCLS {
    if p64.is_null() {
        return core::ptr::null_mut();
    }

    let s = snap::find_or_add(p64, SNAP_CLS);
    if s.is_null() {
        return core::ptr::null_mut();
    }

    let src = &*(p64 as *const Cls64);
    let d = core::ptr::addr_of_mut!((*s).obj.cls);

    (*d).pclsNext = src.pclsNext as usize as PCLS;
    (*d).atomClassName = src.atomClassName;
    (*d).atomNVClassName = src.atomNVClassName;
    (*d).fnid = src.fnid;
    (*d).rpdeskParent = src.rpdeskParent as usize as _;
    (*d).pdce = src.pdce as usize as PVOID;
    (*d).CSF_flags = src.CSF_flags;
    (*d).lpszClientAnsiMenuName = src.lpszClientAnsiMenuName as usize as PSTR;
    (*d).lpszClientUnicodeMenuName = src.lpszClientUnicodeMenuName as usize as PWSTR;
    (*d).spcpdFirst = src.spcpdFirst as usize as _;
    (*d).pclsBase = src.pclsBase as usize as PCLS;
    (*d).pclsClone = src.pclsClone as usize as PCLS;
    (*d).cWndReferenceCount = src.cWndReferenceCount;
    (*d).style = src.style;
    (*d).lpfnWndProc = core::mem::transmute::<usize, WNDPROC>(src.lpfnWndProc as usize);
    (*d).cbclsExtra = src.cbclsExtra;
    (*d).cbwndExtra = src.cbwndExtra;
    (*d).hModule = src.hModule as usize as HINSTANCE;
    (*d).spicn = src.spicn as usize as _;
    (*d).spcur = src.spcur as usize as _;
    (*d).hbrBackground = src.hbrBackground as usize as HBRUSH;
    (*d).lpszMenuName = src.lpszMenuName as usize as PWSTR;
    (*d).lpszAnsiClassName = src.lpszAnsiClassName as usize as PSTR;
    (*d).spicnSm = src.spicnSm as usize as _;

    (*d).set_Unicode(src.Bits & 1);
    (*d).set_Global((src.Bits >> 1) & 1);
    (*d).set_MenuNameIsString((src.Bits >> 2) & 1);

    d
}

// ── the public API used by the translated user32 files ───────────────────────

/// Base of a window's extra bytes — the `cbwndExtra` region, where
/// `Get/SetWindowLongPtr(hwnd, >= 0)` reads and writes.
///
/// Native: the extra bytes sit immediately after the `WND` in the desktop heap,
/// so `pWnd + sizeof(WND)` is the base.
///
/// WoW64: `pWnd` is NOT the window. It is a *normalized 32-bit copy* of the real
/// 64-bit `WND` (see `normalize_wnd`), living inside a heap-allocated `Snap` —
/// and nothing follows it. The window's real extra bytes follow the *64-bit* WND
/// out in the desktop heap. Indexing off the copy reads whatever the heap put
/// after the Snap: that is how user32's EDIT control read a garbage `EDITSTATE*`
/// and faulted at address 0x2A (M11). Recover the object we snapped and index
/// off that instead. `size_of::<Wnd64>()` is the true 64-bit WND size — the
/// `layout_guard` above proves it against the real `WND` on 64-bit builds.
#[cfg(target_arch = "x86")]
pub unsafe fn wnd_extra_base(pWnd: PWND) -> *mut u8 {
    if IsWow64() {
        let s = (pWnd as *mut u8).sub(core::mem::offset_of!(snap::Snap, obj)) as *mut snap::Snap;
        // Prove it really is a Snap before trusting the container_of.
        if core::ptr::addr_of_mut!((*s).obj.wnd) as PWND == pWnd {
            return ((*s).key as *mut u8).add(core::mem::size_of::<Wnd64>());
        }
        // Not a snapshot — we would be reading a wild address. Say so, loudly.
        wow64_unsupported("window extra bytes on a non-snapshot WND");
        return core::ptr::null_mut();
    }
    (pWnd as *mut u8).add(core::mem::size_of::<WND>())
}

#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn wnd_extra_base(pWnd: PWND) -> *mut u8 {
    (pWnd as *mut u8).add(core::mem::size_of::<WND>())
}

/// The class extra bytes (`cbClsExtra`), addressed the same way and broken the
/// same way as the window ones above.
///
/// Native: they sit immediately after the `CLS` in the desktop heap.
///
/// WoW64: `pCls` is a normalized 32-bit COPY of the 64-bit `CLS` inside a
/// heap-allocated `Snap`, and nothing follows it -- so `pCls + 1` indexes into
/// whatever the process heap put after the Snap. Wine's user32:class subtest
/// caught this: `SetClassLongW(i*4, i+1)` then `GetClassLongW(i*4)` disagreed
/// on all eight class words, because the SET went through a syscall to the
/// real object and the GET read a copy that does not contain them. The window
/// half of this was found by the EDIT control faulting at 0x2A (M11); the
/// class half sat behind it until an oracle looked.
#[cfg(target_arch = "x86")]
pub unsafe fn cls_extra_base(pCls: PCLS) -> *mut u8 {
    if IsWow64() {
        let s = (pCls as *mut u8).sub(core::mem::offset_of!(snap::Snap, obj)) as *mut snap::Snap;
        // Prove it really is a Snap before trusting the container_of.
        if core::ptr::addr_of_mut!((*s).obj.cls) as PCLS == pCls {
            return ((*s).key as *mut u8).add(core::mem::size_of::<Cls64>());
        }
        wow64_unsupported("class extra bytes on a non-snapshot CLS");
        return core::ptr::null_mut();
    }
    (pCls as *mut u8).add(core::mem::size_of::<CLS>())
}

#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn cls_extra_base(pCls: PCLS) -> *mut u8 {
    (pCls as *mut u8).add(core::mem::size_of::<CLS>())
}

/// `DesktopPtrToUser` — the one true copy. Every per-file duplicate in the
/// translated user32 sources delegates here, so the WoW64 handling lives in one
/// place. `Ptr` is a kernel desktop-heap address as stored in the shared
/// structures; the result is the client-side address of the same object.
pub unsafe fn desktop_ptr_to_user(ptr: PVOID) -> PVOID {
    GetW32ThreadInfo();
    let pci: PCLIENTINFO = crate::wow64::get_client_info();
    let pdi = (*pci).pDeskInfo;

    if pdi.is_null() {
        return NtUserxGetDesktopMapping(ptr);
    }

    let (base, limit) = if IsWow64() {
        let d = &*(pdi as *const DesktopInfo64);
        (d.pvDesktopBase as usize, d.pvDesktopLimit as usize)
    } else {
        ((*pdi).pvDesktopBase as usize, (*pdi).pvDesktopLimit as usize)
    };

    if (ptr as usize) >= base && (ptr as usize) < limit {
        ((ptr as usize).wrapping_sub((*pci).ulClientDelta as usize)) as PVOID
    } else {
        // Slow path: object lives on another desktop — ask win32k.
        NtUserxGetDesktopMapping(ptr)
    }
}

/// Typed `DesktopPtrToUser` for a `WND`. Use this — NOT a raw
/// `desktop_ptr_to_user` + cast — anywhere the result is dereferenced.
#[inline]
pub unsafe fn desktop_wnd_to_user(kernel_wnd: PVOID) -> PWND {
    if kernel_wnd.is_null() {
        return core::ptr::null_mut();
    }
    let p = desktop_ptr_to_user(kernel_wnd);
    #[cfg(target_arch = "x86")]
    if IsWow64() {
        return normalize_wnd(p);
    }
    p as PWND
}

/// Typed `DesktopPtrToUser` for a `CLS`.
#[inline]
pub unsafe fn desktop_cls_to_user(kernel_cls: PVOID) -> PCLS {
    if kernel_cls.is_null() {
        return core::ptr::null_mut();
    }
    let p = desktop_ptr_to_user(kernel_cls);
    #[cfg(target_arch = "x86")]
    if IsWow64() {
        return normalize_cls(p);
    }
    p as PCLS
}

/// Read entry `index` of the shared USER handle table, normalized into `out`
/// (a caller-owned buffer — see GetUser32Handle). Returns `false` if the index is
/// out of range.
pub unsafe fn read_handle_entry(index: i32, out: *mut USER_HANDLE_ENTRY) -> bool {
    if IsWow64() {
        let table = gHandleTable as *const UserHandleTable64;
        if table.is_null() || index < 0 || index >= (*table).nb_handles {
            return false;
        }
        let entries = SharedPtrToUser((*table).handles as usize as PVOID) as *const UserHandleEntry64;
        if entries.is_null() {
            return false;
        }
        let e = &*entries.offset(index as isize);
        (*out).ptr = e.ptr as usize as PVOID;
        (*out).__bindgen_anon_1.pi = e.pi as usize as PVOID;
        (*out).type_ = e.type_;
        (*out).flags = e.flags;
        (*out).generation = e.generation;
        return true;
    }

    if gHandleTable.is_null() || index < 0 || index >= (*gHandleTable).nb_handles {
        return false;
    }
    *out = *crate::misc::dllmain::gHandleEntries.offset(index as isize);
    true
}

/// `DESKTOPINFO::spwnd`, WoW64-aware.
#[inline]
pub unsafe fn desktop_info_spwnd(pdi: PDESKTOPINFO) -> PWND {
    if pdi.is_null() {
        return core::ptr::null_mut();
    }
    if IsWow64() {
        return (*(pdi as *const DesktopInfo64)).spwnd as usize as PWND;
    }
    (*pdi).spwnd
}

/// `DESKTOPINFO::fsHooks`, WoW64-aware.
#[inline]
pub unsafe fn desktop_info_fshooks(pdi: PDESKTOPINFO) -> DWORD {
    if pdi.is_null() {
        return 0;
    }
    if IsWow64() {
        return (*(pdi as *const DesktopInfo64)).fsHooks;
    }
    (*pdi).fsHooks
}

/// `DESKTOPINFO::hShellWindow`, WoW64-aware.
#[inline]
pub unsafe fn desktop_info_shell_window(pdi: PDESKTOPINFO) -> HWND {
    if pdi.is_null() {
        return core::ptr::null_mut();
    }
    if IsWow64() {
        return (*(pdi as *const DesktopInfo64)).hShellWindow as usize as HWND;
    }
    (*pdi).hShellWindow
}

/// `MENU`/`ITEM`, `HOOK` and `SBINFO` are desktop-heap objects too, and they carry
/// pointers — so a WoW64 client misreads them exactly like it used to misread
/// `WND`. They have no normalizing view yet (nothing on the M9 path reaches them).
/// Rather than let a 32-bit client silently walk a 64-bit `MENU`, say so — once per
/// type. If you hit this, the fix is another `*64` shape above plus a normalizer,
/// not a workaround at the call site.
#[cfg(target_arch = "x86")]
pub unsafe fn wow64_unsupported(type_: &'static str) {
    use core::sync::atomic::{AtomicU32, Ordering};
    static WARNED: AtomicU32 = AtomicU32::new(0);

    let bit = match type_.as_bytes()[0] {
        b'M' => 1u32,
        b'H' => 2,
        b'S' => 4,
        _ => 8,
    };
    if WARNED.fetch_or(bit, Ordering::Relaxed) & bit != 0 {
        return;
    }

    DbgPrint(
        b"[user32] WOW64: %.*s is read from the desktop heap with a 64-bit layout and has no 32-bit view yet - results will be garbage. Add a *64 shape + normalizer in user32-rust/src/wow64.rs\n\0"
            .as_ptr() as *const i8,
        type_.len() as ULONG,
        type_.as_ptr(),
    );
}

#[cfg(not(target_arch = "x86"))]
#[inline]
pub unsafe fn wow64_unsupported(_type_: &'static str) {}
