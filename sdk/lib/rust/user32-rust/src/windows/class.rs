//! windows/class.rs — Rust port of win32ss/user/user32/windows/class.c
//!
//! Window class management: Register/Unregister/GetClassInfo*/Get/SetClassLong*,
//! Get/SetWindowLong*, versioned-class (sxs) resolution.
//!
//! Integrated against the user32-rust substrate (win32k-sys + crate::{ntwrapper,
//! internal, types}). See the integration notes at the bottom of each helper.

use win32k_sys::win32k::*;
use crate::ntwrapper::*;
use crate::internal::*;
// user32 BOOL/FALSE/TRUE are i32; win32k-sys defines them too (u32 FALSE/TRUE) — shadow locally.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;
use core::ffi::c_void;

// ---------------------------------------------------------------------------
// Local externs — symbols not provided by the crate substrate.
// ---------------------------------------------------------------------------

// kernel32 imports (resolve via importlib at link).
extern "system" {
    fn GetModuleHandleW(lpModuleName: LPCWSTR) -> HMODULE;
    fn LoadLibraryW(lpLibFileName: LPCWSTR) -> HMODULE;
    fn GetProcAddress(hModule: HMODULE, lpProcName: LPCSTR) -> *mut c_void;
    fn FreeLibrary(hLibModule: HMODULE) -> BOOL;
    fn GetLastError() -> DWORD;
    fn SetLastError(dwErrCode: DWORD);
    fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: core::ffi::c_int,
        lpWideCharStr: LPWSTR,
        cchWideChar: core::ffi::c_int,
    ) -> core::ffi::c_int;
}

// user32-internal public exports living in OTHER user32 .c files (WINAPI/stdcall).
extern "system" {
    fn GetSysColorBrush(nIndex: core::ffi::c_int) -> HBRUSH;      // syscolor.c
    fn GetAncestor(hwnd: HWND, gaFlags: UINT) -> HWND;            // window.c
    fn GetDesktopWindow() -> HWND;                                // window.c
    fn GetWindow(hWnd: HWND, uCmd: UINT) -> HWND;                 // window.c
    fn RegisterSystemControls() -> BOOL;                         // regcontrol.h
}

// user32-internal FASTCALL helper (user32p.h).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn TestWindowProcess(pWnd: PWND) -> BOOL;                     // user32p.h
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn TestWindowProcess(pWnd: PWND) -> BOOL;                     // user32p.h
}

// ntdll activation-context APIs (not in crate::internal; resolve via importlib).
#[cfg(target_arch = "x86")]
extern "fastcall" {
    fn RtlActivateActivationContextUnsafeFast(
        Frame: *mut RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
        Context: PVOID,
    ) -> PVOID;
    fn RtlDeactivateActivationContextUnsafeFast(
        Frame: *mut RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
    ) -> PVOID;
}
#[cfg(not(target_arch = "x86"))]
extern "C" {
    fn RtlActivateActivationContextUnsafeFast(
        Frame: *mut RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
        Context: PVOID,
    ) -> PVOID;
    fn RtlDeactivateActivationContextUnsafeFast(
        Frame: *mut RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
    ) -> PVOID;
}
extern "system" {
    fn RtlFindActivationContextSectionString(
        dwFlags: ULONG,
        ExtensionGuid: *const c_void,
        SectionType: ULONG,
        SectionName: *mut _UNICODE_STRING,
        ReturnedData: *mut ACTCTX_SECTION_KEYED_DATA,
    ) -> NTSTATUS;
    fn RtlReleaseActivationContext(handle: HANDLE);
}

// CRT.
extern "C" {
    fn wcslen(s: *const u16) -> usize;
    fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
}

// user32 globals (linkable user32 exports).
extern "C" {
    static User32Instance: HINSTANCE;          // dllmain.c / user32p.h
    static RegisterDefaultClasses: DWORD;       // regcontrol.c / regcontrol.h
    static gSharedInfo: SHAREDINFO;             // dllmain.c / user32p.h
}

// ---------------------------------------------------------------------------
// Local constants / type aliases (values verified against the C headers).
// ---------------------------------------------------------------------------

// winbase.h: FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX 0x00000001
const FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX: ULONG = 0x0000_0001;
// winnt.h: ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION 3
const ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION: ULONG = 3;
// winnls.h: CP_ACP 0
const CP_ACP: UINT = 0;
// user32 ntuser.h: MAX_BUFFER_LEN
const MAX_BUFFER_LEN: usize = 256;
const MAX_ATOM_LEN: i32 = 255;

// IS_ATOM / IS_INTRESOURCE — wine/winuser macros: a pointer with the high bits
// clear (value < 0x10000) is an atom / int-resource. Faithful to the C macros.
#[inline]
unsafe fn IS_INTRESOURCE(p: *const c_void) -> bool {
    (p as usize) >> 16 == 0
}
#[inline]
unsafe fn IS_ATOM(p: *const c_void) -> bool {
    // user32p.h: IS_ATOM(x) == (((ULONG_PTR)x) > 0 && ((ULONG_PTR)x) < 0x10000)
    // Upper bound is 0x10000 (high bits clear), NOT MAXINTATOM (0xC000) — real
    // class atoms are >= 0xC000, so a 0xC000 bound misclassifies them as strings.
    let v = p as usize;
    v > 0 && v < 0x10000
}

// strcmpiW — wine unicode helper (host-only inline in C); case-insensitive UTF-16
// compare. Translated faithfully here (ASCII-fold, like wine's strcmpiW).
#[inline]
unsafe fn strcmpiW(mut s1: *const u16, mut s2: *const u16) -> i32 {
    loop {
        let mut c1 = *s1 as u32;
        let mut c2 = *s2 as u32;
        if (b'A' as u32..=b'Z' as u32).contains(&c1) {
            c1 += 0x20;
        }
        if (b'A' as u32..=b'Z' as u32).contains(&c2) {
            c2 += 0x20;
        }
        if c1 != c2 {
            return c1 as i32 - c2 as i32;
        }
        if c1 == 0 {
            return 0;
        }
        s1 = s1.add(1);
        s2 = s2.add(1);
    }
}

// strlenW(s) == wcslen(s) (wine/unicode.h).
#[inline]
unsafe fn strlenW(s: *const u16) -> usize {
    wcslen(s)
}

// NT_SUCCESS(Status) := Status >= 0.
#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// GetWin32ClientInfo() := (PCLIENTINFO)(NtCurrentTeb()->Win32ClientInfo)
// (ntuser.h). Win32ClientInfo is the in-TEB storage array → address-of = the
// CLIENTINFO pointer.
#[inline]
unsafe fn GetWin32ClientInfo() -> PCLIENTINFO {
    core::ptr::addr_of_mut!((*NtCurrentTeb()).Win32ClientInfo) as PCLIENTINFO
}

// GETPFNSERVER(fnid) := gpsi->aStoCidPfn[fnid - FNID_FIRST]   (ntuser.h)
#[inline]
unsafe fn GETPFNSERVER(fnid: i32) -> WNDPROC {
    crate::wow64::psi::pfn_server(fnid as usize)
}
// GETPFNCLIENTA(fnid) := (WNDPROC)(*(((ULONG_PTR*)&gpsi->apfnClientA) + (fnid - FNID_FIRST)))
#[inline]
unsafe fn GETPFNCLIENTA(fnid: i32) -> WNDPROC {
    crate::wow64::psi::pfn_client_a(fnid as usize)
}
// GETPFNCLIENTW(fnid) := (WNDPROC)(*(((ULONG_PTR*)&gpsi->apfnClientW) + (fnid - FNID_FIRST)))
#[inline]
unsafe fn GETPFNCLIENTW(fnid: i32) -> WNDPROC {
    crate::wow64::psi::pfn_client_w(fnid as usize)
}
// UserHMGetHandle(obj) := (obj)->head.h   (ntuser.h)
// NOTE: bindgen flattens the leading `_HEAD` of THRDESKHEAD into
// `__bindgen_padding_0`, so the `.h` field (first member of _HEAD, offset 0 of
// head) is read directly off the head address.
#[inline]
unsafe fn UserHMGetHandle(pWnd: PWND) -> HANDLE {
    *(core::ptr::addr_of!((*pWnd).head) as *const HANDLE)
}

// ((PPROCMARKHEAD)p)->h := the `h` field of the embedded _HEAD at offset 0.
#[inline]
unsafe fn ProcMarkHeadHandle(p: PPROCMARKHEAD) -> HANDLE {
    *(p as *const HANDLE)
}

// SharedPtrToUser (user_x.h, static inline).
#[inline]
unsafe fn SharedPtrToUser(Ptr: PVOID) -> PVOID {
    (Ptr as ULONG_PTR - gSharedInfo.ulSharedDelta) as PVOID
}

// DesktopPtrToUser (user_x.h, static inline).
#[inline]
unsafe fn DesktopPtrToUser(Ptr: PVOID) -> PVOID {
    // WoW64: DESKTOPINFO is 64-bit-laid-out for a 32-bit guest, so the
    // base/limit range check must be done through the normalizing view.
    // Single definition lives in crate::wow64.
    crate::wow64::desktop_ptr_to_user(Ptr)
}

// ---------------------------------------------------------------------------
// Activation-context types (faithful to ntdll-sys bindgen layout; ntdll-sys is
// not a dependency of this crate, so the layouts are mirrored locally — verified
// field-for-field against sdk/lib/rust/ntdll-sys/src/bindgen_*.rs).
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone)]
struct ACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA {
    lpInformation: PVOID,
    lpSectionBase: PVOID,
    ulSectionLength: ULONG,
    lpSectionGlobalDataBase: PVOID,
    ulSectionGlobalDataLength: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct ACTCTX_SECTION_KEYED_DATA {
    cbSize: ULONG,
    ulDataFormatVersion: ULONG,
    lpData: PVOID,
    ulLength: ULONG,
    lpSectionGlobalData: PVOID,
    ulSectionGlobalDataLength: ULONG,
    lpSectionBase: PVOID,
    ulSectionTotalLength: ULONG,
    hActCtx: HANDLE,
    ulAssemblyRosterIndex: ULONG,
    ulFlags: ULONG,
    AssemblyMetadata: ACTCTX_SECTION_KEYED_DATA_ASSEMBLY_METADATA,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct RTL_ACTIVATION_CONTEXT_STACK_FRAME {
    Previous: *mut RTL_ACTIVATION_CONTEXT_STACK_FRAME,
    ActivationContext: PVOID,
    Flags: ULONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
struct RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED {
    Size: SIZE_T,
    Format: ULONG,
    Frame: RTL_ACTIVATION_CONTEXT_STACK_FRAME,
    Extra1: PVOID,
    Extra2: PVOID,
    Extra3: PVOID,
    Extra4: PVOID,
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct strsection_header {
    pub magic: DWORD,
    pub size: ULONG,
    pub unk1: [DWORD; 3],
    pub count: ULONG,
    pub index_offset: ULONG,
    pub unk2: [DWORD; 2],
    pub global_offset: ULONG,
    pub global_len: ULONG,
}

#[repr(C)]
pub struct wndclass_redirect_data {
    pub size: ULONG,
    pub res: DWORD,
    pub name_len: ULONG,
    pub name_offset: ULONG,
    pub module_len: ULONG,
    pub module_offset: ULONG,
}

unsafe fn is_comctl32_class(name: *const u16) -> LPCWSTR {

    static classesW: [[u16; 20]; 19] = [
        [b'C' as u16, b'o' as u16, b'm' as u16, b'b' as u16, b'o' as u16, b'B' as u16, b'o' as u16, b'x' as u16, b'E' as u16, b'x' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0, 0, 0, 0],
        [b'm' as u16, b's' as u16, b'c' as u16, b't' as u16, b'l' as u16, b's' as u16, b'_' as u16, b'h' as u16, b'o' as u16, b't' as u16, b'k' as u16, b'e' as u16, b'y' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0],
        [b'm' as u16, b's' as u16, b'c' as u16, b't' as u16, b'l' as u16, b's' as u16, b'_' as u16, b'p' as u16, b'r' as u16, b'o' as u16, b'g' as u16, b'r' as u16, b'e' as u16, b's' as u16, b's' as u16, b'3' as u16, b'2' as u16, 0, 0, 0],
        [b'm' as u16, b's' as u16, b'c' as u16, b't' as u16, b'l' as u16, b's' as u16, b'_' as u16, b's' as u16, b't' as u16, b'a' as u16, b't' as u16, b'u' as u16, b's' as u16, b'b' as u16, b'a' as u16, b'r' as u16, b'3' as u16, b'2' as u16, 0, 0],
        [b'm' as u16, b's' as u16, b'c' as u16, b't' as u16, b'l' as u16, b's' as u16, b'_' as u16, b't' as u16, b'r' as u16, b'a' as u16, b'c' as u16, b'k' as u16, b'b' as u16, b'a' as u16, b'r' as u16, b'3' as u16, b'2' as u16, 0, 0, 0],
        [b'm' as u16, b's' as u16, b'c' as u16, b't' as u16, b'l' as u16, b's' as u16, b'_' as u16, b'u' as u16, b'p' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'n' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0],
        [b'N' as u16, b'a' as u16, b't' as u16, b'i' as u16, b'v' as u16, b'e' as u16, b'F' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'C' as u16, b't' as u16, b'l' as u16, 0, 0, 0, 0, 0, 0, 0],
        [b'R' as u16, b'e' as u16, b'B' as u16, b'a' as u16, b'r' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'A' as u16, b'n' as u16, b'i' as u16, b'm' as u16, b'a' as u16, b't' as u16, b'e' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'D' as u16, b'a' as u16, b't' as u16, b'e' as u16, b'T' as u16, b'i' as u16, b'm' as u16, b'e' as u16, b'P' as u16, b'i' as u16, b'c' as u16, b'k' as u16, b'3' as u16, b'2' as u16, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'H' as u16, b'e' as u16, b'a' as u16, b'd' as u16, b'e' as u16, b'r' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'I' as u16, b'P' as u16, b'A' as u16, b'd' as u16, b'd' as u16, b'r' as u16, b'e' as u16, b's' as u16, b's' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'L' as u16, b'i' as u16, b's' as u16, b't' as u16, b'V' as u16, b'i' as u16, b'e' as u16, b'w' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'M' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'h' as u16, b'C' as u16, b'a' as u16, b'l' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'P' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'T' as u16, b'a' as u16, b'b' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0],
        [b'S' as u16, b'y' as u16, b's' as u16, b'T' as u16, b'r' as u16, b'e' as u16, b'e' as u16, b'V' as u16, b'i' as u16, b'e' as u16, b'w' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0, 0, 0],
        [b'T' as u16, b'o' as u16, b'o' as u16, b'l' as u16, b'b' as u16, b'a' as u16, b'r' as u16, b'W' as u16, b'i' as u16, b'n' as u16, b'd' as u16, b'o' as u16, b'w' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0, 0],
        [b't' as u16, b'o' as u16, b'o' as u16, b'l' as u16, b't' as u16, b'i' as u16, b'p' as u16, b's' as u16, b'_' as u16, b'c' as u16, b'l' as u16, b'a' as u16, b's' as u16, b's' as u16, b'3' as u16, b'2' as u16, 0, 0, 0, 0],
    ];

    let mut min: i32 = 0;
    let mut max: i32 = (classesW.len() as i32) - 1;

    while min <= max {
        let pos = (min + max) / 2;
        let res = strcmpiW(name, classesW[pos as usize].as_ptr());
        if res == 0 {
            return classesW[pos as usize].as_ptr();
        }
        if res < 0 {
            max = pos - 1;
        } else {
            min = pos + 1;
        }
    }
    core::ptr::null_mut()
}

// ClassNameToVersion — FASTCALL in C. Referenced cross-TU (window.c/window.rs)
// via the decorated `@ClassNameToVersion@20` symbol → exported here.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn ClassNameToVersion(
    lpszClass: *const c_void,
    lpszMenuName: LPCWSTR,
    plpLibFileName: *mut LPCWSTR,
    pContext: *mut *mut c_void,
    bAnsi: i32,
) -> LPCWSTR {
    ClassNameToVersion_impl(lpszClass, lpszMenuName, plpLibFileName, pContext, bAnsi)
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn ClassNameToVersion(
    lpszClass: *const c_void,
    lpszMenuName: LPCWSTR,
    plpLibFileName: *mut LPCWSTR,
    pContext: *mut *mut c_void,
    bAnsi: i32,
) -> LPCWSTR {
    ClassNameToVersion_impl(lpszClass, lpszMenuName, plpLibFileName, pContext, bAnsi)
}

unsafe fn ClassNameToVersion_impl(
    lpszClass: *const c_void,
    _lpszMenuName: LPCWSTR,
    plpLibFileName: *mut LPCWSTR,
    pContext: *mut *mut c_void,
    bAnsi: i32,
) -> LPCWSTR {
    let mut VersionedClass: LPCWSTR = core::ptr::null_mut();
    let Status: NTSTATUS;
    let mut SectionName: _UNICODE_STRING = core::mem::zeroed();
    let mut SectionNameBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut KeyedData: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();
    KeyedData.cbSize = core::mem::size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;

    if lpszClass.is_null() {
        return core::ptr::null_mut();
    }

    if IS_ATOM(lpszClass) {
        RtlInitEmptyUnicodeString(
            &mut SectionName,
            SectionNameBuf.as_mut_ptr(),
            (core::mem::size_of::<u16>() * SectionNameBuf.len()) as u16,
        );
        if NtUserGetAtomName(LOWORD(lpszClass as DWORD_PTR as DWORD), &mut SectionName) == 0 {
            return core::ptr::null_mut();
        }
        SectionName.Length =
            (wcslen(SectionNameBuf.as_ptr()) * core::mem::size_of::<u16>()) as u16;
    } else {
        if bAnsi != 0 {
            let mut AnsiString: ANSI_STRING = core::mem::zeroed();
            RtlInitAnsiString(&mut AnsiString, lpszClass as *const i8 as PCSZ);
            RtlInitEmptyUnicodeString(
                &mut SectionName,
                SectionNameBuf.as_mut_ptr(),
                (core::mem::size_of::<u16>() * SectionNameBuf.len()) as u16,
            );
            RtlAnsiStringToUnicodeString(&mut SectionName, &mut AnsiString, 0u8);
        } else {
            RtlInitUnicodeString(&mut SectionName, lpszClass as *const u16);
        }
    }

    Status = RtlFindActivationContextSectionString(
        FIND_ACTCTX_SECTION_KEY_RETURN_HACTCTX,
        core::ptr::null(),
        ACTIVATION_CONTEXT_SECTION_WINDOW_CLASS_REDIRECTION,
        &mut SectionName,
        &mut KeyedData,
    );

    if NT_SUCCESS(Status) && KeyedData.ulDataFormatVersion == 1 {
        let SectionHeader = KeyedData.lpSectionBase as *mut strsection_header;

        if !SectionHeader.is_null() && (*SectionHeader).count > 0 {
            let WindowRedirectionData = KeyedData.lpData as *mut wndclass_redirect_data;
            if !WindowRedirectionData.is_null() && (*WindowRedirectionData).module_len != 0 {
                let lpLibFileName: LPCWSTR;

                VersionedClass = (WindowRedirectionData as *mut u8)
                    .add((*WindowRedirectionData).name_offset as usize)
                    as *mut u16;
                lpLibFileName = (KeyedData.lpSectionBase as *mut u8)
                    .add((*WindowRedirectionData).module_offset as usize)
                    as *mut u16;

                if !pContext.is_null() {
                    *pContext = KeyedData.hActCtx;
                }
                if !plpLibFileName.is_null() {
                    *plpLibFileName = lpLibFileName;
                }
            }
        }
    }

    if !KeyedData.hActCtx.is_null() {
        RtlReleaseActivationContext(KeyedData.hActCtx);
    }

    if VersionedClass.is_null() {
        VersionedClass = is_comctl32_class(SectionName.Buffer);
        if !VersionedClass.is_null() {
            if !pContext.is_null() {
                *pContext = core::ptr::null_mut();
            }
            if !plpLibFileName.is_null() {
                static COMCTL32: [u16; 9] =
                    [b'c' as u16, b'o' as u16, b'm' as u16, b'c' as u16, b't' as u16,
                     b'l' as u16, b'3' as u16, b'2' as u16, 0];
                *plpLibFileName = COMCTL32.as_ptr();
            }
        }
    }

    VersionedClass
}

// VersionRegisterClass — FASTCALL, declared in regcontrol.h → cross-TU export.
fastcall_fn! {
#[no_mangle]
pub unsafe fn VersionRegisterClass(
    mut pszClass: PCWSTR,
    lpLibFileName: LPCWSTR,
    Contex: HANDLE,
    phLibModule: *mut HMODULE,
) -> BOOL {
    let mut Ret: BOOL = 0;
    let mut hLibModule: HMODULE = core::ptr::null_mut();
    let pRegisterClassNameW: PREGISTERCLASSNAMEW;
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut ClassNameBuf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut Frame: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED =
        core::mem::zeroed();
    Frame.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED>() as SIZE_T;
    Frame.Format = 1;

    RtlActivateActivationContextUnsafeFast(&mut Frame, Contex);

    // _SEH2_TRY body — runs normally; the _SEH2_EXCEPT handler is DEAD CODE
    // under USE_DUMMY_PSEH and is omitted. `Error_Exit` was reached via
    // _SEH2_YIELD(goto Error_Exit) → modeled as `break 'try_block`.
    'try_block: {
        hLibModule = LoadLibraryW(lpLibFileName);
        if !hLibModule.is_null() {
            pRegisterClassNameW = core::mem::transmute(GetProcAddress(
                hLibModule,
                b"RegisterClassNameW\0".as_ptr() as LPCSTR,
            ));
            if pRegisterClassNameW.is_some() {
                if IS_ATOM(pszClass as *const c_void) {
                    RtlInitEmptyUnicodeString(
                        &mut ClassName,
                        ClassNameBuf.as_mut_ptr(),
                        core::mem::size_of_val(&ClassNameBuf) as u16,
                    );
                    if NtUserGetAtomName(
                        LOWORD(pszClass as DWORD_PTR as DWORD) as ATOM,
                        &mut ClassName,
                    ) == 0
                    {
                        break 'try_block;
                    }
                    pszClass = ClassName.Buffer;
                }
                Ret = (pRegisterClassNameW.unwrap())(pszClass) as BOOL;
            }
        }
    }

    if Ret != 0 || hLibModule.is_null() {
        if !phLibModule.is_null() {
            *phLibModule = hLibModule;
        }
    } else {
        let dwLastError = GetLastError();
        FreeLibrary(hLibModule);
        SetLastError(dwLastError);
    }
    RtlDeactivateActivationContextUnsafeFast(&mut Frame);
    Ret
}
}

#[no_mangle]
pub unsafe extern "system" fn GetClassInfoExA(
    mut hInstance: HINSTANCE,
    lpszClass: LPCSTR,
    lpwcx: LPWNDCLASSEXA,
) -> BOOL {
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut pszMenuName: LPCSTR = core::ptr::null_mut();
    let mut hLibModule: HMODULE = core::ptr::null_mut();
    let mut dwLastError: DWORD;
    let mut Ret: BOOL;
    let mut ClassFound: BOOL = FALSE as BOOL;
    let mut ConvertedString: BOOL = FALSE as BOOL;
    let lpszClsVersion: LPCWSTR;
    let mut pCtx: HANDLE = core::ptr::null_mut();
    let mut lpLibFileName: LPCWSTR = core::ptr::null_mut();

    if lpwcx.is_null() {
        SetLastError(ERROR_NOACCESS);
        return FALSE as BOOL;
    }

    if hInstance == User32Instance {
        hInstance = core::ptr::null_mut();
    }

    if lpszClass.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE as BOOL;
    }

    lpszClsVersion = ClassNameToVersion(
        lpszClass as *const c_void,
        core::ptr::null(),
        &mut lpLibFileName,
        &mut pCtx,
        TRUE as BOOL,
    );
    if !lpszClsVersion.is_null() {
        RtlInitUnicodeString(&mut ClassName, lpszClsVersion);
    } else if IS_ATOM(lpszClass as *const c_void) {
        ClassName.Buffer = lpszClass as usize as PWSTR;
    } else {
        ConvertedString = TRUE as BOOL;
        if RtlCreateUnicodeStringFromAsciiz(&mut ClassName, lpszClass as PCSZ) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return FALSE as BOOL;
        }
    }

    if RegisterDefaultClasses == 0 {
        RegisterSystemControls();
    }

    loop {
        Ret = NtUserGetClassInfo(
            hInstance,
            &mut ClassName,
            lpwcx as LPWNDCLASSEXW,
            &mut pszMenuName as *mut LPCSTR as *mut LPWSTR,
            TRUE as BOOL,
        );
        if Ret != 0 {
            break;
        }
        if lpLibFileName.is_null() {
            break;
        }
        if ClassFound == 0 {
            dwLastError = GetLastError();
            if dwLastError == ERROR_CANNOT_FIND_WND_CLASS
                || dwLastError == ERROR_CLASS_DOES_NOT_EXIST
            {
                ClassFound = VersionRegisterClass(
                    ClassName.Buffer,
                    lpLibFileName,
                    pCtx,
                    &mut hLibModule,
                );
                if ClassFound != 0 {
                    continue;
                }
            }
        }
        if !hLibModule.is_null() {
            dwLastError = GetLastError();
            FreeLibrary(hLibModule);
            SetLastError(dwLastError);
            hLibModule = core::ptr::null_mut();
        }
        break;
    }

    if Ret != 0 {
        (*lpwcx).lpszClassName = lpszClass;
    }

    if ConvertedString != 0 {
        RtlFreeUnicodeString(&mut ClassName);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetClassInfoExW(
    mut hInstance: HINSTANCE,
    lpszClass: LPCWSTR,
    lpwcx: LPWNDCLASSEXW,
) -> BOOL {
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut pszMenuName: LPWSTR = core::ptr::null_mut();
    let mut hLibModule: HMODULE = core::ptr::null_mut();
    let mut dwLastError: DWORD;
    let mut Ret: BOOL = 0;
    let mut ClassFound: BOOL = 0;
    let lpszClsVersion: LPCWSTR;
    let mut pCtx: HANDLE = core::ptr::null_mut();
    let mut lpLibFileName: LPCWSTR = core::ptr::null_mut();

    if lpwcx.is_null() {
        SetLastError(ERROR_NOACCESS);
        return 0;
    }

    if hInstance == User32Instance {
        hInstance = core::ptr::null_mut();
    }

    if lpszClass.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    lpszClsVersion = ClassNameToVersion(
        lpszClass as *const c_void,
        core::ptr::null_mut(),
        &mut lpLibFileName,
        &mut pCtx,
        0,
    );
    if !lpszClsVersion.is_null() {
        RtlInitUnicodeString(&mut ClassName, lpszClsVersion);
    } else if IS_ATOM(lpszClass as *const c_void) {
        ClassName.Buffer = lpszClass as ULONG_PTR as PWSTR;
    } else {
        RtlInitUnicodeString(&mut ClassName, lpszClass);
    }

    if RegisterDefaultClasses == 0 {
        RegisterSystemControls();
    }

    loop {
        Ret = NtUserGetClassInfo(
            hInstance,
            &mut ClassName,
            lpwcx,
            &mut pszMenuName,
            0,
        );
        if Ret != 0 {
            break;
        }
        if lpLibFileName.is_null() {
            break;
        }
        if ClassFound == 0 {
            dwLastError = GetLastError();
            if dwLastError == ERROR_CANNOT_FIND_WND_CLASS
                || dwLastError == ERROR_CLASS_DOES_NOT_EXIST
            {
                ClassFound = VersionRegisterClass(
                    ClassName.Buffer,
                    lpLibFileName,
                    pCtx,
                    &mut hLibModule,
                );
                if ClassFound != 0 {
                    continue;
                }
            }
        }
        if !hLibModule.is_null() {
            dwLastError = GetLastError();
            FreeLibrary(hLibModule);
            SetLastError(dwLastError);
            hLibModule = core::ptr::null_mut();
        }
        break;
    }

    if Ret != 0 {
        (*lpwcx).lpszClassName = lpszClass;
    }
    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetClassInfoA(
    hInstance: HINSTANCE,
    lpClassName: LPCSTR,
    lpWndClass: LPWNDCLASSA,
) -> BOOL {
    let mut wcex: WNDCLASSEXA = core::mem::zeroed();
    let retval = GetClassInfoExA(hInstance, lpClassName, &mut wcex);
    if retval != 0 {
        (*lpWndClass).style = wcex.style;
        (*lpWndClass).lpfnWndProc = wcex.lpfnWndProc;
        (*lpWndClass).cbClsExtra = wcex.cbClsExtra;
        (*lpWndClass).cbWndExtra = wcex.cbWndExtra;
        (*lpWndClass).hInstance = wcex.hInstance;
        (*lpWndClass).hIcon = wcex.hIcon;
        (*lpWndClass).hCursor = wcex.hCursor;
        (*lpWndClass).hbrBackground = wcex.hbrBackground;
        (*lpWndClass).lpszMenuName = wcex.lpszMenuName;
        (*lpWndClass).lpszClassName = wcex.lpszClassName;
    }
    retval
}

#[no_mangle]
pub unsafe extern "system" fn GetClassInfoW(
    hInstance: HINSTANCE,
    lpClassName: LPCWSTR,
    lpWndClass: LPWNDCLASSW,
) -> BOOL {
    let mut wcex: WNDCLASSEXW = core::mem::zeroed();
    let retval = GetClassInfoExW(hInstance, lpClassName, &mut wcex);
    if retval != 0 {
        (*lpWndClass).style = wcex.style;
        (*lpWndClass).lpfnWndProc = wcex.lpfnWndProc;
        (*lpWndClass).cbClsExtra = wcex.cbClsExtra;
        (*lpWndClass).cbWndExtra = wcex.cbWndExtra;
        (*lpWndClass).hInstance = wcex.hInstance;
        (*lpWndClass).hIcon = wcex.hIcon;
        (*lpWndClass).hCursor = wcex.hCursor;
        (*lpWndClass).hbrBackground = wcex.hbrBackground;
        (*lpWndClass).lpszMenuName = wcex.lpszMenuName;
        (*lpWndClass).lpszClassName = wcex.lpszClassName;
    }
    retval
}

// IntGetClsWndProc — FASTCALL in C, file-internal → non-exported helper.
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetClsWndProc(pWnd: PWND, Class: PCLS, Ansi: i32) -> DWORD_PTR {
    IntGetClsWndProc_impl(pWnd, Class, Ansi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetClsWndProc(pWnd: PWND, Class: PCLS, Ansi: i32) -> DWORD_PTR {
    IntGetClsWndProc_impl(pWnd, Class, Ansi)
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn IntGetClsWndProc_impl(pWnd: PWND, Class: PCLS, Ansi: i32) -> DWORD_PTR {
    let mut i: i32;
    let gcpd: DWORD_PTR;
    let mut Ret: DWORD_PTR = 0;

    // If server side, sweep through proc list and return the client side proc.
    if (*Class).CSF_flags & (CSF_SERVERSIDEPROC as DWORD) != 0 {
        i = FNID_FIRST as i32;
        while i <= FNID_SWITCH as i32 {
            if GETPFNSERVER(i) == (*Class).lpfnWndProc {
                if Ansi != 0 {
                    Ret = core::mem::transmute::<WNDPROC, DWORD_PTR>(GETPFNCLIENTA(i));
                } else {
                    Ret = core::mem::transmute::<WNDPROC, DWORD_PTR>(GETPFNCLIENTW(i));
                }
            }
            i += 1;
        }
        return Ret;
    }

    // Set return proc.
    Ret = core::mem::transmute::<WNDPROC, DWORD_PTR>((*Class).lpfnWndProc);

    // Return the proc if one of the FnId default class type.
    if (*Class).fnid <= FNID_GHOST as DWORD && (*Class).fnid >= FNID_BUTTON as DWORD {
        if Ansi != 0 {
            if GETPFNCLIENTW((*Class).fnid as i32) == (*Class).lpfnWndProc {
                Ret = core::mem::transmute::<WNDPROC, DWORD_PTR>(GETPFNCLIENTA((*Class).fnid as i32));
            }
        } else {
            if GETPFNCLIENTA((*Class).fnid as i32) == (*Class).lpfnWndProc {
                Ret = core::mem::transmute::<WNDPROC, DWORD_PTR>(GETPFNCLIENTW((*Class).fnid as i32));
            }
        }
    }

    // Return on change or Ansi/Unicode proc equal.
    let ansi_proc = if (*Class).CSF_flags & (CSF_ANSIPROC as DWORD) != 0 { 1 } else { 0 };
    if Ret != core::mem::transmute::<WNDPROC, DWORD_PTR>((*Class).lpfnWndProc)
        || (Ansi != 0) as i32 == ansi_proc
    {
        return Ret;
    }

    gcpd = NtUserGetCPD(
        UserHMGetHandle(pWnd) as HWND,
        (if Ansi != 0 { UserGetCPDA2U } else { UserGetCPDU2A }) | UserGetCPDWndtoCls,
        Ret,
    );

    if gcpd != 0 {
        gcpd
    } else {
        Ret
    }
}

// IntGetWndProc — FASTCALL in C, file-internal → non-exported helper.
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetWndProc(pWnd: PWND, Ansi: i32) -> WNDPROC {
    IntGetWndProc_impl(pWnd, Ansi)
}

#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetWndProc(pWnd: PWND, Ansi: i32) -> WNDPROC {
    IntGetWndProc_impl(pWnd, Ansi)
}

#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
unsafe fn IntGetWndProc_impl(pWnd: PWND, Ansi: i32) -> WNDPROC {
    let mut i: i32;
    let gcpd: WNDPROC;
    let mut Ret: WNDPROC = None;
    let Class: PCLS = crate::wow64::desktop_cls_to_user((*pWnd).pcls as PVOID);

    if Class.is_null() {
        return Ret;
    }

    if (*pWnd).state & WNDS_SERVERSIDEWINDOWPROC != 0 {
        i = FNID_FIRST as i32;
        while i <= FNID_SWITCH as i32 {
            if GETPFNSERVER(i) == (*pWnd).lpfnWndProc {
                if Ansi != 0 {
                    Ret = GETPFNCLIENTA(i);
                } else {
                    Ret = GETPFNCLIENTW(i);
                }
            }
            i += 1;
        }
        return Ret;
    }

    if (*Class).fnid == FNID_EDIT as DWORD {
        Ret = (*pWnd).lpfnWndProc;
    } else {
        Ret = (*pWnd).lpfnWndProc;

        if (*Class).fnid <= FNID_GHOST as DWORD && (*Class).fnid >= FNID_BUTTON as DWORD {
            if Ansi != 0 {
                if GETPFNCLIENTW((*Class).fnid as i32) == (*pWnd).lpfnWndProc {
                    Ret = GETPFNCLIENTA((*Class).fnid as i32);
                }
            } else {
                if GETPFNCLIENTA((*Class).fnid as i32) == (*pWnd).lpfnWndProc {
                    Ret = GETPFNCLIENTW((*Class).fnid as i32);
                }
            }
        }

        if Ret != (*pWnd).lpfnWndProc {
            return Ret;
        }
    }

    let ansi_flag = ((*pWnd).state & WNDS_ANSIWINDOWPROC != 0) as i32;
    if (Ansi != 0) as i32 == ansi_flag {
        return Ret;
    }

    gcpd = core::mem::transmute::<DWORD_PTR, WNDPROC>(NtUserGetCPD(
        UserHMGetHandle(pWnd) as HWND,
        (if Ansi != 0 { UserGetCPDA2U } else { UserGetCPDU2A }) | UserGetCPDWindow,
        Ret.map_or(0, |f| f as usize as DWORD_PTR),
    ));

    if gcpd.is_some() {
        gcpd
    } else {
        Ret
    }
}

// IntGetClassLongA/W — FASTCALL, static in C → non-exported helpers. They are
// referenced from GetClassLong* (all arches) so they are not x86-only.
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetClassLongA(Wnd: PWND, Class: PCLS, nIndex: i32) -> usize {
    IntGetClassLongA_impl(Wnd, Class, nIndex)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetClassLongA(Wnd: PWND, Class: PCLS, nIndex: i32) -> usize {
    IntGetClassLongA_impl(Wnd, Class, nIndex)
}

unsafe fn IntGetClassLongA_impl(Wnd: PWND, Class: PCLS, nIndex: i32) -> usize {
    let mut Ret: usize = 0;

    if nIndex >= 0 {
        let nIndex_u = nIndex as usize;
        if nIndex_u.wrapping_add(core::mem::size_of::<usize>()) < nIndex_u
            || nIndex_u.wrapping_add(core::mem::size_of::<usize>()) > (*Class).cbclsExtra as usize
        {
            SetLastError(ERROR_INVALID_PARAMETER);
        } else {
            /* NOT `Class.add(1)`: under WoW64 `Class` is a normalized 32-bit
             * COPY of the real 64-bit CLS and the extra bytes follow the
             * ORIGINAL. cls_extra_base() resolves both cases -- same shape as
             * wnd_extra_base() beside it. */
            let ExtraBase = crate::wow64::cls_extra_base(Class);
            if ExtraBase.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
            } else {
                Ret = *(ExtraBase.add(nIndex_u) as PULONG_PTR) as usize;
            }
        }
    } else {
        match nIndex {
            GCL_CBWNDEXTRA => {
                Ret = (*Class).cbwndExtra as usize;
            }
            GCL_CBCLSEXTRA => {
                Ret = (*Class).cbclsExtra as usize;
            }
            GCL_HBRBACKGROUND => {
                Ret = (*Class).hbrBackground as usize;
                if Ret != 0 && Ret < 0x4000 {
                    Ret = GetSysColorBrush((Ret as u32 - 1) as core::ffi::c_int) as usize;
                }
            }
            GCL_HMODULE => {
                Ret = (*Class).hModule as usize;
            }
            GCL_MENUNAME => {
                Ret = (*Class).lpszClientAnsiMenuName as usize;
            }
            GCL_STYLE => {
                Ret = (*Class).style as usize;
            }
            GCW_ATOM => {
                Ret = (*Class).atomNVClassName as usize;
            }
            GCLP_HCURSOR => {
                Ret = if !(*Class).spcur.is_null() {
                    ProcMarkHeadHandle(SharedPtrToUser((*Class).spcur as PVOID) as PPROCMARKHEAD) as usize
                } else {
                    0
                };
            }
            GCLP_HICON => {
                Ret = if !(*Class).spicn.is_null() {
                    ProcMarkHeadHandle(SharedPtrToUser((*Class).spicn as PVOID) as PPROCMARKHEAD) as usize
                } else {
                    0
                };
            }
            GCLP_HICONSM => {
                Ret = if !(*Class).spicnSm.is_null() {
                    ProcMarkHeadHandle(SharedPtrToUser((*Class).spicnSm as PVOID) as PPROCMARKHEAD) as usize
                } else {
                    0
                };
            }
            GCLP_WNDPROC => {
                Ret = IntGetClsWndProc(Wnd, Class, 1) as usize;
            }
            _ => {
                SetLastError(ERROR_INVALID_INDEX);
            }
        }
    }

    Ret
}

#[cfg(target_arch = "x86")]
unsafe extern "fastcall" fn IntGetClassLongW(Wnd: PWND, Class: PCLS, nIndex: i32) -> usize {
    IntGetClassLongW_impl(Wnd, Class, nIndex)
}
#[cfg(not(target_arch = "x86"))]
unsafe extern "C" fn IntGetClassLongW(Wnd: PWND, Class: PCLS, nIndex: i32) -> usize {
    IntGetClassLongW_impl(Wnd, Class, nIndex)
}

unsafe fn IntGetClassLongW_impl(Wnd: PWND, Class: PCLS, nIndex: i32) -> usize {
    let mut Ret: usize = 0;

    if nIndex >= 0 {
        let nIndex_u = nIndex as usize;
        if nIndex_u.wrapping_add(core::mem::size_of::<usize>()) < nIndex_u
            || nIndex_u + core::mem::size_of::<usize>() > (*Class).cbclsExtra as usize
        {
            SetLastError(ERROR_INVALID_PARAMETER);
        } else {
            /* NOT `Class.add(1)`: under WoW64 `Class` is a normalized 32-bit
             * COPY of the real 64-bit CLS and the extra bytes follow the
             * ORIGINAL. cls_extra_base() resolves both cases -- same shape as
             * wnd_extra_base() beside it. */
            let ExtraBase = crate::wow64::cls_extra_base(Class);
            if ExtraBase.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER);
            } else {
                Ret = *(ExtraBase.add(nIndex_u) as PULONG_PTR) as usize;
            }
        }
    } else {
        match nIndex {
            GCL_CBWNDEXTRA => {
                Ret = (*Class).cbwndExtra as usize;
            }
            GCL_CBCLSEXTRA => {
                Ret = (*Class).cbclsExtra as usize;
            }
            GCLP_HBRBACKGROUND => {
                Ret = (*Class).hbrBackground as usize;
                if Ret != 0 && Ret < 0x4000 {
                    Ret = GetSysColorBrush((Ret as u32 - 1) as core::ffi::c_int) as usize;
                }
            }
            GCL_HMODULE => {
                Ret = (*Class).hModule as usize;
            }
            GCLP_MENUNAME => {
                Ret = (*Class).lpszClientUnicodeMenuName as usize;
            }
            GCL_STYLE => {
                Ret = (*Class).style as usize;
            }
            GCW_ATOM => {
                Ret = (*Class).atomNVClassName as usize;
            }
            GCLP_HCURSOR => {
                Ret = if !(*Class).spcur.is_null() {
                    ProcMarkHeadHandle(SharedPtrToUser((*Class).spcur as PVOID) as PPROCMARKHEAD) as usize
                } else {
                    0
                };
            }
            GCLP_HICON => {
                Ret = if !(*Class).spicn.is_null() {
                    ProcMarkHeadHandle(SharedPtrToUser((*Class).spicn as PVOID) as PPROCMARKHEAD) as usize
                } else {
                    0
                };
            }
            GCLP_HICONSM => {
                Ret = if !(*Class).spicnSm.is_null() {
                    ProcMarkHeadHandle(SharedPtrToUser((*Class).spicnSm as PVOID) as PPROCMARKHEAD) as usize
                } else {
                    0
                };
            }
            GCLP_WNDPROC => {
                Ret = IntGetClsWndProc(Wnd, Class, 0) as usize;
            }
            _ => {
                SetLastError(ERROR_INVALID_INDEX);
            }
        }
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn GetClassLongA(hWnd: HWND, nIndex: i32) -> u32 {
    let mut Ret: usize = 0;

    let Wnd = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    // _SEH2_TRY body — except handler (Ret = 0) is DEAD CODE under USE_DUMMY_PSEH.
    let Class = crate::wow64::desktop_cls_to_user((*Wnd).pcls as PVOID);
    if !Class.is_null() {
        // class.c: `#ifdef _WIN64` — pointer-sized, so arm64 too, not just amd64.
        #[cfg(target_pointer_width = "64")]
        {
            match nIndex {
                GCLP_HBRBACKGROUND | GCLP_HCURSOR | GCLP_HICON | GCLP_HICONSM
                | GCLP_HMODULE | GCLP_MENUNAME | GCLP_WNDPROC => {
                    SetLastError(ERROR_INVALID_INDEX);
                }
                _ => {
                    Ret = IntGetClassLongA(Wnd, Class, nIndex);
                }
            }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            Ret = IntGetClassLongA(Wnd, Class, nIndex);
        }
    }

    Ret as u32
}

#[no_mangle]
pub unsafe extern "system" fn GetClassLongW(hWnd: HWND, nIndex: i32) -> u32 {
    let Wnd: PWND;
    let Class: PCLS;
    let mut Ret: usize = 0;

    Wnd = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    Class = crate::wow64::desktop_cls_to_user((*Wnd).pcls as PVOID);
    if !Class.is_null() {
        // class.c: `#ifdef _WIN64` — pointer-sized, so arm64 too, not just amd64.
        #[cfg(target_pointer_width = "64")]
        {
            match nIndex {
                GCLP_HBRBACKGROUND | GCLP_HCURSOR | GCLP_HICON | GCLP_HICONSM
                | GCLP_HMODULE | GCLP_MENUNAME | GCLP_WNDPROC => {
                    SetLastError(ERROR_INVALID_INDEX);
                }
                _ => {
                    Ret = IntGetClassLongW(Wnd, Class, nIndex);
                }
            }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            Ret = IntGetClassLongW(Wnd, Class, nIndex);
        }
    }

    Ret as u32
}

// GetClassLongPtrA/W — _WIN64 only in C (#ifdef _WIN64) → non-x86 only.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn GetClassLongPtrA(hWnd: HWND, nIndex: i32) -> ULONG_PTR {
    let mut Ret: usize = 0;

    let Wnd: PWND = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    let Class: PCLS = crate::wow64::desktop_cls_to_user((*Wnd).pcls as PVOID);
    if !Class.is_null() {
        Ret = IntGetClassLongA(Wnd, Class, nIndex);
    }

    Ret as ULONG_PTR
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn GetClassLongPtrW(hWnd: HWND, nIndex: i32) -> ULONG_PTR {
    let Wnd: PWND;
    let Class: PCLS;
    let mut Ret: usize = 0;

    Wnd = ValidateHwnd(hWnd);
    if Wnd.is_null() {
        return 0;
    }

    Class = crate::wow64::desktop_cls_to_user((*Wnd).pcls as PVOID);
    if !Class.is_null() {
        Ret = IntGetClassLongW(Wnd, Class, nIndex);
    }

    Ret as ULONG_PTR
}

// On i386 winuser.h defines `GetClassLongPtrW`/`A` as the macro `GetClassLongW`/`A`
// (faithful to the C macro rewrite). Not a public spec export on i386, but
// cross-TU callers (defwnd.rs) reference `_GetClassLongPtrW@8`, so emit a real
// forwarding export rather than an inline alias.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn GetClassLongPtrW(hWnd: HWND, nIndex: i32) -> ULONG_PTR {
    GetClassLongW(hWnd, nIndex) as ULONG_PTR
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn GetClassLongPtrA(hWnd: HWND, nIndex: i32) -> ULONG_PTR {
    GetClassLongA(hWnd, nIndex) as ULONG_PTR
}

#[no_mangle]
pub unsafe extern "system" fn GetClassNameA(
    hWnd: HWND,
    lpClassName: LPSTR,
    nMaxCount: i32,
) -> i32 {
    let mut tmpbuf: [u16; MAX_ATOM_LEN as usize + 1] = [0; MAX_ATOM_LEN as usize + 1];
    let mut len: i32 = 0;

    if nMaxCount <= 0 {
        return 0;
    }
    if GetClassNameW(
        hWnd,
        tmpbuf.as_mut_ptr(),
        tmpbuf.len() as i32,
    ) == 0
    {
        return 0;
    }
    RtlUnicodeToMultiByteN(
        lpClassName as *mut _,
        (nMaxCount - 1) as u32,
        core::ptr::addr_of_mut!(len) as PULONG,
        tmpbuf.as_ptr() as *mut _,
        (strlenW(tmpbuf.as_ptr()) * core::mem::size_of::<u16>()) as u32,
    );
    *lpClassName.offset(len as isize) = 0;

    len
}

#[no_mangle]
pub unsafe extern "system" fn GetClassNameW(
    hWnd: HWND,
    lpClassName: LPWSTR,
    nMaxCount: i32,
) -> i32 {
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitEmptyUnicodeString(
        &mut ClassName,
        lpClassName,
        (nMaxCount as usize * core::mem::size_of::<u16>()) as u16,
    );

    NtUserGetClassName(hWnd, 0u8 as BOOL, &mut ClassName)
}

#[no_mangle]
pub unsafe extern "system" fn GetClassWord(hwnd: HWND, offset: i32) -> u16 {
    let Wnd: PWND;
    let class: PCLS;
    let mut retvalue: u16 = 0;

    if offset < 0 {
        return GetClassLongA(hwnd, offset) as u16;
    }

    Wnd = ValidateHwnd(hwnd);
    if Wnd.is_null() {
        return 0;
    }

    class = crate::wow64::desktop_cls_to_user((*Wnd).pcls as PVOID);
    if class.is_null() {
        return 0;
    }

    if offset <= (*class).cbclsExtra - core::mem::size_of::<u16>() as i32 {
        memcpy(
            core::ptr::addr_of_mut!(retvalue) as *mut c_void,
            ((class.add(1) as *mut i8).offset(offset as isize)) as *const c_void,
            core::mem::size_of::<u16>(),
        );
    } else {
        SetLastError(ERROR_INVALID_INDEX);
    }

    retvalue
}

unsafe fn IntGetWindowLong(hwnd: HWND, offset: i32, size: u32, unicode: i32) -> isize {
    let mut retvalue: isize = 0;
    let wndPtr: *mut WND;

    const PUBLIC_EXSTYLE: u32 = 0x08000000 | 0x02000000 | 0x00400000 | 0x00100000
        | 0x00080000 | 0x00040000 | 0x00020000 | 0x00010000
        | 0x00004000 | 0x00002000 | 0x00001000 | 0x00000400
        | 0x00000200 | 0x00000100 | 0x00000080 | 0x00000040
        | 0x00000020 | 0x00000010 | 0x00000008 | 0x00000004
        | 0x00000002 | 0x00000001;

    if offset == GWLP_HWNDPARENT {
        let mut parent: HWND = GetAncestor(hwnd, GA_PARENT as UINT);
        if parent == GetDesktopWindow() {
            parent = GetWindow(hwnd, GW_OWNER as UINT);
        }
        return parent as isize;
    }

    wndPtr = ValidateHwnd(hwnd);
    if wndPtr.is_null() {
        SetLastError(ERROR_INVALID_WINDOW_HANDLE);
        return 0;
    }

    if offset >= 0 && (*wndPtr).fnid != FNID_DESKTOP as DWORD {
        if offset > ((*wndPtr).cbwndExtra as i32 - size as i32) {
            SetLastError(ERROR_INVALID_INDEX);
            return 0;
        }
        /* NOT `wndPtr.add(1)`: under WoW64 wndPtr is a normalized 32-bit COPY of
         * the real 64-bit WND, and the extra bytes live after the ORIGINAL.
         * wnd_extra_base() resolves both cases. */
        let ExtraBase = crate::wow64::wnd_extra_base(wndPtr);
        if ExtraBase.is_null() {
            SetLastError(ERROR_INVALID_INDEX);
            return 0;
        }
        retvalue = *(ExtraBase.add(offset as usize) as *mut isize);
        return retvalue;
    }

    match offset {
        x if x == GWLP_USERDATA => retvalue = (*wndPtr).dwUserData as isize,
        x if x == GWL_STYLE => retvalue = (*wndPtr).style as isize,
        x if x == GWL_EXSTYLE => retvalue = ((*wndPtr).ExStyle & PUBLIC_EXSTYLE) as isize,
        x if x == GWLP_ID => retvalue = (*wndPtr).IDMenu as isize,
        x if x == GWLP_HINSTANCE => retvalue = (*wndPtr).hModule as isize,
        -1 => retvalue = core::ptr::addr_of_mut!((*wndPtr).state) as isize,
        x if x == GWLP_WNDPROC => {
            if TestWindowProcess(wndPtr) == 0 {
                SetLastError(ERROR_ACCESS_DENIED);
                retvalue = 0;
            } else {
                retvalue = IntGetWndProc(wndPtr, (unicode == 0) as i32)
                    .map_or(0, |f| f as usize as isize);
            }
        }
        _ => {
            SetLastError(ERROR_INVALID_INDEX);
        }
    }
    retvalue
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowLongA(hWnd: HWND, nIndex: i32) -> i32 {
    IntGetWindowLong(hWnd, nIndex, core::mem::size_of::<i32>() as u32, 0) as i32
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowLongW(hWnd: HWND, nIndex: i32) -> i32 {
    IntGetWindowLong(hWnd, nIndex, core::mem::size_of::<i32>() as u32, 1) as i32
}

// GetWindowLongPtrA/W — _WIN64 only in C → non-x86 only.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn GetWindowLongPtrA(hWnd: HWND, nIndex: i32) -> LONG_PTR {
    IntGetWindowLong(hWnd, nIndex, core::mem::size_of::<isize>() as u32, 0) as LONG_PTR
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> LONG_PTR {
    IntGetWindowLong(hWnd, nIndex, core::mem::size_of::<isize>() as u32, 1) as LONG_PTR
}

// On i386 (non-_WIN64) winuser.h defines `GetWindowLongPtrW` as the macro
// `GetWindowLongW` (and likewise for A), faithful to the C macro rewrite. The
// spec gates the public Ptr exports to x86_64/arm64, but translated cross-TU
// callers (window.rs, defwnd.rs) reference the decorated `_GetWindowLongPtrW@8`
// symbol, so emit a real forwarding export (not #[inline]).
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> LONG_PTR {
    GetWindowLongW(hWnd, nIndex) as LONG_PTR
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn GetWindowLongPtrA(hWnd: HWND, nIndex: i32) -> LONG_PTR {
    GetWindowLongA(hWnd, nIndex) as LONG_PTR
}

#[no_mangle]
pub unsafe extern "system" fn GetWindowWord(hWnd: HWND, nIndex: core::ffi::c_int) -> u16 {
    match nIndex as isize {
        x if x == GWLP_ID as isize
            || x == GWLP_HINSTANCE as isize
            || x == GWLP_HWNDPARENT as isize => {}
        _ => {
            if nIndex < 0 {
                SetLastError(ERROR_INVALID_INDEX);
                return 0;
            }
        }
    }
    IntGetWindowLong(hWnd, nIndex, core::mem::size_of::<u16>() as u32, 0) as u16
}

#[no_mangle]
pub unsafe extern "system" fn RealGetWindowClassW(
    hwnd: HWND,
    pszType: LPWSTR,
    cchType: UINT,
) -> UINT {
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();

    RtlInitEmptyUnicodeString(
        &mut ClassName,
        pszType,
        (cchType as usize * core::mem::size_of::<u16>()) as USHORT,
    );

    NtUserGetClassName(hwnd, 1u8 as BOOL, &mut ClassName) as UINT
}

#[no_mangle]
pub unsafe extern "system" fn RealGetWindowClassA(
    hwnd: HWND,
    pszType: LPSTR,
    cchType: UINT,
) -> UINT {
    let mut tmpbuf: [u16; MAX_ATOM_LEN as usize + 1] = [0; MAX_ATOM_LEN as usize + 1];
    let mut len: UINT = 0;

    if (cchType as i32) <= 0 {
        return 0;
    }
    if RealGetWindowClassW(hwnd, tmpbuf.as_mut_ptr(), tmpbuf.len() as UINT) == 0 {
        return 0;
    }
    RtlUnicodeToMultiByteN(
        pszType,
        cchType - 1,
        core::ptr::addr_of_mut!(len) as PULONG,
        tmpbuf.as_mut_ptr(),
        (strlenW(tmpbuf.as_ptr()) * core::mem::size_of::<u16>()) as ULONG,
    );
    *pszType.add(len as usize) = 0;
    len
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClassExWOWW(
    lpwcx: *mut WNDCLASSEXW,
    pdwWowData: LPDWORD,
    fnID: u16,
    dwFlags: u32,
    ChkRegCls: i32,
) -> ATOM {
    let Atom: ATOM;
    let mut WndClass: WNDCLASSEXW = core::mem::zeroed();
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let mut ClassVersion: _UNICODE_STRING;
    let mut MenuName: _UNICODE_STRING = core::mem::zeroed();
    let mut clsMenuName: CLSMENUNAME = core::mem::zeroed();
    let mut AnsiMenuName: ANSI_STRING = core::mem::zeroed();
    let lpszClsVersion: LPCWSTR;

    if lpwcx.is_null()
        || (*lpwcx).cbSize as usize != core::mem::size_of::<WNDCLASSEXW>()
        || (*lpwcx).cbClsExtra < 0
        || (*lpwcx).cbWndExtra < 0
        || (*lpwcx).lpszClassName.is_null()
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if ChkRegCls != 0 {
        if RegisterDefaultClasses == 0 {
            RegisterSystemControls();
        }
    }

    if (*lpwcx).hInstance == User32Instance {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if (*lpwcx).hInstance.is_null() {
        (*lpwcx).hInstance = GetModuleHandleW(core::ptr::null_mut());
    }

    core::ptr::copy_nonoverlapping(lpwcx, &mut WndClass, 1);

    RtlInitEmptyAnsiString(&mut AnsiMenuName, core::ptr::null_mut(), 0);
    if !IS_INTRESOURCE(WndClass.lpszMenuName as *const c_void) {
        if *WndClass.lpszMenuName != 0 {
            RtlInitUnicodeString(&mut MenuName, WndClass.lpszMenuName);
            RtlUnicodeStringToAnsiString(&mut AnsiMenuName, &mut MenuName, 1u8);
        }
    } else {
        MenuName.Buffer = WndClass.lpszMenuName as LPWSTR;
        AnsiMenuName.Buffer = WndClass.lpszMenuName as PSTR;
    }

    if !WndClass.lpszClassName.is_null()
        && !IS_ATOM(WndClass.lpszClassName as *const c_void)
    {
        RtlInitUnicodeString(&mut ClassName, WndClass.lpszClassName);
    } else {
        ClassName.Length = 0;
        ClassName.MaximumLength = 0;
        ClassName.Buffer = WndClass.lpszClassName as LPWSTR;
    }

    ClassVersion = ClassName;
    if fnID == 0 {
        lpszClsVersion = ClassNameToVersion(
            (*lpwcx).lpszClassName as *const c_void,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            0,
        );
        if !lpszClsVersion.is_null() {
            RtlInitUnicodeString(&mut ClassVersion, lpszClsVersion);
        }
    }

    clsMenuName.pszClientAnsiMenuName = AnsiMenuName.Buffer;
    clsMenuName.pwszClientUnicodeMenuName = MenuName.Buffer;
    clsMenuName.pusMenuName = &mut MenuName;

    Atom = NtUserRegisterClassExWOW(
        &mut WndClass,
        &mut ClassName,
        &mut ClassVersion,
        &mut clsMenuName,
        fnID as DWORD,
        dwFlags,
        pdwWowData,
    ) as ATOM;

    Atom
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClassExA(lpwcx: *const WNDCLASSEXA) -> ATOM {
    let mut WndClass: WNDCLASSEXW = core::mem::zeroed();
    let mut mname: [u16; MAX_BUFFER_LEN] = [0; MAX_BUFFER_LEN];
    let mut cname: [u16; MAX_BUFFER_LEN] = [0; MAX_BUFFER_LEN];

    core::ptr::copy_nonoverlapping(
        lpwcx as *const u8,
        core::ptr::addr_of_mut!(WndClass) as *mut u8,
        core::mem::size_of::<WNDCLASSEXA>(),
    );

    if !WndClass.lpszMenuName.is_null() && !IS_INTRESOURCE(WndClass.lpszMenuName as *const c_void) {
        if *WndClass.lpszMenuName != 0 {
            if MultiByteToWideChar(
                CP_ACP,
                0,
                (*lpwcx).lpszMenuName,
                -1,
                mname.as_mut_ptr(),
                MAX_ATOM_LEN + 1,
            ) == 0
            {
                return 0;
            }

            WndClass.lpszMenuName = mname.as_ptr();
        }
    }

    if !WndClass.lpszClassName.is_null() && !IS_ATOM(WndClass.lpszClassName as *const c_void) {
        if MultiByteToWideChar(
            CP_ACP,
            0,
            (*lpwcx).lpszClassName,
            -1,
            cname.as_mut_ptr(),
            MAX_ATOM_LEN + 1,
        ) == 0
        {
            return 0;
        }

        WndClass.lpszClassName = cname.as_ptr();
    }

    let Atom = RegisterClassExWOWW(
        core::ptr::addr_of_mut!(WndClass),
        core::ptr::null_mut(),
        0,
        CSF_ANSIPROC,
        1,
    );

    Atom
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClassExW(lpwcx: *const WNDCLASSEXW) -> ATOM {
    let Atom = RegisterClassExWOWW(lpwcx as *mut WNDCLASSEXW, core::ptr::null_mut(), 0, 0, 1);

    Atom
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClassA(lpWndClass: *const WNDCLASSA) -> ATOM {
    let mut Class: WNDCLASSEXA = core::mem::zeroed();

    if lpWndClass.is_null() {
        return 0;
    }

    Class.style = (*lpWndClass).style;
    Class.lpfnWndProc = (*lpWndClass).lpfnWndProc;
    Class.cbClsExtra = (*lpWndClass).cbClsExtra;
    Class.cbWndExtra = (*lpWndClass).cbWndExtra;
    Class.hInstance = (*lpWndClass).hInstance;
    Class.hIcon = (*lpWndClass).hIcon;
    Class.hCursor = (*lpWndClass).hCursor;
    Class.hbrBackground = (*lpWndClass).hbrBackground;
    Class.lpszMenuName = (*lpWndClass).lpszMenuName;
    Class.lpszClassName = (*lpWndClass).lpszClassName;

    Class.cbSize = core::mem::size_of::<WNDCLASSEXA>() as u32;
    Class.hIconSm = core::ptr::null_mut();

    RegisterClassExA(&Class)
}

#[no_mangle]
pub unsafe extern "system" fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> ATOM {
    let mut Class: WNDCLASSEXW = core::mem::zeroed();

    if lpWndClass.is_null() {
        return 0;
    }

    Class.style = (*lpWndClass).style;
    Class.lpfnWndProc = (*lpWndClass).lpfnWndProc;
    Class.cbClsExtra = (*lpWndClass).cbClsExtra;
    Class.cbWndExtra = (*lpWndClass).cbWndExtra;
    Class.hInstance = (*lpWndClass).hInstance;
    Class.hIcon = (*lpWndClass).hIcon;
    Class.hCursor = (*lpWndClass).hCursor;
    Class.hbrBackground = (*lpWndClass).hbrBackground;
    Class.lpszMenuName = (*lpWndClass).lpszMenuName;
    Class.lpszClassName = (*lpWndClass).lpszClassName;

    Class.cbSize = core::mem::size_of::<WNDCLASSEXW>() as u32;
    Class.hIconSm = core::ptr::null_mut();

    RegisterClassExW(&Class)
}

#[no_mangle]
pub unsafe extern "system" fn SetClassLongA(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    mut dwNewLong: LONG,
) -> DWORD {
    let lpStr: PSTR = dwNewLong as ULONG_PTR as PSTR;
    let mut Value: _UNICODE_STRING = core::mem::zeroed();
    let mut Allocated: BOOL = 0;
    let Ret: DWORD;

    if nIndex == GCL_MENUNAME as core::ffi::c_int && !lpStr.is_null() {
        if !IS_INTRESOURCE(lpStr as *const c_void) {
            if RtlCreateUnicodeStringFromAsciiz(&mut Value, lpStr as PCSZ) == 0 {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return 0;
            }
            Allocated = 1;
        } else {
            Value.Buffer = lpStr as PWSTR;
        }
        dwNewLong = &mut Value as *mut _UNICODE_STRING as LONG_PTR as LONG;
    } else if nIndex == GCW_ATOM as core::ffi::c_int && !lpStr.is_null() {
        if !IS_ATOM(lpStr as *const c_void) {
            if RtlCreateUnicodeStringFromAsciiz(&mut Value, lpStr as PCSZ) == 0 {
                SetLastError(ERROR_NOT_ENOUGH_MEMORY);
                return 0;
            }
            Allocated = 1;
        } else {
            Value.Buffer = lpStr as PWSTR;
        }
        dwNewLong = &mut Value as *mut _UNICODE_STRING as LONG_PTR as LONG;
    }

    Ret = NtUserSetClassLong(hWnd, nIndex, dwNewLong as ULONG, 1) as DWORD;

    if Allocated != 0 {
        RtlFreeUnicodeString(&mut Value);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn SetClassLongW(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    mut dwNewLong: LONG,
) -> DWORD {
    let lpStr: PWSTR = dwNewLong as ULONG_PTR as PWSTR;
    let mut Value: _UNICODE_STRING = core::mem::zeroed();

    if nIndex == GCL_MENUNAME as core::ffi::c_int && !lpStr.is_null() {
        if !IS_INTRESOURCE(lpStr as *const c_void) {
            RtlInitUnicodeString(&mut Value, lpStr);
        } else {
            Value.Buffer = lpStr;
        }

        dwNewLong = (&mut Value as *mut _UNICODE_STRING) as LONG_PTR as LONG;
    } else if nIndex == GCW_ATOM as core::ffi::c_int && !lpStr.is_null() {
        if !IS_ATOM(lpStr as *const c_void) {
            RtlInitUnicodeString(&mut Value, lpStr);
        } else {
            Value.Buffer = lpStr;
        }

        dwNewLong = (&mut Value as *mut _UNICODE_STRING) as LONG_PTR as LONG;
    }

    NtUserSetClassLong(hWnd, nIndex, dwNewLong as ULONG, 0u8 as BOOL) as DWORD
}

// SetClassLongPtrA/W — _WIN64 only in C → non-x86 only.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn SetClassLongPtrA(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    dwNewLong: isize,
) -> usize {
    NtUserSetClassLongPtr(hWnd, nIndex, dwNewLong as ULONG_PTR as _, 1u8 as BOOL) as usize
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn SetClassLongPtrW(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    dwNewLong: isize,
) -> usize {
    NtUserSetClassLongPtr(hWnd, nIndex, dwNewLong as ULONG_PTR as _, 0u8 as BOOL) as usize
}

#[no_mangle]
pub unsafe extern "system" fn SetClassWord(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    wNewWord: u16,
) -> u16 {
    if (nIndex < 0) && (nIndex != GCW_ATOM as core::ffi::c_int) {
        return 0;
    }

    SetClassLongW(hWnd, nIndex, wNewWord as LONG) as u16
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowWord(hWnd: HWND, nIndex: core::ffi::c_int, wNewWord: u16) -> u16 {
    match nIndex as i64 {
        x if x == GWLP_ID as i64 => {}
        x if x == GWLP_HINSTANCE as i64 => {}
        x if x == GWLP_HWNDPARENT as i64 => {}
        _ => {
            if nIndex < 0 {
                SetLastError(ERROR_INVALID_INDEX);
                return 0;
            }
        }
    }
    /* DO NOT USE NtUserSetWindowLong(Ptr)! */
    NtUserSetWindowWord(hWnd, nIndex, wNewWord)
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowLongA(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    dwNewLong: core::ffi::c_long,
) -> core::ffi::c_long {
    NtUserSetWindowLong(hWnd, nIndex as DWORD, dwNewLong, 1u8 as BOOL)
}

#[no_mangle]
pub unsafe extern "system" fn SetWindowLongW(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    dwNewLong: core::ffi::c_long,
) -> core::ffi::c_long {
    NtUserSetWindowLong(hWnd, nIndex as DWORD, dwNewLong, 0u8 as BOOL)
}

// SetWindowLongPtrA/W — _WIN64 only in C → non-x86 only.
#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn SetWindowLongPtrA(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    dwNewLong: isize,
) -> isize {
    NtUserSetWindowLongPtr(hWnd, nIndex as DWORD, dwNewLong as _, 1u8 as BOOL) as isize
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "system" fn SetWindowLongPtrW(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    dwNewLong: isize,
) -> isize {
    NtUserSetWindowLongPtr(hWnd, nIndex as DWORD, dwNewLong as _, 0u8 as BOOL) as isize
}

// On i386 SetWindowLongPtrW is a macro alias for SetWindowLongW (winuser.h). One real
// export forwarder satisfies the cross-TU `extern { fn SetWindowLongPtrW }` callers.
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "system" fn SetWindowLongPtrW(
    hWnd: HWND,
    nIndex: core::ffi::c_int,
    dwNewLong: isize,
) -> isize {
    SetWindowLongW(hWnd, nIndex, dwNewLong as core::ffi::c_long) as isize
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterClassA(
    lpClassName: LPCSTR,
    hInstance: HINSTANCE,
) -> i32 {
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let Ret: i32;
    let lpszClsVersion: LPCWSTR;
    let mut ConvertedString: i32 = 0;

    lpszClsVersion = ClassNameToVersion(
        lpClassName as *const c_void,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        1,
    );
    if !lpszClsVersion.is_null() {
        RtlInitUnicodeString(&mut ClassName, lpszClsVersion);
    } else if !IS_ATOM(lpClassName as *const c_void) {
        ConvertedString = 1;
        if RtlCreateUnicodeStringFromAsciiz(&mut ClassName, lpClassName as PCSZ) == 0 {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY);
            return 0;
        }
    } else {
        ClassName.Buffer = lpClassName as usize as PWSTR;
    }

    Ret = NtUserUnregisterClass(&mut ClassName, hInstance, core::ptr::null_mut());

    if ConvertedString != 0 {
        RtlFreeUnicodeString(&mut ClassName);
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterClassW(
    lpClassName: LPCWSTR,
    hInstance: HINSTANCE,
) -> i32 {
    let mut ClassName: _UNICODE_STRING = core::mem::zeroed();
    let lpszClsVersion: LPCWSTR;

    lpszClsVersion = ClassNameToVersion(
        lpClassName as *const c_void,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        0,
    );
    if !lpszClsVersion.is_null() {
        RtlInitUnicodeString(&mut ClassName, lpszClsVersion);
    } else if !IS_ATOM(lpClassName as *const c_void) {
        RtlInitUnicodeString(&mut ClassName, lpClassName);
    } else {
        ClassName.Buffer = lpClassName as PWSTR;
    }

    NtUserUnregisterClass(&mut ClassName, hInstance, core::ptr::null_mut())
}
