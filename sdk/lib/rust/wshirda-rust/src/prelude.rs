//! wshirda-rust prelude — the single import surface the translated module uses.
//!
//! wshirda's exports are pure stubs that never dereference their arguments, so — unlike
//! wshtcpip — there are no winsock/WSH/TDI struct layouts to bindgen. This just supplies the
//! pointer/scalar type aliases the export signatures name (a pointer is a pointer; a
//! mistyped opaque pointer cannot cause a wire-format bug because no field is read), the
//! `DLL_*`/`TRUE` constants DllMain switches on, and the one kernel32 import it calls.
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use core::ffi::{c_char, c_void};

// ---- Scalar aliases (Win32 fixed widths — identical on all three arches). ----
pub type BOOL = i32;
pub type INT = i32;
pub type DWORD = u32;
pub type ULONG = u32;
// SOCKET is UINT_PTR (pointer-width): 4 bytes on i386, 8 on amd64/arm64. Getting this width
// right keeps the i386 stdcall @N decoration (and the amd64/arm64 register class) matching
// the .spec, even though the value is never used.
pub type SOCKET = usize;

// ---- Pointer aliases. Every one is opaque `*mut c_void` unless the signature needs a
// concrete pointee width (i32/u32/u16/char), because no callee ever reads through them. ----
pub type PVOID = *mut c_void;
pub type LPVOID = *mut c_void;
pub type HANDLE = *mut c_void;
pub type HMODULE = *mut c_void;
pub type LPINT = *mut INT;
pub type PINT = *mut INT;
pub type LPDWORD = *mut DWORD;
pub type PDWORD = *mut DWORD;
pub type LPBOOL = *mut BOOL;
pub type LPWSTR = *mut u16;
pub type PCHAR = *mut c_char;
pub type LPGUID = *mut c_void; // GUID*, opaque
pub type PSOCKADDR = *mut c_void; // SOCKADDR*, opaque
pub type PSOCKADDR_INFO = *mut c_void; // SOCKADDR_INFO*, opaque
pub type PWINSOCK_MAPPING = *mut c_void; // WINSOCK_MAPPING*, opaque
pub type LPWSAPROTOCOL_INFOW = *mut c_void; // WSAPROTOCOL_INFOW*, opaque
pub type LPWSAOVERLAPPED = *mut c_void; // WSAOVERLAPPED*, opaque
pub type LPWSAOVERLAPPED_COMPLETION_ROUTINE = *mut c_void; // fn ptr, pointer-width
pub type PUNICODE_STRING = *mut c_void; // UNICODE_STRING*, opaque

// ---- DllMain reason codes (winnt.h) + BOOL TRUE. ----
pub const DLL_PROCESS_DETACH: DWORD = 0;
pub const DLL_PROCESS_ATTACH: DWORD = 1;
pub const DLL_THREAD_ATTACH: DWORD = 2;
pub const DLL_THREAD_DETACH: DWORD = 3;
pub const TRUE: BOOL = 1;

// ---- The one Win32 import: DisableThreadLibraryCalls (kernel32, stdcall → extern "system"
// gives `_DisableThreadLibraryCalls@4` on i386). ----
extern "system" {
    pub fn DisableThreadLibraryCalls(hLibModule: HMODULE) -> BOOL;
}
