#![allow(non_snake_case, non_upper_case_globals, unused_unsafe)]
use win32k_sys::win32k::*;
use ntkrnl_sys::ddk::{KeAttachProcess, KeDetachProcess, PsGetCurrentThreadWin32Thread, PsGetThreadId, PsGetThreadProcessId, ZwQueryDefaultLocale};
use ntkrnl_sys::inline::ke::NtCurrentTeb;
use ntkrnl_sys::base::NT_SUCCESS;
use ntkrnl_sys::constants::{STATUS_UNSUCCESSFUL, STATUS_BUFFER_TOO_SMALL};
use ntkrnl_sys::base::STATUS_SUCCESS;
use ntkrnl_sys::ddk::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::interlocked::{InterlockedIncrement, InterlockedDecrement};
use core::ffi::c_void;

// gfIMEShowStatus — DEFINED here (was in ime.c, now flipped to Rust). Other
// win32k objects import it. `BOOL gfIMEShowStatus = (BOOL)-1;`
#[no_mangle]
pub static mut gfIMEShowStatus: BOOL = -1i32;

extern "C" {
    fn GetW32ThreadInfo() -> PTHREADINFO;
    // _wcsnicmp — CRT.
    fn _wcsnicmp(s1: *const u16, s2: *const u16, n: usize) -> i32;
    // RtlInitLargeUnicodeString — same crate (rtlstr.rs) exported symbol.
}

// RtlAllocateHeap / RtlFreeHeap — ntdll/rtl exports, NTAPI stdcall (`@12`).
extern "system" {
    fn RtlAllocateHeap(HeapHandle: PVOID, Flags: ULONG, Size: SIZE_T) -> PVOID;
    fn RtlFreeHeap(HeapHandle: PVOID, Flags: ULONG, BaseAddress: PVOID) -> BOOLEAN;
}

// WndSetOwner — static __inline (window.h): ReplaceWndPtr(&pwnd->spwndOwner, ..).
#[inline(always)]
unsafe fn WndSetOwner(pwnd: PWND, pwndOwner: PWND) {
    if !pwndOwner.is_null() { UserReferenceObject(pwndOwner as PVOID); }
    if !(*pwnd).spwndOwner.is_null() { UserDereferenceObject((*pwnd).spwndOwner as PVOID); }
    (*pwnd).spwndOwner = pwndOwner;
}

// UserRefObjectCo / UserDerefObjectCo — static __inline (object.h). Matches hook.rs.
#[inline(always)]
unsafe fn UserRefObjectCo<T>(obj: *mut T, UserReferenceEntry: *mut USER_REFERENCE_ENTRY) {
    let obj = obj as PVOID;
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    (*UserReferenceEntry).obj = obj;
    UserReferenceObject(obj);
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let entry: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*UserReferenceEntry).Entry);
    (*entry).Next = (*list_head).Next;
    (*list_head).Next = entry;
}
#[inline(always)]
unsafe fn UserDerefObjectCo<T>(obj: *mut T) {
    let obj = obj as PVOID;
    let W32Thread: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let list_head: *mut SINGLE_LIST_ENTRY = core::ptr::addr_of_mut!((*W32Thread).ReferencesList);
    let reference_entry: *mut SINGLE_LIST_ENTRY = (*list_head).Next;
    if !reference_entry.is_null() {
        (*list_head).Next = (*reference_entry).Next;
    }
    UserDereferenceObject(obj);
}

// STDCALL win32k exports: SafeProbeForRead/Write (`_SafeProbe*@12`),
// co_HOOK_CallHooks (`_co_HOOK_CallHooks@16`).
extern "system" {
    fn SafeProbeForRead(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn SafeProbeForWrite(Address: PVOID, Length: SIZE_T, Alignment: ULONG) -> NTSTATUS;
    fn co_HOOK_CallHooks(HookId: core::ffi::c_int, Code: core::ffi::c_int, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

// FASTCALL win32k exports: ValidateHwndNoErr (`@..@4`), co_IntSendMessage (`@..@16`).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    fn co_IntSendMessage(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn ValidateHwndNoErr(hwnd: HWND) -> PWND;
    fn co_IntSendMessage(hWnd: HWND, uMsg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

// ---- macro / FORCEINLINE reimplementations (no exported symbol) ----
// LOWORD(l) = (WORD)((DWORD_PTR)(l) & 0xffff)
#[inline(always)]
fn LOWORD(l: u32) -> u16 { (l & 0xffff) as u16 }
// PRIMARYLANGID(l) = (WORD)(l) & 0x3ff
#[inline(always)]
fn PRIMARYLANGID(l: u16) -> u16 { l & 0x3ff }
// IS_IME_HKL(hKL) = (((ULONG_PTR)hKL) & 0xF0000000) == IME_MASK (0xE0000000)
#[inline(always)]
unsafe fn IS_IME_HKL(hkl: HKL) -> BOOL {
    (((hkl as usize as u32) & 0xF0000000) == 0xE0000000) as BOOL
}
// IS_IMM_MODE() = gpsi && (gpsi->dwSRVIFlags & SRVINFO_IMM32)
#[inline(always)]
unsafe fn IS_IMM_MODE() -> BOOL {
    (!gpsi.is_null() && ((*gpsi).dwSRVIFlags & SRVINFO_IMM32) != 0) as BOOL
}
// ISITHOOKED(HookId) = ((PTHREADINFO)PsGetCurrentThreadWin32Thread())->fsHooks & HOOKID_TO_FLAG(HookId)
// HOOKID_TO_FLAG(HookId) = 1 << (HookId + 1)
#[inline(always)]
unsafe fn ISITHOOKED(hook: core::ffi::c_int) -> BOOL {
    let pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    if pti.is_null() { return FALSE as BOOL; }
    (((*pti).fsHooks & (1u32 << (hook + 1))) != 0) as BOOL
}
// SET_KEY_DOWN(ks, vk, down) — GET_KS_BYTE(vk)=vk*2/8, GET_KS_DOWN_BIT(vk)=1<<((vk%4)*2)
#[inline(always)]
unsafe fn SET_KEY_DOWN(ks: LPBYTE, vk: UINT, down: BOOL) {
    let p = ks.add((vk * 2 / 8) as usize);
    let bit = 1u8 << ((vk % 4) * 2);
    *p = if down != 0 { *p | bit } else { *p & !bit };
}
// SET_KEY_LOCKED(ks, vk, locked) — GET_KS_LOCK_BIT(vk)=1<<((vk%4)*2+1)
#[inline(always)]
unsafe fn SET_KEY_LOCKED(ks: LPBYTE, vk: UINT, locked: BOOL) {
    let p = ks.add((vk * 2 / 8) as usize);
    let bit = 1u8 << ((vk % 4) * 2 + 1);
    *p = if locked != 0 { *p | bit } else { *p & !bit };
}
// DesktopHeapFree(Desktop, lpMem) — desktop.h static __inline.
#[inline(always)]
unsafe fn DesktopHeapFree(Desktop: PDESKTOP, lpMem: PVOID) -> u8 {
    RtlFreeHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, lpMem) as u8
}
// IntDereferenceThreadInfo(pti) — win32.h macro.
#[inline(always)]
unsafe fn IntDereferenceThreadInfo(pti: PTHREADINFO) {
    if InterlockedDecrement((pti as *mut u8).add(core::mem::size_of::<usize>()) as *mut LONG) == 0 {
        UserDeleteW32Thread(pti);
    }
}

// ---- file-local constants / macro reimplementations ----
const USERTAG_IMEHOTKEY: u32 = u32::from_le_bytes(*b"Usih"); // 'hisU'
const HWND_TOP: HWND = core::ptr::null_mut();
// LANGID_KOREAN = MAKELANGID(LANG_KOREAN, SUBLANG_KOREAN)
const LANGID_KOREAN: LANGID = ((SUBLANG_KOREAN << 10) | LANG_KOREAN) as LANGID;
// WINVER for win32k (windef.h default 0x0502).
const WINVER: u32 = 0x0502;
// HWND_TERMINATOR = (HWND)UlongToHandle(1)  (window.h)
const HWND_TERMINATOR: HWND = 1usize as HWND;

// s_uKeyUpVKey — file-static tracking the last key-up VK for IME hotkey processing.
static mut s_uKeyUpVKey: UINT = 0;

// DBE virtual keys (jpnvkeys.h) — not in bindgen.
const VK_DBE_KATAKANA: UINT = 0xF1;
const VK_DBE_HIRAGANA: UINT = 0xF2;
const VK_DBE_ROMAN: UINT = 0xF5;
const VK_DBE_NOROMAN: UINT = 0xF6;
const VK_DBE_ENTERWORDREGISTERMODE: UINT = 0xF7;
const VK_DBE_ENTERCONFIGMODE: UINT = 0xF8;
const VK_DBE_CODEINPUT: UINT = 0xFA;
const VK_DBE_NOCODEINPUT: UINT = 0xFB;

// UlongToHandle(ul) = (HANDLE)(ULONG_PTR)(ul)
#[inline(always)]
fn UlongToHandle(ul: u32) -> HANDLE { ul as usize as HANDLE }
// UlongToPtr(ul) = (PVOID)(ULONG_PTR)(ul)
#[inline(always)]
fn UlongToPtr(ul: u32) -> *mut c_void { ul as usize as *mut c_void }
// HandleToUlong(h) = (ULONG)(ULONG_PTR)(h)
#[inline(always)]
fn HandleToUlong(h: HANDLE) -> ULONG { h as usize as ULONG }
// LOBYTE(w) = w & 0xff
#[inline(always)]
fn LOBYTE(w: WPARAM) -> u32 { (w & 0xff) as u32 }
// MAKELONG(low, high) = (WORD)low | ((WORD)high << 16)
#[inline(always)]
fn MAKELONG(low: u32, high: u32) -> LONG { ((low & 0xffff) | ((high & 0xffff) << 16)) as LONG }
// LANGIDFROMLCID(lcid) = LOWORD(lcid)
#[inline(always)]
fn LANGIDFROMLCID(lcid: LCID) -> u32 { (lcid as u32) & 0xffff }
// ASSERT_REFS_CO — debug-only assertion, no-op in release.
#[inline(always)]
unsafe fn ASSERT_REFS_CO(_Window: PWND) {}

// RtlZeroMemory — compiler-intrinsic memset (no symbol).
#[inline(always)]
unsafe fn RtlZeroMemory(dst: PVOID, len: SIZE_T) {
    core::ptr::write_bytes(dst as *mut u8, 0, len as usize);
}

// UserHMGetHandle(obj) = obj->head.h (object.h macro). Matches input.rs.
#[inline(always)]
unsafe fn UserHMGetHandle<T>(obj: *mut T) -> HANDLE {
    (*(obj as *const _HEAD)).h
}

// head_pti(obj) — bindgen flattened THRDESKHEAD (no `pti` field). pti sits at
// 2*sizeof(usize) (= 8 on i386, 16 on amd64/arm64). Matches input.rs/event.rs.
#[inline(always)]
unsafe fn head_pti<T>(obj: *const T) -> PTHREADINFO {
    *((obj as *const u8).add(2 * core::mem::size_of::<usize>()) as *const PTHREADINFO)
}
// set_head_pti(head, pti) — write THRDESKHEAD.pti (same flattened offset).
#[inline(always)]
unsafe fn set_head_pti<T>(obj: *mut T, pti: PTHREADINFO) {
    *((obj as *mut u8).add(2 * core::mem::size_of::<usize>()) as *mut PTHREADINFO) = pti;
}
// ppi_w32(ppi) — PROCESSINFO embeds a W32PROCESS prefix (offset 0); bindgen
// padding-collapses it. Reach peProcess / W32PF_flags / UserHandleCount through it.
#[inline(always)]
unsafe fn ppi_w32(ppi: PPROCESSINFO) -> *mut W32PROCESS { ppi as *mut W32PROCESS }
// pti_ethread(pti) — W32THREAD.pEThread is the first field of THREADINFO (offset 0).
#[inline(always)]
unsafe fn pti_ethread(pti: PTHREADINFO) -> PETHREAD { *(pti as *const PETHREAD) }
// IntReferenceThreadInfo — win32.h macro (InterlockedIncrement on RefCount).
// THREADINFO embeds W32THREAD: RefCount (LONG) at size_of::<usize>() (after pEThread).
#[inline(always)]
unsafe fn IntReferenceThreadInfo(pti: PTHREADINFO) {
    InterlockedIncrement((pti as *mut u8).add(core::mem::size_of::<usize>()) as *mut LONG);
}
// DesktopHeapAlloc — desktop.h static __inline.
#[inline(always)]
unsafe fn DesktopHeapAlloc(Desktop: PDESKTOP, Bytes: SIZE_T) -> PVOID {
    RtlAllocateHeap((*Desktop).pheapDesktop as PVOID, HEAP_NO_SERIALIZE, Bytes)
}

// IS_KEY_DOWN(ks, vk): ((ks)[vk*2/8] & (1 << ((vk%4)*2))) ? TRUE : FALSE
#[inline(always)]
unsafe fn IS_KEY_DOWN(ks: *const u8, vk: u32) -> BOOL {
    let byte = *ks.add((vk * 2 / 8) as usize);
    if (byte & (1u8 << ((vk % 4) * 2))) != 0 { TRUE as BOOL } else { FALSE as BOOL }
}
// IS_WND_CHILD(pWnd) = pWnd->style & WS_CHILD
#[inline(always)]
unsafe fn IS_WND_CHILD(pwnd: PWND) -> u32 { (*pwnd).style & WS_CHILD }
// IS_WND_MENU(pWnd) = pWnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_MENU]
#[inline(always)]
unsafe fn IS_WND_MENU(pwnd: PWND) -> BOOL {
    ((*(*pwnd).pcls).atomClassName == (*gpsi).atomSysClass[ICLS_MENU as usize]) as BOOL
}
// IS_WND_IMELIKE(pWnd) = (pWnd->pcls->style & CS_IME) ||
//                        (pWnd->pcls->atomClassName == gpsi->atomSysClass[ICLS_IME])
#[inline(always)]
unsafe fn IS_WND_IMELIKE(pwnd: PWND) -> BOOL {
    (((*(*pwnd).pcls).style & CS_IME) != 0
        || (*(*pwnd).pcls).atomClassName == (*gpsi).atomSysClass[ICLS_IME as usize]) as BOOL
}

// ---- file-scope preamble (translated) ----
pub const INVALID_THREAD_ID: ULONG = !0u32;

pub const INVALID_HOTKEY: UINT = !0u32;

pub static mut ghIMC: HIMC = core::ptr::null_mut();
pub static mut gfImeOpen: BOOL = -1i32;
pub static mut gdwImeConversion: DWORD = !0u32;

#[repr(C)]
pub struct tagIMEHOTKEY {
    pub pNext: *mut tagIMEHOTKEY,
    pub dwHotKeyId: DWORD,
    pub uVirtualKey: UINT,
    pub uModifiers: UINT,
    pub hKL: HKL,
}
pub type IMEHOTKEY = tagIMEHOTKEY;
pub type PIMEHOTKEY = *mut tagIMEHOTKEY;

pub static mut gpImeHotKeyList: PIMEHOTKEY = core::ptr::null_mut();
pub static mut glcidSystem: LCID = 0;

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetImeUIFromWnd(pWnd: PWND) -> PIMEUI {
    let pImeWnd: PIMEWND = pWnd as PIMEWND;
    (*pImeWnd).pimeui
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetImeUIFromWnd(pWnd: PWND) -> PIMEUI {
    let pImeWnd: PIMEWND = pWnd as PIMEWND;
    (*pImeWnd).pimeui
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetImeCompatFlags(mut pti: PTHREADINFO) -> DWORD {
    if pti.is_null() {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    }

    (*(*pti).ppi).dwImeCompatFlags
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetImeCompatFlags(mut pti: PTHREADINFO) -> DWORD {
    if pti.is_null() {
        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    }

    (*(*pti).ppi).dwImeCompatFlags
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetImeHotKeyLanguageScore(hKL: HKL, HotKeyLangId: LANGID) -> u32 {
    IntGetImeHotKeyLanguageScoreImpl(hKL, HotKeyLangId)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetImeHotKeyLanguageScore(hKL: HKL, HotKeyLangId: LANGID) -> u32 {
    IntGetImeHotKeyLanguageScoreImpl(hKL, HotKeyLangId)
}

unsafe fn IntGetImeHotKeyLanguageScoreImpl(hKL: HKL, HotKeyLangId: LANGID) -> u32 {
    let lcid: LCID;

    if HotKeyLangId == 0 || HotKeyLangId as u32 == ((hKL as usize as u32) & 0xFFFF) {
        return 3;
    }

    // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
    // C (win32ss/user/ntuser/ime.c, IntGetImeHotKeyLanguageScore):
    //   _SEH2_TRY {
    //       lcid = NtCurrentTeb()->CurrentLocale;
    //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
    //       ERR(...);
    //       lcid = MAKELCID(LANGID_NEUTRAL, SORT_DEFAULT);
    //   } _SEH2_END;
    //
    // Shape: SENTINEL-VALUE SWALLOW, not a status. A faulting TEB read does not
    // abort the function — it substitutes MAKELCID(LANGID_NEUTRAL, SORT_DEFAULT)
    // and falls straight through into the same comparisons below. No probe in
    // the C (NtCurrentTeb() is not a user pointer to validate), so none is added
    // here either — only the read itself must survive a fault, via
    // safe_copy_raw's fault-tolerant copy rather than a bare dereference.
    let mut captured_lcid: LCID = 0;
    lcid = match ntkrnl_sys::probe::safe_copy_raw(
        core::ptr::addr_of_mut!(captured_lcid) as *mut core::ffi::c_void,
        core::ptr::addr_of!((*(NtCurrentTeb() as *mut _TEB)).CurrentLocale) as *const core::ffi::c_void,
        core::mem::size_of::<LCID>(),
    ) {
        Ok(()) => captured_lcid,
        Err(_) => {
            // MAKELCID(LANGID_NEUTRAL, SORT_DEFAULT): LANGID_NEUTRAL == 0,
            // SORT_DEFAULT == 0, so MAKELCID(lgid, srtid) == (srtid << 16) | lgid
            // == 0. No MAKELCID helper exists in this crate; the C macro's
            // literal result for these two constants is 0.
            0
        }
    };

    if HotKeyLangId as u32 == LANGIDFROMLCID(lcid) {
        return 2;
    }

    if glcidSystem == 0 {
        ZwQueryDefaultLocale(0u8, core::ptr::addr_of_mut!(glcidSystem));
    }

    if HotKeyLangId as u32 == LANGIDFROMLCID(glcidSystem) {
        return 1;
    }

    0
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetActiveKeyboardLayout() -> HKL {
    IntGetActiveKeyboardLayout_impl()
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetActiveKeyboardLayout() -> HKL {
    IntGetActiveKeyboardLayout_impl()
}

unsafe fn IntGetActiveKeyboardLayout_impl() -> HKL {
    let pti: PTHREADINFO;

    if !gpqForeground.is_null() && !(*gpqForeground).spwndActive.is_null() {
        pti = head_pti((*gpqForeground).spwndActive);
        if !pti.is_null() && !(*pti).KeyboardLayout.is_null() {
            return (*(*pti).KeyboardLayout).hkl;
        }
    }

    UserGetKeyboardLayout(0)
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetImeHotKeyLangId(dwHotKeyId: DWORD) -> LANGID {
    IntGetImeHotKeyLangIdImpl(dwHotKeyId)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetImeHotKeyLangId(dwHotKeyId: DWORD) -> LANGID {
    IntGetImeHotKeyLangIdImpl(dwHotKeyId)
}

unsafe fn IntGetImeHotKeyLangIdImpl(dwHotKeyId: DWORD) -> LANGID {
    const IME_CHOTKEY: DWORD = 0x10;
    const IME_XHOTKEY: DWORD = 0x90;

    let s_array: [LANGID; 9] = [
        0xFFFFu16 as LANGID,
        ((SUBLANG_CHINESE_SIMPLIFIED << 10) | LANG_CHINESE) as LANGID,
        ((SUBLANG_CHINESE_SIMPLIFIED << 10) | LANG_CHINESE) as LANGID,
        ((SUBLANG_DEFAULT << 10) | LANG_JAPANESE) as LANGID,
        ((SUBLANG_DEFAULT << 10) | LANG_JAPANESE) as LANGID,
        ((SUBLANG_KOREAN << 10) | LANG_KOREAN) as LANGID,
        ((SUBLANG_KOREAN << 10) | LANG_KOREAN) as LANGID,
        ((SUBLANG_CHINESE_TRADITIONAL << 10) | LANG_CHINESE) as LANGID,
        ((SUBLANG_CHINESE_TRADITIONAL << 10) | LANG_CHINESE) as LANGID,
    ];

    if IME_CHOTKEY <= dwHotKeyId && dwHotKeyId < IME_XHOTKEY {
        return s_array[((dwHotKeyId & 0xF0) >> 4) as usize];
    }
    ((SUBLANG_NEUTRAL << 10) | LANG_NEUTRAL) as LANGID
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntAddImeHotKey(
    ppList: *mut PIMEHOTKEY,
    pHotKey: PIMEHOTKEY,
) {
    IntAddImeHotKeyImpl(ppList, pHotKey)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntAddImeHotKey(
    ppList: *mut PIMEHOTKEY,
    pHotKey: PIMEHOTKEY,
) {
    IntAddImeHotKeyImpl(ppList, pHotKey)
}

#[inline]
unsafe fn IntAddImeHotKeyImpl(
    ppList: *mut PIMEHOTKEY,
    pHotKey: PIMEHOTKEY,
) {
    let mut pNode: PIMEHOTKEY;

    if (*ppList).is_null() {
        *ppList = pHotKey;
        return;
    }

    pNode = *ppList;
    while !pNode.is_null() {
        if (*pNode).pNext.is_null() {
            (*pNode).pNext = pHotKey;
            return;
        }
        pNode = (*pNode).pNext;
    }
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetImeHotKeyById(
    pList: PIMEHOTKEY,
    dwHotKeyId: DWORD,
) -> PIMEHOTKEY {
    let mut pNode = pList;
    while !pNode.is_null() {
        if (*pNode).dwHotKeyId == dwHotKeyId {
            return pNode;
        }
        pNode = (*pNode).pNext;
    }
    core::ptr::null_mut()
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetImeHotKeyById(
    pList: PIMEHOTKEY,
    dwHotKeyId: DWORD,
) -> PIMEHOTKEY {
    let mut pNode = pList;
    while !pNode.is_null() {
        if (*pNode).dwHotKeyId == dwHotKeyId {
            return pNode;
        }
        pNode = (*pNode).pNext;
    }
    core::ptr::null_mut()
}

unsafe extern "system" fn IntGetImeHotKeyByKeyAndLang(
    pList: PIMEHOTKEY,
    uModKeys: UINT,
    uLeftRight: UINT,
    uVirtualKey: UINT,
    TargetLangId: LANGID,
) -> PIMEHOTKEY {
    const MOD_KEYS: UINT = (MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_WIN) as UINT;
    const MOD_LEFT_RIGHT: UINT = (MOD_LEFT | MOD_RIGHT) as UINT;

    let mut pNode: PIMEHOTKEY = pList;
    while !pNode.is_null() {
        loop {
            if (*pNode).uVirtualKey != uVirtualKey {
                break;
            }

            let LangID: LANGID = IntGetImeHotKeyLangId((*pNode).dwHotKeyId);
            if LangID != TargetLangId && LangID != 0 {
                break;
            }

            let uModifiers: UINT = (*pNode).uModifiers;
            if uModifiers & (MOD_IGNORE_ALL_MODIFIER as UINT) != 0 {
                return pNode;
            }

            if (uModifiers & MOD_KEYS) != uModKeys {
                break;
            }

            if (uModifiers & uLeftRight) != 0 || (uModifiers & MOD_LEFT_RIGHT) == uLeftRight {
                return pNode;
            }

            break;
        }
        pNode = (*pNode).pNext;
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntDeleteImeHotKey(
    ppList: *mut PIMEHOTKEY,
    pHotKey: PIMEHOTKEY,
) {
    IntDeleteImeHotKeyImpl(ppList, pHotKey)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntDeleteImeHotKey(
    ppList: *mut PIMEHOTKEY,
    pHotKey: PIMEHOTKEY,
) {
    IntDeleteImeHotKeyImpl(ppList, pHotKey)
}

unsafe fn IntDeleteImeHotKeyImpl(
    ppList: *mut PIMEHOTKEY,
    pHotKey: PIMEHOTKEY,
) {
    let mut pNode: PIMEHOTKEY;

    if *ppList == pHotKey {
        *ppList = (*pHotKey).pNext;
        ExFreePoolWithTag(pHotKey as *mut _, USERTAG_IMEHOTKEY);
        return;
    }

    pNode = *ppList;
    while !pNode.is_null() {
        if (*pNode).pNext == pHotKey {
            (*pNode).pNext = (*pHotKey).pNext;
            ExFreePoolWithTag(pHotKey as *mut _, USERTAG_IMEHOTKEY);
            return;
        }
        pNode = (*pNode).pNext;
    }
}

unsafe fn IntGetImeHotKeyByKey(
    pList: PIMEHOTKEY,
    uModKeys: UINT,
    uLeftRight: UINT,
    uVirtualKey: UINT,
) -> PIMEHOTKEY {
    let mut pNode: PIMEHOTKEY;
    let mut ret: PIMEHOTKEY = core::ptr::null_mut();
    let pti: PTHREADINFO = GetW32ThreadInfo();
    let mut LangId: LANGID;
    let hKL: HKL = IntGetActiveKeyboardLayout();
    let fKorean: BOOL = ((PRIMARYLANGID(LOWORD(hKL as usize as u32)) as u32) == LANG_KOREAN as u32) as BOOL;
    let mut nScore: UINT;
    let mut nMaxScore: UINT = 0;

    const MOD_KEYS: UINT = MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_WIN;
    const MOD_LEFT_RIGHT: UINT = MOD_LEFT | MOD_RIGHT;

    pNode = pList;
    while !pNode.is_null() {
        'continue_block: {
            if (*pNode).uVirtualKey != uVirtualKey {
                break 'continue_block;
            }

            if ((*pNode).uModifiers & MOD_IGNORE_ALL_MODIFIER) != 0 {
                // ignore all modifiers
            } else if ((*pNode).uModifiers & MOD_KEYS) != uModKeys {
                break 'continue_block;
            } else if ((*pNode).uModifiers & uLeftRight) != 0
                || ((*pNode).uModifiers & MOD_LEFT_RIGHT) == uLeftRight
            {
                // ok
            } else {
                break 'continue_block;
            }

            LangId = IntGetImeHotKeyLangId((*pNode).dwHotKeyId);
            nScore = IntGetImeHotKeyLanguageScore(hKL, LangId);
            if nScore >= 3 {
                return pNode;
            }

            if fKorean != 0 {
                break 'continue_block;
            }

            if nScore == 0 {
                if (*pNode).dwHotKeyId == IME_CHOTKEY_IME_NONIME_TOGGLE
                    || (*pNode).dwHotKeyId == IME_THOTKEY_IME_NONIME_TOGGLE
                {
                    if LOWORD((*pti).hklPrev as usize as u32) as u32 == LangId as u32 {
                        return pNode;
                    }
                }
            }

            if nMaxScore < nScore {
                nMaxScore = nScore;
                ret = pNode;
            }
        }
        pNode = (*pNode).pNext;
    }

    ret
}

unsafe fn IntCheckImeHotKey(
    MessageQueue: *const USER_MESSAGE_QUEUE,
    uVirtualKey: UINT,
    lParam: LPARAM,
) -> PIMEHOTKEY {
    let pHotKey: PIMEHOTKEY;
    let mut uModifiers: UINT;
    let bKeyUp: BOOL = (lParam & 0x80000000u32 as LPARAM) as BOOL;
    let KeyState: *const BYTE = (*MessageQueue).afKeyState.as_ptr();

    if bKeyUp != 0 {
        if s_uKeyUpVKey != uVirtualKey {
            s_uKeyUpVKey = 0;
            return core::ptr::null_mut();
        }

        s_uKeyUpVKey = 0;
    }

    uModifiers = 0;
    if IS_KEY_DOWN(KeyState, VK_LSHIFT) != 0   { uModifiers |= MOD_SHIFT | MOD_LEFT; }
    if IS_KEY_DOWN(KeyState, VK_RSHIFT) != 0   { uModifiers |= MOD_SHIFT | MOD_RIGHT; }
    if IS_KEY_DOWN(KeyState, VK_LCONTROL) != 0 { uModifiers |= MOD_CONTROL | MOD_LEFT; }
    if IS_KEY_DOWN(KeyState, VK_RCONTROL) != 0 { uModifiers |= MOD_CONTROL | MOD_RIGHT; }
    if IS_KEY_DOWN(KeyState, VK_LMENU) != 0    { uModifiers |= MOD_ALT | MOD_LEFT; }
    if IS_KEY_DOWN(KeyState, VK_RMENU) != 0    { uModifiers |= MOD_ALT | MOD_RIGHT; }

    pHotKey = IntGetImeHotKeyByKey(
        gpImeHotKeyList,
        uModifiers & (MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_WIN),
        uModifiers & (MOD_LEFT | MOD_RIGHT),
        uVirtualKey,
    );
    if !pHotKey.is_null() {
        if bKeyUp != 0 {
            if (*pHotKey).uModifiers & MOD_ON_KEYUP != 0 {
                return pHotKey;
            }
        } else {
            if (*pHotKey).uModifiers & MOD_ON_KEYUP != 0 {
                s_uKeyUpVKey = uVirtualKey;
            } else {
                return pHotKey;
            }
        }
    }

    core::ptr::null_mut()
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntFreeImeHotKeys() {
    let mut pNode: PIMEHOTKEY = gpImeHotKeyList;
    while !pNode.is_null() {
        let pNext = (*pNode).pNext;
        ExFreePoolWithTag(pNode as *mut _, USERTAG_IMEHOTKEY);
        pNode = pNext;
    }
    gpImeHotKeyList = core::ptr::null_mut();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntFreeImeHotKeys() {
    let mut pNode: PIMEHOTKEY = gpImeHotKeyList;
    while !pNode.is_null() {
        let pNext = (*pNode).pNext;
        ExFreePoolWithTag(pNode as *mut _, USERTAG_IMEHOTKEY);
        pNode = pNext;
    }
    gpImeHotKeyList = core::ptr::null_mut();
}

unsafe extern "system" fn IntSetImeHotKey(
    dwHotKeyId: DWORD,
    uModifiers: UINT,
    uVirtualKey: UINT,
    hKL: HKL,
    dwAction: DWORD,
) -> BOOL {
    const MOD_KEYS: UINT = (MOD_CONTROL | MOD_SHIFT | MOD_ALT | MOD_WIN) as UINT;
    const MOD_LEFT_RIGHT: UINT = (MOD_LEFT | MOD_RIGHT) as UINT;

    let mut pNode: PIMEHOTKEY;
    let LangId: LANGID;

    match dwAction {
        SETIMEHOTKEY_DELETE => {
            pNode = IntGetImeHotKeyById(gpImeHotKeyList, dwHotKeyId);
            if pNode.is_null() {
                return FALSE as BOOL;
            }

            IntDeleteImeHotKey(core::ptr::addr_of_mut!(gpImeHotKeyList), pNode);
            TRUE as BOOL
        }

        SETIMEHOTKEY_ADD => {
            if (uVirtualKey & 0xFFFF) as u16 == VK_PACKET as u16 {
                return FALSE as BOOL;
            }

            LangId = IntGetImeHotKeyLangId(dwHotKeyId);
            if LangId == LANGID_KOREAN {
                return FALSE as BOOL;
            }

            pNode = IntGetImeHotKeyByKeyAndLang(
                gpImeHotKeyList,
                uModifiers & MOD_KEYS,
                uModifiers & MOD_LEFT_RIGHT,
                uVirtualKey,
                LangId,
            );
            if pNode.is_null() {
                pNode = IntGetImeHotKeyById(gpImeHotKeyList, dwHotKeyId);
            }

            if !pNode.is_null() {
                (*pNode).uModifiers = uModifiers;
                (*pNode).uVirtualKey = uVirtualKey;
                (*pNode).hKL = hKL;
                return TRUE as BOOL;
            }

            pNode = ExAllocatePoolWithTag(
                PagedPool,
                core::mem::size_of::<IMEHOTKEY>() as _,
                USERTAG_IMEHOTKEY,
            ) as PIMEHOTKEY;
            if pNode.is_null() {
                return FALSE as BOOL;
            }

            (*pNode).pNext = core::ptr::null_mut();
            (*pNode).dwHotKeyId = dwHotKeyId;
            (*pNode).uModifiers = uModifiers;
            (*pNode).uVirtualKey = uVirtualKey;
            (*pNode).hKL = hKL;
            IntAddImeHotKey(core::ptr::addr_of_mut!(gpImeHotKeyList), pNode);
            TRUE as BOOL
        }

        SETIMEHOTKEY_INITIALIZE => {
            IntFreeImeHotKeys();
            TRUE as BOOL
        }

        _ => FALSE as BOOL,
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetImeHotKey(
    dwHotKeyId: DWORD,
    lpuModifiers: PUINT,
    lpuVirtualKey: PUINT,
    lphKL: LPHKL,
) -> BOOL {
    let mut pNode: PIMEHOTKEY = core::ptr::null_mut();

    UserEnterExclusive();

    'block: {
        let mut Status = SafeProbeForWrite(
            lpuModifiers as *mut core::ffi::c_void,
            core::mem::size_of::<UINT>() as _,
            1,
        );
        if NT_SUCCESS(Status) {
            Status = SafeProbeForWrite(
                lpuVirtualKey as *mut core::ffi::c_void,
                core::mem::size_of::<UINT>() as _,
                1,
            );
        }
        if NT_SUCCESS(Status) && !lphKL.is_null() {
            Status = SafeProbeForWrite(
                lphKL as *mut core::ffi::c_void,
                core::mem::size_of::<HKL>() as _,
                1,
            );
        }
        if !NT_SUCCESS(Status) {
            break 'block;
        }

        pNode = IntGetImeHotKeyById(gpImeHotKeyList, dwHotKeyId);
        if pNode.is_null() {
            break 'block;
        }

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
        // C (win32ss/user/ntuser/ime.c, NtUserGetImeHotKey):
        //   _SEH2_TRY {
        //       *lpuModifiers = pNode->uModifiers;
        //       *lpuVirtualKey = pNode->uVirtualKey;
        //       if (lphKL) *lphKL = pNode->hKL;
        //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
        //       ERR(...);
        //       pNode = NULL;
        //   } _SEH2_END;
        //
        // Shape: SENTINEL-VALUE SWALLOW. A fault on ANY of the three writes
        // aborts the _SEH2_TRY block immediately (no partial-write fall-through
        // inside the C: an exception unwinds straight to the handler) and sets
        // pNode = NULL, which is what the return value `!!pNode` below reports
        // to the caller as FALSE. The addresses were already range/alignment
        // probed above via SafeProbeForWrite (matching the C's earlier
        // SafeProbeForWrite calls) — this only has to survive a page going away
        // between the probe and the write, via safe_copy_raw's fault-tolerant
        // copy rather than a bare write_volatile.
        let uModifiers = (*pNode).uModifiers;
        let uVirtualKey = (*pNode).uVirtualKey;
        let hKL = (*pNode).hKL;
        'seh: {
            if ntkrnl_sys::probe::safe_copy_raw(
                lpuModifiers as *mut core::ffi::c_void,
                core::ptr::addr_of!(uModifiers) as *const core::ffi::c_void,
                core::mem::size_of::<UINT>(),
            )
            .is_err()
            {
                pNode = core::ptr::null_mut();
                break 'seh;
            }
            if ntkrnl_sys::probe::safe_copy_raw(
                lpuVirtualKey as *mut core::ffi::c_void,
                core::ptr::addr_of!(uVirtualKey) as *const core::ffi::c_void,
                core::mem::size_of::<UINT>(),
            )
            .is_err()
            {
                pNode = core::ptr::null_mut();
                break 'seh;
            }
            if !lphKL.is_null()
                && ntkrnl_sys::probe::safe_copy_raw(
                    lphKL as *mut core::ffi::c_void,
                    core::ptr::addr_of!(hKL) as *const core::ffi::c_void,
                    core::mem::size_of::<HKL>(),
                )
                .is_err()
            {
                pNode = core::ptr::null_mut();
            }
        }
    }

    UserLeave();
    (!pNode.is_null()) as BOOL
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetImeHotKey(
    dwHotKeyId: DWORD,
    uModifiers: UINT,
    uVirtualKey: UINT,
    hKL: HKL,
    dwAction: DWORD,
) -> BOOL {
    let ret: BOOL;
    UserEnterExclusive();
    ret = IntSetImeHotKey(dwHotKeyId, uModifiers, uVirtualKey, hKL, dwAction);
    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCheckImeHotKey(
    uVirtualKey: UINT,
    lParam: LPARAM,
) -> DWORD {
    let pNode: PIMEHOTKEY;
    let mut ret: DWORD = INVALID_HOTKEY as DWORD;

    UserEnterExclusive();

    'quit: {
        if gpqForeground.is_null() || IS_IMM_MODE() == 0 {
            break 'quit;
        }

        pNode = IntCheckImeHotKey(gpqForeground, uVirtualKey, lParam);
        if !pNode.is_null() {
            ret = (*pNode).dwHotKeyId;
        }
    }

    UserLeave();
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetTopLevelWindow(mut pwnd: PWND) -> PWND {
    if pwnd.is_null() {
        return core::ptr::null_mut();
    }

    while (*pwnd).style & WS_CHILD != 0 {
        pwnd = (*pwnd).spwndParent;
    }

    pwnd
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetTopLevelWindow(mut pwnd: PWND) -> PWND {
    if pwnd.is_null() {
        return core::ptr::null_mut();
    }

    while (*pwnd).style & WS_CHILD != 0 {
        pwnd = (*pwnd).spwndParent;
    }

    pwnd
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntAssociateInputContext(pWnd: PWND, pImc: PIMC) -> HIMC {
    let hOldImc = (*pWnd).hImc;
    (*pWnd).hImc = if !pImc.is_null() {
        UserHMGetHandle(pImc as _) as _
    } else {
        core::ptr::null_mut()
    };
    hOldImc
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntAssociateInputContext(pWnd: PWND, pImc: PIMC) -> HIMC {
    let hOldImc = (*pWnd).hImc;
    (*pWnd).hImc = if !pImc.is_null() {
        UserHMGetHandle(pImc as _) as _
    } else {
        core::ptr::null_mut()
    };
    hOldImc
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetThreadLayoutHandles(
    hNewKL: HKL,
    hOldKL: HKL,
) -> DWORD {
    let pti: PTHREADINFO;
    let pOldKL: PKL;
    let pNewKL: PKL;

    UserEnterExclusive();

    'quit: {
        pti = GetW32ThreadInfo();
        pOldKL = (*pti).KeyboardLayout;
        if !pOldKL.is_null() && (*pOldKL).hkl != hOldKL {
            break 'quit;
        }

        pNewKL = UserHklToKbl(hNewKL);
        if pNewKL.is_null() {
            break 'quit;
        }

        if IS_IME_HKL(hNewKL) != IS_IME_HKL(hOldKL) {
            (*pti).hklPrev = hOldKL;
        }

        UserAssignmentLock(
            core::ptr::addr_of_mut!((*pti).KeyboardLayout) as *mut PVOID,
            pNewKL as PVOID,
        );
        (*(*pti).pClientInfo).hKL = (*pNewKL).hkl;
    }

    UserLeave();
    0
}

// Plain Rust fn, not `extern "C"`/`extern "fastcall"` like the other internal
// helpers in this file: those calling conventions are for FFI-boundary shape
// parity with the recovered C, but `Result<DWORD, NTSTATUS>` (added here to
// propagate a fault out of the write loop below) is not FFI-safe, and nothing
// outside this file calls UserBuildHimcList.
unsafe fn UserBuildHimcList(
    pti: PTHREADINFO,
    dwCount: DWORD,
    phList: *mut HIMC,
) -> Result<DWORD, NTSTATUS> {
    UserBuildHimcList_impl(pti, dwCount, phList)
}

// Returns Ok(dwRealCount) on success, or Err(status) if a write into phList
// faulted — see the dropped-_SEH2 recovery comment at the NtUserBuildHimcList
// call site below. This helper writes phList directly (as the C does, inside
// the _SEH2_TRY at the call site), so the fault-tolerant copy has to live here
// where the raw stores happen.
unsafe fn UserBuildHimcList_impl(
    mut pti: PTHREADINFO,
    dwCount: DWORD,
    phList: *mut HIMC,
) -> Result<DWORD, NTSTATUS> {
    let mut pIMC: PIMC;
    let mut dwRealCount: DWORD = 0;

    if !pti.is_null() {
        pIMC = (*pti).spDefaultImc;
        while !pIMC.is_null() {
            if dwRealCount < dwCount {
                let handle: HIMC = UserHMGetHandle(pIMC) as _;
                ntkrnl_sys::probe::safe_copy_raw(
                    phList.add(dwRealCount as usize) as *mut core::ffi::c_void,
                    core::ptr::addr_of!(handle) as *const core::ffi::c_void,
                    core::mem::size_of::<HIMC>(),
                )?;
            }
            dwRealCount += 1;
            pIMC = (*pIMC).pImcNext;
        }
    } else {
        pti = (*(*gptiCurrent).ppi).ptiList;
        while !pti.is_null() {
            pIMC = (*pti).spDefaultImc;
            while !pIMC.is_null() {
                if dwRealCount < dwCount {
                    let handle: HIMC = UserHMGetHandle(pIMC) as _;
                    ntkrnl_sys::probe::safe_copy_raw(
                        phList.add(dwRealCount as usize) as *mut core::ffi::c_void,
                        core::ptr::addr_of!(handle) as *const core::ffi::c_void,
                        core::mem::size_of::<HIMC>(),
                    )?;
                }
                dwRealCount += 1;
                pIMC = (*pIMC).pImcNext;
            }
            pti = (*pti).ptiSibling;
        }
    }

    Ok(dwRealCount)
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntImmProcessKey(
    MessageQueue: PUSER_MESSAGE_QUEUE,
    pWnd: PWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> UINT {
    IntImmProcessKey_impl(MessageQueue, pWnd, uMsg, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntImmProcessKey(
    MessageQueue: PUSER_MESSAGE_QUEUE,
    pWnd: PWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> UINT {
    IntImmProcessKey_impl(MessageQueue, pWnd, uMsg, wParam, lParam)
}

unsafe fn IntImmProcessKey_impl(
    MessageQueue: PUSER_MESSAGE_QUEUE,
    pWnd: PWND,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> UINT {
    let mut uVirtualKey: UINT;
    let mut ret: UINT;
    let dwHotKeyId: DWORD;
    let pKL: PKL;
    let mut pIMC: PIMC;
    let pImeHotKey: PIMEHOTKEY;
    let hKL: HKL;
    let hWnd: HWND;

    ASSERT_REFS_CO(pWnd);

    match uMsg {
        WM_KEYDOWN | WM_KEYUP | WM_SYSKEYDOWN | WM_SYSKEYUP => {}
        _ => return 0,
    }

    pIMC = core::ptr::null_mut();
    hWnd = UserHMGetHandle(pWnd) as _;
    pKL = (*head_pti(pWnd)).KeyboardLayout;
    if pKL.is_null() {
        return 0;
    }

    uVirtualKey = LOBYTE(wParam) as UINT;
    pImeHotKey = IntCheckImeHotKey(MessageQueue, uVirtualKey, lParam);
    if !pImeHotKey.is_null() {
        dwHotKeyId = (*pImeHotKey).dwHotKeyId;
        hKL = (*pImeHotKey).hKL;
    } else {
        dwHotKeyId = INVALID_HOTKEY as DWORD;
        hKL = core::ptr::null_mut();
    }

    if IME_HOTKEY_DSWITCH_FIRST as DWORD <= dwHotKeyId
        && dwHotKeyId <= IME_HOTKEY_DSWITCH_LAST as DWORD
    {
        if (*pKL).hkl != hKL {
            UserPostMessage(
                hWnd,
                WM_INPUTLANGCHANGEREQUEST,
                if ((*pKL).dwFontSigs & gSystemFS) != 0 {
                    INPUTLANGCHANGE_SYSCHARSET as WPARAM
                } else {
                    0
                },
                hKL as LPARAM,
            );
        }

        if IntGetImeCompatFlags(head_pti(pWnd)) & 0x800000 != 0 {
            return 0;
        }

        return IPHK_HOTKEY as UINT;
    }

    if IS_IMM_MODE() == 0 {
        return 0;
    }

    if dwHotKeyId == INVALID_HOTKEY as DWORD {
        if (*pKL).piiex.is_null() {
            return 0;
        }

        if !(*pWnd).hImc.is_null() {
            pIMC = UserGetObject(gHandleTable, (*pWnd).hImc as _, TYPE_INPUTCONTEXT as u32) as PIMC;
        }
        if pIMC.is_null() {
            return 0;
        }

        if (lParam & ((KF_UP as LPARAM) << 16)) != 0
            && ((*(*pKL).piiex).ImeInfo.fdwProperty & IME_PROP_IGNORE_UPKEYS) != 0
        {
            return 0;
        }

        match uVirtualKey {
            VK_DBE_CODEINPUT | VK_DBE_ENTERCONFIGMODE | VK_DBE_ENTERWORDREGISTERMODE
            | VK_DBE_HIRAGANA | VK_DBE_KATAKANA | VK_DBE_NOCODEINPUT | VK_DBE_NOROMAN
            | VK_DBE_ROMAN => {}
            _ => {
                if uMsg == WM_SYSKEYDOWN || uMsg == WM_SYSKEYUP {
                    if uVirtualKey != VK_MENU as UINT && uVirtualKey != VK_F10 as UINT {
                        return 0;
                    }
                }

                if ((*(*pKL).piiex).ImeInfo.fdwProperty & IME_PROP_NEED_ALTKEY) == 0 {
                    if uVirtualKey == VK_MENU as UINT || (lParam & 0x20000000) != 0 {
                        return 0;
                    }
                }
            }
        }
    }

    if LOBYTE(uVirtualKey as WPARAM) as UINT == VK_PACKET as UINT {
        uVirtualKey = MAKELONG(wParam as u32, (*GetW32ThreadInfo()).wchInjected as u32) as UINT;
    }

    ret = co_IntImmProcessKey(hWnd, (*pKL).hkl, uVirtualKey, lParam, dwHotKeyId);

    if IntGetImeCompatFlags(head_pti(pWnd)) & 0x800000 != 0 {
        ret &= !(IPHK_HOTKEY as UINT);
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserBuildHimcList(
    dwThreadId: DWORD,
    dwCount: DWORD,
    phList: *mut HIMC,
    pdwCount: PDWORD,
) -> NTSTATUS {
    let mut ret: NTSTATUS = STATUS_UNSUCCESSFUL as NTSTATUS;
    let mut dwRealCount: DWORD = 0;
    let pti: PTHREADINFO;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            break 'quit;
        }

        if dwThreadId == 0 {
            pti = gptiCurrent;
        } else if dwThreadId == (!0u32) {
            pti = core::ptr::null_mut();
        } else {
            pti = IntTID2PTI(UlongToHandle(dwThreadId));
            if pti.is_null() || (*pti).rpdesk.is_null() {
                break 'quit;
            }
        }

        // ---- dropped _SEH2_TRY/_SEH2_EXCEPT recovered here ----
        // C (win32ss/user/ntuser/ime.c, NtUserBuildHimcList):
        //   _SEH2_TRY {
        //       ProbeForWrite(phList, dwCount * sizeof(HIMC), 1);
        //       ProbeForWrite(pdwCount, sizeof(DWORD), 1);
        //       *pdwCount = dwRealCount = UserBuildHimcList(pti, dwCount, phList);
        //   } _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER) {
        //       ERR(...);
        //       _SEH2_YIELD(goto Quit);
        //   } _SEH2_END;
        //
        // Shape: STATUS-returning via the untouched `Quit:` fall-through — `ret`
        // was initialized to STATUS_UNSUCCESSFUL above and is never reassigned
        // on this path, so an exception here simply skips straight to
        // `UserLeave(); return ret;` with that value intact. Literal alignment 1
        // on both probes, matching the C. UserBuildHimcList() itself writes
        // phList directly (see its dropped-_SEH2 recovery), so it returns
        // Result<DWORD, NTSTATUS> here rather than a bare DWORD; a fault mid-list
        // propagates out through `?` exactly like the C's exception unwinding out
        // of the same _SEH2_TRY that wrapped the call.
        let probe_status = SafeProbeForWrite(
            phList as *mut core::ffi::c_void,
            (dwCount as usize * core::mem::size_of::<HIMC>()) as _,
            1,
        );
        let probe_status = if NT_SUCCESS(probe_status) {
            SafeProbeForWrite(
                pdwCount as *mut core::ffi::c_void,
                core::mem::size_of::<DWORD>() as _,
                1,
            )
        } else {
            probe_status
        };
        if !NT_SUCCESS(probe_status) {
            ERR!("Exception in NtUserBuildHimcList: %p, %p\n", phList, pdwCount);
            break 'quit;
        }

        match UserBuildHimcList(pti, dwCount, phList).and_then(|count| {
            dwRealCount = count;
            ntkrnl_sys::probe::safe_copy_raw(
                pdwCount as *mut core::ffi::c_void,
                core::ptr::addr_of!(dwRealCount) as *const core::ffi::c_void,
                core::mem::size_of::<DWORD>(),
            )
        }) {
            Ok(()) => {}
            Err(_) => {
                ERR!("Exception in NtUserBuildHimcList: %p, %p\n", phList, pdwCount);
                break 'quit;
            }
        }

        if dwCount < dwRealCount {
            ret = STATUS_BUFFER_TOO_SMALL as NTSTATUS;
        } else {
            ret = STATUS_SUCCESS as NTSTATUS;
        }
    }

    UserLeave();
    ret
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn UserSetImeConversionKeyState(pti: PTHREADINFO, dwConversion: DWORD) {
    UserSetImeConversionKeyStateImpl(pti, dwConversion)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn UserSetImeConversionKeyState(pti: PTHREADINFO, dwConversion: DWORD) {
    UserSetImeConversionKeyStateImpl(pti, dwConversion)
}

unsafe fn UserSetImeConversionKeyStateImpl(pti: PTHREADINFO, dwConversion: DWORD) {

    const VK_DBE_ALPHANUMERIC: UINT = 0xF0;
    const VK_DBE_KATAKANA: UINT = 0xF1;
    const VK_DBE_HIRAGANA: UINT = 0xF2;
    const VK_DBE_SBCSCHAR: UINT = 0xF3;
    const VK_DBE_DBCSCHAR: UINT = 0xF4;
    const VK_DBE_ROMAN: UINT = 0xF5;
    const VK_DBE_NOROMAN: UINT = 0xF6;
    // jpnvkeys.h:24-25. 0xF8/0xF9 are ENTERIMECONFIGMODE/FLUSHSTRING -- this shadow
    // silently aliased two DIFFERENT virtual keys over the correct file-scope pair.
    const VK_DBE_CODEINPUT: UINT = 0xFA;
    const VK_DBE_NOCODEINPUT: UINT = 0xFB;

    if (*pti).KeyboardLayout.is_null() {
        return;
    }

    let hKL: HKL = (*(*pti).KeyboardLayout).hkl;
    let LangID: LANGID = LOWORD(hKL as u32);
    let KeyState: LPBYTE = (*(*pti).MessageQueue).afKeyState.as_mut_ptr() as LPBYTE;

    match PRIMARYLANGID(LangID) as u32 {
        x if x == LANG_JAPANESE as u32 => {
            let bAlphaNumeric: BOOL = ((dwConversion & IME_CMODE_NATIVE) == 0) as BOOL;
            let bKatakana: BOOL = ((bAlphaNumeric == 0) && (dwConversion & IME_CMODE_KATAKANA) != 0) as BOOL;
            let bHiragana: BOOL = ((bAlphaNumeric == 0) && (dwConversion & IME_CMODE_KATAKANA) == 0) as BOOL;
            SET_KEY_DOWN(KeyState, VK_DBE_ALPHANUMERIC, bAlphaNumeric);
            SET_KEY_LOCKED(KeyState, VK_DBE_ALPHANUMERIC, bAlphaNumeric);
            SET_KEY_DOWN(KeyState, VK_DBE_HIRAGANA, bHiragana);
            SET_KEY_LOCKED(KeyState, VK_DBE_HIRAGANA, bHiragana);
            SET_KEY_DOWN(KeyState, VK_DBE_KATAKANA, bKatakana);
            SET_KEY_LOCKED(KeyState, VK_DBE_KATAKANA, bKatakana);

            let bFullShape: BOOL = ((dwConversion & IME_CMODE_FULLSHAPE) != 0) as BOOL;
            SET_KEY_DOWN(KeyState, VK_DBE_DBCSCHAR, bFullShape);
            SET_KEY_LOCKED(KeyState, VK_DBE_DBCSCHAR, bFullShape);
            SET_KEY_DOWN(KeyState, VK_DBE_SBCSCHAR, (bFullShape == 0) as BOOL);
            SET_KEY_LOCKED(KeyState, VK_DBE_SBCSCHAR, (bFullShape == 0) as BOOL);

            let bRoman: BOOL = ((dwConversion & IME_CMODE_ROMAN) != 0) as BOOL;
            SET_KEY_DOWN(KeyState, VK_DBE_ROMAN, bRoman);
            SET_KEY_LOCKED(KeyState, VK_DBE_ROMAN, bRoman);
            SET_KEY_DOWN(KeyState, VK_DBE_NOROMAN, (bRoman == 0) as BOOL);
            SET_KEY_LOCKED(KeyState, VK_DBE_NOROMAN, (bRoman == 0) as BOOL);

            let bCharCode: BOOL = ((dwConversion & IME_CMODE_CHARCODE) != 0) as BOOL;
            SET_KEY_DOWN(KeyState, VK_DBE_CODEINPUT, bCharCode);
            SET_KEY_LOCKED(KeyState, VK_DBE_CODEINPUT, bCharCode);
            SET_KEY_DOWN(KeyState, VK_DBE_NOCODEINPUT, (bCharCode == 0) as BOOL);
            SET_KEY_LOCKED(KeyState, VK_DBE_NOCODEINPUT, (bCharCode == 0) as BOOL);
        }
        x if x == LANG_KOREAN as u32 => {
            SET_KEY_LOCKED(KeyState, VK_HANGUL as UINT, (dwConversion & IME_CMODE_NATIVE) as BOOL);
            SET_KEY_LOCKED(KeyState, VK_JUNJA as UINT, (dwConversion & IME_CMODE_FULLSHAPE) as BOOL);
            SET_KEY_LOCKED(KeyState, VK_HANJA as UINT, (dwConversion & IME_CMODE_HANJACONVERT) as BOOL);
        }
        _ => {}
    }
}
#[no_mangle]
pub unsafe extern "system" fn NtUserNotifyIMEStatus(
    hwnd: HWND,
    fOpen: BOOL,
    dwConversion: DWORD,
) -> DWORD {

    let pwnd: PWND;
    let pti: PTHREADINFO;
    let hKL: HKL;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            break 'quit;
        }

        pwnd = ValidateHwndNoErr(hwnd);
        if pwnd.is_null() {
            ERR!("Invalid HWND %p\n", hwnd);
            break 'quit;
        }

        pti = head_pti(pwnd);
        if pti.is_null() || gptiForeground.is_null() {
            break 'quit;
        }
        if pti != gptiForeground
            && (*pti).MessageQueue != (*gptiForeground).MessageQueue
        {
            break 'quit;
        }
        if ghIMC == (*pwnd).hImc
            && gfImeOpen == (fOpen != 0) as BOOL
            && gdwImeConversion == dwConversion
        {
            break 'quit;
        }

        ghIMC = (*pwnd).hImc;
        if !ghIMC.is_null() {
            gfImeOpen = (fOpen != 0) as BOOL;
            gdwImeConversion = dwConversion;
            UserSetImeConversionKeyState(
                pti,
                if fOpen != 0 { dwConversion } else { IME_CMODE_ALPHANUMERIC },
            );
        }

        if ISITHOOKED(WH_SHELL as core::ffi::c_int) != 0 {
            hKL = if !(*pti).KeyboardLayout.is_null() {
                (*(*pti).KeyboardLayout).hkl
            } else {
                core::ptr::null_mut()
            };
            co_HOOK_CallHooks(
                WH_SHELL as core::ffi::c_int,
                HSHELL_LANGUAGE as core::ffi::c_int,
                hwnd as WPARAM,
                hKL as LPARAM,
            );
        }
    }

    UserLeave();
    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDisableThreadIme(dwThreadID: DWORD) -> BOOL {
    let pti: PTHREADINFO;
    let ptiCurrent: PTHREADINFO;
    let ppi: PPROCESSINFO;
    let mut ret: BOOL = 0;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            break 'quit;
        }

        ptiCurrent = GetW32ThreadInfo();

        if dwThreadID == INVALID_THREAD_ID {
            ppi = (*ptiCurrent).ppi;
            (*ppi_w32(ppi)).W32PF_flags |= W32PF_DISABLEIME;

            'retry: loop {
                let mut p = (*ppi).ptiList;
                while !p.is_null() {
                    (*p).TIF_flags |= TIF_DISABLEIME;

                    if !(*p).spwndDefaultIme.is_null() {
                        co_UserDestroyWindow((*p).spwndDefaultIme as _);
                        (*p).spwndDefaultIme = core::ptr::null_mut();
                        continue 'retry;
                    }

                    p = (*p).ptiSibling;
                }
                break 'retry;
            }
        } else {
            if dwThreadID == 0 {
                pti = ptiCurrent;
            } else {
                pti = IntTID2PTI(UlongToHandle(dwThreadID));

                if pti.is_null() || (*pti).ppi != (*ptiCurrent).ppi {
                    break 'quit;
                }
            }

            (*pti).TIF_flags |= TIF_DISABLEIME;

            if !(*pti).spwndDefaultIme.is_null() {
                co_UserDestroyWindow((*pti).spwndDefaultIme as _);
                (*pti).spwndDefaultIme = core::ptr::null_mut();
            }
        }

        ret = 1;
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetAppImeLevel(hWnd: HWND) -> DWORD {
    let mut ret: DWORD = 0;
    let pWnd: PWND;
    let pti: PTHREADINFO;

    UserEnterShared();

    'quit: {
        pWnd = ValidateHwndNoErr(hWnd);
        if pWnd.is_null() {
            break 'quit;
        }

        if IS_IMM_MODE() == 0 {
            EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            break 'quit;
        }

        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        if (*head_pti(pWnd)).ppi == (*pti).ppi {
            ret = HandleToUlong(UserGetProp(pWnd, AtomImeLevel, 1u8));
        }
    }

    UserLeave();
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UserGetImeInfoEx(
    pWinSta: PWINSTATION_OBJECT,
    pInfoEx: PIMEINFOEX,
    SearchType: IMEINFOEXCLASS,
) -> BOOL {
    UserGetImeInfoEx_impl(pWinSta, pInfoEx, SearchType)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserGetImeInfoEx(
    pWinSta: PWINSTATION_OBJECT,
    pInfoEx: PIMEINFOEX,
    SearchType: IMEINFOEXCLASS,
) -> BOOL {
    UserGetImeInfoEx_impl(pWinSta, pInfoEx, SearchType)
}

unsafe fn UserGetImeInfoEx_impl(
    pWinSta: PWINSTATION_OBJECT,
    pInfoEx: PIMEINFOEX,
    SearchType: IMEINFOEXCLASS,
) -> BOOL {
    let mut pkl: PKL;
    let pklHead: PKL;

    if pWinSta.is_null() || gspklBaseLayout.is_null() {
        return 0;
    }

    pkl = gspklBaseLayout;
    pklHead = gspklBaseLayout;

    if SearchType == ImeInfoExKeyboardLayout {
        loop {
            if (*pInfoEx).hkl == (*pkl).hkl {
                if (*pkl).piiex.is_null() {
                    break;
                }

                *pInfoEx = *(*pkl).piiex;
                return 1;
            }

            pkl = (*pkl).pklNext;
            if pkl == pklHead {
                break;
            }
        }
    } else if SearchType == ImeInfoExImeFileName {
        loop {
            if !(*pkl).piiex.is_null()
                && _wcsnicmp(
                    core::ptr::addr_of!((*(*pkl).piiex).wszImeFile) as *const _,
                    core::ptr::addr_of!((*pInfoEx).wszImeFile) as *const _,
                    (core::mem::size_of_val(&(*(*pkl).piiex).wszImeFile)
                        / core::mem::size_of_val(&(*(*pkl).piiex).wszImeFile[0]))
                        as _,
                ) == 0
            {
                *pInfoEx = *(*pkl).piiex;
                return 1;
            }

            pkl = (*pkl).pklNext;
            if pkl == pklHead {
                break;
            }
        }
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserGetImeInfoEx(
    pImeInfoEx: PIMEINFOEX,
    SearchType: IMEINFOEXCLASS,
) -> BOOL {
    let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
    let mut ret: BOOL = 0;
    let pWinSta: PWINSTATION_OBJECT;
    let Status: NTSTATUS;

    UserEnterShared();

    'quit: {
        if IS_IMM_MODE() == 0 {
            EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            break 'quit;
        }

        {
            let __SehStatus: NTSTATUS =
                SafeProbeForRead(pImeInfoEx as *mut _, core::mem::size_of::<IMEINFOEX>() as _, 1);
            if !NT_SUCCESS(__SehStatus) {
                break 'quit;
            }
        }
        ImeInfoEx = *pImeInfoEx;

        pWinSta = IntGetProcessWindowStation(core::ptr::null_mut());
        ret = UserGetImeInfoEx(pWinSta, &mut ImeInfoEx, SearchType);
        if ret == 0 {
            break 'quit;
        }

        Status = SafeProbeForWrite(pImeInfoEx as *mut _, core::mem::size_of::<IMEINFOEX>() as _, 1);
        if !NT_SUCCESS(Status) {
            ret = 0;
        } else {
            core::ptr::write_volatile(pImeInfoEx, ImeInfoEx);
        }
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetAppImeLevel(hWnd: HWND, dwLevel: DWORD) -> BOOL {
    let mut ret: BOOL = 0;
    let pWnd: PWND;
    let pti: PTHREADINFO;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            break 'quit;
        }

        pWnd = ValidateHwndNoErr(hWnd);
        if pWnd.is_null() {
            ERR!("Invalid HWND: %p\n", hWnd);
            break 'quit;
        }

        pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
        if (*head_pti(pWnd)).ppi == (*pti).ppi {
            ret = UserSetProp(pWnd, AtomImeLevel, UlongToHandle(dwLevel), 1);
        }
    }

    UserLeave();
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UserSetImeInfoEx(
    pWinSta: PWINSTATION_OBJECT,
    pImeInfoEx: PIMEINFOEX,
) -> BOOL {
    UserSetImeInfoEx_impl(pWinSta, pImeInfoEx)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserSetImeInfoEx(
    pWinSta: PWINSTATION_OBJECT,
    pImeInfoEx: PIMEINFOEX,
) -> BOOL {
    UserSetImeInfoEx_impl(pWinSta, pImeInfoEx)
}

unsafe fn UserSetImeInfoEx_impl(
    pWinSta: PWINSTATION_OBJECT,
    pImeInfoEx: PIMEINFOEX,
) -> BOOL {
    let pklHead: PKL;
    let mut pkl: PKL;

    if pWinSta.is_null() || gspklBaseLayout.is_null() {
        return 0;
    }

    pklHead = gspklBaseLayout;
    pkl = gspklBaseLayout;

    loop {
        if (*pkl).hkl != (*pImeInfoEx).hkl {
            pkl = (*pkl).pklNext;
        } else {
            if (*pkl).piiex.is_null() {
                return 0;
            }

            if (*(*pkl).piiex).fLoadFlag == 0 {
                *(*pkl).piiex = *pImeInfoEx;
            }

            return 1;
        }

        if pkl == pklHead {
            break;
        }
    }

    0
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetImeInfoEx(pImeInfoEx: *const IMEINFOEX) -> BOOL {
    let mut ret: BOOL = 0;
    let mut ImeInfoEx: IMEINFOEX = core::mem::zeroed();
    let pWinSta: PWINSTATION_OBJECT;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            break 'quit;
        }

        let __SehStatus = SafeProbeForRead(
            pImeInfoEx as *mut core::ffi::c_void,
            core::mem::size_of::<IMEINFOEX>() as _,
            1,
        );
        if !NT_SUCCESS(__SehStatus) {
            ERR!(
                "Exception in NtUserSetImeInfoEx: pImeInfoEx=%p\n",
                pImeInfoEx,
            );
            break 'quit;
        }

        ImeInfoEx = *pImeInfoEx;

        pWinSta = IntGetProcessWindowStation(core::ptr::null_mut());
        ret = UserSetImeInfoEx(pWinSta, core::ptr::addr_of_mut!(ImeInfoEx));
    }

    UserLeave();
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntImeSetFutureOwner(pImeWnd: PWND, pwndOwner: PWND) {
    IntImeSetFutureOwner_impl(pImeWnd, pwndOwner)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntImeSetFutureOwner(pImeWnd: PWND, pwndOwner: PWND) {
    IntImeSetFutureOwner_impl(pImeWnd, pwndOwner)
}

unsafe fn IntImeSetFutureOwner_impl(pImeWnd: PWND, pwndOwner: PWND) {
    let mut pwndNode: PWND;
    let mut pwndNextOwner: PWND;
    let pti: PTHREADINFO = head_pti(pImeWnd);

    if pwndOwner.is_null() || ((*pwndOwner).style & WS_CHILD) != 0 {
        return;
    }

    pwndNode = pwndOwner;
    loop {
        pwndNextOwner = (*pwndNode).spwndOwner;
        if pwndNextOwner.is_null() || head_pti(pwndNextOwner) != pti {
            break;
        }
        pwndNode = pwndNextOwner;
    }

    if IS_WND_IMELIKE(pwndNode) != 0
        || (((*pwndNode).state2 & WNDS2_BOTTOMMOST) != 0
            && ((*pwndOwner).state2 & WNDS2_BOTTOMMOST) == 0)
    {
        pwndNode = pwndOwner;
    }

    let pwndParent = (*pwndNode).spwndParent;
    if pwndParent.is_null() || pwndOwner != pwndNode {
        WndSetOwner(pImeWnd, pwndNode);
        return;
    }

    let mut pwndSibling = (*pwndParent).spwndChild;
    while !pwndSibling.is_null() {
        'cont: {
            if head_pti(pwndNode) != head_pti(pwndSibling) {
                break 'cont;
            }

            if IS_WND_MENU(pwndSibling) != 0 || IS_WND_IMELIKE(pwndSibling) != 0 {
                break 'cont;
            }

            if ((*pwndSibling).state2 & WNDS2_INDESTROY) != 0 {
                break 'cont;
            }

            if pwndNode == pwndSibling || ((*pwndSibling).style & WS_CHILD) != 0 {
                break 'cont;
            }

            if (*pwndSibling).spwndOwner.is_null()
                || head_pti(pwndSibling) != head_pti((*pwndSibling).spwndOwner)
            {
                pwndNode = pwndSibling;
                WndSetOwner(pImeWnd, pwndNode);
                return;
            }
        }
        pwndSibling = (*pwndSibling).spwndNext;
    }

    WndSetOwner(pImeWnd, pwndNode);
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntGetLastTopMostWindowNoIME(pImeWnd: PWND) -> PWND {
    IntGetLastTopMostWindowNoIME_impl(pImeWnd)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn IntGetLastTopMostWindowNoIME(pImeWnd: PWND) -> PWND {
    IntGetLastTopMostWindowNoIME_impl(pImeWnd)
}

unsafe fn IntGetLastTopMostWindowNoIME_impl(pImeWnd: PWND) -> PWND {
    let mut pwndNode: PWND;
    let mut pwndOwner: PWND;
    let mut pwndLastTopMost: PWND = core::ptr::null_mut();
    let mut bFound: u8;

    pwndNode = UserGetDesktopWindow();
    if pwndNode.is_null() || (*pwndNode).spwndChild.is_null() {
        return core::ptr::null_mut();
    }

    pwndNode = (*pwndNode).spwndChild;
    while !pwndNode.is_null() && ((*pwndNode).ExStyle & WS_EX_TOPMOST) != 0 {
        bFound = 0u8;

        if IS_WND_IMELIKE(pwndNode) != 0 {
            pwndOwner = pwndNode;
            while !pwndOwner.is_null() {
                if pImeWnd == pwndOwner {
                    bFound = 1u8;
                    break;
                }
                pwndOwner = (*pwndOwner).spwndOwner;
            }
        }

        if bFound == 0 {
            pwndLastTopMost = pwndNode;
        }

        pwndNode = (*pwndNode).spwndNext;
    }

    pwndLastTopMost
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntImeSetTopMost(pImeWnd: PWND, bTopMost: BOOL, pwndInsertBefore: PWND) {
    IntImeSetTopMost_impl(pImeWnd, bTopMost, pwndInsertBefore)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn IntImeSetTopMost(pImeWnd: PWND, bTopMost: BOOL, pwndInsertBefore: PWND) {
    IntImeSetTopMost_impl(pImeWnd, bTopMost, pwndInsertBefore)
}

unsafe fn IntImeSetTopMost_impl(pImeWnd: PWND, bTopMost: BOOL, pwndInsertBefore: PWND) {
    let pwndParent: PWND;
    let mut pwndChild: PWND;
    let mut pwndNode: PWND;
    let mut pwndNext: PWND;
    let mut pwndInsertAfter: PWND = core::ptr::null_mut();
    let pwndInsertAfterSave: PWND;

    pwndParent = (*pImeWnd).spwndParent;
    if pwndParent.is_null() {
        return;
    }

    pwndChild = (*pwndParent).spwndChild;

    if bTopMost == 0 {
        pwndInsertAfter = IntGetLastTopMostWindowNoIME(pImeWnd);
        if !pwndInsertBefore.is_null() {
            pwndNode = pwndInsertAfter;
            while !pwndNode.is_null() {
                if (*pwndNode).spwndNext == pwndInsertBefore {
                    break;
                }

                if pwndNode == pImeWnd {
                    return;
                }

                pwndNode = (*pwndNode).spwndNext;
            }

            if pwndNode.is_null() {
                return;
            }

            pwndInsertAfter = pwndNode;
        }

        if (*(*pImeWnd).spwndOwner).state2 & WNDS2_BOTTOMMOST != 0 {
            pwndNode = pwndInsertAfter;
            while !pwndNode.is_null() {
                if pwndNode == (*pImeWnd).spwndOwner {
                    break;
                }

                if IS_WND_IMELIKE(pwndNode) == 0 {
                    pwndInsertAfter = pwndNode;
                }

                pwndNode = (*pwndNode).spwndNext;
            }
        }
    }

    pwndInsertAfterSave = pwndInsertAfter;

    while !pwndChild.is_null() {
        pwndNext = (*pwndChild).spwndNext;

        if IS_WND_IMELIKE(pwndChild) != 0
            && pwndChild != pwndInsertAfter
            && head_pti(pwndChild) == head_pti(pImeWnd)
        {
            pwndNode = pwndChild;
            while !pwndNode.is_null() {
                if pwndNode != pImeWnd {
                    pwndNode = (*pwndNode).spwndOwner;
                    continue;
                }

                IntUnlinkWindow(pwndChild);

                if bTopMost != 0 {
                    (*pwndChild).ExStyle |= WS_EX_TOPMOST;
                } else {
                    (*pwndChild).ExStyle &= !WS_EX_TOPMOST;
                }

                if pwndInsertAfter.is_null() {
                    IntLinkHwnd(pwndChild, HWND_TOP);
                } else {
                    IntLinkHwnd(pwndChild, UserHMGetHandle(pwndInsertAfter) as _);
                }

                pwndInsertAfter = pwndChild;
                break;
            }
        }

        pwndChild = pwndNext;
        if !pwndChild.is_null() && pwndChild == pwndInsertAfterSave && !pwndInsertAfter.is_null() {
            pwndChild = (*pwndInsertAfter).spwndNext;
        }
    }
}

#[cfg(target_pointer_width = "64")]
unsafe extern "C" fn IntImeCheckTopmost(pImeWnd: PWND) {
    IntImeCheckTopmost_impl(pImeWnd)
}

#[cfg(not(target_pointer_width = "64"))]
unsafe extern "fastcall" fn IntImeCheckTopmost(pImeWnd: PWND) {
    IntImeCheckTopmost_impl(pImeWnd)
}

unsafe fn IntImeCheckTopmost_impl(pImeWnd: PWND) {
    let bTopMost: BOOL;
    let pwndOwner: PWND = (*pImeWnd).spwndOwner;
    let mut pwndInsertBefore: PWND = core::ptr::null_mut();

    if pwndOwner.is_null() {
        return;
    }

    if head_pti(pImeWnd) != gptiForeground {
        pwndInsertBefore = pwndOwner;
    }

    bTopMost = ((*pwndOwner).ExStyle & WS_EX_TOPMOST != 0) as BOOL;
    IntImeSetTopMost(pImeWnd, bTopMost, pwndInsertBefore);
}
#[no_mangle]
pub unsafe extern "system" fn NtUserSetImeOwnerWindow(hImeWnd: HWND, hwndFocus: HWND) -> BOOL {
    let mut ret: BOOL = 0;
    let pImeWnd: PWND;
    let pwndFocus: PWND;
    let mut pwndTopLevel: PWND;
    let mut pwndNode: PWND;
    let pwndActive: PWND;
    let ptiIme: PTHREADINFO;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            break 'quit;
        }

        pImeWnd = ValidateHwndNoErr(hImeWnd);
        if pImeWnd.is_null() || (*pImeWnd).fnid as u32 != FNID_IME {
            ERR!("Not IME window: %p\n", hImeWnd);
            break 'quit;
        }

        pwndFocus = ValidateHwndNoErr(hwndFocus);
        if !pwndFocus.is_null() {
            if IS_WND_IMELIKE(pwndFocus) != 0 {
                break 'quit;
            }

            pwndTopLevel = IntGetTopLevelWindow(pwndFocus);

            pwndNode = pwndTopLevel;
            while !pwndNode.is_null() {
                if (*(*pwndNode).pcls).atomClassName == (*gpsi).atomSysClass[ICLS_IME as usize] {
                    pwndTopLevel = core::ptr::null_mut();
                    break;
                }
                pwndNode = (*pwndNode).spwndOwner;
            }

            WndSetOwner(pImeWnd, pwndTopLevel);
            IntImeCheckTopmost(pImeWnd);
        } else {
            ptiIme = head_pti(pImeWnd);
            pwndActive = (*(*ptiIme).MessageQueue).spwndActive;

            if pwndActive.is_null() || pwndActive != (*pImeWnd).spwndOwner {
                if !pwndActive.is_null()
                    && ptiIme == head_pti(pwndActive)
                    && IS_WND_IMELIKE(pwndActive) == 0
                {
                    WndSetOwner(pImeWnd, pwndActive);
                } else {
                    IntImeSetFutureOwner(pImeWnd, (*pImeWnd).spwndOwner);
                }

                IntImeCheckTopmost(pImeWnd);
            }
        }

        ret = 1;
    }

    UserLeave();
    ret
}

#[no_mangle]
pub unsafe extern "C" fn AllocInputContextObject(
    mut pDesk: PDESKTOP,
    pti: PTHREADINFO,
    Size: SIZE_T,
    HandleOwner: *mut PVOID,
) -> PVOID {
    let ObjHead: PTHRDESKHEAD;

    if pDesk.is_null() {
        pDesk = (*pti).rpdesk;
    }

    ObjHead = DesktopHeapAlloc(pDesk, Size) as PTHRDESKHEAD;
    if ObjHead.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory(ObjHead as PVOID, Size);

    (*ObjHead).pSelf = ObjHead as PVOID;
    (*ObjHead).rpdesk = pDesk;
    set_head_pti(ObjHead, pti);
    IntReferenceThreadInfo(pti);
    core::ptr::write_volatile(HandleOwner, pti as PVOID);
    (*ppi_w32((*pti).ppi)).UserHandleCount += 1;

    ObjHead as PVOID
}

#[no_mangle]
pub unsafe extern "C" fn UserFreeInputContext(Object: PVOID) {
    let ObjHead: PTHRDESKHEAD = Object as PTHRDESKHEAD;
    let pDesk: PDESKTOP = (*ObjHead).rpdesk;
    let pIMC: PIMC = Object as PIMC;
    let mut pNode: PIMC;
    let pti: PTHREADINFO;

    if pIMC.is_null() {
        return;
    }

    pti = head_pti(pIMC);
    pNode = (*pti).spDefaultImc;
    while !pNode.is_null() {
        if (*pNode).pImcNext == pIMC {
            (*pNode).pImcNext = (*pIMC).pImcNext;
            break;
        }
        pNode = (*pNode).pImcNext;
    }

    DesktopHeapFree(pDesk, Object);

    (*ppi_w32((*pti).ppi)).UserHandleCount -= 1;
    IntDereferenceThreadInfo(pti);
}

#[no_mangle]
pub unsafe extern "C" fn UserDestroyInputContext(Object: PVOID) -> BOOLEAN {
    let pIMC: PIMC = Object as PIMC;
    if pIMC.is_null() {
        return 1u8;
    }

    UserMarkObjectDestroy(pIMC as PVOID);
    UserDeleteObject(UserHMGetHandle(pIMC), TYPE_INPUTCONTEXT);
    1u8
}

unsafe fn IntDestroyInputContext(pIMC: PIMC) -> BOOL {
    let hIMC = UserHMGetHandle(pIMC as _);
    let pti: PTHREADINFO = head_pti(pIMC);

    if pti != gptiCurrent {
        EngSetLastError(ERROR_ACCESS_DENIED);
        return 0;
    }

    if pIMC == (*pti).spDefaultImc {
        EngSetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let pwndChild = (*(*(*(*pti).rpdesk).pDeskInfo).spwnd).spwndChild;
    let pwl: PWINDOWLIST = IntBuildHwndList(pwndChild, (IACE_LIST | IACE_CHILDREN) as _, pti);
    if !pwl.is_null() {
        let mut phwnd = (*pwl).ahwnd.as_mut_ptr();
        while *phwnd != HWND_TERMINATOR {
            let pWnd = UserGetObjectNoErr(gHandleTable, *phwnd as _, TYPE_WINDOW as _) as PWND;
            if !pWnd.is_null() && (*pWnd).hImc == hIMC as _ {
                IntAssociateInputContext(pWnd, (*pti).spDefaultImc);
            }
            phwnd = phwnd.add(1);
        }

        IntFreeHwndList(pwl);
    }

    UserDeleteObject(hIMC, TYPE_INPUTCONTEXT as _);
    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserDestroyInputContext(hIMC: HIMC) -> BOOL {
    let mut ret: BOOL = 0;
    let pIMC: PIMC;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            break 'quit;
        }

        pIMC = UserGetObjectNoErr(gHandleTable, hIMC as _, TYPE_INPUTCONTEXT) as PIMC;
        if !pIMC.is_null() {
            ret = IntDestroyInputContext(pIMC);
        }
    }

    UserLeave();
    ret
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn UserCreateInputContext(dwClientImcData: ULONG_PTR) -> PIMC {
    UserCreateInputContext_impl(dwClientImcData)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn UserCreateInputContext(dwClientImcData: ULONG_PTR) -> PIMC {
    UserCreateInputContext_impl(dwClientImcData)
}

unsafe fn UserCreateInputContext_impl(dwClientImcData: ULONG_PTR) -> PIMC {
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let pdesk: PDESKTOP = (*pti).rpdesk;

    if IS_IMM_MODE() == 0 || ((*pti).TIF_flags & TIF_DISABLEIME) != 0 {
        ERR!("IME is disabled\n");
        return core::ptr::null_mut();
    }

    if pdesk.is_null() {
        return core::ptr::null_mut();
    }

    if dwClientImcData != 0 && (*pti).spDefaultImc.is_null() {
        return core::ptr::null_mut();
    }

    let pIMC: PIMC = UserCreateObject(
        gHandleTable,
        pdesk,
        pti,
        core::ptr::null_mut(),
        TYPE_INPUTCONTEXT as u32,
        core::mem::size_of::<IMC>() as _,
    ) as PIMC;
    if pIMC.is_null() {
        return core::ptr::null_mut();
    }

    UserDereferenceObject(pIMC as PVOID);

    if dwClientImcData != 0 {
        (*pIMC).pImcNext = (*(*pti).spDefaultImc).pImcNext;
        (*(*pti).spDefaultImc).pImcNext = pIMC;
    } else {
        UserAssignmentLock(
            core::ptr::addr_of_mut!((*pti).spDefaultImc) as *mut PVOID,
            pIMC as PVOID,
        );
        (*pIMC).pImcNext = core::ptr::null_mut();
    }

    (*pIMC).dwClientImcData = dwClientImcData;
    pIMC
}
#[no_mangle]
pub unsafe extern "system" fn NtUserCreateInputContext(dwClientImcData: ULONG_PTR) -> HIMC {
    let pIMC: PIMC;
    let mut ret: HIMC = core::ptr::null_mut();

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            EngSetLastError(ERROR_CALL_NOT_IMPLEMENTED);
            break 'quit;
        }

        if dwClientImcData == 0 {
            EngSetLastError(ERROR_INVALID_PARAMETER);
            break 'quit;
        }

        pIMC = UserCreateInputContext(dwClientImcData);
        if !pIMC.is_null() {
            ret = UserHMGetHandle(pIMC) as HIMC;
        }
    }

    UserLeave();
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntAssociateInputContextEx(pWnd: PWND, mut pIMC: PIMC, dwFlags: DWORD) -> DWORD {
    IntAssociateInputContextExImpl(pWnd, pIMC, dwFlags)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntAssociateInputContextEx(pWnd: PWND, pIMC: PIMC, dwFlags: DWORD) -> DWORD {
    IntAssociateInputContextExImpl(pWnd, pIMC, dwFlags)
}

unsafe fn IntAssociateInputContextExImpl(pWnd: PWND, mut pIMC: PIMC, dwFlags: DWORD) -> DWORD {
    let mut ret: DWORD = 0;
    let pwl: PWINDOWLIST;
    let bIgnoreNullImc: BOOL = (dwFlags & IACE_IGNORENOCONTEXT) as BOOL;
    let pti: PTHREADINFO = head_pti(pWnd);
    let pwndFocus: PWND = (*(*pti).MessageQueue).spwndFocus;
    let mut pwndTarget: PWND;
    let mut phwnd: *mut HWND;
    let mut hIMC: HIMC;

    if dwFlags & IACE_DEFAULT != 0 {
        pIMC = (*pti).spDefaultImc;
    } else {
        if !pIMC.is_null() && pti != head_pti(pIMC) {
            return 2;
        }
    }

    if (*head_pti(pWnd)).ppi != (*GetW32ThreadInfo()).ppi
        || (!pIMC.is_null() && (*pIMC).head.rpdesk != (*pWnd).head.rpdesk)
    {
        return 2;
    }

    if (dwFlags & IACE_CHILDREN != 0) && !(*pWnd).spwndChild.is_null() {
        pwl = IntBuildHwndList((*pWnd).spwndChild, IACE_CHILDREN | IACE_LIST, pti);
        if !pwl.is_null() {
            phwnd = (*pwl).ahwnd.as_mut_ptr();
            while *phwnd != HWND_TERMINATOR as HWND {
                pwndTarget = ValidateHwndNoErr(*phwnd);
                if pwndTarget.is_null() {
                    phwnd = phwnd.add(1);
                    continue;
                }

                hIMC = if !pIMC.is_null() {
                    UserHMGetHandle(pIMC as _) as _
                } else {
                    core::ptr::null_mut()
                };
                if (*pwndTarget).hImc == hIMC || (bIgnoreNullImc != 0 && (*pwndTarget).hImc.is_null()) {
                    phwnd = phwnd.add(1);
                    continue;
                }

                IntAssociateInputContext(pwndTarget, pIMC);
                if pwndTarget == pwndFocus {
                    ret = 1;
                }

                phwnd = phwnd.add(1);
            }

            IntFreeHwndList(pwl);
        }
    }

    if bIgnoreNullImc == 0 || !(*pWnd).hImc.is_null() {
        hIMC = if !pIMC.is_null() {
            UserHMGetHandle(pIMC as _) as _
        } else {
            core::ptr::null_mut()
        };
        if (*pWnd).hImc != hIMC {
            IntAssociateInputContext(pWnd, pIMC);
            if pWnd == pwndFocus {
                ret = 1;
            }
        }
    }

    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserAssociateInputContext(
    hWnd: HWND,
    hIMC: HIMC,
    dwFlags: DWORD,
) -> DWORD {
    let mut ret: DWORD = 2;
    let pWnd: PWND;
    let pIMC: PIMC;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            break 'quit;
        }

        pWnd = ValidateHwndNoErr(hWnd);
        if pWnd.is_null() {
            ERR!("Invalid HWND: %p\n", hWnd);
            break 'quit;
        }

        pIMC = if !hIMC.is_null() {
            UserGetObjectNoErr(gHandleTable, hIMC as *mut core::ffi::c_void, TYPE_INPUTCONTEXT) as PIMC
        } else {
            core::ptr::null_mut()
        };
        ret = IntAssociateInputContextEx(pWnd, pIMC, dwFlags);
    }

    UserLeave();
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn UserUpdateInputContext(pIMC: PIMC, dwType: DWORD, dwValue: DWORD_PTR) -> BOOL {
    UserUpdateInputContextImpl(pIMC, dwType, dwValue)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn UserUpdateInputContext(pIMC: PIMC, dwType: DWORD, dwValue: DWORD_PTR) -> BOOL {
    UserUpdateInputContextImpl(pIMC, dwType, dwValue)
}

unsafe fn UserUpdateInputContextImpl(pIMC: PIMC, dwType: DWORD, dwValue: DWORD_PTR) -> BOOL {
    let pti: PTHREADINFO = GetW32ThreadInfo();
    let ptiIMC: PTHREADINFO = head_pti(pIMC);

    if (*pti).ppi != (*ptiIMC).ppi {
        return 0;
    }

    match dwType {
        UIC_CLIENTIMCDATA => {
            if (*pIMC).dwClientImcData != 0 {
                return 0;
            }
            (*pIMC).dwClientImcData = dwValue;
        }
        UIC_IMEWINDOW => {
            if ValidateHwndNoErr(dwValue as HWND) == core::ptr::null_mut() {
                return 0;
            }
            (*pIMC).hImeWnd = dwValue as HWND;
        }
        _ => {
            ERR!("Unhandled dwType: %lu\n", dwType);
            return 0;
        }
    }

    1
}
#[no_mangle]
pub unsafe extern "system" fn NtUserUpdateInputContext(
    hIMC: HIMC,
    dwType: DWORD,
    dwValue: DWORD_PTR,
) -> BOOL {
    let pIMC: PIMC;
    let mut ret: BOOL = 0;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            ERR!("!IS_IMM_MODE()\n");
            break 'quit;
        }

        pIMC = UserGetObject(gHandleTable, hIMC as _, TYPE_INPUTCONTEXT) as PIMC;
        if pIMC.is_null() {
            ERR!("Invalid HIMC %p\n", hIMC);
            break 'quit;
        }

        ret = UserUpdateInputContext(pIMC, dwType, dwValue);
    }

    UserLeave();
    ret
}
#[no_mangle]
pub unsafe extern "system" fn NtUserQueryInputContext(hIMC: HIMC, dwType: DWORD) -> DWORD_PTR {
    let pIMC: PIMC;
    let ptiIMC: PTHREADINFO;
    let mut ret: DWORD_PTR = 0;

    UserEnterExclusive();

    'quit: {
        if IS_IMM_MODE() == 0 {
            break 'quit;
        }

        pIMC = UserGetObject(gHandleTable, hIMC as _, TYPE_INPUTCONTEXT) as PIMC;
        if pIMC.is_null() {
            ERR!("Invalid HIMC %p\n", hIMC);
            break 'quit;
        }

        ptiIMC = head_pti(pIMC);

        match dwType {
            QIC_INPUTPROCESSID => {
                ret = PsGetThreadProcessId(pti_ethread(ptiIMC) as _) as DWORD_PTR;
            }
            QIC_INPUTTHREADID => {
                ret = PsGetThreadId(pti_ethread(ptiIMC) as _) as DWORD_PTR;
            }
            QIC_DEFAULTWINDOWIME => {
                if !(*ptiIMC).spwndDefaultIme.is_null() {
                    ret = UserHMGetHandle((*ptiIMC).spwndDefaultIme) as DWORD_PTR;
                }
            }
            QIC_DEFAULTIMC => {
                if !(*ptiIMC).spDefaultImc.is_null() {
                    ret = UserHMGetHandle((*ptiIMC).spDefaultImc) as DWORD_PTR;
                }
            }
            _ => {
                FIXME!(b"dwType: %ld\n\0".as_ptr() as *const i8, dwType);
            }
        }
    }

    UserLeave();
    ret
}

unsafe fn IntFindNonImeRelatedWndOfSameThread(pwndParent: PWND, pwndTarget: PWND) -> BOOL {
    let ptiTarget: PTHREADINFO = head_pti(pwndTarget);

    let mut pwnd = (*pwndParent).spwndChild;
    while !pwnd.is_null() {
        'continue_outer: {
            if pwnd == pwndTarget || head_pti(pwnd) != ptiTarget || IS_WND_MENU(pwnd) != 0 {
                break 'continue_outer;
            }

            if IS_WND_CHILD(pwnd) == 0 {
                let mut bFound1 = FALSE;
                let mut pwndOwner = pwnd;
                while !pwndOwner.is_null() {
                    if IS_WND_IMELIKE(pwndOwner) != 0 {
                        bFound1 = TRUE;
                        break;
                    }
                    pwndOwner = (*pwndOwner).spwndOwner;
                }
                if bFound1 != FALSE {
                    break 'continue_outer;
                }
            }

            let mut pwndNode = pwnd;

            if IS_WND_CHILD(pwndNode) != 0 {
                let mut bFound2 = FALSE;
                while IS_WND_CHILD(pwndNode) != 0 {
                    if head_pti(pwndNode) != ptiTarget {
                        break;
                    }
                    if IS_WND_IMELIKE(pwndNode) != 0 {
                        bFound2 = TRUE;
                        break;
                    }
                    pwndNode = (*pwndNode).spwndParent;
                }
                if bFound2 != FALSE {
                    break 'continue_outer;
                }
            }

            if IS_WND_CHILD(pwndNode) == 0 {
                let mut bFound3 = FALSE;
                while !pwndNode.is_null() {
                    if head_pti(pwndNode) != ptiTarget {
                        break;
                    }
                    if IS_WND_IMELIKE(pwndNode) != 0 {
                        bFound3 = TRUE;
                        break;
                    }
                    pwndNode = (*pwndNode).spwndOwner;
                }
                if bFound3 != FALSE {
                    break 'continue_outer;
                }
            }

            return TRUE as BOOL;
        }

        pwnd = (*pwnd).spwndNext;
    }

    FALSE as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntWantImeWindow(pwndTarget: PWND) -> BOOL {
    IntWantImeWindowImpl(pwndTarget)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntWantImeWindow(pwndTarget: PWND) -> BOOL {
    IntWantImeWindowImpl(pwndTarget)
}

unsafe fn IntWantImeWindowImpl(pwndTarget: PWND) -> BOOL {
    let rpdesk: PDESKTOP;
    let rpwinstaParent: PWINSTATION_OBJECT;
    let mut pwndNode: PWND;
    let pwndParent: PWND = (*pwndTarget).spwndParent;

    if (*gptiCurrent).TIF_flags & TIF_DISABLEIME != 0 {
        return FALSE as BOOL;
    }

    if IS_WND_IMELIKE(pwndTarget) != 0 {
        return FALSE as BOOL;
    }

    if (*pwndTarget).fnid as u32 == FNID_DESKTOP || (*pwndTarget).fnid as u32 == FNID_MESSAGEWND {
        return FALSE as BOOL;
    }

    if (*pwndTarget).state & WNDS_SERVERSIDEWINDOWPROC != 0 {
        return FALSE as BOOL;
    }

    rpdesk = (*pwndTarget).head.rpdesk;
    if rpdesk.is_null() {
        return FALSE as BOOL;
    }

    rpwinstaParent = (*rpdesk).rpwinstaParent;
    if rpwinstaParent.is_null() || ((*rpwinstaParent).Flags & WSS_NOIO) != 0 {
        return FALSE as BOOL;
    }

    pwndNode = pwndParent;
    while !pwndNode.is_null() {
        if rpdesk != (*pwndNode).head.rpdesk {
            break;
        }

        if pwndNode == (*rpdesk).spwndMessage {
            return FALSE as BOOL;
        }

        pwndNode = (*pwndNode).spwndParent;
    }

    TRUE as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn co_IntCreateDefaultImeWindow(pwndTarget: PWND, hInst: HINSTANCE) -> PWND {
    co_IntCreateDefaultImeWindow_impl(pwndTarget, hInst)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn co_IntCreateDefaultImeWindow(pwndTarget: PWND, hInst: HINSTANCE) -> PWND {
    co_IntCreateDefaultImeWindow_impl(pwndTarget, hInst)
}

unsafe fn co_IntCreateDefaultImeWindow_impl(pwndTarget: PWND, hInst: HINSTANCE) -> PWND {
    let mut WindowName: LARGE_UNICODE_STRING = core::mem::zeroed();
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut pImeWnd: PWND;
    let pimeui: PIMEUI;
    let mut Cs: CREATESTRUCTW = core::mem::zeroed();
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();
    let pti: PTHREADINFO = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    let pid: HANDLE = PsGetThreadProcessId(pti_ethread(pti) as _);
    let Status: NTSTATUS;

    if (*pti).spDefaultImc.is_null() && pid == gpidLogon {
        UserCreateInputContext(0);
    }

    if (*pti).spDefaultImc.is_null() || IS_WND_IMELIKE(pwndTarget) != 0 || (*(*pti).rpdesk).pheapDesktop.is_null() {
        return core::ptr::null_mut();
    }

    if IS_WND_CHILD(pwndTarget) != 0 && ((*pwndTarget).style & WS_VISIBLE as u32) == 0 &&
        (*head_pti((*pwndTarget).spwndParent)).ppi != (*pti).ppi {
        return core::ptr::null_mut();
    }

    // The "Default IME" name must outlive co_UserCreateWindowEx below, which
    // reads WindowName.Buffer. Binding to a local (not a temporary) keeps the
    // backing storage alive for the rest of the function; passing
    // `widestring_default_ime().as_ptr()` directly would dangle after this stmt.
    let ime_name = widestring_default_ime();
    RtlInitLargeUnicodeString(
        core::ptr::addr_of_mut!(WindowName),
        ime_name.as_ptr(),
        0,
    );

    ClassName.Buffer = UlongToPtr((*gpsi).atomSysClass[ICLS_IME as usize] as ULONG) as *mut u16;
    ClassName.Length = 0;
    ClassName.MaximumLength = 0;

    UserRefObjectCo(pwndTarget, core::ptr::addr_of_mut!(Ref));

    RtlZeroMemory(
        core::ptr::addr_of_mut!(Cs) as *mut _,
        core::mem::size_of::<CREATESTRUCTW>() as _,
    );
    Cs.style = (WS_POPUP | WS_DISABLED) as _;
    Cs.hInstance = hInst;
    Cs.hwndParent = UserHMGetHandle(pwndTarget) as _;
    Cs.lpszName = WindowName.Buffer;
    Cs.lpszClass = ClassName.Buffer;

    pImeWnd = co_UserCreateWindowEx(
        core::ptr::addr_of_mut!(Cs),
        core::ptr::addr_of_mut!(ClassName),
        core::ptr::addr_of_mut!(WindowName) as PLARGE_STRING,
        core::ptr::null_mut(),
        WINVER as u32,
    );
    if !pImeWnd.is_null() {
        pimeui = IntGetImeUIFromWnd(pImeWnd);
        Status = SafeProbeForWrite(pimeui as *mut _, core::mem::size_of_val(&*pimeui) as _, 1);
        if !NT_SUCCESS(Status) {
            ERR!("Exception in co_IntCreateDefaultImeWindow: %p\n", pimeui);
        } else {
            (*pimeui).__bindgen_anon_1.set_fDefault(1);
            if IS_WND_CHILD(pwndTarget) != 0 && head_pti((*pwndTarget).spwndParent) != pti {
                (*pimeui).__bindgen_anon_1.set_fChildThreadDef(1);
            }
        }
    }

    UserDerefObjectCo(pwndTarget);
    pImeWnd
}

unsafe fn widestring_default_ime() -> [u16; 12] {
    [0x44, 0x65, 0x66, 0x61, 0x75, 0x6c, 0x74, 0x20, 0x49, 0x4d, 0x45, 0x00]
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntImeCanDestroyDefIMEforChild(
    pImeWnd: PWND,
    pwndTarget: PWND,
) -> BOOL {
    IntImeCanDestroyDefIMEforChild_impl(pImeWnd, pwndTarget)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntImeCanDestroyDefIMEforChild(
    pImeWnd: PWND,
    pwndTarget: PWND,
) -> BOOL {
    IntImeCanDestroyDefIMEforChild_impl(pImeWnd, pwndTarget)
}

unsafe fn IntImeCanDestroyDefIMEforChild_impl(pImeWnd: PWND, pwndTarget: PWND) -> BOOL {
    let mut pwndNode: PWND;
    let pimeui: PIMEUI;
    let SafeImeUI: IMEUI;
    let Status: NTSTATUS;

    pimeui = IntGetImeUIFromWnd(pImeWnd);
    if pimeui.is_null() {
        return FALSE as BOOL;
    }

    Status = SafeProbeForRead(pimeui as _, core::mem::size_of::<IMEUI>() as _, 1);
    if !NT_SUCCESS(Status) {
        ERR!("Exception in IntImeCanDestroyDefIMEforChild: %p\n", pimeui);
    } else {
        SafeImeUI = *pimeui;
        if SafeImeUI.__bindgen_anon_1.fChildThreadDef() == 0 {
            return FALSE as BOOL;
        }
    }

    if (*pwndTarget).spwndParent.is_null()
        || head_pti(pwndTarget) == head_pti((*pwndTarget).spwndParent)
    {
        return FALSE as BOOL;
    }

    pwndNode = pwndTarget;
    while !pwndNode.is_null() {
        if pwndNode == (*(*(*pwndNode).head.rpdesk).pDeskInfo).spwnd {
            break;
        }

        if IntFindNonImeRelatedWndOfSameThread((*pwndNode).spwndParent, pwndTarget) != 0 {
            return FALSE as BOOL;
        }

        pwndNode = (*pwndNode).spwndParent;
    }

    TRUE as BOOL
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntImeCanDestroyDefIME(pImeWnd: PWND, pwndTarget: PWND) -> BOOL {
    IntImeCanDestroyDefIME_impl(pImeWnd, pwndTarget)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntImeCanDestroyDefIME(pImeWnd: PWND, pwndTarget: PWND) -> BOOL {
    IntImeCanDestroyDefIME_impl(pImeWnd, pwndTarget)
}

unsafe fn IntImeCanDestroyDefIME_impl(pImeWnd: PWND, pwndTarget: PWND) -> BOOL {
    let mut pwndNode: PWND;
    let pimeui: PIMEUI;
    let SafeImeUI: IMEUI;
    let Status: NTSTATUS;

    pimeui = IntGetImeUIFromWnd(pImeWnd);
    if pimeui.is_null() {
        return FALSE as BOOL;
    }

    // Check IMEUI.fDestroy
    Status = SafeProbeForRead(pimeui as *mut core::ffi::c_void, core::mem::size_of::<IMEUI>() as _, 1);
    if !NT_SUCCESS(Status) {
        ERR!("Exception in IntImeCanDestroyDefIME: %p\n", pimeui);
    } else {
        SafeImeUI = *pimeui;
        if SafeImeUI.__bindgen_anon_1.fDestroy() != 0 {
            return FALSE as BOOL;
        }
    }

    // Any ancestor of pImeWnd is pwndTarget?
    if !(*pImeWnd).spwndOwner.is_null() {
        pwndNode = (*pImeWnd).spwndOwner;
        while !pwndNode.is_null() {
            if pwndNode == pwndTarget {
                break;
            }
            pwndNode = (*pwndNode).spwndOwner;
        }

        if pwndNode.is_null() {
            return FALSE as BOOL;
        }
    }

    // Any ancestor of pwndTarget is IME-like?
    pwndNode = pwndTarget;
    while !pwndNode.is_null() {
        if IS_WND_IMELIKE(pwndNode) != 0 {
            return FALSE as BOOL;
        }
        pwndNode = (*pwndNode).spwndOwner;
    }

    // Adjust the ordering and top-mode status
    IntImeSetFutureOwner(pImeWnd, pwndTarget);
    pwndNode = (*pImeWnd).spwndOwner;
    while !pwndNode.is_null() {
        if pwndNode == pImeWnd {
            break;
        }
        pwndNode = (*pwndNode).spwndNext;
    }
    if pwndNode == pImeWnd {
        IntImeCheckTopmost(pImeWnd);
    }

    // Is the owner of pImeWnd NULL or pwndTarget?
    if !(*pImeWnd).spwndOwner.is_null() && pwndTarget != (*pImeWnd).spwndOwner {
        return FALSE as BOOL;
    }

    WndSetOwner(pImeWnd, core::ptr::null_mut());
    TRUE as BOOL
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntCheckImeShowStatus(pwndIme: PWND, pti: PTHREADINFO) -> BOOL {
    IntCheckImeShowStatusImpl(pwndIme, pti)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntCheckImeShowStatus(pwndIme: PWND, pti: PTHREADINFO) -> BOOL {
    IntCheckImeShowStatusImpl(pwndIme, pti)
}

unsafe fn IntCheckImeShowStatusImpl(pwndIme: PWND, pti: PTHREADINFO) -> BOOL {
    let mut ret: BOOL;
    let mut bDifferent: BOOL;
    let pwl: PWINDOWLIST;
    let mut phwnd: *mut HWND;
    let mut pwndNode: PWND;
    let mut pwndIMC: PWND;
    let ptiCurrent: PTHREADINFO = GetW32ThreadInfo();
    let mut pimeui: PIMEUI;
    let mut SafeImeUI: IMEUI;

    if (*pwndIme).state2 & WNDS2_INDESTROY != 0 {
        return 0;
    }

    pwl = IntBuildHwndList((*(*pwndIme).spwndParent).spwndChild, IACE_LIST as _, core::ptr::null_mut());
    if pwl.is_null() {
        return 0;
    }

    ret = 1;
    phwnd = (*pwl).ahwnd.as_mut_ptr();
    while *phwnd != HWND_TERMINATOR {
        pwndNode = ValidateHwndNoErr(*phwnd);

        if pwndNode.is_null() || pwndIme == pwndNode {
            phwnd = phwnd.add(1);
            continue;
        }

        if (*(*pwndNode).pcls).atomClassName != (*gpsi).atomSysClass[ICLS_IME as usize]
            || (*pwndNode).state2 & WNDS2_INDESTROY != 0
        {
            phwnd = phwnd.add(1);
            continue;
        }

        pimeui = IntGetImeUIFromWnd(pwndNode);
        if pimeui.is_null() {
            phwnd = phwnd.add(1);
            continue;
        }

        if !pti.is_null() && pti != head_pti(pwndNode) {
            phwnd = phwnd.add(1);
            continue;
        }

        bDifferent = 0;
        if (*head_pti(pwndNode)).ppi != (*ptiCurrent).ppi {
            KeAttachProcess(core::ptr::addr_of_mut!((*(*ppi_w32((*head_pti(pwndNode)).ppi)).peProcess).Pcb) as *mut _);
            bDifferent = 1;
        }

        SafeImeUI = *pimeui;
        if SafeImeUI.__bindgen_anon_1.fShowStatus() != 0 {
            pwndIMC = ValidateHwndNoErr((*pimeui).hwndIMC);
            if !pwndIMC.is_null() {
                (*pimeui).__bindgen_anon_1.set_fShowStatus(0);
            }
        } else {
            pwndIMC = core::ptr::null_mut();
        }

        if bDifferent != 0 {
            KeDetachProcess();
        }

        if !pwndIMC.is_null()
            && !head_pti(pwndIMC).is_null()
            && (*head_pti(pwndIMC)).TIF_flags & TIF_INCLEANUP == 0
        {
            let hImeWnd: HWND;
            let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

            UserRefObjectCo(pwndIMC, &mut Ref);

            hImeWnd = UserHMGetHandle(pwndIMC) as _;
            co_IntSendMessage(hImeWnd, WM_IME_NOTIFY, IMN_CLOSESTATUSWINDOW as _, 0);

            UserDerefObjectCo(pwndIMC);
        }

        phwnd = phwnd.add(1);
    }

    IntFreeHwndList(pwl);
    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSendMessageToUI(
    ptiIME: PTHREADINFO,
    pimeui: PIMEUI,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    IntSendMessageToUI_impl(ptiIME, pimeui, uMsg, wParam, lParam)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSendMessageToUI(
    ptiIME: PTHREADINFO,
    pimeui: PIMEUI,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    IntSendMessageToUI_impl(ptiIME, pimeui, uMsg, wParam, lParam)
}

unsafe fn IntSendMessageToUI_impl(
    ptiIME: PTHREADINFO,
    pimeui: PIMEUI,
    uMsg: UINT,
    wParam: WPARAM,
    lParam: LPARAM,
) -> LRESULT {
    let pwndUI: PWND;
    let mut ret: LRESULT = 0;
    let SafeImeUI: IMEUI;
    let mut bDifferent: BOOL = 0;
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    // Attach to the process if necessary
    if ptiIME != GetW32ThreadInfo() {
        bDifferent = 1;
        KeAttachProcess(core::ptr::addr_of_mut!((*(*ppi_w32((*ptiIME).ppi)).peProcess).Pcb) as *mut _);
    }

    // Get the pwndUI
    SafeImeUI = *pimeui;
    pwndUI = ValidateHwndNoErr(SafeImeUI.hwndUI);

    'quit: {
        if pwndUI.is_null() {
            break 'quit;
        }

        // Increment the recursion count of the IME procedure.
        InterlockedIncrement(core::ptr::addr_of_mut!((*pimeui).nCntInIMEProc));

        // Detach from the process if necessary
        if bDifferent != 0 {
            KeDetachProcess();
        }

        UserRefObjectCo(pwndUI, &mut Ref);
        ret = co_IntSendMessage(UserHMGetHandle(pwndUI) as _, uMsg, wParam, lParam);
        UserDerefObjectCo(pwndUI);

        // Attach to the process if necessary
        if bDifferent != 0 {
            KeAttachProcess(core::ptr::addr_of_mut!((*(*ppi_w32((*ptiIME).ppi)).peProcess).Pcb) as *mut _);
        }

        // Decrement the recursion count of the IME procedure
        InterlockedDecrement(core::ptr::addr_of_mut!((*pimeui).nCntInIMEProc));
    }

    // Detach from the process if necessary
    if bDifferent != 0 {
        KeDetachProcess();
    }

    ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntSendOpenStatusNotify(
    ptiIME: PTHREADINFO,
    pimeui: PIMEUI,
    pWnd: PWND,
    bOpen: BOOL,
) -> () {
    IntSendOpenStatusNotify_impl(ptiIME, pimeui, pWnd, bOpen)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntSendOpenStatusNotify(
    ptiIME: PTHREADINFO,
    pimeui: PIMEUI,
    pWnd: PWND,
    bOpen: BOOL,
) -> () {
    IntSendOpenStatusNotify_impl(ptiIME, pimeui, pWnd, bOpen)
}

unsafe fn IntSendOpenStatusNotify_impl(
    ptiIME: PTHREADINFO,
    pimeui: PIMEUI,
    pWnd: PWND,
    bOpen: BOOL,
) {

    let wParam: WPARAM = if bOpen != 0 {
        IMN_OPENSTATUSWINDOW as WPARAM
    } else {
        IMN_CLOSESTATUSWINDOW as WPARAM
    };
    let ptiWnd: PTHREADINFO = head_pti(pWnd);
    let mut Ref: USER_REFERENCE_ENTRY = core::mem::zeroed();

    if (*ptiWnd).dwExpWinVer >= WINVER_WINNT4 as _ && (*pWnd).hImc != core::ptr::null_mut() {
        UserRefObjectCo(pWnd as *mut core::ffi::c_void, &mut Ref);
        co_IntSendMessage(
            UserHMGetHandle(pWnd as *mut core::ffi::c_void) as HWND,
            WM_IME_NOTIFY,
            wParam,
            0,
        );
        UserDerefObjectCo(pWnd as *mut core::ffi::c_void);
    } else {
        IntSendMessageToUI(ptiIME, pimeui, WM_IME_NOTIFY, wParam, 0);
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntNotifyImeShowStatus(pImeWnd: PWND) {
    IntNotifyImeShowStatus_impl(pImeWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntNotifyImeShowStatus(pImeWnd: PWND) {
    IntNotifyImeShowStatus_impl(pImeWnd)
}

unsafe fn IntNotifyImeShowStatus_impl(pImeWnd: PWND) {
    let pimeui: PIMEUI;
    let mut pWnd: PWND;
    let pti: PTHREADINFO;
    let ptiIME: PTHREADINFO;
    let bShow: BOOL;
    let mut bSendNotify: BOOL = 0;
    let mut SafeImeUI: IMEUI = core::mem::zeroed();

    if IS_IMM_MODE() == 0 || ((*pImeWnd).state2 & WNDS2_INDESTROY) != 0 {
        return;
    }

    pti = PsGetCurrentThreadWin32Thread() as PTHREADINFO;
    ptiIME = head_pti(pImeWnd);

    pimeui = IntGetImeUIFromWnd(pImeWnd);
    if pimeui.is_null() {
        ERR!("Invalid IMEWND %p\n", pImeWnd);
        return;
    }

    if pti != ptiIME {
        KeAttachProcess(core::ptr::addr_of_mut!((*(*ppi_w32((*ptiIME).ppi)).peProcess).Pcb) as *mut _);
    }

    // DUMMY_PSEH: execute TRY body, omit handler
    SafeImeUI = *pimeui;

    bShow = ((gfIMEShowStatus == 1) && SafeImeUI.__bindgen_anon_1.fCtrlShowStatus() != 0) as BOOL;

    pWnd = ValidateHwndNoErr(SafeImeUI.hwndIMC);
    if pWnd.is_null() {
        pWnd = (*(*ptiIME).MessageQueue).spwndFocus;
    }

    if !pWnd.is_null() {
        bSendNotify = 1;
        (*pimeui).__bindgen_anon_1.set_fShowStatus(bShow as u32);
    }

    if pti != ptiIME {
        KeDetachProcess();
    }

    if bSendNotify != 0 {
        IntSendOpenStatusNotify(ptiIME, core::ptr::addr_of_mut!(SafeImeUI), pWnd, bShow);
    }

    if ((*pImeWnd).state2 & WNDS2_INDESTROY) == 0 {
        IntCheckImeShowStatus(pImeWnd, core::ptr::null_mut());
    }
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntBroadcastImeShowStatusChange(pImeWnd: PWND, bShow: BOOL) -> BOOL {
    IntBroadcastImeShowStatusChange_impl(pImeWnd, bShow)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntBroadcastImeShowStatusChange(pImeWnd: PWND, bShow: BOOL) -> BOOL {
    IntBroadcastImeShowStatusChange_impl(pImeWnd, bShow)
}

unsafe fn IntBroadcastImeShowStatusChange_impl(pImeWnd: PWND, bShow: BOOL) -> BOOL {
    if gfIMEShowStatus == bShow || IS_IMM_MODE() == 0 {
        return 1;
    }

    gfIMEShowStatus = bShow;
    IntNotifyImeShowStatus(pImeWnd);
    1
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntCheckImeShowStatusInThread(pImeWnd: PWND) {
    IntCheckImeShowStatusInThreadImpl(pImeWnd)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntCheckImeShowStatusInThread(pImeWnd: PWND) {
    IntCheckImeShowStatusInThreadImpl(pImeWnd)
}

unsafe fn IntCheckImeShowStatusInThreadImpl(pImeWnd: PWND) {
    if IS_IMM_MODE() != 0 && ((*pImeWnd).state2 & WNDS2_INDESTROY) == 0 {
        IntCheckImeShowStatus(pImeWnd, head_pti(pImeWnd));
    }
}


// ABI-corrected extern decls — i386 calling-convention must match the
// definition; a flat extern "C" decl of a stdcall/fastcall fn is UB -> ud2.
extern "system" {
    fn RtlInitLargeUnicodeString(dst: PLARGE_UNICODE_STRING, src: *const u16, max: ULONG);
}
