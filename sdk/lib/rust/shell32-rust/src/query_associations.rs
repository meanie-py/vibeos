// CQueryAssociations — IQueryAssociations over the HKCR association registry.
// Hand-stitched module: machine scaffold (interfaces/vtable/thunks/refcount/activation)
// kept from the cached translation; GetString hand-ported from
// dll/win32/shell32/CQueryAssociations.cpp:233-471 (the cached body was damaged);
// helper bodies repaired in place. See CQueryAssociations-stitch-report.md.

use core::ffi::c_void;
// One declaration per external symbol: island.rs.
use crate::island::CLSIDFromString;
use crate::island::HeapAlloc;
use crate::island::LoadStringW;
use crate::island::PathRemoveExtensionW;
use crate::island::RegCloseKey;
use crate::island::RegEnumKeyExW;
use crate::island::RegGetValueW;
use crate::island::RegOpenKeyExW;
use crate::island::RegQueryInfoKeyW;
use crate::island::SHQueryValueExW;
use crate::island::VerQueryValueW;
use crate::shellcom::*;
use crate::types::*;

// Deterministic from shlwapi.h
#[repr(C)]
pub struct IQueryAssociations {
    pub lpVtbl: *const IQueryAssociationsVtbl,
}
#[repr(C)]
pub struct IQueryAssociationsVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IQueryAssociations, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IQueryAssociations) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IQueryAssociations) -> ULONG,
    pub Init: unsafe extern "system" fn(This: *mut IQueryAssociations, flags: ASSOCF, pszAssoc: LPCWSTR, hkProgid: HKEY, hwnd: HWND) -> HRESULT,
    pub GetString: unsafe extern "system" fn(This: *mut IQueryAssociations, flags: ASSOCF, str: ASSOCSTR, pszExtra: LPCWSTR, pszOut: LPWSTR, pcchOut: *mut DWORD) -> HRESULT,
    pub GetKey: unsafe extern "system" fn(This: *mut IQueryAssociations, flags: ASSOCF, key: ASSOCKEY, pszExtra: LPCWSTR, phkeyOut: *mut HKEY) -> HRESULT,
    pub GetData: unsafe extern "system" fn(This: *mut IQueryAssociations, flags: ASSOCF, data: ASSOCDATA, pszExtra: LPCWSTR, pvOut: LPVOID, pcbOut: *mut DWORD) -> HRESULT,
    pub GetEnum: unsafe extern "system" fn(This: *mut IQueryAssociations, flags: ASSOCF, assocenum: ASSOCENUM, pszExtra: LPCWSTR, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT,
}
extern "C" {
    pub static IID_IQueryAssociations: IID;
}

// Deterministic COM scaffold for CQueryAssociations; method bodies are generated separately.
extern "system" {
    fn GetProcessHeap() -> HANDLE;
    fn HeapFree(heap: HANDLE, flags: DWORD, memory: *mut core::ffi::c_void) -> BOOL;
    fn Shell32RustObjectLock();
    fn Shell32RustObjectUnlock();
}
extern "C" {
    pub static IID_IUnknown: IID;
}

#[repr(C)]
pub struct CQueryAssociations {
    pub IQueryAssociations_iface: IQueryAssociations,
    pub ref_count: core::sync::atomic::AtomicI32,
    pub hkeySource: HKEY,
    pub hkeyProgID: HKEY,
}

unsafe extern "system" fn CQueryAssociations_IQueryAssociations_QueryInterface(This: *mut IQueryAssociations, riid: REFIID, ppvObject: *mut *mut core::ffi::c_void) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CQueryAssociations, IQueryAssociations_iface)) as *mut CQueryAssociations;
    CQueryAssociations_QueryInterface(this, riid, ppvObject)
}
unsafe extern "system" fn CQueryAssociations_IQueryAssociations_AddRef(This: *mut IQueryAssociations) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CQueryAssociations, IQueryAssociations_iface)) as *mut CQueryAssociations;
    CQueryAssociations_AddRef(this)
}
unsafe extern "system" fn CQueryAssociations_IQueryAssociations_Release(This: *mut IQueryAssociations) -> ULONG {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CQueryAssociations, IQueryAssociations_iface)) as *mut CQueryAssociations;
    CQueryAssociations_Release(this)
}
unsafe extern "system" fn CQueryAssociations_IQueryAssociations_Init(This: *mut IQueryAssociations, flags: ASSOCF, pszAssoc: LPCWSTR, hkProgid: HKEY, hwnd: HWND) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CQueryAssociations, IQueryAssociations_iface)) as *mut CQueryAssociations;
    CQueryAssociations_Init_body(this, flags, pszAssoc, hkProgid, hwnd)
}
unsafe extern "system" fn CQueryAssociations_IQueryAssociations_GetString(This: *mut IQueryAssociations, flags: ASSOCF, str: ASSOCSTR, pszExtra: LPCWSTR, pszOut: LPWSTR, pcchOut: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CQueryAssociations, IQueryAssociations_iface)) as *mut CQueryAssociations;
    CQueryAssociations_GetString_body(this, flags, str, pszExtra, pszOut, pcchOut)
}
unsafe extern "system" fn CQueryAssociations_IQueryAssociations_GetKey(This: *mut IQueryAssociations, flags: ASSOCF, key: ASSOCKEY, pszExtra: LPCWSTR, phkeyOut: *mut HKEY) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CQueryAssociations, IQueryAssociations_iface)) as *mut CQueryAssociations;
    CQueryAssociations_GetKey_body(this, flags, key, pszExtra, phkeyOut)
}
unsafe extern "system" fn CQueryAssociations_IQueryAssociations_GetData(This: *mut IQueryAssociations, flags: ASSOCF, data: ASSOCDATA, pszExtra: LPCWSTR, pvOut: LPVOID, pcbOut: *mut DWORD) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CQueryAssociations, IQueryAssociations_iface)) as *mut CQueryAssociations;
    CQueryAssociations_GetData_body(this, flags, data, pszExtra, pvOut, pcbOut)
}
unsafe extern "system" fn CQueryAssociations_IQueryAssociations_GetEnum(This: *mut IQueryAssociations, flags: ASSOCF, assocenum: ASSOCENUM, pszExtra: LPCWSTR, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    let this = (This as *mut u8).sub(core::mem::offset_of!(CQueryAssociations, IQueryAssociations_iface)) as *mut CQueryAssociations;
    CQueryAssociations_GetEnum_body(this, flags, assocenum, pszExtra, riid, ppvOut)
}

pub static CQUERYASSOCIATIONS_IQUERYASSOCIATIONS_VTBL: IQueryAssociationsVtbl = IQueryAssociationsVtbl {
    QueryInterface: CQueryAssociations_IQueryAssociations_QueryInterface,
    AddRef: CQueryAssociations_IQueryAssociations_AddRef,
    Release: CQueryAssociations_IQueryAssociations_Release,
    Init: CQueryAssociations_IQueryAssociations_Init,
    GetString: CQueryAssociations_IQueryAssociations_GetString,
    GetKey: CQueryAssociations_IQueryAssociations_GetKey,
    GetData: CQueryAssociations_IQueryAssociations_GetData,
    GetEnum: CQueryAssociations_IQueryAssociations_GetEnum,
};

unsafe fn CQueryAssociations_QueryInterface(
    this: *mut CQueryAssociations, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IUnknown)) || cpp_com_iid_equal(riid, core::ptr::addr_of!(IID_IQueryAssociations)) {
        *ppv = core::ptr::addr_of_mut!((*this).IQueryAssociations_iface) as *mut core::ffi::c_void;
    }
    else { return E_NOINTERFACE; }
    CQueryAssociations_AddRef(this);
    S_OK
}

unsafe fn CQueryAssociations_AddRef(this: *mut CQueryAssociations) -> ULONG {
    (*this).ref_count.fetch_add(1, core::sync::atomic::Ordering::SeqCst) as ULONG + 1
}

unsafe fn CQueryAssociations_Release(this: *mut CQueryAssociations) -> ULONG {
    let refs = (*this).ref_count.fetch_sub(1, core::sync::atomic::Ordering::SeqCst) - 1;
    if refs == 0 {
        CQueryAssociations_dtor_body(this);
        Shell32RustObjectUnlock();
        HeapFree(GetProcessHeap(), 0, this as *mut core::ffi::c_void);
    }
    refs as ULONG
}

#[inline]
unsafe fn cpp_com_iid_equal(left: REFIID, right: REFIID) -> bool {
    !left.is_null() && !right.is_null()
        && (*left).Data1 == (*right).Data1
        && (*left).Data2 == (*right).Data2
        && (*left).Data3 == (*right).Data3
        && (*left).Data4 == (*right).Data4
}

// Activation: CLSID_QueryAssociations currently maps to CQueryAssociations in shell32.cpp.
// The generated C++ adapter redirects ATL activation to this factory.
#[no_mangle] pub unsafe extern "system" fn CQueryAssociations_CreateInstance(
    outer: *mut IUnknown, riid: REFIID, ppv: *mut *mut core::ffi::c_void,
) -> HRESULT {
    if ppv.is_null() { return E_POINTER; }
    *ppv = core::ptr::null_mut();
    if !outer.is_null() { return CLASS_E_NOAGGREGATION; }
    let this = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (core::mem::size_of::<CQueryAssociations>()) as _) as *mut CQueryAssociations;
    if this.is_null() { return E_OUTOFMEMORY; }
    core::ptr::write(this, CQueryAssociations {
        IQueryAssociations_iface: IQueryAssociations { lpVtbl: core::ptr::addr_of!(CQUERYASSOCIATIONS_IQUERYASSOCIATIONS_VTBL) },
        ref_count: core::sync::atomic::AtomicI32::new(1),
        hkeySource: core::mem::zeroed(),
        hkeyProgID: core::mem::zeroed(),
    });
    Shell32RustObjectLock();
    CQueryAssociations_ctor_body(this);
    let hr = CQueryAssociations_QueryInterface(this, riid, ppv);
    CQueryAssociations_Release(this);
    hr
}

// ── Deterministic C++ support closure selected from source identifiers ──────────────────

// shlwapi.h enum types. ASSOCF is `typedef DWORD ASSOCF`; the ASSOCSTR/ASSOCKEY/
// ASSOCDATA/ASSOCENUM enums are 4-byte ints — DWORD aliases keep the vtable ABI exact.
pub type ASSOCF = DWORD;
pub type ASSOCSTR = DWORD;
pub type ASSOCKEY = DWORD;
pub type ASSOCDATA = DWORD;
pub type ASSOCENUM = DWORD;

pub const ASSOCF_NOTRUNCATE: ASSOCF = 0x0000_0020; // sdk/include/psdk/shlwapi.h:865
// ASSOCSTR enum, sdk/include/psdk/shlwapi.h:886-902 (_WIN32_IE >= IE60 members included)
pub const ASSOCSTR_COMMAND: ASSOCSTR = 1;
pub const ASSOCSTR_EXECUTABLE: ASSOCSTR = 2;
pub const ASSOCSTR_FRIENDLYDOCNAME: ASSOCSTR = 3;
pub const ASSOCSTR_FRIENDLYAPPNAME: ASSOCSTR = 4;
pub const ASSOCSTR_CONTENTTYPE: ASSOCSTR = 14;
pub const ASSOCSTR_DEFAULTICON: ASSOCSTR = 15;
pub const ASSOCSTR_SHELLEXTENSION: ASSOCSTR = 16;
// ASSOCDATA enum, sdk/include/psdk/shlwapi.h:925-935 (MSIDESCRIPTOR=1 … EDITFLAGS=5)
pub const ASSOCDATA_EDITFLAGS: ASSOCDATA = 5;

pub const KEY_READ: REGSAM = 0x20019; // sdk/include/xdk/winnt_old.h:545
pub const E_UNEXPECTED: HRESULT = 0x8000_FFFFu32 as HRESULT;
// winerror.h values (typed LONG for HRESULT_FROM_WIN32)
pub const ERROR_FILE_NOT_FOUND: LONG = 2; // sdk/include/psdk/winerror.h:227
pub const ERROR_INSUFFICIENT_BUFFER: LONG = 122; // sdk/include/psdk/winerror.h:316
pub const ERROR_NO_ASSOCIATION: LONG = 1155; // sdk/include/psdk/winerror.h:1001

pub const IDS_ANY_FILE: UINT = 299; // dll/win32/shell32/shresdef.h:184 (free-helper closure)
pub const IDS_DIRECTORY: UINT = 200; // dll/win32/shell32/shresdef.h:158 (free-helper closure)

extern "system" {
    // shlwapi
    fn StrChrW(text: LPCWSTR, ch: WCHAR) -> LPWSTR;
    fn PathFindExtensionW(pszPath: LPCWSTR) -> LPWSTR;
    fn PathStripPathW(pszPath: LPWSTR);
    // advapi32
    // kernel32
    fn ExpandEnvironmentStringsW(lpSrc: LPCWSTR, lpDst: LPWSTR, nSize: DWORD) -> DWORD;
    fn SearchPathW(
        lpPath: LPCWSTR,
        lpFileName: LPCWSTR,
        lpExtension: LPCWSTR,
        nBufferLength: DWORD,
        lpBuffer: LPWSTR,
        lpFilePart: *mut LPWSTR,
    ) -> DWORD;
    // version
    fn GetFileVersionInfoSizeW(lptstrFilename: LPCWSTR, lpdwHandle: LPDWORD) -> DWORD;
    fn GetFileVersionInfoW(lptstrFilename: LPCWSTR, dwHandle: DWORD, dwLen: DWORD, lpData: LPVOID) -> BOOL;
    fn LCMapStringW(locale: DWORD, flags: DWORD, source: LPCWSTR, source_len: INT, destination: LPWSTR, destination_len: INT) -> INT;
    fn StrTrimW(text: LPWSTR, trim_chars: LPCWSTR) -> BOOL;
    // ole32
}

extern "C" {
    fn memcpy(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void, bytes: usize) -> *mut core::ffi::c_void;
    // msvcrt — swprintf is the legacy variadic form ReactOS shell32 uses (matches
    // shellole.rs; wine/unicode.h:64 maps sprintfW → _swprintf, the same export).
    fn swprintf(buffer: *mut WCHAR, format: *const WCHAR, ...) -> INT;
    fn wcscat(dst: LPWSTR, src: LPCWSTR) -> LPWSTR;
    static mut shell32_hInstance: HINSTANCE;
}

// windef.h LOWORD/HIWORD
#[inline]
fn LOWORD(value: DWORD) -> WORD {
    (value & 0xFFFF) as WORD
}
#[inline]
fn HIWORD(value: DWORD) -> WORD {
    ((value >> 16) & 0xFFFF) as WORD
}

// Deterministic UTF-16 literals from the selected C++ helper closure.
pub static CPP_COM_WSTR_0C71B3CB5580: [WCHAR; 8] = [0x0063, 0x006F, 0x006D, 0x006D, 0x0061, 0x006E, 0x0064, 0x0000]; // L"command"
pub static CPP_COM_WSTR_3BE8B25B7ED1: [WCHAR; 12] = [0x0044, 0x0065, 0x0066, 0x0061, 0x0075, 0x006C, 0x0074, 0x0049, 0x0063, 0x006F, 0x006E, 0x0000]; // L"DefaultIcon"
pub static CPP_COM_WSTR_43CC49772B80: [WCHAR; 41] = [0x005C, 0x0053, 0x0074, 0x0072, 0x0069, 0x006E, 0x0067, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0025, 0x0030, 0x0034, 0x0078, 0x0025, 0x0030, 0x0034, 0x0078, 0x005C, 0x0046, 0x0069, 0x006C, 0x0065, 0x0044, 0x0065, 0x0073, 0x0063, 0x0072, 0x0069, 0x0070, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000]; // L"\\StringFileInfo\\%04x%04x\\FileDescription"
pub static CPP_COM_WSTR_54E2098886DA: [WCHAR; 13] = [0x0043, 0x006F, 0x006E, 0x0074, 0x0065, 0x006E, 0x0074, 0x0020, 0x0054, 0x0079, 0x0070, 0x0065, 0x0000]; // L"Content Type"
pub static CPP_COM_WSTR_6AC3419A3E2E: [WCHAR; 25] = [0x005C, 0x0056, 0x0061, 0x0072, 0x0046, 0x0069, 0x006C, 0x0065, 0x0049, 0x006E, 0x0066, 0x006F, 0x005C, 0x0054, 0x0072, 0x0061, 0x006E, 0x0073, 0x006C, 0x0061, 0x0074, 0x0069, 0x006F, 0x006E, 0x0000]; // L"\\VarFileInfo\\Translation"
pub static CPP_COM_WSTR_82DB442E7A60: [WCHAR; 5] = [0x006F, 0x0070, 0x0065, 0x006E, 0x0000]; // L"open"
pub static CPP_COM_WSTR_89E3C2C9E5FD: [WCHAR; 7] = [0x0050, 0x0072, 0x006F, 0x0067, 0x0049, 0x0044, 0x0000]; // L"ProgID"
pub static CPP_COM_WSTR_A2E2913C930F: [WCHAR; 10] = [0x0045, 0x0064, 0x0069, 0x0074, 0x0046, 0x006C, 0x0061, 0x0067, 0x0073, 0x0000]; // L"EditFlags"
pub static CPP_COM_WSTR_E01CFD69C238: [WCHAR; 9] = [0x0053, 0x0068, 0x0065, 0x006C, 0x006C, 0x0045, 0x0078, 0x005C, 0x0000]; // L"ShellEx\\"
pub static CPP_COM_WSTR_EE89605B31B0: [WCHAR; 6] = [0x0073, 0x0068, 0x0065, 0x006C, 0x006C, 0x0000]; // L"shell"

static mut DIRECTORY_DESCRIPTION_CACHE: [WCHAR; 33] = [0; 33];
static mut ANY_FILE_FORMAT_CACHE: [WCHAR; 33] = [0; 33];
static W_FRIENDLY_TYPE_NAME: [WCHAR; 17] = [
    0x0046, 0x0072, 0x0069, 0x0065, 0x006E, 0x0064, 0x006C, 0x0079, 0x0054, 0x0079,
    0x0070, 0x0065, 0x004E, 0x0061, 0x006D, 0x0065, 0,
];
static W_EMPTY: [WCHAR; 1] = [0];
static W_TRIM_EMPTY_EXTENSION: [WCHAR; 3] = [0x0020, 0x002D, 0]; // L" -"

const LOCALE_USER_DEFAULT: DWORD = 0x0400;
const LCMAP_UPPERCASE: DWORD = 0x0200;
const STRSAFE_E_INSUFFICIENT_BUFFER: HRESULT = 0x8007007Au32 as HRESULT;

unsafe fn string_cch_copy_w(destination: LPWSTR, capacity: usize, source: LPCWSTR) -> HRESULT {
    if destination.is_null() || source.is_null() || capacity == 0 {
        return E_INVALIDARG;
    }
    let mut index = 0usize;
    while *source.add(index) != 0 && index + 1 < capacity {
        *destination.add(index) = *source.add(index);
        index += 1;
    }
    *destination.add(index) = 0;
    if *source.add(index) == 0 { S_OK } else { STRSAFE_E_INSUFFICIENT_BUFFER }
}

unsafe fn string_cch_format_one_w(
    destination: LPWSTR,
    capacity: usize,
    format: LPCWSTR,
    argument: LPCWSTR,
) -> HRESULT {
    if destination.is_null() || format.is_null() || capacity == 0 {
        return E_INVALIDARG;
    }
    let mut out = 0usize;
    let mut input = 0usize;
    let mut status = S_OK;
    while *format.add(input) != 0 {
        let mut replacement = core::ptr::null();
        if *format.add(input) == b'%' as WCHAR {
            if *format.add(input + 1) == b's' as WCHAR {
                replacement = argument;
                input += 2;
            } else if *format.add(input + 1) == b'l' as WCHAR
                && *format.add(input + 2) == b's' as WCHAR
            {
                replacement = argument;
                input += 3;
            }
        }
        if !replacement.is_null() {
            let mut at = 0usize;
            while *replacement.add(at) != 0 {
                if out + 1 >= capacity {
                    status = STRSAFE_E_INSUFFICIENT_BUFFER;
                    break;
                }
                *destination.add(out) = *replacement.add(at);
                out += 1;
                at += 1;
            }
            if status < 0 { break; }
            continue;
        }
        if out + 1 >= capacity {
            status = STRSAFE_E_INSUFFICIENT_BUFFER;
            break;
        }
        *destination.add(out) = *format.add(input);
        out += 1;
        input += 1;
    }
    *destination.add(out) = 0;
    status
}

#[no_mangle]
pub unsafe extern "C" fn SHELL32_AssocGetFSDirectoryDescription(
    buffer: PWSTR,
    capacity: UINT,
) -> HRESULT {
    if DIRECTORY_DESCRIPTION_CACHE[0] == 0 {
        LoadStringW(
            shell32_hInstance,
            IDS_DIRECTORY,
            DIRECTORY_DESCRIPTION_CACHE.as_mut_ptr(),
            DIRECTORY_DESCRIPTION_CACHE.len() as INT,
        );
    }
    string_cch_copy_w(buffer, capacity as usize, DIRECTORY_DESCRIPTION_CACHE.as_ptr())
}

unsafe fn get_extension_default_description(
    extension: PCWSTR,
    buffer: PWSTR,
    capacity: UINT,
) -> HRESULT {
    if ANY_FILE_FORMAT_CACHE[0] == 0 {
        LoadStringW(
            shell32_hInstance,
            IDS_ANY_FILE,
            ANY_FILE_FORMAT_CACHE.as_mut_ptr(),
            ANY_FILE_FORMAT_CACHE.len() as INT,
        );
    }
    string_cch_format_one_w(
        buffer,
        capacity as usize,
        ANY_FILE_FORMAT_CACHE.as_ptr(),
        extension,
    )
}

unsafe fn assoc_get_extension_description(
    mut extension: PCWSTR,
    buffer: PWSTR,
    capacity: UINT,
) -> HRESULT {
    if *extension == 0 || (*extension.add(1) == 0 && *extension == b'.' as WCHAR) {
        let result = get_extension_default_description(W_EMPTY.as_ptr(), buffer, capacity);
        if result >= 0 {
            StrTrimW(buffer, W_TRIM_EMPTY_EXTENSION.as_ptr());
        }
        return result;
    }

    let mut key: HKEY = core::ptr::null_mut();
    let mut result = crate::wine::classes::HCR_GetProgIdKeyOfExtension(extension, &mut key, TRUE);
    if result >= 0 {
        let mut error = crate::vista::RegLoadMUIStringW(
            key,
            W_FRIENDLY_TYPE_NAME.as_ptr(),
            buffer,
            capacity,
            core::ptr::null_mut(),
            0,
            core::ptr::null(),
        );
        if error != 0 && result == S_OK {
            let mut bytes = capacity * core::mem::size_of::<WCHAR>() as DWORD;
            error = RegGetValueW(
                key,
                core::ptr::null(),
                core::ptr::null(),
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                buffer.cast(),
                &mut bytes,
            );
        }
        RegCloseKey(key);
        if error == 0 {
            return error;
        }
    }

    let mut uppercase = [0u16; MAX_PATH + 33];
    extension = extension.add(1);
    if LCMapStringW(
        LOCALE_USER_DEFAULT,
        LCMAP_UPPERCASE,
        extension,
        -1,
        uppercase.as_mut_ptr(),
        uppercase.len() as INT,
    ) != 0
    {
        extension = uppercase.as_ptr();
    }
    get_extension_default_description(extension, buffer, capacity)
}

#[no_mangle]
pub unsafe extern "C" fn SHELL32_AssocGetFileDescription(
    name: PCWSTR,
    buffer: PWSTR,
    capacity: UINT,
) -> HRESULT {
    assoc_get_extension_description(PathFindExtensionW(name), buffer, capacity)
}

// ── Method bodies ───────────────────────────────────────────────────────────────────────

// CQueryAssociations.cpp:97-99 — members are zero-inited by the HEAP_ZERO_MEMORY factory.
pub unsafe fn CQueryAssociations_ctor_body(this: *mut CQueryAssociations) -> () {
    let _ = this;
}

// CQueryAssociations.cpp:101-103 — empty destructor (the C++ leaks any open keys too).
pub unsafe fn CQueryAssociations_dtor_body(this: *mut CQueryAssociations) -> () {
    let _ = this;
}

// CQueryAssociations.cpp:120-215
pub unsafe fn CQueryAssociations_Init_body(this: *mut CQueryAssociations, cfFlags: ASSOCF, mut pszAssoc: LPCWSTR, hkeyProgid: HKEY, hWnd: HWND) -> HRESULT {
    if !hWnd.is_null() {
        // FIXME("hwnd != NULL not supported\n");
    }

    if cfFlags != 0 {
        // FIXME("unsupported flags: %x\n", cfFlags);
    }

    RegCloseKey((*this).hkeySource);
    RegCloseKey((*this).hkeyProgID);
    (*this).hkeySource = core::ptr::null_mut();
    (*this).hkeyProgID = core::ptr::null_mut();
    if !pszAssoc.is_null() {
        let mut progId: *mut WCHAR = core::ptr::null_mut();
        let hr: HRESULT;
        let pchDotExt: LPCWSTR;

        if !StrChrW(pszAssoc, 0x005Cu16 /* L'\\' */).is_null() {
            pchDotExt = PathFindExtensionW(pszAssoc);
            if !pchDotExt.is_null() && *pchDotExt != 0u16 {
                pszAssoc = pchDotExt;
            }
        }

        let mut ret: LONG = RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            pszAssoc,
            0,
            KEY_READ,
            &mut (*this).hkeySource,
        );
        if ret != 0 {
            return S_OK;
        }

        /* if this is a progid */
        if *pszAssoc != 0x2Eu16 /* '.' */ && *pszAssoc != 0x7Bu16 /* '{' */ {
            (*this).hkeyProgID = (*this).hkeySource;
            return S_OK;
        }

        /* if it's not a progid, it's a file extension or clsid */
        if *pszAssoc == 0x2Eu16 {
            /* for a file extension, the progid is the default value */
            hr = CQueryAssociations_GetValue_body(
                this,
                (*this).hkeySource,
                core::ptr::null(),
                &mut progId as *mut *mut WCHAR as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );
            if hr < 0 {
                return S_OK;
            }
        } else /* if (*pszAssoc == '{') */ {
            let mut progIdKey: HKEY = core::ptr::null_mut();
            /* for a clsid, the progid is the default value of the ProgID subkey */
            ret = RegOpenKeyExW(
                (*this).hkeySource,
                CPP_COM_WSTR_89E3C2C9E5FD.as_ptr(),
                0,
                KEY_READ,
                &mut progIdKey,
            );
            if ret != ERROR_SUCCESS {
                return S_OK;
            }
            hr = CQueryAssociations_GetValue_body(
                this,
                progIdKey,
                core::ptr::null(),
                &mut progId as *mut *mut WCHAR as *mut *mut core::ffi::c_void,
                core::ptr::null_mut(),
            );
            if hr < 0 {
                return S_OK;
            }
            RegCloseKey(progIdKey);
        }

        /* open the actual progid key, the one with the shell subkey */
        RegOpenKeyExW(
            HKEY_CLASSES_ROOT,
            progId,
            0,
            KEY_READ,
            &mut (*this).hkeyProgID,
        );
        HeapFree(GetProcessHeap(), 0, progId as *mut core::ffi::c_void);

        S_OK
    } else if !hkeyProgid.is_null() {
        /* reopen the key so we don't end up closing a key owned by the caller */
        RegOpenKeyExW(
            hkeyProgid,
            core::ptr::null(),
            0,
            KEY_READ,
            &mut (*this).hkeyProgID,
        );
        (*this).hkeySource = (*this).hkeyProgID;
        S_OK
    } else {
        E_INVALIDARG
    }
}

// CQueryAssociations.cpp:233-471 — HAND-PORTED (cached body was damaged).
// pcchOut contract (via ReturnString, cpp:795-834): pcchOut is IN buffer size /
// OUT required size in WCHARs INCLUDING the terminator; on a too-small buffer the
// data is truncated to *pcchOut chars WITHOUT forced NUL (ERROR_INSUFFICIENT_BUFFER),
// or with ASSOCF_NOTRUNCATE nothing but out[0]=0 is written (E_POINTER); pszOut==NULL
// probes: *pcchOut = required and S_FALSE.
pub unsafe fn CQueryAssociations_GetString_body(this: *mut CQueryAssociations, flags: ASSOCF, str: ASSOCSTR, pszExtra: LPCWSTR, pszOut: LPWSTR, pcchOut: *mut DWORD) -> HRESULT {
    let unimplemented_flags: ASSOCF = !ASSOCF_NOTRUNCATE;
    let mut len: DWORD = 0;
    let mut hr: HRESULT;
    let mut path: [WCHAR; MAX_PATH] = [0; MAX_PATH];

    if flags & unimplemented_flags != 0 {
        // FIXME("%08x: unimplemented flags\n", flags & unimplemented_flags);
    }

    if pcchOut.is_null() {
        return E_UNEXPECTED;
    }

    if (*this).hkeySource.is_null() && (*this).hkeyProgID.is_null() {
        return HRESULT_FROM_WIN32(ERROR_NO_ASSOCIATION);
    }

    match str {
        ASSOCSTR_COMMAND => {
            // cpp:263-273
            let mut command: *mut WCHAR = core::ptr::null_mut();
            hr = CQueryAssociations_GetCommand_body(this, pszExtra, &mut command);
            if hr >= 0 /* SUCCEEDED */ {
                hr = CQueryAssociations_ReturnString_body(this, flags, pszOut, pcchOut, command, (wcslen(command) + 1) as DWORD);
                HeapFree(GetProcessHeap(), 0, command as *mut core::ffi::c_void);
            }
            hr
        }
        ASSOCSTR_EXECUTABLE => {
            // cpp:274-283
            hr = CQueryAssociations_GetExecutable_body(this, pszExtra, path.as_mut_ptr(), MAX_PATH as DWORD, &mut len);
            if hr < 0 /* FAILED_UNEXPECTEDLY == FAILED(hr) (shellutils.h:90, non-DBG) */ {
                return hr;
            }
            len += 1;
            CQueryAssociations_ReturnString_body(this, flags, pszOut, pcchOut, path.as_ptr(), len)
        }
        ASSOCSTR_FRIENDLYDOCNAME => {
            // cpp:284-322
            let mut pszFileType: *mut WCHAR = core::ptr::null_mut();

            hr = CQueryAssociations_GetValue_body(this, (*this).hkeySource, core::ptr::null(), &mut pszFileType as *mut *mut WCHAR as *mut *mut core::ffi::c_void, core::ptr::null_mut());
            if hr < 0 {
                return hr;
            }
            let mut size: DWORD = 0;
            let mut ret: LONG = RegGetValueW(
                HKEY_CLASSES_ROOT,
                pszFileType,
                core::ptr::null(),
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut size,
            );
            if ret == ERROR_SUCCESS {
                let docName = HeapAlloc(GetProcessHeap(), 0, (size as usize) as _) as *mut WCHAR;
                if !docName.is_null() {
                    ret = RegGetValueW(
                        HKEY_CLASSES_ROOT,
                        pszFileType,
                        core::ptr::null(),
                        RRF_RT_REG_SZ,
                        core::ptr::null_mut(),
                        docName as PVOID,
                        &mut size,
                    );
                    if ret == ERROR_SUCCESS {
                        hr = CQueryAssociations_ReturnString_body(this, flags, pszOut, pcchOut, docName, (wcslen(docName) + 1) as DWORD);
                    } else {
                        hr = HRESULT_FROM_WIN32(ret);
                    }
                    HeapFree(GetProcessHeap(), 0, docName as *mut core::ffi::c_void);
                } else {
                    hr = E_OUTOFMEMORY;
                }
            } else {
                hr = HRESULT_FROM_WIN32(ret);
            }
            HeapFree(GetProcessHeap(), 0, pszFileType as *mut core::ffi::c_void);
            hr
        }
        ASSOCSTR_FRIENDLYAPPNAME => {
            // cpp:323-376
            let mut verinfoW: PVOID = core::ptr::null_mut();
            let mut size: DWORD = 0;
            let mut retval: DWORD = 0;
            let mut flen: UINT = 0;
            let mut bufW: *mut WCHAR = core::ptr::null_mut();
            let mut fileDescW: [WCHAR; 41] = [0; 41];

            hr = CQueryAssociations_GetExecutable_body(this, pszExtra, path.as_mut_ptr(), MAX_PATH as DWORD, &mut len);
            if hr < 0 {
                return hr;
            }
            // C control flow: two `goto get_friendly_name_fail` sites plus fall-through
            // when no FileDescription matched → modeled as a labeled block whose exit
            // lands on the fail path below.
            'get_friendly_name: {
                retval = GetFileVersionInfoSizeW(path.as_ptr(), &mut size);
                if retval == 0 {
                    break 'get_friendly_name; // goto get_friendly_name_fail
                }
                verinfoW = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, (retval as usize) as _);
                if verinfoW.is_null() {
                    return E_OUTOFMEMORY;
                }
                if GetFileVersionInfoW(path.as_ptr(), 0, retval, verinfoW) == 0 {
                    break 'get_friendly_name; // goto get_friendly_name_fail
                }
                if VerQueryValueW(
                    verinfoW,
                    CPP_COM_WSTR_6AC3419A3E2E.as_ptr(), // L"\\VarFileInfo\\Translation"
                    &mut bufW as *mut *mut WCHAR as *mut LPVOID,
                    &mut flen,
                ) != 0
                {
                    let langCodeDesc = bufW as *const DWORD;
                    // for (i = 0; i < flen / sizeof(DWORD); i++) — flen and bufW are
                    // clobbered by the inner VerQueryValueW, exactly as in the C.
                    let mut i: UINT = 0;
                    while (i as usize) < (flen as usize) / core::mem::size_of::<DWORD>() {
                        swprintf(
                            fileDescW.as_mut_ptr(),
                            CPP_COM_WSTR_43CC49772B80.as_ptr(), // L"\\StringFileInfo\\%04x%04x\\FileDescription"
                            LOWORD(*langCodeDesc.add(i as usize)) as UINT,
                            HIWORD(*langCodeDesc.add(i as usize)) as UINT,
                        );
                        if VerQueryValueW(
                            verinfoW,
                            fileDescW.as_ptr(),
                            &mut bufW as *mut *mut WCHAR as *mut LPVOID,
                            &mut flen,
                        ) != 0
                        {
                            /* Does strlenW(bufW) == 0 mean we use the filename? */
                            len = (wcslen(bufW) + 1) as DWORD;
                            hr = CQueryAssociations_ReturnString_body(this, flags, pszOut, pcchOut, bufW, len);
                            HeapFree(GetProcessHeap(), 0, verinfoW);
                            return hr;
                        }
                        i += 1;
                    }
                }
            }
            // get_friendly_name_fail: (cpp:369-375)
            PathRemoveExtensionW(path.as_mut_ptr());
            PathStripPathW(path.as_mut_ptr());
            hr = CQueryAssociations_ReturnString_body(this, flags, pszOut, pcchOut, path.as_ptr(), (wcslen(path.as_ptr()) + 1) as DWORD);
            HeapFree(GetProcessHeap(), 0, verinfoW);
            hr
        }
        ASSOCSTR_CONTENTTYPE => {
            // cpp:377-404
            let mut size: DWORD = 0;
            let mut ret: LONG = RegGetValueW(
                (*this).hkeySource,
                core::ptr::null(),
                CPP_COM_WSTR_54E2098886DA.as_ptr(), // L"Content Type"
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut size,
            );
            if ret != ERROR_SUCCESS {
                return HRESULT_FROM_WIN32(ret);
            }
            let contentType = HeapAlloc(GetProcessHeap(), 0, (size as usize) as _) as *mut WCHAR;
            if !contentType.is_null() {
                ret = RegGetValueW(
                    (*this).hkeySource,
                    core::ptr::null(),
                    CPP_COM_WSTR_54E2098886DA.as_ptr(),
                    RRF_RT_REG_SZ,
                    core::ptr::null_mut(),
                    contentType as PVOID,
                    &mut size,
                );
                if ret == ERROR_SUCCESS {
                    hr = CQueryAssociations_ReturnString_body(this, flags, pszOut, pcchOut, contentType, (wcslen(contentType) + 1) as DWORD);
                } else {
                    hr = HRESULT_FROM_WIN32(ret);
                }
                HeapFree(GetProcessHeap(), 0, contentType as *mut core::ffi::c_void);
            } else {
                hr = E_OUTOFMEMORY;
            }
            hr
        }
        ASSOCSTR_DEFAULTICON => {
            // cpp:405-436
            let mut size: DWORD = 0;
            let mut ret: LONG = RegGetValueW(
                (*this).hkeyProgID,
                CPP_COM_WSTR_3BE8B25B7ED1.as_ptr(), // L"DefaultIcon"
                core::ptr::null(),
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut size,
            );
            if ret == ERROR_SUCCESS {
                let icon = HeapAlloc(GetProcessHeap(), 0, (size as usize) as _) as *mut WCHAR;
                if !icon.is_null() {
                    ret = RegGetValueW(
                        (*this).hkeyProgID,
                        CPP_COM_WSTR_3BE8B25B7ED1.as_ptr(),
                        core::ptr::null(),
                        RRF_RT_REG_SZ,
                        core::ptr::null_mut(),
                        icon as PVOID,
                        &mut size,
                    );
                    if ret == ERROR_SUCCESS {
                        hr = CQueryAssociations_ReturnString_body(this, flags, pszOut, pcchOut, icon, (wcslen(icon) + 1) as DWORD);
                    } else {
                        hr = HRESULT_FROM_WIN32(ret);
                    }
                    HeapFree(GetProcessHeap(), 0, icon as *mut core::ffi::c_void);
                } else {
                    // The pre-fix C mapped ret (== ERROR_SUCCESS here) through
                    // HRESULT_FROM_WIN32, reporting S_OK on alloc failure (upstream 7cba1c80a75).
                    hr = E_OUTOFMEMORY;
                }
            } else {
                hr = HRESULT_FROM_WIN32(ret);
            }
            hr
        }
        ASSOCSTR_SHELLEXTENSION => {
            // cpp:437-463
            // WCHAR keypath[ARRAY_SIZE(L"ShellEx\\") + 39] == [WCHAR; 9 + 39]
            let mut keypath: [WCHAR; 48] = [0; 48];
            let mut guid: [WCHAR; 39] = [0; 39];
            let mut clsid: CLSID = core::mem::zeroed();
            let mut hkey: HKEY = core::ptr::null_mut();

            hr = CLSIDFromString(pszExtra, &mut clsid);
            if hr < 0 {
                return hr;
            }
            wcscpy(keypath.as_mut_ptr(), CPP_COM_WSTR_E01CFD69C238.as_ptr()); // L"ShellEx\\"
            wcscat(keypath.as_mut_ptr(), pszExtra);
            let mut ret: LONG = RegOpenKeyExW((*this).hkeySource, keypath.as_ptr(), 0, KEY_READ, &mut hkey);
            if ret != 0 {
                return HRESULT_FROM_WIN32(ret);
            }
            let mut size: DWORD = core::mem::size_of_val(&guid) as DWORD;
            ret = RegGetValueW(
                hkey,
                core::ptr::null(),
                core::ptr::null(),
                RRF_RT_REG_SZ,
                core::ptr::null_mut(),
                guid.as_mut_ptr() as PVOID,
                &mut size,
            );
            RegCloseKey(hkey);
            if ret != 0 {
                return HRESULT_FROM_WIN32(ret);
            }
            CQueryAssociations_ReturnString_body(this, flags, pszOut, pcchOut, guid.as_ptr(), size / (core::mem::size_of::<WCHAR>() as DWORD))
        }
        _ => {
            // FIXME("assocstr %d unimplemented!\n", str);
            E_NOTIMPL
        }
    }
}

// CQueryAssociations.cpp:488-497 — stub in the C++ too.
pub unsafe fn CQueryAssociations_GetKey_body(this: *mut CQueryAssociations, cfFlags: ASSOCF, assockey: ASSOCKEY, pszExtra: LPCWSTR, phkeyOut: *mut HKEY) -> HRESULT {
    // FIXME("(%p,0x%8x,0x%8x,%s,%p)-stub!\n", ...);
    let _ = (this, cfFlags, assockey, pszExtra, phkeyOut);
    E_NOTIMPL
}

// CQueryAssociations.cpp:515-558
pub unsafe fn CQueryAssociations_GetData_body(this: *mut CQueryAssociations, cfFlags: ASSOCF, assocdata: ASSOCDATA, pszExtra: LPCWSTR, pvOut: LPVOID, pcbOut: *mut DWORD) -> HRESULT {
    let _ = pszExtra;
    if cfFlags != 0 {
        // FIXME("Unsupported flags: %x\n", cfFlags);
    }

    match assocdata {
        ASSOCDATA_EDITFLAGS => {
            if (*this).hkeyProgID.is_null() {
                return HRESULT_FROM_WIN32(ERROR_NO_ASSOCIATION);
            }

            let mut data: *mut core::ffi::c_void = core::ptr::null_mut();
            let mut size: DWORD = 0;
            let hres = CQueryAssociations_GetValue_body(
                this,
                (*this).hkeyProgID,
                CPP_COM_WSTR_A2E2913C930F.as_ptr(), // L"EditFlags"
                &mut data,
                &mut size,
            );
            if hres < 0 {
                return hres;
            }

            if pcbOut.is_null() {
                HeapFree(GetProcessHeap(), 0, data);
                return hres;
            }

            let hres = CQueryAssociations_ReturnData_body(this, pvOut, pcbOut, data, size);
            HeapFree(GetProcessHeap(), 0, data);
            hres
        }
        _ => {
            // FIXME("Unsupported ASSOCDATA value: %d\n", assocdata);
            E_NOTIMPL
        }
    }
}

// CQueryAssociations.cpp:579-587 — E_NOTIMPL in the C++ too.
pub unsafe fn CQueryAssociations_GetEnum_body(this: *mut CQueryAssociations, cfFlags: ASSOCF, assocenum: ASSOCENUM, pszExtra: LPCWSTR, riid: REFIID, ppvOut: *mut LPVOID) -> HRESULT {
    let _ = (this, cfFlags, assocenum, pszExtra, riid, ppvOut);
    E_NOTIMPL
}

// CQueryAssociations.cpp:589-616
pub unsafe fn CQueryAssociations_GetValue_body(this: *mut CQueryAssociations, hkey: HKEY, name: *const WCHAR, data: *mut *mut core::ffi::c_void, data_size: *mut DWORD) -> HRESULT {
    let _ = this;
    let mut size: DWORD = 0;

    let mut ret: LONG = (SHQueryValueExW(
        hkey,
        name,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut size,
    )) as i32;
    if ret != ERROR_SUCCESS {
        return HRESULT_FROM_WIN32(ret);
    }

    if size == 0 {
        return E_FAIL;
    }

    *data = HeapAlloc(GetProcessHeap(), 0, (size as usize) as _);
    if (*data).is_null() {
        return E_OUTOFMEMORY;
    }

    ret = (SHQueryValueExW(
        hkey,
        name,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        *data,
        &mut size,
    )) as i32;
    if ret != ERROR_SUCCESS {
        HeapFree(GetProcessHeap(), 0, *data);
        return HRESULT_FROM_WIN32(ret);
    }

    if !data_size.is_null() {
        *data_size = size;
    }

    S_OK
}

// CQueryAssociations.cpp:618-713
pub unsafe fn CQueryAssociations_GetCommand_body(this: *mut CQueryAssociations, mut extra: *const WCHAR, command: *mut *mut WCHAR) -> HRESULT {
    let mut hkeyCommand: HKEY = core::ptr::null_mut();
    let mut hkeyShell: HKEY = core::ptr::null_mut();
    let mut hkeyVerb: HKEY = core::ptr::null_mut();
    let mut hr: HRESULT;
    let mut ret: LONG;
    let mut extra_from_reg: *mut WCHAR = core::ptr::null_mut();
    let mut filetype: *mut WCHAR = core::ptr::null_mut();

    /* When looking for file extension it's possible to have a default value
     that points to another key that contains 'shell/<verb>/command' subtree. */
    hr = CQueryAssociations_GetValue_body(this, (*this).hkeySource, core::ptr::null(), &mut filetype as *mut *mut WCHAR as *mut *mut core::ffi::c_void, core::ptr::null_mut());
    if hr == S_OK {
        let mut hkeyFile: HKEY = core::ptr::null_mut();

        ret = RegOpenKeyExW(HKEY_CLASSES_ROOT, filetype, 0, KEY_READ, &mut hkeyFile);
        HeapFree(GetProcessHeap(), 0, filetype as *mut core::ffi::c_void);

        if ret == ERROR_SUCCESS {
            ret = RegOpenKeyExW(hkeyFile, CPP_COM_WSTR_EE89605B31B0.as_ptr(), 0, KEY_READ, &mut hkeyShell);
            RegCloseKey(hkeyFile);
        } else {
            ret = RegOpenKeyExW((*this).hkeySource, CPP_COM_WSTR_EE89605B31B0.as_ptr(), 0, KEY_READ, &mut hkeyShell);
        }
    } else {
        ret = RegOpenKeyExW((*this).hkeySource, CPP_COM_WSTR_EE89605B31B0.as_ptr(), 0, KEY_READ, &mut hkeyShell);
    }

    if ret != 0 {
        return HRESULT_FROM_WIN32(ret);
    }

    if extra.is_null() {
        /* check for default verb */
        hr = CQueryAssociations_GetValue_body(this, hkeyShell, core::ptr::null(), &mut extra_from_reg as *mut *mut WCHAR as *mut *mut core::ffi::c_void, core::ptr::null_mut());
        if hr < 0 {
            hr = CQueryAssociations_GetValue_body(this, hkeyShell, CPP_COM_WSTR_82DB442E7A60.as_ptr(), &mut extra_from_reg as *mut *mut WCHAR as *mut *mut core::ffi::c_void, core::ptr::null_mut());
        }
        if hr < 0 {
            /* no default verb, try first subkey */
            let mut max_subkey_len: DWORD = 0;

            ret = RegQueryInfoKeyW(
                hkeyShell,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                &mut max_subkey_len,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if ret != 0 {
                RegCloseKey(hkeyShell);
                return HRESULT_FROM_WIN32(ret);
            }

            max_subkey_len += 1;
            extra_from_reg = HeapAlloc(GetProcessHeap(), 0, ((max_subkey_len as usize) * core::mem::size_of::<WCHAR>()) as _) as *mut WCHAR;
            if extra_from_reg.is_null() {
                RegCloseKey(hkeyShell);
                return E_OUTOFMEMORY;
            }

            ret = RegEnumKeyExW(
                hkeyShell,
                0,
                extra_from_reg,
                &mut max_subkey_len,
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::null_mut(),
            );
            if ret != 0 {
                HeapFree(GetProcessHeap(), 0, extra_from_reg as *mut core::ffi::c_void);
                RegCloseKey(hkeyShell);
                return HRESULT_FROM_WIN32(ret);
            }
        }
        extra = extra_from_reg;
    }

    /* open verb subkey */
    ret = RegOpenKeyExW(hkeyShell, extra, 0, KEY_READ, &mut hkeyVerb);
    HeapFree(GetProcessHeap(), 0, extra_from_reg as *mut core::ffi::c_void);
    RegCloseKey(hkeyShell);
    if ret != 0 {
        return HRESULT_FROM_WIN32(ret);
    }
    /* open command subkey */
    ret = RegOpenKeyExW(hkeyVerb, CPP_COM_WSTR_0C71B3CB5580.as_ptr(), 0, KEY_READ, &mut hkeyCommand);
    RegCloseKey(hkeyVerb);
    if ret != 0 {
        return HRESULT_FROM_WIN32(ret);
    }
    hr = CQueryAssociations_GetValue_body(this, hkeyCommand, core::ptr::null(), command as *mut *mut core::ffi::c_void, core::ptr::null_mut());
    RegCloseKey(hkeyCommand);
    hr
}

// CQueryAssociations.cpp:715-773 — *len = SearchPathW char count EXCLUDING the NUL.
pub unsafe fn CQueryAssociations_GetExecutable_body(this: *mut CQueryAssociations, pszExtra: LPCWSTR, path: LPWSTR, pathlen: DWORD, len: *mut DWORD) -> HRESULT {
    let mut pszCommand: *mut WCHAR = core::ptr::null_mut();
    let pszStart: *mut WCHAR;
    let mut pszEnd: *mut WCHAR;

    let hr = CQueryAssociations_GetCommand_body(this, pszExtra, &mut pszCommand);
    if hr < 0 /* FAILED_UNEXPECTEDLY == FAILED(hr) (shellutils.h:90, non-DBG) */ {
        return hr;
    }

    let mut expLen: DWORD = ExpandEnvironmentStringsW(pszCommand, core::ptr::null_mut(), 0);
    if expLen > 0 {
        expLen += 1;
        let buf = HeapAlloc(GetProcessHeap(), 0, ((expLen as usize) * core::mem::size_of::<WCHAR>()) as _) as *mut WCHAR;
        ExpandEnvironmentStringsW(pszCommand, buf, expLen);
        HeapFree(GetProcessHeap(), 0, pszCommand as *mut core::ffi::c_void);
        pszCommand = buf;
    }

    /* cleanup pszCommand */
    if *pszCommand == 0x0022u16 /* '"' */ {
        pszStart = pszCommand.add(1);
        pszEnd = wcschr(pszStart, 0x0022u16);
        if !pszEnd.is_null() {
            *pszEnd = 0;
        }
        *len = SearchPathW(core::ptr::null(), pszStart, core::ptr::null(), pathlen, path, core::ptr::null_mut());
    } else {
        pszStart = pszCommand;
        // for (pszEnd = pszStart; (pszEnd = strchrW(pszEnd, ' ')); pszEnd++)
        pszEnd = pszStart;
        loop {
            pszEnd = wcschr(pszEnd, 0x0020u16 /* ' ' */);
            if pszEnd.is_null() {
                break;
            }
            let c: WCHAR = *pszEnd;
            *pszEnd = 0;
            *len = SearchPathW(core::ptr::null(), pszStart, core::ptr::null(), pathlen, path, core::ptr::null_mut());
            if *len != 0 {
                break;
            }
            *pszEnd = c;
            pszEnd = pszEnd.add(1);
        }
        if pszEnd.is_null() {
            *len = SearchPathW(core::ptr::null(), pszStart, core::ptr::null(), pathlen, path, core::ptr::null_mut());
        }
    }

    HeapFree(GetProcessHeap(), 0, pszCommand as *mut core::ffi::c_void);
    if *len == 0 {
        return HRESULT_FROM_WIN32(ERROR_FILE_NOT_FOUND);
    }
    S_OK
}

// CQueryAssociations.cpp:775-793
pub unsafe fn CQueryAssociations_ReturnData_body(this: *mut CQueryAssociations, out: *mut core::ffi::c_void, outlen: *mut DWORD, data: *const core::ffi::c_void, datalen: DWORD) -> HRESULT {
    let _ = this;
    if !out.is_null() {
        if *outlen < datalen {
            *outlen = datalen;
            return E_POINTER;
        }
        *outlen = datalen;
        memcpy(out, data, datalen as usize);
        S_OK
    } else {
        *outlen = datalen;
        S_FALSE
    }
}

// CQueryAssociations.cpp:795-834 — datalen counts WCHARs INCLUDING the terminator.
pub unsafe fn CQueryAssociations_ReturnString_body(this: *mut CQueryAssociations, flags: ASSOCF, out: LPWSTR, outlen: *mut DWORD, data: LPCWSTR, datalen: DWORD) -> HRESULT {
    let _ = this;
    let mut hr: HRESULT = S_OK;
    let len: DWORD;

    if out.is_null() {
        *outlen = datalen;
        return S_FALSE;
    }

    if *outlen < datalen {
        if flags & ASSOCF_NOTRUNCATE != 0 {
            len = 0;
            if *outlen > 0 {
                *out = 0;
            }
            hr = E_POINTER;
        } else {
            len = core::cmp::min(*outlen, datalen);
            hr = HRESULT_FROM_WIN32(ERROR_INSUFFICIENT_BUFFER);
        }
        *outlen = datalen;
    } else {
        *outlen = datalen;
        len = datalen;
    }

    if len != 0 {
        // Truncation copies raw chars with NO forced terminator — exact C semantics.
        memcpy(
            out as *mut core::ffi::c_void,
            data as *const core::ffi::c_void,
            (len as usize) * core::mem::size_of::<WCHAR>(),
        );
    }

    hr
}
