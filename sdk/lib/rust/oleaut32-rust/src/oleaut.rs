use crate::prelude::*;
use crate::typelib::LoadTypeLib;
use core::ffi::c_void;

// `stdmethod!` (the STDMETHODCALLTYPE vtbl-thunk macro that used to live here) is
// now crate-level in winemacros.rs — a file-local `macro_rules!` was invisible to
// every other module, so each new COM file re-hit the same wall.

// CRT printf family (cdecl varargs) — the only function externs kept locally;
// every other former local decl is provided by the prelude with the correct
// per-arch ABI (extern "system"/"C" there — local "C" decls were an i386
// stdcall/cdecl break).
extern "C" {
    fn sprintf(s: *mut i8, format: *const i8, ...) -> i32;
    fn snprintf(s: *mut i8, n: usize, format: *const i8, ...) -> i32;
}

// GUID data symbols provided by the C side of the DLL link (uuid lib + the
// widl proxy/stub unit); not in com-sys/prelude. Data statics carry no
// calling convention, so extern "C" is exact.
extern "C" {
    static IID_IPSFactoryBuffer: GUID;
    static IID_IClassFactory: GUID;
    static CLSID_StdFont: GUID;
    static CLSID_StdPicture: GUID;
}

// The oleaut32 proxy CLSIDs used to be emitted by oleaut.c's `#include "initguid.h"`
// (which turned the DEFINE_GUID in oleaut32_oaidl.h + typelib.h into definitions).
// Now that oleaut.c is flipped out, oleaut.rs is the single C-visible definition
// site. The generated widl proxy (`PROXY_CLSID=CLSID_PSFactoryBuffer` →
// `extern CLSID CLSID_PSFactoryBuffer;`) and every consumer bind to these.
//   - PSFactoryBuffer: oleaut32_oaidl.idl coclass uuid b196b286-bab4-101a-…
//   - the six PS* (Dispatch/EnumVariant/TypeInfo/TypeLib/OAInterface/TypeComp):
//     typelib.h DEFINE_OLEGUID base 000204xx-0000-0000-C000-000000000046.
macro_rules! def_ps_clsid {
    ($name:ident, $d1:expr, $d2:expr, $d3:expr, $d4:expr) => {
        #[no_mangle]
        pub static $name: GUID = GUID { Data1: $d1, Data2: $d2, Data3: $d3, Data4: $d4 };
    };
    ($name:ident, $d1:expr) => {
        def_ps_clsid!($name, $d1, 0, 0, [0xC0, 0, 0, 0, 0, 0, 0, 0x46]);
    };
}
def_ps_clsid!(CLSID_PSFactoryBuffer, 0xb196b286, 0xbab4, 0x101a,
    [0xb6, 0x9c, 0x00, 0xaa, 0x00, 0x34, 0x1d, 0x07]);
def_ps_clsid!(CLSID_PSDispatch,    0x00020420);
def_ps_clsid!(CLSID_PSEnumVariant, 0x00020421);
def_ps_clsid!(CLSID_PSTypeInfo,    0x00020422);
def_ps_clsid!(CLSID_PSTypeLib,     0x00020423);
def_ps_clsid!(CLSID_PSOAInterface, 0x00020424);
def_ps_clsid!(CLSID_PSTypeComp,    0x00020425);

// winreg.h:10: #define HKEY_CLASSES_ROOT ((HKEY)(LONG_PTR)(LONG)0x80000000)
// — the LONG→LONG_PTR step SIGN-EXTENDS on 64-bit (0xFFFFFFFF80000000).
const HKEY_CLASSES_ROOT: HKEY = 0x80000000u32 as i32 as isize as usize as HKEY;

// ---- file-scope preamble (translated) ----
static mut bstr_cache_enabled: BOOL = 0;

static mut cs_bstr_cache: CRITICAL_SECTION = CRITICAL_SECTION {
    DebugInfo: unsafe { &raw const cs_bstr_cache_dbg as *const _ as *mut _ },
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};

static mut cs_bstr_cache_dbg: CRITICAL_SECTION_DEBUG = CRITICAL_SECTION_DEBUG {
    Type: 0,
    CreatorBackTraceIndex: 0,
    CriticalSection: unsafe { &raw mut cs_bstr_cache },
    ProcessLocksList: _LIST_ENTRY {
        Flink: unsafe { &raw mut cs_bstr_cache_dbg.ProcessLocksList },
        Blink: unsafe { &raw mut cs_bstr_cache_dbg.ProcessLocksList },
    },
    EntryCount: 0,
    ContentionCount: 0,
    // The trailing member is a union (ndk/rtltypes.h:1432-1442); zero it via the
    // pointer-sized arm, which covers every alternative.
    c2rust_unnamed: RTL_CRITICAL_SECTION_DEBUG_u { WineDebugString: 0 },
};

#[repr(C)]
pub struct bstr_t {
    #[cfg(target_pointer_width = "64")]
    pub pad: DWORD,
    pub size: DWORD,
    pub u: bstr_t_u,
}

#[repr(C)]
pub union bstr_t_u {
    pub ptr: [CHAR; 1],
    pub str: [WCHAR; 1],
    pub dwptr: [DWORD; 1],
}

pub type Pbstr_t = *mut bstr_t;

pub const BUCKET_SIZE: usize = 16;
pub const BUCKET_BUFFER_SIZE: usize = 6;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct bstr_cache_entry_t {
    pub head: u16,
    pub cnt: u16,
    pub buf: [*mut bstr_t; BUCKET_BUFFER_SIZE],
}

pub type Pbstr_cache_entry_t = *mut bstr_cache_entry_t;

pub const ARENA_INUSE_FILLER: u8 = 0x55;
pub const ARENA_TAIL_FILLER: u8 = 0xab;
pub const ARENA_FREE_FILLER: u32 = 0xfeeefeee;

static mut bstr_cache: [bstr_cache_entry_t; 0x10000 / BUCKET_SIZE] = [bstr_cache_entry_t {
    head: 0,
    cnt: 0,
    buf: [core::ptr::null_mut(); BUCKET_BUFFER_SIZE],
}; 0x10000 / BUCKET_SIZE];

static _delimiter: [WCHAR; 2] = ['!' as WCHAR, 0];
static mut pdelimiter: *const WCHAR = unsafe { &_delimiter[0] };

static dispatch_typelib_ps_vtbl: IPSFactoryBufferVtbl = IPSFactoryBufferVtbl {
    QueryInterface: Some(dispatch_typelib_ps_QueryInterface),
    AddRef: Some(dispatch_typelib_ps_AddRef),
    Release: Some(dispatch_typelib_ps_Release),
    CreateProxy: Some(dispatch_typelib_ps_CreateProxy),
    CreateStub: Some(dispatch_typelib_ps_CreateStub),
};

static mut dispatch_typelib_ps: IPSFactoryBuffer = IPSFactoryBuffer {
    lpVtbl: &dispatch_typelib_ps_vtbl as *const _ as *mut _,
};

#[repr(C)]
struct ifacepsredirect_data {
    size: ULONG,
    mask: DWORD,
    iid: GUID,
    nummethods: ULONG,
    tlbid: GUID,
    base: GUID,
    name_len: ULONG,
    name_offset: ULONG,
}

#[repr(C)]
struct tlibredirect_data {
    size: ULONG,
    res: DWORD,
    name_len: ULONG,
    name_offset: ULONG,
    langid: LANGID,
    flags: WORD,
    help_len: ULONG,
    help_offset: ULONG,
    major_version: WORD,
    minor_version: WORD,
}

unsafe fn bstr_alloc_size(size: usize) -> usize {
    (core::mem::offset_of!(bstr_t, u) + size + core::mem::size_of::<WCHAR>() + BUCKET_SIZE - 1) & !(BUCKET_SIZE - 1)
}

unsafe fn bstr_from_str(str: BSTR) -> *mut bstr_t {
    (str as *mut u8).sub(core::mem::offset_of!(bstr_t, u)) as *mut bstr_t
}

unsafe fn get_cache_entry_from_idx(cache_idx: u32) -> *mut bstr_cache_entry_t {
    if bstr_cache_enabled != 0 && (cache_idx as usize) < (core::mem::size_of_val(&*&raw const bstr_cache) / core::mem::size_of::<bstr_cache_entry_t>()) {
        (&raw mut bstr_cache).cast::<bstr_cache_entry_t>().add(cache_idx as usize)
    } else {
        core::ptr::null_mut()
    }
}

unsafe fn get_cache_entry(size: usize) -> *mut bstr_cache_entry_t {
    let cache_idx = (core::mem::offset_of!(bstr_t, u) + size + core::mem::size_of::<u16>() - 1) / BUCKET_SIZE;
    get_cache_entry_from_idx(cache_idx as u32)
}

unsafe fn get_cache_entry_from_alloc_size(alloc_size: usize) -> *mut bstr_cache_entry_t {
    let cache_idx: u32;
    if alloc_size < BUCKET_SIZE {
        return core::ptr::null_mut();
    }
    cache_idx = ((alloc_size - BUCKET_SIZE) / BUCKET_SIZE) as u32;
    return get_cache_entry_from_idx(cache_idx);
}

unsafe fn alloc_bstr(size: usize) -> *mut bstr_t {
    
    let cache_entry = get_cache_entry(size);
    let mut ret: *mut bstr_t;

    if !cache_entry.is_null() {
        EnterCriticalSection(core::ptr::addr_of_mut!(cs_bstr_cache));

        let mut cache_entry = cache_entry;
        if (*cache_entry).cnt == 0 {
            cache_entry = get_cache_entry(size + BUCKET_SIZE);
            if !cache_entry.is_null() && (*cache_entry).cnt == 0 {
                cache_entry = core::ptr::null_mut();
            }
        }

        if !cache_entry.is_null() {
            ret = (*cache_entry).buf[(*cache_entry).head as usize];
            (*cache_entry).head = ((*cache_entry).head + 1) % BUCKET_BUFFER_SIZE as u16;
            (*cache_entry).cnt -= 1;
        } else {
            ret = core::ptr::null_mut();
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!(cs_bstr_cache));

        if !cache_entry.is_null() {
            if 0 != 0 {
                let field_offset = core::mem::offset_of!(bstr_t, u);
                let fill_size = (field_offset + size + 2 * core::mem::size_of::<WCHAR>() - 1) & !(core::mem::size_of::<WCHAR>() - 1);
                core::ptr::write_bytes(ret as *mut u8, ARENA_INUSE_FILLER, fill_size);
                core::ptr::write_bytes((ret as *mut u8).add(fill_size), ARENA_TAIL_FILLER, bstr_alloc_size(size) - fill_size);
            }
            (*ret).size = size as u32;
            return ret;
        }
    }

    ret = CoTaskMemAlloc(bstr_alloc_size(size) as SIZE_T) as *mut bstr_t;
    if !ret.is_null() {
        (*ret).size = size as u32;
    }
    ret
}

#[no_mangle]
pub unsafe extern "system" fn SysStringLen(str: BSTR) -> UINT {
    if !str.is_null() {
        (*bstr_from_str(str)).size / core::mem::size_of::<WCHAR>() as DWORD
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn SysStringByteLen(str: BSTR) -> UINT {
    if !str.is_null() {
        (*bstr_from_str(str)).size
    } else {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn SysAllocString(str: LPCOLESTR) -> BSTR {
    if str.is_null() {
        return 0 as BSTR;
    }

    SysAllocStringLen(str, lstrlenW(str) as u32)
}

unsafe fn get_malloc() -> *mut IMalloc {
    static mut malloc: *mut IMalloc = core::ptr::null_mut();

    if malloc.is_null() {
        CoGetMalloc(1, &raw mut malloc);
    }

    malloc
}

#[no_mangle]
pub unsafe extern "system" fn SysFreeString(str: BSTR) {
    let cache_entry: *mut bstr_cache_entry_t;
    let bstr: *mut bstr_t;
    let malloc: *mut IMalloc = get_malloc();
    let alloc_size: usize;

    if str.is_null() {
        return;
    }

    bstr = bstr_from_str(str);

    alloc_size = IMalloc_GetSize(malloc, bstr as LPVOID) as usize;
    if alloc_size == !0usize {
        return;
    }

    cache_entry = get_cache_entry_from_alloc_size(alloc_size);
    if !cache_entry.is_null() {
        let mut i: u32;

        EnterCriticalSection(core::ptr::addr_of_mut!(cs_bstr_cache));

        i = 0;
        while i < (*cache_entry).cnt as u32 {
            if (*cache_entry).buf[(((*cache_entry).head as u32 + i) % BUCKET_BUFFER_SIZE as u32) as usize] == bstr {
                WARN!(b"String already is in cache!\n\0".as_ptr() as LPCSTR);
                LeaveCriticalSection(core::ptr::addr_of_mut!(cs_bstr_cache));
                return;
            }
            i += 1;
        }

        if ((*cache_entry).cnt as usize) < BUCKET_BUFFER_SIZE {
            (*cache_entry).buf[(((*cache_entry).head as u32 + (*cache_entry).cnt as u32) % BUCKET_BUFFER_SIZE as u32) as usize] = bstr;
            (*cache_entry).cnt += 1;

            if 0 != 0 {
                let n: u32 = ((alloc_size - core::mem::offset_of!(bstr_t, u)) / core::mem::size_of::<DWORD>()) as u32;
                i = 0;
                while i < n {
                    (*bstr).u.dwptr[i as usize] = ARENA_FREE_FILLER;
                    i += 1;
                }
            }

            LeaveCriticalSection(core::ptr::addr_of_mut!(cs_bstr_cache));
            return;
        }

        LeaveCriticalSection(core::ptr::addr_of_mut!(cs_bstr_cache));
    }

    CoTaskMemFree(bstr as LPVOID);
}

#[no_mangle]
pub unsafe extern "system" fn SysAllocStringLen(str: *const OLECHAR, len: u32) -> BSTR {
    let bstr: *mut bstr_t;
    let size: DWORD;

    if len >= ((u32::MAX - core::mem::size_of::<WCHAR>() as u32 - core::mem::size_of::<DWORD>() as u32) / core::mem::size_of::<WCHAR>() as u32) {
        return core::ptr::null_mut();
    }

    TRACE!(b"%s\n\0".as_ptr() as PCHAR, debugstr_wn(str, len as i32));

    size = len * core::mem::size_of::<WCHAR>() as u32;
    bstr = alloc_bstr(size as usize);
    if bstr.is_null() {
        return core::ptr::null_mut();
    }

    if !str.is_null() {
        memcpy((*bstr).u.str.as_mut_ptr() as *mut core::ffi::c_void, str as *const core::ffi::c_void, size as usize);
        *(*bstr).u.str.as_mut_ptr().add(len as usize) = 0;
    } else {
        memset((*bstr).u.str.as_mut_ptr() as *mut core::ffi::c_void, 0, (size + core::mem::size_of::<WCHAR>() as u32) as usize);
    }

    (*bstr).u.str.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "system" fn SysReAllocStringLen(old: *mut BSTR, str: *const OLECHAR, len: u32) -> i32 {

    if len >= ((u32::MAX - core::mem::size_of::<WCHAR>() as u32 - core::mem::size_of::<DWORD>() as u32) / core::mem::size_of::<WCHAR>() as u32) {
        return 0;
    }

    if (*old) != core::ptr::null_mut() {
        let newbytelen = (len * core::mem::size_of::<WCHAR>() as u32) as DWORD;
        let old_bstr = bstr_from_str(*old);
        let bstr = CoTaskMemRealloc(old_bstr as LPVOID, bstr_alloc_size(newbytelen as usize) as SIZE_T) as *mut bstr_t;

        if bstr.is_null() {
            return 0;
        }

        *old = (*bstr).u.str.as_mut_ptr();
        (*bstr).size = newbytelen;
        if !str.is_null() && (*old_bstr).u.str.as_ptr() != str {
            memmove((*bstr).u.str.as_mut_ptr() as *mut core::ffi::c_void, str as *const core::ffi::c_void, newbytelen as usize);
        }
        *(*bstr).u.str.as_mut_ptr().offset(len as isize) = 0;
    } else {
        *old = SysAllocStringLen(str, len);
    }

    1
}

#[no_mangle]
pub unsafe extern "system" fn SysAllocStringByteLen(str_: LPCSTR, len: u32) -> *mut u16 {

    if len >= (u32::MAX - core::mem::size_of::<u16>() as u32 - core::mem::size_of::<u32>() as u32) {
        return core::ptr::null_mut();
    }

    let bstr = alloc_bstr(len as usize);
    if bstr.is_null() {
        return core::ptr::null_mut();
    }

    if !str_.is_null() {
        memcpy((*bstr).u.ptr.as_mut_ptr() as *mut core::ffi::c_void, str_ as *const core::ffi::c_void, len as usize);
        *(*bstr).u.ptr.as_mut_ptr().add(len as usize) = 0;
    } else {
        memset((*bstr).u.ptr.as_mut_ptr() as *mut core::ffi::c_void, 0, (len + 1) as usize);
    }
    *(*bstr).u.str.as_mut_ptr().add((len as usize + core::mem::size_of::<u16>() - 1) / core::mem::size_of::<u16>() ) = 0;

    (*bstr).u.str.as_mut_ptr()
}

#[no_mangle]
pub unsafe extern "system" fn SysReAllocString(old: LPBSTR, str: LPCOLESTR) -> INT {
    if old.is_null() {
        return 0;
    }

    SysFreeString(*old);

    *old = SysAllocString(str);

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn SetOaNoCache() {
    TRACE!(b"\n\0".as_ptr() as LPCSTR);
    bstr_cache_enabled = 0;
}

#[no_mangle]
pub unsafe extern "system" fn RegisterActiveObject(
    punk: LPUNKNOWN,
    rcid: REFCLSID,
    dwFlags: DWORD,
    pdwRegister: LPDWORD,
) -> HRESULT {
    let mut guidbuf: [WCHAR; 80] = [0; 80];
    let mut ret: HRESULT;
    let mut runobtable: LPRUNNINGOBJECTTABLE = core::ptr::null_mut();
    let mut moniker: LPMONIKER = core::ptr::null_mut();
    let mut rot_flags: DWORD = ROTFLAGS_REGISTRATIONKEEPSALIVE;

    StringFromGUID2(rcid, guidbuf.as_mut_ptr(), 39);
    ret = CreateItemMoniker(pdelimiter, guidbuf.as_ptr(), &mut moniker);
    if FAILED(ret) {
        return ret;
    }
    ret = GetRunningObjectTable(0, &mut runobtable);
    if FAILED(ret) {
        IMoniker_Release(moniker);
        return ret;
    }
    if dwFlags == ACTIVEOBJECT_WEAK {
        rot_flags = 0;
    }
    ret = IRunningObjectTable_Register(runobtable, rot_flags, punk, moniker, pdwRegister);
    IRunningObjectTable_Release(runobtable);
    IMoniker_Release(moniker);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn RevokeActiveObject(xregister: DWORD, _reserved: LPVOID) -> HRESULT {

    let mut runobtable: LPRUNNINGOBJECTTABLE = core::ptr::null_mut();
    let mut ret: HRESULT;

    ret = GetRunningObjectTable(0, &mut runobtable);
    if FAILED(ret) {
        return ret;
    }
    ret = IRunningObjectTable_Revoke(runobtable, xregister);
    if SUCCEEDED(ret) {
        ret = 0;
    }
    IRunningObjectTable_Release(runobtable);
    ret
}

#[no_mangle]
pub unsafe extern "system" fn GetActiveObject(
    rcid: *const GUID,
    _preserved: LPVOID,
    ppunk: *mut *mut core::ffi::c_void,
) -> HRESULT {

    let mut guidbuf: [u16; 80] = [0; 80];
    let mut ret: HRESULT;
    let mut runobtable: LPRUNNINGOBJECTTABLE = core::ptr::null_mut();
    let mut moniker: LPMONIKER = core::ptr::null_mut();

    StringFromGUID2(rcid, guidbuf.as_mut_ptr(), 39);
    ret = CreateItemMoniker(pdelimiter, guidbuf.as_ptr(), &mut moniker);
    if FAILED(ret) {
        return ret;
    }
    ret = GetRunningObjectTable(0, &mut runobtable);
    if FAILED(ret) {
        IMoniker_Release(moniker);
        return ret;
    }
    ret = IRunningObjectTable_GetObject(runobtable, moniker, ppunk as *mut *mut IUnknown);
    IRunningObjectTable_Release(runobtable);
    IMoniker_Release(moniker);
    return ret;
}
#[no_mangle]
pub unsafe extern "system" fn OaBuildVersion() -> ULONG {
    
    match GetVersion() & 0x8000ffff {
        0x80000a03 => MAKELONG(0xffff, 20) as ULONG,
        0x00003303 => MAKELONG(0xffff, 30) as ULONG,
        0x80000004 | 0x80000a04 | 0x00000004 | 0x00000005 => MAKELONG(0xffff, 40) as ULONG,
        // 0x206 is Win8/10/11 (Wine 10.0); unknown versions now answer as Win10.
        0x00000105 | 0x00000205 | 0x00000006 | 0x00000106 | 0x00000206 => MAKELONG(0xffff, 50) as ULONG,
        _ => {
            FIXME!(b"Version value not known yet. Please investigate it !\n\0".as_ptr() as PCHAR);
            MAKELONG(0xffff, 50) as ULONG
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn OleTranslateColor(
    clr: u32,
    hpal: HPALETTE,
    pColorRef: *mut COLORREF,
) -> HRESULT {
    let mut colorref: COLORREF = 0;
    let b: u8 = HIBYTE(HIWORD(clr));

    TRACE!(b"(%08x, %p, %p)\n\0".as_ptr() as LPCSTR, clr, hpal, pColorRef);

    let pColorRef = if pColorRef.is_null() {
        &mut colorref
    } else {
        &mut *pColorRef
    };

    match b {
        0x00 => {
            if hpal != 0 as HPALETTE {
                *pColorRef = PALETTERGB(GetRValue(clr), GetGValue(clr), GetBValue(clr));
            } else {
                *pColorRef = clr;
            }
        }

        0x01 => {
            if hpal != 0 as HPALETTE {
                let mut pe: PALETTEENTRY = core::mem::zeroed();
                if GetPaletteEntries(hpal, LOWORD(clr) as UINT, 1, &mut pe) == 0 {
                    return 0x80070057u32 as HRESULT;
                }
            }

            *pColorRef = clr;
        }

        0x02 => {
            *pColorRef = clr;
        }

        0x80 => {
            let index: i32 = LOBYTE(LOWORD(clr)) as i32;

            if (index < COLOR_SCROLLBAR as i32) || (index > COLOR_MENUBAR as i32) {
                return 0x80070057u32 as HRESULT;
            }

            *pColorRef = GetSysColor(index);
        }

        _ => {
            return 0x80070057u32 as HRESULT;
        }
    }

    0u32 as HRESULT
}

unsafe fn actctx_get_typelib_module(iid: *const GUID, module: *mut WCHAR, len: DWORD) -> BOOL {
    let mut data: ACTCTX_SECTION_KEYED_DATA = core::mem::zeroed();
    data.cbSize = core::mem::size_of::<ACTCTX_SECTION_KEYED_DATA>() as ULONG;
    
    if FindActCtxSectionGuid(
        0,
        core::ptr::null(),
        ACTIVATION_CONTEXT_SECTION_COM_INTERFACE_REDIRECTION,
        iid,
        &mut data,
    ) == 0
    {
        return 0;
    }

    let iface = data.lpData as *const ifacepsredirect_data;
    if FindActCtxSectionGuid(
        0,
        core::ptr::null(),
        ACTIVATION_CONTEXT_SECTION_COM_TYPE_LIBRARY_REDIRECTION,
        core::ptr::addr_of!((*iface).tlbid),
        &mut data,
    ) == 0
    {
        return 0;
    }

    let tlib = data.lpData as *const tlibredirect_data;
    let ptrW = (data.lpSectionBase as *const u8).add((*tlib).name_offset as usize) as *const WCHAR;

    if (*tlib).name_len / core::mem::size_of::<WCHAR>() as ULONG >= len {
        ERR!(b"need larger module buffer, %u\n\0".as_ptr() as *const i8, (*tlib).name_len);
        return 0;
    }

    memcpy(
        module as *mut c_void,
        ptrW as *const c_void,
        (*tlib).name_len as usize,
    );
    *module.add(((*tlib).name_len / core::mem::size_of::<WCHAR>() as ULONG) as usize) = 0;
    1
}

unsafe fn reg_get_typelib_module(iid: REFIID, module: *mut u16, len: u32) -> i32 {

    let opposite: REGSAM = if core::mem::size_of::<*mut core::ffi::c_void>() == 8 {
        KEY_WOW64_32KEY
    } else {
        KEY_WOW64_64KEY
    };
    
    let mut tlguid: [i8; 200] = [0; 200];
    // Wine 10.0: 316 = "Typelib\\" + tlguid (200) + "\\" + ver (100) +
    // "\\0\\win32" + NUL; 300 could be overrun by maximal registry values.
    let mut typelibkey: [i8; 316] = [0; 316];
    let mut interfacekey: [i8; 300] = [0; 300];
    let mut ver: [i8; 100] = [0; 100];
    let mut tlfn: [i8; 260] = [0; 260];
    let mut tlguidlen: u32;
    let mut verlen: u32;
    let mut type_: u32 = 0;
    let mut tlfnlen: i32;
    let mut err: i32;
    let mut is_wow64: i32 = 0;
    let mut ikey: HKEY = core::ptr::null_mut();

    sprintf(
        interfacekey.as_mut_ptr(),
        b"Interface\\{%08x-%04x-%04x-%02x%02x-%02x%02x%02x%02x%02x%02x}\\Typelib\0".as_ptr() as *const i8,
        (*iid).Data1,
        (*iid).Data2 as c_uint,
        (*iid).Data3 as c_uint,
        (*iid).Data4[0] as c_uint,
        (*iid).Data4[1] as c_uint,
        (*iid).Data4[2] as c_uint,
        (*iid).Data4[3] as c_uint,
        (*iid).Data4[4] as c_uint,
        (*iid).Data4[5] as c_uint,
        (*iid).Data4[6] as c_uint,
        (*iid).Data4[7] as c_uint
    );

    err = RegOpenKeyExA(0x80000000u32 as HKEY, interfacekey.as_ptr(), 0, KEY_READ, &mut ikey);
    if err != 0 && (opposite == KEY_WOW64_32KEY || (IsWow64Process(GetCurrentProcess(), &mut is_wow64) != 0 && is_wow64 != 0)) {
        err = RegOpenKeyExA(0x80000000u32 as HKEY, interfacekey.as_ptr(), 0, KEY_READ | opposite, &mut ikey);
    }

    if err != 0 {
        ERR!(b"No %s key found.\n\0".as_ptr() as *const i8, interfacekey.as_ptr());
        return -2147467259i32;
    }

    tlguidlen = core::mem::size_of_val(&tlguid) as u32;
    if RegQueryValueExA(ikey, core::ptr::null(), core::ptr::null_mut(), &mut type_, tlguid.as_mut_ptr() as *mut u8, &mut tlguidlen) != 0 {
        ERR!(b"Getting typelib guid failed.\n\0".as_ptr() as *const i8);
        RegCloseKey(ikey);
        return -2147467259i32;
    }

    verlen = core::mem::size_of_val(&ver) as u32;
    if RegQueryValueExA(ikey, b"Version\0".as_ptr() as LPCSTR, core::ptr::null_mut(), &mut type_, ver.as_mut_ptr() as *mut u8, &mut verlen) != 0 {
        ERR!(b"Could not get version value?\n\0".as_ptr() as *const i8);
        RegCloseKey(ikey);
        return -2147467259i32;
    }

    RegCloseKey(ikey);

    snprintf(
        typelibkey.as_mut_ptr(),
        core::mem::size_of_val(&typelibkey),
        b"Typelib\\%s\\%s\\0\\win%u\0".as_ptr() as *const i8,
        tlguid.as_ptr(),
        ver.as_ptr(),
        if core::mem::size_of::<*mut core::ffi::c_void>() == 8 { 64u32 } else { 32u32 }
    );

    tlfnlen = core::mem::size_of_val(&tlfn) as i32;
    if RegQueryValueA(0x80000000u32 as HKEY, typelibkey.as_ptr(), tlfn.as_mut_ptr(), &mut tlfnlen) != 0 {
        #[cfg(target_pointer_width = "64")]
        {
            snprintf(
                typelibkey.as_mut_ptr(),
                core::mem::size_of_val(&typelibkey),
                b"Typelib\\%s\\%s\\0\\win32\0".as_ptr() as *const i8,
                tlguid.as_ptr(),
                ver.as_ptr()
            );
            tlfnlen = core::mem::size_of_val(&tlfn) as i32;
            if RegQueryValueA(0x80000000u32 as HKEY, typelibkey.as_ptr(), tlfn.as_mut_ptr(), &mut tlfnlen) != 0 {
                ERR!(b"Could not get typelib fn?\n\0".as_ptr() as *const i8);
                return -2147467259i32;
            }
        }
        #[cfg(not(target_pointer_width = "64"))]
        {
            ERR!(b"Could not get typelib fn?\n\0".as_ptr() as *const i8);
            return -2147467259i32;
        }
    }

    MultiByteToWideChar(0, 0, tlfn.as_ptr(), -1, module, len as i32);
    0i32
}

unsafe fn get_typeinfo_for_iid(iid: REFIID, typeinfo: *mut *mut ITypeInfo) -> HRESULT {
    let mut module: [u16; MAX_PATH as usize] = core::mem::zeroed();
    let mut typelib: *mut ITypeLib = core::ptr::null_mut();
    let mut hr: HRESULT;

    *typeinfo = core::ptr::null_mut();

    module[0] = 0;
    if actctx_get_typelib_module(iid, module.as_mut_ptr(), module.len() as DWORD) == 0 {
        hr = reg_get_typelib_module(iid, module.as_mut_ptr(), module.len() as u32);
        if FAILED(hr) {
            return hr;
        }
    }

    // The crate exposes two ABI-identical `ITypeLib` COM shims (com_sys vs olesys);
    // LoadTypeLib is typed against olesys' while our `typelib` var and the
    // ITypeLib_* accessors below use com_sys'. Bridge with a raw-pointer cast — same
    // underlying COM object, just a translation-surface type split.
    hr = LoadTypeLib(
        module.as_ptr(),
        &mut typelib as *mut *mut ITypeLib as *mut *mut crate::olesys::ITypeLib,
    );
    if hr != S_OK {
        ERR!(b"Failed to load typelib for %s, but it should be there.\n\0".as_ptr() as PCHAR, debugstr_guid(iid));
        return hr;
    }

    hr = ITypeLib_GetTypeInfoOfGuid(typelib, iid, typeinfo);
    ITypeLib_Release(typelib);
    if hr != S_OK {
        ERR!(b"typelib does not contain info for %s\n\0".as_ptr() as PCHAR, debugstr_guid(iid));
    }

    hr
}

stdmethod! { fn dispatch_typelib_ps_QueryInterface(
    iface: *mut IPSFactoryBuffer,
    iid: *const GUID,
    out: *mut *mut core::ffi::c_void,
) -> HRESULT {
    const S_OK: HRESULT = 0;
    const E_NOINTERFACE: HRESULT = 0x80004002u32 as HRESULT;

    if IsEqualIID(iid, &IID_IPSFactoryBuffer) || IsEqualIID(iid, &IID_IUnknown) {
        *out = iface as *mut core::ffi::c_void;
        return S_OK;
    }

    FIXME!(b"No interface for %s.\n\0".as_ptr() as *const i8, debugstr_guid(iid));
    *out = core::ptr::null_mut();
    return E_NOINTERFACE;
} }

stdmethod! { fn dispatch_typelib_ps_AddRef(iface: *mut IPSFactoryBuffer) -> ULONG {
    2
} }

stdmethod! { fn dispatch_typelib_ps_Release(iface: *mut IPSFactoryBuffer) -> ULONG {
    return 1;
} }

unsafe fn dispatch_create_proxy(
    outer: *mut IUnknown,
    proxy: *mut *mut IRpcProxyBuffer,
    out: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut factory: *mut IPSFactoryBuffer = core::ptr::null_mut();
    let hr = OLEAUTPS_DllGetClassObject(
        &CLSID_PSFactoryBuffer,
        &IID_IPSFactoryBuffer,
        &mut factory as *mut *mut IPSFactoryBuffer as *mut *mut core::ffi::c_void,
    );
    if FAILED(hr) {
        return hr;
    }

    let hr = IPSFactoryBuffer_CreateProxy(factory, outer, &IID_IDispatch, proxy, out);
    IPSFactoryBuffer_Release(factory);
    hr
}

stdmethod! { fn dispatch_typelib_ps_CreateProxy(
    iface: *mut IPSFactoryBuffer,
    outer: *mut IUnknown,
    iid: REFIID,
    proxy: *mut *mut IRpcProxyBuffer,
    out: *mut *mut core::ffi::c_void,
) -> HRESULT {
    let mut typeinfo: *mut ITypeInfo = core::ptr::null_mut();
    let mut attr: *mut TYPEATTR = core::ptr::null_mut();
    let mut hr: HRESULT;

    if IsEqualGUID(iid, &IID_IDispatch) {
        return dispatch_create_proxy(outer, proxy, out);
    }

    hr = get_typeinfo_for_iid(iid, &mut typeinfo);
    if FAILED(hr) {
        return hr;
    }

    hr = ITypeInfo_GetTypeAttr(typeinfo, &mut attr);
    if FAILED(hr) {
        ITypeInfo_Release(typeinfo);
        return hr;
    }

    if (*attr).typekind == TKIND_INTERFACE || ((*attr).wTypeFlags & TYPEFLAG_FDUAL as u16) != 0 {
        hr = CreateProxyFromTypeInfo(typeinfo, outer, iid, proxy, out);
    } else {
        hr = dispatch_create_proxy(outer, proxy, out);
    }

    if FAILED(hr) {
        ERR!(b"Failed to create proxy, hr %#x.\n\0".as_ptr() as PCHAR, hr);
    }

    ITypeInfo_ReleaseTypeAttr(typeinfo, attr);
    ITypeInfo_Release(typeinfo);
    hr
} }

unsafe fn dispatch_create_stub(server: *mut IUnknown, stub: *mut *mut IRpcStubBuffer) -> HRESULT {
    let mut factory: *mut IPSFactoryBuffer = core::ptr::null_mut();
    let mut hr: HRESULT;

    hr = OLEAUTPS_DllGetClassObject(
        &CLSID_PSFactoryBuffer,
        &IID_IPSFactoryBuffer,
        &mut factory as *mut *mut IPSFactoryBuffer as *mut LPVOID,
    );
    if FAILED(hr) {
        return hr;
    }

    hr = IPSFactoryBuffer_CreateStub(factory, &IID_IDispatch, server, stub);
    IPSFactoryBuffer_Release(factory);
    return hr;
}

// ---- wine-10 dispinterface_create_stub ------------------------------------------------
//
// A dispinterface's IID is a DIID_*, identical to IDispatch on the wire, but the object
// need not answer QueryInterface(IID_IDispatch). rpcrt4's standard CreateStub queries the
// server for the iid it is given, so asking it for an IID_IDispatch stub QI'd the object
// for IDispatch (Wine's tmarshal test_marshal_dispinterface fails on exactly that). Like
// wine-10, build the CStdStubBuffer by hand from rpcrt4's exported methods and the
// IDispatch stub vtbl of the widl-generated oaidl proxy file, querying for `iid` instead.

/// rpcproxy.h CInterfaceStubHeader.
#[repr(C)]
#[derive(Clone, Copy)]
struct CInterfaceStubHeader {
    piid: *const GUID,
    pServerInfo: *const c_void,
    DispatchTableCount: ULONG,
    pDispatchTable: *const c_void,
}

/// IRpcStubBufferVtbl, slot for slot.
#[repr(C)]
struct RpcStubBufferVtbl {
    QueryInterface: *const c_void,
    AddRef: *const c_void,
    Release: unsafe extern "system" fn(*mut CStdStubBuffer) -> ULONG,
    Connect: *const c_void,
    Disconnect: *const c_void,
    Invoke: *const c_void,
    IsIIDSupported: *const c_void,
    CountRefs: *const c_void,
    DebugServerQueryInterface: *const c_void,
    DebugServerRelease: *const c_void,
}

/// rpcproxy.h CInterfaceStubVtbl: the header sits immediately before the vtbl, and
/// rpcrt4 finds it from `lpVtbl` by that adjacency.
#[repr(C)]
struct CInterfaceStubVtbl {
    header: CInterfaceStubHeader,
    Vtbl: RpcStubBufferVtbl,
}

/// rpcproxy.h CStdStubBuffer.
#[repr(C)]
struct CStdStubBuffer {
    lpVtbl: *const RpcStubBufferVtbl,
    RefCount: LONG,
    pvServerObject: *mut IUnknown,
    pCallFactoryVtbl: *const c_void,
    pAsyncIID: *const GUID,
    pPSFactory: *mut c_void,
}

#[repr(C)]
struct dispinterface_stub {
    stub_vtbl: CInterfaceStubVtbl,
    stub_buffer: CStdStubBuffer,
}

/// rpcproxy.h ProxyFileInfo, up to the field read here.
#[repr(C)]
struct ProxyFileInfoHead {
    pProxyVtblList: *const c_void,
    pStubVtblList: *const *const CInterfaceStubVtbl,
}

extern "C" {
    // oleaut32_oaidl_p.c, generated by widl into this DLL.
    static oleaut32_oaidl_ProxyFileInfo: ProxyFileInfoHead;
}

extern "system" {
    fn CStdStubBuffer_QueryInterface(This: *mut c_void, riid: REFIID, ppv: *mut *mut c_void) -> HRESULT;
    fn CStdStubBuffer_AddRef(This: *mut c_void) -> ULONG;
    fn CStdStubBuffer_Connect(This: *mut c_void, server: *mut IUnknown) -> HRESULT;
    fn CStdStubBuffer_Disconnect(This: *mut c_void);
    fn CStdStubBuffer_Invoke(This: *mut c_void, msg: *mut c_void, chan: *mut c_void) -> HRESULT;
    fn CStdStubBuffer_IsIIDSupported(This: *mut c_void, riid: REFIID) -> *mut c_void;
    fn CStdStubBuffer_CountRefs(This: *mut c_void) -> ULONG;
    fn CStdStubBuffer_DebugServerQueryInterface(This: *mut c_void, ppv: *mut *mut c_void) -> HRESULT;
    fn CStdStubBuffer_DebugServerRelease(This: *mut c_void, pv: *mut c_void);
}

unsafe extern "system" fn dispinterface_stub_Release(iface: *mut CStdStubBuffer) -> ULONG {
    let stub = (iface as *mut u8).sub(core::mem::offset_of!(dispinterface_stub, stub_buffer))
        as *mut dispinterface_stub;
    let refcount = InterlockedDecrement(core::ptr::addr_of_mut!((*iface).RefCount)) as ULONG;

    if refcount == 0 {
        // Copied from NdrCStdStubBuffer_Release(), as Wine does.
        CStdStubBuffer_Disconnect(iface as *mut c_void);
        HeapFree(GetProcessHeap(), 0, stub as *mut c_void);
    }
    refcount
}

unsafe fn find_idispatch_stub_vtbl() -> *const CInterfaceStubVtbl {
    let mut vtbl = oleaut32_oaidl_ProxyFileInfo.pStubVtblList;
    while !(*vtbl).is_null() {
        if IsEqualGUID((**vtbl).header.piid, &IID_IDispatch) {
            return *vtbl;
        }
        vtbl = vtbl.add(1);
    }
    core::ptr::null()
}

unsafe fn dispinterface_create_stub(
    server: *mut IUnknown,
    iid: REFIID,
    stub: *mut *mut IRpcStubBuffer,
) -> HRESULT {
    let stub_vtbl = find_idispatch_stub_vtbl();
    if stub_vtbl.is_null() {
        // Wine asserts: the oaidl proxy file always carries IDispatch.
        ERR!(b"No IDispatch stub in the oaidl proxy file.\n\0".as_ptr() as PCHAR);
        return E_FAIL;
    }

    let object = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, core::mem::size_of::<dispinterface_stub>() as SIZE_T)
        as *mut dispinterface_stub;
    if object.is_null() {
        return E_OUTOFMEMORY;
    }

    let mut dispatch: *mut c_void = core::ptr::null_mut();
    let hr = IUnknown_QueryInterface(server, iid, &mut dispatch);
    if FAILED(hr) {
        ERR!(b"Object does not support the dispinterface iid.\n\0".as_ptr() as PCHAR);
        HeapFree(GetProcessHeap(), 0, object as *mut c_void);
        return hr;
    }

    let v = core::ptr::addr_of_mut!((*object).stub_vtbl);
    (*v).header = (*stub_vtbl).header;
    (*v).Vtbl = RpcStubBufferVtbl {
        QueryInterface: CStdStubBuffer_QueryInterface as *const c_void,
        AddRef: CStdStubBuffer_AddRef as *const c_void,
        Release: dispinterface_stub_Release,
        Connect: CStdStubBuffer_Connect as *const c_void,
        Disconnect: CStdStubBuffer_Disconnect as *const c_void,
        Invoke: CStdStubBuffer_Invoke as *const c_void,
        IsIIDSupported: CStdStubBuffer_IsIIDSupported as *const c_void,
        CountRefs: CStdStubBuffer_CountRefs as *const c_void,
        DebugServerQueryInterface: CStdStubBuffer_DebugServerQueryInterface as *const c_void,
        DebugServerRelease: CStdStubBuffer_DebugServerRelease as *const c_void,
    };
    (*object).stub_buffer.lpVtbl = core::ptr::addr_of!((*v).Vtbl);
    (*object).stub_buffer.RefCount = 1;
    (*object).stub_buffer.pvServerObject = dispatch as *mut IUnknown;
    // rpcrt4 would also fill pPSFactory, used only by its own Release (replaced above).

    *stub = core::ptr::addr_of_mut!((*object).stub_buffer) as *mut IRpcStubBuffer;
    S_OK
}

stdmethod! { fn dispatch_typelib_ps_CreateStub(
    iface: *mut IPSFactoryBuffer,
    iid: REFIID,
    server: *mut IUnknown,
    stub: *mut *mut IRpcStubBuffer,
) -> HRESULT {
    let mut typeinfo: *mut ITypeInfo = core::ptr::null_mut();
    let mut attr: *mut TYPEATTR = core::ptr::null_mut();
    let mut hr: HRESULT;

    if IsEqualGUID(iid, &IID_IDispatch) {
        return dispatch_create_stub(server, stub);
    }

    hr = get_typeinfo_for_iid(iid, &mut typeinfo);
    if FAILED(hr) {
        return hr;
    }

    hr = ITypeInfo_GetTypeAttr(typeinfo, &mut attr);
    if FAILED(hr) {
        ITypeInfo_Release(typeinfo);
        return hr;
    }

    if (*attr).typekind == TKIND_INTERFACE || ((*attr).wTypeFlags & TYPEFLAG_FDUAL as u16) != 0 {
        hr = CreateStubFromTypeInfo(typeinfo, iid, server, stub);
    } else {
        hr = dispinterface_create_stub(server, iid, stub);
    }

    if FAILED(hr) {
        ERR!(b"Failed to create stub, hr %#x.\n\0".as_ptr() as PCHAR, hr);
    }

    ITypeInfo_ReleaseTypeAttr(typeinfo, attr);
    ITypeInfo_Release(typeinfo);
    hr
} }

#[no_mangle]
pub unsafe extern "system" fn DllGetClassObject(rclsid: *const GUID, iid: *const GUID, ppv: *mut LPVOID) -> HRESULT {

    *ppv = core::ptr::null_mut();
    
    if IsEqualGUID(rclsid, &CLSID_StdFont) {
        if IsEqualGUID(iid, &IID_IClassFactory) {
            _get_STDFONT_CF(ppv);
            IClassFactory_AddRef(*ppv as *mut IClassFactory);
            return 0;
        }
    }

    if IsEqualGUID(rclsid, &CLSID_StdPicture) {
        if IsEqualGUID(iid, &IID_IClassFactory) {
            _get_STDPIC_CF(ppv);
            IClassFactory_AddRef(*ppv as *mut IClassFactory);
            return 0;
        }
    }

    if IsEqualGUID(rclsid, &CLSID_PSDispatch) || IsEqualGUID(rclsid, &CLSID_PSOAInterface) {
        return IPSFactoryBuffer_QueryInterface(core::ptr::addr_of_mut!(dispatch_typelib_ps), iid, ppv);
    }

    if IsEqualCLSID(rclsid, &CLSID_PSTypeComp) ||
        IsEqualCLSID(rclsid, &CLSID_PSTypeInfo) ||
        IsEqualCLSID(rclsid, &CLSID_PSTypeLib) ||
        IsEqualCLSID(rclsid, &CLSID_PSDispatch) ||
        IsEqualCLSID(rclsid, &CLSID_PSEnumVariant) {
        return OLEAUTPS_DllGetClassObject(&CLSID_PSFactoryBuffer, iid, ppv);
    }

    return OLEAUTPS_DllGetClassObject(rclsid, iid, ppv);
}

#[no_mangle]
pub unsafe extern "system" fn DllCanUnloadNow() -> HRESULT {
    S_FALSE
}

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInstDll: HINSTANCE, fdwReason: DWORD, lpvReserved: LPVOID) -> BOOL {
    static oanocacheW: [u16; 10] = [b'o' as u16, b'a' as u16, b'n' as u16, b'o' as u16, b'c' as u16, b'a' as u16, b'c' as u16, b'h' as u16, b'e' as u16, 0];

    if fdwReason == DLL_PROCESS_ATTACH {
        bstr_cache_enabled = (GetEnvironmentVariableW(oanocacheW.as_ptr(), core::ptr::null_mut(), 0) == 0) as BOOL;
    }

    OLEAUTPS_DllMain(hInstDll, fdwReason, lpvReserved)
}

#[no_mangle]
pub unsafe extern "system" fn DllRegisterServer() -> HRESULT {
    OLEAUTPS_DllRegisterServer()
}

#[no_mangle]
pub unsafe extern "system" fn DllUnregisterServer() -> HRESULT {
    OLEAUTPS_DllUnregisterServer()
}

#[no_mangle]
pub unsafe extern "system" fn OleIconToCursor(
    _hinstExe: HINSTANCE,
    hIcon: HICON,
) -> HCURSOR {
    
    FIXME!(
        b"(%p,%p), partially implemented.\n\0".as_ptr(),
        hinstExe,
        hIcon,
    );
    CopyCursor(hIcon as HCURSOR)
}

// The C arrays are `static const WCHAR *name[]` — arrays of raw pointers. A Rust
// `static` requires `Sync`, which raw pointers lack; this transparent newtype
// carries the same layout and asserts thread-safety (the tables are immutable
// after program load, matching the C `const`).
#[repr(transparent)]
struct AltMonthNames([*const u16; 13]);
unsafe impl Sync for AltMonthNames {}

#[no_mangle]
pub unsafe extern "system" fn GetAltMonthNames(lcid: LCID, str: *mut *mut LPOLESTR) -> HRESULT {
    static ar_month1W: [u16; 5] = [0x645,0x62d,0x631,0x645,0];
    static ar_month2W: [u16; 4] = [0x635,0x641,0x631,0];
    static ar_month3W: [u16; 11] = [0x631,0x628,0x64a,0x639,0x20,0x627,0x644,0x627,0x648,0x644,0];
    static ar_month4W: [u16; 12] = [0x631,0x628,0x64a,0x639,0x20,0x627,0x644,0x62b,0x627,0x646,0x64a,0];
    static ar_month5W: [u16; 13] = [0x62c,0x645,0x627,0x62f,0x649,0x20,0x627,0x644,0x627,0x648,0x644,0x649,0];
    static ar_month6W: [u16; 14] = [0x62c,0x645,0x627,0x62f,0x649,0x20,0x627,0x644,0x62b,0x627,0x646,0x64a,0x629,0];
    static ar_month7W: [u16; 4] = [0x631,0x62c,0x628,0];
    static ar_month8W: [u16; 6] = [0x634,0x639,0x628,0x627,0x646,0];
    static ar_month9W: [u16; 6] = [0x631,0x645,0x636,0x627,0x646,0];
    static ar_month10W: [u16; 5] = [0x634,0x648,0x627,0x643,0];
    static ar_month11W: [u16; 10] = [0x630,0x648,0x20,0x627,0x644,0x642,0x639,0x62f,0x629,0];
    static ar_month12W: [u16; 9] = [0x630,0x648,0x20,0x627,0x644,0x62d,0x62c,0x629,0];

    static arabic_hijri: AltMonthNames = AltMonthNames([
        ar_month1W.as_ptr(),
        ar_month2W.as_ptr(),
        ar_month3W.as_ptr(),
        ar_month4W.as_ptr(),
        ar_month5W.as_ptr(),
        ar_month6W.as_ptr(),
        ar_month7W.as_ptr(),
        ar_month8W.as_ptr(),
        ar_month9W.as_ptr(),
        ar_month10W.as_ptr(),
        ar_month11W.as_ptr(),
        ar_month12W.as_ptr(),
        core::ptr::null(),
    ]);

    static pl_month1W: [u16; 9] = [0x73,0x74,0x79,0x63,0x7a,0x6e,0x69,0x61,0];
    static pl_month2W: [u16; 7] = [0x6c,0x75,0x74,0x65,0x67,0x6f,0];
    static pl_month3W: [u16; 6] = [0x6d,0x61,0x72,0x63,0x61,0];
    static pl_month4W: [u16; 9] = [0x6b,0x77,0x69,0x65,0x74,0x6e,0x69,0x61,0];
    static pl_month5W: [u16; 5] = [0x6d,0x61,0x6a,0x61,0];
    static pl_month6W: [u16; 8] = [0x63,0x7a,0x65,0x72,0x77,0x63,0x61,0];
    static pl_month7W: [u16; 6] = [0x6c,0x69,0x70,0x63,0x61,0];
    static pl_month8W: [u16; 9] = [0x73,0x69,0x65,0x72,0x70,0x6e,0x69,0x61,0];
    static pl_month9W: [u16; 9] = [0x77,0x72,0x7a,0x65,0x15b,0x6e,0x69,0x61,0];
    static pl_month10W: [u16; 13] = [0x70,0x61,0x17a,0x64,0x7a,0x69,0x65,0x72,0x6e,0x69,0x6b,0x61,0];
    static pl_month11W: [u16; 10] = [0x6c,0x69,0x73,0x74,0x6f,0x70,0x61,0x64,0x61,0];
    static pl_month12W: [u16; 8] = [0x67,0x72,0x75,0x64,0x6e,0x69,0x61,0];

    static polish_genitive_names: AltMonthNames = AltMonthNames([
        pl_month1W.as_ptr(),
        pl_month2W.as_ptr(),
        pl_month3W.as_ptr(),
        pl_month4W.as_ptr(),
        pl_month5W.as_ptr(),
        pl_month6W.as_ptr(),
        pl_month7W.as_ptr(),
        pl_month8W.as_ptr(),
        pl_month9W.as_ptr(),
        pl_month10W.as_ptr(),
        pl_month11W.as_ptr(),
        pl_month12W.as_ptr(),
        core::ptr::null(),
    ]);

    static ru_month1W: [u16; 7] = [0x44f,0x43d,0x432,0x430,0x440,0x44f,0];
    static ru_month2W: [u16; 8] = [0x444,0x435,0x432,0x440,0x430,0x43b,0x44f,0];
    static ru_month3W: [u16; 6] = [0x43c,0x430,0x440,0x442,0x430,0];
    static ru_month4W: [u16; 7] = [0x430,0x43f,0x440,0x435,0x43b,0x44f,0];
    static ru_month5W: [u16; 4] = [0x43c,0x430,0x44f,0];
    static ru_month6W: [u16; 5] = [0x438,0x44e,0x43d,0x44f,0];
    static ru_month7W: [u16; 5] = [0x438,0x44e,0x43b,0x44f,0];
    static ru_month8W: [u16; 8] = [0x430,0x432,0x433,0x443,0x441,0x442,0x430,0];
    static ru_month9W: [u16; 9] = [0x441,0x435,0x43d,0x442,0x44f,0x431,0x440,0x44f,0];
    static ru_month10W: [u16; 8] = [0x43e,0x43a,0x442,0x44f,0x431,0x440,0x44f,0];
    static ru_month11W: [u16; 7] = [0x43d,0x43e,0x44f,0x431,0x440,0x44f,0];
    static ru_month12W: [u16; 8] = [0x434,0x435,0x43a,0x430,0x431,0x440,0x44f,0];

    static russian_genitive_names: AltMonthNames = AltMonthNames([
        ru_month1W.as_ptr(),
        ru_month2W.as_ptr(),
        ru_month3W.as_ptr(),
        ru_month4W.as_ptr(),
        ru_month5W.as_ptr(),
        ru_month6W.as_ptr(),
        ru_month7W.as_ptr(),
        ru_month8W.as_ptr(),
        ru_month9W.as_ptr(),
        ru_month10W.as_ptr(),
        ru_month11W.as_ptr(),
        ru_month12W.as_ptr(),
        core::ptr::null(),
    ]);

    TRACE!(b"GetAltMonthNames(%#x, %p)\n\0".as_ptr() as LPCSTR, lcid, str);

    let langid = LANGIDFROMLCID(lcid);
    let primary_lang = PRIMARYLANGID(langid);

    if primary_lang == LANG_ARABIC {
        *str = arabic_hijri.0.as_ptr() as *mut LPOLESTR;
    } else if primary_lang == LANG_POLISH {
        *str = polish_genitive_names.0.as_ptr() as *mut LPOLESTR;
    } else if primary_lang == LANG_RUSSIAN {
        *str = russian_genitive_names.0.as_ptr() as *mut LPOLESTR;
    } else {
        *str = core::ptr::null_mut();
    }

    0
}
