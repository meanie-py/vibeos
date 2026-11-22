use crate::crypt32::oid::crypt_oid_free;
use crate::crypt32::oid::crypt_oid_init;
use crate::crypt32::sip::crypt_sip_free;
use crate::crypt32::chain::default_chain_engine_free;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_3F2853D54547: [u16; 70] = [
    0x0053u16, 0x006F, 0x0066, 0x0074, 0x0077, 0x0061, 0x0072, 0x0065, 0x005C, 0x0050,
    0x006F, 0x006C, 0x0069, 0x0063, 0x0069, 0x0065, 0x0073, 0x005C, 0x004D, 0x0069,
    0x0063, 0x0072, 0x006F, 0x0073, 0x006F, 0x0066, 0x0074, 0x005C, 0x0053, 0x0079,
    0x0073, 0x0074, 0x0065, 0x006D, 0x0043, 0x0065, 0x0072, 0x0074, 0x0069, 0x0066,
    0x0069, 0x0063, 0x0061, 0x0074, 0x0065, 0x0073, 0x005C, 0x0054, 0x0072, 0x0075,
    0x0073, 0x0074, 0x0065, 0x0064, 0x0050, 0x0075, 0x0062, 0x006C, 0x0069, 0x0073,
    0x0068, 0x0065, 0x0072, 0x005C, 0x0053, 0x0061, 0x0066, 0x0065, 0x0072, 0x0000,
];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[derive(Copy, Clone)]
#[repr(C)]
pub struct C2RustUnnamed_11 {
    pub Flags: DWORD,
    pub CreatorBackTraceIndexHigh: WORD,
    pub SpareWORD: WORD,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union C2RustUnnamed_10 {
    pub WineDebugString: DWORD_PTR,
    pub Spare: [DWORD_PTR; 1],
    pub c2rust_unnamed: C2RustUnnamed_11,
}

pub static MS_ENH_RSA_AES_PROV_W: [::core::ffi::c_ushort; 54] = unsafe {
    ::core::mem::transmute::<
        [u8; 108],
        [::core::ffi::c_ushort; 54],
    >(
        *b"M\0i\0c\0r\0o\0s\0o\0f\0t\0 \0E\0n\0h\0a\0n\0c\0e\0d\0 \0R\0S\0A\0 \0a\0n\0d\0 \0A\0E\0S\0 \0C\0r\0y\0p\0t\0o\0g\0r\0a\0p\0h\0i\0c\0 \0P\0r\0o\0v\0i\0d\0e\0r\0\0\0",
    )
};

static mut hDefProv: HCRYPTPROV = 0;

#[no_mangle]
pub static mut hInstance: HINSTANCE = ::core::ptr::null::<HINSTANCE__>() as *mut HINSTANCE__;

static mut prov_param_cs_debug: CRITICAL_SECTION_DEBUG = unsafe { _RTL_CRITICAL_SECTION_DEBUG {
        Type: 0 as WORD,
        CreatorBackTraceIndex: 0 as WORD,
        CriticalSection: &raw mut prov_param_cs,
        ProcessLocksList: _LIST_ENTRY {
            Flink: &raw mut prov_param_cs_debug.ProcessLocksList,
            Blink: &raw mut prov_param_cs_debug.ProcessLocksList,
        },
        EntryCount: 0 as DWORD,
        ContentionCount: 0 as DWORD,
        __bindgen_anon_1: _RTL_CRITICAL_SECTION_DEBUG__bindgen_ty_1 { Spare: [0; 1] },
    }};

static mut prov_param_cs: CRITICAL_SECTION = unsafe {
    _RTL_CRITICAL_SECTION {
        DebugInfo: &raw const prov_param_cs_debug as PRTL_CRITICAL_SECTION_DEBUG,
        LockCount: -(1 as ::core::ffi::c_int) as LONG,
        RecursionCount: 0 as LONG,
        OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        SpinCount: 0 as ULONG_PTR,
    }
};

// WINE_DEFAULT_DEBUG_CHANNEL(crypt): debug-channel macro; the expansion is not
// reproduced here (debug channel registration is not needed for the translation).

// C: { 0, 0, &prov_param_cs, { &ProcessLocksList, &ProcessLocksList }, 0, 0,
//      { (DWORD_PTR)(__FILE__ ": prov_param_cs") } }
// The last element is the address of a string literal cast to an integer; a
// pointer-to-integer cast is not const-evaluable in a static initializer, so it
// is written as 0 here.

// C: { &prov_param_cs_debug, -1, 0, 0, 0, 0 }

pub type HLRUCACHE = *mut core::ffi::c_void;

#[no_mangle]
pub unsafe extern "system" fn DllMain(hInst: HINSTANCE, reason: DWORD, pvReserved: PVOID) -> BOOL {
    match reason {
        DLL_PROCESS_ATTACH => {
            hInstance = hInst;
            DisableThreadLibraryCalls(hInst);
            init_empty_store();
            crypt_oid_init();
            if __wine_init_unix_call!() != 0 {
                return (FALSE) as BOOL;
            }
            __reactos_call_unix_process_attach(core::ptr::null_mut::<core::ffi::c_void>());
        }
        DLL_PROCESS_DETACH => {
            if pvReserved.is_null() {
                crypt_oid_free();
                crypt_sip_free();
                default_chain_engine_free();
                if hDefProv != 0 {
                    CryptReleaseContext(hDefProv, 0);
                }
                __reactos_call_unix_process_detach(core::ptr::null_mut::<core::ffi::c_void>());
            }
        }
        _ => {}
    }
    return (TRUE) as BOOL;
}

unsafe extern "C" fn CRYPT_GetDefaultProvider() -> HCRYPTPROV {
    if hDefProv == 0 {
        let mut prov: HCRYPTPROV = 0;
        if CryptAcquireContextW(
            &raw mut prov,
            core::ptr::null::<WCHAR>(),
            MS_ENH_RSA_AES_PROV_W.as_ptr() as LPCWSTR,
            PROV_RSA_AES as DWORD,
            CRYPT_VERIFYCONTEXT as DWORD,
        ) == 0
        {
            return hDefProv;
        }
        InterlockedCompareExchangePointer(
            &raw mut hDefProv as *mut PVOID as *mut *mut ::core::ffi::c_void,
            prov as *mut ::core::ffi::c_void,
            core::ptr::null_mut(),
        );
        if hDefProv != prov {
            CryptReleaseContext(prov, 0 as DWORD);
        }
    }
    return hDefProv;
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptCreateLruCache(unknown: *mut ::core::ffi::c_void, out: *mut HLRUCACHE) -> BOOL {
    *out = 0xbaadf00d as *mut ::core::ffi::c_void as HLRUCACHE;
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptFindLruEntry(unk0: DWORD, unk1: DWORD) -> BOOL {
    // FIXME("(%08lx, %08lx): stub!\n", unk0, unk1);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptFindLruEntryData(
    unk0: DWORD,
    unk1: DWORD,
    unk2: DWORD,
) -> BOOL {
    // FIXME("(%08lx, %08lx, %08lx): stub!\n", unk0, unk1, unk2);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptCreateLruEntry(
    h: HLRUCACHE,
    unk0: DWORD,
    unk1: DWORD,
) -> BOOL {
    // FIXME("(%p, %08lx, %08lx): stub!\n", h, unk0, unk1);
    (FALSE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptFlushLruCache(
    h: HLRUCACHE,
    unk0: DWORD,
    unk1: DWORD,
) -> DWORD {
    // FIXME("(%p, %08lx, %08lx): stub!\n", h, unk0, unk1);
    0
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptFreeLruCache(
    h: HLRUCACHE,
    unk0: DWORD,
    unk1: DWORD,
) -> HLRUCACHE {
    // FIXME("(%p, %08lx, %08lx): stub!\n", h, unk0, unk1);
    h
}

#[no_mangle]
pub unsafe extern "system" fn CryptMemAlloc(cbSize: ULONG) -> LPVOID {
    malloc((cbSize) as _)
}

#[no_mangle]
pub unsafe extern "system" fn CryptMemRealloc(pv: LPVOID, cbSize: ULONG) -> LPVOID {
    realloc(pv, (cbSize) as _)
}

#[no_mangle]
pub unsafe extern "system" fn CryptMemFree(pv: LPVOID) {
    free(pv);
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptAllocTls() -> DWORD {
    TlsAlloc()
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptDetachTls(dwTlsIndex: DWORD) -> LPVOID {
    let mut ret: LPVOID;

    ret = TlsGetValue(dwTlsIndex);
    TlsSetValue(dwTlsIndex, core::ptr::null_mut());
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptGetTls(dwTlsIndex: DWORD) -> LPVOID {
    TlsGetValue(dwTlsIndex)
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptSetTls(dwTlsIndex: DWORD, lpTlsValue: LPVOID) -> BOOL {
    TlsSetValue(dwTlsIndex, lpTlsValue)
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptFreeTls(dwTlsIndex: DWORD, unknown: DWORD) -> BOOL {
    let mut ret: BOOL;
    ret = TlsFree(dwTlsIndex);
    if ret == 0 {
        SetLastError((E_INVALIDARG) as DWORD);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptGetOssGlobal(x: DWORD) -> BOOL {
    // FIXME("%08lx\n", x);
    (FALSE as i32)
}

unsafe extern "C" fn is_supported_algid(mut prov: HCRYPTPROV, mut algid: ALG_ID) -> BOOL {
    let mut prov_algs: PROV_ENUMALGS = _PROV_ENUMALGS {
        aiAlgid: 0,
        dwBitLen: 0,
        dwNameLen: 0,
        szName: [0; 20],
    };
    let mut size: DWORD = ::core::mem::size_of::<PROV_ENUMALGS>() as DWORD;
    let mut ret: BOOL = (FALSE as i32);
    /* This enumeration is not thread safe */
    EnterCriticalSection(core::ptr::addr_of_mut!(prov_param_cs));
    if CryptGetProvParam(
        prov,
        PP_ENUMALGS as DWORD,
        core::ptr::addr_of_mut!(prov_algs) as *mut BYTE,
        core::ptr::addr_of_mut!(size),
        CRYPT_FIRST as DWORD,
    ) != 0
    {
        loop {
            if prov_algs.aiAlgid == algid {
                ret = TRUE as BOOL;
                break;
            }
            if !(CryptGetProvParam(
                prov,
                PP_ENUMALGS as DWORD,
                core::ptr::addr_of_mut!(prov_algs) as *mut BYTE,
                core::ptr::addr_of_mut!(size),
                CRYPT_NEXT as DWORD,
            ) != 0)
            {
                break;
            }
        }
    }
    LeaveCriticalSection(core::ptr::addr_of_mut!(prov_param_cs));
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptGetDefaultCryptProv(algid: ALG_ID) -> HCRYPTPROV {
    let mut prov: HCRYPTPROV = 0;
    let mut defprov: HCRYPTPROV = 0;
    TRACE!("(%08x)\n", algid);

    defprov = CRYPT_GetDefaultProvider();

    if algid != 0 && is_supported_algid(defprov, algid) == 0 {
        let mut i: DWORD = 0;
        let mut type_: DWORD = 0;
        let mut size: DWORD = 0;

        while CryptEnumProvidersW(
            i,
            core::ptr::null_mut(),
            0,
            core::ptr::addr_of_mut!(type_),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(size),
        ) != 0
        {
            let mut name: *mut WCHAR = CryptMemAlloc(size as ULONG) as *mut WCHAR;
            if !name.is_null() {
                if CryptEnumProvidersW(
                    i,
                    core::ptr::null_mut(),
                    0,
                    core::ptr::addr_of_mut!(type_),
                    name as LPWSTR,
                    core::ptr::addr_of_mut!(size),
                ) != 0
                {
                    if CryptAcquireContextW(
                        core::ptr::addr_of_mut!(prov),
                        core::ptr::null::<WCHAR>(),
                        name as LPCWSTR,
                        type_,
                        CRYPT_VERIFYCONTEXT,
                    ) != 0
                    {
                        if is_supported_algid(prov, algid) != 0 {
                            CryptMemFree(name as LPVOID);
                            return prov;
                        }
                        CryptReleaseContext(prov, 0);
                    }
                }
                CryptMemFree(name as LPVOID);
            }
            i = i.wrapping_add(1);
        }

        SetLastError((E_INVALIDARG) as DWORD);
        return 0;
    }

    CryptContextAddRef(defprov, core::ptr::null_mut(), 0);
    return defprov;
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptReadTrustedPublisherDWORDValueFromRegistry(name: LPCWSTR, value: *mut DWORD) -> BOOL {
    let mut key: HKEY = core::ptr::null_mut();
    let mut rc: LONG;
    let mut ret: BOOL = (FALSE as i32);

    *value = 0;
    rc = RegCreateKeyW(HKEY_LOCAL_MACHINE, RUSTCONV_WSTR_3F2853D54547.as_ptr(), core::ptr::addr_of_mut!(key)) as LONG;
    if rc == (ERROR_SUCCESS as LONG) {
        let mut size: DWORD = core::mem::size_of::<DWORD>() as DWORD;

        if RegQueryValueExW(key, name, core::ptr::null_mut(), core::ptr::null_mut(), value as LPBYTE, core::ptr::addr_of_mut!(size)) == 0 {
            ret = (TRUE as i32);
        }
        RegCloseKey(key);
    }
    return ret;
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptInstallOssGlobal(mut x: DWORD, mut y: DWORD, mut z: DWORD) -> DWORD {
    static mut ret: i32 = 8;
    ret += 1;
    WINE_FIXME!("%08lx %08lx %08lx, return value %d\n", x, y, z, ret);
    return ret as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptInstallAsn1Module(
    x: ASN1module_t,
    y: DWORD,
    z: *mut core::ffi::c_void,
) -> BOOL {
    // FIXME("(%p %08lx %p): stub\n", x, y, z);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptUninstallAsn1Module(x: HCRYPTASN1MODULE) -> BOOL {
    // FIXME("(%08lx): stub\n", x);
    (TRUE as i32)
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptGetAsn1Decoder(x: HCRYPTASN1MODULE) -> ASN1decoding_t {
    // FIXME("(%08lx): stub\n", x);
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn I_CryptGetAsn1Encoder(x: HCRYPTASN1MODULE) -> ASN1encoding_t {
    // FIXME("(%08lx): stub\n", x);
    NULL
}

#[no_mangle]
pub unsafe extern "system" fn CryptProtectMemory(data: *mut core::ffi::c_void, len: DWORD, flags: DWORD) -> BOOL {
    static mut fixme_once: core::ffi::c_int = 0;
    let __old = fixme_once;
    fixme_once = fixme_once + 1;
    if __old == 0 {
        FIXME!("(%p %lu %08lx): stub\n", data, len, flags);
    }
    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn CryptUnprotectMemory(data: *mut core::ffi::c_void, len: DWORD, flags: DWORD) -> BOOL {
    static mut fixme_once: i32 = 0;
    let fresh = fixme_once;
    fixme_once += 1;
    if fresh == 0 {
        FIXME!("(%p %lu %08lx): stub\n", data, len, flags);
    }
    return (TRUE) as BOOL;
}
