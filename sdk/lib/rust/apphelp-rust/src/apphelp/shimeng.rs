use crate::apphelp::sdbread::SdbFindFirstTag;
use crate::apphelp::sdbread::SdbFindNextTag;
use crate::apphelp::hsdb::SdbGetAppPatchDir;
use crate::apphelp::sdbread::SdbGetStringTagPtr;
use crate::apphelp::hsdb::SdbInitDatabase;
use crate::apphelp::sdbread::SdbReadDWORDTag;
use crate::apphelp::sdbread::SdbReadQWORDTag;
use crate::apphelp::hsdb::SdbReleaseDatabase;
use crate::apphelp::hsdb::SdbTagIDToTagRef;
use crate::apphelp::hsdb::SdbTagRefToTagID;
use crate::apphelp::hsdb::SdbUnpackAppCompatData;
use crate::apphelp::sdbapi::SdbpAlloc;
use crate::apphelp::sdbapi::SdbpStrDup;
use crate::apphelp::sdbapi::SdbpStrsize;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_4477EECC4237: [u16; 11] = [
    0x0077u16, 0x0069, 0x006E, 0x0064, 0x0062, 0x0067, 0x002E, 0x0065, 0x0078, 0x0065,
    0x0000,
];
pub static RUSTCONV_WSTR_80073828D8A0: [u16; 10] = [0x006Eu16, 0x0074, 0x0064, 0x006C, 0x006C, 0x002E, 0x0064, 0x006C, 0x006C, 0x0000];
pub static RUSTCONV_WSTR_8B04A92EBFBE: [u16; 13] = [
    0x0076u16, 0x0065, 0x0072, 0x0069, 0x0066, 0x0069, 0x0065, 0x0072, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_8D71B3FAAB82: [u16; 2] = [0x0020u16, 0x0000];
pub static RUSTCONV_WSTR_95F9B480D535: [u16; 10] = [0x0073u16, 0x006C, 0x0073, 0x0076, 0x0063, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_990385C876A2: [u16; 13] = [
    0x006Bu16, 0x0065, 0x0072, 0x006E, 0x0065, 0x006C, 0x0033, 0x0032, 0x002E, 0x0064,
    0x006C, 0x006C, 0x0000,
];
pub static RUSTCONV_WSTR_9A6EFE4D86A5: [u16; 3] = [0x005Cu16, 0x002F, 0x0000];
pub static RUSTCONV_WSTR_A1F835C798F8: [u16; 9] = [0x006Eu16, 0x0074, 0x0073, 0x0064, 0x002E, 0x0065, 0x0078, 0x0065, 0x0000];
pub static RUSTCONV_WSTR_ABF250C9AB1D: [u16; 15] = [
    0x005Fu16, 0x005F, 0x0043, 0x004F, 0x004D, 0x0050, 0x0041, 0x0054, 0x005F, 0x004C,
    0x0041, 0x0059, 0x0045, 0x0052, 0x0000,
];
pub static RUSTCONV_WSTR_B4AFD365CBFE: [u16; 20] = [
    0x0053u16, 0x0048, 0x0049, 0x004D, 0x0045, 0x004E, 0x0047, 0x005F, 0x0044, 0x0045,
    0x0042, 0x0055, 0x0047, 0x005F, 0x004C, 0x0045, 0x0056, 0x0045, 0x004C, 0x0000,
];
pub static RUSTCONV_WSTR_B7FC02907B1F: [u16; 2] = [0x005Cu16, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub static mut Ntdll: _UNICODE_STRING = _UNICODE_STRING {
    Length: {
        let mut n = 0usize;
        while unsafe { *RUSTCONV_WSTR_80073828D8A0.as_ptr().add(n) } != 0 {
            n += 1;
        }
        (n * 2) as u16
    },
    MaximumLength: {
        let mut n = 0usize;
        while unsafe { *RUSTCONV_WSTR_80073828D8A0.as_ptr().add(n) } != 0 {
            n += 1;
        }
        ((n + 1) * 2) as u16
    },
    Buffer: RUSTCONV_WSTR_80073828D8A0.as_ptr() as *mut u16,
};

pub static mut Kernel32: _UNICODE_STRING = _UNICODE_STRING {
    Length: {
        let mut n = 0usize;
        while unsafe { *RUSTCONV_WSTR_990385C876A2.as_ptr().add(n) } != 0 {
            n += 1;
        }
        (n * 2) as u16
    },
    MaximumLength: {
        let mut n = 0usize;
        while unsafe { *RUSTCONV_WSTR_990385C876A2.as_ptr().add(n) } != 0 {
            n += 1;
        }
        ((n + 1) * 2) as u16
    },
    Buffer: RUSTCONV_WSTR_990385C876A2.as_ptr() as *mut u16,
};

pub static mut Verifier: _UNICODE_STRING = _UNICODE_STRING {
    Length: {
        let mut n = 0usize;
        while unsafe { *RUSTCONV_WSTR_8B04A92EBFBE.as_ptr().add(n) } != 0 {
            n += 1;
        }
        (n * 2) as u16
    },
    MaximumLength: {
        let mut n = 0usize;
        while unsafe { *RUSTCONV_WSTR_8B04A92EBFBE.as_ptr().add(n) } != 0 {
            n += 1;
        }
        ((n + 1) * 2) as u16
    },
    Buffer: RUSTCONV_WSTR_8B04A92EBFBE.as_ptr() as *mut u16,
};

pub static mut g_hInstance: HMODULE = core::ptr::null_mut();

pub static mut g_WindowsDirectory: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};

pub static mut g_System32Directory: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};

pub static mut g_SxsDirectory: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};

pub static mut g_LoadingShimDll: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};

pub static mut g_ShimEngDebugLevel: ULONG = 0xffffffff;
pub static mut g_bComPlusImage: BOOL = 0;
pub static mut g_bShimDuringInit: BOOL = 0;
pub static mut g_bShimEngInitialized: BOOL = 0;
pub static mut g_bInternalHooksUsed: BOOL = 0;

pub static mut g_pShimInfo: ARRAY = unsafe { core::mem::zeroed() };
pub static mut g_pHookArray: ARRAY = unsafe { core::mem::zeroed() };
pub static mut g_InExclude: ARRAY = unsafe { core::mem::zeroed() };

pub type GETPROCADDRESSPROC = unsafe extern "system" fn(HINSTANCE, LPCSTR) -> FARPROC;

pub static mut g_IntHookEx: [HOOKAPIEX; 1] = [
    HOOKAPIEX {
        LibraryName: b"kernel32.dll\0".as_ptr() as *const i8,
        FunctionName: b"GetProcAddress\0".as_ptr() as *const i8,
        ReplacementFunction: StubGetProcAddress as _,
        OriginalFunction: core::ptr::null_mut(),
        pShimInfo: core::ptr::null_mut(),
        ApiLink: core::ptr::null_mut(),
    },
];

pub const MAX_LAYER_LENGTH: u32 = 256;
unsafe extern "C" fn ARRAY_InitWorker(Array: PARRAY, ItemSize: usize) -> BOOL {
    (*Array).Data__ = core::ptr::null_mut();
    let v = 0;
    (*Array).Size__ = v;
    (*Array).MaxSize__ = v;
    (*Array).ItemSize__ = ItemSize as DWORD; // sizeof() is size_t in C, stored in a DWORD

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ARRAY_EnsureSize(Array: PARRAY, ItemSize: DWORD, GrowWith: DWORD) -> BOOL {
    let mut pNewData: PVOID;
    let mut Count: DWORD;

    if (*Array).MaxSize__ > (*Array).Size__ {
        return (TRUE) as BOOL;
    }

    Count = (*Array).Size__.wrapping_add(GrowWith);
    pNewData = SeiAlloc!(Count.wrapping_mul(ItemSize));

    if pNewData.is_null() {
        SHIMENG_FAIL!(b"Failed to allocate %d bytes\n\0".as_ptr() as *const i8, Count.wrapping_mul(ItemSize));
        return (FALSE) as BOOL;
    }
    (*Array).MaxSize__ = Count;

    if !(*Array).Data__.is_null() {
        memcpy(pNewData, (*Array).Data__, ((*Array).Size__.wrapping_mul(ItemSize)) as _);
        SeiFree!((*Array).Data__);
    }
    (*Array).Data__ = pNewData;

    return (TRUE) as BOOL;
}

unsafe extern "C" fn ARRAY_AppendWorker(Array: PARRAY, ItemSize: usize, GrowWith: DWORD) -> PVOID {
    let ItemSize = ItemSize as DWORD;
    let mut pData: PBYTE;

    if ARRAY_EnsureSize(Array, ItemSize, GrowWith) == 0 {
        return core::ptr::null_mut();
    }

    pData = ((*Array).Data__).cast();
    pData = pData.add((*Array).Size__.wrapping_mul(ItemSize) as usize);
    (*Array).Size__ = (*Array).Size__.wrapping_add(1);

    return pData as PVOID;
}

unsafe extern "C" fn ARRAY_AtWorker(Array: PARRAY, ItemSize: usize, n: DWORD) -> PVOID {
    let ItemSize = ItemSize as DWORD;
    let mut pData: PBYTE;

    pData = (*Array).Data__ as PBYTE;
    return pData.add(n.wrapping_mul(ItemSize) as usize) as PVOID;
}
#[no_mangle]
pub unsafe extern "C" fn SeiInitDebugSupport() {
    static mut DebugKey: _UNICODE_STRING = RTL_CONSTANT_STRING_STATIC!(RUSTCONV_WSTR_B4AFD365CBFE);
    let mut DebugValue: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut NewLevel: ULONG = SEI_MSG; /* Show some basic info in the logs, unless configured different */
    let mut Buffer: [u16; 40] = [0u16; 40];

    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(DebugValue),
        core::ptr::addr_of_mut!(Buffer) as PWSTR,
        (core::mem::size_of::<[u16; 40]>() as ULONG) as USHORT,
    );

    Status = RtlQueryEnvironmentVariable_U(
        core::ptr::null_mut(),
        core::ptr::addr_of!(DebugKey),
        core::ptr::addr_of_mut!(DebugValue),
    );

    if NT_SUCCESS!(Status) {
        if !NT_SUCCESS!(RtlUnicodeStringToInteger(
            core::ptr::addr_of_mut!(DebugValue),
            10,
            core::ptr::addr_of_mut!(NewLevel),
        )) {
            NewLevel = 0;
        }
    }
    g_ShimEngDebugLevel = NewLevel;
}

#[no_mangle]
pub unsafe extern "C" fn SeiDbgPrint_v(Level: SEI_LOG_LEVEL, Function: PCSTR, Format: PCSTR, ap_list: *mut ::core::ffi::c_void) -> BOOL {
    let mut Buffer: [i8; 512] = [0; 512];
    let mut Current: *mut i8 = Buffer.as_mut_ptr();
    let LevelStr: PCSTR;
    let mut Length: usize = core::mem::size_of::<[i8; 512]>();
    // The C trampoline passes `&ap`; a va_list is a `char*` on every Windows target we build.
    let ArgList: *mut i8 = *(ap_list as *mut *mut i8);
    let mut hr: HRESULT;

    if g_ShimEngDebugLevel == 0xffffffff {
        SeiInitDebugSupport();
    }

    if Level > g_ShimEngDebugLevel {
        return (FALSE) as BOOL;
    }

    match Level {
        SEI_MSG => {
            LevelStr = b"MSG \0".as_ptr() as PCSTR;
        }
        SEI_FAIL => {
            LevelStr = b"FAIL\0".as_ptr() as PCSTR;
        }
        SEI_WARN => {
            LevelStr = b"WARN\0".as_ptr() as PCSTR;
        }
        SEI_INFO => {
            LevelStr = b"INFO\0".as_ptr() as PCSTR;
        }
        _ => {
            LevelStr = b"USER\0".as_ptr() as PCSTR;
        }
    }

    if !Function.is_null() {
        hr = StringCchPrintfExA!(Current, Length, core::ptr::addr_of_mut!(Current), core::ptr::addr_of_mut!(Length), STRSAFE_NULL_ON_FAILURE, b"[%s] [%s] \0".as_ptr() as PCSTR, LevelStr, Function);
    } else {
        hr = StringCchPrintfExA!(Current, Length, core::ptr::addr_of_mut!(Current), core::ptr::addr_of_mut!(Length), STRSAFE_NULL_ON_FAILURE, b"[%s] \0".as_ptr() as PCSTR, LevelStr);
    }

    if !((hr) >= 0) {
        return (FALSE) as BOOL;
    }

    hr = StringCchVPrintfExA!(Current, Length, core::ptr::addr_of_mut!(Current), core::ptr::addr_of_mut!(Length), STRSAFE_NULL_ON_FAILURE, Format, ArgList);
    if !((hr) >= 0) {
        return (FALSE) as BOOL;
    }

    DbgPrint(b"%s\0".as_ptr() as PCSTR, Buffer.as_ptr());
    return (TRUE) as BOOL;
}

extern "C" {
    // The C-variadic entry is a trampoline in rust_varargs_shim.c (stable Rust
    // cannot define one); it `va_start`s and calls `SeiDbgPrint_v` above.
    pub fn SeiDbgPrint(Level: SEI_LOG_LEVEL, Function: PCSTR, Format: PCSTR, ...) -> BOOL;
}


unsafe extern "C" fn SeiIsOrdinalName(lpProcName: LPCSTR) -> BOOL {
    return ((lpProcName as ULONG_PTR) <= (MAXUSHORT as ULONG_PTR)) as BOOL;
}
#[no_mangle]
pub unsafe extern "C" fn SeiPrintFunctionName(lpProcName: LPCSTR, szOrdProcFmt: *mut i8) -> LPCSTR {
    if SeiIsOrdinalName(lpProcName) != 0 {
        StringCchPrintfA!(szOrdProcFmt, 10, b"#%Iu\0".as_ptr() as *const i8, lpProcName as usize);
        return szOrdProcFmt as LPCSTR;
    }
    return lpProcName;
}
#[no_mangle]
pub unsafe extern "C" fn SeiCompareFunctionName(lpProcName1: LPCSTR, lpProcName2: LPCSTR) -> i32 {
    let Ord1: BOOL = SeiIsOrdinalName(lpProcName1);
    let Ord2: BOOL = SeiIsOrdinalName(lpProcName2);

    /* One is an ordinal, the other not */
    if Ord1 != Ord2 {
        return 1;
    }

    /* Compare ordinals */
    if Ord1 != 0 {
        return ((lpProcName1 as usize) != (lpProcName2 as usize)) as i32;
    }

    /* Compare names */
    return strcmp(lpProcName1, lpProcName2);
}
#[no_mangle]
pub unsafe extern "C" fn SeiGetModuleFromAddress(addr: PVOID) -> PVOID {
    let mut hModule: PVOID = core::ptr::null_mut();
    RtlPcToFileHeader(addr, core::ptr::addr_of_mut!(hModule));
    return hModule;
}
#[no_mangle]
pub unsafe extern "C" fn NotifyShims(dwReason: DWORD, Info: PVOID) {
    let mut n: DWORD = 0;
    while n < ARRAY_Size!(core::ptr::addr_of_mut!(g_pShimInfo)) {
        let pShimModule: PSHIMMODULE = *ARRAY_At!(core::ptr::addr_of_mut!(g_pShimInfo), PSHIMMODULE, n);
        if (*pShimModule).pNotifyShims.is_none() {
            n += 1;
            continue;
        }

        (((*pShimModule).pNotifyShims).unwrap_unchecked())(dwReason, Info);
        n += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiCheckComPlusImage(BaseAddress: PVOID) {
    let mut ComSectionSize: ULONG = 0;
    g_bComPlusImage = (RtlImageDirectoryEntryToData(BaseAddress, (TRUE) as BOOLEAN, (IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR) as USHORT, core::ptr::addr_of_mut!(ComSectionSize)) != core::ptr::null_mut()) as BOOL;

    SHIMENG_INFO!(
        b"COM+ executable %s\n\0".as_ptr() as *const i8,
        if g_bComPlusImage != 0 {
            b"TRUE\0".as_ptr() as *const i8
        } else {
            b"FALSE\0".as_ptr() as *const i8
        },
    );
}
#[no_mangle]
pub unsafe extern "C" fn SeiGetShimModuleInfo(BaseAddress: PVOID) -> PSHIMMODULE {
    let mut n: DWORD;

    n = 0;
    while n < ARRAY_Size!(core::ptr::addr_of_mut!(g_pShimInfo)) {
        let pShimModule: PSHIMMODULE = *ARRAY_At!(core::ptr::addr_of_mut!(g_pShimInfo), PSHIMMODULE, n);

        if (*pShimModule).BaseAddress == BaseAddress {
            return pShimModule;
        }
        n += 1;
    }
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "C" fn SeiCreateShimModuleInfo(DllName: PCWSTR, BaseAddress: *mut core::ffi::c_void) -> PSHIMMODULE {
    static mut GetHookAPIs: _STRING = RTL_CONSTANT_STRING_A!(b"GetHookAPIs\0");
    static mut NotifyShims: _STRING = RTL_CONSTANT_STRING_A!(b"NotifyShims\0");
    let mut pData: *mut PSHIMMODULE;
    let mut Data: PSHIMMODULE;
    let mut pGetHookAPIs: Option<unsafe extern "system" fn(*const i8, *const u16, *mut u32) -> *mut tagHOOKAPIEX> = None;
    let mut pNotifyShims: Option<unsafe extern "system" fn(u32, *mut core::ffi::c_void) -> i32> = None;

    if !NT_SUCCESS!(LdrGetProcedureAddress(BaseAddress, core::ptr::addr_of!(GetHookAPIs) as PANSI_STRING, 0, core::ptr::addr_of_mut!(pGetHookAPIs) as *mut *mut core::ffi::c_void))
        || !NT_SUCCESS!(LdrGetProcedureAddress(BaseAddress, core::ptr::addr_of!(NotifyShims) as PANSI_STRING, 0, core::ptr::addr_of_mut!(pNotifyShims) as *mut *mut core::ffi::c_void))
    {
        SHIMENG_WARN!(b"Failed to resolve entry points for %S\n\0".as_ptr() as *const i8, DllName);
        return core::ptr::null_mut();
    }

    pData = ARRAY_Append!(core::ptr::addr_of_mut!(g_pShimInfo), PSHIMMODULE) as *mut PSHIMMODULE;
    if pData.is_null() {
        return core::ptr::null_mut();
    }

    *pData = SeiAlloc!(core::mem::size_of::<SHIMMODULE>()) as PSHIMMODULE;

    Data = *pData;

    RtlCreateUnicodeString(core::ptr::addr_of_mut!((*Data).Name), DllName);
    (*Data).BaseAddress = BaseAddress;

    (*Data).pGetHookAPIs = pGetHookAPIs;
    (*Data).pNotifyShims = pNotifyShims;

    ARRAY_Init!(core::ptr::addr_of_mut!((*Data).EnabledShims), PSHIMINFO);

    return Data;
}
#[no_mangle]
pub unsafe extern "C" fn SeiAppendHookInfo(pShimModuleInfo: PSHIMMODULE, pHookApi: PHOOKAPIEX, dwHookCount: DWORD, ShimName: PCWSTR) -> PSHIMINFO {
    let mut pData: *mut PSHIMINFO = core::ptr::null_mut();
    let mut Data: PSHIMINFO = core::ptr::null_mut();

    pData = ARRAY_Append!(core::ptr::addr_of_mut!((*pShimModuleInfo).EnabledShims), PSHIMINFO) as *mut PSHIMINFO;
    if pData.is_null() {
        return core::ptr::null_mut();
    }

    *pData = SeiAlloc!(core::mem::size_of::<SHIMINFO>()) as PSHIMINFO;
    Data = *pData;

    if Data.is_null() {
        return core::ptr::null_mut();
    }

    (*Data).ShimName = SdbpStrDup(ShimName);
    if (*Data).ShimName.is_null() {
        return core::ptr::null_mut();
    }

    (*Data).pHookApi = pHookApi;
    (*Data).dwHookCount = dwHookCount;
    (*Data).pShimModule = pShimModuleInfo;
    ARRAY_Init!(core::ptr::addr_of_mut!((*Data).InExclude), INEXCLUDE);
    return Data;
}
#[no_mangle]
pub unsafe extern "C" fn SeiFindHookModuleInfo(ModuleName: PUNICODE_STRING, mut BaseAddress: PVOID) -> PHOOKMODULEINFO {
    let mut n: DWORD;

    if ModuleName.is_null() && BaseAddress.is_null() {
        BaseAddress = (*NtCurrentPeb()).ImageBaseAddress;
    }

    n = 0;
    while n < ARRAY_Size!(core::ptr::addr_of_mut!(g_pHookArray)) {
        let pModuleInfo: PHOOKMODULEINFO = ARRAY_At!(core::ptr::addr_of_mut!(g_pHookArray), HOOKMODULEINFO, n) as PHOOKMODULEINFO;

        if !BaseAddress.is_null() && BaseAddress == (*pModuleInfo).BaseAddress {
            return pModuleInfo;
        }

        if BaseAddress.is_null() && RtlEqualUnicodeString(ModuleName, core::ptr::addr_of_mut!((*pModuleInfo).Name), (TRUE) as BOOLEAN) != 0 {
            return pModuleInfo;
        }

        n += 1;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn SeiFindHookModuleInfoForImportDescriptor(DllBase: PBYTE, ImportDescriptor: PIMAGE_IMPORT_DESCRIPTOR) -> PHOOKMODULEINFO {
    let mut DllName: _UNICODE_STRING = core::mem::zeroed();
    let mut DllHandle: PVOID = core::ptr::null_mut();
    let mut Success: NTSTATUS;

    if RtlCreateUnicodeStringFromAsciiz(core::ptr::addr_of_mut!(DllName), DllBase.add((*ImportDescriptor).Name as usize) as PCSZ) == 0 {
        SHIMENG_FAIL!(b"Unable to convert dll name to unicode\n\0".as_ptr() as *const i8);
        return core::ptr::null_mut();
    }

    Success = LdrGetDllHandle(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(DllName), core::ptr::addr_of_mut!(DllHandle));

    if !NT_SUCCESS!(Success) {
        SHIMENG_FAIL!(b"Unable to get module handle for %wZ (%p)\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(DllName), DllBase);
        RtlFreeUnicodeString(core::ptr::addr_of_mut!(DllName));

        return core::ptr::null_mut();
    }
    RtlFreeUnicodeString(core::ptr::addr_of_mut!(DllName));

    return SeiFindHookModuleInfo(core::ptr::null_mut(), DllHandle);
}

unsafe extern "C" fn SeiGetStringPtr(pdb: PDB, tag: TAGID, r#type: TAG) -> LPCWSTR {
    let tagEntry: TAGID = SdbFindFirstTag(pdb, tag, r#type);
    if tagEntry == TAGID_NULL {
        return core::ptr::null();
    }

    return SdbGetStringTagPtr(pdb, tagEntry);
}

unsafe extern "C" fn SeiGetDWORD(pdb: PDB, tag: TAGID, r#type: TAG) -> DWORD {
    let tagEntry: TAGID = SdbFindFirstTag(pdb, tag, r#type);
    if tagEntry == TAGID_NULL {
        return 0;
    }

    return SdbReadDWORDTag(pdb, tagEntry, 0);
}

unsafe extern "C" fn SeiGetQWORD(pdb: PDB, tag: TAGID, type_: TAG) -> QWORD {
    let tagEntry: TAGID = SdbFindFirstTag(pdb, tag, type_);
    if tagEntry == TAGID_NULL {
        return 0;
    }

    return SdbReadQWORDTag(pdb, tagEntry, 0);
}

unsafe extern "C" fn SeiAddShim(trShimRef: TAGREF, pShimRef: PARRAY) {
    let mut Data: *mut TAGREF;

    Data = ARRAY_Append!(pShimRef, TAGREF);
    if Data.is_null() {
        return;
    }

    *Data = trShimRef;
}

unsafe extern "C" fn SeiAddFlag(PDB: PDB, tiFlagRef: TAGID, pFlagInfo: PFLAGINFO) {
    let pdb = PDB;
    let mut Flag: ULARGE_INTEGER = core::mem::zeroed();

    /* Resolve the FLAG_REF to the real FLAG node */
    let FlagTag: TAGID = SeiGetDWORD(pdb, tiFlagRef, (TAG_FLAG_TAGID) as TAG);

    if FlagTag == TAGID_NULL {
        return;
    }

    (*pFlagInfo).AppCompatFlags.QuadPart |= SeiGetQWORD(pdb, FlagTag, (TAG_FLAG_MASK_KERNEL) as TAG);
    (*pFlagInfo).AppCompatFlagsUser.QuadPart |= SeiGetQWORD(pdb, FlagTag, (TAG_FLAG_MASK_USER) as TAG);
    Flag.QuadPart = SeiGetQWORD(pdb, FlagTag, (TAG_FLAG_PROCESSPARAM) as TAG);
    (*pFlagInfo).ProcessParameters_Flags |= Flag.__bindgen_anon_1.LowPart;
}

unsafe extern "C" fn SeiSetLayerEnvVar(wszLayer: LPCWSTR) {
    let mut Status: NTSTATUS;
    let mut VarName: _UNICODE_STRING = RTL_CONSTANT_STRING!(RUSTCONV_WSTR_ABF250C9AB1D.as_ptr());
    let mut Value: _UNICODE_STRING = core::mem::zeroed();

    RtlInitUnicodeString(core::ptr::addr_of_mut!(Value), wszLayer);

    Status = RtlSetEnvironmentVariable(core::ptr::null_mut(), core::ptr::addr_of_mut!(VarName), core::ptr::addr_of_mut!(Value));
    if NT_SUCCESS!(Status) {
        SHIMENG_INFO!(b"%wZ=%wZ\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(VarName), core::ptr::addr_of_mut!(Value));
    } else {
        SHIMENG_FAIL!(b"Failed to set %wZ: 0x%x\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(VarName), Status);
    }
}

unsafe extern "C" fn SeiBuildShimRefArray(hsdb: HSDB, pQuery: *mut SDBQUERYRESULT, pShimRef: PARRAY, pFlagInfo: PFLAGINFO) {
    let mut wszLayerEnvVar: [u16; (MAX_LAYER_LENGTH) as _] = [0u16; (MAX_LAYER_LENGTH) as _];
    let mut n: DWORD;

    n = 0;
    while n < (*pQuery).dwExeCount {
        let mut pdb: PDB = core::mem::zeroed();
        let mut tag: TAGID = core::mem::zeroed();
        if SdbTagRefToTagID(hsdb, (*pQuery).atrExes[n as usize], core::ptr::addr_of_mut!(pdb), core::ptr::addr_of_mut!(tag)) != 0 {
            let ExeName: LPCWSTR = SeiGetStringPtr(pdb, tag, (TAG_NAME) as TAG);
            let mut ShimRef: TAGID = SdbFindFirstTag(pdb, tag, (TAG_SHIM_REF) as TAG);
            let mut FlagRef: TAGID = SdbFindFirstTag(pdb, tag, (TAG_FLAG_REF) as TAG);

            if !ExeName.is_null() {
                SeiDbgPrint(SEI_MSG, core::ptr::null_mut(), b"ShimInfo(Exe(%S))\n\0".as_ptr() as *const i8, ExeName);
            }

            while ShimRef != TAGID_NULL {
                let mut trShimRef: TAGREF = core::mem::zeroed();
                if SdbTagIDToTagRef(hsdb, pdb, ShimRef, core::ptr::addr_of_mut!(trShimRef)) != 0 {
                    SeiAddShim(trShimRef, pShimRef);
                }

                ShimRef = SdbFindNextTag(pdb, tag, ShimRef);
            }

            while FlagRef != TAGID_NULL {
                SeiAddFlag(pdb, FlagRef, pFlagInfo);

                FlagRef = SdbFindNextTag(pdb, tag, FlagRef);
            }
        }
        n += 1;
    }

    n = 0;
    while n < (*pQuery).dwLayerCount {
        let mut pdb: PDB = core::mem::zeroed();
        let mut tag: TAGID = core::mem::zeroed();
        if SdbTagRefToTagID(hsdb, (*pQuery).atrLayers[n as usize], core::ptr::addr_of_mut!(pdb), core::ptr::addr_of_mut!(tag)) != 0 {
            let LayerName: LPCWSTR = SeiGetStringPtr(pdb, tag, (TAG_NAME) as TAG);
            let mut ShimRef: TAGID = SdbFindFirstTag(pdb, tag, (TAG_SHIM_REF) as TAG);
            let mut FlagRef: TAGID = SdbFindFirstTag(pdb, tag, (TAG_FLAG_REF) as TAG);

            if !LayerName.is_null() {
                let mut hr: HRESULT = 0;
                SeiDbgPrint(SEI_MSG, core::ptr::null_mut(), b"ShimInfo(Layer(%S))\n\0".as_ptr() as *const i8, LayerName);
                if wszLayerEnvVar[0 as usize] != 0 {
                    StringCchCatW(wszLayerEnvVar.as_mut_ptr(), (ARRAYSIZE!(wszLayerEnvVar)) as _, RUSTCONV_WSTR_8D71B3FAAB82.as_ptr());
                }
                hr = StringCchCatW(wszLayerEnvVar.as_mut_ptr(), (ARRAYSIZE!(wszLayerEnvVar)) as _, LayerName);
                if !(hr >= 0) {
                    SHIMENG_FAIL!(b"Unable to append %S\n\0".as_ptr() as *const i8, LayerName);
                }
            }

            while ShimRef != TAGID_NULL {
                let mut trShimRef: TAGREF = core::mem::zeroed();
                if SdbTagIDToTagRef(hsdb, pdb, ShimRef, core::ptr::addr_of_mut!(trShimRef)) != 0 {
                    SeiAddShim(trShimRef, pShimRef);
                }

                ShimRef = SdbFindNextTag(pdb, tag, ShimRef);
            }

            while FlagRef != TAGID_NULL {
                SeiAddFlag(pdb, FlagRef, pFlagInfo);

                FlagRef = SdbFindNextTag(pdb, tag, FlagRef);
            }
        }
        n += 1;
    }
    if wszLayerEnvVar[0 as usize] != 0 {
        SeiSetLayerEnvVar(wszLayerEnvVar.as_ptr());
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiAddHooks(hooks: PHOOKAPIEX, dwHookCount: DWORD, pShim: PSHIMINFO) {
    let mut n: DWORD;
    let mut j: DWORD;
    let mut UnicodeModName: _UNICODE_STRING = core::mem::zeroed();
    let mut Buf: [u16; 512] = [0u16; 512];

    RtlInitEmptyUnicodeString(
        core::ptr::addr_of_mut!(UnicodeModName),
        Buf.as_mut_ptr(),
        core::mem::size_of::<[u16; 512]>() as u16,
    );

    n = 0;
    while n < dwHookCount {
        'body: {
            let mut AnsiString: ANSI_STRING = core::mem::zeroed();
            let mut DllHandle: PVOID = core::ptr::null_mut();
            let mut hook: PHOOKAPIEX = hooks.add(n as usize);
            let mut pHookApi: *mut PHOOKAPIEX;
            let mut HookModuleInfo: PHOOKMODULEINFO = core::ptr::null_mut();

            RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiString), (*hook).LibraryName);
            if !NT_SUCCESS!(RtlAnsiStringToUnicodeString(
                core::ptr::addr_of_mut!(UnicodeModName),
                core::ptr::addr_of_mut!(AnsiString),
                (FALSE) as BOOLEAN,
            )) {
                SHIMENG_FAIL!(
                    b"Unable to convert %s to Unicode\n\0".as_ptr() as *const i8,
                    (*hook).LibraryName,
                );
                break 'body;
            }

            if NT_SUCCESS!(LdrGetDllHandle(
                core::ptr::null_mut(),
                core::ptr::null_mut(),
                core::ptr::addr_of_mut!(UnicodeModName),
                core::ptr::addr_of_mut!(DllHandle),
            )) {
                HookModuleInfo = SeiFindHookModuleInfo(core::ptr::null_mut(), DllHandle);
            } else {
                HookModuleInfo = SeiFindHookModuleInfo(
                    core::ptr::addr_of_mut!(UnicodeModName),
                    core::ptr::null_mut(),
                );
                DllHandle = core::ptr::null_mut();
            }

            if HookModuleInfo.is_null() {
                HookModuleInfo = ARRAY_Append!(
                    core::ptr::addr_of_mut!(g_pHookArray),
                    HOOKMODULEINFO,
                );
                if HookModuleInfo.is_null() {
                    break 'body;
                }

                (*HookModuleInfo).BaseAddress = DllHandle;
                ARRAY_Init!(
                    core::ptr::addr_of_mut!((*HookModuleInfo).HookApis),
                    PHOOKAPIEX,
                );
                RtlCreateUnicodeString(
                    core::ptr::addr_of_mut!((*HookModuleInfo).Name),
                    UnicodeModName.Buffer,
                );
            }

            (*hook).pShimInfo = pShim;

            j = 0;
            while j < ARRAY_Size!(core::ptr::addr_of_mut!((*HookModuleInfo).HookApis)) {
                let mut HookApi: PHOOKAPIEX = *ARRAY_At!(
                    core::ptr::addr_of_mut!((*HookModuleInfo).HookApis),
                    PHOOKAPIEX,
                    j,
                );
                let CmpResult: i32 = SeiCompareFunctionName(
                    (*hook).FunctionName,
                    (*HookApi).FunctionName,
                );
                if CmpResult == 0 {
                    while !(*HookApi).ApiLink.is_null() {
                        HookApi = (*HookApi).ApiLink;
                    }
                    (*HookApi).ApiLink = hook;
                    hook = core::ptr::null_mut();
                    break;
                }
                j += 1;
            }
            /* No place found yet, append it */
            if !hook.is_null() {
                pHookApi = ARRAY_Append!(
                    core::ptr::addr_of_mut!((*HookModuleInfo).HookApis),
                    PHOOKAPIEX,
                ) as *mut PHOOKAPIEX;
                if !pHookApi.is_null() {
                    *pHookApi = hook;
                }
            }
        }
        n += 1;
    }
}

// `_ReturnAddress()` has no stable-Rust spelling, so the exported entry is a naked thunk that
// hands its own return address to `StubGetProcAddress_impl` as a third argument (the same
// technique as user32-wsprintf-rust's varargs thunks). The thunk keeps the export's calling
// convention (stdcall on i386: `ret 8`) and is the frame whose caller `_ReturnAddress` names.
#[cfg(target_arch = "x86")]
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "system" fn StubGetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> FARPROC {
    core::arch::naked_asm!(
        "push dword ptr [esp]",
        "push dword ptr [esp + 12]",
        "push dword ptr [esp + 12]",
        "call {h}",
        "add esp, 12",
        "ret 8",
        h = sym StubGetProcAddress_impl,
    );
}
#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "system" fn StubGetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> FARPROC {
    core::arch::naked_asm!(
        ".seh_proc StubGetProcAddress",
        "mov r8, [rsp]",
        "sub rsp, 40",
        ".seh_stackalloc 40",
        ".seh_endprologue",
        "call {h}",
        "add rsp, 40",
        "ret",
        ".seh_endproc",
        h = sym StubGetProcAddress_impl,
    );
}
#[cfg(target_arch = "aarch64")]
#[unsafe(naked)]
#[no_mangle]
pub unsafe extern "system" fn StubGetProcAddress(hModule: HINSTANCE, lpProcName: LPCSTR) -> FARPROC {
    core::arch::naked_asm!(
        ".seh_proc StubGetProcAddress",
        "stp x29, x30, [sp, #-16]!",
        ".seh_save_fplr_x 16",
        "mov x29, sp",
        ".seh_set_fp",
        ".seh_endprologue",
        "mov x2, x30",
        "bl {h}",
        ".seh_startepilogue",
        "ldp x29, x30, [sp], #16",
        ".seh_save_fplr_x 16",
        ".seh_endepilogue",
        "ret",
        ".seh_endproc",
        h = sym StubGetProcAddress_impl,
    );
}

unsafe extern "C" fn StubGetProcAddress_impl(hModule: HINSTANCE, lpProcName: LPCSTR, mut Addr: PVOID) -> FARPROC {
    let mut HookModuleInfo: PHOOKMODULEINFO;
    let mut proc: FARPROC = {
        let f: GETPROCADDRESSPROC = core::mem::transmute(g_IntHookEx[0].OriginalFunction);
        f(hModule, lpProcName)
    };
    let mut szOrdProcFmt: [i8; 10] = [0i8; 10];

    Addr = SeiGetModuleFromAddress(Addr);
    if SE_IsShimDll(Addr) != 0
    {
        SHIMENG_MSG!(b"Not touching GetProcAddress for shim dll (%p!%s)\0".as_ptr() as *const i8, hModule, SeiPrintFunctionName(lpProcName, szOrdProcFmt.as_mut_ptr()));
        return proc;
    }

    SHIMENG_INFO!(b"(GetProcAddress(%p!%s) => %p\n\0".as_ptr() as *const i8, hModule, SeiPrintFunctionName(lpProcName, szOrdProcFmt.as_mut_ptr()), proc);

    HookModuleInfo = SeiFindHookModuleInfo(core::ptr::null_mut(), (hModule).cast());

    if !HookModuleInfo.is_null()
    {
        let mut n: DWORD;
        n = 0;
        while n < ARRAY_Size!(core::ptr::addr_of_mut!((*HookModuleInfo).HookApis)) as DWORD
        {
            let HookApi: PHOOKAPIEX = *ARRAY_At!(core::ptr::addr_of_mut!((*HookModuleInfo).HookApis), PHOOKAPIEX, n);
            let CmpResult: i32 = SeiCompareFunctionName(lpProcName, (*HookApi).FunctionName);
            if CmpResult == 0
            {
                SHIMENG_MSG!(b"Redirecting %p to %p\n\0".as_ptr() as *const i8, proc, (*HookApi).ReplacementFunction);
                proc = core::mem::transmute((*HookApi).ReplacementFunction);
                break;
            }
            n += 1;
        }
    }

    return proc;
}
#[no_mangle]
pub unsafe extern "C" fn SeiResolveAPI(HookModuleInfo: PHOOKMODULEINFO) {
    let mut n: DWORD;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();

    n = 0;
    while n < ARRAY_Size!(core::ptr::addr_of_mut!((*HookModuleInfo).HookApis)) {
        let mut Status: NTSTATUS;
        let mut ProcAddress: PVOID = core::ptr::null_mut();
        let mut HookApi: PHOOKAPIEX = *ARRAY_At!(
            core::ptr::addr_of_mut!((*HookModuleInfo).HookApis),
            PHOOKAPIEX,
            n
        );

        'next: {
            if SeiIsOrdinalName((*HookApi).FunctionName) == 0 {
                RtlInitAnsiString(core::ptr::addr_of_mut!(AnsiString), (*HookApi).FunctionName);
                Status = LdrGetProcedureAddress(
                    (*HookModuleInfo).BaseAddress,
                    core::ptr::addr_of_mut!(AnsiString),
                    0,
                    core::ptr::addr_of_mut!(ProcAddress)
                );
            } else {
                Status = LdrGetProcedureAddress(
                    (*HookModuleInfo).BaseAddress,
                    core::ptr::null_mut(),
                    ((*HookApi).FunctionName as ULONG_PTR) as ULONG,
                    core::ptr::addr_of_mut!(ProcAddress)
                );
            }

            if !NT_SUCCESS!(Status) {
                let mut szOrdProcFmt = [0i8; 10];
                let lpFunctionName = SeiPrintFunctionName((*HookApi).FunctionName, szOrdProcFmt.as_mut_ptr());
                SHIMENG_FAIL!(
                    b"Unable to retrieve %s!%s\n\0".as_ptr() as *const i8,
                    (*HookApi).LibraryName,
                    lpFunctionName
                );
                break 'next;
            }

            (*HookApi).OriginalFunction = ProcAddress;
            if !(*HookApi).ApiLink.is_null() {
                SHIMENG_MSG!(b"TODO: Figure out how to handle conflicting In/Exports with ApiLink!\n\0".as_ptr() as *const i8);
            }
            while !(*HookApi).ApiLink.is_null() {
                (*(*HookApi).ApiLink).OriginalFunction = (*HookApi).OriginalFunction;
                (*HookApi).OriginalFunction = (*(*HookApi).ApiLink).ReplacementFunction;
                HookApi = (*HookApi).ApiLink;
            }
        }

        n += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiResolveAPIs() {
    let mut n: DWORD;
    n = 0;
    while n < ARRAY_Size!(core::ptr::addr_of_mut!(g_pHookArray)) {
        let pModuleInfo: PHOOKMODULEINFO = ARRAY_At!(core::ptr::addr_of_mut!(g_pHookArray), HOOKMODULEINFO, n) as PHOOKMODULEINFO;

        /* Is this module loaded? */
        if !(*pModuleInfo).BaseAddress.is_null() {
            SeiResolveAPI(pModuleInfo);
        }
        n += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiCombineHookInfo() {
    let mut r#mod: DWORD;
    let mut n: DWORD;

    /* Enumerate all Shim modules */
    r#mod = 0;
    while r#mod < ARRAY_Size!(core::ptr::addr_of_mut!(g_pShimInfo)) {
        let pShimModule: PSHIMMODULE = *(ARRAY_At!(core::ptr::addr_of_mut!(g_pShimInfo), PSHIMMODULE, r#mod) as *mut PSHIMMODULE);
        let dwShimCount: DWORD = ARRAY_Size!(core::ptr::addr_of_mut!((*pShimModule).EnabledShims));

        /* Enumerate all Shims */
        n = 0;
        while n < dwShimCount {
            let pShim: PSHIMINFO = *(ARRAY_At!(core::ptr::addr_of_mut!((*pShimModule).EnabledShims), PSHIMINFO, n) as *mut PSHIMINFO);

            let hooks: PHOOKAPIEX = (*pShim).pHookApi;
            let dwHookCount: DWORD = (*pShim).dwHookCount;

            SeiAddHooks(hooks, dwHookCount, pShim);

            n += 1;
        }

        r#mod += 1;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiAddInternalHooks(dwNumHooks: DWORD) {
    if dwNumHooks == 0 {
        g_bInternalHooksUsed = (FALSE) as i32;
        return;
    }

    SeiAddHooks(
        core::ptr::addr_of_mut!(g_IntHookEx) as *mut HOOKAPIEX,
        (core::mem::size_of::<[HOOKAPIEX; 1]>() / core::mem::size_of::<HOOKAPIEX>()) as DWORD,
        core::ptr::null_mut(),
    );
    g_bInternalHooksUsed = (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn SeiPatchNewImport(FirstThunk: PIMAGE_THUNK_DATA, HookApi: PHOOKAPIEX, LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    let mut OldProtection: ULONG = 0;
    let mut Ptr: PVOID = core::ptr::null_mut();
    let mut Size: SIZE_T = 0;
    let mut Status: NTSTATUS;
    let mut szOrdProcFmt: [i8; 10] = [0; 10];

    SHIMENG_INFO!(b"Hooking API \"%s!%s\" for DLL \"%wZ\"\n\0".as_ptr() as *const i8, (*HookApi).LibraryName, SeiPrintFunctionName((*HookApi).FunctionName, szOrdProcFmt.as_mut_ptr()), core::ptr::addr_of_mut!((*LdrEntry).BaseDllName));

    Ptr = core::ptr::addr_of_mut!((*FirstThunk).u1.Function) as PVOID;
    Size = (core::mem::size_of_val(&(*FirstThunk).u1.Function)) as _;
    Status = NtProtectVirtualMemory(NtCurrentProcess(), core::ptr::addr_of_mut!(Ptr), core::ptr::addr_of_mut!(Size), PAGE_EXECUTE_READWRITE, core::ptr::addr_of_mut!(OldProtection));

    if !NT_SUCCESS!(Status) {
        SHIMENG_FAIL!(b"Unable to unprotect 0x%p\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!((*FirstThunk).u1.Function));
        return;
    }

    SHIMENG_INFO!(b"changing 0x%p to 0x%p\n\0".as_ptr() as *const i8, (*FirstThunk).u1.Function, (*HookApi).ReplacementFunction);
    (*FirstThunk).u1.Function = (*HookApi).ReplacementFunction as ULONG_PTR;

    Size = (core::mem::size_of_val(&(*FirstThunk).u1.Function)) as _;
    Status = NtProtectVirtualMemory(NtCurrentProcess(), core::ptr::addr_of_mut!(Ptr), core::ptr::addr_of_mut!(Size), OldProtection, core::ptr::addr_of_mut!(OldProtection));

    if !NT_SUCCESS!(Status) {
        SHIMENG_WARN!(b"Unable to reprotect 0x%p\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!((*FirstThunk).u1.Function));
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiFindInExclude(InExclude: PARRAY, DllName: PCUNICODE_STRING) -> PINEXCLUDE {
    let mut n: DWORD;

    n = 0;
    while n < ARRAY_Size!(InExclude) {
        let InEx: PINEXCLUDE = ARRAY_At!(InExclude, INEXCLUDE, n);

        if RtlEqualUnicodeString(core::ptr::addr_of!((*InEx).Module), DllName, (TRUE) as BOOLEAN) != 0 {
            return InEx;
        }
        n += 1;
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "C" fn SeiIsExcluded(LdrEntry: PLDR_DATA_TABLE_ENTRY, HookApi: PHOOKAPIEX) -> BOOL {
    let mut pShimInfo: PSHIMINFO = (*HookApi).pShimInfo;
    let mut InExclude: PINEXCLUDE;
    let mut IsExcluded: BOOL = (FALSE) as i32;
    let mut szOrdProcFmt: [i8; 10] = [0; 10];

    if pShimInfo.is_null() {
        /* Internal hook, do not exclude it */
        return (FALSE) as BOOL;
    }

    /* By default, everything from System32 or WinSxs is excluded */
    if RtlPrefixUnicodeString(addr_of_mut!(g_System32Directory), addr_of_mut!((*LdrEntry).FullDllName), (TRUE) as BOOLEAN) != 0
        || RtlPrefixUnicodeString(addr_of_mut!(g_SxsDirectory), addr_of_mut!((*LdrEntry).FullDllName), (TRUE) as BOOLEAN) != 0
    {
        IsExcluded = (TRUE) as i32;
    }

    InExclude = SeiFindInExclude(addr_of_mut!((*pShimInfo).InExclude), addr_of_mut!((*LdrEntry).BaseDllName));
    if !InExclude.is_null() {
        /* If it is on the 'exclude' list, bail out */
        if (*InExclude).Include == 0 {
            SHIMENG_INFO!(
                b"Module '%wZ' excluded for shim %S, API '%s!%s', because it on in the exclude list.\n\0".as_ptr() as *const i8,
                addr_of_mut!((*LdrEntry).BaseDllName),
                (*pShimInfo).ShimName,
                (*HookApi).LibraryName,
                SeiPrintFunctionName((*HookApi).FunctionName, szOrdProcFmt.as_mut_ptr()),
            );

            return (TRUE) as BOOL;
        }
        /* If it is on the 'include' list, override System32 / Winsxs check. */
        if IsExcluded != 0 {
            SHIMENG_INFO!(
                b"Module '%wZ' included for shim %S, API '%s!%s', because it is on the include list.\n\0".as_ptr() as *const i8,
                addr_of_mut!((*LdrEntry).BaseDllName),
                (*pShimInfo).ShimName,
                (*HookApi).LibraryName,
                SeiPrintFunctionName((*HookApi).FunctionName, szOrdProcFmt.as_mut_ptr()),
            );

        }
        IsExcluded = (FALSE) as i32;
    }

    if IsExcluded != 0 {
        SHIMENG_INFO!(
            b"Module '%wZ' excluded for shim %S, API '%s!%s', because it is in System32/WINSXS.\n\0".as_ptr() as *const i8,
            addr_of_mut!((*LdrEntry).BaseDllName),
            (*pShimInfo).ShimName,
            (*HookApi).LibraryName,
            SeiPrintFunctionName((*HookApi).FunctionName, szOrdProcFmt.as_mut_ptr()),
        );
    }

    return IsExcluded;
}
#[no_mangle]
pub unsafe extern "C" fn SeiAppendInExclude(dest: PARRAY, ModuleName: PCWSTR, IsInclude: BOOL) {
    let mut InExclude: PINEXCLUDE;
    let mut ModuleNameU: _UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(core::ptr::addr_of_mut!(ModuleNameU), ModuleName);

    InExclude = SeiFindInExclude(dest, core::ptr::addr_of_mut!(ModuleNameU));
    if !InExclude.is_null() {
        (*InExclude).Include = IsInclude;
        return;
    }

    InExclude = ARRAY_Append!(dest, INEXCLUDE);
    if !InExclude.is_null() {
        let ModuleNameCopy: PCWSTR = SdbpStrDup(ModuleName);
        RtlInitUnicodeString(core::ptr::addr_of_mut!((*InExclude).Module), ModuleNameCopy);
        (*InExclude).Include = IsInclude;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiReadInExclude(pdb: PDB, parent: TAGID, dest: PARRAY) {
    let mut InExcludeTag: TAGID;

    InExcludeTag = SdbFindFirstTag(pdb, parent, (TAG_INEXCLUD) as TAG);

    while InExcludeTag != TAGID_NULL {
        let mut ModuleName: PCWSTR;
        let ModuleTag: TAGID = SdbFindFirstTag(pdb, InExcludeTag, (TAG_MODULE) as TAG);
        let IncludeTag: TAGID = SdbFindFirstTag(pdb, InExcludeTag, (TAG_INCLUDE) as TAG);

        ModuleName = SdbGetStringTagPtr(pdb, ModuleTag);
        if !ModuleName.is_null() {
            SeiAppendInExclude(dest, ModuleName, (IncludeTag != TAGID_NULL) as BOOL);
        } else {
            SHIMENG_WARN!(b"INEXCLUDE without Module: 0x%x\n\0".as_ptr() as *const i8, InExcludeTag);
        }

        InExcludeTag = SdbFindNextTag(pdb, parent, InExcludeTag);
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiBuildGlobalInclExclList(hsdb: HSDB) {
    let mut pdb: PDB = core::mem::zeroed();
    let mut tr: TAGREF = TAGREF_ROOT;
    let mut root: TAGID = core::mem::zeroed();
    let mut db: TAGID = core::mem::zeroed();
    let mut library: TAGID = core::mem::zeroed();

    if SdbTagRefToTagID(hsdb, tr, core::ptr::addr_of_mut!(pdb), core::ptr::addr_of_mut!(root)) == 0 {
        SHIMENG_WARN!(b"Unable to resolve database root\n\0".as_ptr() as *const i8);
        return;
    }
    db = SdbFindFirstTag(pdb, root, (TAG_DATABASE) as TAG);
    if db == TAGID_NULL {
        SHIMENG_WARN!(b"Unable to resolve database\n\0".as_ptr() as *const i8);
        return;
    }
    library = SdbFindFirstTag(pdb, db, (TAG_LIBRARY) as TAG);
    if library == TAGID_NULL {
        SHIMENG_WARN!(b"Unable to resolve library\n\0".as_ptr() as *const i8);
        return;
    }

    SeiReadInExclude(pdb, library, core::ptr::addr_of_mut!(g_InExclude));
}
#[no_mangle]
pub unsafe extern "C" fn SeiBuildInclExclList(pdb: PDB, ShimTag: TAGID, pShimInfo: PSHIMINFO) {
    let mut n: DWORD;

    /* First duplicate the global in/excludes */
    n = 0;
    while n < ARRAY_Size!(core::ptr::addr_of_mut!(g_InExclude)) {
        let InEx: PINEXCLUDE = ARRAY_At!(core::ptr::addr_of_mut!(g_InExclude), INEXCLUDE, n);
        SeiAppendInExclude(
            core::ptr::addr_of_mut!((*pShimInfo).InExclude),
            (*InEx).Module.Buffer,
            (*InEx).Include,
        );
        n += 1;
    }

    /* Now read this shim's in/excludes (possibly overriding the global ones) */
    SeiReadInExclude(pdb, ShimTag, core::ptr::addr_of_mut!((*pShimInfo).InExclude));
}
#[no_mangle]
pub unsafe extern "C" fn SeiPatchImportsByName(mut OriginalThunk: PIMAGE_THUNK_DATA, mut FirstThunk: PIMAGE_THUNK_DATA, HookApi: PHOOKAPIEX, LdrEntry: PLDR_DATA_TABLE_ENTRY) -> DWORD {
    let mut dwFound: DWORD = 0;

    while (*OriginalThunk).u1.AddressOfData != 0 && (*FirstThunk).u1.Function != 0 {
        if !IMAGE_SNAP_BY_ORDINAL!((*OriginalThunk).u1.Function) && SeiIsOrdinalName((*HookApi).FunctionName) == 0 {
            let mut ImportName: PIMAGE_IMPORT_BY_NAME;

            ImportName = ((*LdrEntry).DllBase as PBYTE).add((*OriginalThunk).u1.Function as usize) as PIMAGE_IMPORT_BY_NAME;
            if strcmp(core::ptr::addr_of!((*ImportName).Name) as PCSTR, (*HookApi).FunctionName) == 0 {
                SeiPatchNewImport(FirstThunk, HookApi, LdrEntry);
                dwFound += 1;
            }
        } else {
            if SeiIsOrdinalName((*HookApi).FunctionName) != 0 && (IMAGE_ORDINAL!((*OriginalThunk).u1.Function) as usize as PCSTR == (*HookApi).FunctionName) {
                SeiPatchNewImport(FirstThunk, HookApi, LdrEntry);
                dwFound += 1;
            }
        }
        OriginalThunk = OriginalThunk.add(1);
        FirstThunk = FirstThunk.add(1);
    }

    return dwFound;
}
#[no_mangle]
pub unsafe extern "C" fn SeiPatchImportsByAddress(mut FirstThunk: PIMAGE_THUNK_DATA, HookApi: PHOOKAPIEX, LdrEntry: PLDR_DATA_TABLE_ENTRY) -> DWORD {
    let mut dwFound: DWORD = 0;

    while (*FirstThunk).u1.Function != 0 {
        if (*FirstThunk).u1.Function as PULONG_PTR == (*HookApi).OriginalFunction as PULONG_PTR {
            SeiPatchNewImport(FirstThunk, HookApi, LdrEntry);
            dwFound += 1;
        }
        FirstThunk = FirstThunk.add(1);
    }

    return dwFound;
}
#[no_mangle]
pub unsafe extern "C" fn SeiHookImports(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    let mut HasImportNameTable: BOOLEAN;
    let mut Size: ULONG = 0;
    let mut ImportDescriptor: PIMAGE_IMPORT_DESCRIPTOR;
    let mut DllBase: PBYTE = ((*LdrEntry).DllBase).cast();

    if SE_IsShimDll(DllBase as PVOID) != 0
        || (g_hInstance as usize) == ((*LdrEntry).DllBase as usize)
        || RtlEqualUnicodeString(
            core::ptr::addr_of_mut!(g_LoadingShimDll),
            core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
            (TRUE) as BOOLEAN,
        ) != 0
    {
        SHIMENG_INFO!(
            b"Skipping shim module 0x%p \"%wZ\"\n\0".as_ptr() as *const i8,
            (*LdrEntry).DllBase,
            core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
        );
        return;
    }

    if (*LdrEntry).Flags & LDRP_COMPAT_DATABASE_PROCESSED != 0 {
        SHIMENG_INFO!(
            b"Skipping module 0x%p \"%wZ\" because it was already processed\n\0".as_ptr() as *const i8,
            (*LdrEntry).DllBase,
            core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
        );
        return;
    }

    ImportDescriptor = RtlImageDirectoryEntryToData(
        DllBase as PVOID,
        (TRUE) as BOOLEAN,
        (IMAGE_DIRECTORY_ENTRY_IMPORT) as USHORT,
        core::ptr::addr_of_mut!(Size),
    ) as PIMAGE_IMPORT_DESCRIPTOR;
    if ImportDescriptor.is_null() {
        SHIMENG_INFO!(
            b"Skipping module 0x%p \"%wZ\" due to no iat found\n\0".as_ptr() as *const i8,
            (*LdrEntry).DllBase,
            core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
        );
        return;
    }

    SHIMENG_INFO!(
        b"Hooking module 0x%p \"%wZ\"\n\0".as_ptr() as *const i8,
        (*LdrEntry).DllBase,
        core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
    );

    HasImportNameTable = (*ImportDescriptor).__bindgen_anon_1.OriginalFirstThunk as BOOLEAN;
    if HasImportNameTable == 0 {
        /* Some PEs (eg. built with Borland toolchain or compressed with UPX) may not have an import name table.
         * In that case we have to rely solely on an import address table. */
        SHIMENG_INFO!(
            b"Module 0x%p \"%wZ\" does not have an import name table\n\0".as_ptr() as *const i8,
            (*LdrEntry).DllBase,
            core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
        );
    }

    while (*ImportDescriptor).Name != 0
        && (if HasImportNameTable != 0 {
            (*ImportDescriptor).__bindgen_anon_1.OriginalFirstThunk
        } else {
            (*ImportDescriptor).FirstThunk
        }) != 0
    {
        let HookModuleInfo: PHOOKMODULEINFO;

        /* Do we have hooks for this module? */
        HookModuleInfo = SeiFindHookModuleInfoForImportDescriptor(DllBase, ImportDescriptor);

        if !HookModuleInfo.is_null() {
            let mut OriginalThunk: PIMAGE_THUNK_DATA;
            let mut FirstThunk: PIMAGE_THUNK_DATA;
            let mut n: DWORD;

            n = 0;
            while n < ARRAY_Size!(core::ptr::addr_of_mut!((*HookModuleInfo).HookApis)) {
                'body: {
                    let mut dwFound: DWORD;
                    let HookApi: PHOOKAPIEX = *ARRAY_At!(
                        core::ptr::addr_of_mut!((*HookModuleInfo).HookApis),
                        PHOOKAPIEX,
                        n,
                    );

                    /* Check if this module should be excluded from being hooked (system32/winsxs, global or shim exclude) */
                    if SeiIsExcluded(LdrEntry, HookApi) != 0 {
                        break 'body;
                    }

                    FirstThunk = DllBase.add((*ImportDescriptor).FirstThunk as usize) as PIMAGE_THUNK_DATA;

                    if HasImportNameTable != 0 {
                        OriginalThunk = DllBase.add((*ImportDescriptor).__bindgen_anon_1.OriginalFirstThunk as usize) as PIMAGE_THUNK_DATA;
                        dwFound = SeiPatchImportsByName(OriginalThunk, FirstThunk, HookApi, LdrEntry);
                    } else {
                        dwFound = SeiPatchImportsByAddress(FirstThunk, HookApi, LdrEntry);
                    }

                    /* Sadly, IAT does not have to be sorted, and can even contain duplicate entries. */
                    if dwFound != 1 {
                        let mut szOrdProcFmt: [i8; 10] = [0i8; 10];
                        let FuncName: LPCSTR = SeiPrintFunctionName(
                            (*HookApi).FunctionName,
                            szOrdProcFmt.as_mut_ptr(),
                        );

                        /* One entry not found. */
                        if dwFound == 0 {
                            SHIMENG_INFO!(
                                b"Entry \"%s!%s\" not found for \"%wZ\"\n\0".as_ptr() as *const i8,
                                (*HookApi).LibraryName,
                                FuncName,
                                core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                            );
                        } else {
                            SHIMENG_INFO!(
                                b"Entry \"%s!%s\" found %d times for \"%wZ\"\n\0".as_ptr() as *const i8,
                                (*HookApi).LibraryName,
                                FuncName,
                                dwFound,
                                core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                            );
                        }
                    }
                }
                n += 1;
            }
        }

        ImportDescriptor = ImportDescriptor.add(1);
    }

    /* Mark this module as processed. */
    (*LdrEntry).Flags |= LDRP_COMPAT_DATABASE_PROCESSED;
}
#[no_mangle]
pub unsafe extern "C" fn PatchNewModules(Peb: PPEB) {
    let mut ListHead: PLIST_ENTRY;
    let mut ListEntry: PLIST_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;

    ListHead = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InLoadOrderModuleList);
    ListEntry = (*ListHead).Flink;

    while ListHead != ListEntry {
        LdrEntry = CONTAINING_RECORD!(ListEntry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
        SeiHookImports(LdrEntry);

        ListEntry = (*ListEntry).Flink;
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiInitPaths() {
    let WindowsDirectory: PWSTR = SdbpStrDup((*SharedUserData).NtSystemRoot.as_ptr() as PWSTR);
    RtlInitUnicodeString(core::ptr::addr_of_mut!(g_WindowsDirectory), WindowsDirectory);

    g_System32Directory.MaximumLength = ((g_WindowsDirectory.Length as u32)
        + (SdbpStrsize(SYSTEM32.as_ptr()) as u32)) as u16;
    g_System32Directory.Buffer = SdbpAlloc((g_System32Directory.MaximumLength) as SIZE_T) as PWSTR;
    RtlCopyUnicodeString(core::ptr::addr_of_mut!(g_System32Directory), core::ptr::addr_of_mut!(g_WindowsDirectory));
    RtlAppendUnicodeToString(core::ptr::addr_of_mut!(g_System32Directory), SYSTEM32.as_ptr());

    g_SxsDirectory.MaximumLength = ((g_WindowsDirectory.Length as u32)
        + (SdbpStrsize(WINSXS.as_ptr()) as u32)) as u16;
    g_SxsDirectory.Buffer = SdbpAlloc((g_SxsDirectory.MaximumLength) as SIZE_T) as PWSTR;
    RtlCopyUnicodeString(core::ptr::addr_of_mut!(g_SxsDirectory), core::ptr::addr_of_mut!(g_WindowsDirectory));
    RtlAppendUnicodeToString(core::ptr::addr_of_mut!(g_SxsDirectory), WINSXS.as_ptr());
}
#[no_mangle]
pub unsafe extern "C" fn SeiSetEntryProcessed(Peb: PPEB) {
    let mut ListHead: PLIST_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;

    ListHead = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InInitializationOrderModuleList);
    Entry = (*ListHead).Flink;
    while Entry != ListHead {
        LdrEntry = (Entry as *mut u8).sub(core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks)) as PLDR_DATA_TABLE_ENTRY;
        Entry = (*Entry).Flink;

        if RtlEqualUnicodeString(core::ptr::addr_of_mut!((*LdrEntry).BaseDllName), core::ptr::addr_of!(Ntdll), (TRUE) as BOOLEAN) != 0 ||
            RtlEqualUnicodeString(core::ptr::addr_of_mut!((*LdrEntry).BaseDllName), core::ptr::addr_of!(Kernel32), (TRUE) as BOOLEAN) != 0 ||
            RtlEqualUnicodeString(core::ptr::addr_of_mut!((*LdrEntry).BaseDllName), core::ptr::addr_of!(Verifier), (TRUE) as BOOLEAN) != 0 ||
            RtlEqualUnicodeString(core::ptr::addr_of_mut!((*LdrEntry).BaseDllName), core::ptr::addr_of!(g_LoadingShimDll), (TRUE) as BOOLEAN) != 0 ||
            SE_IsShimDll((*LdrEntry).DllBase) != 0 ||
            (((*LdrEntry).Flags & LDRP_ENTRY_PROCESSED) != 0)
        {
            SHIMENG_WARN!(b"Don't mess with 0x%p '%wZ'\n\0".as_ptr() as *const i8, (*LdrEntry).DllBase, core::ptr::addr_of_mut!((*LdrEntry).BaseDllName));
        } else {
            SHIMENG_WARN!(b"Touching        0x%p '%wZ'\n\0".as_ptr() as *const i8, (*LdrEntry).DllBase, core::ptr::addr_of_mut!((*LdrEntry).BaseDllName));
            (*LdrEntry).Flags |= (LDRP_ENTRY_PROCESSED | LDRP_SHIMENG_SUPPRESSED_ENTRY);
        }
    }

    ListHead = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InMemoryOrderModuleList);
    Entry = (*ListHead).Flink;
    SHIMENG_INFO!(b"In memory:\n\0".as_ptr() as *const i8);
    while Entry != ListHead {
        LdrEntry = (Entry as *mut u8).sub(core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InMemoryOrderLinks)) as PLDR_DATA_TABLE_ENTRY;
        Entry = (*Entry).Flink;

        SHIMENG_INFO!(b"    0x%p '%wZ'\n\0".as_ptr() as *const i8, (*LdrEntry).DllBase, core::ptr::addr_of_mut!((*LdrEntry).BaseDllName));
    }

    ListHead = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InLoadOrderModuleList);
    Entry = (*ListHead).Flink;
    SHIMENG_INFO!(b"In load:\n\0".as_ptr() as *const i8);
    while Entry != ListHead {
        LdrEntry = (Entry as *mut u8).sub(core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as PLDR_DATA_TABLE_ENTRY;
        Entry = (*Entry).Flink;

        SHIMENG_INFO!(b"    0x%p '%wZ'\n\0".as_ptr() as *const i8, (*LdrEntry).DllBase, core::ptr::addr_of_mut!((*LdrEntry).BaseDllName));
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiResetEntryProcessed(Peb: PPEB) {
    let mut ListHead: PLIST_ENTRY;
    let mut Entry: PLIST_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;

    ListHead = core::ptr::addr_of_mut!((*(*NtCurrentPeb()).Ldr).InInitializationOrderModuleList);
    Entry = (*ListHead).Flink;
    while Entry != ListHead {
        LdrEntry = CONTAINING_RECORD!(Entry, LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks);
        Entry = (*Entry).Flink;

        if SE_IsShimDll((*LdrEntry).DllBase) != 0
            || (g_hInstance as *mut core::ffi::c_void) == (*LdrEntry).DllBase
            || RtlEqualUnicodeString(
                core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                core::ptr::addr_of!(Ntdll),
                (TRUE) as BOOLEAN,
            ) != 0
            || RtlEqualUnicodeString(
                core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                core::ptr::addr_of!(Kernel32),
                (TRUE) as BOOLEAN,
            ) != 0
            || RtlEqualUnicodeString(
                core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                core::ptr::addr_of!(Verifier),
                (TRUE) as BOOLEAN,
            ) != 0
            || ((*LdrEntry).Flags & LDRP_SHIMENG_SUPPRESSED_ENTRY) == 0
        {
            SHIMENG_WARN!(
                b"Don't mess with 0x%p '%wZ'\n\0".as_ptr() as *const i8,
                (*LdrEntry).DllBase,
                core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
            );
        } else {
            SHIMENG_WARN!(
                b"Resetting       0x%p '%wZ'\n\0".as_ptr() as *const i8,
                (*LdrEntry).DllBase,
                core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
            );
            (*LdrEntry).Flags &= !(LDRP_ENTRY_PROCESSED | LDRP_SHIMENG_SUPPRESSED_ENTRY);
        }
    }
}
#[no_mangle]
pub unsafe extern "C" fn SeiInit(ProcessImage: LPCWSTR, hsdb: HSDB, pQuery: *mut SDBQUERYRESULT, ProcessInit: BOOLEAN) {
    let mut n: DWORD;
    let mut ShimRefArray: ARRAY = core::mem::zeroed();
    let mut dwTotalHooks: DWORD = 0;
    let mut ShimFlags: FLAGINFO = core::mem::zeroed();

    let Peb: PPEB = NtCurrentPeb();

    /* We should only be called once! */

    ARRAY_Init!(core::ptr::addr_of_mut!(ShimRefArray), TAGREF);
    ARRAY_Init!(core::ptr::addr_of_mut!(g_pShimInfo), PSHIMMODULE);
    ARRAY_Init!(core::ptr::addr_of_mut!(g_pHookArray), HOOKMODULEINFO);
    ARRAY_Init!(core::ptr::addr_of_mut!(g_InExclude), INEXCLUDE);
    core::ptr::write_bytes(core::ptr::addr_of_mut!(ShimFlags) as *mut u8, 0, core::mem::size_of::<FLAGINFO>());

    SeiInitPaths();

    SeiCheckComPlusImage((*Peb).ImageBaseAddress);

    if ProcessInit != 0 {
        /* Mark all modules loaded until now as 'LDRP_ENTRY_PROCESSED' so that their entrypoint is not called while we are loading shims */
        SeiSetEntryProcessed(Peb);
    }

    /* TODO:
    if (pQuery->trApphelp)
        SeiDisplayAppHelp(?pQuery->trApphelp?);
    */

    SeiDbgPrint(SEI_MSG, core::ptr::null_mut(), b"ShimInfo(ExePath(%S))\n\0".as_ptr() as *const i8, ProcessImage);
    SeiBuildShimRefArray(hsdb, pQuery, core::ptr::addr_of_mut!(ShimRefArray), core::ptr::addr_of_mut!(ShimFlags));
    if ShimFlags.AppCompatFlags.QuadPart != 0 {
        SeiDbgPrint(SEI_MSG, core::ptr::null_mut(), b"Using KERNEL apphack flags 0x%llx\n\0".as_ptr() as *const i8, ShimFlags.AppCompatFlags.QuadPart);
        (*Peb).AppCompatFlags.QuadPart |= ShimFlags.AppCompatFlags.QuadPart;
    }
    if ShimFlags.AppCompatFlagsUser.QuadPart != 0 {
        SeiDbgPrint(SEI_MSG, core::ptr::null_mut(), b"Using USER apphack flags 0x%llx\n\0".as_ptr() as *const i8, ShimFlags.AppCompatFlagsUser.QuadPart);
        (*Peb).AppCompatFlagsUser.QuadPart |= ShimFlags.AppCompatFlagsUser.QuadPart;
    }
    if ShimFlags.ProcessParameters_Flags != 0 {
        SeiDbgPrint(SEI_MSG, core::ptr::null_mut(), b"Using ProcessParameters flags 0x%x\n\0".as_ptr() as *const i8, ShimFlags.ProcessParameters_Flags);
        (*(*Peb).ProcessParameters).Flags |= ShimFlags.ProcessParameters_Flags;
    }
    SeiDbgPrint(SEI_MSG, core::ptr::null_mut(), b"ShimInfo(Complete)\n\0".as_ptr() as *const i8);

    SHIMENG_INFO!(b"Got %d shims\n\0".as_ptr() as *const i8, ARRAY_Size!(core::ptr::addr_of_mut!(ShimRefArray)));
    SeiBuildGlobalInclExclList(hsdb);

    /* Walk all shims referenced (in layers + exes), and load their modules */
    n = 0;
    loop {
        if !(n < ARRAY_Size!(core::ptr::addr_of_mut!(ShimRefArray))) {
            break;
        }
        'body: {
            let mut pdb: PDB = core::mem::zeroed();
            let mut ShimRef: TAGID = core::mem::zeroed();

            let tr: TAGREF = *ARRAY_At!(core::ptr::addr_of_mut!(ShimRefArray), TAGREF, n);

            if SdbTagRefToTagID(hsdb, tr, core::ptr::addr_of_mut!(pdb), core::ptr::addr_of_mut!(ShimRef)) != 0 {
                let mut ShimName: LPCWSTR = core::mem::zeroed();
                let mut DllName: LPCWSTR = core::mem::zeroed();
                let mut CommandLine: LPCWSTR = core::ptr::null();
                let mut ShimTag: TAGID = core::mem::zeroed();
                let mut FullNameBuffer: [u16; (MAX_PATH) as _] = core::mem::zeroed();
                let mut UnicodeDllName: _UNICODE_STRING = core::mem::zeroed();
                let mut BaseAddress: PVOID = core::ptr::null_mut();
                let mut pShimModuleInfo: PSHIMMODULE = core::ptr::null_mut();
                let mut AnsiCommandLine: ANSI_STRING = RTL_CONSTANT_STRING_A!(b"\0");
                let mut pShimInfo: PSHIMINFO = core::ptr::null_mut();
                let mut pHookApi: PHOOKAPIEX = core::mem::zeroed();
                let mut dwHookCount: DWORD = core::mem::zeroed();

                ShimName = SeiGetStringPtr(pdb, ShimRef, (TAG_NAME) as TAG);
                if ShimName.is_null() {
                    SHIMENG_FAIL!(b"Failed to retrieve the name for 0x%x\n\0".as_ptr() as *const i8, tr);
                    break 'body;
                }

                CommandLine = SeiGetStringPtr(pdb, ShimRef, (TAG_COMMAND_LINE) as TAG);
                if !CommandLine.is_null() && *CommandLine != 0 {
                    RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeDllName), CommandLine);
                    if NT_SUCCESS!(RtlUnicodeStringToAnsiString(core::ptr::addr_of_mut!(AnsiCommandLine), core::ptr::addr_of_mut!(UnicodeDllName), (TRUE) as BOOLEAN)) {
                        SHIMENG_INFO!(b"COMMAND LINE %s for %S\0".as_ptr() as *const i8, AnsiCommandLine.Buffer, ShimName);
                    } else {
                        AnsiCommandLine.Buffer = b"\0".as_ptr() as *mut i8;
                        CommandLine = core::ptr::null();
                    }
                }

                ShimTag = SeiGetDWORD(pdb, ShimRef, (TAG_SHIM_TAGID) as TAG);
                if ShimTag == 0 {
                    SHIMENG_FAIL!(b"Failed to resolve %S to a shim\n\0".as_ptr() as *const i8, ShimName);
                    break 'body;
                }

                if !((SdbGetAppPatchDir(core::ptr::null_mut(), FullNameBuffer.as_mut_ptr(), ARRAYSIZE!(FullNameBuffer))) >= 0) {
                    SHIMENG_WARN!(b"Failed to get the AppPatch dir\n\0".as_ptr() as *const i8);
                    break 'body;
                }

                DllName = SeiGetStringPtr(pdb, ShimTag, (TAG_DLLFILE) as TAG);
                if DllName.is_null() ||
                    !((StringCchCatW(FullNameBuffer.as_mut_ptr(), (ARRAYSIZE!(FullNameBuffer)) as _, RUSTCONV_WSTR_B7FC02907B1F.as_ptr())) >= 0) ||
                    !((StringCchCatW(FullNameBuffer.as_mut_ptr(), (ARRAYSIZE!(FullNameBuffer)) as _, DllName)) >= 0)
                {
                    SHIMENG_WARN!(b"Failed to build a full path for %S\n\0".as_ptr() as *const i8, ShimName);
                    break 'body;
                }

                RtlInitUnicodeString(core::ptr::addr_of_mut!(g_LoadingShimDll), DllName);
                RtlInitUnicodeString(core::ptr::addr_of_mut!(UnicodeDllName), FullNameBuffer.as_mut_ptr());
                if NT_SUCCESS!(LdrGetDllHandle(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(UnicodeDllName), core::ptr::addr_of_mut!(BaseAddress))) {
                    /* This shim dll was already loaded, let's find it */
                    pShimModuleInfo = SeiGetShimModuleInfo(BaseAddress);
                } else if !NT_SUCCESS!(LdrLoadDll(core::ptr::null_mut(), core::ptr::null_mut(), core::ptr::addr_of_mut!(UnicodeDllName), core::ptr::addr_of_mut!(BaseAddress))) {
                    SHIMENG_WARN!(b"Failed to load %wZ for %S\n\0".as_ptr() as *const i8, core::ptr::addr_of_mut!(UnicodeDllName), ShimName);
                    break 'body;
                }
                RtlInitUnicodeString(core::ptr::addr_of_mut!(g_LoadingShimDll), core::ptr::null_mut());
                /* No shim module found (or we just loaded it) */
                if pShimModuleInfo.is_null() {
                    pShimModuleInfo = SeiCreateShimModuleInfo(DllName, BaseAddress);
                    if pShimModuleInfo.is_null() {
                        SHIMENG_FAIL!(b"Failed to allocate ShimInfo for %S\n\0".as_ptr() as *const i8, DllName);
                        break 'body;
                    }
                }

                SHIMENG_INFO!(b"Shim DLL 0x%p \"%wZ\" loaded\n\0".as_ptr() as *const i8, BaseAddress, core::ptr::addr_of_mut!(UnicodeDllName));
                SHIMENG_INFO!(b"Using SHIM \"%S!%S\"\n\0".as_ptr() as *const i8, DllName, ShimName);

                /* Ask this shim what hooks it needs (and pass along the commandline) */
                dwHookCount = 0;
                pHookApi = (((*pShimModuleInfo).pGetHookAPIs).unwrap_unchecked())(AnsiCommandLine.Buffer, ShimName, core::ptr::addr_of_mut!(dwHookCount));
                SHIMENG_INFO!(b"GetHookAPIs returns %d hooks for DLL \"%wZ\" SHIM \"%S\"\n\0".as_ptr() as *const i8, dwHookCount, core::ptr::addr_of_mut!(UnicodeDllName), ShimName);
                if dwHookCount != 0 && !pHookApi.is_null() {
                    pShimInfo = SeiAppendHookInfo(pShimModuleInfo, pHookApi, dwHookCount, ShimName);
                } else {
                    dwHookCount = 0;
                }

                /* If this shim has hooks, create the include / exclude lists */
                if !pShimInfo.is_null() {
                    SeiBuildInclExclList(pdb, ShimTag, pShimInfo);
                }

                if !CommandLine.is_null() && *CommandLine != 0 {
                    RtlFreeAnsiString(core::ptr::addr_of_mut!(AnsiCommandLine));
                }

                dwTotalHooks += dwHookCount;
            }
        }
        n += 1;
    }

    SeiAddInternalHooks(dwTotalHooks);
    SeiCombineHookInfo();
    SeiResolveAPIs();
    PatchNewModules(Peb);

    if ProcessInit != 0 {
        /* Remove the 'LDRP_ENTRY_PROCESSED' flag from entries we modified, so that the loader can continue to process them */
        SeiResetEntryProcessed(Peb);
    }
    g_bShimEngInitialized = (TRUE) as i32;
}
#[no_mangle]
pub unsafe extern "C" fn SeiGetShimData(ProcessImage: PUNICODE_STRING, pShimData: PVOID, pHsdb: *mut HSDB, pQuery: *mut SDBQUERYRESULT) -> BOOL {
    static mut ForbiddenShimmingApps: [_UNICODE_STRING; 3] = [
        RTL_CONSTANT_STRING_STATIC!(RUSTCONV_WSTR_A1F835C798F8),
        RTL_CONSTANT_STRING_STATIC!(RUSTCONV_WSTR_4477EECC4237),
        RTL_CONSTANT_STRING_STATIC!(RUSTCONV_WSTR_95F9B480D535),
    ];
    static mut PathDividerFind: _UNICODE_STRING = RTL_CONSTANT_STRING_STATIC!(RUSTCONV_WSTR_9A6EFE4D86A5);
    let mut ProcessName: _UNICODE_STRING = core::mem::zeroed();
    let mut PathDivider: USHORT = 0;
    let mut hsdb: HSDB = core::ptr::null_mut();
    let mut n: DWORD;

    if !NT_SUCCESS!(RtlFindCharInUnicodeString(RTL_FIND_CHAR_IN_UNICODE_STRING_START_AT_END, ProcessImage, core::ptr::addr_of!(PathDividerFind), &mut PathDivider as *mut USHORT)) {
        PathDivider = 0;
    }

    if PathDivider != 0 {
        PathDivider += core::mem::size_of::<u16>() as USHORT;
    }

    ProcessName.Buffer = (*ProcessImage).Buffer.add((PathDivider as usize) / core::mem::size_of::<u16>());
    ProcessName.Length = (*ProcessImage).Length - PathDivider;
    ProcessName.MaximumLength = (*ProcessImage).MaximumLength - PathDivider;

    n = 0;
    while n < (ARRAYSIZE!(ForbiddenShimmingApps) as DWORD) {
        if RtlEqualUnicodeString(core::ptr::addr_of!(ProcessName), (core::ptr::addr_of!(ForbiddenShimmingApps) as *const _UNICODE_STRING).add(n as usize), (TRUE) as BOOLEAN) != 0 {
            SHIMENG_MSG!(b"Not shimming %wZ\n\0".as_ptr() as *const i8, (core::ptr::addr_of!(ForbiddenShimmingApps) as *const _UNICODE_STRING).add(n as usize));
            return (FALSE) as BOOL;
        }
        n += 1;
    }

    /* We should probably load all db's here, but since we do not support that yet... */
    hsdb = SdbInitDatabase(HID_DOS_PATHS | SDB_DATABASE_MAIN_SHIM, core::ptr::null_mut());
    if !hsdb.is_null() {
        if SdbUnpackAppCompatData(hsdb, (*ProcessImage).Buffer, pShimData, pQuery) != 0 {
            *pHsdb = hsdb;
            return (TRUE) as BOOL;
        }
        SdbReleaseDatabase(hsdb);
    }
    return (FALSE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SE_InstallBeforeInit(ProcessImage: PUNICODE_STRING, pShimData: PVOID) {
    let mut hsdb: HSDB = core::ptr::null_mut();
    let mut QueryResult: SDBQUERYRESULT = core::mem::zeroed();
    let _ = 0;
    SHIMENG_INFO!(b"(%wZ, %p)\n\0".as_ptr() as *const i8, ProcessImage, pShimData);

    if !(SeiGetShimData(ProcessImage, pShimData, core::ptr::addr_of_mut!(hsdb), core::ptr::addr_of_mut!(QueryResult)) != 0) {
        SHIMENG_FAIL!(b"Failed to get shim data\n\0".as_ptr() as *const i8);
        return;
    }

    g_bShimDuringInit = (TRUE) as i32;
    SeiInit((*ProcessImage).Buffer, hsdb, core::ptr::addr_of_mut!(QueryResult), (TRUE) as BOOLEAN);
    g_bShimDuringInit = (FALSE) as i32;

    SdbReleaseDatabase(hsdb);
}

#[no_mangle]
pub unsafe extern "system" fn SE_InstallAfterInit(ProcessImage: PUNICODE_STRING, pShimData: PVOID) {
    NotifyShims(SHIM_NOTIFY_ATTACH, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SE_ProcessDying() {
    SHIMENG_MSG!(b"()\n\0".as_ptr() as *const i8);
    NotifyShims(SHIM_NOTIFY_DETACH, core::ptr::null_mut());
}

#[no_mangle]
pub unsafe extern "system" fn SE_DllLoaded(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    let mut HookModuleInfo: PHOOKMODULEINFO;
    SHIMENG_INFO!(
        b"%sINIT. loading DLL \"%wZ\"\n\0".as_ptr() as *const i8,
        if g_bShimDuringInit != 0 {
            b"\0".as_ptr() as *const i8
        } else {
            b"AFTER \0".as_ptr() as *const i8
        },
        core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
    );

    HookModuleInfo = SeiFindHookModuleInfo(
        core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
        core::ptr::null_mut(),
    );
    if !HookModuleInfo.is_null() {
        (*HookModuleInfo).BaseAddress = (*LdrEntry).DllBase;
        SeiResolveAPI(HookModuleInfo);
    }

    SeiHookImports(LdrEntry);

    NotifyShims(SHIM_REASON_DLL_LOAD, (LdrEntry).cast());
}

#[no_mangle]
pub unsafe extern "system" fn SE_DllUnloaded(LdrEntry: PLDR_DATA_TABLE_ENTRY) {
    SHIMENG_INFO!(b"(%p)\n\0".as_ptr() as *const i8, LdrEntry);

    /* Should we unhook here? */

    NotifyShims(SHIM_REASON_DLL_UNLOAD, LdrEntry as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn SE_IsShimDll(BaseAddress: PVOID) -> BOOL {
    SHIMENG_INFO!(b"(%p)\n\0".as_ptr() as *const i8, BaseAddress);

    return (SeiGetShimModuleInfo(BaseAddress) != core::ptr::null_mut()) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SE_DynamicShim(ProcessImage: LPCWSTR, hsdb: HSDB, pQueryResult: PVOID, Module: LPCSTR, lpdwDynamicToken: LPDWORD) -> BOOL {
    if ((g_bShimEngInitialized) != 0) {
        SHIMENG_MSG!(b"ReactOS HACK(CORE-13283): ShimEng already initialized!\n\0".as_ptr() as *const i8);
        return (TRUE) as BOOL;
    }

    g_bShimDuringInit = (TRUE) as i32;
    SeiInit(ProcessImage, hsdb, pQueryResult as *mut SDBQUERYRESULT, (FALSE) as BOOLEAN);
    g_bShimDuringInit = (FALSE) as i32;

    LdrInitShimEngineDynamic(g_hInstance as PVOID);

    return (TRUE) as BOOL;
}
