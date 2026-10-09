//! mmsys-rust prelude — the single import surface the translated modules use.
//! Bridges the bindgen `types` island and the translated C. Anything bindgen leaves out
//! (CRT, inlines, sibling-C externs, hand-transcribed consts) is declared here ONCE.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

pub use crate::types::*;
pub use core::ffi::{c_char, c_int, c_long, c_short, c_uchar, c_uint, c_ushort, c_void};
// C's `size_t`: bindgen spells it `usize` inline, so no island carries the name the C uses.
pub type size_t = usize;
pub use core::ptr::{addr_of, addr_of_mut};

// ================= debug ======================================================================
// reactos/debug.h DPRINT/UNIMPLEMENTED — no-op MACROS, never out-of-line fns
// (field manual §9.65: an out-of-line no-op is an optimization barrier). debug_macros.py
// rewrites C's bare `UNIMPLEMENTED;` to `UNIMPLEMENTED!()`, so the prelude must have it.
// DPRINT1 PRINTS in this DBG=1 tree, kernel and user mode alike: it comes from
// win32macros-rust with the other macros.
#[macro_export] macro_rules! DPRINT  { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED { ($($arg:tt)*) => {{}}; }
#[macro_export] macro_rules! UNIMPLEMENTED_ONCE { ($($arg:tt)*) => {{}}; }
pub use crate::{DPRINT, UNIMPLEMENTED, UNIMPLEMENTED_ONCE};

// ================= CRT ========================================================================
// NEVER hand-write memcpy/memset/memmove/memcmp in Rust: LLVM folds any byte-copy loop back into
// a call to the function itself → infinite self-recursion → silent hang. Import them (AGENTS.md).
extern "C" {
    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(dest: *mut c_void, c: c_int, n: usize) -> *mut c_void;
    pub fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memcmp(a: *const c_void, b: *const c_void, n: usize) -> c_int;
}

// bindgen does not emit NULL (a macro) nor NT_SUCCESS (an inline).
pub const NULL: *mut c_void = core::ptr::null_mut();
#[inline(always)]
pub fn NT_SUCCESS(s: NTSTATUS) -> bool {
    s >= 0
}
// Translations reach for the C MACRO forms too (`NT_SUCCESS!(st)`, `CONTAINING_RECORD!(p, T, f)`)
// and a fresh crate otherwise starts with `cannot find macro` in every module that walks a list.
// Address arithmetic, so neither needs an `unsafe` context of its own.
#[macro_export] macro_rules! NT_SUCCESS { ($s:expr) => { (($s) as i32) >= 0 }; }
#[macro_export] macro_rules! CONTAINING_RECORD {
    // An indexed member (`Tail.Overlay.DriverContext[0]`): offsetof the array plus the
    // element stride, as clang folds it -- offset_of! refuses an index.
    ($addr:expr, $ty:ty, $($field:ident).+ [ $idx:expr ] $(,)?) => {{
        const fn __elem<T, const N: usize>(_: *const [T; N]) -> usize {
            core::mem::size_of::<T>()
        }
        let slot = core::mem::MaybeUninit::<$ty>::uninit();
        let stride = __elem(unsafe { core::ptr::addr_of!((*slot.as_ptr()).$($field).+) });
        (($addr) as usize - (core::mem::offset_of!($ty, $($field).+) + ($idx) as usize * stride))
            as *mut $ty
    }};
    // C's field argument is a member PATH (`Tail.Overlay.ListEntry`), and the model
    // often leaves a trailing comma after a multi-line call.
    ($addr:expr, $ty:ty, $($field:ident).+ $(,)?) => {
        (($addr) as usize - core::mem::offset_of!($ty, $($field).+)) as *mut $ty
    };
}
pub use crate::{CONTAINING_RECORD, NT_SUCCESS};
// The Win32 header macros, ported once in win32macros-rust, not per module.
pub use win32macros_rust::{ALIGN_UP, ARRAYSIZE, ASSERT, C_ASSERT, CopyMemory, DBG_UNREFERENCED_LOCAL_VARIABLE, DPRINT1, DWLP_DLGPROC, DWLP_MSGRESULT, DWLP_USER, FIELD_OFFSET, GET_X_LPARAM, GET_Y_LPARAM, GetBValue, GetGValue, GetRValue, HIBYTE, HIWORD, HandleToUlong, IDE_SET_ADVANCE_PIO_MODE, IDE_SET_DEFAULT_PIO_MODE, IDE_SET_MWDMA_MODE, IDE_SET_SWDMA_MODE, IDE_SET_UDMA_MODE, INVALID_HANDLE_VALUE, InitializeObjectAttributes, IntToPtr, IsContainerPartition, IsEqualGUID, IsFTPartition, IsRecognizedPartition, LANGIDFROMLCID, LOBYTE, LOWORD, LongToHandle, LongToPtr, MAKEINTRESOURCEA, MAKEINTRESOURCEW, MAKELANGID, MAKELCID, MAKELONG, MAKEWORD, MoveMemory, PAGE_ROUND_UP, PRIMARYLANGID, PtrToInt, PtrToLong, PtrToUint, PtrToUlong, RGB, RTL_CONSTANT_STRING, RTL_CONTAINS_FIELD, RTL_FIELD_SIZE, RTL_NUMBER_OF, RTL_NUMBER_OF_FIELD, RTL_SIZEOF_THROUGH_FIELD, RtlCopyMemory, RtlFillMemory, RtlGetProcessHeap, RtlMoveMemory, RtlZeroMemory, SORTIDFROMLCID, SRB_STATUS, SUBLANGID, UIntToPtr, ULongToHandle, ULongToPtr, UNICODE_NULL, UNREFERENCED_PARAMETER, UintToPtr, UlongToHandle, UlongToPtr, ZeroMemory, _countof, iswalnum, iswalpha, iswcntrl, iswdigit, iswgraph, iswlower, iswprint, iswpunct, iswspace, iswupper, iswxdigit, max, min, wide_bytes};

// The island re-exports the base NT identity: pub use ntdll_sys::nt::*;

// ==== port resolve: tree consensus (generated -- re-run `rustconv port resolve`) ====
extern "C" {
    pub fn _wcsdup(text: LPCWSTR) -> LPWSTR; // shell32-rust/src/new_menu.rs
    pub fn _wcsicmp(s1: *const u16, s2: *const u16) -> i32; // inflib-rust/src/infcore.rs
    pub fn _wtoi(s: *const u16) -> i32; // userinit-rust/src/preamble.rs
    pub fn free(p: *mut c_void); // desk-rust/src/prelude.rs
    pub fn wcschr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/wine/cred.rs
    pub fn wcscmp(a: *const u16, b: *const u16) -> c_int; // browser-rust/src/prelude.rs
    pub fn wcslen(s: *const u16) -> usize; // acpi-rust/src/buspdo.rs
    pub fn wcsrchr(s: *const u16, c: u16) -> *mut u16; // advapi32-rust/src/prelude.rs
}
extern "system" {
    pub fn ChangeServiceConfigW( arg1: SC_HANDLE, arg2: DWORD, arg3: DWORD, arg4: DWORD, arg5: LPCWSTR, arg6: LPCWSTR, arg7: LPDWORD, arg8: LPCWSTR, arg9: LPCWSTR, arg10: LPCWSTR, arg11: LPCWSTR, ) -> BOOL; // syssetup-rust/src/types_amd64.rs
    pub fn CheckDlgButton(arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn CloseHandle(hObject: HANDLE) -> BOOL; // advapi32-rust/src/externs.rs
    pub fn CloseServiceHandle(arg1: SC_HANDLE) -> BOOL; // netcfgx-rust/src/prelude.rs
    pub fn CreateProcessW( arg1: LPCWSTR, arg2: LPWSTR, arg3: LPSECURITY_ATTRIBUTES, arg4: LPSECURITY_ATTRIBUTES, arg5: BOOL, arg6: DWORD, arg7: PVOID, arg8: LPCWSTR, arg9: LPSTARTUPINFOW, arg10: LPPROCESS_INFORMATION, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeleteObject(arg1: HGDIOBJ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DestroyIcon(arg1: HICON) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DeviceCreateHardwarePageEx(parent: HWND, guids: *mut GUID, num_guids: UINT, display_mode: DWORD) -> HWND; // shell32-rust/src/dialogs/drvdefext.rs
    pub fn DisableThreadLibraryCalls(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn DrawIconEx( arg1: HDC, arg2: ::core::ffi::c_int, arg3: ::core::ffi::c_int, arg4: HICON, arg5: ::core::ffi::c_int, arg6: ::core::ffi::c_int, arg7: UINT, arg8: HBRUSH, arg9: UINT, ) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn EnableWindow(arg1: HWND, arg2: BOOL) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ExpandEnvironmentStringsW(arg1: LPCWSTR, arg2: LPWSTR, arg3: DWORD) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn FindClose(arg1: HANDLE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindFirstFileW(arg1: LPCWSTR, arg2: LPWIN32_FIND_DATAW) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn FindNextFileW(arg1: HANDLE, arg2: LPWIN32_FIND_DATAW) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn FindWindowW(arg1: LPCWSTR, arg2: LPCWSTR) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn FreeLibrary(arg1: HMODULE) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn GetDlgItem(arg1: HWND, arg2: ::core::ffi::c_int) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetFileAttributesW(lpFileName: LPCWSTR) -> DWORD; // aclui-rust/src/types_amd64.rs
    pub fn GetOpenFileNameW(arg1: LPOPENFILENAMEW) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn GetParent(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn GetProcAddress(arg1: HINSTANCE, arg2: LPCSTR) -> FARPROC; // aclui-rust/src/types_amd64.rs
    pub fn GetProcessHeap() -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn GetSystemMetrics(arg1: ::core::ffi::c_int) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "GetWindowLongW")]
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: INT) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn GetWindowsDirectoryW(arg1: LPWSTR, arg2: UINT) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn HeapAlloc(arg1: HANDLE, arg2: DWORD, arg3: SIZE_T) -> PVOID; // aclui-rust/src/types_amd64.rs
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL; // advapi32-rust/src/wine/security.rs
    pub fn ImageList_AddMasked( himl: HIMAGELIST, hbmImage: HBITMAP, crMask: COLORREF, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_Create( cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, flags: UINT, cInitial: ::core::ffi::c_int, cGrow: ::core::ffi::c_int, ) -> HIMAGELIST; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn ImageList_ReplaceIcon( himl: HIMAGELIST, i: ::core::ffi::c_int, hicon: HICON, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn IsDlgButtonChecked(arg1: HWND, arg2: ::core::ffi::c_int) -> UINT; // aclui-rust/src/types_amd64.rs
    pub fn KillTimer(arg1: HWND, arg2: UINT_PTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: LPCWSTR) -> HICON; // aclui-rust/src/types_amd64.rs
    pub fn LoadImageW( hInst: HINSTANCE, name: LPCWSTR, type_: UINT, cx: ::core::ffi::c_int, cy: ::core::ffi::c_int, fuLoad: UINT, ) -> HANDLE; // aclui-rust/src/types_amd64.rs
    pub fn LoadLibraryW(arg1: LPCWSTR) -> HINSTANCE; // aclui-rust/src/types_amd64.rs
    pub fn LoadStringW(instance: HINSTANCE, id: UINT, buffer: LPWSTR, capacity: INT) -> INT; // shell32-rust/src/cfs_folder.rs
    pub fn MessageBoxW( hWnd: HWND, lpText: LPCWSTR, lpCaption: LPCWSTR, uType: UINT, ) -> ::core::ffi::c_int; // aclui-rust/src/types_amd64.rs
    pub fn OpenSCManagerW(arg1: LPCWSTR, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // netcfgx-rust/src/prelude.rs
    pub fn OpenServiceW(arg1: SC_HANDLE, arg2: LPCWSTR, arg3: DWORD) -> SC_HANDLE; // netcfgx-rust/src/prelude.rs
    pub fn PlaySoundW(sound: LPCWSTR, module: HANDLE, flags: DWORD) -> BOOL; // shell32-rust/src/folders/CRecycleBinEnum.rs
    pub fn PropertySheetW(arg1: LPCPROPSHEETHEADERW) -> INT_PTR; // aclui-rust/src/types_amd64.rs
    pub fn RegCloseKey(hKey: HKEY) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegCreateKeyExW( hKey: HKEY, lpSubKey: LPCWSTR, Reserved: DWORD, lpClass: LPWSTR, dwOptions: DWORD, samDesired: REGSAM, lpSecurityAttributes: LPSECURITY_ATTRIBUTES, phkResult: PHKEY, lpdwDisposition: PDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumKeyExW( hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, lpcchName: LPDWORD, lpReserved: LPDWORD, lpClass: LPWSTR, lpcchClass: LPDWORD, lpftLastWriteTime: PFILETIME, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegEnumKeyW(hKey: HKEY, dwIndex: DWORD, lpName: LPWSTR, cchName: DWORD) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegOpenKeyExW( arg1: HKEY, arg2: LPCWSTR, arg3: DWORD, arg4: REGSAM, arg5: PHKEY, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegQueryValueExW( hKey: HKEY, lpValueName: LPCWSTR, lpReserved: LPDWORD, lpType: LPDWORD, lpData: LPBYTE, lpcbData: LPDWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn RegSetValueExW( hKey: HKEY, lpValueName: LPCWSTR, Reserved: DWORD, dwType: DWORD, lpData: *const BYTE, cbData: DWORD, ) -> LSTATUS; // desk-rust/src/types_amd64.rs
    pub fn SendDlgItemMessageW( arg1: HWND, arg2: ::core::ffi::c_int, arg3: UINT, arg4: WPARAM, arg5: LPARAM, ) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SendMessageW(arg1: HWND, arg2: UINT, arg3: WPARAM, arg4: LPARAM) -> LRESULT; // aclui-rust/src/types_amd64.rs
    pub fn SetDlgItemTextW(arg1: HWND, arg2: ::core::ffi::c_int, arg3: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetFocus(arg1: HWND) -> HWND; // aclui-rust/src/types_amd64.rs
    pub fn SetTimer(arg1: HWND, arg2: UINT_PTR, arg3: UINT, arg4: TIMERPROC) -> UINT_PTR; // aclui-rust/src/types_amd64.rs
    #[cfg_attr(target_arch = "x86", link_name = "SetWindowLongW")]
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: INT, dwNewLong: LONG_PTR) -> LONG_PTR; // shell32-rust/src/copy_to_menu.rs
    pub fn SetWindowTextW(arg1: HWND, arg2: LPCWSTR) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn SetupCloseInfFile(arg1: HINF); // desk-rust/src/types_amd64.rs
    pub fn SetupDefaultQueueCallbackW( arg1: PVOID, arg2: UINT, arg3: UINT_PTR, arg4: UINT_PTR, ) -> UINT; // desk-rust/src/types_amd64.rs
    pub fn SetupDiCallClassInstaller( arg1: DI_FUNCTION, arg2: HDEVINFO, arg3: PSP_DEVINFO_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiCreateDeviceInfoList(arg1: *const GUID, arg2: HWND) -> HDEVINFO; // desk-rust/src/types_amd64.rs
    pub fn SetupDiCreateDeviceInfoW( arg1: HDEVINFO, arg2: PCWSTR, arg3: *const GUID, arg4: PCWSTR, arg5: HWND, arg6: DWORD, arg7: PSP_DEVINFO_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiDestroyDeviceInfoList(arg1: HDEVINFO) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiEnumDeviceInterfaces( arg1: HDEVINFO, arg2: PSP_DEVINFO_DATA, arg3: *const GUID, arg4: DWORD, arg5: PSP_DEVICE_INTERFACE_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetClassDevsW( arg1: *const GUID, arg2: PCWSTR, arg3: HWND, arg4: DWORD, ) -> HDEVINFO; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetDeviceInterfaceDetailW( DeviceInfoSet: HDEVINFO, DeviceInterfaceData: PSP_DEVICE_INTERFACE_DATA, DeviceInterfaceDetailData: PSP_DEVICE_INTERFACE_DETAIL_DATA_W, DeviceInterfaceDetailDataSize: DWORD, RequiredSize: PDWORD, DeviceInfoData: PSP_DEVINFO_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiGetINFClassW( InfName: PCWSTR, ClassGuid: LPGUID, ClassName: PWSTR, ClassNameSize: DWORD, RequiredSize: PDWORD, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupDiSetDeviceRegistryPropertyW( DeviceInfoSet: HDEVINFO, DeviceInfoData: PSP_DEVINFO_DATA, Property: DWORD, PropertyBuffer: *const BYTE, PropertyBufferSize: DWORD, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupInitDefaultQueueCallback(arg1: HWND) -> PVOID; // desk-rust/src/types_amd64.rs
    pub fn SetupInstallFromInfSectionW( arg1: HWND, arg2: HINF, arg3: PCWSTR, arg4: UINT, arg5: HKEY, arg6: PCWSTR, arg7: UINT, arg8: PSP_FILE_CALLBACK_W, arg9: PVOID, arg10: HDEVINFO, arg11: PSP_DEVINFO_DATA, ) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupInstallServicesFromInfSectionW(arg1: HINF, arg2: PCWSTR, arg3: DWORD) -> BOOL; // desk-rust/src/types_amd64.rs
    pub fn SetupOpenInfFileW( FileName: PCWSTR, InfClass: PCWSTR, InfStyle: DWORD, ErrorLine: PUINT, ) -> HINF; // desk-rust/src/types_amd64.rs
    pub fn SetupTermDefaultQueueCallback(arg1: PVOID); // desk-rust/src/types_amd64.rs
    pub fn ShellExecuteW( hwnd: HWND, lpOperation: LPCWSTR, lpFile: LPCWSTR, lpParameters: LPCWSTR, lpDirectory: LPCWSTR, nShowCmd: INT, ) -> HINSTANCE; // desk-rust/src/types_amd64.rs
    pub fn ShowWindow(arg1: HWND, arg2: ::core::ffi::c_int) -> BOOL; // aclui-rust/src/types_amd64.rs
    pub fn StartServiceW( hService: SC_HANDLE, dwNumServiceArgs: DWORD, lpServiceArgVectors: *mut LPCWSTR, ) -> BOOL; // netcfgx-rust/src/prelude.rs
}
// ==== end port resolve ====

// ==== port resolve: cross-TU definitions (generated) ====
pub use crate::mmsys::audio::AudioDlgProc;
pub use crate::mmsys::sounds::SoundsDlgProc;
pub use crate::mmsys::voice::VoiceDlgProc;
pub use crate::mmsys::volume::VolumeDlgProc;
pub use crate::mmsys::mmsys::hApplet;
// ==== end cross-TU definitions ====
pub use strsafe_rust::{RtlStringCbPrintfA, RtlStringCbPrintfW, RtlStringCchPrintfA, RtlStringCchPrintfExA, RtlStringCchPrintfExW, RtlStringCchPrintfW, StringCbCatW, StringCbCopyNExW, StringCbCopyW, StringCbPrintfExW, StringCbPrintfW, StringCchCatW, StringCchCopyNExW, StringCchCopyNW, StringCchCopyW, StringCchLengthW, StringCchPrintfW, StringCchVPrintfW};
pub use strsafe_rust::ntstrsafe::{RtlStringCatNWorkerW, RtlStringCatWorkerW, RtlStringCbCatNW, RtlStringCbCatW, RtlStringCbCopyNA, RtlStringCbCopyNW, RtlStringCbCopyW, RtlStringCchCatNW, RtlStringCchCatW, RtlStringCchCopyA, RtlStringCchCopyNW, RtlStringCchCopyW, RtlStringCopyNWorkerW, RtlStringCopyWorkerA, RtlStringCopyWorkerW, RtlStringLengthWorkerW};

// ==== port facts: constants evaluated by clang from the C headers (generated -- `rustconv port resolve`) ====
pub const SND_ALIAS_SYSTEMDEFAULT: DWORD = 0x4453; // clang, volume.c
pub const TVN_SELCHANGED: u32 = 0xFFFFFE3D; // clang, sounds.c
// ==== end port facts ====

// ==== port local types: types the module's .c files define, laid out by c2rust (generated -- `rustconv port resolve`) ====
// HIMAGELIST: c2rust, sounds.c
pub type HIMAGELIST = *mut _IMAGELIST;
// HTREEITEM: c2rust, sounds.c
pub type HTREEITEM = *mut _TREEITEM;
// PAGE_DATA: c2rust, speakervolume.c
pub type PAGE_DATA = _PAGE_DATA;
// _PAGE_DATA: c2rust, speakervolume.c
#[derive(Copy, Clone)]
#[repr(C)]
pub struct _PAGE_DATA {
    pub hMixer: HMIXER,
    pub volumeControlID: DWORD,
    pub volumeChannels: DWORD,
    pub volumeMinimum: DWORD,
    pub volumeMaximum: DWORD,
    pub volumeStep: DWORD,
    pub volumeValues: PMIXERCONTROLDETAILS_UNSIGNED,
    pub volumeSync: BOOL,
}
// ==== end port local types ====

// ==== port externs: C functions declared outside the island -- c2rust signature, clang calling convention (generated -- `rustconv port resolve`) ====
extern "system" {
    pub fn mciSendCommandW(_: MCIDEVICEID, _: UINT, _: DWORD_PTR, _: DWORD_PTR) -> MCIERROR; // sounds.c
    pub fn midiOutGetDevCapsW(uDeviceID: UINT_PTR, pmoc: LPMIDIOUTCAPSW, cbmoc: UINT) -> MMRESULT; // audio.c
    pub fn midiOutGetNumDevs() -> UINT; // audio.c
    pub fn mixerClose(_: HMIXER) -> MMRESULT; // speakervolume.c
    pub fn mixerGetControlDetailsW(_: HMIXEROBJ, _: LPMIXERCONTROLDETAILS, _: DWORD) -> MMRESULT; // speakervolume.c
    pub fn mixerGetDevCapsW(uMxId: UINT_PTR, pmxcaps: LPMIXERCAPSW, cbmxcaps: UINT) -> MMRESULT; // volume.c
    pub fn mixerGetID(_: HMIXEROBJ, _: PUINT, _: DWORD) -> MMRESULT; // audio.c
    pub fn mixerGetLineControlsW(_: HMIXEROBJ, _: LPMIXERLINECONTROLSW, _: DWORD) -> MMRESULT; // speakervolume.c
    pub fn mixerGetLineInfoW(_: HMIXEROBJ, _: LPMIXERLINEW, _: DWORD) -> MMRESULT; // speakervolume.c
    pub fn mixerGetNumDevs() -> UINT; // volume.c
    pub fn mixerOpen(_: LPHMIXER, _: UINT, _: DWORD_PTR, _: DWORD_PTR, _: DWORD) -> MMRESULT; // speakervolume.c
    pub fn mixerSetControlDetails(_: HMIXEROBJ, _: LPMIXERCONTROLDETAILS, _: DWORD) -> MMRESULT; // speakervolume.c
    pub fn waveInGetDevCapsW(uDeviceID: UINT_PTR, pwic: LPWAVEINCAPSW, cbwic: UINT) -> MMRESULT; // audio.c
    pub fn waveInGetNumDevs() -> UINT; // audio.c
    pub fn waveOutGetDevCapsW(_: UINT_PTR, _: LPWAVEOUTCAPSW, _: UINT) -> MMRESULT; // audio.c
    pub fn waveOutGetNumDevs() -> UINT; // sounds.c
}
// ==== end port externs ====

// ==== port resolve: UNICODE TCHAR family (generated) ====
pub type TCHAR = WCHAR;
pub type PTCHAR = *mut WCHAR;
pub type TBYTE = WCHAR;
pub type PTBYTE = *mut WCHAR;
pub type LPTSTR = LPWSTR;
pub type LPCTSTR = LPCWSTR;
pub type PTSTR = LPWSTR;
pub type PCTSTR = LPCWSTR;
pub type LPTCH = LPWSTR;
pub type PTCH = LPWSTR;
// ==== end UNICODE TCHAR family ====

// ==== port macros: SDK header macros, lowered from the preprocessor's own expansion (generated -- re-run `rustconv port macros`) ====
#[macro_export] macro_rules! ComboBox_AddString { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((323) as _), ((0) as _), (((($p1) as LPCTSTR)) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_DeleteString { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((324) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetCount { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((326) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetCurSel { ($p0:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((327) as _), ((0) as _), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_GetItemData { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((336) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as UINT_PTR)) as LRESULT) }; }
#[macro_export] macro_rules! ComboBox_SetCurSel { ($p0:expr, $p1:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((334) as _), (((($p1) as c_int)) as WPARAM), ((0) as _))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ComboBox_SetItemData { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((((SendMessageW((($p0) as _), ((337) as _), (((($p1) as c_int)) as WPARAM), (($p2) as LPARAM))) as DWORD)) as c_int) }; }
#[macro_export] macro_rules! ImageList_AddIcon { ($p0:expr, $p1:expr $(,)?) => { ImageList_ReplaceIcon((($p0) as _), (((-(1 as c_int))) as _), (($p1) as _)) }; }
#[macro_export] macro_rules! IsEqualIID { ($p0:expr, $p1:expr $(,)?) => { (((memcmp((($p0) as _), (($p1) as _), ((core::mem::size_of::<GUID>()) as _))) as isize == 0) as c_int) }; }
#[macro_export] macro_rules! MAKEINTRESOURCE { ($p0:expr $(,)?) => { ((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as ULONG_PTR)) as LPWSTR) }; }
#[macro_export] macro_rules! MAKELPARAM { ($p0:expr, $p1:expr $(,)?) => { (((((((((((((($p0) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) | (((((((($p1) as DWORD_PTR) & 0xffff)) as WORD)) as DWORD) << 16))) as LONG)) as DWORD)) as LPARAM) }; }
#[macro_export] macro_rules! PropSheet_Changed { ($p0:expr, $p1:expr $(,)?) => { SendMessageW((($p0) as _), (((1024 + 104)) as _), (($p1) as WPARAM), ((0) as _)) }; }
#[macro_export] macro_rules! TreeView_DeleteAllItems { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 1)) as _), ((0) as _), (((((((-(0x10000 as c_int))) as ULONG_PTR)) as HTREEITEM)) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_GetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 62)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
#[macro_export] macro_rules! TreeView_GetSelection { ($p0:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 10)) as _), ((0x9) as WPARAM), ((((core::ptr::null_mut::<c_void>()) as HTREEITEM)) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_InsertItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 50)) as _), ((0) as _), (((($p1) as LPTVINSERTSTRUCTW)) as LPARAM))) as HTREEITEM) }; }
#[macro_export] macro_rules! TreeView_SetImageList { ($p0:expr, $p1:expr, $p2:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 9)) as _), (($p2) as _), (((($p1) as HIMAGELIST)) as LPARAM))) as HIMAGELIST) }; }
#[macro_export] macro_rules! TreeView_SetItem { ($p0:expr, $p1:expr $(,)?) => { ((SendMessageW((($p0) as _), (((0x1100 + 63)) as _), ((0) as _), (($p1) as LPARAM))) as BOOL) }; }
pub const HKEY_CURRENT_USER: HKEY = ((((((0x80000001u32) as LONG)) as LONG_PTR)) as HKEY);
pub const HKEY_LOCAL_MACHINE: HKEY = ((((((0x80000002u32) as LONG)) as LONG_PTR)) as HKEY);
pub const INVALID_FILE_ATTRIBUTES: DWORD = (((-(1 as c_int))) as DWORD);
pub type REFCLSID = *const CLSID;
pub type REFIID = *const IID;
pub const S_OK: HRESULT = ((0) as HRESULT);
pub const TVI_FIRST: HTREEITEM = (((((-(0xffff as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_ROOT: HTREEITEM = (((((-(0x10000 as c_int))) as ULONG_PTR)) as HTREEITEM);
pub const TVI_SORT: HTREEITEM = (((((-(0xfffd as c_int))) as ULONG_PTR)) as HTREEITEM);
pub use crate::{ComboBox_AddString, ComboBox_DeleteString, ComboBox_GetCount, ComboBox_GetCurSel, ComboBox_GetItemData, ComboBox_SetCurSel, ComboBox_SetItemData, ImageList_AddIcon, IsEqualIID, MAKEINTRESOURCE, MAKELPARAM, PropSheet_Changed, TreeView_DeleteAllItems, TreeView_GetItem, TreeView_GetSelection, TreeView_InsertItem, TreeView_SetImageList, TreeView_SetItem};
// ==== end SDK header macros ====

// ==== port macros: the module's own C macros (generated -- `rustconv port macros`) ====
// dll/cpl/mmsys/mmsys.h:23: #define CONST_STR_LEN(str) (_countof(str) - 1)
#[allow(unused_macros)]
macro_rules! CONST_STR_LEN {
    ($str:expr $(,)?) => {
        (_countof!($str) - 1)
    };
}
#[allow(unused_imports)]
pub(crate) use CONST_STR_LEN;
// ==== end port macros ====
