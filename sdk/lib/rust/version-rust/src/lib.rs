#![no_std]
#[allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#[allow(dead_code)]
#[allow(unused_assignments)]
#[allow(non_camel_case_types)]

// Wine's TRACE/WARN/FIXME debug macros. In Wine these are C preprocessor macros that
// compile away in release builds; the translation had made them no-op C-variadic fns,
// which stable Rust cannot define (`c_variadic`, rust-lang#44930). As macros they keep
// the original `TRACE!(...)` call syntax, still evaluate their arguments, cost nothing,
// and need no C trampoline. Defined here (not below) because macro_rules! is visible
// only AFTER its definition point.
macro_rules! TRACE { ($($arg:expr),* $(,)?) => {{ $( let _ = &$arg; )* }}; }
macro_rules! WARN  { ($($arg:expr),* $(,)?) => {{ $( let _ = &$arg; )* }}; }
macro_rules! FIXME { ($($arg:expr),* $(,)?) => {{ $( let _ = &$arg; )* }}; }

use core::ffi::c_void;

// ============================================================================
// Base Win32 type aliases
// ============================================================================
pub type BOOL = i32;
pub type CHAR = i8;
pub type WCHAR = u16;
pub type BYTE = u8;
pub type WORD = u16;
pub type DWORD = u32;
pub type INT = i32;
pub type UINT = u32;
pub type LONG = i32;
pub type ULONG = u32;
pub type SIZE_T = usize;
pub type HANDLE = *mut c_void;
pub type HFILE = i32;
pub type HMODULE = *mut c_void;
pub type HRSRC = *mut c_void;
pub type HGLOBAL = *mut c_void;
pub type LANGID = WORD;
pub type LPCSTR = *const CHAR;
pub type LPSTR = *mut CHAR;
pub type LPCWSTR = *const WCHAR;
pub type LPWSTR = *mut WCHAR;
pub type LPVOID = *mut c_void;
pub type LPCVOID = *const c_void;
pub type LPBYTE = *mut BYTE;
pub type LPDWORD = *mut DWORD;
pub type PUINT = *mut UINT;
pub type LPBOOL = *mut BOOL;
pub type LPOFSTRUCT = *mut OFSTRUCT;

// ============================================================================
// PE/COFF structs
// ============================================================================

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_DOS_HEADER {
    pub e_magic: WORD,
    pub e_cblp: WORD,
    pub e_cp: WORD,
    pub e_crlc: WORD,
    pub e_cparhdr: WORD,
    pub e_minalloc: WORD,
    pub e_maxalloc: WORD,
    pub e_ss: WORD,
    pub e_sp: WORD,
    pub e_csum: WORD,
    pub e_ip: WORD,
    pub e_cs: WORD,
    pub e_lfarlc: WORD,
    pub e_ovno: WORD,
    pub e_res: [WORD; 4],
    pub e_oemid: WORD,
    pub e_oeminfo: WORD,
    pub e_res2: [WORD; 10],
    pub e_lfanew: LONG,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_FILE_HEADER {
    pub Machine: WORD,
    pub NumberOfSections: WORD,
    pub TimeDateStamp: DWORD,
    pub PointerToSymbolTable: DWORD,
    pub NumberOfSymbols: DWORD,
    pub SizeOfOptionalHeader: WORD,
    pub Characteristics: WORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_DATA_DIRECTORY {
    pub VirtualAddress: DWORD,
    pub Size: DWORD,
}
pub type PIMAGE_DATA_DIRECTORY = *mut IMAGE_DATA_DIRECTORY;

#[repr(C)]
#[derive(Copy, Clone)]
pub union IMAGE_SECTION_HEADER_Misc {
    pub PhysicalAddress: DWORD,
    pub VirtualSize: DWORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_SECTION_HEADER {
    pub Name: [BYTE; 8],
    pub Misc: IMAGE_SECTION_HEADER_Misc,
    pub VirtualAddress: DWORD,
    pub SizeOfRawData: DWORD,
    pub PointerToRawData: DWORD,
    pub PointerToRelocations: DWORD,
    pub PointerToLinenumbers: DWORD,
    pub NumberOfRelocations: WORD,
    pub NumberOfLinenumbers: WORD,
    pub Characteristics: DWORD,
}
pub type PIMAGE_SECTION_HEADER = *mut IMAGE_SECTION_HEADER;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_OPTIONAL_HEADER32 {
    pub Magic: WORD,
    pub MajorLinkerVersion: BYTE,
    pub MinorLinkerVersion: BYTE,
    pub SizeOfCode: DWORD,
    pub SizeOfInitializedData: DWORD,
    pub SizeOfUninitializedData: DWORD,
    pub AddressOfEntryPoint: DWORD,
    pub BaseOfCode: DWORD,
    pub BaseOfData: DWORD,
    pub ImageBase: DWORD,
    pub SectionAlignment: DWORD,
    pub FileAlignment: DWORD,
    pub MajorOperatingSystemVersion: WORD,
    pub MinorOperatingSystemVersion: WORD,
    pub MajorImageVersion: WORD,
    pub MinorImageVersion: WORD,
    pub MajorSubsystemVersion: WORD,
    pub MinorSubsystemVersion: WORD,
    pub Win32VersionValue: DWORD,
    pub SizeOfImage: DWORD,
    pub SizeOfHeaders: DWORD,
    pub CheckSum: DWORD,
    pub Subsystem: WORD,
    pub DllCharacteristics: WORD,
    pub SizeOfStackReserve: DWORD,
    pub SizeOfStackCommit: DWORD,
    pub SizeOfHeapReserve: DWORD,
    pub SizeOfHeapCommit: DWORD,
    pub LoaderFlags: DWORD,
    pub NumberOfRvaAndSizes: DWORD,
    pub DataDirectory: [IMAGE_DATA_DIRECTORY; 16],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_OPTIONAL_HEADER64 {
    pub Magic: WORD,
    pub MajorLinkerVersion: BYTE,
    pub MinorLinkerVersion: BYTE,
    pub SizeOfCode: DWORD,
    pub SizeOfInitializedData: DWORD,
    pub SizeOfUninitializedData: DWORD,
    pub AddressOfEntryPoint: DWORD,
    pub BaseOfCode: DWORD,
    pub ImageBase: ULONGLONG,
    pub SectionAlignment: DWORD,
    pub FileAlignment: DWORD,
    pub MajorOperatingSystemVersion: WORD,
    pub MinorOperatingSystemVersion: WORD,
    pub MajorImageVersion: WORD,
    pub MinorImageVersion: WORD,
    pub MajorSubsystemVersion: WORD,
    pub MinorSubsystemVersion: WORD,
    pub Win32VersionValue: DWORD,
    pub SizeOfImage: DWORD,
    pub SizeOfHeaders: DWORD,
    pub CheckSum: DWORD,
    pub Subsystem: WORD,
    pub DllCharacteristics: WORD,
    pub SizeOfStackReserve: ULONGLONG,
    pub SizeOfStackCommit: ULONGLONG,
    pub SizeOfHeapReserve: ULONGLONG,
    pub SizeOfHeapCommit: ULONGLONG,
    pub LoaderFlags: DWORD,
    pub NumberOfRvaAndSizes: DWORD,
    pub DataDirectory: [IMAGE_DATA_DIRECTORY; 16],
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_NT_HEADERS32 {
    pub Signature: DWORD,
    pub FileHeader: IMAGE_FILE_HEADER,
    pub OptionalHeader: IMAGE_OPTIONAL_HEADER32,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_NT_HEADERS64 {
    pub Signature: DWORD,
    pub FileHeader: IMAGE_FILE_HEADER,
    pub OptionalHeader: IMAGE_OPTIONAL_HEADER64,
}

pub type ULONGLONG = u64;

// Resource directory types
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_RESOURCE_DIRECTORY {
    pub Characteristics: DWORD,
    pub TimeDateStamp: DWORD,
    pub MajorVersion: WORD,
    pub MinorVersion: WORD,
    pub NumberOfNamedEntries: WORD,
    pub NumberOfIdEntries: WORD,
}
pub type PIMAGE_RESOURCE_DIRECTORY = *mut IMAGE_RESOURCE_DIRECTORY;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_RESOURCE_DIRECTORY_ENTRY_u {
    pub Id: DWORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union IMAGE_RESOURCE_DIRECTORY_ENTRY_u2_s2 {
    pub OffsetToDirectory: DWORD,
    pub OffsetToData: DWORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_RESOURCE_DIRECTORY_ENTRY_u2_s1 {
    pub _pad: DWORD,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union IMAGE_RESOURCE_DIRECTORY_ENTRY_u2 {
    pub s1: IMAGE_RESOURCE_DIRECTORY_ENTRY_u2_s1,
    pub s2: IMAGE_RESOURCE_DIRECTORY_ENTRY_u2_s2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_RESOURCE_DIRECTORY_ENTRY {
    pub u: IMAGE_RESOURCE_DIRECTORY_ENTRY_u,
    pub u2: IMAGE_RESOURCE_DIRECTORY_ENTRY_u2,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_RESOURCE_DATA_ENTRY {
    pub OffsetToData: DWORD,
    pub Size: DWORD,
    pub CodePage: DWORD,
    pub Reserved: DWORD,
}

// IMAGE_OS2_HEADER (NE header)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct IMAGE_OS2_HEADER {
    pub ne_magic: WORD,
    pub ne_ver: BYTE,
    pub ne_rev: BYTE,
    pub ne_enttab: WORD,
    pub ne_cbenttab: WORD,
    pub ne_crc: LONG,
    pub ne_flags: WORD,
    pub ne_autodata: WORD,
    pub ne_heap: WORD,
    pub ne_stack: WORD,
    pub ne_csip: LONG,
    pub ne_sssp: LONG,
    pub ne_cseg: WORD,
    pub ne_cmod: WORD,
    pub ne_cbnrestab: WORD,
    pub ne_segtab: WORD,
    pub ne_rsrctab: WORD,
    pub ne_restab: WORD,
    pub ne_modtab: WORD,
    pub ne_imptab: WORD,
    pub ne_nrestab: LONG,
    pub ne_cmovent: WORD,
    pub ne_align: WORD,
    pub ne_cres: WORD,
    pub ne_exetyp: BYTE,
    pub ne_flagsothers: BYTE,
    pub ne_pretthunks: WORD,
    pub ne_psegrefbytes: WORD,
    pub ne_swaparea: WORD,
    pub ne_expver: WORD,
}

// ============================================================================
// _UNICODE_STRING (for Rtl* calls)
// ============================================================================
#[repr(C)]
pub struct _UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: LPWSTR,
}

// ============================================================================
// OFSTRUCT (used by OpenFile / LZOpenFile)
// ============================================================================
#[repr(C)]
pub struct OFSTRUCT {
    pub cBytes: BYTE,
    pub fFixedDisk: BYTE,
    pub nErrCode: WORD,
    pub Reserved1: WORD,
    pub Reserved2: WORD,
    pub szPathName: [CHAR; 128],
}

// ============================================================================
// VS_FIXEDFILEINFO (version resource fixed info)
// ============================================================================
#[repr(C)]
pub struct VS_FIXEDFILEINFO {
    pub dwSignature: DWORD,
    pub dwStrucVersion: DWORD,
    pub dwFileVersionMS: DWORD,
    pub dwFileVersionLS: DWORD,
    pub dwProductVersionMS: DWORD,
    pub dwProductVersionLS: DWORD,
    pub dwFileFlagsMask: DWORD,
    pub dwFileFlags: DWORD,
    pub dwFileOS: DWORD,
    pub dwFileType: DWORD,
    pub dwFileSubtype: DWORD,
    pub dwFileDateMS: DWORD,
    pub dwFileDateLS: DWORD,
}

// ============================================================================
// Constants
// ============================================================================
pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;

// PE signatures
pub const IMAGE_DOS_SIGNATURE: WORD = 0x5A4D;
pub const IMAGE_OS2_SIGNATURE: DWORD = 0x0000454E; // 'NE'
pub const IMAGE_NT_SIGNATURE: DWORD = 0x00004550;  // 'PE'

// Optional header magic
pub const IMAGE_NT_OPTIONAL_HDR32_MAGIC: WORD = 0x10B;
pub const IMAGE_NT_OPTIONAL_HDR64_MAGIC: WORD = 0x20B;

// Directory entries
pub const IMAGE_DIRECTORY_ENTRY_RESOURCE: usize = 2;

// Version resource types
pub const VS_FILE_INFO: WORD = 16;
pub const VS_VERSION_INFO: WORD = 1;
pub const VS_FFI_SIGNATURE: DWORD = 0xFEEF04BD;

// File version flags
pub const FILE_VER_GET_LOCALISED: DWORD = 0x01;
pub const VS_FF_DEBUG: DWORD = 0x00000001;
pub const VS_FF_PRERELEASE: DWORD = 0x00000002;
pub const VS_FF_PATCHED: DWORD = 0x00000004;
pub const VS_FF_PRIVATEBUILD: DWORD = 0x00000008;
pub const VS_FF_INFOINFERRED: DWORD = 0x00000010;
pub const VS_FF_SPECIALBUILD: DWORD = 0x00000020;

// VFT (file type)
pub const VFT_UNKNOWN: DWORD = 0;
pub const VFT_APP: DWORD = 1;
pub const VFT_DLL: DWORD = 2;
pub const VFT_DRV: DWORD = 3;
pub const VFT_FONT: DWORD = 4;
pub const VFT_VXD: DWORD = 5;
pub const VFT_STATIC_LIB: DWORD = 7;

// VFT2 subtypes
pub const VFT2_UNKNOWN: DWORD = 0;
pub const VFT2_DRV_PRINTER: DWORD = 1;
pub const VFT2_DRV_KEYBOARD: DWORD = 2;
pub const VFT2_DRV_LANGUAGE: DWORD = 3;
pub const VFT2_DRV_DISPLAY: DWORD = 4;
pub const VFT2_DRV_MOUSE: DWORD = 5;
pub const VFT2_DRV_NETWORK: DWORD = 6;
pub const VFT2_DRV_SYSTEM: DWORD = 7;
pub const VFT2_DRV_INSTALLABLE: DWORD = 8;
pub const VFT2_DRV_SOUND: DWORD = 9;
pub const VFT2_DRV_COMM: DWORD = 10;
pub const VFT2_DRV_INPUTMETHOD: DWORD = 11;
pub const VFT2_DRV_VERSIONED_PRINTER: DWORD = 12;
pub const VFT2_FONT_RASTER: DWORD = 1;
pub const VFT2_FONT_VECTOR: DWORD = 2;
pub const VFT2_FONT_TRUETYPE: DWORD = 3;

// VOS
pub const VOS_UNKNOWN: DWORD = 0;
pub const VOS_DOS: DWORD = 0x00010000;
pub const VOS_OS216: DWORD = 0x00020000;
pub const VOS_OS232: DWORD = 0x00030000;
pub const VOS_NT: DWORD = 0x00040000;
pub const VOS__BASE: DWORD = 0x00000000;
pub const VOS__WINDOWS16: DWORD = 0x00000001;
pub const VOS__WINDOWS32: DWORD = 0x00000004;
pub const VOS__PM16: DWORD = 0x00000002;
pub const VOS__PM32: DWORD = 0x00000003;

// Language
pub const LANG_NEUTRAL: WORD = 0x00;
pub const SUBLANG_NEUTRAL: WORD = 0x00;
pub const SUBLANG_DEFAULT: WORD = 0x01;
pub const LANG_ENGLISH: WORD = 0x09;

// Error codes
pub const ERROR_INVALID_PARAMETER: DWORD = 87;
pub const ERROR_BAD_PATHNAME: DWORD = 161;
pub const ERROR_RESOURCE_DATA_NOT_FOUND: DWORD = 1812;
pub const ERROR_RESOURCE_TYPE_NOT_FOUND: DWORD = 1813;
pub const ERROR_FILE_NOT_FOUND: DWORD = 2;
pub const ERROR_INVALID_DATA: DWORD = 13;
pub const ERROR_ACCESS_DENIED: DWORD = 5;
pub const ERROR_SHARING_VIOLATION: DWORD = 32;

// File attributes
pub const INVALID_FILE_ATTRIBUTES: DWORD = 0xFFFFFFFF;
pub const FILE_ATTRIBUTE_READONLY: DWORD = 0x00000001;

// OF_ flags
pub const OF_READ: u32 = 0x0000;
pub const OF_EXIST: u32 = 0x4000;
pub const OF_CREATE: u32 = 0x1000;
pub const OF_SHARE_EXCLUSIVE: u32 = 0x0010;

// HFILE_ERROR
pub const HFILE_ERROR: HFILE = -1;

// LoadLibrary flags
pub const LOAD_LIBRARY_AS_DATAFILE: DWORD = 0x00000002;

// SEEK
pub const SEEK_SET: i32 = 0;
pub const SEEK_CUR: i32 = 1;

// CP
pub const CP_ACP: UINT = 0;

// MAX_PATH
pub const MAX_PATH: usize = 260;

// VFFF / VFF / VIF flags
pub const VFFF_ISSHAREDFILE: DWORD = 0x0001;
pub const VFF_CURNEDEST: DWORD = 0x0001;
pub const VFF_FILEINUSE: DWORD = 0x0002;
pub const VFF_BUFFTOOSMALL: DWORD = 0x0004;

// VIFF
pub const VIFF_FORCEINSTALL: DWORD = 0x0001;
pub const VIFF_DONTDELETEOLD: DWORD = 0x0002;

// VIF
pub const VIF_CANNOTREADSRC: DWORD = 0x00010000;
pub const VIF_MISMATCH: DWORD = 0x00020000;
pub const VIF_SRCOLD: DWORD = 0x00040000;
pub const VIF_DIFFTYPE: DWORD = 0x00080000;
pub const VIF_DIFFLANG: DWORD = 0x00100000;
pub const VIF_OUTOFMEMORY: DWORD = 0x00008000;
pub const VIF_OUTOFSPACE: DWORD = 0x00000100;
pub const VIF_CANNOTCREATE: DWORD = 0x00000200;
pub const VIF_CANNOTDELETE: DWORD = 0x00000400;
pub const VIF_CANNOTRENAME: DWORD = 0x00000800;
pub const VIF_CANNOTDELETECUR: DWORD = 0x00004000;
pub const VIF_WRITEPROT: DWORD = 0x00010001;
pub const VIF_ACCESSVIOLATION: DWORD = 0x00000001;
pub const VIF_SHARINGVIOLATION: DWORD = 0x00000002;
pub const VIF_TEMPFILE: DWORD = 0x00000004;
pub const VIF_BUFFTOOSMALL: DWORD = 0x00000010;

// LZERROR
pub const LZERROR_BADINHANDLE: LONG = -1;
pub const LZERROR_BADOUTHANDLE: LONG = -2;
pub const LZERROR_READ: LONG = -3;
pub const LZERROR_WRITE: LONG = -4;
pub const LZERROR_GLOBALLOC: LONG = -5;
pub const LZERROR_GLOBLOCK: LONG = -6;
pub const LZERROR_BADVALUE: LONG = -7;
pub const LZERROR_UNKNOWNALG: LONG = -8;

// ============================================================================
// Helper inline functions (replacing C macros)
// ============================================================================
#[inline]
pub const fn LOWORD(l: DWORD) -> i32 { (l as u32 & 0xFFFF) as i32 }
#[inline]
pub const fn HIWORD(l: DWORD) -> i32 { ((l as u32 >> 16) & 0xFFFF) as i32 }
#[inline]
pub const fn LOBYTE(w: WORD) -> i32 { (w & 0xFF) as i32 }
#[inline]
pub const fn HIBYTE(w: WORD) -> i32 { ((w >> 8) & 0xFF) as i32 }
#[inline]
pub const fn MAKELANGID(primary: WORD, sub: WORD) -> WORD {
    ((sub as u16) << 10) | (primary as u16)
}
#[inline]
pub const fn PRIMARYLANGID(lgid: WORD) -> WORD { lgid & 0x3FF }

// ============================================================================
// Forward declarations for NT/Win32 APIs (resolved at link time)
// ============================================================================

// Minimal TEB exposing only NtCurrentTeb().CurrentLocale. The field offset is
// pointer-width-dependent: 0xC0 on i386, 0x108 on amd64/arm64 (64-bit TEB layout).
#[cfg(target_arch = "x86")]
#[repr(C)]
pub struct TEB {
    // CurrentLocale sits at TEB+0xC4 on i386 (NT_TIB 0x1C + fields up through
    // WOW32Reserved). 0xC0 (the prior value) read WOW32Reserved instead.
    _pad: [u8; 0xC4],
    pub CurrentLocale: u32,
}
#[cfg(not(target_arch = "x86"))]
#[repr(C)]
pub struct TEB {
    _pad: [u8; 0x108],
    pub CurrentLocale: u32,
}

// Wine debug channel for TRACE_ON(__wine_dbch_ver)
pub const __wine_dbch_ver: u32 = 0;

// NtCurrentTeb() is an inline intrinsic (reads the TEB self-pointer from the
// thread register), NOT an exported symbol on amd64/arm64 — provide it inline.
// TEB self-pointer lives at NT_TIB.Self: fs:[0x18] (i386), gs:[0x30] (amd64), x18 (arm64).
#[inline(always)]
pub unsafe fn NtCurrentTeb() -> *mut TEB {
    let teb: usize;
    #[cfg(target_arch = "x86")]
    core::arch::asm!("mov {}, fs:[0x18]", out(reg) teb, options(nostack, pure, readonly));
    #[cfg(target_arch = "x86_64")]
    core::arch::asm!("mov {}, gs:[0x30]", out(reg) teb, options(nostack, pure, readonly));
    #[cfg(target_arch = "aarch64")]
    core::arch::asm!("mov {}, x18", out(reg) teb, options(nostack, pure, readonly));
    teb as *mut TEB
}

extern "system" {
    pub fn RtlCreateUnicodeStringFromAsciiz(
        TargetString: *mut _UNICODE_STRING,
        SourceString: LPCSTR,
    );
    pub fn RtlFreeUnicodeString(UnicodeString: *mut _UNICODE_STRING);
}

extern "system" {
    pub fn GetUserDefaultLangID() -> LANGID;
    pub fn GetSystemDefaultLangID() -> LANGID;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn GetLastError() -> DWORD;
    pub fn GetProcessHeap() -> HANDLE;
    pub fn GetVersion() -> DWORD;

    pub fn HeapAlloc(hHeap: HANDLE, dwFlags: DWORD, dwBytes: SIZE_T) -> LPVOID;
    pub fn HeapFree(hHeap: HANDLE, dwFlags: DWORD, lpMem: LPVOID) -> BOOL;

    pub fn LoadLibraryExW(
        lpLibFileName: LPCWSTR,
        hFile: HANDLE,
        dwFlags: DWORD,
    ) -> HMODULE;
    pub fn FreeLibrary(hLibModule: HMODULE) -> BOOL;

    pub fn FindResourceW(
        hModule: HMODULE,
        lpName: LPCWSTR,
        lpType: LPCWSTR,
    ) -> HRSRC;
    pub fn FindResourceExW(
        hModule: HMODULE,
        lpType: LPCWSTR,
        lpName: LPCWSTR,
        wLanguage: WORD,
    ) -> HRSRC;
    pub fn LoadResource(hModule: HMODULE, hResInfo: HRSRC) -> HGLOBAL;
    pub fn LockResource(hResData: HGLOBAL) -> LPVOID;
    pub fn FreeResource(hResData: HGLOBAL) -> BOOL;
    pub fn SizeofResource(hModule: HMODULE, hResInfo: HRSRC) -> DWORD;

    pub fn LZOpenFileW(
        lpFileName: LPWSTR,
        lpReOpenBuf: LPOFSTRUCT,
        wStyle: WORD,
    ) -> HFILE;
    pub fn LZOpenFileA(
        lpFileName: LPSTR,
        lpReOpenBuf: LPOFSTRUCT,
        wStyle: WORD,
    ) -> HFILE;
    pub fn LZClose(hFile: HFILE);
    pub fn LZSeek(hFile: HFILE, lOffset: LONG, nOrigin: i32) -> LONG;
    pub fn LZRead(hFile: HFILE, lpBuffer: LPSTR, nCount: i32) -> i32;
    pub fn LZCopy(hfSource: HFILE, hfDest: HFILE) -> LONG;

    pub fn lstrcmpiA(s1: *const CHAR, s2: *const CHAR) -> i32;
    pub fn lstrcpynA(dest: *mut CHAR, src: *const CHAR, maxlen: i32) -> *mut CHAR;
    pub fn lstrlenW(s: *const WCHAR) -> i32;
    pub fn lstrcpynW(dest: *mut WCHAR, src: *const WCHAR, maxlen: i32) -> *mut WCHAR;

    pub fn MultiByteToWideChar(
        CodePage: UINT,
        dwFlags: DWORD,
        lpMultiByteStr: LPCSTR,
        cbMultiByte: i32,
        lpWideCharStr: LPWSTR,
        cchWideChar: i32,
    ) -> i32;
    pub fn WideCharToMultiByte(
        CodePage: UINT,
        dwFlags: DWORD,
        lpWideCharStr: LPCWSTR,
        cchWideChar: i32,
        lpMultiByteStr: LPSTR,
        cbMultiByte: i32,
        lpDefaultChar: LPCSTR,
        lpUsedDefaultChar: LPBOOL,
    ) -> i32;

    pub fn GetFileAttributesA(lpFileName: LPCSTR) -> DWORD;
    pub fn DeleteFileA(lpFileName: LPCSTR) -> BOOL;
    pub fn MoveFileA(lpExistingFileName: LPCSTR, lpNewFileName: LPCSTR) -> BOOL;
    pub fn GetTempFileNameA(
        lpPathName: LPCSTR,
        lpPrefixString: LPCSTR,
        uUnique: UINT,
        lpTempFileName: LPSTR,
    ) -> UINT;
    pub fn OpenFile(
        lpFileName: *const CHAR,
        lpReOpenBuff: *mut OFSTRUCT,
        uStyle: u32,
    ) -> i32;
    pub fn _lclose(hFile: HFILE) -> HFILE;

    pub fn GetWindowsDirectoryA(lpBuffer: LPSTR, uSize: UINT) -> UINT;
    pub fn GetWindowsDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    pub fn GetSystemDirectoryA(lpBuffer: LPSTR, uSize: UINT) -> UINT;
    pub fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;

}

// Wine debug helpers are C macros, not linkable symbols — provide no-op shims
// (same approach as kernel32-rust/src/prelude.rs).
// LANGIDFROMLCID is a C macro ((WORD)(lcid)), not a linkable symbol.
#[inline]
pub fn LANGIDFROMLCID(lcid: u32) -> LANGID { lcid as LANGID }
#[inline]
pub unsafe fn TRACE_ON(_channel: u32) -> i32 { 0 }
#[inline]
pub unsafe fn debugstr_a(s: LPCSTR) -> LPCSTR { s }
#[inline]
pub unsafe fn debugstr_w(s: LPCWSTR) -> LPCWSTR { s }

// CRT/libc helpers are cdecl (msvcrt exports them undecorated, NOT _name@N), so
// they must live in an extern "C" block, not extern "system" — on i386 the latter
// is stdcall and would look up _strlen@4 etc. which don't exist.
extern "C" {
    pub fn sprintf(buffer: *mut CHAR, format: *const CHAR, ...) -> i32;

    pub fn memcpy(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    pub fn memset(s: *mut c_void, c: i32, n: usize) -> *mut c_void;
    pub fn memcmp(s1: *const c_void, s2: *const c_void, n: usize) -> i32;

    pub fn strlen(s: *const CHAR) -> usize;
    pub fn strcpy(dest: *mut CHAR, src: *const CHAR) -> *mut CHAR;
    pub fn strcat(dest: *mut CHAR, src: *const CHAR) -> *mut CHAR;
    pub fn strrchr(s: *const CHAR, c: i32) -> *mut CHAR;
    pub fn _strnicmp(s1: *const CHAR, s2: *const CHAR, n: usize) -> i32;
    pub fn _wcsnicmp(s1: *const WCHAR, s2: *const WCHAR, n: usize) -> i32;
    pub fn wcsicmp(s1: *const WCHAR, s2: *const WCHAR) -> i32;
    pub fn wcslen(s: *const WCHAR) -> usize;
}

// Wine TRACE/WARN/FIXME are C MACROS (not linkable symbols) — see the macro_rules!
// definitions near the top of this file. They used to be no-op C-variadic *fns*, which
// stable Rust cannot define (`c_variadic`, rust-lang#44930); a macro is both the shape
// Wine actually had and zero-cost, so version-rust needs no C shim at all.

// ============================================================================
// Helper: MAKEINTRESOURCEW (inline)
// ============================================================================
#[inline]
pub unsafe fn MAKEINTRESOURCEW(id: WORD) -> LPWSTR {
    id as usize as LPWSTR
}

// ============================================================================
// Helper: min / max
// ============================================================================
#[inline]
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b { a } else { b }
}

#[inline]
pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b { a } else { b }
}

// ============================================================================
// Helper: ARRAY_SIZE
// ============================================================================
pub fn ARRAY_SIZE<T>(_arr: &[T]) -> usize {
    core::mem::size_of_val(_arr) / core::mem::size_of::<T>()
}

// ============================================================================
// Helper: VersionInfoIs16
// ============================================================================
#[inline]
pub unsafe fn VersionInfoIs16(ver: *const c_void) -> BOOL {
    // C: `((const VS_VERSION_INFO_STRUCT16 *)ver)->szKey[0] >= ' '` where szKey
    // is CHAR (signed). Compare as signed i8 so high bytes (0x80-0xFF) read as
    // negative (< ' ') exactly as the C does — casting to u8 first diverged.
    //
    // szKey[0] is at offset 4 (after wLength/wValueLength) — NOT offset 0. Reading
    // `*(ver as *const CHAR)` tested wLength's low byte, so every 32-bit block whose
    // length's low byte was >= 0x20 took the 16-bit path and every query failed
    // (found by rustconv difftest-contract, version.json).
    if (*(*(ver as *const VS_VERSION_INFO_STRUCT16)).szKey.as_ptr()) >= b' ' as CHAR { 1 } else { 0 }
}

// ============================================================================
// Helper: DWORD_ALIGN
// ============================================================================
#[inline]
pub unsafe fn DWORD_ALIGN(base: *const u8, ptr: *const u8) -> *mut BYTE {
    let diff = (ptr as usize).wrapping_sub(base as usize);
    let aligned = (diff + 3) & !3;
    base.add(aligned) as *mut BYTE
}

// ============================================================================
// VersionInfo16_Value / VersionInfo32_Value
// ============================================================================
#[inline]
pub unsafe fn VersionInfo16_Value(ver: *const VS_VERSION_INFO_STRUCT16) -> LPVOID {
    let key = (*ver).szKey.as_ptr();
    let key_end = key.add(strlen(key) + 1);
    DWORD_ALIGN(ver as *const u8, key_end as *const u8) as LPVOID
}

#[inline]
pub unsafe fn VersionInfo32_Value(ver: *const VS_VERSION_INFO_STRUCT32) -> LPVOID {
    let key = (*ver).szKey.as_ptr();
    let key_end = key.add(wcslen(key) + 1);
    DWORD_ALIGN(ver as *const u8, key_end as *const u8) as LPVOID
}

// ============================================================================
// VersionInfo16_Children / VersionInfo32_Children
// ============================================================================
#[inline]
pub unsafe fn VersionInfo16_Children(ver: *const VS_VERSION_INFO_STRUCT16) -> *const VS_VERSION_INFO_STRUCT16 {
    let val = VersionInfo16_Value(ver);
    let val_len = ((*ver).wValueLength as usize + 3) & !3;
    (val as *const u8).add(val_len) as *const VS_VERSION_INFO_STRUCT16
}

#[inline]
pub unsafe fn VersionInfo32_Children(ver: *const VS_VERSION_INFO_STRUCT32) -> *const VS_VERSION_INFO_STRUCT32 {
    let val = VersionInfo32_Value(ver);
    let multiplier: usize = if (*ver).wType != 0 { 2 } else { 1 };
    let val_len = (((*ver).wValueLength as usize) * multiplier + 3) & !3;
    (val as *const u8).add(val_len) as *const VS_VERSION_INFO_STRUCT32
}

// ============================================================================
// VersionInfo16_Next / VersionInfo32_Next
// ============================================================================
#[inline]
pub unsafe fn VersionInfo16_Next(ver: *const VS_VERSION_INFO_STRUCT16) -> *const VS_VERSION_INFO_STRUCT16 {
    let len = ((*ver).wLength as usize + 3) & !3;
    (ver as *const u8).add(len) as *const VS_VERSION_INFO_STRUCT16
}

#[inline]
pub unsafe fn VersionInfo32_Next(ver: *const VS_VERSION_INFO_STRUCT32) -> *const VS_VERSION_INFO_STRUCT32 {
    let len = ((*ver).wLength as usize + 3) & !3;
    (ver as *const u8).add(len) as *const VS_VERSION_INFO_STRUCT32
}

// ============================================================================
// Panic handler for no_std staticlib
// ============================================================================
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[no_mangle]
extern "C" fn rust_eh_personality() {}

// ############################################################################
// === BELOW THIS LINE: verbatim from version_rust_draft.rs (lines 39-2023) ===
// ############################################################################
// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct NE_NAMEINFO {
    pub offset: WORD,
    pub length: WORD,
    pub flags: WORD,
    pub id: WORD,
    pub handle: WORD,
    pub usage: WORD,
}
pub type PNE_NAMEINFO = *mut NE_NAMEINFO;

#[repr(C)]
pub struct NE_TYPEINFO {
    pub type_id: WORD,
    pub count: WORD,
    pub resloader: DWORD,
}
pub type PNE_TYPEINFO = *mut NE_TYPEINFO;

#[repr(C)]
pub struct VS_VERSION_INFO_STRUCT16 {
    pub wLength: WORD,
    pub wValueLength: WORD,
    pub szKey: [CHAR; 1],
}
pub type PVS_VERSION_INFO_STRUCT16 = *mut VS_VERSION_INFO_STRUCT16;

#[repr(C)]
pub struct VS_VERSION_INFO_STRUCT32 {
    pub wLength: WORD,
    pub wValueLength: WORD,
    pub wType: WORD, /* 1:Text, 0:Binary */
    pub szKey: [WCHAR; 1],
}
pub type PVS_VERSION_INFO_STRUCT32 = *mut VS_VERSION_INFO_STRUCT32;
unsafe fn find_entry_by_id(
    dir: *const IMAGE_RESOURCE_DIRECTORY,
    id: WORD,
    root: *const core::ffi::c_void,
) -> *const IMAGE_RESOURCE_DIRECTORY {
    let entry: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    let mut min: i32;
    let mut max: i32;
    let mut pos: i32;

    entry = dir.offset(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    min = (*dir).NumberOfNamedEntries as i32;
    max = min + (*dir).NumberOfIdEntries as i32 - 1;
    while min <= max {
        pos = (min + max) / 2;
        // C `u.Id` is a USHORT (16-bit) overlaying the low word of the Name
        // union; read it as such so the high word (NameOffset/NameIsString
        // bits) can never leak into the compare.
        if ((*entry.offset(pos as isize)).u.Id & 0xFFFF) == id as DWORD {
            // C `u2.s2.OffsetToDirectory` is a :31 bitfield — the top bit is
            // DataIsDirectory and MUST be masked off before use as an offset.
            return (root as *const u8)
                .offset(((*entry.offset(pos as isize)).u2.s2.OffsetToDirectory & 0x7FFF_FFFF) as isize)
                as *const IMAGE_RESOURCE_DIRECTORY;
        }
        if ((*entry.offset(pos as isize)).u.Id & 0xFFFF) > id as DWORD {
            max = pos - 1;
        } else {
            min = pos + 1;
        }
    }
    core::ptr::null()
}
unsafe fn find_entry_default(
    dir: *const IMAGE_RESOURCE_DIRECTORY,
    root: *const core::ffi::c_void,
) -> *const IMAGE_RESOURCE_DIRECTORY {
    let entry: *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    entry = dir.offset(1) as *const IMAGE_RESOURCE_DIRECTORY_ENTRY;
    // OffsetToDirectory is a :31 bitfield in C — mask off the DataIsDirectory
    // top bit before using it as an offset.
    return (root as *const u8)
        .offset(((*entry).u2.s2.OffsetToDirectory & 0x7FFF_FFFF) as isize) as *const IMAGE_RESOURCE_DIRECTORY;
}

unsafe fn push_language(list: *mut u16, pos: i32, lang: u16) -> i32 {
    let mut i: i32;
    i = 0i32;
    while i < pos {
        if *list.offset(i as isize) == lang {
            return pos;
        }
        i += 1;
    }
    *list.offset(pos as isize) = lang;
    return pos + 1i32;
}
unsafe fn find_entry_language(
    dir: *const IMAGE_RESOURCE_DIRECTORY,
    root: *const core::ffi::c_void,
    flags: DWORD,
) -> *const IMAGE_RESOURCE_DIRECTORY {
    let mut ret: *const IMAGE_RESOURCE_DIRECTORY;
    let mut list: [WORD; 9] = [0; 9];
    let mut pos: i32 = 0;
    let mut i: i32;

    if flags & FILE_VER_GET_LOCALISED != 0 {
        pos = push_language(list.as_mut_ptr(), pos, MAKELANGID(LANG_NEUTRAL, SUBLANG_NEUTRAL));
        pos = push_language(list.as_mut_ptr(), pos, LANGIDFROMLCID((*NtCurrentTeb()).CurrentLocale));
        pos = push_language(list.as_mut_ptr(), pos, GetUserDefaultLangID());
        pos = push_language(
            list.as_mut_ptr(),
            pos,
            MAKELANGID(PRIMARYLANGID(GetUserDefaultLangID()), SUBLANG_NEUTRAL),
        );
        pos = push_language(
            list.as_mut_ptr(),
            pos,
            MAKELANGID(PRIMARYLANGID(GetUserDefaultLangID()), SUBLANG_DEFAULT),
        );
        pos = push_language(list.as_mut_ptr(), pos, GetSystemDefaultLangID());
        pos = push_language(
            list.as_mut_ptr(),
            pos,
            MAKELANGID(PRIMARYLANGID(GetSystemDefaultLangID()), SUBLANG_NEUTRAL),
        );
        pos = push_language(
            list.as_mut_ptr(),
            pos,
            MAKELANGID(PRIMARYLANGID(GetSystemDefaultLangID()), SUBLANG_DEFAULT),
        );
        pos = push_language(
            list.as_mut_ptr(),
            pos,
            MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT),
        );
    } else {
        pos = push_language(
            list.as_mut_ptr(),
            pos,
            MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT),
        );
    }

    i = 0;
    while i < pos {
        ret = find_entry_by_id(dir, list[i as usize], root);
        if !ret.is_null() {
            return ret;
        }
        i += 1;
    }
    return find_entry_default(dir, root);
}
unsafe fn read_xx_header(lzfd: HFILE) -> i32 {

    let mut mzh: IMAGE_DOS_HEADER = core::mem::zeroed();
    let mut magic: [i8; 3] = [0i8; 3];

    LZSeek(lzfd, 0, 0);
    if core::mem::size_of::<IMAGE_DOS_HEADER>() as i32 != LZRead(lzfd, &mut mzh as *mut IMAGE_DOS_HEADER as LPSTR, core::mem::size_of::<IMAGE_DOS_HEADER>() as i32) {
        return 0;
    }
    if mzh.e_magic != IMAGE_DOS_SIGNATURE {
        if memcmp(&mzh as *const IMAGE_DOS_HEADER as *const core::ffi::c_void, "\x7fELF\0".as_ptr() as *const core::ffi::c_void, 4) == 0 {
            return 1;
        }
        if *(&mzh as *const IMAGE_DOS_HEADER as *const u32) == 0xfeedface || *(&mzh as *const IMAGE_DOS_HEADER as *const u32) == 0xcefaedfe {
            return 1;
        }
        return 0;
    }

    LZSeek(lzfd, mzh.e_lfanew as i32, 0);
    if 2 != LZRead(lzfd, magic.as_mut_ptr() as LPSTR, 2) {
        return 0;
    }

    LZSeek(lzfd, mzh.e_lfanew as i32, 0);

    if magic[0] == 'N' as i8 && magic[1] == 'E' as i8 {
        return IMAGE_OS2_SIGNATURE as i32;
    }
    if magic[0] == 'P' as i8 && magic[1] == 'E' as i8 {
        return IMAGE_NT_SIGNATURE as i32;
    }

    magic[2] = '\0' as i8;
    WARN!("Can't handle %s files.\n\0".as_ptr() as *const i8, magic.as_ptr() as *const i8);
    return 0;
}
unsafe fn find_ne_resource(lzfd: HFILE, resLen: *mut DWORD, resOff: *mut DWORD) -> BOOL {
    const typeid: WORD = VS_FILE_INFO | 0x8000;
    const resid: WORD = VS_VERSION_INFO | 0x8000;
    let mut nehd: IMAGE_OS2_HEADER = core::mem::zeroed();
    let mut typeInfo: *mut NE_TYPEINFO = core::ptr::null_mut();
    let mut nameInfo: *mut NE_NAMEINFO = core::ptr::null_mut();
    let nehdoffset: DWORD;
    let mut resTab: LPBYTE = core::ptr::null_mut();
    let resTabSize: DWORD;
    let mut count: i32;

    /* Read in NE header */
    nehdoffset = LZSeek(lzfd, 0, SEEK_CUR) as DWORD;
    if core::mem::size_of::<IMAGE_OS2_HEADER>() as i32 != LZRead(lzfd, &mut nehd as *mut IMAGE_OS2_HEADER as LPSTR, core::mem::size_of::<IMAGE_OS2_HEADER>() as i32) {
        return 0;
    }

    resTabSize = ((nehd.ne_restab as i32) - (nehd.ne_rsrctab as i32)) as DWORD;
    if resTabSize == 0 {
        TRACE!(b"No resources in NE dll\n\0".as_ptr() as LPCSTR);
        return 0;
    }

    /* Read in resource table */
    resTab = HeapAlloc(GetProcessHeap(), 0, resTabSize as usize) as LPBYTE;
    if resTab.is_null() {
        return 0;
    }

    LZSeek(lzfd, (nehd.ne_rsrctab as i32) + nehdoffset as i32, SEEK_SET);
    if resTabSize as i32 != LZRead(lzfd, resTab as LPSTR, resTabSize as i32) {
        HeapFree(GetProcessHeap(), 0, resTab as *mut core::ffi::c_void);
        return 0;
    }

    /* Find resource */
    typeInfo = (resTab.offset(2)) as *mut NE_TYPEINFO;
    while (*typeInfo).type_id != 0 {
        if (*typeInfo).type_id == typeid {
            break;
        }
        typeInfo = (typeInfo.offset(1) as *mut u8).offset((*typeInfo).count as isize * core::mem::size_of::<NE_NAMEINFO>() as isize) as *mut NE_TYPEINFO;
    }
    if (*typeInfo).type_id == 0 {
        TRACE!(b"No typeid entry found\n\0".as_ptr() as LPCSTR);
        HeapFree(GetProcessHeap(), 0, resTab as *mut core::ffi::c_void);
        return 0;
    }

    nameInfo = typeInfo.offset(1) as *mut NE_NAMEINFO;

    count = (*typeInfo).count as i32;
    while count > 0 {
        if (*nameInfo).id == resid {
            break;
        }
        count -= 1;
        nameInfo = nameInfo.offset(1);
    }
    if count == 0 {
        TRACE!(b"No resid entry found\n\0".as_ptr() as LPCSTR);
        HeapFree(GetProcessHeap(), 0, resTab as *mut core::ffi::c_void);
        return 0;
    }

    /* Return resource data */
    if !resLen.is_null() {
        *resLen = ((*nameInfo).length as DWORD) << *(resTab as *mut WORD) as DWORD;
    }
    if !resOff.is_null() {
        *resOff = ((*nameInfo).offset as DWORD) << *(resTab as *mut WORD) as DWORD;
    }

    HeapFree(GetProcessHeap(), 0, resTab as *mut core::ffi::c_void);
    return 1;
}
unsafe fn find_pe_resource(
    lzfd: HFILE,
    resLen: *mut DWORD,
    resOff: *mut DWORD,
    flags: DWORD,
) -> BOOL {
    #[derive(Copy, Clone)]
    union PeHd {
        nt32: IMAGE_NT_HEADERS32,
        nt64: IMAGE_NT_HEADERS64,
    }
    #[allow(invalid_value)]
    let mut pehd: PeHd = unsafe { core::mem::zeroed() };
    let pehdoffset: DWORD;
    let resDataDir: PIMAGE_DATA_DIRECTORY;
    let sections: PIMAGE_SECTION_HEADER;
    let resSection: LPBYTE;
    let section_size: DWORD;
    let data_size: DWORD;
    let resDir: *const core::ffi::c_void;
    let mut resPtr: *const IMAGE_RESOURCE_DIRECTORY;
    let resData: *const IMAGE_RESOURCE_DATA_ENTRY;
    let mut i: i32;
    let mut len: i32;
    let nSections: i32;
    let mut ret: BOOL = 0;

    'done: {
    pehdoffset = LZSeek(lzfd, 0, SEEK_CUR) as DWORD;
    len = LZRead(lzfd, &mut pehd as *mut PeHd as LPSTR, core::mem::size_of::<PeHd>() as i32);
    if len < core::mem::size_of::<IMAGE_FILE_HEADER>() as i32 {
        return 0;
    }
    if len < core::mem::size_of::<PeHd>() as i32 {
        memset(
            (&mut pehd as *mut PeHd as *mut u8).offset(len as isize) as *mut core::ffi::c_void,
            0,
            core::mem::size_of::<PeHd>() - len as usize,
        );
    }

    match pehd.nt32.OptionalHeader.Magic {
        IMAGE_NT_OPTIONAL_HDR32_MAGIC => {
            resDataDir = pehd.nt32.OptionalHeader.DataDirectory.as_mut_ptr().offset(IMAGE_DIRECTORY_ENTRY_RESOURCE as isize);
        }
        IMAGE_NT_OPTIONAL_HDR64_MAGIC => {
            resDataDir = pehd.nt64.OptionalHeader.DataDirectory.as_mut_ptr().offset(IMAGE_DIRECTORY_ENTRY_RESOURCE as isize);
        }
        _ => {
            return 0;
        }
    }

    if (*resDataDir).Size == 0 {
        TRACE!(b"No resources in PE dll\n\0".as_ptr() as LPCSTR);
        return 0;
    }

    nSections = pehd.nt32.FileHeader.NumberOfSections as i32;
    sections = HeapAlloc(
        GetProcessHeap(),
        0,
        (nSections as usize) * core::mem::size_of::<IMAGE_SECTION_HEADER>(),
    ) as PIMAGE_SECTION_HEADER;
    if sections.is_null() {
        return 0;
    }

    len = (core::mem::offset_of!(IMAGE_NT_HEADERS32, OptionalHeader) + pehd.nt32.FileHeader.SizeOfOptionalHeader as usize) as i32;
    LZSeek(lzfd, (pehdoffset + len as DWORD) as LONG, SEEK_SET);

    if (nSections as usize) * core::mem::size_of::<IMAGE_SECTION_HEADER>()
        != LZRead(
            lzfd,
            sections as LPSTR,
            ((nSections as usize) * core::mem::size_of::<IMAGE_SECTION_HEADER>()) as i32,
        ) as usize
    {
        HeapFree(GetProcessHeap(), 0, sections as *mut core::ffi::c_void);
        return 0;
    }

    i = 0;
    while i < nSections {
        if (*resDataDir).VirtualAddress >= (*sections.offset(i as isize)).VirtualAddress
            && (*resDataDir).VirtualAddress
                < (*sections.offset(i as isize)).VirtualAddress
                    + (*sections.offset(i as isize)).SizeOfRawData
        {
            break;
        }
        i += 1;
    }

    if i == nSections {
        HeapFree(GetProcessHeap(), 0, sections as *mut core::ffi::c_void);
        TRACE!(b"Couldn't find resource section\n\0".as_ptr() as LPCSTR);
        return 0;
    }

    data_size = (*sections.offset(i as isize)).SizeOfRawData;
    section_size = max(data_size, (*sections.offset(i as isize)).Misc.VirtualSize);
    resSection = HeapAlloc(
        GetProcessHeap(),
        0,
        section_size as usize,
    ) as LPBYTE;
    if resSection.is_null() {
        HeapFree(GetProcessHeap(), 0, sections as *mut core::ffi::c_void);
        return 0;
    }

    LZSeek(lzfd, (*sections.offset(i as isize)).PointerToRawData as LONG, SEEK_SET);
    if data_size != LZRead(lzfd, resSection as LPSTR, data_size as i32) as DWORD {
        break 'done;
    }
    if data_size < section_size {
        memset(
            (resSection as *mut u8).offset(data_size as isize) as *mut core::ffi::c_void,
            0,
            (section_size - data_size) as usize,
        );
    }

    resDir = (resSection as *const u8)
        .offset(
            ((*resDataDir).VirtualAddress - (*sections.offset(i as isize)).VirtualAddress) as isize,
        ) as *const core::ffi::c_void;

    resPtr = resDir as *const IMAGE_RESOURCE_DIRECTORY;
    resPtr = find_entry_by_id(resPtr, VS_FILE_INFO, resDir);
    if resPtr.is_null() {
        TRACE!(b"No typeid entry found\n\0".as_ptr() as LPCSTR);
        break 'done;
    }
    resPtr = find_entry_by_id(resPtr, VS_VERSION_INFO, resDir);
    if resPtr.is_null() {
        TRACE!(b"No resid entry found\n\0".as_ptr() as LPCSTR);
        break 'done;
    }
    resPtr = find_entry_language(resPtr, resDir, flags);
    if resPtr.is_null() {
        TRACE!(b"No default language entry found\n\0".as_ptr() as LPCSTR);
        break 'done;
    }

    resData = resPtr as *const IMAGE_RESOURCE_DATA_ENTRY;
    i = 0;
    while i < nSections {
        if (*resData).OffsetToData >= (*sections.offset(i as isize)).VirtualAddress
            && (*resData).OffsetToData
                < (*sections.offset(i as isize)).VirtualAddress
                    + (*sections.offset(i as isize)).SizeOfRawData
        {
            break;
        }
        i += 1;
    }

    if i == nSections {
        TRACE!(b"Couldn't find resource data section\n\0".as_ptr() as LPCSTR);
        break 'done;
    }

    if !resLen.is_null() {
        *resLen = (*resData).Size;
    }
    if !resOff.is_null() {
        *resOff = (*resData).OffsetToData
            - (*sections.offset(i as isize)).VirtualAddress
            + (*sections.offset(i as isize)).PointerToRawData;
    }
    ret = 1;

    HeapFree(GetProcessHeap(), 0, resSection as *mut core::ffi::c_void);
    HeapFree(GetProcessHeap(), 0, sections as *mut core::ffi::c_void);
    }
    return ret;
}
unsafe fn find_version_resource(
    lzfd: HFILE,
    reslen: *mut DWORD,
    offset: *mut DWORD,
    flags: DWORD,
) -> DWORD {
    let mut magic: DWORD = read_xx_header(lzfd) as DWORD;

    match magic {
        IMAGE_OS2_SIGNATURE => {
            if find_ne_resource(lzfd, reslen, offset) == 0 {
                magic = 0;
            }
        }
        IMAGE_NT_SIGNATURE => {
            if find_pe_resource(lzfd, reslen, offset, flags) == 0 {
                magic = 0;
            }
        }
        _ => {}
    }
    return magic;
}
unsafe fn print_vffi_debug(vffi: *const VS_FIXEDFILEINFO) {
    let mut versioned_printer: BOOL = 0;

    if ((*vffi).dwFileType == VFT_DLL) || ((*vffi).dwFileType == VFT_DRV) {
        if (*vffi).dwFileSubtype == VFT2_DRV_VERSIONED_PRINTER {
            versioned_printer = 1;
        } else if ((*vffi).dwFileSubtype == VFT2_DRV_PRINTER)
            && ((*vffi).dwFileVersionMS != (*vffi).dwProductVersionMS)
            && ((*vffi).dwFileVersionMS > 0)
            && ((*vffi).dwFileVersionMS <= 3)
        {
            versioned_printer = 1;
        }
    }

    TRACE!(
        b"structversion=%u.%u, \0".as_ptr() as *const i8,
        HIWORD((*vffi).dwStrucVersion),
        LOWORD((*vffi).dwStrucVersion),
    );
    if versioned_printer != 0 {
        let mode: WORD = LOWORD((*vffi).dwFileVersionMS) as WORD;
        let ver_rev: WORD = HIWORD((*vffi).dwFileVersionLS) as WORD;
        TRACE!(
            b"fileversion=%u.%u.%u.%u (%s.major.minor.release), \0".as_ptr() as *const i8,
            (*vffi).dwFileVersionMS,
            HIBYTE(ver_rev),
            LOBYTE(ver_rev),
            LOWORD((*vffi).dwFileVersionLS),
            if mode == 3 {
                b"Usermode\0".as_ptr() as *const i8
            } else if mode <= 2 {
                b"Kernelmode\0".as_ptr() as *const i8
            } else {
                b"?\0".as_ptr() as *const i8
            },
        );
    } else {
        TRACE!(
            b"fileversion=%u.%u.%u.%u, \0".as_ptr() as *const i8,
            HIWORD((*vffi).dwFileVersionMS),
            LOWORD((*vffi).dwFileVersionMS),
            HIWORD((*vffi).dwFileVersionLS),
            LOWORD((*vffi).dwFileVersionLS),
        );
    }
    TRACE!(
        b"productversion=%u.%u.%u.%u\n\0".as_ptr() as *const i8,
        HIWORD((*vffi).dwProductVersionMS),
        LOWORD((*vffi).dwProductVersionMS),
        HIWORD((*vffi).dwProductVersionLS),
        LOWORD((*vffi).dwProductVersionLS),
    );

    TRACE!(
        b"flagmask=0x%x, flags=0x%x %s%s%s%s%s%s\n\0".as_ptr() as *const i8,
        (*vffi).dwFileFlagsMask,
        (*vffi).dwFileFlags,
        if ((*vffi).dwFileFlags & VS_FF_DEBUG) != 0 {
            b"DEBUG,\0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        if ((*vffi).dwFileFlags & VS_FF_PRERELEASE) != 0 {
            b"PRERELEASE,\0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        if ((*vffi).dwFileFlags & VS_FF_PATCHED) != 0 {
            b"PATCHED,\0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        if ((*vffi).dwFileFlags & VS_FF_PRIVATEBUILD) != 0 {
            b"PRIVATEBUILD,\0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        if ((*vffi).dwFileFlags & VS_FF_INFOINFERRED) != 0 {
            b"INFOINFERRED,\0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        if ((*vffi).dwFileFlags & VS_FF_SPECIALBUILD) != 0 {
            b"SPECIALBUILD,\0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
    );

    TRACE!(b"(\0".as_ptr() as *const i8);

    TRACE!(
        b"OS=0x%x.0x%x \0".as_ptr() as *const i8,
        HIWORD((*vffi).dwFileOS),
        LOWORD((*vffi).dwFileOS),
    );

    match (*vffi).dwFileOS & 0xFFFF0000 {
        VOS_DOS => {
            TRACE!(b"DOS,\0".as_ptr() as *const i8);
        }
        VOS_OS216 => {
            TRACE!(b"OS/2-16,\0".as_ptr() as *const i8);
        }
        VOS_OS232 => {
            TRACE!(b"OS/2-32,\0".as_ptr() as *const i8);
        }
        VOS_NT => {
            TRACE!(b"NT,\0".as_ptr() as *const i8);
        }
        _ => {
            TRACE!(
                b"UNKNOWN(0x%x),\0".as_ptr() as *const i8,
                (*vffi).dwFileOS & 0xFFFF0000,
            );
        }
    }

    match LOWORD((*vffi).dwFileOS) as DWORD {
        VOS__BASE => {
            TRACE!(b"BASE\0".as_ptr() as *const i8);
        }
        VOS__WINDOWS16 => {
            TRACE!(b"WIN16\0".as_ptr() as *const i8);
        }
        VOS__WINDOWS32 => {
            TRACE!(b"WIN32\0".as_ptr() as *const i8);
        }
        VOS__PM16 => {
            TRACE!(b"PM16\0".as_ptr() as *const i8);
        }
        VOS__PM32 => {
            TRACE!(b"PM32\0".as_ptr() as *const i8);
        }
        _ => {
            TRACE!(
                b"UNKNOWN(0x%x)\0".as_ptr() as *const i8,
                LOWORD((*vffi).dwFileOS),
            );
        }
    }

    TRACE!(b")\n\0".as_ptr() as *const i8);

    match (*vffi).dwFileType {
        VFT_APP => {
            TRACE!(b"filetype=APP\0".as_ptr() as *const i8);
        }
        VFT_DLL => {
            TRACE!(b"filetype=DLL\0".as_ptr() as *const i8);
            if (*vffi).dwFileSubtype != 0 {
                if versioned_printer != 0 {
                    TRACE!(b",PRINTER\0".as_ptr() as *const i8);
                }
                TRACE!(
                    b" (subtype=0x%x)\0".as_ptr() as *const i8,
                    (*vffi).dwFileSubtype,
                );
            }
        }
        VFT_DRV => {
            TRACE!(b"filetype=DRV,\0".as_ptr() as *const i8);
            match (*vffi).dwFileSubtype {
                VFT2_DRV_PRINTER => {
                    TRACE!(b"PRINTER\0".as_ptr() as *const i8);
                }
                VFT2_DRV_KEYBOARD => {
                    TRACE!(b"KEYBOARD\0".as_ptr() as *const i8);
                }
                VFT2_DRV_LANGUAGE => {
                    TRACE!(b"LANGUAGE\0".as_ptr() as *const i8);
                }
                VFT2_DRV_DISPLAY => {
                    TRACE!(b"DISPLAY\0".as_ptr() as *const i8);
                }
                VFT2_DRV_MOUSE => {
                    TRACE!(b"MOUSE\0".as_ptr() as *const i8);
                }
                VFT2_DRV_NETWORK => {
                    TRACE!(b"NETWORK\0".as_ptr() as *const i8);
                }
                VFT2_DRV_SYSTEM => {
                    TRACE!(b"SYSTEM\0".as_ptr() as *const i8);
                }
                VFT2_DRV_INSTALLABLE => {
                    TRACE!(b"INSTALLABLE\0".as_ptr() as *const i8);
                }
                VFT2_DRV_SOUND => {
                    TRACE!(b"SOUND\0".as_ptr() as *const i8);
                }
                VFT2_DRV_COMM => {
                    TRACE!(b"COMM\0".as_ptr() as *const i8);
                }
                VFT2_DRV_INPUTMETHOD => {
                    TRACE!(b"INPUTMETHOD\0".as_ptr() as *const i8);
                }
                VFT2_DRV_VERSIONED_PRINTER => {
                    TRACE!(b"VERSIONED_PRINTER\0".as_ptr() as *const i8);
                }
                _ => {
                    TRACE!(
                        b"UNKNOWN(0x%x)\0".as_ptr() as *const i8,
                        (*vffi).dwFileSubtype,
                    );
                }
            }
        }
        VFT_FONT => {
            TRACE!(b"filetype=FONT,\0".as_ptr() as *const i8);
            match (*vffi).dwFileSubtype {
                VFT2_FONT_RASTER => {
                    TRACE!(b"RASTER\0".as_ptr() as *const i8);
                }
                VFT2_FONT_VECTOR => {
                    TRACE!(b"VECTOR\0".as_ptr() as *const i8);
                }
                VFT2_FONT_TRUETYPE => {
                    TRACE!(b"TRUETYPE\0".as_ptr() as *const i8);
                }
                _ => {
                    TRACE!(
                        b"UNKNOWN(0x%x)\0".as_ptr() as *const i8,
                        (*vffi).dwFileSubtype,
                    );
                }
            }
        }
        VFT_VXD => {
            TRACE!(b"filetype=VXD\0".as_ptr() as *const i8);
        }
        VFT_STATIC_LIB => {
            TRACE!(b"filetype=STATIC_LIB\0".as_ptr() as *const i8);
        }
        _ => {
            TRACE!(
                b"filetype=Unknown(0x%x)\0".as_ptr() as *const i8,
                (*vffi).dwFileType,
            );
        }
    }

    TRACE!(b"\n\0".as_ptr() as *const i8);
    TRACE!(
        b"filedate=0x%x.0x%x\n\0".as_ptr() as *const i8,
        (*vffi).dwFileDateMS,
        (*vffi).dwFileDateLS,
    );
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeW(
    filename: LPCWSTR,
    handle: LPDWORD,
) -> DWORD {
    return GetFileVersionInfoSizeExW(FILE_VER_GET_LOCALISED, filename, handle);
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeA(
    filename: LPCSTR,
    handle: LPDWORD,
) -> DWORD {
    return GetFileVersionInfoSizeExA(FILE_VER_GET_LOCALISED, filename, handle);
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeExW(
    flags: DWORD,
    filename: LPCWSTR,
    handle: LPDWORD,
) -> DWORD {
    let mut len: DWORD = 0;
    let mut offset: DWORD = 0;
    let mut magic: DWORD = 1;
    let mut lzfd: HFILE = 0;
    let mut hModule: HMODULE = core::ptr::null_mut();
    let mut ofs: OFSTRUCT = core::mem::zeroed();

    TRACE!(
        b"(0x%x,%s,%p)\n\0".as_ptr() as *const i8,
        flags,
        debugstr_w(filename),
        handle,
    );

    if !handle.is_null() {
        *handle = 0;
    }

    if filename.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    if *filename == 0 {
        SetLastError(ERROR_BAD_PATHNAME);
        return 0;
    }
    if flags & !FILE_VER_GET_LOCALISED != 0 {
        FIXME!(
            b"flags 0x%x ignored\n\0".as_ptr() as *const i8,
            flags & !FILE_VER_GET_LOCALISED,
        );
    }

    lzfd = LZOpenFileW(filename as *mut u16, &mut ofs, OF_READ as WORD);
    if lzfd != HFILE_ERROR {
        magic = find_version_resource(lzfd, &mut len, &mut offset, flags);
        LZClose(lzfd);
    }

    if magic == 1 {
        hModule = LoadLibraryExW(filename, 0 as HANDLE, LOAD_LIBRARY_AS_DATAFILE as DWORD);
        if !hModule.is_null() {
            let mut hRsrc: HRSRC = core::ptr::null_mut();
            if flags & FILE_VER_GET_LOCALISED == 0 {
                let english: LANGID = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
                hRsrc = FindResourceExW(
                    hModule,
                    MAKEINTRESOURCEW(VS_VERSION_INFO),
                    VS_FILE_INFO as *mut u16,
                    english,
                );
            }
            if hRsrc.is_null() {
                hRsrc = FindResourceW(
                    hModule,
                    MAKEINTRESOURCEW(VS_VERSION_INFO),
                    VS_FILE_INFO as *mut u16,
                );
            }
            if !hRsrc.is_null() {
                magic = IMAGE_NT_SIGNATURE;
                len = SizeofResource(hModule, hRsrc);
            }
            FreeLibrary(hModule);
        }
    }

    match magic {
        IMAGE_OS2_SIGNATURE => {
            SetLastError(0);
            return (len - core::mem::size_of::<VS_FIXEDFILEINFO>() as DWORD) * 4;
        }
        IMAGE_NT_SIGNATURE => {
            SetLastError(0);
            return (len * 2) + 4;
        }
        _ => {
            if lzfd == HFILE_ERROR {
                SetLastError(ofs.nErrCode as DWORD as DWORD);
            } else if GetVersion() & 0x80000000 != 0 {
                SetLastError(ERROR_FILE_NOT_FOUND);
            } else {
                SetLastError(ERROR_RESOURCE_DATA_NOT_FOUND);
            }
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoSizeExA(
    flags: DWORD,
    filename: LPCSTR,
    handle: LPDWORD,
) -> DWORD {
    let mut filenameW: _UNICODE_STRING = core::mem::zeroed();
    let retval: DWORD;

    TRACE!(b"(0x%x,%s,%p)\n\0".as_ptr() as *const i8, flags, debugstr_a(filename), handle);

    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut filenameW, filename);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    retval = GetFileVersionInfoSizeExW(flags, filenameW.Buffer, handle);

    RtlFreeUnicodeString(&mut filenameW);

    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoExW(
    flags: DWORD,
    filename: LPCWSTR,
    handle: DWORD,
    datasize: DWORD,
    data: LPVOID,
) -> BOOL {
    static signature: [u8; 4] = [b'F', b'E', b'2', b'X'];
    let mut len: DWORD = 0;
    let mut offset: DWORD = 0;
    let mut magic: DWORD = 1;
    let lzfd: HFILE;
    let mut ofs: OFSTRUCT = core::mem::zeroed();
    let hModule: HMODULE;
    let vvis: *mut VS_VERSION_INFO_STRUCT32 = data as *mut VS_VERSION_INFO_STRUCT32;

    TRACE!(
        b"(0x%x,%s,%d,size=%d,data=%p)\n\0".as_ptr() as *const i8,
        flags,
        debugstr_w(filename),
        handle,
        datasize,
        data,
    );

    if data.is_null() {
        SetLastError(ERROR_INVALID_DATA);
        return 0;
    }
    if (flags & !FILE_VER_GET_LOCALISED) != 0 {
        FIXME!(
            b"flags 0x%x ignored\n\0".as_ptr() as *const i8,
            flags & !FILE_VER_GET_LOCALISED,
        );
    }

    lzfd = LZOpenFileW(filename as LPWSTR, &mut ofs, OF_READ as WORD);
    if lzfd != HFILE_ERROR {
        magic = find_version_resource(lzfd, &mut len, &mut offset, flags);
        if magic > 1 {
            LZSeek(lzfd, offset as LONG, 0);
            len = LZRead(lzfd, data as LPSTR, min(len, datasize) as i32) as DWORD;
        }
        LZClose(lzfd);
    }

    if (magic == 1)
        && {
            hModule = LoadLibraryExW(filename, 0 as HMODULE, LOAD_LIBRARY_AS_DATAFILE);
            hModule != 0 as HMODULE
        }
    {
        let mut hRsrc: HRSRC = 0 as HRSRC;
        if (flags & FILE_VER_GET_LOCALISED) == 0 {
            let english: LANGID = MAKELANGID(LANG_ENGLISH, SUBLANG_DEFAULT);
            hRsrc = FindResourceExW(
                hModule,
                MAKEINTRESOURCEW(VS_VERSION_INFO),
                VS_FILE_INFO as LPWSTR,
                english,
            );
        }
        if hRsrc.is_null() {
            hRsrc = FindResourceW(
                hModule,
                MAKEINTRESOURCEW(VS_VERSION_INFO),
                VS_FILE_INFO as LPWSTR,
            );
        }
        if !hRsrc.is_null() {
            let hMem: HGLOBAL = LoadResource(hModule, hRsrc);
            magic = IMAGE_NT_SIGNATURE;
            len = min(SizeofResource(hModule, hRsrc), datasize);
            memcpy(data, LockResource(hMem), len as usize);
            FreeResource(hMem);
        }
        FreeLibrary(hModule);
    }

    match magic {
        IMAGE_OS2_SIGNATURE => {
            if TRACE_ON(__wine_dbch_ver) != 0 {
                print_vffi_debug(
                    VersionInfo16_Value(data as *const VS_VERSION_INFO_STRUCT16)
                        as *const VS_FIXEDFILEINFO,
                );
            }
            SetLastError(0);
            return 1;
        }
        IMAGE_NT_SIGNATURE => {
            len = (*vvis).wLength as DWORD + core::mem::size_of_val(&signature) as DWORD;
            if datasize >= len {
                memcpy(
                    (data as *mut u8).offset((*vvis).wLength as isize) as *mut core::ffi::c_void,
                    signature.as_ptr() as *const core::ffi::c_void,
                    core::mem::size_of_val(&signature),
                );
            }
            if TRACE_ON(__wine_dbch_ver) != 0 {
                print_vffi_debug(VersionInfo32_Value(vvis) as *const VS_FIXEDFILEINFO);
            }
            SetLastError(0);
            return 1;
        }
        _ => {
            SetLastError(if lzfd == HFILE_ERROR {
                ofs.nErrCode as DWORD
            } else {
                ERROR_RESOURCE_DATA_NOT_FOUND
            });
            return 0;
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoExA(
    flags: DWORD,
    filename: LPCSTR,
    handle: DWORD,
    datasize: DWORD,
    data: LPVOID,
) -> BOOL {
    let mut filenameW: _UNICODE_STRING = core::mem::zeroed();
    let retval: BOOL;

    TRACE!(
        "(0x%x,%s,%d,size=%d,data=%p)\n".as_ptr() as *const i8,
        flags,
        debugstr_a(filename),
        handle,
        datasize,
        data,
    );

    if !filename.is_null() {
        RtlCreateUnicodeStringFromAsciiz(&mut filenameW, filename);
    } else {
        filenameW.Buffer = core::ptr::null_mut();
    }

    retval = GetFileVersionInfoExW(flags, filenameW.Buffer, handle, datasize, data);

    RtlFreeUnicodeString(&mut filenameW);

    return retval;
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoW(
    filename: LPCWSTR,
    handle: DWORD,
    datasize: DWORD,
    data: LPVOID,
) -> BOOL {
    return GetFileVersionInfoExW(FILE_VER_GET_LOCALISED, filename, handle, datasize, data);
}

#[no_mangle]
pub unsafe extern "system" fn GetFileVersionInfoA(
    filename: LPCSTR,
    handle: DWORD,
    datasize: DWORD,
    data: LPVOID,
) -> BOOL {
    return GetFileVersionInfoExA(FILE_VER_GET_LOCALISED, filename, handle, datasize, data);
}
unsafe fn VersionInfo16_FindChild(
    info: *const VS_VERSION_INFO_STRUCT16,
    szKey: LPCSTR,
    cbKey: UINT,
) -> *const VS_VERSION_INFO_STRUCT16 {
    let mut child: *const VS_VERSION_INFO_STRUCT16 = VersionInfo16_Children(info);

    while (child as *const u8) < (info as *const u8).add((*info).wLength as usize) {
        // version.c declares `CHAR szKey[1]` / `WCHAR szKey[1]` -- the key string runs
        // past the struct, so bindgen's `[T; 1]` panicked reading its terminator for
        // any key longer than one character, i.e. every real key.
        if _strnicmp((*child).szKey.as_ptr(), szKey, cbKey as usize) == 0
            && (&raw const (*child).szKey[0]).add(cbKey as usize).read() == 0 {
            return child;
        }

        if (*child).wLength == 0 {
            return core::ptr::null();
        }
        child = VersionInfo16_Next(child);
    }

    core::ptr::null()
}
unsafe fn VersionInfo32_FindChild(
    info: *const VS_VERSION_INFO_STRUCT32,
    szKey: LPCWSTR,
    cbKey: UINT,
) -> *const VS_VERSION_INFO_STRUCT32 {
    let mut child: *const VS_VERSION_INFO_STRUCT32 = VersionInfo32_Children(info);

    while (child as *const u8) < (info as *const u8).add((*info).wLength as usize) {
        if _wcsnicmp((*child).szKey.as_ptr(), szKey, cbKey as usize) == 0
            && (&raw const (*child).szKey[0]).add(cbKey as usize).read() == 0 {
            return child;
        }

        if (*child).wLength == 0 {
            return core::ptr::null();
        }
        child = VersionInfo32_Next(child);
    }

    core::ptr::null()
}
unsafe fn VersionInfo16_QueryValue(
    mut info: *const VS_VERSION_INFO_STRUCT16,
    mut lpSubBlock: LPCSTR,
    lplpBuffer: *mut LPVOID,
    puLen: *mut UINT,
) -> BOOL {
    while *lpSubBlock != 0 {
        /* Find next path component */
        let mut lpNextSlash: LPCSTR;
        lpNextSlash = lpSubBlock;
        while *lpNextSlash != 0 {
            if *lpNextSlash == b'\\' as i8 {
                break;
            }
            lpNextSlash = lpNextSlash.offset(1);
        }

        /* Skip empty components */
        if lpNextSlash == lpSubBlock {
            lpSubBlock = lpSubBlock.offset(1);
            continue;
        }

        /* We have a non-empty component: search info for key */
        info = VersionInfo16_FindChild(info, lpSubBlock, (lpNextSlash as isize - lpSubBlock as isize) as UINT);
        if info.is_null() {
            if !puLen.is_null() {
                *puLen = 0;
            }
            SetLastError(ERROR_RESOURCE_TYPE_NOT_FOUND);
            return 0;
        }

        /* Skip path component */
        lpSubBlock = lpNextSlash;
    }

    /* Return value */
    *lplpBuffer = VersionInfo16_Value(info);
    if !puLen.is_null() {
        *puLen = (*info).wValueLength as UINT;
    }

    return 1;
}
unsafe fn VersionInfo32_QueryValue(
    mut info: *const VS_VERSION_INFO_STRUCT32,
    mut lpSubBlock: LPCWSTR,
    lplpBuffer: *mut LPVOID,
    puLen: *mut UINT,
    pbText: *mut BOOL,
) -> BOOL {
    TRACE!("lpSubBlock : (%s)\n\0".as_ptr() as *const i8, debugstr_w(lpSubBlock));

    while *lpSubBlock != 0 {
        let mut lpNextSlash: LPCWSTR;
        lpNextSlash = lpSubBlock;
        while *lpNextSlash != 0 {
            if *lpNextSlash == '\\' as u16 {
                break;
            }
            lpNextSlash = lpNextSlash.offset(1);
        }

        if lpNextSlash == lpSubBlock {
            lpSubBlock = lpSubBlock.offset(1);
            continue;
        }

        // C's `lpNextSlash - lpSubBlock` on LPCWSTR counts WCHARs; subtracting the
        // addresses as integers counted BYTES, so every key compared twice its length
        // and no 32-bit child was ever found (rustconv difftest-contract, version.json).
        info = VersionInfo32_FindChild(info, lpSubBlock, lpNextSlash.offset_from(lpSubBlock) as UINT);
        if info.is_null() {
            if !puLen.is_null() {
                *puLen = 0;
            }
            SetLastError(ERROR_RESOURCE_TYPE_NOT_FOUND);
            return 0;
        }

        lpSubBlock = lpNextSlash;
    }

    *lplpBuffer = VersionInfo32_Value(info);

    if (*info).wValueLength == 0 {
        *lplpBuffer = (*info).szKey.as_ptr().offset(wcslen((*info).szKey.as_ptr()) as isize) as LPVOID;
    }

    if !puLen.is_null() {
        *puLen = (*info).wValueLength as UINT;
    }
    if !pbText.is_null() {
        *pbText = (*info).wType as BOOL;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn VerQueryValueA(
    pBlock: LPCVOID,
    lpSubBlock: LPCSTR,
    lplpBuffer: *mut LPVOID,
    puLen: PUINT,
) -> BOOL {
    static rootA: [i8; 2] = [b'\\' as i8, 0];
    let info: *const VS_VERSION_INFO_STRUCT16 = pBlock as *const VS_VERSION_INFO_STRUCT16;

    TRACE!(
        b"(%p,%s,%p,%p)\n\0".as_ptr() as *const i8,
        pBlock,
        debugstr_a(lpSubBlock),
        lplpBuffer,
        puLen,
    );

    if pBlock.is_null() {
        return 0;
    }

    let lpSubBlock: LPCSTR = if lpSubBlock.is_null() || *lpSubBlock == 0 {
        rootA.as_ptr()
    } else {
        lpSubBlock
    };

    if VersionInfoIs16(info as *const c_void) == 0 {
        let ret: BOOL;
        let mut isText: BOOL = 0;
        let mut len: INT;
        let lpSubBlockW: LPWSTR;
        let mut value_len: UINT = 0;

        len = MultiByteToWideChar(CP_ACP, 0, lpSubBlock, -1, core::ptr::null_mut(), 0);
        lpSubBlockW = HeapAlloc(
            GetProcessHeap(),
            0,
            (len as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
        ) as LPWSTR;

        if lpSubBlockW.is_null() {
            return 0;
        }

        MultiByteToWideChar(CP_ACP, 0, lpSubBlock, -1, lpSubBlockW, len);

        ret = VersionInfo32_QueryValue(
            pBlock as *const VS_VERSION_INFO_STRUCT32,
            lpSubBlockW,
            lplpBuffer,
            &mut value_len,
            &mut isText,
        );
        if !puLen.is_null() {
            *puLen = value_len;
        }

        HeapFree(GetProcessHeap(), 0, lpSubBlockW as LPVOID);

        if ret != 0 && isText != 0 {
            let lpBufferA: LPSTR = (pBlock as *const u8)
                .add((*info).wLength as usize)
                .add(4) as LPSTR;
            let pos: DWORD = (*lplpBuffer as *const u8)
                .offset_from(pBlock as *const u8) as DWORD;
            len = WideCharToMultiByte(
                CP_ACP,
                0,
                *lplpBuffer as LPCWSTR,
                value_len as i32,
                (lpBufferA as *mut u8).add(pos as usize) as LPSTR,
                ((*info).wLength as DWORD).wrapping_sub(pos) as i32,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            *lplpBuffer = (lpBufferA as *mut u8).add(pos as usize) as LPVOID;
            if !puLen.is_null() {
                *puLen = len as UINT;
            }
        }
        return ret;
    }

    return VersionInfo16_QueryValue(info, lpSubBlock, lplpBuffer, puLen);
}

#[no_mangle]
pub unsafe extern "system" fn VerQueryValueW(
    pBlock: LPCVOID,
    lpSubBlock: LPCWSTR,
    lplpBuffer: *mut LPVOID,
    puLen: *mut UINT,
) -> BOOL {
    static rootW: [WCHAR; 2] = [b'\\' as WCHAR, 0];
    // version.c: `'\\','V','a','r','F','i','l','e','I','n','f','o','\\','T','r','a','n',
    // 's','l','a','t','i','o','n',0`. The translation had truncated it to "\\VarFileInfo\\T",
    // so a 16-bit block's Translation value was wrongly re-encoded as a string.
    static varfileinfoW: [WCHAR; 25] = [
        b'\\' as WCHAR, b'V' as WCHAR, b'a' as WCHAR, b'r' as WCHAR,
        b'F' as WCHAR, b'i' as WCHAR, b'l' as WCHAR, b'e' as WCHAR,
        b'I' as WCHAR, b'n' as WCHAR, b'f' as WCHAR, b'o' as WCHAR,
        b'\\' as WCHAR, b'T' as WCHAR, b'r' as WCHAR, b'a' as WCHAR,
        b'n' as WCHAR, b's' as WCHAR, b'l' as WCHAR, b'a' as WCHAR,
        b't' as WCHAR, b'i' as WCHAR, b'o' as WCHAR, b'n' as WCHAR, 0,
    ];

    let info: *const VS_VERSION_INFO_STRUCT32 = pBlock as *const VS_VERSION_INFO_STRUCT32;

    TRACE!(
        b"(%p,%s,%p,%p)\n\0".as_ptr() as *const i8,
        pBlock,
        debugstr_w(lpSubBlock),
        lplpBuffer,
        puLen,
    );

    if pBlock.is_null() {
        return 0;
    }

    let lpSubBlock: LPCWSTR = if lpSubBlock.is_null() || *lpSubBlock == 0 {
        rootW.as_ptr()
    } else {
        lpSubBlock
    };

    if VersionInfoIs16(info as *const c_void) != 0 {
        let ret: BOOL;
        let mut len: i32;
        let lpSubBlockA: LPSTR;

        len = WideCharToMultiByte(CP_ACP, 0, lpSubBlock, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut());
        lpSubBlockA = HeapAlloc(GetProcessHeap(), 0, (len as usize) * core::mem::size_of::<CHAR>()) as LPSTR;

        if lpSubBlockA.is_null() {
            return 0;
        }

        WideCharToMultiByte(CP_ACP, 0, lpSubBlock, -1, lpSubBlockA, len, core::ptr::null_mut(), core::ptr::null_mut());

        ret = VersionInfo16_QueryValue(pBlock as *const VS_VERSION_INFO_STRUCT16, lpSubBlockA, lplpBuffer, puLen);

        HeapFree(GetProcessHeap(), 0, lpSubBlockA as LPVOID);

        if ret != 0 && wcsicmp(lpSubBlock, rootW.as_ptr()) != 0 && wcsicmp(lpSubBlock, varfileinfoW.as_ptr()) != 0 {
            let lpBufferW: LPWSTR = (pBlock as LPSTR).add((*info).wLength as usize) as LPWSTR;
            let pos: DWORD = (*lplpBuffer as LPCSTR).offset_from(pBlock as LPCSTR) as DWORD;
            let max: DWORD = (((*info).wLength as DWORD) - core::mem::size_of::<VS_FIXEDFILEINFO>() as DWORD) * 4 - (*info).wLength as DWORD;

            len = MultiByteToWideChar(
                CP_ACP,
                0,
                *lplpBuffer as LPCSTR,
                -1,
                lpBufferW.offset(pos as isize),
                (max as usize / core::mem::size_of::<WCHAR>()) as i32 - pos as i32,
            );
            *lplpBuffer = lpBufferW.offset(pos as isize) as LPVOID;
            if !puLen.is_null() {
                *puLen = len as UINT;
            }
        }
        return ret;
    }

    return VersionInfo32_QueryValue(info, lpSubBlock, lplpBuffer, puLen, core::ptr::null_mut());
}
unsafe fn testFileExistenceA(
    path: *const i8,
    file: *const i8,
    excl: i32,
) -> i32 {
    let mut filename: [i8; 1024] = [0i8; 1024];
    let filenamelen: i32;
    let mut fileinfo: OFSTRUCT = core::mem::zeroed();
    fileinfo.cBytes = core::mem::size_of::<OFSTRUCT>() as u8;

    if !path.is_null() {
        strcpy(filename.as_mut_ptr(), path);
        filenamelen = strlen(filename.as_mut_ptr()) as i32;

        if filenamelen != 0 {
            if filename[(filenamelen - 1) as usize] != '\\' as i8 {
                strcat(filename.as_mut_ptr(), b"\\\0".as_ptr() as *const i8);
            }
        } else {
            strcpy(filename.as_mut_ptr(), b".\\\0".as_ptr() as *const i8);
        }
    } else {
        filename[0] = 0i8;
    }

    strcat(filename.as_mut_ptr(), file);

    let open_flags: u32 = 0x0001u32 | if excl != 0 { 0x0004u32 } else { 0u32 };
    (OpenFile(filename.as_mut_ptr(), &mut fileinfo, open_flags) != -1i32) as i32
}
unsafe fn testFileExistenceW(path: *const u16, file: *const u16, excl: i32) -> i32 {
    let filename: *mut i8;
    let pathlen: u32;
    let filelen: u32;
    let ret: i32;
    let mut fileinfo: OFSTRUCT = core::mem::zeroed();

    fileinfo.cBytes = core::mem::size_of::<OFSTRUCT>() as u8;

    pathlen = WideCharToMultiByte(0, 0, path, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as u32;
    filelen = WideCharToMultiByte(0, 0, file, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as u32;
    filename = HeapAlloc(GetProcessHeap(), 0, (pathlen + filelen + 2) as usize) as *mut i8;

    WideCharToMultiByte(0, 0, path, -1, filename, pathlen as i32, core::ptr::null_mut(), core::ptr::null_mut());
    /* Add a trailing \ if necessary */
    if pathlen > 1 {
        if *filename.offset((pathlen - 2) as isize) != '\\' as i8 {
            strcpy(filename.offset((pathlen - 1) as isize), b"\\\0".as_ptr() as *const i8);
        }
    } else {
        /* specify the current directory */
        strcpy(filename, b".\\\0".as_ptr() as *const i8);
    }

    WideCharToMultiByte(0, 0, file, -1, filename.offset(strlen(filename) as isize), filelen as i32, core::ptr::null_mut(), core::ptr::null_mut());

    ret = if OpenFile(filename, &mut fileinfo as *mut OFSTRUCT, 0x0001 | if excl != 0 { 0x00000040 } else { 0 }) != -1 { 1 } else { 0 };
    HeapFree(GetProcessHeap(), 0, filename as *mut core::ffi::c_void);
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn VerFindFileA(
    flags: DWORD,
    lpszFilename: LPCSTR,
    lpszWinDir: LPCSTR,
    lpszAppDir: LPCSTR,
    lpszCurDir: LPSTR,
    lpuCurDirLen: PUINT,
    lpszDestDir: LPSTR,
    lpuDestDirLen: PUINT,
) -> DWORD {
    let mut retval: DWORD = 0;
    let mut curDir: *const i8;
    let destDir: *const i8;
    let curDirSizeReq: u32;
    let destDirSizeReq: u32;
    let mut winDir: [i8; MAX_PATH as usize] = [0i8; MAX_PATH as usize];
    let mut systemDir: [i8; MAX_PATH as usize] = [0i8; MAX_PATH as usize];

    TRACE!(
        b"flags = %x filename=%s windir=%s appdir=%s curdirlen=%p(%u) destdirlen=%p(%u)\n\0"
            .as_ptr() as *const i8,
        flags,
        debugstr_a(lpszFilename),
        debugstr_a(lpszWinDir),
        debugstr_a(lpszAppDir),
        lpuCurDirLen,
        if !lpuCurDirLen.is_null() { *lpuCurDirLen } else { 0 },
        lpuDestDirLen,
        if !lpuDestDirLen.is_null() { *lpuDestDirLen } else { 0 },
    );

    GetSystemDirectoryA(systemDir.as_mut_ptr(), core::mem::size_of::<[i8; MAX_PATH as usize]>() as UINT);
    curDir = b"\0".as_ptr() as *const i8;

    if flags & VFFF_ISSHAREDFILE != 0 {
        destDir = systemDir.as_mut_ptr();
        if !lpszFilename.is_null() {
            if testFileExistenceA(destDir, lpszFilename, 0) != 0 {
                curDir = destDir;
            } else if !lpszAppDir.is_null() && testFileExistenceA(lpszAppDir, lpszFilename, 0) != 0 {
                curDir = lpszAppDir;
            }

            if testFileExistenceA(systemDir.as_mut_ptr(), lpszFilename, 0) == 0 {
                retval |= VFF_CURNEDEST;
            }
        }
    } else {
        destDir = if !lpszAppDir.is_null() { lpszAppDir } else { b"\0".as_ptr() as *const i8 };
        if !lpszFilename.is_null() {
            GetWindowsDirectoryA(winDir.as_mut_ptr(), MAX_PATH as UINT);
            if testFileExistenceA(destDir, lpszFilename, 0) != 0 {
                curDir = destDir;
            } else if testFileExistenceA(winDir.as_mut_ptr(), lpszFilename, 0) != 0 {
                curDir = winDir.as_mut_ptr();
            } else if testFileExistenceA(systemDir.as_mut_ptr(), lpszFilename, 0) != 0 {
                curDir = systemDir.as_mut_ptr();
            }

            if !lpszAppDir.is_null() && *lpszAppDir != 0 {
                if testFileExistenceA(lpszAppDir, lpszFilename, 0) == 0 {
                    retval |= VFF_CURNEDEST;
                }
            } else if testFileExistenceA(core::ptr::null_mut(), lpszFilename, 0) != 0 {
                retval |= VFF_CURNEDEST;
            }
        }
    }

    if !lpszFilename.is_null() && testFileExistenceA(curDir, lpszFilename, 0) != 0 {
        if !lpszFilename.is_null() && testFileExistenceA(curDir, lpszFilename, 1) == 0 {
            retval |= VFF_FILEINUSE;
        }
    }

    curDirSizeReq = (strlen(curDir) + 1) as u32;
    destDirSizeReq = (strlen(destDir) + 1) as u32;

    if !lpuDestDirLen.is_null() && !lpszDestDir.is_null() {
        if *lpuDestDirLen < destDirSizeReq {
            retval |= VFF_BUFFTOOSMALL;
        }
        lstrcpynA(lpszDestDir, destDir, *lpuDestDirLen as i32);
        *lpuDestDirLen = destDirSizeReq;
    }
    if !lpuCurDirLen.is_null() && !lpszCurDir.is_null() {
        if *lpuCurDirLen < curDirSizeReq {
            retval |= VFF_BUFFTOOSMALL;
        }
        lstrcpynA(lpszCurDir, curDir, *lpuCurDirLen as i32);
        *lpuCurDirLen = curDirSizeReq;
    }

    TRACE!(
        b"ret = %u (%s%s%s) curdir=%s destdir=%s\n\0".as_ptr() as *const i8,
        retval,
        if retval & VFF_CURNEDEST != 0 {
            b"VFF_CURNEDEST \0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        if retval & VFF_FILEINUSE != 0 {
            b"VFF_FILEINUSE \0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        if retval & VFF_BUFFTOOSMALL != 0 {
            b"VFF_BUFFTOOSMALL \0".as_ptr() as *const i8
        } else {
            b"\0".as_ptr() as *const i8
        },
        debugstr_a(lpszCurDir),
        debugstr_a(lpszDestDir),
    );

    retval
}

#[no_mangle]
pub unsafe extern "system" fn VerFindFileW(
    flags: DWORD,
    lpszFilename: LPCWSTR,
    lpszWinDir: LPCWSTR,
    lpszAppDir: LPCWSTR,
    lpszCurDir: LPWSTR,
    lpuCurDirLen: PUINT,
    lpszDestDir: LPWSTR,
    lpuDestDirLen: PUINT,
) -> DWORD {
    static emptyW: WCHAR = 0u16;
    let mut retval: DWORD = 0;
    let mut curDir: *const WCHAR;
    let destDir: *const WCHAR;
    let curDirSizeReq: u32;
    let destDirSizeReq: u32;
    let mut winDir: [WCHAR; MAX_PATH as usize] = [0u16; MAX_PATH as usize];
    let mut systemDir: [WCHAR; MAX_PATH as usize] = [0u16; MAX_PATH as usize];

    TRACE!(
        b"flags = %x filename=%s windir=%s appdir=%s curdirlen=%p(%u) destdirlen=%p(%u)\n\0".as_ptr() as LPCSTR,
        flags,
        debugstr_w(lpszFilename),
        debugstr_w(lpszWinDir),
        debugstr_w(lpszAppDir),
        lpuCurDirLen,
        if !lpuCurDirLen.is_null() { *lpuCurDirLen } else { 0 },
        lpuDestDirLen,
        if !lpuDestDirLen.is_null() { *lpuDestDirLen } else { 0 }
    );

    GetSystemDirectoryW(systemDir.as_mut_ptr(), ARRAY_SIZE(&systemDir) as u32);
    curDir = &emptyW as *const WCHAR;

    if (flags & VFFF_ISSHAREDFILE) != 0 {
        destDir = systemDir.as_ptr();
        if !lpszFilename.is_null() {
            if testFileExistenceW(destDir, lpszFilename, 0) != 0 {
                curDir = destDir;
            } else if !lpszAppDir.is_null()
                && testFileExistenceW(lpszAppDir, lpszFilename, 0) != 0
            {
                curDir = lpszAppDir;
                retval |= VFF_CURNEDEST;
            }
        }
    } else {
        destDir = if !lpszAppDir.is_null() {
            lpszAppDir
        } else {
            &emptyW as *const WCHAR
        };
        if !lpszFilename.is_null() {
            GetWindowsDirectoryW(winDir.as_mut_ptr(), MAX_PATH as u32);
            if testFileExistenceW(destDir, lpszFilename, 0) != 0 {
                curDir = destDir;
            } else if testFileExistenceW(winDir.as_ptr(), lpszFilename, 0) != 0 {
                curDir = winDir.as_ptr();
                retval |= VFF_CURNEDEST;
            } else if testFileExistenceW(systemDir.as_ptr(), lpszFilename, 0) != 0 {
                curDir = systemDir.as_ptr();
                retval |= VFF_CURNEDEST;
            }
        }
    }

    if !lpszFilename.is_null() && testFileExistenceW(curDir, lpszFilename, 1) == 0 {
        retval |= VFF_FILEINUSE;
    }

    curDirSizeReq = (lstrlenW(curDir) + 1) as u32;
    destDirSizeReq = (lstrlenW(destDir) + 1) as u32;

    if !lpuDestDirLen.is_null() && !lpszDestDir.is_null() {
        if *lpuDestDirLen < destDirSizeReq {
            retval |= VFF_BUFFTOOSMALL;
        }
        lstrcpynW(lpszDestDir, destDir, *lpuDestDirLen as i32);
        *lpuDestDirLen = destDirSizeReq;
    }
    if !lpuCurDirLen.is_null() && !lpszCurDir.is_null() {
        if *lpuCurDirLen < curDirSizeReq {
            retval |= VFF_BUFFTOOSMALL;
        }
        lstrcpynW(lpszCurDir, curDir, *lpuCurDirLen as i32);
        *lpuCurDirLen = curDirSizeReq;
    }

    TRACE!(
        b"ret = %u (%s%s%s) curdir=%s destdir=%s\n\0".as_ptr() as LPCSTR,
        retval,
        if (retval & VFF_CURNEDEST) != 0 {
            b"VFF_CURNEDEST \0".as_ptr() as LPCSTR
        } else {
            b"\0".as_ptr() as LPCSTR
        },
        if (retval & VFF_FILEINUSE) != 0 {
            b"VFF_FILEINUSE \0".as_ptr() as LPCSTR
        } else {
            b"\0".as_ptr() as LPCSTR
        },
        if (retval & VFF_BUFFTOOSMALL) != 0 {
            b"VFF_BUFFTOOSMALL \0".as_ptr() as LPCSTR
        } else {
            b"\0".as_ptr() as LPCSTR
        },
        debugstr_w(lpszCurDir),
        debugstr_w(lpszDestDir)
    );
    return retval;
}
unsafe fn _fetch_versioninfo(
    fn_: LPSTR,
    vffi: *mut *mut VS_FIXEDFILEINFO,
) -> LPBYTE {
    let mut alloclen: DWORD;
    let mut buf: LPBYTE;
    let mut ret: DWORD;

    alloclen = 1000;
    buf = HeapAlloc(GetProcessHeap(), 0, alloclen as usize) as LPBYTE;
    if buf.is_null() {
        WARN!(b"Memory exhausted while fetching version info!\n\0".as_ptr() as LPCSTR);
        return core::ptr::null_mut();
    }
    loop {
        ret = GetFileVersionInfoA(fn_, 0, alloclen, buf as LPVOID) as DWORD;
        if ret == 0 {
            HeapFree(GetProcessHeap(), 0, buf as LPVOID);
            return core::ptr::null_mut();
        }
        if alloclen < *(buf as *const WORD) as DWORD {
            alloclen = *(buf as *const WORD) as DWORD;
            HeapFree(GetProcessHeap(), 0, buf as LPVOID);
            buf = HeapAlloc(GetProcessHeap(), 0, alloclen as usize) as LPBYTE;
            if buf.is_null() {
                WARN!(b"Memory exhausted while fetching version info!\n\0".as_ptr() as LPCSTR);
                return core::ptr::null_mut();
            }
        } else {
            *vffi = buf.offset(0x14) as *mut VS_FIXEDFILEINFO;
            if (*(*vffi)).dwSignature == 0x004f0049 {
                *vffi = buf.offset(0x28) as *mut VS_FIXEDFILEINFO;
            }
            if (*(*vffi)).dwSignature != VS_FFI_SIGNATURE {
                WARN!(
                    b"Bad VS_FIXEDFILEINFO signature 0x%08x\n\0".as_ptr() as LPCSTR,
                    (*(*vffi)).dwSignature,
                );
            }
            return buf;
        }
    }
}
unsafe fn _error2vif(error: DWORD) -> DWORD {
    match error {
        ERROR_ACCESS_DENIED => VIF_ACCESSVIOLATION,
        ERROR_SHARING_VIOLATION => VIF_SHARINGVIOLATION,
        _ => 0,
    }
}

#[no_mangle]
pub unsafe extern "system" fn VerInstallFileA(
    flags: DWORD,
    srcfilename: LPCSTR,
    destfilename: LPCSTR,
    srcdir: LPCSTR,
    destdir: LPCSTR,
    curdir: LPCSTR,
    tmpfile: LPSTR,
    tmpfilelen: PUINT,
) -> DWORD {

    let pdest: LPCSTR;
    let mut destfn: [i8; 260] = [0i8; 260];
    let mut tmpfn: [i8; 260] = [0i8; 260];
    let mut srcfn: [i8; 260] = [0i8; 260];
    let hfsrc: HFILE;
    let hfdst: HFILE;
    let mut attr: DWORD;
    let mut xret: DWORD;
    let mut tmplast: DWORD;
    let ret: LONG;
    let buf1: LPBYTE;
    let buf2: LPBYTE;
    let mut ofs: OFSTRUCT = core::mem::zeroed();

    TRACE!(
        b"(%x,%s,%s,%s,%s,%s,%p,%d)\n\0".as_ptr() as *const i8,
        flags,
        debugstr_a(srcfilename),
        debugstr_a(destfilename),
        debugstr_a(srcdir),
        debugstr_a(destdir),
        debugstr_a(curdir),
        tmpfile,
        *tmpfilelen,
    );
    xret = 0;
    if srcdir.is_null() || srcfilename.is_null() {
        return VIF_CANNOTREADSRC;
    }
    sprintf(
        srcfn.as_mut_ptr(),
        b"%s\\%s\0".as_ptr() as *const i8,
        srcdir,
        srcfilename,
    );
    if destdir.is_null() || *destdir == 0 {
        pdest = srcdir;
    } else {
        pdest = destdir;
    }
    sprintf(
        destfn.as_mut_ptr(),
        b"%s\\%s\0".as_ptr() as *const i8,
        pdest,
        destfilename,
    );
    hfsrc = LZOpenFileA(srcfn.as_mut_ptr(), &mut ofs, OF_READ as WORD);
    if (hfsrc as i32) < 0 {
        return VIF_CANNOTREADSRC;
    }
    sprintf(
        tmpfn.as_mut_ptr(),
        b"%s\\%s\0".as_ptr() as *const i8,
        pdest,
        destfilename,
    );
    tmplast = strlen(pdest) as DWORD + 1;
    attr = GetFileAttributesA(tmpfn.as_mut_ptr());
    if attr != INVALID_FILE_ATTRIBUTES {
        if attr & FILE_ATTRIBUTE_READONLY != 0 {
            LZClose(hfsrc);
            return VIF_WRITEPROT;
        }
    }
    attr = INVALID_FILE_ATTRIBUTES;
    if flags & VIFF_FORCEINSTALL != 0 {
        if *tmpfile != 0 {
            sprintf(
                tmpfn.as_mut_ptr(),
                b"%s\\%s\0".as_ptr() as *const i8,
                pdest,
                tmpfile,
            );
            tmplast = strlen(pdest) as DWORD + 1;
            attr = GetFileAttributesA(tmpfn.as_mut_ptr());
        }
    }
    if attr == INVALID_FILE_ATTRIBUTES {
        let s: LPSTR;

        GetTempFileNameA(pdest, b"ver\0".as_ptr() as *const i8, 0, tmpfn.as_mut_ptr());
        s = strrchr(tmpfn.as_mut_ptr(), b'\\' as i32);
        if !s.is_null() {
            tmplast = (s as usize - tmpfn.as_mut_ptr() as usize) as DWORD;
        } else {
            tmplast = 0;
        }
        hfdst = OpenFile(tmpfn.as_mut_ptr(), &mut ofs, OF_CREATE);
        if hfdst == HFILE_ERROR {
            LZClose(hfsrc);
            return VIF_CANNOTCREATE;
        }
        ret = LZCopy(hfsrc, hfdst);
        _lclose(hfdst);
        if ret < 0 {
            match ret {
                LZERROR_BADINHANDLE | LZERROR_READ | LZERROR_BADVALUE | LZERROR_UNKNOWNALG => {
                    xret = VIF_CANNOTREADSRC;
                }
                LZERROR_BADOUTHANDLE | LZERROR_WRITE => {
                    xret = VIF_OUTOFSPACE;
                }
                LZERROR_GLOBALLOC | LZERROR_GLOBLOCK => {
                    xret = VIF_OUTOFMEMORY;
                }
                _ => {
                    FIXME!(
                        b"Unknown LZCopy error %d, ignoring.\n\0".as_ptr() as *const i8,
                        ret,
                    );
                    xret = 0;
                }
            }
            if xret != 0 {
                LZClose(hfsrc);
                return xret;
            }
        }
    }
    if flags & VIFF_FORCEINSTALL == 0 {
        let mut destvffi: *mut VS_FIXEDFILEINFO = core::ptr::null_mut();
        let mut tmpvffi: *mut VS_FIXEDFILEINFO = core::ptr::null_mut();
        buf1 = _fetch_versioninfo(destfn.as_mut_ptr(), &mut destvffi);
        if !buf1.is_null() {
            buf2 = _fetch_versioninfo(tmpfn.as_mut_ptr(), &mut tmpvffi);
            if !buf2.is_null() {
                let mut tbuf1: *mut i8 = core::ptr::null_mut();
                let mut tbuf2: *mut i8 = core::ptr::null_mut();
                static TRANS_ARRAY: [i8; 25] = unsafe { core::mem::transmute::<[u8; 25], [i8; 25]>(*b"\\VarFileInfo\\Translation\0") };
                let mut len1: UINT;
                let mut len2: UINT;

                len1 = 40;
                len2 = 40;

                if ((*destvffi).dwFileVersionMS > (*tmpvffi).dwFileVersionMS)
                    || ((*destvffi).dwFileVersionMS == (*tmpvffi).dwFileVersionMS
                        && (*destvffi).dwFileVersionLS > (*tmpvffi).dwFileVersionLS)
                {
                    xret |= VIF_MISMATCH | VIF_SRCOLD;
                }
                if ((*destvffi).dwFileType != (*tmpvffi).dwFileType)
                    || ((*destvffi).dwFileSubtype != (*tmpvffi).dwFileSubtype)
                {
                    xret |= VIF_MISMATCH | VIF_DIFFTYPE;
                }
                if VerQueryValueA(
                    buf1 as LPCVOID,
                    TRANS_ARRAY.as_ptr(),
                    &mut tbuf1 as *mut *mut i8 as *mut LPVOID,
                    &mut len1,
                ) != 0
                    && VerQueryValueA(
                        buf2 as LPCVOID,
                        TRANS_ARRAY.as_ptr(),
                        &mut tbuf2 as *mut *mut i8 as *mut LPVOID,
                        &mut len2,
                    ) != 0
                {
                }
                HeapFree(GetProcessHeap(), 0, buf2 as LPVOID);
            } else {
                xret = VIF_MISMATCH | VIF_SRCOLD;
            }
            HeapFree(GetProcessHeap(), 0, buf1 as LPVOID);
        }
    }
    if xret != 0 {
        if *tmpfilelen < strlen(tmpfn.as_mut_ptr().offset(tmplast as isize)) as UINT {
            xret |= VIF_BUFFTOOSMALL;
            DeleteFileA(tmpfn.as_mut_ptr());
        } else {
            strcpy(tmpfile, tmpfn.as_mut_ptr().offset(tmplast as isize));
            *tmpfilelen = strlen(tmpfn.as_mut_ptr().offset(tmplast as isize)) as UINT + 1;
            xret |= VIF_TEMPFILE;
        }
    } else {
        if INVALID_FILE_ATTRIBUTES != GetFileAttributesA(destfn.as_mut_ptr()) {
            if DeleteFileA(destfn.as_mut_ptr()) == 0 {
                xret |= _error2vif(GetLastError()) | VIF_CANNOTDELETE;
                DeleteFileA(tmpfn.as_mut_ptr());
                LZClose(hfsrc);
                return xret;
            }
        }
        if (flags & VIFF_DONTDELETEOLD) == 0
            && !curdir.is_null()
            && *curdir != 0
            && lstrcmpiA(curdir, pdest) != 0
        {
            let mut curfn: [i8; 260] = [0i8; 260];

            sprintf(
                curfn.as_mut_ptr(),
                b"%s\\%s\0".as_ptr() as *const i8,
                curdir,
                destfilename,
            );
            if INVALID_FILE_ATTRIBUTES != GetFileAttributesA(curfn.as_mut_ptr()) {
                if DeleteFileA(curfn.as_mut_ptr()) == 0 {
                    xret |= _error2vif(GetLastError()) | VIF_CANNOTDELETECUR;
                }
            }
        }
        if MoveFileA(tmpfn.as_mut_ptr(), destfn.as_mut_ptr()) == 0 {
            xret |= _error2vif(GetLastError()) | VIF_CANNOTRENAME;
            DeleteFileA(tmpfn.as_mut_ptr());
        }
    }
    LZClose(hfsrc);
    return xret;
}

#[no_mangle]
pub unsafe extern "system" fn VerInstallFileW(
    flags: DWORD,
    srcfilename: LPCWSTR,
    destfilename: LPCWSTR,
    srcdir: LPCWSTR,
    destdir: LPCWSTR,
    curdir: LPCWSTR,
    tmpfile: LPWSTR,
    tmpfilelen: PUINT,
) -> DWORD {
    let mut wsrcf: LPSTR = core::ptr::null_mut();
    let mut wsrcd: LPSTR = core::ptr::null_mut();
    let mut wdestf: LPSTR = core::ptr::null_mut();
    let mut wdestd: LPSTR = core::ptr::null_mut();
    let mut wtmpf: LPSTR = core::ptr::null_mut();
    let mut wcurd: LPSTR = core::ptr::null_mut();
    let mut ret: DWORD = 0;
    let mut len: UINT = 0;

    if !srcfilename.is_null() {
        len = WideCharToMultiByte(CP_ACP, 0, srcfilename, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as UINT;
        let alloc_ptr = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T);
        wsrcf = alloc_ptr as LPSTR;
        if !wsrcf.is_null() {
            WideCharToMultiByte(CP_ACP, 0, srcfilename, -1, wsrcf, len as i32, core::ptr::null_mut(), core::ptr::null_mut());
        } else {
            ret = VIF_OUTOFMEMORY;
        }
    }
    if !srcdir.is_null() && ret == 0 {
        len = WideCharToMultiByte(CP_ACP, 0, srcdir, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as UINT;
        let alloc_ptr = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T);
        wsrcd = alloc_ptr as LPSTR;
        if !wsrcd.is_null() {
            WideCharToMultiByte(CP_ACP, 0, srcdir, -1, wsrcd, len as i32, core::ptr::null_mut(), core::ptr::null_mut());
        } else {
            ret = VIF_OUTOFMEMORY;
        }
    }
    if !destfilename.is_null() && ret == 0 {
        len = WideCharToMultiByte(CP_ACP, 0, destfilename, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as UINT;
        let alloc_ptr = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T);
        wdestf = alloc_ptr as LPSTR;
        if !wdestf.is_null() {
            WideCharToMultiByte(CP_ACP, 0, destfilename, -1, wdestf, len as i32, core::ptr::null_mut(), core::ptr::null_mut());
        } else {
            ret = VIF_OUTOFMEMORY;
        }
    }
    if !destdir.is_null() && ret == 0 {
        len = WideCharToMultiByte(CP_ACP, 0, destdir, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as UINT;
        let alloc_ptr = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T);
        wdestd = alloc_ptr as LPSTR;
        if !wdestd.is_null() {
            WideCharToMultiByte(CP_ACP, 0, destdir, -1, wdestd, len as i32, core::ptr::null_mut(), core::ptr::null_mut());
        } else {
            ret = VIF_OUTOFMEMORY;
        }
    }
    if !curdir.is_null() && ret == 0 {
        len = WideCharToMultiByte(CP_ACP, 0, curdir, -1, core::ptr::null_mut(), 0, core::ptr::null_mut(), core::ptr::null_mut()) as UINT;
        let alloc_ptr = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T);
        wcurd = alloc_ptr as LPSTR;
        if !wcurd.is_null() {
            WideCharToMultiByte(CP_ACP, 0, curdir, -1, wcurd, len as i32, core::ptr::null_mut(), core::ptr::null_mut());
        } else {
            ret = VIF_OUTOFMEMORY;
        }
    }
    if ret == 0 {
        len = (*tmpfilelen) * core::mem::size_of::<WCHAR>() as UINT;
        let alloc_ptr = HeapAlloc(GetProcessHeap(), 0, len as SIZE_T);
        wtmpf = alloc_ptr as LPSTR;
        if wtmpf.is_null() {
            ret = VIF_OUTOFMEMORY;
        }
    }
    if ret == 0 {
        ret = VerInstallFileA(flags, wsrcf, wdestf, wsrcd, wdestd, wcurd, wtmpf, &mut len as *mut UINT);
    }
    if ret == 0 {
        *tmpfilelen = MultiByteToWideChar(CP_ACP, 0, wtmpf, -1, tmpfile, *tmpfilelen as i32) as UINT;
    } else if (ret & VIF_BUFFTOOSMALL) != 0 {
        *tmpfilelen = len;
    }

    HeapFree(GetProcessHeap(), 0, wsrcf as LPVOID);
    HeapFree(GetProcessHeap(), 0, wsrcd as LPVOID);
    HeapFree(GetProcessHeap(), 0, wdestf as LPVOID);
    HeapFree(GetProcessHeap(), 0, wdestd as LPVOID);
    HeapFree(GetProcessHeap(), 0, wtmpf as LPVOID);
    HeapFree(GetProcessHeap(), 0, wcurd as LPVOID);
    ret
}
