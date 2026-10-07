// This file mirrors ldrinit.c's manipulation of C-owned loader globals; reads
// and writes through `static mut` are expected and intentional.
#![allow(static_mut_refs)]

use ntdll_sys::nt::*;
use core::ffi::c_void;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ===========================================================================
// Prelude — ntdllp.h / ntstatus.h / compat_undoc.h / intrin surface absent
// from the ntdll-sys prelude. (See memory feedback_rustconv_header_types_integration.)
// ===========================================================================

// `RTL_CONSTANT_STRING(L"...")` file-scope UNICODE_STRING statics. The
// translator dropped the string-literal initializers (emitting {0,0,null}),
// which null-derefs once the loader hashes BaseDllName.Buffer[0]
// (memory feedback_rustconv_drops_table_default_data). Rebuild each as a
// NUL-terminated [u16; N] backing array + a UNICODE_STRING pointing at it.
// `Length` excludes the NUL; `MaximumLength` includes it (matches
// RTL_CONSTANT_STRING semantics).
macro_rules! rtl_constant_string {
    ($name:ident, $backing:ident, $s:literal) => {
        static $backing: [u16; { $s.len() + 1 }] = {
            let bytes = $s.as_bytes();
            let mut arr = [0u16; { $s.len() + 1 }];
            let mut i = 0;
            while i < bytes.len() {
                // ASCII-only string literals — 1 byte == 1 WCHAR.
                arr[i] = bytes[i] as u16;
                i += 1;
            }
            arr
        };
        #[no_mangle]
        pub static mut $name: _UNICODE_STRING = _UNICODE_STRING {
            Length: ($s.len() * 2) as USHORT,
            MaximumLength: (($s.len() + 1) * 2) as USHORT,
            Buffer: $backing.as_ptr() as *mut u16,
        };
    };
}

#[inline]
fn NT_SUCCESS(status: NTSTATUS) -> bool {
    status >= 0
}

// ntstatus.h cast-macros not in the bindgen surface (verified vs ntstatus.h).
const STATUS_INVALID_IMAGE_FORMAT: NTSTATUS = 0xC000_007Bu32 as NTSTATUS;
const STATUS_CONFLICTING_ADDRESSES: NTSTATUS = 0xC000_0018u32 as NTSTATUS;
const STATUS_DLL_INIT_FAILED: NTSTATUS = 0xC000_0142u32 as NTSTATUS;
const STATUS_APP_INIT_FAILURE: NTSTATUS = 0xC000_0145u32 as NTSTATUS;

// ntdllp.h: DbgPrintEx page-heap notify flag.
const DPH_FLAG_DLL_NOTIFY: ULONG = 0x40;

// compat_undoc.h.
const REACTOS_SHIMDATA_MAGIC: u32 = 0xAC0D_EDAB;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct ReactOS_ShimData {
    pub dwReserved1: [u32; 130],
    pub dwSize: u32,
    pub dwMagic: u32,
    pub dwReserved2: [u32; 242],
    pub dwRosProcessCompatVersion: u32,
}

// COMPAT_GUID_* — process-compat manifest GUIDs (ldrinit.c file-scope #defines).
const COMPAT_GUID_VISTA: GUID = GUID {
    Data1: 0xe2011457,
    Data2: 0x1546,
    Data3: 0x43c5,
    Data4: [0xa5, 0xfe, 0x00, 0x8d, 0xee, 0xe3, 0xd3, 0xf0],
};
const COMPAT_GUID_WIN7: GUID = GUID {
    Data1: 0x35138b9a,
    Data2: 0x5d96,
    Data3: 0x4fbd,
    Data4: [0x8e, 0x2d, 0xa2, 0x44, 0x02, 0x25, 0xf9, 0x3a],
};
const COMPAT_GUID_WIN8: GUID = GUID {
    Data1: 0x4a2f28e3,
    Data2: 0x53b9,
    Data3: 0x4441,
    Data4: [0xba, 0x9c, 0xd6, 0x9d, 0x4a, 0x4a, 0x6e, 0x38],
};
const COMPAT_GUID_WIN81: GUID = GUID {
    Data1: 0x1f676c76,
    Data2: 0x80e1,
    Data3: 0x4239,
    Data4: [0x95, 0xbb, 0x83, 0xd0, 0xf6, 0xd0, 0xda, 0x78],
};
const COMPAT_GUID_WIN10: GUID = GUID {
    Data1: 0x8e0f7a12,
    Data2: 0xbfb3,
    Data3: 0x4fe8,
    Data4: [0xb9, 0xa5, 0x48, 0xfd, 0x50, 0xa1, 0x5a, 0x9a],
};

// ldrp.h: LDRP_TLS_DATA — per-DLL TLS bookkeeping entry.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LDRP_TLS_DATA {
    pub TlsLinks: LIST_ENTRY,
    pub TlsDirectory: IMAGE_TLS_DIRECTORY,
}
pub type _LDRP_TLS_DATA = LDRP_TLS_DATA;

// User-mode KUSER_SHARED_DATA at the fixed VA 0x7FFE0000. NOT a raw-dylib
// import (that breaks the freeldr PE loader — memory
// feedback_win32k_raw_dylib_freeldr_oom); the fixed-VA cast is correct.
#[allow(non_upper_case_globals)]
const SharedUserData: *mut KUSER_SHARED_DATA = 0x7FFE_0000usize as *mut KUSER_SHARED_DATA;

// intrin.h / rtl macros rendered as inline fns.
#[inline]
fn UInt32x32To64(a: u32, b: u32) -> u64 {
    (a as u64) * (b as u64)
}
#[inline]
fn Int64ShrlMod32(val: u64, shift: u32) -> u64 {
    val >> shift
}
#[inline]
fn YieldProcessor() {
    core::hint::spin_loop();
}
#[inline]
unsafe fn RtlZeroMemory(dest: PVOID, length: usize) {
    core::ptr::write_bytes(dest as *mut u8, 0, length);
}
#[inline]
unsafe fn RtlCopyMemory(dest: PVOID, src: *const c_void, length: usize) {
    core::ptr::copy_nonoverlapping(src as *const u8, dest as *mut u8, length);
}

// Interlocked intrinsics (not in the ntdll-sys surface). LONG-keyed.
#[inline]
unsafe fn _InterlockedIncrement(p: *mut LONG) -> LONG {
    (*(p as *const core::sync::atomic::AtomicI32)).fetch_add(1, core::sync::atomic::Ordering::SeqCst) + 1
}
#[inline]
unsafe fn _InterlockedCompareExchange(p: *mut LONG, exch: LONG, comp: LONG) -> LONG {
    match (*(p as *const core::sync::atomic::AtomicI32)).compare_exchange(
        comp,
        exch,
        core::sync::atomic::Ordering::SeqCst,
        core::sync::atomic::Ordering::SeqCst,
    ) {
        Ok(v) => v,
        Err(v) => v,
    }
}
#[inline]
unsafe fn InterlockedCompareExchangePointer(
    p: *mut *mut c_void,
    exch: *mut c_void,
    comp: *mut c_void,
) -> *mut c_void {
    let atom = &*(p as *const core::sync::atomic::AtomicPtr<c_void>);
    match atom.compare_exchange(comp, exch, core::sync::atomic::Ordering::SeqCst, core::sync::atomic::Ordering::SeqCst) {
        Ok(v) => v,
        Err(v) => v,
    }
}

/// `RtlGetProcessHeap` — FORCEINLINE in rtlfuncs.h (no exported symbol).
#[inline(always)]
unsafe fn RtlGetProcessHeap() -> PVOID {
    (*NtCurrentPeb()).ProcessHeap
}

/// True if the UTF-16 image path (len in chars) ends with "smss.exe"
/// (ASCII, case-insensitive). Used only to gate the arm64 FEX rung-0e
/// self-test to the first user process.
#[cfg(target_arch = "aarch64")]
unsafe fn unicode_tail_is_smss(buf: *const u16, len: usize) -> bool {
    const NEEDLE: &[u8] = b"smss.exe";
    if buf.is_null() || len < NEEDLE.len() {
        return false;
    }
    let start = len - NEEDLE.len();
    for (i, &nc) in NEEDLE.iter().enumerate() {
        let c = *buf.add(start + i);
        let lc = if (b'A' as u16..=b'Z' as u16).contains(&c) {
            c + 32
        } else {
            c
        };
        if lc != nc as u16 {
            return false;
        }
    }
    true
}

/// `RtlInitEmptyUnicodeString` — FORCEINLINE in rtlfuncs.h (no exported symbol).
/// Shadows the bindgen extern decl (which has no backing symbol) so callers
/// inline instead of emitting an unresolved reference.
#[inline(always)]
unsafe fn RtlInitEmptyUnicodeString(s: *mut UNICODE_STRING, buffer: PWSTR, buffer_size: USHORT) {
    (*s).Length = 0;
    (*s).MaximumLength = buffer_size;
    (*s).Buffer = buffer;
}

// C runtime / still-C ntdll helper symbols. _snwprintf is a CRT variadic
// (declared, not defined). RtlDoesFileExists_UStr lives in ldrutils.
extern "C" {
    fn _snwprintf(buf: *mut u16, count: usize, fmt: *const u16, ...) -> i32;
}
extern "system" {
    fn RtlDoesFileExists_UStr(FileName: *mut _UNICODE_STRING) -> u8;
}

// Loader helpers DEFINED IN OTHER ntdll files (ldrpe.c / ldrapi.c / ldrutils
// / sxs / rtl). NTAPI. Signatures per ldrp.h + defining TUs.
extern "system" {
    fn LdrpCallInitRoutine(
        EntryPoint: PVOID,
        BaseAddress: PVOID,
        Reason: ULONG,
        Context: PVOID,
    ) -> BOOLEAN;
    fn LdrpClearLoadInProgress() -> ULONG;
    fn LdrpAllocateDataTableEntry(BaseAddress: PVOID) -> PLDR_DATA_TABLE_ENTRY;
    fn LdrpFetchAddressOfEntryPoint(ImageBase: PVOID) -> PVOID;
    fn LdrpInsertMemoryTableEntry(LdrEntry: PLDR_DATA_TABLE_ENTRY);
    fn LdrpSetProtection(ViewBase: PVOID, Restore: BOOLEAN) -> NTSTATUS;
    fn LdrpWalkImportDescriptor(DllPath: LPWSTR, LdrEntry: PLDR_DATA_TABLE_ENTRY) -> NTSTATUS;
    fn LdrpLoadShimEngine(ImageName: PWSTR, ProcessImage: PUNICODE_STRING, pShimData: PVOID,);
    fn AVrfInitializeVerifier() -> NTSTATUS;
    fn RtlpInitDeferredCriticalSection();
    fn RtlpInitializeVectoredExceptionHandling();
    fn RtlInitializeHeapManager();
    fn RtlpInitializeLocaleTable() -> NTSTATUS;
    fn RtlpInitializeActCtx(pOldShimData: *mut PVOID) -> NTSTATUS;
    fn RtlpInitializeKeyedEvent();
    fn RtlpInitializeThreadPooling();
    // Already translated #[no_mangle] in sibling crate files (ldrutils.rs /
    // verifier.rs). Declared here to link cross-file within ntdll-rust.
    fn LdrpCallTlsInitializers(LdrEntry: PLDR_DATA_TABLE_ENTRY, Reason: ULONG);
    fn LdrpInitializeApplicationVerifierPackage(
        KeyHandle: HANDLE,
        Peb: PPEB,
        SystemWide: BOOLEAN,
        ReadAdvancedOptions: BOOLEAN,
    ) -> NTSTATUS;
}

// Data symbols DEFINED IN OTHER ntdll C TUs (rtl heap/timeout/page-heap state
// + the top-level TEB-being-loaded marker). Match the C declaration types.
extern "C" {
    static mut LdrpHeap: PVOID;
    static mut RtlpTimeout: LARGE_INTEGER;
    static mut RtlpTimeoutDisable: BOOLEAN;
    static mut RtlpPageHeapEnabled: BOOLEAN;
    static mut RtlpDphGlobalFlags: ULONG;
    static mut LdrpTopLevelDllBeingLoadedTeb: PTEB;
    // DEFINED #[no_mangle] in ldrutils.rs (same crate); link to those.
    static mut g_ShimsEnabled: BOOLEAN;
    static mut g_pfnSE_ProcessDying: PVOID;
    static mut g_pfnSE_InstallAfterInit: PVOID;
}

// SList-header-width selector — referenced only under `#ifdef _WIN64` in
// ldrinit.c (defined in sdk/lib/rtl/slist.c on every arch). 64-bit gated.
#[cfg(target_pointer_width = "64")]
extern "C" {
    static mut RtlpUse16ByteSLists: BOOLEAN;
}

// ---- file-scope preamble (translated) ----
#[no_mangle]
pub static mut ImageExecOptionsKey: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut Wow64ExecOptionsKey: HANDLE = core::ptr::null_mut();
// RTL_CONSTANT_STRING file-scope statics (ldrinit.c:24-28). Backing arrays
// restore the string literals the translator dropped.
rtl_constant_string!(
    ImageExecOptionsString,
    IMAGE_EXEC_OPTIONS_STRING_BUF,
    "\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Image File Execution Options"
);
rtl_constant_string!(Wow64OptionsString, WOW64_OPTIONS_STRING_BUF, "");
rtl_constant_string!(NtDllString, NTDLL_STRING_BUF, "ntdll.dll");
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
rtl_constant_string!(Wow64DllString, WOW64_DLL_STRING_BUF, "wow64.dll");
#[cfg(target_arch = "aarch64")]
rtl_constant_string!(WowX64DllString, WOWX64_DLL_STRING_BUF, "wowx64.dll");
rtl_constant_string!(Kernel32String, KERNEL32_STRING_BUF, "kernel32.dll");
// C declares LdrpDotLocal `const` but it's only ever read; keep it as the
// same mutable-static shape as the rest for uniformity.
rtl_constant_string!(LdrpDotLocal, LDRP_DOTLOCAL_BUF, ".Local");
#[no_mangle]
pub static mut LdrpInLdrInit: BOOLEAN = 0;
#[no_mangle]
pub static mut LdrpProcessInitialized: LONG = 0;
#[no_mangle]
pub static mut LdrpLoaderLockInit: BOOLEAN = 0;
#[no_mangle]
pub static mut LdrpLdrDatabaseIsSetup: BOOLEAN = 0;
#[no_mangle]
pub static mut LdrpShutdownInProgress: BOOLEAN = 0;
#[no_mangle]
pub static mut LdrpShutdownThreadId: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut LdrpDllValidation: BOOLEAN = 0;
#[no_mangle]
pub static mut LdrpImageEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
#[no_mangle]
pub static mut LdrpTopLevelDllBeingLoaded: PUNICODE_STRING = core::ptr::null_mut();
#[no_mangle]
pub static mut StringBuffer: [WCHAR; 156] = [0; 156];
#[no_mangle]
pub static mut LdrpCurrentDllInitializer: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
#[no_mangle]
pub static mut LdrpNtDllDataTableEntry: PLDR_DATA_TABLE_ENTRY = core::ptr::null_mut();
#[no_mangle]
pub static mut Kernel32ProcessInitPostImportFunction: Option<unsafe extern "system" fn() -> NTSTATUS> = None;
#[no_mangle]
pub static mut Kernel32BaseQueryModuleData: Option<unsafe extern "system" fn(LPSTR, LPSTR, PVOID, PVOID, PVOID) -> BOOL> = None;
#[no_mangle]
pub static mut TlsBitMap: RTL_BITMAP = RTL_BITMAP {
    SizeOfBitMap: 0,
    Buffer: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut TlsExpansionBitMap: RTL_BITMAP = RTL_BITMAP {
    SizeOfBitMap: 0,
    Buffer: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut FlsBitMap: RTL_BITMAP = RTL_BITMAP {
    SizeOfBitMap: 0,
    Buffer: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut LdrpImageHasTls: BOOLEAN = 0;
#[no_mangle]
pub static mut LdrpTlsList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut LdrpNumberOfTlsEntries: ULONG = 0;
#[no_mangle]
pub static mut LdrpNumberOfProcessors: ULONG = 0;
#[no_mangle]
pub static mut NtDllBase: PVOID = core::ptr::null_mut();
pub const LDR_HASH_TABLE_ENTRIES: usize = 32;
#[no_mangle]
pub static mut LdrpHashTable: [LIST_ENTRY; LDR_HASH_TABLE_ENTRIES] = [LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
}; LDR_HASH_TABLE_ENTRIES];
#[no_mangle]
pub static mut LdrpKnownDllObjectDirectory: HANDLE = core::ptr::null_mut();
#[no_mangle]
pub static mut LdrpKnownDllPath: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut LdrpKnownDllPathBuffer: [WCHAR; 128] = [0; 128];
#[no_mangle]
pub static mut LdrpDefaultPath: _UNICODE_STRING = _UNICODE_STRING {
    Length: 0,
    MaximumLength: 0,
    Buffer: core::ptr::null_mut(),
};
#[no_mangle]
pub static mut PebLdr: PEB_LDR_DATA = unsafe { core::mem::zeroed() };

/// THIS ntdll's module database: `&PebLdr` once process init has built it,
/// NULL before (exactly what `PEB->Ldr` reads in an ordinary process, where
/// the two are the same pointer). The loader reads its own data through here
/// and never through `PEB->Ldr`/`PEB->LoaderLock`, because in an x86-64 guest
/// process on arm64 two ntdlls share one PEB: the native arm64 one and the
/// guest's amd64 one, run under emulation by wowx64.dll. Each publishes its
/// list into the PEB at its own process init, the guest's last — so a loader
/// that read the PEB would walk the OTHER architecture's modules (native
/// unwinding would look up libwowx64fex.dll's .pdata in the guest's list and
/// find nothing; the guest's LdrpInit would see a non-NULL `PEB->Ldr` and
/// skip process init entirely). Same for the loader lock: use
/// `LdrpLoaderLock` directly.
#[inline(always)]
pub unsafe fn LdrpLdrData() -> PPEB_LDR_DATA {
    if PebLdr.Initialized == 0 {
        core::ptr::null_mut()
    } else {
        core::ptr::addr_of_mut!(PebLdr)
    }
}
#[no_mangle]
pub static mut LdrpLoaderLockDebug: RTL_CRITICAL_SECTION_DEBUG = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut LdrpLoaderLock: RTL_CRITICAL_SECTION = RTL_CRITICAL_SECTION {
    DebugInfo: unsafe { &mut LdrpLoaderLockDebug as *mut _ },
    LockCount: -1,
    RecursionCount: 0,
    OwningThread: core::ptr::null_mut(),
    LockSemaphore: core::ptr::null_mut(),
    SpinCount: 0,
};
#[no_mangle]
pub static mut FastPebLock: RTL_CRITICAL_SECTION = unsafe { core::mem::zeroed() };
#[no_mangle]
pub static mut ShowSnaps: BOOLEAN = 0;
#[no_mangle]
pub static mut LdrpFatalHardErrorCount: ULONG = 0;
#[no_mangle]
pub static mut LdrpActiveUnloadCount: ULONG = 0;
#[no_mangle]
pub static mut RtlpDisableHeapLookaside: ULONG = 0;
#[no_mangle]
pub static mut RtlpShutdownProcessFlags: ULONG = 0;

#[cfg(target_pointer_width = "64")]
pub const DEFAULT_SECURITY_COOKIE: u64 = 0x00002B992DDFA232;
#[cfg(not(target_pointer_width = "64"))]
pub const DEFAULT_SECURITY_COOKIE: u32 = 0xBB40E64E;

#[no_mangle]
pub unsafe extern "system" fn LdrpInitSecurityCookie(LdrEntry: PLDR_DATA_TABLE_ENTRY) -> PVOID {
    let mut Counter: LARGE_INTEGER = core::mem::zeroed();
    let mut NewCookie: ULONG_PTR;

    let Cookie = LdrpFetchAddressOfSecurityCookie((*LdrEntry).DllBase, (*LdrEntry).SizeOfImage) as PULONG_PTR;

    if !Cookie.is_null() {
        #[cfg(target_pointer_width = "64")]
        let default_cookie: ULONG_PTR = 0x00002B992DDFA232;
        #[cfg(target_pointer_width = "32")]
        let default_cookie: ULONG_PTR = 0xBB40E64E;

        if (*Cookie == default_cookie) || (*Cookie == 0xBB40) {
            NtQueryPerformanceCounter(&mut Counter, core::ptr::null_mut());

            NewCookie = (Counter.__bindgen_anon_1.LowPart as ULONG_PTR) ^ (Counter.__bindgen_anon_1.HighPart as ULONG_PTR);
            NewCookie ^= (*NtCurrentTeb()).ClientId.UniqueProcess as ULONG_PTR;
            NewCookie ^= (*NtCurrentTeb()).ClientId.UniqueThread as ULONG_PTR;

            while core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).SystemTime.High1Time))
                != core::ptr::read_volatile(core::ptr::addr_of!((*SharedUserData).SystemTime.High2Time))
            {
                YieldProcessor();
            }

            NewCookie ^= Int64ShrlMod32(
                UInt32x32To64(
                    (*SharedUserData).TickCountMultiplier,
                    (*SharedUserData).__bindgen_anon_2.__bindgen_anon_1.ReservedTickCountOverlay[0],
                ),
                24,
            ) as ULONG_PTR
                + ((*SharedUserData).TickCountMultiplier as ULONG_PTR
                    * (((*SharedUserData).__bindgen_anon_2.__bindgen_anon_1.ReservedTickCountOverlay[1] as ULONG_PTR) << 8));

            if *Cookie == 0xBB40 {
                NewCookie &= 0xFFFF;
            }

            if (NewCookie == 0) || (NewCookie == *Cookie) {
                NewCookie = *Cookie - 1;
            }

            *Cookie = NewCookie;
        }
    }

    Cookie as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn LdrOpenImageFileOptionsKey(
    SubKey: PUNICODE_STRING,
    Wow64: u8,
    NewKeyHandle: PHANDLE,
) -> NTSTATUS {
    let RootKeyLocation: *mut HANDLE = if Wow64 != 0 {
        core::ptr::addr_of_mut!(Wow64ExecOptionsKey)
    } else {
        core::ptr::addr_of_mut!(ImageExecOptionsKey)
    };

    let mut RootKey: HANDLE = *RootKeyLocation;

    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.ObjectName = if Wow64 != 0 {
        core::ptr::addr_of_mut!(Wow64OptionsString)
    } else {
        core::ptr::addr_of_mut!(ImageExecOptionsString)
    };
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut Status = ZwOpenKey(
        core::ptr::addr_of_mut!(RootKey),
        KEY_ENUMERATE_SUB_KEYS,
        core::ptr::addr_of_mut!(ObjectAttributes),
    );

    if NT_SUCCESS(Status) {
        if !InterlockedCompareExchangePointer(
            RootKeyLocation as *mut *mut core::ffi::c_void,
            RootKey as *mut core::ffi::c_void,
            core::ptr::null_mut(),
        )
        .is_null()
        {
            NtClose(RootKey);
            RootKey = *RootKeyLocation;
        }

        let mut SubKeyString: _UNICODE_STRING = *SubKey;
        let mut p1: *mut u16 = (SubKeyString.Buffer as usize + SubKeyString.Length as usize) as *mut u16;
        while SubKeyString.Length != 0 {
            if *p1.offset(-1) == b'\\' as u16 {
                break;
            }
            p1 = p1.offset(-1);
            SubKeyString.Length -= core::mem::size_of::<u16>() as u16;
        }
        SubKeyString.Buffer = p1;
        SubKeyString.Length = (*SubKey).Length - SubKeyString.Length;

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
        ObjectAttributes.RootDirectory = RootKey;
        ObjectAttributes.ObjectName = core::ptr::addr_of_mut!(SubKeyString);
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = ZwOpenKey(NewKeyHandle, GENERIC_READ, core::ptr::addr_of_mut!(ObjectAttributes));
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrQueryImageFileKeyOption(
    KeyHandle: HANDLE,
    ValueName: PCWSTR,
    Type_: ULONG,
    Buffer: PVOID,
    mut BufferSize: ULONG,
    ReturnedLength: PULONG,
) -> NTSTATUS {
    let mut KeyInfo: [ULONG; 256] = [0; 256];
    let mut ValueNameString: _UNICODE_STRING = core::mem::zeroed();
    let mut IntegerString: _UNICODE_STRING = core::mem::zeroed();
    let KeyInfoSize: ULONG;
    let mut ResultSize: ULONG = 0;
    let mut KeyValueInformation: PKEY_VALUE_PARTIAL_INFORMATION =
        KeyInfo.as_mut_ptr() as PKEY_VALUE_PARTIAL_INFORMATION;
    let mut FreeHeap: BOOLEAN = 0u8;
    let mut Status: NTSTATUS;

    Status = RtlInitUnicodeStringEx(&mut ValueNameString, ValueName);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = ZwQueryValueKey(
        KeyHandle,
        &mut ValueNameString,
        KeyValuePartialInformation,
        KeyValueInformation as PVOID,
        core::mem::size_of_val(&KeyInfo) as ULONG,
        &mut ResultSize,
    );

    if Status == STATUS_BUFFER_OVERFLOW {
        KeyInfoSize = core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG
            + (*KeyValueInformation).DataLength;
        KeyValueInformation =
            RtlAllocateHeap(RtlGetProcessHeap(), 0, KeyInfoSize as _) as PKEY_VALUE_PARTIAL_INFORMATION;
        if !KeyValueInformation.is_null() {
            Status = ZwQueryValueKey(
                KeyHandle,
                &mut ValueNameString,
                KeyValuePartialInformation,
                KeyValueInformation as PVOID,
                KeyInfoSize,
                &mut ResultSize,
            );
            FreeHeap = 1u8;
        } else {
            Status = STATUS_NO_MEMORY;
        }
    }

    if NT_SUCCESS(Status) {
        if (*KeyValueInformation).Type == REG_BINARY {
            if !Buffer.is_null() && (*KeyValueInformation).DataLength <= BufferSize {
                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*KeyValueInformation).Data) as *const u8,
                    Buffer as *mut u8,
                    (*KeyValueInformation).DataLength as usize,
                );
            } else {
                Status = STATUS_BUFFER_OVERFLOW;
            }

            if !ReturnedLength.is_null() {
                *ReturnedLength = (*KeyValueInformation).DataLength;
            }
        } else if (*KeyValueInformation).Type == REG_DWORD {
            if (*KeyValueInformation).Type != Type_ {
                Status = STATUS_OBJECT_TYPE_MISMATCH;
            } else {
                if !Buffer.is_null()
                    && BufferSize == core::mem::size_of::<ULONG>() as ULONG
                    && (*KeyValueInformation).DataLength <= BufferSize
                {
                    core::ptr::copy_nonoverlapping(
                        core::ptr::addr_of!((*KeyValueInformation).Data) as *const u8,
                        Buffer as *mut u8,
                        (*KeyValueInformation).DataLength as usize,
                    );
                } else {
                    Status = STATUS_BUFFER_OVERFLOW;
                }

                if !ReturnedLength.is_null() {
                    *ReturnedLength = (*KeyValueInformation).DataLength;
                }
            }
        } else if (*KeyValueInformation).Type != REG_SZ {
            Status = STATUS_OBJECT_TYPE_MISMATCH;
        } else {
            if Type_ == REG_DWORD {
                if BufferSize != core::mem::size_of::<ULONG>() as ULONG {
                    BufferSize = 0;
                    Status = STATUS_INFO_LENGTH_MISMATCH;
                } else {
                    IntegerString.Buffer = core::ptr::addr_of!((*KeyValueInformation).Data) as PWSTR;
                    IntegerString.Length =
                        ((*KeyValueInformation).DataLength as USHORT).wrapping_sub(core::mem::size_of::<WCHAR>() as USHORT);
                    IntegerString.MaximumLength = (*KeyValueInformation).DataLength as USHORT;
                    Status = RtlUnicodeStringToInteger(&mut IntegerString, 0, Buffer as PULONG);
                }
            } else {
                if (*KeyValueInformation).DataLength > BufferSize {
                    Status = STATUS_BUFFER_OVERFLOW;
                } else {
                    BufferSize = (*KeyValueInformation).DataLength;
                }

                core::ptr::copy_nonoverlapping(
                    core::ptr::addr_of!((*KeyValueInformation).Data) as *const u8,
                    Buffer as *mut u8,
                    BufferSize as usize,
                );
            }

            if !ReturnedLength.is_null() {
                *ReturnedLength = (*KeyValueInformation).DataLength;
            }
        }
    }

    if FreeHeap != 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, KeyValueInformation as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrQueryImageFileExecutionOptionsEx(
    SubKey: PUNICODE_STRING,
    ValueName: PCWSTR,
    Type_: ULONG,
    Buffer: PVOID,
    BufferSize: ULONG,
    ReturnedLength: PULONG,
    Wow64: u8,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();

    let mut Status: NTSTATUS = LdrOpenImageFileOptionsKey(SubKey, Wow64, &mut KeyHandle);

    if NT_SUCCESS(Status) {
        Status = LdrQueryImageFileKeyOption(
            KeyHandle,
            ValueName,
            Type_,
            Buffer,
            BufferSize,
            ReturnedLength,
        );

        NtClose(KeyHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrQueryImageFileExecutionOptions(
    SubKey: PUNICODE_STRING,
    ValueName: PCWSTR,
    Type_: ULONG,
    Buffer: PVOID,
    BufferSize: ULONG,
    ReturnedLength: PULONG,
) -> NTSTATUS {
    LdrQueryImageFileExecutionOptionsEx(
        SubKey,
        ValueName,
        Type_,
        Buffer,
        BufferSize,
        ReturnedLength,
        0u8,
    )
}

#[no_mangle]
pub unsafe extern "system" fn LdrpEnsureLoaderLockIsHeld() {
}
#[no_mangle]
pub unsafe extern "system" fn LdrpFetchAddressOfSecurityCookie(BaseAddress: PVOID, SizeOfImage: ULONG) -> PVOID {
    let mut DirSize: ULONG = 0;
    let mut Cookie: PVOID = core::ptr::null_mut();

    if RtlImageNtHeader(BaseAddress).is_null() {
        return core::ptr::null_mut();
    }

    let ConfigDir = RtlImageDirectoryEntryToData(
        BaseAddress,
        1u8,
        IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG as u16,
        &mut DirSize,
    ) as PIMAGE_LOAD_CONFIG_DIRECTORY;

    if ConfigDir.is_null()
        || (DirSize != 64 && (*ConfigDir).Size != DirSize)
        || ((*ConfigDir).Size < 0x48)
    {
        return core::ptr::null_mut();
    }

    Cookie = (*ConfigDir).SecurityCookie as PVOID;

    if (Cookie as PCHAR) <= (BaseAddress as PCHAR)
        || (Cookie as PCHAR) >= (BaseAddress as PCHAR).add(SizeOfImage as usize)
    {
        Cookie = core::ptr::null_mut();
    }

    Cookie
}

#[no_mangle]
pub unsafe extern "system" fn LdrpInitializeThread(_Context: PCONTEXT) {
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED =
        core::mem::zeroed();
    let Status: NTSTATUS;
    let mut EntryPoint: PVOID;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(LdrpLoaderLock));

    Status = RtlAllocateActivationContextStack(core::ptr::addr_of_mut!(
        (*NtCurrentTeb()).ActivationContextStackPointer
    ));
    if !NT_SUCCESS(Status) {
        // Warning logged
    }

    'exit: {
        if LdrpShutdownInProgress != 0 {
            break 'exit;
        }

        LdrpAllocateTls();

        ListHead = core::ptr::addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InMemoryOrderModuleList);
        NextEntry = (*ListHead).Flink;
        while NextEntry != ListHead {
            LdrEntry = (NextEntry as usize
                - core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InMemoryOrderLinks))
                as PLDR_DATA_TABLE_ENTRY;

            if (*Peb).ImageBaseAddress != (*LdrEntry).DllBase {
                if ((*LdrEntry).Flags & LDRP_DONT_CALL_FOR_THREADS) == 0 {
                    EntryPoint = (*LdrEntry).EntryPoint;

                    if !EntryPoint.is_null()
                        && ((*LdrEntry).Flags & LDRP_PROCESS_ATTACH_CALLED) != 0
                        && ((*LdrEntry).Flags & LDRP_IMAGE_DLL) != 0
                    {
                        ActCtx.Size = core::mem::size_of::<
                            RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
                        >() as SIZE_T;
                        ActCtx.Format = 1;
                        core::ptr::write_bytes(
                            core::ptr::addr_of_mut!(ActCtx.Frame) as *mut u8,
                            0,
                            core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
                        );

                        RtlActivateActivationContextUnsafeFast(
                            core::ptr::addr_of_mut!(ActCtx),
                            (*LdrEntry).EntryPointActivationContext,
                        );

                        if (*LdrEntry).TlsIndex != 0 {
                            if LdrpShutdownInProgress == 0 {
                                LdrpCallTlsInitializers(LdrEntry, DLL_THREAD_ATTACH);
                            }
                        }

                        if LdrpShutdownInProgress == 0 {
                            LdrpCallInitRoutine(
                                (*LdrEntry).EntryPoint,
                                (*LdrEntry).DllBase,
                                DLL_THREAD_ATTACH,
                                core::ptr::null_mut(),
                            );
                        }

                        RtlDeactivateActivationContextUnsafeFast(core::ptr::addr_of_mut!(ActCtx));
                    }
                }
            }

            NextEntry = (*NextEntry).Flink;
        }

        if LdrpImageHasTls != 0 && LdrpShutdownInProgress == 0 {
            ActCtx.Size = core::mem::size_of::<
                RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
            >() as SIZE_T;
            ActCtx.Format = 1;
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!(ActCtx.Frame) as *mut u8,
                0,
                core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
            );

            RtlActivateActivationContextUnsafeFast(
                core::ptr::addr_of_mut!(ActCtx),
                (*LdrpImageEntry).EntryPointActivationContext,
            );

            LdrpCallTlsInitializers(LdrpImageEntry, DLL_THREAD_ATTACH);

            RtlDeactivateActivationContextUnsafeFast(core::ptr::addr_of_mut!(ActCtx));
        }
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(LdrpLoaderLock));
}

#[no_mangle]
pub unsafe extern "system" fn LdrpRunInitializeRoutines(Context: PCONTEXT) -> NTSTATUS {
    let mut LocalArray: [PLDR_DATA_TABLE_ENTRY; 16] = [core::ptr::null_mut(); 16];
    let mut ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;
    let LdrRootEntry: *mut PLDR_DATA_TABLE_ENTRY;
    let mut OldInitializer: PLDR_DATA_TABLE_ENTRY;
    let mut EntryPoint: PVOID;
    let Count: ULONG;
    let mut i: ULONG;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED = core::mem::zeroed();
    let mut BreakOnDllLoad: ULONG;
    let OldTldTeb: PTEB;
    let mut DllStatus: BOOLEAN;

    LdrpEnsureLoaderLockIsHeld();

    Count = LdrpClearLoadInProgress();
    if Count != 0 {
        if Count > 16 {
            LdrRootEntry = RtlAllocateHeap(
                LdrpHeap,
                0,
                ((Count as usize) * core::mem::size_of::<PLDR_DATA_TABLE_ENTRY>()) as _,
            ) as *mut PLDR_DATA_TABLE_ENTRY;
            if LdrRootEntry.is_null() {
                return STATUS_NO_MEMORY;
            }
        } else {
            LdrRootEntry = LocalArray.as_mut_ptr();
        }
    } else {
        LdrRootEntry = core::ptr::null_mut();
    }

    ListHead = core::ptr::addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InInitializationOrderModuleList);
    NextEntry = (*ListHead).Flink;
    i = 0;
    while NextEntry != ListHead {
        LdrEntry = (NextEntry as usize
            - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks))
            as PLDR_DATA_TABLE_ENTRY;

        if !LdrRootEntry.is_null() {
            if ((*LdrEntry).Flags & LDRP_ENTRY_PROCESSED) == 0 {
                LdrpInitSecurityCookie(LdrEntry);

                if !(*LdrEntry).EntryPoint.is_null() {
                    *LdrRootEntry.add(i as usize) = LdrEntry;
                    i += 1;
                }
            }
        }

        (*LdrEntry).Flags |= LDRP_ENTRY_PROCESSED;
        NextEntry = (*NextEntry).Flink;
    }

    Status = STATUS_SUCCESS;

    if !Context.is_null() {
        if let Some(func) = Kernel32ProcessInitPostImportFunction {
            Status = func();
            if !NT_SUCCESS(Status) {
                DbgPrint(b"LDR: LdrpRunInitializeRoutines - Failed running kernel32 post-import function, Status=0x%08lx\n\0".as_ptr() as *const i8, Status);
            }
        }
        Kernel32ProcessInitPostImportFunction = None;
    }

    if LdrRootEntry.is_null() {
        return Status;
    }

    OldTldTeb = LdrpTopLevelDllBeingLoadedTeb;
    LdrpTopLevelDllBeingLoadedTeb = NtCurrentTeb();

    i = 0;
    'quickie: {
        while i < Count {
            LdrEntry = *LdrRootEntry.add(i as usize);

            i += 1;

            EntryPoint = (*LdrEntry).EntryPoint as PVOID;

            BreakOnDllLoad = 0;
            if (*Peb).BeingDebugged != 0 || (*Peb).ReadImageFileExecOptions != 0 {
                Status = LdrQueryImageFileExecutionOptions(
                    core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                    w!("BreakOnDllLoad").as_ptr() as PCWSTR,
                    REG_DWORD,
                    core::ptr::addr_of_mut!(BreakOnDllLoad) as PVOID,
                    core::mem::size_of::<ULONG>() as ULONG,
                    core::ptr::null_mut(),
                );
                if !NT_SUCCESS(Status) {
                    BreakOnDllLoad = 0;
                }

                Status = STATUS_SUCCESS;
            }

            if BreakOnDllLoad != 0 {
                DbgBreakPoint();
            }

            if !EntryPoint.is_null() {
                OldInitializer = LdrpCurrentDllInitializer;
                LdrpCurrentDllInitializer = LdrEntry;

                ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED>() as SIZE_T;
                ActCtx.Format = 1;
                core::ptr::write_bytes(
                    core::ptr::addr_of_mut!(ActCtx.Frame) as *mut u8,
                    0,
                    core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
                );

                RtlActivateActivationContextUnsafeFast(
                    core::ptr::addr_of_mut!(ActCtx),
                    (*LdrEntry).EntryPointActivationContext,
                );

                // _SEH2_TRY body
                if (*LdrEntry).TlsIndex != 0 && !Context.is_null() {
                    LdrpCallTlsInitializers(LdrEntry, DLL_PROCESS_ATTACH);
                }

                DllStatus = LdrpCallInitRoutine(
                    EntryPoint,
                    (*LdrEntry).DllBase,
                    DLL_PROCESS_ATTACH,
                    Context as PVOID,
                );

                RtlDeactivateActivationContextUnsafeFast(core::ptr::addr_of_mut!(ActCtx));

                LdrpCurrentDllInitializer = OldInitializer;

                (*LdrEntry).Flags |= LDRP_PROCESS_ATTACH_CALLED;

                if DllStatus == 0 {
                    /* Name the culprit: this aborts the whole process with
                     * STATUS_DLL_INIT_FAILED, and "FAILED PID=n" alone tells you
                     * nothing about WHICH DllMain returned FALSE. */
                    DbgPrint(
                        b"[LDR] DllMain(%wZ) returned FALSE -> STATUS_DLL_INIT_FAILED PID=%lu\n\0"
                            .as_ptr() as *const i8,
                        core::ptr::addr_of_mut!((*LdrEntry).BaseDllName),
                        (*NtCurrentTeb()).ClientId.UniqueProcess as ULONG,
                    );
                    Status = STATUS_DLL_INIT_FAILED;
                    break 'quickie;
                }
            }
        }

        ListHead = core::ptr::addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InInitializationOrderModuleList);
        NextEntry = (*NextEntry).Flink;
        while NextEntry != ListHead {
            LdrEntry = (NextEntry as usize
                - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks))
                as PLDR_DATA_TABLE_ENTRY;
            let _ = LdrEntry;
            NextEntry = (*NextEntry).Flink;
        }

        if LdrpImageHasTls != 0 && !Context.is_null() {
            ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED>() as SIZE_T;
            ActCtx.Format = 1;
            core::ptr::write_bytes(
                core::ptr::addr_of_mut!(ActCtx.Frame) as *mut u8,
                0,
                core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
            );

            RtlActivateActivationContextUnsafeFast(
                core::ptr::addr_of_mut!(ActCtx),
                (*LdrpImageEntry).EntryPointActivationContext,
            );

            // _SEH2_TRY body
            LdrpCallTlsInitializers(LdrpImageEntry, DLL_PROCESS_ATTACH);

            RtlDeactivateActivationContextUnsafeFast(core::ptr::addr_of_mut!(ActCtx));
        }
    }

    LdrpTopLevelDllBeingLoadedTeb = OldTldTeb;

    if LdrRootEntry != LocalArray.as_mut_ptr() {
        RtlFreeHeap(LdrpHeap, 0, LdrRootEntry as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrShutdownProcess() -> NTSTATUS {
    let Peb: PPEB = NtCurrentPeb();
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED =
        core::mem::zeroed();
    let mut EntryPoint: PVOID;

    if LdrpShutdownInProgress != 0 {
        return STATUS_SUCCESS;
    }

    if g_ShimsEnabled != 0 {
        let SE_ProcessDying: extern "system" fn() =
            core::mem::transmute(RtlDecodeSystemPointer(g_pfnSE_ProcessDying));
        SE_ProcessDying();
    }

    if ShowSnaps != 0 {
        // DPRINT1
    }

    LdrpShutdownThreadId = (*NtCurrentTeb()).RealClientId.UniqueThread;
    LdrpShutdownInProgress = 1u8;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(LdrpLoaderLock));

    if (*SharedUserData).TraceLogging != 0 {
        // FIXME
    }

    ListHead = core::ptr::addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InInitializationOrderModuleList);
    NextEntry = (*ListHead).Blink;
    while NextEntry != ListHead {
        LdrEntry = (NextEntry as usize
            - core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks))
            as PLDR_DATA_TABLE_ENTRY;
        NextEntry = (*NextEntry).Blink;

        if (*Peb).ImageBaseAddress != (*LdrEntry).DllBase {
            EntryPoint = (*LdrEntry).EntryPoint;

            if !EntryPoint.is_null()
                && ((*LdrEntry).Flags & LDRP_PROCESS_ATTACH_CALLED) != 0
                && (*LdrEntry).Flags != 0
            {
                ActCtx.Size = core::mem::size_of_val(&ActCtx) as SIZE_T;
                ActCtx.Format = 1;
                RtlZeroMemory(
                    core::ptr::addr_of_mut!(ActCtx.Frame) as PVOID,
                    core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
                );

                RtlActivateActivationContextUnsafeFast(
                    core::ptr::addr_of_mut!(ActCtx),
                    (*LdrEntry).EntryPointActivationContext,
                );

                if (*LdrEntry).TlsIndex != 0 {
                    LdrpCallTlsInitializers(LdrEntry, DLL_PROCESS_DETACH);
                }

                LdrpCallInitRoutine(
                    EntryPoint,
                    (*LdrEntry).DllBase,
                    DLL_PROCESS_DETACH,
                    1usize as PVOID,
                );

                RtlDeactivateActivationContextUnsafeFast(core::ptr::addr_of_mut!(ActCtx));
            }
        }
    }

    if LdrpImageHasTls != 0 {
        ActCtx.Size = core::mem::size_of_val(&ActCtx) as SIZE_T;
        ActCtx.Format = 1;
        RtlZeroMemory(
            core::ptr::addr_of_mut!(ActCtx.Frame) as PVOID,
            core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
        );

        RtlActivateActivationContextUnsafeFast(
            core::ptr::addr_of_mut!(ActCtx),
            (*LdrpImageEntry).EntryPointActivationContext,
        );

        LdrpCallTlsInitializers(LdrpImageEntry, DLL_PROCESS_DETACH);

        RtlDeactivateActivationContextUnsafeFast(core::ptr::addr_of_mut!(ActCtx));
    }

    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(LdrpLoaderLock));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LdrShutdownThread() -> NTSTATUS {
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    let Teb: PTEB = NtCurrentTeb();
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED =
        core::mem::zeroed();
    let mut EntryPoint: PVOID;

    RtlEnterCriticalSection(core::ptr::addr_of_mut!(LdrpLoaderLock));

    ListHead = core::ptr::addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InInitializationOrderModuleList);
    NextEntry = (*ListHead).Blink;
    while NextEntry != ListHead {
        LdrEntry = (NextEntry as *mut u8).offset(
            -(core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks) as isize),
        ) as PLDR_DATA_TABLE_ENTRY;
        NextEntry = (*NextEntry).Blink;

        if (*Peb).ImageBaseAddress != (*LdrEntry).DllBase {
            if ((*LdrEntry).Flags & LDRP_DONT_CALL_FOR_THREADS) == 0
                && ((*LdrEntry).Flags & LDRP_PROCESS_ATTACH_CALLED) != 0
                && ((*LdrEntry).Flags & LDRP_IMAGE_DLL) != 0
            {
                EntryPoint = (*LdrEntry).EntryPoint;

                if !EntryPoint.is_null() {
                    ActCtx.Size = core::mem::size_of::<
                        RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
                    >() as SIZE_T;
                    ActCtx.Format = 1;
                    RtlZeroMemory(
                        core::ptr::addr_of_mut!(ActCtx.Frame) as PVOID,
                        core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
                    );

                    RtlActivateActivationContextUnsafeFast(
                        core::ptr::addr_of_mut!(ActCtx),
                        (*LdrEntry).EntryPointActivationContext,
                    );

                    if (*LdrEntry).TlsIndex != 0 {
                        if LdrpShutdownInProgress == 0 {
                            LdrpCallTlsInitializers(LdrEntry, DLL_THREAD_DETACH);
                        }
                    }

                    if LdrpShutdownInProgress == 0 {
                        LdrpCallInitRoutine(
                            EntryPoint,
                            (*LdrEntry).DllBase,
                            DLL_THREAD_DETACH,
                            core::ptr::null_mut(),
                        );
                    }

                    RtlDeactivateActivationContextUnsafeFast(core::ptr::addr_of_mut!(ActCtx));
                }
            }
        }
    }

    if LdrpImageHasTls != 0 {
        ActCtx.Size = core::mem::size_of::<
            RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED,
        >() as SIZE_T;
        ActCtx.Format = 1;
        RtlZeroMemory(
            core::ptr::addr_of_mut!(ActCtx.Frame) as PVOID,
            core::mem::size_of::<RTL_ACTIVATION_CONTEXT_STACK_FRAME>(),
        );

        RtlActivateActivationContextUnsafeFast(
            core::ptr::addr_of_mut!(ActCtx),
            (*LdrpImageEntry).EntryPointActivationContext,
        );

        LdrpCallTlsInitializers(LdrpImageEntry, DLL_THREAD_DETACH);

        RtlDeactivateActivationContextUnsafeFast(core::ptr::addr_of_mut!(ActCtx));
    }

    LdrpFreeTls();
    RtlLeaveCriticalSection(core::ptr::addr_of_mut!(LdrpLoaderLock));

    if !(*Teb).TlsExpansionSlots.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Teb).TlsExpansionSlots as PVOID);
    }

    if !(*Teb).FlsData.is_null() {
        let mut n: ULONG;
        let FlsHighIndex: ULONG;
        let pFlsData: PRTL_FLS_DATA;
        let mut lpCallback: PFLS_CALLBACK_FUNCTION;

        pFlsData = (*Teb).FlsData as PRTL_FLS_DATA;

        RtlAcquirePebLock();
        FlsHighIndex = (*(*NtCurrentTeb()).ProcessEnvironmentBlock).FlsHighIndex;
        RemoveEntryList(core::ptr::addr_of_mut!((*pFlsData).ListEntry));
        RtlReleasePebLock();

        n = 1;
        while n <= FlsHighIndex {
            let raw_cb: PVOID = (*(*NtCurrentTeb()).ProcessEnvironmentBlock)
                .FlsCallback
                .offset(n as isize)
                .read();
            lpCallback = core::mem::transmute::<PVOID, PFLS_CALLBACK_FUNCTION>(raw_cb);
            if lpCallback.is_some() && !(*core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(n as usize)).is_null() {
                (lpCallback.unwrap())(*core::ptr::addr_of!((*pFlsData).Data).cast::<PVOID>().add(n as usize));
            }
            n += 1;
        }

        RtlFreeHeap(RtlGetProcessHeap(), 0, pFlsData as PVOID);
        (*Teb).FlsData = core::ptr::null_mut();
    }

    if (*Teb).HasFiberData != 0 {
        RtlFreeHeap(RtlGetProcessHeap(), 0, (*Teb).NtTib.__bindgen_anon_1.FiberData);
        (*Teb).NtTib.__bindgen_anon_1.FiberData = core::ptr::null_mut();
    }

    RtlFreeThreadActivationContextStack();

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn LdrpInitializeTls() -> NTSTATUS {
    let mut NextEntry: PLIST_ENTRY;
    let _ListHead: PLIST_ENTRY;
    let mut LdrEntry: PLDR_DATA_TABLE_ENTRY;
    let mut TlsDirectory: PIMAGE_TLS_DIRECTORY;
    let mut TlsData: *mut LDRP_TLS_DATA;
    let mut Size: ULONG = 0;

    InitializeListHead(core::ptr::addr_of_mut!(LdrpTlsList));

    let LdrData = crate::ldr::ldrinit::LdrpLdrData();
    let ListHead: PLIST_ENTRY = core::ptr::addr_of_mut!((*LdrData).InLoadOrderModuleList);
    NextEntry = (*ListHead).Flink;

    while ListHead != NextEntry {
        LdrEntry = (NextEntry as usize - core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks)) as PLDR_DATA_TABLE_ENTRY;
        NextEntry = (*NextEntry).Flink;

        /* A 32-bit module's TLS directory is IMAGE_TLS_DIRECTORY32 — 4-byte
         * fields, 24 bytes — not the native shape this code is compiled
         * against (8-byte fields, 40 bytes). Reading one here yields a garbage
         * AddressOfIndex, and the store through it below is a wild write.
         *
         * This is not hypothetical: the process image of a WoW64 process IS a
         * 32-bit PE and it sits in this (64-bit) module list. Every guest we
         * ran before M11 was hand-written and no-CRT, so none had a TLS
         * directory at all and this loop skipped them on the null check. The
         * first CRT-linked guest (testgui.exe) has one, and it AV'd here.
         *
         * An x86-64 guest on arm64 (wowx64) is the same hazard in a form the
         * old PE32-magic test could not see: its directory HAS the native
         * shape, so it would have been registered and its callbacks later run
         * natively — x86 bytes executed as arm64. Hence the MACHINE test.
         *
         * Guest modules' TLS belongs to the guest's own ntdll, under
         * emulation. The native loader must not touch them. (Cf.
         * ImageNeedsRelocation in LdrpInitializeProcess, which dodges the
         * 64-bit-header-over-a-32-bit-image trap for OptionalHeader.ImageBase.) */
        if crate::ldr::ldrutils::LdrpIsForeignMachineImage((*LdrEntry).DllBase) {
            continue;
        }

        TlsDirectory = RtlImageDirectoryEntryToData(
            (*LdrEntry).DllBase,
            1u8,
            IMAGE_DIRECTORY_ENTRY_TLS as u16,
            core::ptr::addr_of_mut!(Size),
        ) as PIMAGE_TLS_DIRECTORY;

        if TlsDirectory.is_null() {
            continue;
        }

        if LdrpImageHasTls == 0 {
            LdrpImageHasTls = 1;
        }

        TlsData = RtlAllocateHeap(RtlGetProcessHeap(), 0, core::mem::size_of::<LDRP_TLS_DATA>() as SIZE_T) as *mut LDRP_TLS_DATA;
        if TlsData.is_null() {
            return STATUS_NO_MEMORY;
        }

        (*LdrEntry).LoadCount = 0xFFFFu16;
        (*LdrEntry).TlsIndex = -1i32 as u16;

        (*TlsData).TlsDirectory = *TlsDirectory;
        InsertTailList(core::ptr::addr_of_mut!(LdrpTlsList), core::ptr::addr_of_mut!((*TlsData).TlsLinks));

        *((*TlsData).TlsDirectory.AddressOfIndex as *mut LONG) = LdrpNumberOfTlsEntries as LONG;
        (*TlsData).TlsDirectory.Characteristics = LdrpNumberOfTlsEntries;
        LdrpNumberOfTlsEntries += 1;
    }

    LdrpAllocateTls()
}
#[no_mangle]
pub unsafe extern "system" fn LdrpAllocateTls() -> NTSTATUS {
    let Teb: PTEB = NtCurrentTeb();
    let mut NextEntry: PLIST_ENTRY;
    let ListHead: PLIST_ENTRY;
    let mut TlsData: *mut LDRP_TLS_DATA;
    let mut TlsDataSize: SIZE_T;
    let TlsVector: *mut PVOID;

    if LdrpNumberOfTlsEntries == 0 {
        return STATUS_SUCCESS;
    }

    TlsVector = RtlAllocateHeap(
        RtlGetProcessHeap(),
        0,
        (LdrpNumberOfTlsEntries as SIZE_T) * core::mem::size_of::<PVOID>() as SIZE_T,
    ) as *mut PVOID;
    if TlsVector.is_null() {
        return STATUS_NO_MEMORY;
    }
    (*Teb).ThreadLocalStoragePointer = TlsVector as PVOID;

    ListHead = core::ptr::addr_of_mut!(LdrpTlsList);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        TlsData = (NextEntry as *mut u8)
            .offset(-(core::mem::offset_of!(LDRP_TLS_DATA, TlsLinks) as isize))
            as *mut LDRP_TLS_DATA;
        NextEntry = (*NextEntry).Flink;

        TlsDataSize = ((*TlsData).TlsDirectory.EndAddressOfRawData
            - (*TlsData).TlsDirectory.StartAddressOfRawData) as SIZE_T;
        let idx = (*TlsData).TlsDirectory.Characteristics as isize;
        *TlsVector.offset(idx) = RtlAllocateHeap(RtlGetProcessHeap(), 0, TlsDataSize as _);
        if (*TlsVector.offset(idx)).is_null() {
            return STATUS_NO_MEMORY;
        }

        RtlCopyMemory(
            *TlsVector.offset(idx),
            (*TlsData).TlsDirectory.StartAddressOfRawData as *const c_void,
            TlsDataSize as usize,
        );
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn LdrpFreeTls() {
    let Teb: PTEB = NtCurrentTeb();

    let TlsVector = (*Teb).ThreadLocalStoragePointer as *mut *mut core::ffi::c_void;
    if TlsVector.is_null() {
        return;
    }

    let ListHead: PLIST_ENTRY = core::ptr::addr_of_mut!(LdrpTlsList);
    let mut NextEntry: PLIST_ENTRY = (*ListHead).Flink;
    while NextEntry != ListHead {
        let TlsData = (NextEntry as *mut u8).offset(-(core::mem::offset_of!(_LDRP_TLS_DATA, TlsLinks) as isize)) as *mut _LDRP_TLS_DATA;
        NextEntry = (*NextEntry).Flink;

        let index = (*TlsData).TlsDirectory.Characteristics as usize;
        if !(*TlsVector.add(index)).is_null() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, *TlsVector.add(index));
        }
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, TlsVector as *mut core::ffi::c_void);
}
#[no_mangle]
pub unsafe extern "system" fn LdrpInitializeExecutionOptions(
    ImagePathName: PUNICODE_STRING,
    Peb: PPEB,
    OptionsKey: PHANDLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ExecuteOptions: ULONG = 0;
    let mut MinimumStackCommit: ULONG = 0;
    let mut GlobalFlag: ULONG = 0;

    if OptionsKey.is_null() {
        return STATUS_INVALID_HANDLE;
    }

    *OptionsKey = core::ptr::null_mut();

    Status = LdrOpenImageFileOptionsKey(ImagePathName, 0, &mut KeyHandle);

    if NT_SUCCESS(Status) {
        *OptionsKey = KeyHandle;
    }

    if !KeyHandle.is_null() {
        Status = LdrQueryImageFileKeyOption(
            KeyHandle,
            w!("ExecuteOptions").as_ptr() as PCWSTR,
            4,
            core::ptr::addr_of_mut!(ExecuteOptions) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(Status) {
            // TODO
        }

        if (*Peb).__bindgen_anon_1.__bindgen_anon_1.ImageUsesLargePages() != 0 {
            // UNIMPLEMENTED
        }

        LdrQueryImageFileKeyOption(
            KeyHandle,
            w!("DisableHeapLookaside").as_ptr() as PCWSTR,
            REG_DWORD,
            core::ptr::addr_of_mut!(RtlpDisableHeapLookaside) as PVOID,
            core::mem::size_of_val(&RtlpDisableHeapLookaside) as ULONG,
            core::ptr::null_mut(),
        );

        LdrQueryImageFileKeyOption(
            KeyHandle,
            w!("ShutdownFlags").as_ptr() as PCWSTR,
            REG_DWORD,
            core::ptr::addr_of_mut!(RtlpShutdownProcessFlags) as PVOID,
            core::mem::size_of_val(&RtlpShutdownProcessFlags) as ULONG,
            core::ptr::null_mut(),
        );

        LdrQueryImageFileKeyOption(
            KeyHandle,
            w!("MinimumStackCommitInBytes").as_ptr() as PCWSTR,
            REG_DWORD,
            core::ptr::addr_of_mut!(MinimumStackCommit) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::null_mut(),
        );

        if ((*Peb).MinimumStackCommit as ULONG) < MinimumStackCommit {
            (*Peb).MinimumStackCommit = MinimumStackCommit as usize as _;
        }

        Status = LdrQueryImageFileKeyOption(
            KeyHandle,
            w!("GlobalFlag").as_ptr() as PCWSTR,
            REG_DWORD,
            core::ptr::addr_of_mut!(GlobalFlag) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(Status) {
            (*Peb).NtGlobalFlag = GlobalFlag;
        } else {
            GlobalFlag = 0;
        }

        if (*Peb).NtGlobalFlag & (FLG_APPLICATION_VERIFIER | FLG_HEAP_PAGE_ALLOCS) != 0 {
            Status = LdrpInitializeApplicationVerifierPackage(KeyHandle, Peb, 0u8, 0u8);
            if !NT_SUCCESS(Status) {
                // DPRINT1
            }
        }
    } else {
        if (*Peb).NtGlobalFlag & (FLG_APPLICATION_VERIFIER | FLG_HEAP_PAGE_ALLOCS) != 0 {
            Status = LdrpInitializeApplicationVerifierPackage(KeyHandle, Peb, 1u8, 0u8);
            if !NT_SUCCESS(Status) {
                // DPRINT1
            }
        }
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "system" fn LdrpValidateImageForMp(_LdrDataTableEntry: PLDR_DATA_TABLE_ENTRY) {
}
#[no_mangle]
pub unsafe extern "system" fn LdrpDisableProcessCompatGuidDetection() -> u8 {
    let mut PolicyKey: _UNICODE_STRING = core::mem::zeroed();
    let PolicyKeyStr: &[u16] = &[
        '\\' as u16, 'R' as u16, 'e' as u16, 'g' as u16, 'i' as u16, 's' as u16, 't' as u16, 'r' as u16, 'y' as u16,
        '\\' as u16, 'M' as u16, 'A' as u16, 'C' as u16, 'H' as u16, 'I' as u16, 'N' as u16, 'E' as u16,
        '\\' as u16, 'S' as u16, 'o' as u16, 'f' as u16, 't' as u16, 'w' as u16, 'a' as u16, 'r' as u16, 'e' as u16,
        '\\' as u16, 'P' as u16, 'o' as u16, 'l' as u16, 'i' as u16, 'c' as u16, 'i' as u16, 'e' as u16, 's' as u16,
        '\\' as u16, 'M' as u16, 'i' as u16, 'c' as u16, 'r' as u16, 'o' as u16, 's' as u16, 'o' as u16, 'f' as u16, 't' as u16,
        '\\' as u16, 'W' as u16, 'i' as u16, 'n' as u16, 'd' as u16, 'o' as u16, 'w' as u16, 's' as u16,
        '\\' as u16, 'A' as u16, 'p' as u16, 'p' as u16, 'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'a' as u16, 't' as u16,
        0,
    ];
    PolicyKey.Length = ((PolicyKeyStr.len() - 1) * 2) as u16;
    PolicyKey.MaximumLength = (PolicyKeyStr.len() * 2) as u16;
    PolicyKey.Buffer = PolicyKeyStr.as_ptr() as *mut u16;

    let mut DisableDetection: _UNICODE_STRING = core::mem::zeroed();
    let DisableDetectionStr: &[u16] = &[
        'D' as u16, 'i' as u16, 's' as u16, 'a' as u16, 'b' as u16, 'l' as u16, 'e' as u16,
        'C' as u16, 'o' as u16, 'm' as u16, 'p' as u16, 'a' as u16, 't' as u16,
        'G' as u16, 'u' as u16, 'i' as u16, 'd' as u16,
        'D' as u16, 'e' as u16, 't' as u16, 'e' as u16, 'c' as u16, 't' as u16, 'i' as u16, 'o' as u16, 'n' as u16,
        0,
    ];
    DisableDetection.Length = ((DisableDetectionStr.len() - 1) * 2) as u16;
    DisableDetection.MaximumLength = (DisableDetectionStr.len() * 2) as u16;
    DisableDetection.Buffer = DisableDetectionStr.as_ptr() as *mut u16;

    let mut PolicyKeyAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    PolicyKeyAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    PolicyKeyAttributes.RootDirectory = core::ptr::null_mut();
    PolicyKeyAttributes.ObjectName = core::ptr::addr_of_mut!(PolicyKey);
    PolicyKeyAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    PolicyKeyAttributes.SecurityDescriptor = core::ptr::null_mut();
    PolicyKeyAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let mut KeyInfo: KEY_VALUE_PARTIAL_INFORMATION = core::mem::zeroed();
    let mut ResultLength: u32 = 0;
    let mut KeyHandle: *mut core::ffi::c_void = core::ptr::null_mut();

    let Status = NtOpenKey(&mut KeyHandle, KEY_QUERY_VALUE, &mut PolicyKeyAttributes);
    if NT_SUCCESS(Status) {
        let Status = NtQueryValueKey(
            KeyHandle,
            &mut DisableDetection,
            KeyValuePartialInformation,
            core::ptr::addr_of_mut!(KeyInfo) as *mut core::ffi::c_void,
            core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as u32,
            &mut ResultLength,
        );
        NtClose(KeyHandle);
        if NT_SUCCESS(Status)
            && KeyInfo.Type == REG_DWORD
            && KeyInfo.DataLength == core::mem::size_of::<u32>() as u32
            && KeyInfo.Data[0] == 1
        {
            return 1u8;
        }
    }
    0u8
}

#[no_mangle]
pub unsafe extern "system" fn LdrpInitializeProcessCompat(
    pProcessActctx: PVOID,
    pOldShimData: *mut PVOID,
) {
    struct KnownCompatGuid {
        Guid: *const GUID,
        Version: DWORD,
    }

    let KnownCompatGuids: [KnownCompatGuid; 5] = [
        KnownCompatGuid { Guid: &COMPAT_GUID_WIN10, Version: _WIN32_WINNT_WIN10 as DWORD },
        KnownCompatGuid { Guid: &COMPAT_GUID_WIN81, Version: _WIN32_WINNT_WINBLUE as DWORD },
        KnownCompatGuid { Guid: &COMPAT_GUID_WIN8, Version: _WIN32_WINNT_WIN8 as DWORD },
        KnownCompatGuid { Guid: &COMPAT_GUID_WIN7, Version: _WIN32_WINNT_WIN7 as DWORD },
        KnownCompatGuid { Guid: &COMPAT_GUID_VISTA, Version: _WIN32_WINNT_VISTA as DWORD },
    ];

    const BUFFER_LEN: usize = (core::mem::size_of::<COMPATIBILITY_CONTEXT_ELEMENT>() * 10
        + core::mem::size_of::<ACTIVATION_CONTEXT_COMPATIBILITY_INFORMATION>())
        / core::mem::size_of::<ULONG>();
    let mut Buffer: [ULONG; BUFFER_LEN] = [0; BUFFER_LEN];

    let ContextCompatInfo: *mut ACTIVATION_CONTEXT_COMPATIBILITY_INFORMATION;
    let mut SizeRequired: SIZE_T;
    let Status: NTSTATUS;
    let mut n: DWORD;
    let mut cur: DWORD;
    let mut pShimData: *mut ReactOS_ShimData = *pOldShimData as *mut ReactOS_ShimData;

    if !pShimData.is_null() {
        if (*pShimData).dwMagic != REACTOS_SHIMDATA_MAGIC
            || (*pShimData).dwSize as usize != core::mem::size_of::<ReactOS_ShimData>()
        {
            return;
        }
        if (*pShimData).dwRosProcessCompatVersion != 0 {
            return;
        }
    }

    SizeRequired = core::mem::size_of_val(&Buffer) as SIZE_T;
    Status = RtlQueryInformationActivationContext(
        RTL_QUERY_ACTIVATION_CONTEXT_FLAG_NO_ADDREF,
        pProcessActctx,
        core::ptr::null_mut(),
        CompatibilityInformationInActivationContext,
        Buffer.as_mut_ptr() as PVOID,
        core::mem::size_of_val(&Buffer) as SIZE_T,
        &mut SizeRequired,
    );

    if !NT_SUCCESS(Status) {
        return;
    }

    ContextCompatInfo = Buffer.as_mut_ptr() as *mut ACTIVATION_CONTEXT_COMPATIBILITY_INFORMATION;
    if (*ContextCompatInfo).ElementCount == 0 {
        return;
    }

    cur = (KnownCompatGuids.len() as DWORD).wrapping_sub(1);
    loop {
        if cur == DWORD::MAX {
            break;
        }
        n = 0;
        while n < (*ContextCompatInfo).ElementCount {
            let element = (*ContextCompatInfo).Elements.as_ptr().offset(n as isize);
            if (*element).Type == ACTCTX_COMPATIBILITY_ELEMENT_TYPE_OS
                && RtlCompareMemory(
                    core::ptr::addr_of!((*element).Id) as *const _,
                    KnownCompatGuids[cur as usize].Guid as *const _,
                    core::mem::size_of::<GUID>() as SIZE_T,
                ) == core::mem::size_of::<GUID>() as SIZE_T
            {
                if LdrpDisableProcessCompatGuidDetection() != 0 {
                    return;
                }

                if pShimData.is_null() {
                    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;

                    pShimData = RtlAllocateHeap(
                        (*Peb).ProcessHeap,
                        HEAP_ZERO_MEMORY,
                        core::mem::size_of::<ReactOS_ShimData>() as SIZE_T,
                    ) as *mut ReactOS_ShimData;

                    if pShimData.is_null() {
                        return;
                    }

                    (*pShimData).dwSize = core::mem::size_of::<ReactOS_ShimData>() as DWORD;
                    (*pShimData).dwMagic = REACTOS_SHIMDATA_MAGIC;

                    (*Peb).pShimData = pShimData as PVOID;
                    *pOldShimData = pShimData as PVOID;
                }

                (*pShimData).dwRosProcessCompatVersion = KnownCompatGuids[cur as usize].Version;
                return;
            }
            n += 1;
        }
        cur = cur.wrapping_sub(1);
    }
}
#[no_mangle]
pub unsafe extern "system" fn LdrpInitializeDotLocalSupport(ProcessParameters: PRTL_USER_PROCESS_PARAMETERS) {
    let ImagePathName: _UNICODE_STRING = (*ProcessParameters).ImagePathName;
    let mut LocalBuffer: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    let mut DotLocal: _UNICODE_STRING = core::mem::zeroed();
    let mut Status: i32;
    let RequiredSize: u32;

    RequiredSize = ImagePathName.Length as u32
        + LdrpDotLocal.Length as u32
        + core::mem::size_of::<u16>() as u32;

    if RequiredSize as usize <= core::mem::size_of_val(&LocalBuffer) {
        RtlInitEmptyUnicodeString(
            &mut DotLocal,
            LocalBuffer.as_mut_ptr(),
            core::mem::size_of_val(&LocalBuffer) as u16,
        );
    } else if RequiredSize <= 0xFFFE {
        DotLocal.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, RequiredSize as _) as *mut u16;
        DotLocal.Length = 0;
        DotLocal.MaximumLength = RequiredSize as u16;
    } else {
        DotLocal.Buffer = core::ptr::null_mut();
        DotLocal.Length = 0;
        DotLocal.MaximumLength = 0;
    }

    if !DotLocal.Buffer.is_null() {
        Status = RtlAppendUnicodeStringToString(&mut DotLocal, &ImagePathName);
        if NT_SUCCESS(Status) {
            Status = RtlAppendUnicodeStringToString(&mut DotLocal, &LdrpDotLocal);
        }

        if NT_SUCCESS(Status) {
            if RtlDoesFileExists_UStr(&mut DotLocal) != 0 {
                (*ProcessParameters).Flags |= RTL_USER_PROCESS_PARAMETERS_PRIVATE_DLL_PATH;
            }
        }

        if DotLocal.Buffer != LocalBuffer.as_mut_ptr() {
            RtlFreeHeap(RtlGetProcessHeap(), 0, DotLocal.Buffer as *mut core::ffi::c_void);
        }
    }
}


/// Set once process init has seen a WoW64 process (LdrpInitializeProcess).
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
static mut LdrpIsWow64Process: bool = false;

/// A new thread of a WoW64 process whose start is 32-bit code: hand it to
/// wow64.dll to run as a 32-bit thread. Never returns if it does.
///
/// A 64-bit process -- native, or an x86-64 guest under wowx64 -- that does
/// CreateRemoteThread into a WoW64 process names a 32-bit routine, but the
/// thread the kernel makes is native and starts from a native CONTEXT: the
/// Win32 shape the native kernel32's BaseThreadStartup(Start, Param), the
/// RtlCreateUserThread shape Start(Param) itself. Run here, either one calls
/// 32-bit code as native. Windows' WoW64 converts such a startup context for
/// every thread it initializes (ThunkStartupContext64TO32); here the loader
/// asks, and hands over only a thread whose start lies in no NATIVE image --
/// so WoW64's own threads (wow64.dll's guest-thread start, the CPU layer's)
/// and native ones (a debugger's break-in) still run as they always did.
/// "Native" is the image's machine, not just its presence in our list: the
/// i386 main image is in it too.
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn LdrpDivertWow64Thread(Context: PCONTEXT) {
    #[cfg(target_arch = "aarch64")]
    const NATIVE_MACHINE: u16 = IMAGE_FILE_MACHINE_ARM64 as u16;
    #[cfg(target_arch = "x86_64")]
    const NATIVE_MACHINE: u16 = IMAGE_FILE_MACHINE_AMD64 as u16;

    if !LdrpIsWow64Process || Context.is_null() {
        return;
    }
    #[cfg(target_arch = "aarch64")]
    let (Pc, Arg0, Arg1) = (
        (*Context).Pc,
        (*Context).__bindgen_anon_1.X[0],
        (*Context).__bindgen_anon_1.X[1],
    );
    #[cfg(target_arch = "x86_64")]
    let (Pc, Arg0, Arg1) = ((*Context).Rip, (*Context).Rcx, (*Context).Rdx);

    let IsNative = |Address: u64| -> bool {
        let mut Base: PVOID = core::ptr::null_mut();
        if RtlPcToFileHeader(Address as PVOID, &mut Base).is_null() {
            return false;
        }
        let Nt = RtlImageNtHeader(Base);
        !Nt.is_null() && (*Nt).FileHeader.Machine == NATIVE_MACHINE
    };
    let Export = |Dll: *mut _UNICODE_STRING, Name: &[u8]| -> u64 {
        let mut Base: PVOID = core::ptr::null_mut();
        let mut Ansi: ANSI_STRING = core::mem::zeroed();
        let mut Address: PVOID = core::ptr::null_mut();
        RtlInitAnsiString(&mut Ansi, Name.as_ptr() as PCSZ);
        if !NT_SUCCESS(LdrGetDllHandle(core::ptr::null_mut(), core::ptr::null_mut(), Dll, &mut Base))
            || !NT_SUCCESS(LdrGetProcedureAddress(Base, &mut Ansi, 0, &mut Address))
        {
            return 0;
        }
        Address as u64
    };

    let ThreadStart = Export(core::ptr::addr_of_mut!(Kernel32String), b"BaseThreadStart\0");
    let (Start, Param, Win32Thread) = if ThreadStart != 0 && Pc == ThreadStart {
        if IsNative(Arg0) {
            return;
        }
        (Arg0, Arg1, 1u8)
    } else {
        if IsNative(Pc) {
            return;
        }
        (Pc, Arg0, 0u8)
    };

    let Handoff = Export(core::ptr::addr_of_mut!(Wow64DllString), b"Wow64StartGuestThreadFrom64\0");
    if Handoff == 0 {
        DbgPrint(b"[LDR] WoW64 thread with a 32-bit start %p, but no wow64.dll handoff\n\0".as_ptr() as *const i8,
                 Start as PVOID);
        return;
    }
    let Handoff: unsafe extern "system" fn(ULONG_PTR, ULONG_PTR, BOOLEAN) = core::mem::transmute(Handoff);
    Handoff(Start as ULONG_PTR, Param as ULONG_PTR, Win32Thread);
}

/// Hand the initial thread to a guest layer — wow64.dll for an i386 image,
/// wowx64.dll for an x86-64 image on arm64 — by calling its
/// `<InitName>(Context)`, which never returns: it runs the guest from here.
/// Returns only on failure, with the status to fail process init with.
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
unsafe fn LdrpEnterGuestLayer(
    DllName: *mut _UNICODE_STRING,
    InitName: &[u8],
    Context: PCONTEXT,
) -> NTSTATUS {
    const STATUS_ENTRYPOINT_NOT_FOUND: NTSTATUS = 0xC000_0139u32 as NTSTATUS;
    let mut Base: PVOID = core::ptr::null_mut();
    let mut Status = LdrLoadDll(core::ptr::null_mut(), core::ptr::null_mut(), DllName, &mut Base);
    if !NT_SUCCESS(Status) {
        DbgPrint(b"[LDR] guest layer %wZ: load FAILED 0x%08lx\n\0".as_ptr() as *const i8, DllName, Status);
        return Status;
    }
    let mut Name: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&mut Name, InitName.as_ptr() as PCSZ);
    let mut InitAddr: PVOID = core::ptr::null_mut();
    Status = LdrGetProcedureAddress(Base, &mut Name, 0, &mut InitAddr);
    if !NT_SUCCESS(Status) || InitAddr.is_null() {
        DbgPrint(b"[LDR] guest layer %wZ: %s not found (0x%08lx)\n\0".as_ptr() as *const i8,
                 DllName, InitName.as_ptr(), Status);
        return STATUS_ENTRYPOINT_NOT_FOUND;
    }
    let Init: unsafe extern "system" fn(PCONTEXT) = core::mem::transmute(InitAddr);

    /* Publish process-init completion BEFORE handing the thread over.
     *
     * The guest layer never returns — it drives the guest from here — so our
     * caller's `LdrpInLdrInit = 0` + `_InterlockedIncrement(
     * &LdrpProcessInitialized)` never run, and LdrpProcessInitialized is left
     * at 1 forever. Any LATER thread then enters LdrpInitialize's
     * `while _InterlockedCompareExchange(&LdrpProcessInitialized, 1, 0)
     * == 1` wait and spins on ZwDelayExecution for the lifetime of the
     * process, never reaching its start routine. That is invisible while a
     * guest is single-threaded, and it is exactly what made every created
     * WoW64 guest thread hang silently (task #64).
     *
     * Publishing here is accurate, not a workaround: the native loader HAS
     * finished process init at this point (LdrpRunInitializeRoutines
     * returned); all that remains is the handoff itself. */
    LdrpInLdrInit = 0u8;
    _InterlockedIncrement(core::ptr::addr_of_mut!(LdrpProcessInitialized));

    Init(Context);
    DbgPrint(b"[LDR] guest layer %wZ: %s RETURNED\n\0".as_ptr() as *const i8, DllName, InitName.as_ptr());
    STATUS_INVALID_IMAGE_FORMAT
}
#[no_mangle]
pub unsafe extern "system" fn LdrpInitializeProcess(
    Context: PCONTEXT,
    SystemArgument1: PVOID,
) -> NTSTATUS {
    let mut HeapParameters: RTL_HEAP_PARAMETERS = core::mem::zeroed();
    let mut ComSectionSize: ULONG = 0;
    let mut BaseProcessInitPostImportName: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(
        &mut BaseProcessInitPostImportName,
        b"BaseProcessInitPostImport\0".as_ptr() as PCSZ,
    );
    let mut BaseQueryModuleDataName: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(
        &mut BaseQueryModuleDataName,
        b"BaseQueryModuleData\0".as_ptr() as PCSZ,
    );
    let mut OldShimData: PVOID;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SymLinkHandle: HANDLE = core::ptr::null_mut();
    let mut CommandLine: _UNICODE_STRING = core::mem::zeroed();
    let mut NtSystemRoot: _UNICODE_STRING = core::mem::zeroed();
    let mut ImagePathName: _UNICODE_STRING = core::mem::zeroed();
    let mut FullPath: _UNICODE_STRING = core::mem::zeroed();
    let mut ImageFileName: _UNICODE_STRING = core::mem::zeroed();
    let mut KnownDllString: _UNICODE_STRING = core::mem::zeroed();
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;
    let mut IsDotNetImage: BOOLEAN = 0u8;
    let mut FreeCurDir: BOOLEAN = 0u8;
    let ProcessParameters: PRTL_USER_PROCESS_PARAMETERS;
    let mut ConfigSize: ULONG = 0;
    let mut CurrentDirectory: _UNICODE_STRING = core::mem::zeroed();
    let mut OptionsKey: HANDLE = core::ptr::null_mut();
    let mut HeapFlags: ULONG;
    let NtHeader: PIMAGE_NT_HEADERS;
    let mut NtDllName: LPWSTR = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let ImportStatus: NTSTATUS;
    let mut NlsTable: NLSTABLEINFO = core::mem::zeroed();
    let LoadConfig: PIMAGE_LOAD_CONFIG_DIRECTORY;
    let Teb: PTEB = NtCurrentTeb();
    let ListHead: PLIST_ENTRY;
    let mut NextEntry: PLIST_ENTRY;
    let mut i: ULONG;
    let mut ImagePath: PWSTR;
    let mut DebugProcessHeapOnly: ULONG = 0;
    let mut NtLdrEntry: PLDR_DATA_TABLE_ENTRY;
    let mut Current: PWCHAR;
    let mut ExecuteOptions: ULONG = 0;
    let ViewBase: PVOID;

    /* Set a NULL SEH Filter */
    RtlSetUnhandledExceptionFilter(None);

    /* Get the image path */
    ImagePath = (*(*Peb).ProcessParameters).ImagePathName.Buffer;

    /* Check if it's not normalized */
    if ((*(*Peb).ProcessParameters).Flags & RTL_USER_PROCESS_PARAMETERS_NORMALIZED) == 0 {
        /* Normalize it */
        ImagePath = ((ImagePath as ULONG_PTR) + ((*Peb).ProcessParameters as ULONG_PTR)) as PWSTR;
    }

    /* Create a unicode string for the Image Path */
    ImagePathName.Length = (*(*Peb).ProcessParameters).ImagePathName.Length;
    ImagePathName.MaximumLength = ImagePathName.Length + core::mem::size_of::<WCHAR>() as USHORT;
    ImagePathName.Buffer = ImagePath;

    /* Get the NT Headers */
    NtHeader = RtlImageNtHeader((*Peb).ImageBaseAddress);

    /* Get the execution options */
    Status = LdrpInitializeExecutionOptions(&mut ImagePathName, Peb, &mut OptionsKey);

    /* Check if this is a .NET executable */
    if !RtlImageDirectoryEntryToData(
        (*Peb).ImageBaseAddress,
        1u8,
        IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR as USHORT,
        &mut ComSectionSize,
    )
    .is_null()
    {
        IsDotNetImage = 1u8;
    }

    /* Save the NTDLL Base address */
    NtDllBase = SystemArgument1;

    /* If this is a Native Image */
    if (*NtHeader).OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_NATIVE as USHORT {
        LdrpDllValidation = 1u8;
    }

    /* Save the old Shim Data */
    OldShimData = (*Peb).pShimData;

    /* Save the number of processors and CS timeout */
    LdrpNumberOfProcessors = (*Peb).NumberOfProcessors;
    RtlpTimeout = (*Peb).CriticalSectionTimeout;

    /* Normalize the parameters */
    ProcessParameters = RtlNormalizeProcessParams((*Peb).ProcessParameters);
    if !ProcessParameters.is_null() {
        ImageFileName = (*ProcessParameters).ImagePathName;
        CommandLine = (*ProcessParameters).CommandLine;
    } else {
        RtlInitUnicodeString(&mut ImageFileName, core::ptr::null_mut());
        RtlInitUnicodeString(&mut CommandLine, core::ptr::null_mut());
    }

    /* Initialize NLS data */
    RtlInitNlsTables(
        (*Peb).AnsiCodePageData as PUSHORT,
        (*Peb).OemCodePageData as PUSHORT,
        (*Peb).UnicodeCaseTableData as PUSHORT,
        &mut NlsTable,
    );

    /* Reset NLS Translations */
    RtlResetRtlTranslations(&mut NlsTable);

    /* Get the Image Config Directory */
    LoadConfig = RtlImageDirectoryEntryToData(
        (*Peb).ImageBaseAddress,
        1u8,
        IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG as USHORT,
        &mut ConfigSize,
    ) as PIMAGE_LOAD_CONFIG_DIRECTORY;

    /* Setup the Heap Parameters */
    core::ptr::write_bytes(
        &mut HeapParameters as *mut _ as *mut u8,
        0,
        core::mem::size_of::<RTL_HEAP_PARAMETERS>(),
    );
    HeapFlags = HEAP_GROWABLE;
    HeapParameters.Length = core::mem::size_of::<RTL_HEAP_PARAMETERS>() as ULONG;

    /* Check if we have Configuration Data */
    if !LoadConfig.is_null() && ConfigSize != 0 {
        let security_cookie_valid = ConfigSize as usize
            >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, SecurityCookie)
                + core::mem::size_of::<PVOID>());
        if security_cookie_valid || ConfigSize == (*LoadConfig).Size {
            if ConfigSize as usize != core::mem::size_of::<IMAGE_LOAD_CONFIG_DIRECTORY>() {
                // WARN: differing LOAD_CONFIG size
            }

            let gfs_valid = ConfigSize as usize
                >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, GlobalFlagsSet)
                    + core::mem::size_of::<DWORD>());
            if gfs_valid && (*LoadConfig).GlobalFlagsSet != 0 {
                (*Peb).NtGlobalFlag |= (*LoadConfig).GlobalFlagsSet;
            }

            let gfc_valid = ConfigSize as usize
                >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, GlobalFlagsClear)
                    + core::mem::size_of::<DWORD>());
            if gfc_valid && (*LoadConfig).GlobalFlagsClear != 0 {
                (*Peb).NtGlobalFlag &= !(*LoadConfig).GlobalFlagsClear;
            }

            let cs_valid = ConfigSize as usize
                >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, CriticalSectionDefaultTimeout)
                    + core::mem::size_of::<DWORD>());
            if cs_valid && (*LoadConfig).CriticalSectionDefaultTimeout != 0 {
                RtlpTimeout.QuadPart =
                    ((*LoadConfig).CriticalSectionDefaultTimeout as i64) * (-10000i64);
            }

            let decommit_block_valid = ConfigSize as usize
                >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, DeCommitFreeBlockThreshold)
                    + core::mem::size_of::<SIZE_T>());
            if decommit_block_valid && (*LoadConfig).DeCommitFreeBlockThreshold != 0 {
                HeapParameters.DeCommitFreeBlockThreshold =
                    (*LoadConfig).DeCommitFreeBlockThreshold as SIZE_T;
            }

            let decommit_total_valid = ConfigSize as usize
                >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, DeCommitTotalFreeThreshold)
                    + core::mem::size_of::<SIZE_T>());
            if decommit_total_valid && (*LoadConfig).DeCommitTotalFreeThreshold != 0 {
                HeapParameters.DeCommitTotalFreeThreshold =
                    (*LoadConfig).DeCommitTotalFreeThreshold as SIZE_T;
            }

            let max_alloc_valid = ConfigSize as usize
                >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, MaximumAllocationSize)
                    + core::mem::size_of::<SIZE_T>());
            if max_alloc_valid && (*LoadConfig).MaximumAllocationSize != 0 {
                HeapParameters.MaximumAllocationSize = (*LoadConfig).MaximumAllocationSize as SIZE_T;
            }

            let vmt_valid = ConfigSize as usize
                >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, VirtualMemoryThreshold)
                    + core::mem::size_of::<SIZE_T>());
            if vmt_valid && (*LoadConfig).VirtualMemoryThreshold != 0 {
                HeapParameters.VirtualMemoryThreshold =
                    (*LoadConfig).VirtualMemoryThreshold as SIZE_T;
            }

            let phf_valid = ConfigSize as usize
                >= (core::mem::offset_of!(IMAGE_LOAD_CONFIG_DIRECTORY, ProcessHeapFlags)
                    + core::mem::size_of::<DWORD>());
            if phf_valid && (*LoadConfig).ProcessHeapFlags != 0 {
                HeapFlags = (*LoadConfig).ProcessHeapFlags;
            }
        }
    }

    /* Check for custom affinity mask */
    if (*Peb).ImageProcessAffinityMask != 0 {
        Status = NtSetInformationProcess(
            -1isize as HANDLE,
            ProcessAffinityMask,
            core::ptr::addr_of_mut!((*Peb).ImageProcessAffinityMask) as PVOID,
            core::mem::size_of_val(&(*Peb).ImageProcessAffinityMask) as ULONG,
        );
    }

    /* Check if verbose debugging (ShowSnaps) was requested */
    ShowSnaps = if ((*Peb).NtGlobalFlag & FLG_SHOW_LDR_SNAPS) != 0 { 1u8 } else { 0u8 };

    /* If the CS timeout is longer than 1 hour, disable it */
    if RtlpTimeout.QuadPart < (3600i64) * (-10000000i64) {
        RtlpTimeoutDisable = 1u8;
    }

    /* Initialize Critical Section Data */
    RtlpInitDeferredCriticalSection();

    /* Initialize VEH Call lists */
    RtlpInitializeVectoredExceptionHandling();

    /* Set TLS/FLS Bitmap data */
    (*Peb).FlsBitmap = core::ptr::addr_of_mut!(FlsBitMap) as PVOID;
    (*Peb).TlsBitmap = core::ptr::addr_of_mut!(TlsBitMap) as PVOID;
    (*Peb).TlsExpansionBitmap = core::ptr::addr_of_mut!(TlsExpansionBitMap) as PVOID;

    /* Initialize FLS Bitmap */
    RtlInitializeBitMap(
        &mut FlsBitMap,
        core::ptr::addr_of_mut!((*Peb).FlsBitmapBits) as PULONG,
        FLS_MAXIMUM_AVAILABLE,
    );
    RtlSetBit(&mut FlsBitMap, 0);
    InitializeListHead(core::ptr::addr_of_mut!((*Peb).FlsListHead));

    /* Initialize TLS Bitmap */
    RtlInitializeBitMap(
        &mut TlsBitMap,
        core::ptr::addr_of_mut!((*Peb).TlsBitmapBits) as PULONG,
        TLS_MINIMUM_AVAILABLE,
    );
    RtlSetBit(&mut TlsBitMap, 0);
    /* arm64: the slot ntdll's own `_errno` lives in (crtexports.rs), taken
     * before anything can TlsAlloc it. */
    #[cfg(target_arch = "aarch64")]
    RtlSetBit(&mut TlsBitMap, crate::crtexports::NTDLL_TLS_ERRNO);
    RtlInitializeBitMap(
        &mut TlsExpansionBitMap,
        core::ptr::addr_of_mut!((*Peb).TlsExpansionBitmapBits) as PULONG,
        TLS_EXPANSION_SLOTS,
    );
    RtlSetBit(&mut TlsExpansionBitMap, 0);

    /* Initialize the Hash Table */
    i = 0;
    while i < LDR_HASH_TABLE_ENTRIES as ULONG {
        InitializeListHead(core::ptr::addr_of_mut!(LdrpHashTable[i as usize]));
        i += 1;
    }

    /* Initialize the Loader Lock */
    RtlInitializeCriticalSection(&mut LdrpLoaderLock);
    LdrpLoaderLockInit = 1u8;

    /* Setup Fast PEB Lock */
    RtlInitializeCriticalSection(&mut FastPebLock);
    (*Peb).FastPebLock = &mut FastPebLock;

    /* For old executables, use 16-byte aligned heap */
    if (*NtHeader).OptionalHeader.MajorSubsystemVersion <= 3
        && (*NtHeader).OptionalHeader.MinorSubsystemVersion < 51
    {
        HeapFlags |= HEAP_CREATE_ALIGN_16;
    }

    /* Setup the Heap */
    RtlInitializeHeapManager();
    (*Peb).ProcessHeap = RtlCreateHeap(
        HeapFlags,
        core::ptr::null_mut(),
        (*NtHeader).OptionalHeader.SizeOfHeapReserve as SIZE_T,
        (*NtHeader).OptionalHeader.SizeOfHeapCommit as SIZE_T,
        core::ptr::null_mut(),
        &mut HeapParameters,
    );

    if (*Peb).ProcessHeap.is_null() {
        return STATUS_NO_MEMORY;
    }

    Status = RtlpInitializeLocaleTable();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Allocate an Activation Context Stack */
    Status = RtlAllocateActivationContextStack(core::ptr::addr_of_mut!(
        (*Teb).ActivationContextStackPointer
    ));
    if !NT_SUCCESS(Status) {
        return Status;
    }

    core::ptr::write_bytes(
        &mut HeapParameters as *mut _ as *mut u8,
        0,
        core::mem::size_of::<RTL_HEAP_PARAMETERS>(),
    );
    HeapFlags = HEAP_GROWABLE | HEAP_CLASS_1;
    HeapParameters.Length = core::mem::size_of::<RTL_HEAP_PARAMETERS>() as ULONG;
    LdrpHeap = RtlCreateHeap(
        HeapFlags,
        core::ptr::null_mut(),
        0x10000,
        0x6000,
        core::ptr::null_mut(),
        &mut HeapParameters,
    );
    if LdrpHeap.is_null() {
        return STATUS_NO_MEMORY;
    }

    /* Check for Debug Heap */
    if !OptionsKey.is_null() {
        Status = LdrQueryImageFileKeyOption(
            OptionsKey,
            w!("DebugProcessHeapOnly").as_ptr() as PCWSTR,
            REG_DWORD,
            core::ptr::addr_of_mut!(DebugProcessHeapOnly) as PVOID,
            core::mem::size_of::<ULONG>() as ULONG,
            core::ptr::null_mut(),
        );

        if NT_SUCCESS(Status) {
            if RtlpPageHeapEnabled != 0 && DebugProcessHeapOnly != 0 {
                RtlpDphGlobalFlags &= !DPH_FLAG_DLL_NOTIFY;
                RtlpPageHeapEnabled = 0u8;
            }
        }
    }

    /* Build the NTDLL Path */
    FullPath.Buffer = StringBuffer.as_mut_ptr();
    FullPath.Length = 0;
    FullPath.MaximumLength = core::mem::size_of_val(&StringBuffer) as USHORT;
    RtlInitUnicodeString(&mut NtSystemRoot, (*SharedUserData).NtSystemRoot.as_ptr());
    RtlAppendUnicodeStringToString(&mut FullPath, &NtSystemRoot);
    RtlAppendUnicodeToString(
        &mut FullPath,
        w!("\\System32\\").as_ptr() as PCWSTR,
    );

    /* Open the Known DLLs directory. A 32-bit process under WOW64 must use
     * \KnownDlls32 — \KnownDlls holds the HOST-architecture image sections
     * (mapping one of those hands an arm64 kernel32 to an x86 guest). When
     * \KnownDlls32 doesn't exist (arm64 hosts don't populate it yet) the open
     * fails and the loader falls back to file paths, which the wow64 layer
     * redirects system32 -> syswow64. */
    #[cfg(target_arch = "x86")]
    let KnownDllDirName: PCWSTR = {
        extern "system" {
            fn NtQueryInformationProcess(
                ProcessHandle: HANDLE,
                ProcessInformationClass: u32,
                ProcessInformation: PVOID,
                ProcessInformationLength: ULONG,
                ReturnLength: PULONG,
            ) -> NTSTATUS;
        }
        const PROCESS_WOW64_INFORMATION: u32 = 26;
        let mut Peb32Ptr: ULONG_PTR = 0;
        let QueryStatus = NtQueryInformationProcess(
            -1isize as HANDLE,
            PROCESS_WOW64_INFORMATION,
            core::ptr::addr_of_mut!(Peb32Ptr) as PVOID,
            core::mem::size_of::<ULONG_PTR>() as ULONG,
            core::ptr::null_mut(),
        );
        if NT_SUCCESS(QueryStatus) && Peb32Ptr != 0 {
            w!("\\KnownDlls32").as_ptr() as PCWSTR
        } else {
            w!("\\KnownDlls").as_ptr() as PCWSTR
        }
    };
    #[cfg(not(target_arch = "x86"))]
    let KnownDllDirName: PCWSTR = w!("\\KnownDlls").as_ptr() as PCWSTR;
    RtlInitUnicodeString(&mut KnownDllString, KnownDllDirName);
    // InitializeObjectAttributes
    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KnownDllString;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
    Status = ZwOpenDirectoryObject(
        &mut LdrpKnownDllObjectDirectory,
        DIRECTORY_QUERY | DIRECTORY_TRAVERSE,
        &mut ObjectAttributes,
    );

    /* Check if it exists */
    if NT_SUCCESS(Status) {
        RtlInitUnicodeString(
            &mut KnownDllString,
            w!("KnownDllPath").as_ptr() as PCWSTR,
        );
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = LdrpKnownDllObjectDirectory;
        ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
        ObjectAttributes.ObjectName = &mut KnownDllString;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();
        Status = NtOpenSymbolicLinkObject(
            &mut SymLinkHandle,
            SYMBOLIC_LINK_QUERY,
            &mut ObjectAttributes,
        );
        if NT_SUCCESS(Status) {
            LdrpKnownDllPath.Length = 0;
            LdrpKnownDllPath.MaximumLength =
                core::mem::size_of_val(&LdrpKnownDllPathBuffer) as USHORT;
            LdrpKnownDllPath.Buffer = LdrpKnownDllPathBuffer.as_mut_ptr();
            Status = ZwQuerySymbolicLinkObject(
                SymLinkHandle,
                &mut LdrpKnownDllPath,
                core::ptr::null_mut(),
            );
            NtClose(SymLinkHandle);
            if !NT_SUCCESS(Status) {
                return Status;
            }
        }
    }

    /* Check if we failed */
    if !NT_SUCCESS(Status) {
        LdrpKnownDllObjectDirectory = core::ptr::null_mut();
        RtlInitUnicodeString(&mut LdrpKnownDllPath, StringBuffer.as_ptr());
        LdrpKnownDllPath.Length -= core::mem::size_of::<WCHAR>() as USHORT;
    }

    /* If we have process parameters, get the default path and current path */
    if !ProcessParameters.is_null() {
        if (*ProcessParameters).DllPath.Length != 0 {
            LdrpDefaultPath = *(core::ptr::addr_of!((*ProcessParameters).DllPath) as PUNICODE_STRING);
        } else {
            LdrpInitFailure(STATUS_INVALID_PARAMETER);
        }

        CurrentDirectory = (*ProcessParameters).CurrentDirectory.DosPath;

        if CurrentDirectory.Buffer.is_null()
            || *CurrentDirectory.Buffer == 0
            || CurrentDirectory.Length == 0
        {
            CurrentDirectory.Buffer = RtlAllocateHeap(
                (*Peb).ProcessHeap,
                0,
                (3 * core::mem::size_of::<WCHAR>() + core::mem::size_of::<WCHAR>()) as _,
            ) as PWSTR;
            if CurrentDirectory.Buffer.is_null() {
                // FIXME
            }

            core::ptr::copy_nonoverlapping(
                (*SharedUserData).NtSystemRoot.as_ptr() as *const u8,
                CurrentDirectory.Buffer as *mut u8,
                3 * core::mem::size_of::<WCHAR>(),
            );
            *CurrentDirectory.Buffer.add(3) = 0;
            CurrentDirectory.Length = (3 * core::mem::size_of::<WCHAR>()) as USHORT;
            CurrentDirectory.MaximumLength =
                CurrentDirectory.Length + core::mem::size_of::<WCHAR>() as USHORT;

            FreeCurDir = 1u8;
        }
    }

    /* Setup Loader Data */
    (*Peb).Ldr = core::ptr::addr_of_mut!(PebLdr);
    InitializeListHead(core::ptr::addr_of_mut!(PebLdr.InLoadOrderModuleList));
    InitializeListHead(core::ptr::addr_of_mut!(PebLdr.InMemoryOrderModuleList));
    InitializeListHead(core::ptr::addr_of_mut!(PebLdr.InInitializationOrderModuleList));
    PebLdr.Length = core::mem::size_of::<PEB_LDR_DATA>() as ULONG;
    PebLdr.Initialized = 1u8;

    /* Allocate a data entry for the Image */
    LdrpImageEntry = LdrpAllocateDataTableEntry((*Peb).ImageBaseAddress);

    /* Set it up */
    (*LdrpImageEntry).EntryPoint = LdrpFetchAddressOfEntryPoint((*LdrpImageEntry).DllBase);
    (*LdrpImageEntry).LoadCount = -1i32 as USHORT;
    (*LdrpImageEntry).EntryPointActivationContext = core::ptr::null_mut();
    (*LdrpImageEntry).FullDllName = ImageFileName;

    if IsDotNetImage != 0 {
        (*LdrpImageEntry).Flags = LDRP_COR_IMAGE;
    } else {
        (*LdrpImageEntry).Flags = 0;
    }

    /* Check if the name is empty */
    if *ImageFileName.Buffer == 0 {
        (*LdrpImageEntry).BaseDllName = (*LdrpImageEntry).FullDllName;
    } else {
        Current = ImageFileName.Buffer;
        while *Current != 0 {
            let c = *Current;
            Current = Current.add(1);
            if c == b'\\' as WCHAR {
                NtDllName = Current;
            }
        }

        if NtDllName.is_null() {
            (*LdrpImageEntry).BaseDllName = (*LdrpImageEntry).FullDllName;
        } else {
            (*LdrpImageEntry).BaseDllName.Length = (ImageFileName.Buffer as ULONG_PTR
                + ImageFileName.Length as ULONG_PTR
                - NtDllName as ULONG_PTR)
                as USHORT;
            (*LdrpImageEntry).BaseDllName.MaximumLength =
                (*LdrpImageEntry).BaseDllName.Length + core::mem::size_of::<WCHAR>() as USHORT;
            (*LdrpImageEntry).BaseDllName.Buffer = ((ImageFileName.Buffer as ULONG_PTR)
                + (ImageFileName.Length as ULONG_PTR
                    - (*LdrpImageEntry).BaseDllName.Length as ULONG_PTR))
                as PWSTR;
        }
    }

    /* Processing done, insert it */
    LdrpInsertMemoryTableEntry(LdrpImageEntry);
    (*LdrpImageEntry).Flags |= LDRP_ENTRY_PROCESSED;

    /* Now add an entry for NTDLL */
    NtLdrEntry = LdrpAllocateDataTableEntry(SystemArgument1);
    (*NtLdrEntry).Flags = LDRP_IMAGE_DLL;
    (*NtLdrEntry).EntryPoint = LdrpFetchAddressOfEntryPoint((*NtLdrEntry).DllBase);
    (*NtLdrEntry).LoadCount = -1i32 as USHORT;
    (*NtLdrEntry).EntryPointActivationContext = core::ptr::null_mut();

    (*NtLdrEntry).FullDllName.Length = FullPath.Length;
    (*NtLdrEntry).FullDllName.MaximumLength = FullPath.MaximumLength;
    (*NtLdrEntry).FullDllName.Buffer = StringBuffer.as_mut_ptr();
    RtlAppendUnicodeStringToString(&mut (*NtLdrEntry).FullDllName, &NtDllString);

    (*NtLdrEntry).BaseDllName.Length = NtDllString.Length;
    (*NtLdrEntry).BaseDllName.MaximumLength = NtDllString.MaximumLength;
    (*NtLdrEntry).BaseDllName.Buffer = NtDllString.Buffer;

    /* Processing done, insert it */
    LdrpNtDllDataTableEntry = NtLdrEntry;
    LdrpInsertMemoryTableEntry(NtLdrEntry);

    /* Link the Init Order List */
    InsertHeadList(
        core::ptr::addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InInitializationOrderModuleList),
        core::ptr::addr_of_mut!((*LdrpNtDllDataTableEntry).InInitializationOrderLinks),
    );

    /* Initialize Wine's active context implementation for the current process */
    RtlpInitializeActCtx(&mut OldShimData);

    /* Set the current directory */
    Status = RtlSetCurrentDirectory_U(&mut CurrentDirectory);
    if !NT_SUCCESS(Status) {
        if FreeCurDir != 0 {
            RtlFreeUnicodeString(&mut CurrentDirectory);
        }

        CurrentDirectory = NtSystemRoot;
        RtlSetCurrentDirectory_U(&mut CurrentDirectory);
    } else {
        if FreeCurDir != 0 {
            RtlFreeUnicodeString(&mut CurrentDirectory);
        }
    }

    /* Check if we should look for a .local file */
    if !ProcessParameters.is_null()
        && ((*ProcessParameters).Flags & RTL_USER_PROCESS_PARAMETERS_LOCAL_DLL_PATH) == 0
    {
        LdrpInitializeDotLocalSupport(ProcessParameters);
    }

    /* Check if the Application Verifier was enabled */
    if ((*Peb).NtGlobalFlag & FLG_APPLICATION_VERIFIER) != 0 {
        Status = AVrfInitializeVerifier();
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    if IsDotNetImage != 0 {
        // FIXME
    }

    /* WoW64 (Phase C): detect a WoW64 process early (i386 image on an arm64
     * host — EPROCESS.Wow64Process/PEB32 set by MmCreatePeb) but DEFER the
     * actual divert. The vendored CPU dll (libwow64fex) imports kernel32, so
     * the wow64.dll load below must happen AFTER LdrpRunInitializeRoutines has
     * initialized kernel32 at the normal time (registering the process with
     * csrss) — loading it earlier runs kernel32's DllMain re-entrantly under
     * the loader lock and hangs. So we only set a flag here; the handoff is
     * right after the init routines run. Mirrors Win10's LdrpLoadWow64 timing. */
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    let IsWow64Process = {
        extern "system" {
            fn NtQueryInformationProcess(
                ProcessHandle: HANDLE,
                ProcessInformationClass: u32,
                ProcessInformation: PVOID,
                ProcessInformationLength: ULONG,
                ReturnLength: PULONG,
            ) -> NTSTATUS;
        }
        const PROCESS_WOW64_INFORMATION: u32 = 26;
        let mut Peb32Ptr: ULONG_PTR = 0;
        let QueryStatus = NtQueryInformationProcess(
            -1isize as HANDLE,
            PROCESS_WOW64_INFORMATION,
            core::ptr::addr_of_mut!(Peb32Ptr) as PVOID,
            core::mem::size_of::<ULONG_PTR>() as ULONG,
            core::ptr::null_mut(),
        );
        NT_SUCCESS(QueryStatus) && Peb32Ptr != 0
    };

    /* A WoW64 process's 64-bit TlsSlots[0..WOW64_TLS_MAX_NUMBER) are the
     * WoW64 layer's, addressed by fixed index: wow64.dll's CPURESERVED (1),
     * WOW64INFO (10) and callback frame (12), FEX's per-thread state (15..18).
     * Windows keeps TlsAlloc off them; so must we, before the first DLL that
     * could TlsAlloc initializes. Unreserved, the first TlsAlloc in the process
     * gets slot 1: for as long as msvcrt was dragged in early (ntdll's old
     * msvcrt forwarders) its DllMain took it harmlessly, and the day msvcrt
     * stopped loading libwow64fex's rpmalloc got slot 1 instead, its thread
     * heap and the CPU area overwrote each other, and every 32-bit guest
     * crashed in rpmalloc's heap_get_page_generic. */
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    if IsWow64Process {
        LdrpIsWow64Process = true;
        const WOW64_TLS_MAX_NUMBER: ULONG = 19;
        RtlSetBits(&mut TlsBitMap, 1, WOW64_TLS_MAX_NUMBER - 1);
    }

    /* wowx64 (BTCpu64 — docs/engineering/wow64-x64-guest-design.md, Fork B):
     * an x86-64 main image on an arm64 host is an x86-64 GUEST process. Its
     * PE32+ headers have the native shape, so everything this loader does to
     * the image as DATA stays valid (mapping, relocation below), but none of
     * its CODE may run here: its imports are amd64 DLLs, resolved by the
     * guest's own loader under emulation, and its entry point is x86 machine
     * code. Same divert point and timing as WoW64; the guest layer is
     * wowx64.dll. Keyed on the image itself — the kernel needs no flag for a
     * process whose PEB, TEB and address space are the native 64-bit ones. */
    #[cfg(target_arch = "aarch64")]
    let IsX64GuestProcess =
        (*NtHeader).FileHeader.Machine == 0x8664 /* IMAGE_FILE_MACHINE_AMD64 */;

    /* An x86-64 guest's Win32 layer is the GUEST's amd64 kernel32, loaded by
     * its own loader under emulation. The native one would be pure baggage
     * (wowx64.dll and its CPU layer import ntdll alone) and worse than
     * baggage: its DllMain registers the process with csrss and writes the
     * shared PEB, both of which the guest kernel32 then does again. A WoW64
     * process keeps it, as on Windows: neither wow64.dll nor libwow64fex
     * imports it, but a thread a 64-bit process creates in it starts in the
     * native kernel32's BaseThreadStartup (LdrpDivertWow64Thread). */
    #[cfg(target_arch = "aarch64")]
    let NeedNativeKernel32 = !IsX64GuestProcess;
    #[cfg(not(target_arch = "aarch64"))]
    let NeedNativeKernel32 = true;

    if NeedNativeKernel32
        && ((*NtHeader).OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_WINDOWS_GUI as USHORT
            || (*NtHeader).OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_WINDOWS_CUI as USHORT)
    {
        let mut Kernel32BaseAddress: PVOID = core::ptr::null_mut();
        let mut FunctionAddress: PVOID = core::ptr::null_mut();

        Status = LdrLoadDll(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut Kernel32String,
            &mut Kernel32BaseAddress,
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = LdrGetProcedureAddress(
            Kernel32BaseAddress,
            &mut BaseProcessInitPostImportName,
            0,
            &mut FunctionAddress,
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }
        Kernel32ProcessInitPostImportFunction = core::mem::transmute(FunctionAddress);

        Status = LdrGetProcedureAddress(
            Kernel32BaseAddress,
            &mut BaseQueryModuleDataName,
            0,
            &mut FunctionAddress,
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }
        Kernel32BaseQueryModuleData = core::mem::transmute(FunctionAddress);
    }

    /* Walk the IAT and load all the DLLs. For a WoW64 process the image is
     * i386 — its imports are 32-bit and must be resolved by the 32-bit loader
     * running under emulation, NOT here against 64-bit modules. An x86-64
     * guest's imports are amd64 DLLs: the same reasoning, one width up. Skip. */
    #[cfg(target_arch = "aarch64")]
    let SkipImageImports = IsWow64Process || IsX64GuestProcess;
    #[cfg(target_arch = "x86_64")]
    let SkipImageImports = IsWow64Process;
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    {
        if SkipImageImports {
            ImportStatus = STATUS_SUCCESS;
        } else {
            ImportStatus = LdrpWalkImportDescriptor(LdrpDefaultPath.Buffer, LdrpImageEntry);
        }
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    {
        ImportStatus = LdrpWalkImportDescriptor(LdrpDefaultPath.Buffer, LdrpImageEntry);
    }

    /* NtHeader has the native optional-header layout here, so its ImageBase
     * field is not valid for an i386 WoW64 image.  Guest relocation belongs
     * to the 32-bit loader after the handoff below. */
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    let ImageNeedsRelocation =
        !IsWow64Process && (*Peb).ImageBaseAddress != (*NtHeader).OptionalHeader.ImageBase as PVOID;
    #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
    let ImageNeedsRelocation =
        (*Peb).ImageBaseAddress != (*NtHeader).OptionalHeader.ImageBase as PVOID;
    if ImageNeedsRelocation {
        ViewBase = (*Peb).ImageBaseAddress;
        Status = LdrpSetProtection(ViewBase, 0u8);
        if !NT_SUCCESS(Status) {
            return Status;
        }

        Status = LdrRelocateImageWithBias(
            ViewBase,
            0i64,
            core::ptr::null_mut(),
            STATUS_SUCCESS as ULONG,
            STATUS_CONFLICTING_ADDRESSES as ULONG,
            STATUS_INVALID_IMAGE_FORMAT as ULONG,
        ) as NTSTATUS;
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if !Context.is_null() {
            return STATUS_INVALID_IMAGE_FORMAT;
        }

        Status = LdrpSetProtection(ViewBase, 1u8);
        if !NT_SUCCESS(Status) {
            return Status;
        }
    }

    /* Lock the DLLs */
    ListHead = core::ptr::addr_of_mut!((*crate::ldr::ldrinit::LdrpLdrData()).InLoadOrderModuleList);
    NextEntry = (*ListHead).Flink;
    while ListHead != NextEntry {
        NtLdrEntry = (NextEntry as usize
            - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks))
            as PLDR_DATA_TABLE_ENTRY;
        (*NtLdrEntry).LoadCount = -1i32 as USHORT;
        NextEntry = (*NextEntry).Flink;
    }

    /* Phase 0 is done */
    LdrpLdrDatabaseIsSetup = 1u8;

    /* Check whether all static imports were properly loaded and return here */
    if !NT_SUCCESS(ImportStatus) {
        return ImportStatus;
    }

    /* Initialize the keyed event for condition variables */
    RtlpInitializeKeyedEvent();
    RtlpInitializeThreadPooling();

    /* Initialize TLS */
    Status = LdrpInitializeTls();
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* Notify the debugger now */
    if (*Peb).BeingDebugged != 0 {
        DbgBreakPoint();
        ShowSnaps = if ((*Peb).NtGlobalFlag & FLG_SHOW_LDR_SNAPS) != 0 { 1u8 } else { 0u8 };
    }

    /* Validate the Image for MP Usage */
    if LdrpNumberOfProcessors > 1 {
        LdrpValidateImageForMp(LdrpImageEntry);
    }

    /* Check NX options and set them */
    if (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.NXSupportPolicy() == NX_SUPPORT_POLICY_ALWAYSON as UCHAR {
        ExecuteOptions = MEM_EXECUTE_OPTION_DISABLE
            | MEM_EXECUTE_OPTION_DISABLE_THUNK_EMULATION
            | MEM_EXECUTE_OPTION_PERMANENT;
    } else if (*SharedUserData).__bindgen_anon_1.__bindgen_anon_1.NXSupportPolicy() == NX_SUPPORT_POLICY_ALWAYSOFF as UCHAR {
        ExecuteOptions = MEM_EXECUTE_OPTION_ENABLE | MEM_EXECUTE_OPTION_PERMANENT;
    }
    Status = NtSetInformationProcess(
        -1isize as HANDLE,
        ProcessExecuteFlags,
        core::ptr::addr_of_mut!(ExecuteOptions) as PVOID,
        core::mem::size_of::<ULONG>() as ULONG,
    );
    if !NT_SUCCESS(Status) {
        // continue
    }

    // FIXME: Should be done by Application Compatibility features
    RtlInitEmptyUnicodeString(&mut (*Peb).CSDVersion, core::ptr::null_mut(), 0);
    if (((*Peb).OSCSDVersion >> 8) & 0xFF) != 0 {
        let mut szCSDVersion: [WCHAR; 128] = [0; 128];
        let i_res: LONG;
        let mut Length: USHORT = (szCSDVersion.len() - 1) as USHORT;
        i_res = _snwprintf(
            szCSDVersion.as_mut_ptr(),
            Length as usize,
            w!("Service Pack %d").as_ptr() as PCWSTR,
            (((*Peb).OSCSDVersion >> 8) & 0xFF) as ULONG,
        );
        if i_res < 0 {
            szCSDVersion[Length as usize] = 0;
        }

        Length = Length * core::mem::size_of::<WCHAR>() as USHORT;
        (*Peb).CSDVersion.Buffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            (Length + core::mem::size_of::<WCHAR>() as USHORT) as SIZE_T,
        ) as PWSTR;
        if !(*Peb).CSDVersion.Buffer.is_null() {
            (*Peb).CSDVersion.Length = Length;
            (*Peb).CSDVersion.MaximumLength = Length + core::mem::size_of::<WCHAR>() as USHORT;

            RtlCopyMemory(
                (*Peb).CSDVersion.Buffer as PVOID,
                szCSDVersion.as_ptr() as *const c_void,
                (*Peb).CSDVersion.MaximumLength as usize,
            );
            *(*Peb)
                .CSDVersion
                .Buffer
                .add(((*Peb).CSDVersion.Length as usize) / core::mem::size_of::<WCHAR>()) = 0;
        }
    }

    /* Check if we had Shim Data */
    if !OldShimData.is_null() {
        (*Peb).AppCompatInfo = core::ptr::null_mut();
        LdrpLoadShimEngine(OldShimData as PWCHAR, &mut ImagePathName, OldShimData);
    }

    /* Now call the Init Routines */
    Status = LdrpRunInitializeRoutines(Context);
    if !NT_SUCCESS(Status) {
        return Status;
    }

    /* WoW64 (Phase C) handoff — AFTER init routines so kernel32 (pulled in by
     * the CPU dll libwow64fex) is already initialized at the normal time.
     * Loads wow64.dll and calls Wow64LdrpInitialize, which never returns (it
     * drives the i386 side under emulation). Everything below here in normal
     * process init — the i386 exe's entry, etc. — is the 32-bit loader's job
     * inside emulation, not ours. Mirrors Win10's LdrpLoadWow64. */
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    if IsWow64Process {
        DbgPrint(b"[LDR] WoW64 process: init routines done, loading wow64.dll\n\0".as_ptr() as *const i8);
        return LdrpEnterGuestLayer(
            core::ptr::addr_of_mut!(Wow64DllString),
            b"Wow64LdrpInitialize\0",
            Context,
        );
    }

    /* wowx64 handoff: the same shape for an x86-64 guest (see the
     * IsX64GuestProcess note above). wowx64!WowX64LdrpInitialize loads the
     * FEX CPU layer and runs the guest from the image entry; it never
     * returns. */
    #[cfg(target_arch = "aarch64")]
    if IsX64GuestProcess {
        DbgPrint(b"[LDR] x86-64 guest process: init routines done, loading wowx64.dll\n\0".as_ptr() as *const i8);
        return LdrpEnterGuestLayer(
            core::ptr::addr_of_mut!(WowX64DllString),
            b"WowX64LdrpInitialize\0",
            Context,
        );
    }

    /* Notify Shim Engine */
    if g_ShimsEnabled != 0 {
        type SeInstallAfterInitFn =
            unsafe extern "system" fn(PUNICODE_STRING, PVOID);
        let SE_InstallAfterInit: SeInstallAfterInitFn =
            core::mem::transmute(RtlDecodeSystemPointer(g_pfnSE_InstallAfterInit));
        SE_InstallAfterInit(&mut ImagePathName, OldShimData);
    }

    /* Check if we have a user-defined Post Process Routine */
    if NT_SUCCESS(Status) {
        if let Some(PostProcessInit) = (*Peb).PostProcessInitRoutine {
            PostProcessInit();
        }
    }

    /* Close the key if we have one opened */
    if !OptionsKey.is_null() {
        NtClose(OptionsKey);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn LdrpInitFailure(Status: NTSTATUS) {
    let mut Response: ULONG = 0;
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;

    let _ = Peb;

    if LdrpFatalHardErrorCount == 0 {
        ZwRaiseHardError(
            STATUS_APP_INIT_FAILURE,
            1,
            0,
            core::ptr::addr_of!(Status) as PULONG_PTR,
            OptionOk,
            &mut Response,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn LdrpInit(
    Context: PCONTEXT,
    SystemArgument1: PVOID,
    SystemArgument2: PVOID,
) {
    let mut Timeout: LARGE_INTEGER = core::mem::zeroed();
    let Teb: PTEB = NtCurrentTeb();
    let Status: NTSTATUS;
    let mut LoaderStatus: NTSTATUS = STATUS_SUCCESS;
    let mut MemoryBasicInfo: MEMORY_BASIC_INFORMATION = core::mem::zeroed();
    let Peb: PPEB = (*NtCurrentTeb()).ProcessEnvironmentBlock;

    let _ = SystemArgument2;

    #[cfg(target_pointer_width = "64")]
    {
        RtlpUse16ByteSLists = (*SharedUserData).ProcessorFeatures[PF_COMPARE_EXCHANGE128 as usize];
    }

    if (*Teb).DeallocationStack.is_null() {
        Status = NtQueryVirtualMemory(
            -1isize as HANDLE,
            (*Teb).NtTib.StackLimit,
            MemoryBasicInformation,
            core::ptr::addr_of_mut!(MemoryBasicInfo) as PVOID,
            core::mem::size_of::<MEMORY_BASIC_INFORMATION>() as _,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            LdrpInitFailure(Status);
            RtlRaiseStatus(Status);
            return;
        }

        (*Teb).DeallocationStack = MemoryBasicInfo.AllocationBase;
    }

    while _InterlockedCompareExchange(core::ptr::addr_of_mut!(LdrpProcessInitialized), 1, 0) == 1 {
        Timeout.QuadPart = (30i64) * (-10000i64);

        while LdrpProcessInitialized == 1 {
            ZwDelayExecution(0u8, core::ptr::addr_of_mut!(Timeout));
        }
    }

    let IsInitialThread = crate::ldr::ldrinit::LdrpLdrData().is_null();
    if IsInitialThread {
        (*Peb).LoaderLock = core::ptr::addr_of_mut!(LdrpLoaderLock);

        LdrpInLdrInit = 1u8;

        LoaderStatus = LdrpInitializeProcess(Context, SystemArgument1);

        if NT_SUCCESS(LoaderStatus) && (*Peb).MinimumStackCommit != 0 {
            // UNIMPLEMENTED
        }

        LdrpInLdrInit = 0u8;

        if NT_SUCCESS(LoaderStatus) {
            _InterlockedIncrement(core::ptr::addr_of_mut!(LdrpProcessInitialized));

            /* FEX bring-up rung 0e: user-mode fault-delivery self-test, ARMED
             * as a permanent arm64 regression tripwire (both probes PASS + boot
             * green, 2026-07-08 — the earlier "destabilizes smss" wedge was the
             * RtlUnwindEx missing ContextRecord sync, fixed in rtl-except).
             * Gated to smss.exe; a regression prints [SEH-POC-U] FAIL/CORRUPT
             * on serial or kills smss, which the arm64 smoke catches. Results:
             * docs/engineering/fex-bringup-plan.md rung 0e. arm64 only (the
             * shim is arch-gated in ntdll-cargo/build.rs). */
            #[cfg(target_arch = "aarch64")] // nolint: cfg-vs-c (arm64 SEH probe (fex-bringup-plan rung 0e), Rust-only)
            {
                const SEH_POC_U_ARMED: bool = true;
                let pp = (*Peb).ProcessParameters;
                if SEH_POC_U_ARMED
                    && !pp.is_null()
                    && unicode_tail_is_smss(
                        (*pp).ImagePathName.Buffer,
                        ((*pp).ImagePathName.Length / 2) as usize,
                    )
                {
                    extern "C" {
                        fn SehPocUserRun();
                    }
                    SehPocUserRun();
                }
            }
        }
    } else {
        if (*Peb).InheritedAddressSpace != 0 {
            LoaderStatus = STATUS_NOT_IMPLEMENTED;
        } else {
            LdrpInitializeThread(Context);
        }
    }

    NtTestAlert();

    if !NT_SUCCESS(LoaderStatus) {
        LdrpInitFailure(LoaderStatus);
        RtlRaiseStatus(LoaderStatus);
    }

    /* A thread (never the initial one: that is the guest layer's from process
     * init) of a WoW64 process whose start is 32-bit code. */
    #[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
    if LdrpIsWow64Process && !IsInitialThread && NT_SUCCESS(LoaderStatus) {
        LdrpDivertWow64Thread(Context);
    }
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS NT User-Mode Library
//  * FILE:            dll/ntdll/ldr/ldrinit.c
//  * PURPOSE:         User-Mode Process/Thread Startup
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  *                  Aleksey Bragin (aleksey@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntdll.h>
// #include <compat_undoc.h>
// #include <compatguid_undoc.h>
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* GLOBALS *******************************************************************/
//
// HANDLE ImageExecOptionsKey;
// HANDLE Wow64ExecOptionsKey;
// UNICODE_STRING ImageExecOptionsString = RTL_CONSTANT_STRING(L"\\Registry\\Machine\\Software\\Microsoft\\Windows NT\\CurrentVersion\\Image File Execution Options");
// UNICODE_STRING Wow64OptionsString = RTL_CONSTANT_STRING(L"");
// UNICODE_STRING NtDllString = RTL_CONSTANT_STRING(L"ntdll.dll");
// UNICODE_STRING Kernel32String = RTL_CONSTANT_STRING(L"kernel32.dll");
// const UNICODE_STRING LdrpDotLocal = RTL_CONSTANT_STRING(L".Local");
//
// BOOLEAN LdrpInLdrInit;
// LONG LdrpProcessInitialized;
// BOOLEAN LdrpLoaderLockInit;
// BOOLEAN LdrpLdrDatabaseIsSetup;
// BOOLEAN LdrpShutdownInProgress;
// HANDLE LdrpShutdownThreadId;
//
// BOOLEAN LdrpDllValidation;
//
// PLDR_DATA_TABLE_ENTRY LdrpImageEntry;
// PUNICODE_STRING LdrpTopLevelDllBeingLoaded;
// WCHAR StringBuffer[156];
// extern PTEB LdrpTopLevelDllBeingLoadedTeb; // defined in rtlsupp.c!
// PLDR_DATA_TABLE_ENTRY LdrpCurrentDllInitializer;
// PLDR_DATA_TABLE_ENTRY LdrpNtDllDataTableEntry;
//
// static NTSTATUS (WINAPI *Kernel32ProcessInitPostImportFunction)(VOID);
// static BOOL (WINAPI *Kernel32BaseQueryModuleData)(IN LPSTR ModuleName, IN LPSTR Unk1, IN PVOID Unk2, IN PVOID Unk3, IN PVOID Unk4);
//
// RTL_BITMAP TlsBitMap;
// RTL_BITMAP TlsExpansionBitMap;
// RTL_BITMAP FlsBitMap;
// BOOLEAN LdrpImageHasTls;
// LIST_ENTRY LdrpTlsList;
// ULONG LdrpNumberOfTlsEntries;
// ULONG LdrpNumberOfProcessors;
// PVOID NtDllBase;
// extern LARGE_INTEGER RtlpTimeout;
// extern BOOLEAN RtlpTimeoutDisable;
// LIST_ENTRY LdrpHashTable[LDR_HASH_TABLE_ENTRIES];
// HANDLE LdrpKnownDllObjectDirectory;
// UNICODE_STRING LdrpKnownDllPath;
// WCHAR LdrpKnownDllPathBuffer[128];
// UNICODE_STRING LdrpDefaultPath;
//
// PEB_LDR_DATA PebLdr;
//
// RTL_CRITICAL_SECTION_DEBUG LdrpLoaderLockDebug;
// RTL_CRITICAL_SECTION LdrpLoaderLock =
// {
//     &LdrpLoaderLockDebug,
//     -1,
//     0,
//     0,
//     0,
//     0
// };
// RTL_CRITICAL_SECTION FastPebLock;
//
// BOOLEAN ShowSnaps;
//
// ULONG LdrpFatalHardErrorCount;
// ULONG LdrpActiveUnloadCount;
//
// //extern LIST_ENTRY RtlCriticalSectionList;
//
// VOID NTAPI RtlpInitializeVectoredExceptionHandling(VOID);
// VOID NTAPI RtlpInitDeferredCriticalSection(VOID);
// VOID NTAPI RtlInitializeHeapManager(VOID);
// NTSTATUS NTAPI RtlpInitializeLocaleTable(VOID);
//
// ULONG RtlpDisableHeapLookaside; // TODO: Move to heap.c
// ULONG RtlpShutdownProcessFlags; // TODO: Use it
//
// NTSTATUS LdrPerformRelocations(PIMAGE_NT_HEADERS NTHeaders, PVOID ImageBase);
// NTSTATUS NTAPI RtlpInitializeActCtx(PVOID* pOldShimData);
// extern BOOLEAN RtlpUse16ByteSLists;
//
// #ifdef _WIN64
// #define DEFAULT_SECURITY_COOKIE 0x00002B992DDFA232ll
// #else
// #define DEFAULT_SECURITY_COOKIE 0xBB40E64E
// #endif
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LdrOpenImageFileOptionsKey(
//     _In_ PUNICODE_STRING SubKey,
//     _In_ BOOLEAN Wow64,
//     _Out_ PHANDLE NewKeyHandle)
// {
//     PHANDLE RootKeyLocation;
//     HANDLE RootKey;
//     UNICODE_STRING SubKeyString;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     PWCHAR p1;
//
//     /* Check which root key to open */
//     if (Wow64)
//         RootKeyLocation = &Wow64ExecOptionsKey;
//     else
//         RootKeyLocation = &ImageExecOptionsKey;
//
//     /* Get the current key */
//     RootKey = *RootKeyLocation;
//
//     /* Setup the object attributes */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                Wow64 ?
//                                &Wow64OptionsString : &ImageExecOptionsString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//
//     /* Open the root key */
//     Status = ZwOpenKey(&RootKey, KEY_ENUMERATE_SUB_KEYS, &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         /* Write the key handle */
//         if (InterlockedCompareExchangePointer(RootKeyLocation, RootKey, NULL) != NULL)
//         {
//             /* Someone already opened it, use it instead */
//             NtClose(RootKey);
//             RootKey = *RootKeyLocation;
//         }
//
//         /* Extract the name */
//         SubKeyString = *SubKey;
//         p1 = (PWCHAR)((ULONG_PTR)SubKeyString.Buffer + SubKeyString.Length);
//         while (SubKeyString.Length)
//         {
//             if (p1[-1] == L'\\') break;
//             p1--;
//             SubKeyString.Length -= sizeof(*p1);
//         }
//         SubKeyString.Buffer = p1;
//         SubKeyString.Length = SubKey->Length - SubKeyString.Length;
//
//         /* Setup the object attributes */
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &SubKeyString,
//                                    OBJ_CASE_INSENSITIVE,
//                                    RootKey,
//                                    NULL);
//
//         /* Open the setting key */
//         Status = ZwOpenKey((PHANDLE)NewKeyHandle, GENERIC_READ, &ObjectAttributes);
//     }
//
//     /* Return to caller */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LdrQueryImageFileKeyOption(
//     _In_ HANDLE KeyHandle,
//     _In_ PCWSTR ValueName,
//     _In_ ULONG Type,
//     _Out_ PVOID Buffer,
//     _In_ ULONG BufferSize,
//     _Out_opt_ PULONG ReturnedLength)
// {
//     ULONG KeyInfo[256];
//     UNICODE_STRING ValueNameString, IntegerString;
//     ULONG KeyInfoSize, ResultSize;
//     PKEY_VALUE_PARTIAL_INFORMATION KeyValueInformation = (PKEY_VALUE_PARTIAL_INFORMATION)&KeyInfo;
//     BOOLEAN FreeHeap = FALSE;
//     NTSTATUS Status;
//
//     /* Build a string for the value name */
//     Status = RtlInitUnicodeStringEx(&ValueNameString, ValueName);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Query the value */
//     Status = ZwQueryValueKey(KeyHandle,
//                              &ValueNameString,
//                              KeyValuePartialInformation,
//                              KeyValueInformation,
//                              sizeof(KeyInfo),
//                              &ResultSize);
//     if (Status == STATUS_BUFFER_OVERFLOW)
//     {
//         /* Our local buffer wasn't enough, allocate one */
//         KeyInfoSize = sizeof(KEY_VALUE_PARTIAL_INFORMATION) +
//                       KeyValueInformation->DataLength;
//         KeyValueInformation = RtlAllocateHeap(RtlGetProcessHeap(),
//                                               0,
//                                               KeyInfoSize);
//         if (KeyValueInformation != NULL)
//         {
//             /* Try again */
//             Status = ZwQueryValueKey(KeyHandle,
//                                      &ValueNameString,
//                                      KeyValuePartialInformation,
//                                      KeyValueInformation,
//                                      KeyInfoSize,
//                                      &ResultSize);
//             FreeHeap = TRUE;
//         }
//         else
//         {
//             /* Give up this time */
//             Status = STATUS_NO_MEMORY;
//         }
//     }
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Handle binary data */
//         if (KeyValueInformation->Type == REG_BINARY)
//         {
//             /* Check validity */
//             if ((Buffer) && (KeyValueInformation->DataLength <= BufferSize))
//             {
//                 /* Copy into buffer */
//                 RtlMoveMemory(Buffer,
//                               &KeyValueInformation->Data,
//                               KeyValueInformation->DataLength);
//             }
//             else
//             {
//                 Status = STATUS_BUFFER_OVERFLOW;
//             }
//
//             /* Copy the result length */
//             if (ReturnedLength) *ReturnedLength = KeyValueInformation->DataLength;
//         }
//         else if (KeyValueInformation->Type == REG_DWORD)
//         {
//             /* Check for valid type */
//             if (KeyValueInformation->Type != Type)
//             {
//                 /* Error */
//                 Status = STATUS_OBJECT_TYPE_MISMATCH;
//             }
//             else
//             {
//                 /* Check validity */
//                 if ((Buffer) &&
//                     (BufferSize == sizeof(ULONG)) &&
//                     (KeyValueInformation->DataLength <= BufferSize))
//                 {
//                     /* Copy into buffer */
//                     RtlMoveMemory(Buffer,
//                                   &KeyValueInformation->Data,
//                                   KeyValueInformation->DataLength);
//                 }
//                 else
//                 {
//                     Status = STATUS_BUFFER_OVERFLOW;
//                 }
//
//                 /* Copy the result length */
//                 if (ReturnedLength) *ReturnedLength = KeyValueInformation->DataLength;
//             }
//         }
//         else if (KeyValueInformation->Type != REG_SZ)
//         {
//             /* We got something weird */
//             Status = STATUS_OBJECT_TYPE_MISMATCH;
//         }
//         else
//         {
//             /*  String, check what you requested */
//             if (Type == REG_DWORD)
//             {
//                 /* Validate */
//                 if (BufferSize != sizeof(ULONG))
//                 {
//                     /* Invalid size */
//                     BufferSize = 0;
//                     Status = STATUS_INFO_LENGTH_MISMATCH;
//                 }
//                 else
//                 {
//                     /* OK, we know what you want... */
//                     IntegerString.Buffer = (PWSTR)KeyValueInformation->Data;
//                     IntegerString.Length = (USHORT)KeyValueInformation->DataLength -
//                                            sizeof(WCHAR);
//                     IntegerString.MaximumLength = (USHORT)KeyValueInformation->DataLength;
//                     Status = RtlUnicodeStringToInteger(&IntegerString, 0, (PULONG)Buffer);
//                 }
//             }
//             else
//             {
//                 /* Validate */
//                 if (KeyValueInformation->DataLength > BufferSize)
//                 {
//                     /* Invalid */
//                     Status = STATUS_BUFFER_OVERFLOW;
//                 }
//                 else
//                 {
//                     /* Set the size */
//                     BufferSize = KeyValueInformation->DataLength;
//                 }
//
//                 /* Copy the string */
//                 RtlMoveMemory(Buffer, &KeyValueInformation->Data, BufferSize);
//             }
//
//             /* Copy the result length */
//             if (ReturnedLength) *ReturnedLength = KeyValueInformation->DataLength;
//         }
//     }
//
//     /* Check if buffer was in heap */
//     if (FreeHeap) RtlFreeHeap(RtlGetProcessHeap(), 0, KeyValueInformation);
//
//     /* Return status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LdrQueryImageFileExecutionOptionsEx(
//     _In_ PUNICODE_STRING SubKey,
//     _In_ PCWSTR ValueName,
//     _In_ ULONG Type,
//     _Out_ PVOID Buffer,
//     _In_ ULONG BufferSize,
//     _Out_opt_ PULONG ReturnedLength,
//     _In_ BOOLEAN Wow64)
// {
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//
//     /* Open a handle to the key */
//     Status = LdrOpenImageFileOptionsKey(SubKey, Wow64, &KeyHandle);
//
//     /* Check for success */
//     if (NT_SUCCESS(Status))
//     {
//         /* Query the data */
//         Status = LdrQueryImageFileKeyOption(KeyHandle,
//                                             ValueName,
//                                             Type,
//                                             Buffer,
//                                             BufferSize,
//                                             ReturnedLength);
//
//         /* Close the key */
//         NtClose(KeyHandle);
//     }
//
//     /* Return to caller */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LdrQueryImageFileExecutionOptions(
//     _In_ PUNICODE_STRING SubKey,
//     _In_ PCWSTR ValueName,
//     _In_ ULONG Type,
//     _Out_ PVOID Buffer,
//     _In_ ULONG BufferSize,
//     _Out_opt_ PULONG ReturnedLength)
// {
//     /* Call the newer function */
//     return LdrQueryImageFileExecutionOptionsEx(SubKey,
//                                                ValueName,
//                                                Type,
//                                                Buffer,
//                                                BufferSize,
//                                                ReturnedLength,
//                                                FALSE);
// }
//
// VOID
// NTAPI
// LdrpEnsureLoaderLockIsHeld(VOID)
// {
//     // Ignored atm
// }
//
// PVOID
// NTAPI
// LdrpFetchAddressOfSecurityCookie(PVOID BaseAddress, ULONG SizeOfImage)
// {
//     PIMAGE_LOAD_CONFIG_DIRECTORY ConfigDir;
//     ULONG DirSize;
//     PVOID Cookie = NULL;
//
//     /* Check NT header first */
//     if (!RtlImageNtHeader(BaseAddress)) return NULL;
//
//     /* Get the pointer to the config directory */
//     ConfigDir = RtlImageDirectoryEntryToData(BaseAddress,
//                                              TRUE,
//                                              IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG,
//                                              &DirSize);
//
//     /* Check for sanity */
//     if (!ConfigDir ||
//         (DirSize != 64 && ConfigDir->Size != DirSize) ||
//         (ConfigDir->Size < 0x48))
//         return NULL;
//
//     /* Now get the cookie */
//     Cookie = (PVOID)ConfigDir->SecurityCookie;
//
//     /* Check this cookie */
//     if ((PCHAR)Cookie <= (PCHAR)BaseAddress ||
//         (PCHAR)Cookie >= (PCHAR)BaseAddress + SizeOfImage)
//     {
//         Cookie = NULL;
//     }
//
//     /* Return validated security cookie */
//     return Cookie;
// }
//
// PVOID
// NTAPI
// LdrpInitSecurityCookie(PLDR_DATA_TABLE_ENTRY LdrEntry)
// {
//     PULONG_PTR Cookie;
//     LARGE_INTEGER Counter;
//     ULONG_PTR NewCookie;
//
//     /* Fetch address of the cookie */
//     Cookie = LdrpFetchAddressOfSecurityCookie(LdrEntry->DllBase, LdrEntry->SizeOfImage);
//
//     if (Cookie)
//     {
//         /* Check if it's a default one */
//         if ((*Cookie == DEFAULT_SECURITY_COOKIE) ||
//             (*Cookie == 0xBB40))
//         {
//             /* Make up a cookie from a bunch of values which may uniquely represent
//                current moment of time, environment, etc */
//             NtQueryPerformanceCounter(&Counter, NULL);
//
//             NewCookie = Counter.LowPart ^ Counter.HighPart;
//             NewCookie ^= (ULONG_PTR)NtCurrentTeb()->ClientId.UniqueProcess;
//             NewCookie ^= (ULONG_PTR)NtCurrentTeb()->ClientId.UniqueThread;
//
//             /* Loop like it's done in KeQueryTickCount(). We don't want to call it directly. */
//             while (SharedUserData->SystemTime.High1Time != SharedUserData->SystemTime.High2Time)
//             {
//                 YieldProcessor();
//             };
//
//             /* Calculate the milliseconds value and xor it to the cookie */
//             NewCookie ^= Int64ShrlMod32(UInt32x32To64(SharedUserData->TickCountMultiplier, SharedUserData->TickCount.LowPart), 24) +
//                 (SharedUserData->TickCountMultiplier * (SharedUserData->TickCount.High1Time << 8));
//
//             /* Make the cookie 16bit if necessary */
//             if (*Cookie == 0xBB40) NewCookie &= 0xFFFF;
//
//             /* If the result is 0 or the same as we got, just subtract one from the existing value
//                and that's it */
//             if ((NewCookie == 0) || (NewCookie == *Cookie))
//             {
//                 NewCookie = *Cookie - 1;
//             }
//
//             /* Set the new cookie value */
//             *Cookie = NewCookie;
//         }
//     }
//
//     return Cookie;
// }
//
// VOID
// NTAPI
// LdrpInitializeThread(IN PCONTEXT Context)
// {
//     PPEB Peb = NtCurrentPeb();
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PLIST_ENTRY NextEntry, ListHead;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED ActCtx;
//     NTSTATUS Status;
//     PVOID EntryPoint;
//
//     DPRINT("LdrpInitializeThread() called for %wZ (%p/%p)\n",
//             &LdrpImageEntry->BaseDllName,
//             NtCurrentTeb()->RealClientId.UniqueProcess,
//             NtCurrentTeb()->RealClientId.UniqueThread);
//
//     /* Acquire the loader Lock */
//     RtlEnterCriticalSection(&LdrpLoaderLock);
//
//     /* Allocate an Activation Context Stack */
//     DPRINT("ActivationContextStack %p\n", NtCurrentTeb()->ActivationContextStackPointer);
//     Status = RtlAllocateActivationContextStack(&NtCurrentTeb()->ActivationContextStackPointer);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Warning: Unable to allocate ActivationContextStack\n");
//     }
//
//     /* Make sure we are not shutting down */
//     if (LdrpShutdownInProgress) goto Exit;
//
//     /* Allocate TLS */
//     LdrpAllocateTls();
//
//     /* Start at the beginning */
//     ListHead = &Peb->Ldr->InMemoryOrderModuleList;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         /* Get the current entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry, LDR_DATA_TABLE_ENTRY, InMemoryOrderLinks);
//
//         /* Make sure it's not ourselves */
//         if (Peb->ImageBaseAddress != LdrEntry->DllBase)
//         {
//             /* Check if we should call */
//             if (!(LdrEntry->Flags & LDRP_DONT_CALL_FOR_THREADS))
//             {
//                 /* Get the entrypoint */
//                 EntryPoint = LdrEntry->EntryPoint;
//
//                 /* Check if we are ready to call it */
//                 if ((EntryPoint) &&
//                     (LdrEntry->Flags & LDRP_PROCESS_ATTACH_CALLED) &&
//                     (LdrEntry->Flags & LDRP_IMAGE_DLL))
//                 {
//                     /* Set up the Act Ctx */
//                     ActCtx.Size = sizeof(ActCtx);
//                     ActCtx.Format = 1;
//                     RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//                     /* Activate the ActCtx */
//                     RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                                            LdrEntry->EntryPointActivationContext);
//
//                     _SEH2_TRY
//                     {
//                         /* Check if it has TLS */
//                         if (LdrEntry->TlsIndex)
//                         {
//                             /* Make sure we're not shutting down */
//                             if (!LdrpShutdownInProgress)
//                             {
//                                 /* Call TLS */
//                                 LdrpCallTlsInitializers(LdrEntry, DLL_THREAD_ATTACH);
//                             }
//                         }
//
//                         /* Make sure we're not shutting down */
//                         if (!LdrpShutdownInProgress)
//                         {
//                             /* Call the Entrypoint */
//                             DPRINT("%wZ - Calling entry point at %p for thread attaching, %p/%p\n",
//                                    &LdrEntry->BaseDllName, LdrEntry->EntryPoint,
//                                    NtCurrentTeb()->RealClientId.UniqueProcess,
//                                    NtCurrentTeb()->RealClientId.UniqueThread);
//                             LdrpCallInitRoutine(LdrEntry->EntryPoint,
//                                                 LdrEntry->DllBase,
//                                                 DLL_THREAD_ATTACH,
//                                                 NULL);
//                         }
//                     }
//                     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                     {
//                         DPRINT1("WARNING: Exception 0x%x during LdrpCallInitRoutine(DLL_THREAD_ATTACH) for %wZ\n",
//                                 _SEH2_GetExceptionCode(), &LdrEntry->BaseDllName);
//                     }
//                     _SEH2_END;
//
//                     /* Deactivate the ActCtx */
//                     RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//                 }
//             }
//         }
//
//         /* Next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Check for TLS */
//     if (LdrpImageHasTls && !LdrpShutdownInProgress)
//     {
//         /* Set up the Act Ctx */
//         ActCtx.Size = sizeof(ActCtx);
//         ActCtx.Format = 1;
//         RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//         /* Activate the ActCtx */
//         RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                                LdrpImageEntry->EntryPointActivationContext);
//
//         _SEH2_TRY
//         {
//             /* Do TLS callbacks */
//             LdrpCallTlsInitializers(LdrpImageEntry, DLL_THREAD_ATTACH);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Do nothing */
//         }
//         _SEH2_END;
//
//         /* Deactivate the ActCtx */
//         RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//     }
//
// Exit:
//
//     /* Release the loader lock */
//     RtlLeaveCriticalSection(&LdrpLoaderLock);
//
//     DPRINT("LdrpInitializeThread() done\n");
// }
//
// NTSTATUS
// NTAPI
// LdrpRunInitializeRoutines(IN PCONTEXT Context OPTIONAL)
// {
//     PLDR_DATA_TABLE_ENTRY LocalArray[16];
//     PLIST_ENTRY ListHead;
//     PLIST_ENTRY NextEntry;
//     PLDR_DATA_TABLE_ENTRY LdrEntry, *LdrRootEntry, OldInitializer;
//     PVOID EntryPoint;
//     ULONG Count, i;
//     //ULONG BreakOnInit;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PPEB Peb = NtCurrentPeb();
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED ActCtx;
//     ULONG BreakOnDllLoad;
//     PTEB OldTldTeb;
//     BOOLEAN DllStatus;
//
//     DPRINT("LdrpRunInitializeRoutines() called for %wZ (%p/%p)\n",
//         &LdrpImageEntry->BaseDllName,
//         NtCurrentTeb()->RealClientId.UniqueProcess,
//         NtCurrentTeb()->RealClientId.UniqueThread);
//
//     /* Check the Loader Lock */
//     LdrpEnsureLoaderLockIsHeld();
//
//      /* Get the number of entries to call */
//     if ((Count = LdrpClearLoadInProgress()))
//     {
//         /* Check if we can use our local buffer */
//         if (Count > 16)
//         {
//             /* Allocate space for all the entries */
//             LdrRootEntry = RtlAllocateHeap(LdrpHeap,
//                                            0,
//                                            Count * sizeof(*LdrRootEntry));
//             if (!LdrRootEntry) return STATUS_NO_MEMORY;
//         }
//         else
//         {
//             /* Use our local array */
//             LdrRootEntry = LocalArray;
//         }
//     }
//     else
//     {
//         /* Don't need one */
//         LdrRootEntry = NULL;
//     }
//
//     /* Show debug message */
//     if (ShowSnaps)
//     {
//         DPRINT1("[%p,%p] LDR: Real INIT LIST for Process %wZ\n",
//                 NtCurrentTeb()->RealClientId.UniqueThread,
//                 NtCurrentTeb()->RealClientId.UniqueProcess,
//                 &Peb->ProcessParameters->ImagePathName);
//     }
//
//     /* Loop in order */
//     ListHead = &Peb->Ldr->InInitializationOrderModuleList;
//     NextEntry = ListHead->Flink;
//     i = 0;
//     while (NextEntry != ListHead)
//     {
//         /* Get the Data Entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry, LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks);
//
//         /* Check if we have a Root Entry */
//         if (LdrRootEntry)
//         {
//             /* Check flags */
//             if (!(LdrEntry->Flags & LDRP_ENTRY_PROCESSED))
//             {
//                 /* Setup the Cookie for the DLL */
//                 LdrpInitSecurityCookie(LdrEntry);
//
//                 /* Check for valid entrypoint */
//                 if (LdrEntry->EntryPoint)
//                 {
//                     /* Write in array */
//                     ASSERT(i < Count);
//                     LdrRootEntry[i] = LdrEntry;
//
//                     /* Display debug message */
//                     if (ShowSnaps)
//                     {
//                         DPRINT1("[%p,%p] LDR: %wZ init routine %p\n",
//                                 NtCurrentTeb()->RealClientId.UniqueThread,
//                                 NtCurrentTeb()->RealClientId.UniqueProcess,
//                                 &LdrEntry->FullDllName,
//                                 LdrEntry->EntryPoint);
//                     }
//                     i++;
//                 }
//             }
//         }
//
//         /* Set the flag */
//         LdrEntry->Flags |= LDRP_ENTRY_PROCESSED;
//         NextEntry = NextEntry->Flink;
//     }
//
//     Status = STATUS_SUCCESS;
//
//     /* If we got a context, then we have to call Kernel32 for TS support */
//     if (Context)
//     {
//         /* Check if we have one */
//         if (Kernel32ProcessInitPostImportFunction)
//         {
//             /* Call it */
//             Status = Kernel32ProcessInitPostImportFunction();
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("LDR: LdrpRunInitializeRoutines - Failed running kernel32 post-import function, Status=0x%08lx\n", Status);
//             }
//         }
//         /* Clear it */
//         Kernel32ProcessInitPostImportFunction = NULL;
//     }
//
//     /* No root entry? return */
//     if (!LdrRootEntry)
//         return Status;
//
//     /* Set the TLD TEB */
//     OldTldTeb = LdrpTopLevelDllBeingLoadedTeb;
//     LdrpTopLevelDllBeingLoadedTeb = NtCurrentTeb();
//
//     /* Loop */
//     i = 0;
//     while (i < Count)
//     {
//         /* Get an entry */
//         LdrEntry = LdrRootEntry[i];
//
//         /* FIXME: Verify NX Compat */
//
//         /* Move to next entry */
//         i++;
//
//         /* Get its entrypoint */
//         EntryPoint = LdrEntry->EntryPoint;
//
//         /* Are we being debugged? */
//         BreakOnDllLoad = 0;
//         if (Peb->BeingDebugged || Peb->ReadImageFileExecOptions)
//         {
//             /* Check if we should break on load */
//             Status = LdrQueryImageFileExecutionOptions(&LdrEntry->BaseDllName,
//                                                        L"BreakOnDllLoad",
//                                                        REG_DWORD,
//                                                        &BreakOnDllLoad,
//                                                        sizeof(ULONG),
//                                                        NULL);
//             if (!NT_SUCCESS(Status)) BreakOnDllLoad = 0;
//
//             /* Reset status back to STATUS_SUCCESS */
//             Status = STATUS_SUCCESS;
//         }
//
//         /* Break if aksed */
//         if (BreakOnDllLoad)
//         {
//             /* Check if we should show a message */
//             if (ShowSnaps)
//             {
//                 DPRINT1("LDR: %wZ loaded.", &LdrEntry->BaseDllName);
//                 DPRINT1(" - About to call init routine at %p\n", EntryPoint);
//             }
//
//             /* Break in debugger */
//             DbgBreakPoint();
//         }
//
//         /* Make sure we have an entrypoint */
//         if (EntryPoint)
//         {
//             /* Save the old Dll Initializer and write the current one */
//             OldInitializer = LdrpCurrentDllInitializer;
//             LdrpCurrentDllInitializer = LdrEntry;
//
//             /* Set up the Act Ctx */
//             ActCtx.Size = sizeof(ActCtx);
//             ActCtx.Format = 1;
//             RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//             /* Activate the ActCtx */
//             RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                                    LdrEntry->EntryPointActivationContext);
//
//             _SEH2_TRY
//             {
//                 /* Check if it has TLS */
//                 if (LdrEntry->TlsIndex && Context)
//                 {
//                     /* Call TLS */
//                     LdrpCallTlsInitializers(LdrEntry, DLL_PROCESS_ATTACH);
//                 }
//
//                 /* Call the Entrypoint */
//                 if (ShowSnaps)
//                 {
//                     DPRINT1("%wZ - Calling entry point at %p for DLL_PROCESS_ATTACH\n",
//                             &LdrEntry->BaseDllName, EntryPoint);
//                 }
//                 {
//                     CHAR dllNameA[64];
//                     ULONG i, len = LdrEntry->BaseDllName.Length / sizeof(WCHAR);
//                     if (len > 60) len = 60;
//                     for (i = 0; i < len; i++)
//                         dllNameA[i] = (CHAR)LdrEntry->BaseDllName.Buffer[i];
//                     dllNameA[len] = 0;
//                     DbgPrint("[LDR] DllMain(%s) ATTACH PID=%lu Base=%p\n", dllNameA,
//                              (ULONG)(ULONG_PTR)NtCurrentTeb()->ClientId.UniqueProcess,
//                              LdrEntry->DllBase);
//                 }
//                 DllStatus = LdrpCallInitRoutine(EntryPoint,
//                                                 LdrEntry->DllBase,
//                                                 DLL_PROCESS_ATTACH,
//                                                 Context);
//                 {
//                     CHAR dllNameA[64];
//                     ULONG i, len = LdrEntry->BaseDllName.Length / sizeof(WCHAR);
//                     if (len > 60) len = 60;
//                     for (i = 0; i < len; i++)
//                         dllNameA[i] = (CHAR)LdrEntry->BaseDllName.Buffer[i];
//                     dllNameA[len] = 0;
//                     DbgPrint("[LDR] DllMain(%s) ret=%d PID=%lu\n", dllNameA, DllStatus,
//                              (ULONG)(ULONG_PTR)NtCurrentTeb()->ClientId.UniqueProcess);
//                 }
//             }
//             _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//             {
//                 DllStatus = FALSE;
//                 DbgPrint("[LDR] EXCEPTION during DllMain PID=%lu\n",
//                         (ULONG)(ULONG_PTR)NtCurrentTeb()->ClientId.UniqueProcess);
//             }
//             _SEH2_END;
//
//             /* Deactivate the ActCtx */
//             RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//
//             /* Save the Current DLL Initializer */
//             LdrpCurrentDllInitializer = OldInitializer;
//
//             /* Mark the entry as processed */
//             LdrEntry->Flags |= LDRP_PROCESS_ATTACH_CALLED;
//
//             /* Fail if DLL init failed */
//             if (!DllStatus)
//             {
//                 DbgPrint("[LDR] FAILED PID=%lu\n",
//                     (ULONG)(ULONG_PTR)NtCurrentTeb()->ClientId.UniqueProcess);
//                 Status = STATUS_DLL_INIT_FAILED;
//                 goto Quickie;
//             }
//         }
//     }
//
//     /* Loop in order */
//     ListHead = &Peb->Ldr->InInitializationOrderModuleList;
//     NextEntry = NextEntry->Flink;
//     while (NextEntry != ListHead)
//     {
//         /* Get the Data Entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry, LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks);
//
//         /* FIXME: Verify NX Compat */
//         // LdrpCheckNXCompatibility()
//
//         /* Next entry */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Check for TLS */
//     if (LdrpImageHasTls && Context)
//     {
//         /* Set up the Act Ctx */
//         ActCtx.Size = sizeof(ActCtx);
//         ActCtx.Format = 1;
//         RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//         /* Activate the ActCtx */
//         RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                                LdrpImageEntry->EntryPointActivationContext);
//
//         _SEH2_TRY
//         {
//             /* Do TLS callbacks */
//             LdrpCallTlsInitializers(LdrpImageEntry, DLL_PROCESS_ATTACH);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//         }
//         _SEH2_END;
//
//         /* Deactivate the ActCtx */
//         RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//     }
//
// Quickie:
//     /* Restore old TEB */
//     LdrpTopLevelDllBeingLoadedTeb = OldTldTeb;
//
//     /* Check if the array is in the heap */
//     if (LdrRootEntry != LocalArray)
//     {
//         /* Free the array */
//         RtlFreeHeap(LdrpHeap, 0, LdrRootEntry);
//     }
//
//     /* Return to caller */
//     DPRINT("LdrpRunInitializeRoutines() done\n");
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LdrShutdownProcess(VOID)
// {
//     PPEB Peb = NtCurrentPeb();
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PLIST_ENTRY NextEntry, ListHead;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED ActCtx;
//     PVOID EntryPoint;
//
//     DPRINT("LdrShutdownProcess() called for %wZ\n", &LdrpImageEntry->BaseDllName);
//     if (LdrpShutdownInProgress) return STATUS_SUCCESS;
//
//     /* Tell the Shim Engine */
//     if (g_ShimsEnabled)
//     {
//         VOID(NTAPI *SE_ProcessDying)();
//         SE_ProcessDying = RtlDecodeSystemPointer(g_pfnSE_ProcessDying);
//         SE_ProcessDying();
//     }
//
//     /* Tell the world */
//     if (ShowSnaps)
//     {
//         DPRINT1("\n");
//     }
//
//     /* Set the shutdown variables */
//     LdrpShutdownThreadId = NtCurrentTeb()->RealClientId.UniqueThread;
//     LdrpShutdownInProgress = TRUE;
//
//     /* Enter the Loader Lock */
//     RtlEnterCriticalSection(&LdrpLoaderLock);
//
//     /* Cleanup trace logging data (Etw) */
//     if (SharedUserData->TraceLogging)
//     {
//         /* FIXME */
//         DPRINT1("We don't support Etw yet.\n");
//     }
//
//     /* Start at the end */
//     ListHead = &Peb->Ldr->InInitializationOrderModuleList;
//     NextEntry = ListHead->Blink;
//     while (NextEntry != ListHead)
//     {
//         /* Get the current entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry, LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks);
//         NextEntry = NextEntry->Blink;
//
//         /* Make sure it's not ourselves */
//         if (Peb->ImageBaseAddress != LdrEntry->DllBase)
//         {
//             /* Get the entrypoint */
//             EntryPoint = LdrEntry->EntryPoint;
//
//             /* Check if we are ready to call it */
//             if (EntryPoint &&
//                 (LdrEntry->Flags & LDRP_PROCESS_ATTACH_CALLED) &&
//                 LdrEntry->Flags)
//             {
//                 /* Set up the Act Ctx */
//                 ActCtx.Size = sizeof(ActCtx);
//                 ActCtx.Format = 1;
//                 RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//                 /* Activate the ActCtx */
//                 RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                                        LdrEntry->EntryPointActivationContext);
//
//                 _SEH2_TRY
//                 {
//                     /* Check if it has TLS */
//                     if (LdrEntry->TlsIndex)
//                     {
//                         /* Call TLS */
//                         LdrpCallTlsInitializers(LdrEntry, DLL_PROCESS_DETACH);
//                     }
//
//                     /* Call the Entrypoint */
//                     DPRINT("%wZ - Calling entry point at %p for thread detaching\n",
//                            &LdrEntry->BaseDllName, LdrEntry->EntryPoint);
//                     LdrpCallInitRoutine(EntryPoint,
//                                         LdrEntry->DllBase,
//                                         DLL_PROCESS_DETACH,
//                                         (PVOID)1);
//                 }
//                 _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                 {
//                     DPRINT1("WARNING: Exception 0x%x during LdrpCallInitRoutine(DLL_PROCESS_DETACH) for %wZ\n",
//                             _SEH2_GetExceptionCode(), &LdrEntry->BaseDllName);
//                 }
//                 _SEH2_END;
//
//                 /* Deactivate the ActCtx */
//                 RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//             }
//         }
//     }
//
//     /* Check for TLS */
//     if (LdrpImageHasTls)
//     {
//         /* Set up the Act Ctx */
//         ActCtx.Size = sizeof(ActCtx);
//         ActCtx.Format = 1;
//         RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//         /* Activate the ActCtx */
//         RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                                LdrpImageEntry->EntryPointActivationContext);
//
//         _SEH2_TRY
//         {
//             /* Do TLS callbacks */
//             LdrpCallTlsInitializers(LdrpImageEntry, DLL_PROCESS_DETACH);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Do nothing */
//         }
//         _SEH2_END;
//
//         /* Deactivate the ActCtx */
//         RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//     }
//
//     /* FIXME: Do Heap detection and Etw final shutdown */
//
//     /* Release the lock */
//     RtlLeaveCriticalSection(&LdrpLoaderLock);
//     DPRINT("LdrpShutdownProcess() done\n");
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// LdrShutdownThread(VOID)
// {
//     PPEB Peb = NtCurrentPeb();
//     PTEB Teb = NtCurrentTeb();
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PLIST_ENTRY NextEntry, ListHead;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_EXTENDED ActCtx;
//     PVOID EntryPoint;
//
//     DPRINT("LdrShutdownThread() called for %wZ\n",
//             &LdrpImageEntry->BaseDllName);
//
//     /* Cleanup trace logging data (Etw) */
//     if (SharedUserData->TraceLogging)
//     {
//         /* FIXME */
//         DPRINT1("We don't support Etw yet.\n");
//     }
//
//     /* Get the Ldr Lock */
//     RtlEnterCriticalSection(&LdrpLoaderLock);
//
//     /* Start at the end */
//     ListHead = &Peb->Ldr->InInitializationOrderModuleList;
//     NextEntry = ListHead->Blink;
//     while (NextEntry != ListHead)
//     {
//         /* Get the current entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry, LDR_DATA_TABLE_ENTRY, InInitializationOrderLinks);
//         NextEntry = NextEntry->Blink;
//
//         /* Make sure it's not ourselves */
//         if (Peb->ImageBaseAddress != LdrEntry->DllBase)
//         {
//             /* Check if we should call */
//             if (!(LdrEntry->Flags & LDRP_DONT_CALL_FOR_THREADS) &&
//                 (LdrEntry->Flags & LDRP_PROCESS_ATTACH_CALLED) &&
//                 (LdrEntry->Flags & LDRP_IMAGE_DLL))
//             {
//                 /* Get the entrypoint */
//                 EntryPoint = LdrEntry->EntryPoint;
//
//                 /* Check if we are ready to call it */
//                 if (EntryPoint)
//                 {
//                     /* Set up the Act Ctx */
//                     ActCtx.Size = sizeof(ActCtx);
//                     ActCtx.Format = 1;
//                     RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//                     /* Activate the ActCtx */
//                     RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                                            LdrEntry->EntryPointActivationContext);
//
//                     _SEH2_TRY
//                     {
//                         /* Check if it has TLS */
//                         if (LdrEntry->TlsIndex)
//                         {
//                             /* Make sure we're not shutting down */
//                             if (!LdrpShutdownInProgress)
//                             {
//                                 /* Call TLS */
//                                 LdrpCallTlsInitializers(LdrEntry, DLL_THREAD_DETACH);
//                             }
//                         }
//
//                         /* Make sure we're not shutting down */
//                         if (!LdrpShutdownInProgress)
//                         {
//                             /* Call the Entrypoint */
//                             DPRINT("%wZ - Calling entry point at %p for thread detaching\n",
//                                    &LdrEntry->BaseDllName, LdrEntry->EntryPoint);
//                             LdrpCallInitRoutine(EntryPoint,
//                                                 LdrEntry->DllBase,
//                                                 DLL_THREAD_DETACH,
//                                                 NULL);
//                         }
//                     }
//                     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//                     {
//                         DPRINT1("WARNING: Exception 0x%x during LdrpCallInitRoutine(DLL_THREAD_DETACH) for %wZ\n",
//                                 _SEH2_GetExceptionCode(), &LdrEntry->BaseDllName);
//                     }
//                     _SEH2_END;
//
//                     /* Deactivate the ActCtx */
//                     RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//                 }
//             }
//         }
//     }
//
//     /* Check for TLS */
//     if (LdrpImageHasTls)
//     {
//         /* Set up the Act Ctx */
//         ActCtx.Size = sizeof(ActCtx);
//         ActCtx.Format = 1;
//         RtlZeroMemory(&ActCtx.Frame, sizeof(RTL_ACTIVATION_CONTEXT_STACK_FRAME));
//
//         /* Activate the ActCtx */
//         RtlActivateActivationContextUnsafeFast(&ActCtx,
//                                                LdrpImageEntry->EntryPointActivationContext);
//
//         _SEH2_TRY
//         {
//             /* Do TLS callbacks */
//             LdrpCallTlsInitializers(LdrpImageEntry, DLL_THREAD_DETACH);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Do nothing */
//         }
//         _SEH2_END;
//
//         /* Deactivate the ActCtx */
//         RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//     }
//
//     /* Free TLS */
//     LdrpFreeTls();
//     RtlLeaveCriticalSection(&LdrpLoaderLock);
//
//     /* Check for expansion slots */
//     if (Teb->TlsExpansionSlots)
//     {
//         /* Free expansion slots */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Teb->TlsExpansionSlots);
//     }
//
//     /* Check for FLS Data */
//     if (Teb->FlsData)
//     {
//         /* Mimic BaseRundownFls */
//         ULONG n, FlsHighIndex;
//         PRTL_FLS_DATA pFlsData;
//         PFLS_CALLBACK_FUNCTION lpCallback;
//
//         pFlsData = Teb->FlsData;
//
//         RtlAcquirePebLock();
//         FlsHighIndex = NtCurrentPeb()->FlsHighIndex;
//         RemoveEntryList(&pFlsData->ListEntry);
//         RtlReleasePebLock();
//
//         for (n = 1; n <= FlsHighIndex; ++n)
//         {
//             lpCallback = NtCurrentPeb()->FlsCallback[n];
//             if (lpCallback && pFlsData->Data[n])
//             {
//                 lpCallback(pFlsData->Data[n]);
//             }
//         }
//
//         RtlFreeHeap(RtlGetProcessHeap(), 0, pFlsData);
//         Teb->FlsData = NULL;
//     }
//
//     /* Check for Fiber data */
//     if (Teb->HasFiberData)
//     {
//         /* Free Fiber data*/
//         RtlFreeHeap(RtlGetProcessHeap(), 0, Teb->NtTib.FiberData);
//         Teb->NtTib.FiberData = NULL;
//     }
//
//     /* Free the activation context stack */
//     RtlFreeThreadActivationContextStack();
//     DPRINT("LdrShutdownThread() done\n");
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// LdrpInitializeTls(VOID)
// {
//     PLIST_ENTRY NextEntry, ListHead;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PIMAGE_TLS_DIRECTORY TlsDirectory;
//     PLDRP_TLS_DATA TlsData;
//     ULONG Size;
//
//     /* Initialize the TLS List */
//     InitializeListHead(&LdrpTlsList);
//
//     /* Loop all the modules */
//     ListHead = &NtCurrentPeb()->Ldr->InLoadOrderModuleList;
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         /* Get the entry */
//         LdrEntry = CONTAINING_RECORD(NextEntry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//         NextEntry = NextEntry->Flink;
//
//         /* Get the TLS directory */
//         TlsDirectory = RtlImageDirectoryEntryToData(LdrEntry->DllBase,
//                                                     TRUE,
//                                                     IMAGE_DIRECTORY_ENTRY_TLS,
//                                                     &Size);
//
//         /* Check if we have a directory */
//         if (!TlsDirectory) continue;
//
//         /* Check if the image has TLS */
//         if (!LdrpImageHasTls) LdrpImageHasTls = TRUE;
//
//         /* Show debug message */
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: Tls Found in %wZ at %p\n",
//                     &LdrEntry->BaseDllName,
//                     TlsDirectory);
//         }
//
//         /* Allocate an entry */
//         TlsData = RtlAllocateHeap(RtlGetProcessHeap(), 0, sizeof(LDRP_TLS_DATA));
//         if (!TlsData) return STATUS_NO_MEMORY;
//
//         /* Lock the DLL and mark it for TLS Usage */
//         LdrEntry->LoadCount = -1;
//         LdrEntry->TlsIndex = -1;
//
//         /* Save the cached TLS data */
//         TlsData->TlsDirectory = *TlsDirectory;
//         InsertTailList(&LdrpTlsList, &TlsData->TlsLinks);
//
//         /* Update the index */
//         *(PLONG)TlsData->TlsDirectory.AddressOfIndex = LdrpNumberOfTlsEntries;
//         TlsData->TlsDirectory.Characteristics = LdrpNumberOfTlsEntries++;
//     }
//
//     /* Done setting up TLS, allocate entries */
//     return LdrpAllocateTls();
// }
//
// NTSTATUS
// NTAPI
// LdrpAllocateTls(VOID)
// {
//     PTEB Teb = NtCurrentTeb();
//     PLIST_ENTRY NextEntry, ListHead;
//     PLDRP_TLS_DATA TlsData;
//     SIZE_T TlsDataSize;
//     PVOID *TlsVector;
//
//     /* Check if we have any entries */
//     if (!LdrpNumberOfTlsEntries)
//         return STATUS_SUCCESS;
//
//     /* Allocate the vector array */
//     TlsVector = RtlAllocateHeap(RtlGetProcessHeap(),
//                                     0,
//                                     LdrpNumberOfTlsEntries * sizeof(PVOID));
//     if (!TlsVector) return STATUS_NO_MEMORY;
//     Teb->ThreadLocalStoragePointer = TlsVector;
//
//     /* Loop the TLS Array */
//     ListHead = &LdrpTlsList;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         /* Get the entry */
//         TlsData = CONTAINING_RECORD(NextEntry, LDRP_TLS_DATA, TlsLinks);
//         NextEntry = NextEntry->Flink;
//
//         /* Allocate this vector */
//         TlsDataSize = TlsData->TlsDirectory.EndAddressOfRawData -
//                       TlsData->TlsDirectory.StartAddressOfRawData;
//         TlsVector[TlsData->TlsDirectory.Characteristics] = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                                            0,
//                                                                            TlsDataSize);
//         if (!TlsVector[TlsData->TlsDirectory.Characteristics])
//         {
//             /* Out of memory */
//             return STATUS_NO_MEMORY;
//         }
//
//         /* Show debug message */
//         if (ShowSnaps)
//         {
//             DPRINT1("LDR: TlsVector %p Index %lu = %p copied from %x to %p\n",
//                     TlsVector,
//                     TlsData->TlsDirectory.Characteristics,
//                     &TlsVector[TlsData->TlsDirectory.Characteristics],
//                     TlsData->TlsDirectory.StartAddressOfRawData,
//                     TlsVector[TlsData->TlsDirectory.Characteristics]);
//         }
//
//         /* Copy the data */
//         RtlCopyMemory(TlsVector[TlsData->TlsDirectory.Characteristics],
//                       (PVOID)TlsData->TlsDirectory.StartAddressOfRawData,
//                       TlsDataSize);
//     }
//
//     /* Done */
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// LdrpFreeTls(VOID)
// {
//     PLIST_ENTRY ListHead, NextEntry;
//     PLDRP_TLS_DATA TlsData;
//     PVOID *TlsVector;
//     PTEB Teb = NtCurrentTeb();
//
//     /* Get a pointer to the vector array */
//     TlsVector = Teb->ThreadLocalStoragePointer;
//     if (!TlsVector) return;
//
//     /* Loop through it */
//     ListHead = &LdrpTlsList;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         TlsData = CONTAINING_RECORD(NextEntry, LDRP_TLS_DATA, TlsLinks);
//         NextEntry = NextEntry->Flink;
//
//         /* Free each entry */
//         if (TlsVector[TlsData->TlsDirectory.Characteristics])
//         {
//             RtlFreeHeap(RtlGetProcessHeap(),
//                         0,
//                         TlsVector[TlsData->TlsDirectory.Characteristics]);
//         }
//     }
//
//     /* Free the array itself */
//     RtlFreeHeap(RtlGetProcessHeap(),
//                 0,
//                 TlsVector);
// }
//
// NTSTATUS
// NTAPI
// LdrpInitializeExecutionOptions(PUNICODE_STRING ImagePathName, PPEB Peb, PHANDLE OptionsKey)
// {
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//     ULONG ExecuteOptions, MinimumStackCommit = 0, GlobalFlag;
//
//     /* Return error if we were not provided a pointer where to save the options key handle */
//     if (!OptionsKey) return STATUS_INVALID_HANDLE;
//
//     /* Zero initialize the options key pointer */
//     *OptionsKey = NULL;
//
//     /* Open the options key */
//     Status = LdrOpenImageFileOptionsKey(ImagePathName, 0, &KeyHandle);
//
//     /* Save it if it was opened successfully */
//     if (NT_SUCCESS(Status))
//         *OptionsKey = KeyHandle;
//
//     if (KeyHandle)
//     {
//         /* There are image specific options, read them starting with NXCOMPAT */
//         Status = LdrQueryImageFileKeyOption(KeyHandle,
//                                             L"ExecuteOptions",
//                                             4,
//                                             &ExecuteOptions,
//                                             sizeof(ExecuteOptions),
//                                             0);
//
//         if (NT_SUCCESS(Status))
//         {
//             /* TODO: Set execution options for the process */
//             /*
//             if (ExecuteOptions == 0)
//                 ExecuteOptions = 1;
//             else
//                 ExecuteOptions = 2;
//             ZwSetInformationProcess(NtCurrentProcess(),
//                                     ProcessExecuteFlags,
//                                     &ExecuteOptions,
//                                     sizeof(ULONG));*/
//
//         }
//
//         /* Check if this image uses large pages */
//         if (Peb->ImageUsesLargePages)
//         {
//             /* TODO: If it does, open large page key */
//             UNIMPLEMENTED;
//         }
//
//         /* Get various option values */
//         LdrQueryImageFileKeyOption(KeyHandle,
//                                    L"DisableHeapLookaside",
//                                    REG_DWORD,
//                                    &RtlpDisableHeapLookaside,
//                                    sizeof(RtlpDisableHeapLookaside),
//                                    NULL);
//
//         LdrQueryImageFileKeyOption(KeyHandle,
//                                    L"ShutdownFlags",
//                                    REG_DWORD,
//                                    &RtlpShutdownProcessFlags,
//                                    sizeof(RtlpShutdownProcessFlags),
//                                    NULL);
//
//         LdrQueryImageFileKeyOption(KeyHandle,
//                                    L"MinimumStackCommitInBytes",
//                                    REG_DWORD,
//                                    &MinimumStackCommit,
//                                    sizeof(MinimumStackCommit),
//                                    NULL);
//
//         /* Update PEB's minimum stack commit if it's lower */
//         if (Peb->MinimumStackCommit < MinimumStackCommit)
//             Peb->MinimumStackCommit = MinimumStackCommit;
//
//         /* Set the global flag */
//         Status = LdrQueryImageFileKeyOption(KeyHandle,
//                                             L"GlobalFlag",
//                                             REG_DWORD,
//                                             &GlobalFlag,
//                                             sizeof(GlobalFlag),
//                                             NULL);
//
//         if (NT_SUCCESS(Status))
//             Peb->NtGlobalFlag = GlobalFlag;
//         else
//             GlobalFlag = 0;
//
//         /* Call AVRF if necessary */
//         if (Peb->NtGlobalFlag & (FLG_APPLICATION_VERIFIER | FLG_HEAP_PAGE_ALLOCS))
//         {
//             Status = LdrpInitializeApplicationVerifierPackage(KeyHandle, Peb, FALSE, FALSE);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("AVRF: LdrpInitializeApplicationVerifierPackage failed with %08X\n", Status);
//             }
//         }
//     }
//     else
//     {
//         /* There are no image-specific options, so perform global initialization */
//         if (Peb->NtGlobalFlag & (FLG_APPLICATION_VERIFIER | FLG_HEAP_PAGE_ALLOCS))
//         {
//             /* Initialize app verifier package */
//             Status = LdrpInitializeApplicationVerifierPackage(KeyHandle, Peb, TRUE, FALSE);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("AVRF: LdrpInitializeApplicationVerifierPackage failed with %08X\n", Status);
//             }
//         }
//     }
//
//     return STATUS_SUCCESS;
// }
//
// VOID
// NTAPI
// LdrpValidateImageForMp(IN PLDR_DATA_TABLE_ENTRY LdrDataTableEntry)
// {
//     DPRINT("LdrpValidateImageForMp is unimplemented\n");
//     // TODO:
//     // Scan the LockPrefixTable in the load config directory
// }
//
// BOOLEAN
// NTAPI
// LdrpDisableProcessCompatGuidDetection(VOID)
// {
//     UNICODE_STRING PolicyKey = RTL_CONSTANT_STRING(L"\\Registry\\MACHINE\\Software\\Policies\\Microsoft\\Windows\\AppCompat");
//     UNICODE_STRING DisableDetection = RTL_CONSTANT_STRING(L"DisableCompatGuidDetection");
//     OBJECT_ATTRIBUTES PolicyKeyAttributes = RTL_CONSTANT_OBJECT_ATTRIBUTES(&PolicyKey, OBJ_CASE_INSENSITIVE);
//     KEY_VALUE_PARTIAL_INFORMATION KeyInfo;
//     ULONG ResultLength;
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//
//     Status = NtOpenKey(&KeyHandle, KEY_QUERY_VALUE, &PolicyKeyAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         Status = NtQueryValueKey(KeyHandle,
//                                  &DisableDetection,
//                                  KeyValuePartialInformation,
//                                  &KeyInfo,
//                                  sizeof(KeyInfo),
//                                  &ResultLength);
//         NtClose(KeyHandle);
//         if ((NT_SUCCESS(Status)) &&
//             (KeyInfo.Type == REG_DWORD) &&
//             (KeyInfo.DataLength == sizeof(ULONG)) &&
//             (KeyInfo.Data[0] == TRUE))
//         {
//             return TRUE;
//         }
//     }
//     return FALSE;
// }
//
//
// VOID
// NTAPI
// LdrpInitializeProcessCompat(PVOID pProcessActctx, PVOID* pOldShimData)
// {
//     static const struct
//     {
//         const GUID* Guid;
//         const DWORD Version;
//     } KnownCompatGuids[] = {
//         { &COMPAT_GUID_WIN10, _WIN32_WINNT_WIN10 },
//         { &COMPAT_GUID_WIN81, _WIN32_WINNT_WINBLUE },
//         { &COMPAT_GUID_WIN8, _WIN32_WINNT_WIN8 },
//         { &COMPAT_GUID_WIN7, _WIN32_WINNT_WIN7 },
//         { &COMPAT_GUID_VISTA, _WIN32_WINNT_VISTA },
//     };
//
//     ULONG Buffer[(sizeof(COMPATIBILITY_CONTEXT_ELEMENT) * 10 + sizeof(ACTIVATION_CONTEXT_COMPATIBILITY_INFORMATION)) / sizeof(ULONG)];
//     ACTIVATION_CONTEXT_COMPATIBILITY_INFORMATION* ContextCompatInfo;
//     SIZE_T SizeRequired;
//     NTSTATUS Status;
//     DWORD n, cur;
//     ReactOS_ShimData* pShimData = *pOldShimData;
//
//     if (pShimData)
//     {
//         if (pShimData->dwMagic != REACTOS_SHIMDATA_MAGIC ||
//             pShimData->dwSize != sizeof(ReactOS_ShimData))
//         {
//             DPRINT1("LdrpInitializeProcessCompat: Corrupt pShimData (0x%x, %u)\n", pShimData->dwMagic, pShimData->dwSize);
//             return;
//         }
//         if (pShimData->dwRosProcessCompatVersion)
//         {
//             if (pShimData->dwRosProcessCompatVersion == REACTOS_COMPATVERSION_IGNOREMANIFEST)
//             {
//                 DPRINT1("LdrpInitializeProcessCompat: ProcessCompatVersion set to ignore manifest\n");
//             }
//             else
//             {
//                 DPRINT1("LdrpInitializeProcessCompat: ProcessCompatVersion already set to 0x%x\n", pShimData->dwRosProcessCompatVersion);
//             }
//             return;
//         }
//     }
//
//     SizeRequired = sizeof(Buffer);
//     Status = RtlQueryInformationActivationContext(RTL_QUERY_ACTIVATION_CONTEXT_FLAG_NO_ADDREF,
//                                                   pProcessActctx,
//                                                   NULL,
//                                                   CompatibilityInformationInActivationContext,
//                                                   Buffer,
//                                                   sizeof(Buffer),
//                                                   &SizeRequired);
//
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LdrpInitializeProcessCompat: Unable to query process actctx with status %x\n", Status);
//         return;
//     }
//
//     ContextCompatInfo = (ACTIVATION_CONTEXT_COMPATIBILITY_INFORMATION*)Buffer;
//     /* No Compatibility elements present, bail out */
//     if (ContextCompatInfo->ElementCount == 0)
//         return;
//
//     /* Search for known GUIDs, starting from oldest to newest.
//        Note that on Windows it is somewhat reversed, starting from the latest known
//        version, going down. But we are not Windows, trying to allow a lower version,
//        we are ReactOS trying to fake a higher version. So we interpret what Windows
//        does as "try the closest version to the actual version", so we start with the
//        lowest version, which is closest to Windows 2003, which we mostly are. */
//     for (cur = RTL_NUMBER_OF(KnownCompatGuids) - 1; cur != -1; --cur)
//     {
//         for (n = 0; n < ContextCompatInfo->ElementCount; ++n)
//         {
//             if (ContextCompatInfo->Elements[n].Type == ACTCTX_COMPATIBILITY_ELEMENT_TYPE_OS &&
//                 RtlCompareMemory(&ContextCompatInfo->Elements[n].Id, KnownCompatGuids[cur].Guid, sizeof(GUID)) == sizeof(GUID))
//             {
//                 if (LdrpDisableProcessCompatGuidDetection())
//                 {
//                     DPRINT1("LdrpInitializeProcessCompat: Not applying automatic fix for winver 0x%x due to policy\n", KnownCompatGuids[cur].Version);
//                     return;
//                 }
//
//                 /* If this process did not need shim data before, allocate and store it */
//                 if (pShimData == NULL)
//                 {
//                     PPEB Peb = NtCurrentPeb();
//
//                     ASSERT(Peb->pShimData == NULL);
//                     pShimData = RtlAllocateHeap(Peb->ProcessHeap, HEAP_ZERO_MEMORY, sizeof(*pShimData));
//
//                     if (!pShimData)
//                     {
//                         DPRINT1("LdrpInitializeProcessCompat: Unable to allocated %u bytes\n", sizeof(*pShimData));
//                         return;
//                     }
//
//                     pShimData->dwSize = sizeof(*pShimData);
//                     pShimData->dwMagic = REACTOS_SHIMDATA_MAGIC;
//
//                     Peb->pShimData = pShimData;
//                     *pOldShimData = pShimData;
//                 }
//
//                 /* Store the lowest found version, and bail out. */
//                 pShimData->dwRosProcessCompatVersion = KnownCompatGuids[cur].Version;
//                 DPRINT1("LdrpInitializeProcessCompat: Found guid for winver 0x%x in manifest from %wZ\n",
//                         KnownCompatGuids[cur].Version,
//                         &(NtCurrentPeb()->ProcessParameters->ImagePathName));
//                 return;
//             }
//         }
//     }
// }
//
// VOID
// NTAPI
// LdrpInitializeDotLocalSupport(PRTL_USER_PROCESS_PARAMETERS ProcessParameters)
// {
//     UNICODE_STRING ImagePathName = ProcessParameters->ImagePathName;
//     WCHAR LocalBuffer[MAX_PATH];
//     UNICODE_STRING DotLocal;
//     NTSTATUS Status;
//     ULONG RequiredSize;
//
//     RequiredSize = ImagePathName.Length + LdrpDotLocal.Length + sizeof(UNICODE_NULL);
//     if (RequiredSize <= sizeof(LocalBuffer))
//     {
//         RtlInitEmptyUnicodeString(&DotLocal, LocalBuffer, sizeof(LocalBuffer));
//     }
//     else if (RequiredSize <= UNICODE_STRING_MAX_BYTES)
//     {
//         DotLocal.Buffer = RtlAllocateHeap(RtlGetProcessHeap(), 0, RequiredSize);
//         DotLocal.Length = 0;
//         DotLocal.MaximumLength = RequiredSize;
//         if (!DotLocal.Buffer)
//             DPRINT1("LDR: Failed to allocate memory for .local check\n");
//     }
//     else
//     {
//         DotLocal.Buffer = NULL;
//         DotLocal.Length = 0;
//         DotLocal.MaximumLength = 0;
//         DPRINT1("LDR: String too big for .local check\n");
//     }
//
//     if (DotLocal.Buffer)
//     {
//         Status = RtlAppendUnicodeStringToString(&DotLocal, &ImagePathName);
//         ASSERT(NT_SUCCESS(Status));
//         if (NT_SUCCESS(Status))
//         {
//             Status = RtlAppendUnicodeStringToString(&DotLocal, &LdrpDotLocal);
//             ASSERT(NT_SUCCESS(Status));
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             if (RtlDoesFileExists_UStr(&DotLocal))
//             {
//                 ProcessParameters->Flags |= RTL_USER_PROCESS_PARAMETERS_PRIVATE_DLL_PATH;
//             }
//         }
//         else
//         {
//             DPRINT1("LDR: Failed to append: 0x%lx\n", Status);
//         }
//
//         if (DotLocal.Buffer != LocalBuffer)
//         {
//             RtlFreeHeap(RtlGetProcessHeap(), 0, DotLocal.Buffer);
//         }
//     }
// }
//
//
// NTSTATUS
// NTAPI
// LdrpInitializeProcess(IN PCONTEXT Context,
//                       IN PVOID SystemArgument1)
// {
//     RTL_HEAP_PARAMETERS HeapParameters;
//     ULONG ComSectionSize;
//     ANSI_STRING BaseProcessInitPostImportName = RTL_CONSTANT_STRING("BaseProcessInitPostImport");
//     ANSI_STRING BaseQueryModuleDataName = RTL_CONSTANT_STRING("BaseQueryModuleData");
//     PVOID OldShimData;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     //UNICODE_STRING LocalFileName, FullImageName;
//     HANDLE SymLinkHandle;
//     //ULONG DebugHeapOnly;
//     UNICODE_STRING CommandLine, NtSystemRoot, ImagePathName, FullPath, ImageFileName, KnownDllString;
//     PPEB Peb = NtCurrentPeb();
//     BOOLEAN IsDotNetImage = FALSE;
//     BOOLEAN FreeCurDir = FALSE;
//     //HANDLE CompatKey;
//     PRTL_USER_PROCESS_PARAMETERS ProcessParameters;
//     //LPWSTR ImagePathBuffer;
//     ULONG ConfigSize;
//     UNICODE_STRING CurrentDirectory;
//     HANDLE OptionsKey;
//     ULONG HeapFlags;
//     PIMAGE_NT_HEADERS NtHeader;
//     LPWSTR NtDllName = NULL;
//     NTSTATUS Status, ImportStatus;
//     NLSTABLEINFO NlsTable;
//     PIMAGE_LOAD_CONFIG_DIRECTORY LoadConfig;
//     PTEB Teb = NtCurrentTeb();
//     PLIST_ENTRY ListHead;
//     PLIST_ENTRY NextEntry;
//     ULONG i;
//     PWSTR ImagePath;
//     ULONG DebugProcessHeapOnly = 0;
//     PLDR_DATA_TABLE_ENTRY NtLdrEntry;
//     PWCHAR Current;
//     ULONG ExecuteOptions = 0;
//     PVOID ViewBase;
//
//     /* Set a NULL SEH Filter */
//     RtlSetUnhandledExceptionFilter(NULL);
//
//     /* Debug: trace process initialization entry */
//     DPRINT1("LdrpInitializeProcess: ENTER (ImageBase=%p)\n", Peb->ImageBaseAddress);
//
//     /* Get the image path */
//     ImagePath = Peb->ProcessParameters->ImagePathName.Buffer;
//
//     /* Check if it's not normalized */
//     if (!(Peb->ProcessParameters->Flags & RTL_USER_PROCESS_PARAMETERS_NORMALIZED))
//     {
//         /* Normalize it*/
//         ImagePath = (PWSTR)((ULONG_PTR)ImagePath + (ULONG_PTR)Peb->ProcessParameters);
//     }
//
//     /* Create a unicode string for the Image Path */
//     ImagePathName.Length = Peb->ProcessParameters->ImagePathName.Length;
//     ImagePathName.MaximumLength = ImagePathName.Length + sizeof(WCHAR);
//     ImagePathName.Buffer = ImagePath;
//
//     /* Get the NT Headers */
//     NtHeader = RtlImageNtHeader(Peb->ImageBaseAddress);
//
//     /* Get the execution options */
//     Status = LdrpInitializeExecutionOptions(&ImagePathName, Peb, &OptionsKey);
//
//     /* Check if this is a .NET executable */
//     if (RtlImageDirectoryEntryToData(Peb->ImageBaseAddress,
//                                      TRUE,
//                                      IMAGE_DIRECTORY_ENTRY_COM_DESCRIPTOR,
//                                      &ComSectionSize))
//     {
//         /* Remember this for later */
//         IsDotNetImage = TRUE;
//     }
//
//     /* Save the NTDLL Base address */
//     NtDllBase = SystemArgument1;
//
//     /* If this is a Native Image */
//     if (NtHeader->OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_NATIVE)
//     {
//         /* Then do DLL Validation */
//         LdrpDllValidation = TRUE;
//     }
//
//     /* Save the old Shim Data */
//     OldShimData = Peb->pShimData;
//
//     /* ReactOS specific: do not clear it. (Windows starts doing the same in later versions) */
//     //Peb->pShimData = NULL;
//
//     /* Save the number of processors and CS timeout */
//     LdrpNumberOfProcessors = Peb->NumberOfProcessors;
//     RtlpTimeout = Peb->CriticalSectionTimeout;
//
//     /* Normalize the parameters */
//     ProcessParameters = RtlNormalizeProcessParams(Peb->ProcessParameters);
//     if (ProcessParameters)
//     {
//         /* Save the Image and Command Line Names */
//         ImageFileName = ProcessParameters->ImagePathName;
//         CommandLine = ProcessParameters->CommandLine;
//     }
//     else
//     {
//         /* It failed, initialize empty strings */
//         RtlInitUnicodeString(&ImageFileName, NULL);
//         RtlInitUnicodeString(&CommandLine, NULL);
//     }
//
//     /* Initialize NLS data */
//     RtlInitNlsTables(Peb->AnsiCodePageData,
//                      Peb->OemCodePageData,
//                      Peb->UnicodeCaseTableData,
//                      &NlsTable);
//
//     /* Reset NLS Translations */
//     RtlResetRtlTranslations(&NlsTable);
//
//     /* Get the Image Config Directory */
//     LoadConfig = RtlImageDirectoryEntryToData(Peb->ImageBaseAddress,
//                                               TRUE,
//                                               IMAGE_DIRECTORY_ENTRY_LOAD_CONFIG,
//                                               &ConfigSize);
//
//     /* Setup the Heap Parameters */
//     RtlZeroMemory(&HeapParameters, sizeof(HeapParameters));
//     HeapFlags = HEAP_GROWABLE;
//     HeapParameters.Length = sizeof(HeapParameters);
//
//     /* Check if we have Configuration Data */
// #define VALID_CONFIG_FIELD(Name) (ConfigSize >= RTL_SIZEOF_THROUGH_FIELD(IMAGE_LOAD_CONFIG_DIRECTORY, Name))
//     /* The 'original' load config ends after SecurityCookie */
//     if ((LoadConfig) && ConfigSize && (VALID_CONFIG_FIELD(SecurityCookie) || ConfigSize == LoadConfig->Size))
//     {
//         if (ConfigSize != sizeof(IMAGE_LOAD_CONFIG_DIRECTORY))
//             DPRINT1("WARN: Accepting different LOAD_CONFIG size!\n");
//         else
//             DPRINT1("Applying LOAD_CONFIG\n");
//
//         if (VALID_CONFIG_FIELD(GlobalFlagsSet) && LoadConfig->GlobalFlagsSet)
//             Peb->NtGlobalFlag |= LoadConfig->GlobalFlagsSet;
//
//         if (VALID_CONFIG_FIELD(GlobalFlagsClear) && LoadConfig->GlobalFlagsClear)
//             Peb->NtGlobalFlag &= ~LoadConfig->GlobalFlagsClear;
//
//         /* Convert the default CS timeout from milliseconds to 100ns units */
//         if (VALID_CONFIG_FIELD(CriticalSectionDefaultTimeout) && LoadConfig->CriticalSectionDefaultTimeout)
//             RtlpTimeout.QuadPart = Int32x32To64(LoadConfig->CriticalSectionDefaultTimeout, -10000);
//
//         if (VALID_CONFIG_FIELD(DeCommitFreeBlockThreshold) && LoadConfig->DeCommitFreeBlockThreshold)
//             HeapParameters.DeCommitFreeBlockThreshold = LoadConfig->DeCommitFreeBlockThreshold;
//
//         if (VALID_CONFIG_FIELD(DeCommitTotalFreeThreshold) && LoadConfig->DeCommitTotalFreeThreshold)
//             HeapParameters.DeCommitTotalFreeThreshold = LoadConfig->DeCommitTotalFreeThreshold;
//
//         if (VALID_CONFIG_FIELD(MaximumAllocationSize) && LoadConfig->MaximumAllocationSize)
//             HeapParameters.MaximumAllocationSize = LoadConfig->MaximumAllocationSize;
//
//         if (VALID_CONFIG_FIELD(VirtualMemoryThreshold) && LoadConfig->VirtualMemoryThreshold)
//             HeapParameters.VirtualMemoryThreshold = LoadConfig->VirtualMemoryThreshold;
//
//         if (VALID_CONFIG_FIELD(ProcessHeapFlags) && LoadConfig->ProcessHeapFlags)
//             HeapFlags = LoadConfig->ProcessHeapFlags;
//     }
// #undef VALID_CONFIG_FIELD
//
//     /* Check for custom affinity mask */
//     if (Peb->ImageProcessAffinityMask)
//     {
//         /* Set it */
//         Status = NtSetInformationProcess(NtCurrentProcess(),
//                                          ProcessAffinityMask,
//                                          &Peb->ImageProcessAffinityMask,
//                                          sizeof(Peb->ImageProcessAffinityMask));
//     }
//
//     /* Check if verbose debugging (ShowSnaps) was requested */
//     ShowSnaps = Peb->NtGlobalFlag & FLG_SHOW_LDR_SNAPS;
//
//     /* Start verbose debugging messages right now if they were requested */
//     if (ShowSnaps)
//     {
//         DPRINT1("LDR: PID: 0x%p started - '%wZ'\n",
//                 Teb->ClientId.UniqueProcess,
//                 &CommandLine);
//     }
//
//     /* If the CS timeout is longer than 1 hour, disable it */
//     if (RtlpTimeout.QuadPart < Int32x32To64(3600, -10000000))
//         RtlpTimeoutDisable = TRUE;
//
//     /* Initialize Critical Section Data */
//     RtlpInitDeferredCriticalSection();
//
//     /* Initialize VEH Call lists */
//     RtlpInitializeVectoredExceptionHandling();
//
//     /* Set TLS/FLS Bitmap data */
//     Peb->FlsBitmap = &FlsBitMap;
//     Peb->TlsBitmap = &TlsBitMap;
//     Peb->TlsExpansionBitmap = &TlsExpansionBitMap;
//
//     /* Initialize FLS Bitmap */
//     RtlInitializeBitMap(&FlsBitMap,
//                         Peb->FlsBitmapBits,
//                         FLS_MAXIMUM_AVAILABLE);
//     RtlSetBit(&FlsBitMap, 0);
//     InitializeListHead(&Peb->FlsListHead);
//
//     /* Initialize TLS Bitmap */
//     RtlInitializeBitMap(&TlsBitMap,
//                         Peb->TlsBitmapBits,
//                         TLS_MINIMUM_AVAILABLE);
//     RtlSetBit(&TlsBitMap, 0);
//     RtlInitializeBitMap(&TlsExpansionBitMap,
//                         Peb->TlsExpansionBitmapBits,
//                         TLS_EXPANSION_SLOTS);
//     RtlSetBit(&TlsExpansionBitMap, 0);
//
//     /* Initialize the Hash Table */
//     for (i = 0; i < LDR_HASH_TABLE_ENTRIES; i++)
//     {
//         InitializeListHead(&LdrpHashTable[i]);
//     }
//
//     /* Initialize the Loader Lock */
//     // FIXME: What's the point of initing it manually, if two lines lower
//     //        a call to RtlInitializeCriticalSection() is being made anyway?
//     //InsertTailList(&RtlCriticalSectionList, &LdrpLoaderLock.DebugInfo->ProcessLocksList);
//     //LdrpLoaderLock.DebugInfo->CriticalSection = &LdrpLoaderLock;
//     RtlInitializeCriticalSection(&LdrpLoaderLock);
//     LdrpLoaderLockInit = TRUE;
//
//     /* Check if User Stack Trace Database support was requested */
//     if (Peb->NtGlobalFlag & FLG_USER_STACK_TRACE_DB)
//     {
//         DPRINT1("We don't support user stack trace databases yet\n");
//     }
//
//     /* Setup Fast PEB Lock */
//     RtlInitializeCriticalSection(&FastPebLock);
//     Peb->FastPebLock = &FastPebLock;
//     //Peb->FastPebLockRoutine = (PPEBLOCKROUTINE)RtlEnterCriticalSection;
//     //Peb->FastPebUnlockRoutine = (PPEBLOCKROUTINE)RtlLeaveCriticalSection;
//
//     /* Setup Callout Lock */
//     //RtlInitializeCriticalSection(&RtlpCalloutEntryLock);
//
//     /* For old executables, use 16-byte aligned heap */
//     if ((NtHeader->OptionalHeader.MajorSubsystemVersion <= 3) &&
//         (NtHeader->OptionalHeader.MinorSubsystemVersion < 51))
//     {
//         HeapFlags |= HEAP_CREATE_ALIGN_16;
//     }
//
//     /* Setup the Heap */
//     RtlInitializeHeapManager();
//     Peb->ProcessHeap = RtlCreateHeap(HeapFlags,
//                                      NULL,
//                                      NtHeader->OptionalHeader.SizeOfHeapReserve,
//                                      NtHeader->OptionalHeader.SizeOfHeapCommit,
//                                      NULL,
//                                      &HeapParameters);
//
//     if (!Peb->ProcessHeap)
//     {
//         DPRINT1("Failed to create process heap\n");
//         return STATUS_NO_MEMORY;
//     }
//     DPRINT1("LdrpInitializeProcess: heap created OK\n");
//
//     Status = RtlpInitializeLocaleTable();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to initialize locale table\n");
//         return Status;
//     }
//
//     /* Allocate an Activation Context Stack */
//     Status = RtlAllocateActivationContextStack(&Teb->ActivationContextStackPointer);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     RtlZeroMemory(&HeapParameters, sizeof(HeapParameters));
//     HeapFlags = HEAP_GROWABLE | HEAP_CLASS_1;
//     HeapParameters.Length = sizeof(HeapParameters);
//     LdrpHeap = RtlCreateHeap(HeapFlags, 0, 0x10000, 0x6000, 0, &HeapParameters);
//     if (!LdrpHeap)
//     {
//         DPRINT1("Failed to create loader private heap\n");
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Check for Debug Heap */
//     if (OptionsKey)
//     {
//         /* Query the setting */
//         Status = LdrQueryImageFileKeyOption(OptionsKey,
//                                             L"DebugProcessHeapOnly",
//                                             REG_DWORD,
//                                             &DebugProcessHeapOnly,
//                                             sizeof(ULONG),
//                                             NULL);
//
//         if (NT_SUCCESS(Status))
//         {
//             /* Reset DPH if requested */
//             if (RtlpPageHeapEnabled && DebugProcessHeapOnly)
//             {
//                 RtlpDphGlobalFlags &= ~DPH_FLAG_DLL_NOTIFY;
//                 RtlpPageHeapEnabled = FALSE;
//             }
//         }
//     }
//
//     /* Build the NTDLL Path */
//     FullPath.Buffer = StringBuffer;
//     FullPath.Length = 0;
//     FullPath.MaximumLength = sizeof(StringBuffer);
//     RtlInitUnicodeString(&NtSystemRoot, SharedUserData->NtSystemRoot);
//     RtlAppendUnicodeStringToString(&FullPath, &NtSystemRoot);
//     RtlAppendUnicodeToString(&FullPath, L"\\System32\\");
//
//     DPRINT1("LdrpInitializeProcess: opening KnownDlls\n");
//     /* Open the Known DLLs directory */
//     RtlInitUnicodeString(&KnownDllString, L"\\KnownDlls");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KnownDllString,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                NULL);
//     Status = ZwOpenDirectoryObject(&LdrpKnownDllObjectDirectory,
//                                    DIRECTORY_QUERY | DIRECTORY_TRAVERSE,
//                                    &ObjectAttributes);
//
//     /* Check if it exists */
//     if (NT_SUCCESS(Status))
//     {
//         /* Open the Known DLLs Path */
//         RtlInitUnicodeString(&KnownDllString, L"KnownDllPath");
//         InitializeObjectAttributes(&ObjectAttributes,
//                                    &KnownDllString,
//                                    OBJ_CASE_INSENSITIVE,
//                                    LdrpKnownDllObjectDirectory,
//                                    NULL);
//         Status = NtOpenSymbolicLinkObject(&SymLinkHandle,
//                                           SYMBOLIC_LINK_QUERY,
//                                           &ObjectAttributes);
//         if (NT_SUCCESS(Status))
//         {
//             /* Query the path */
//             LdrpKnownDllPath.Length = 0;
//             LdrpKnownDllPath.MaximumLength = sizeof(LdrpKnownDllPathBuffer);
//             LdrpKnownDllPath.Buffer = LdrpKnownDllPathBuffer;
//             Status = ZwQuerySymbolicLinkObject(SymLinkHandle, &LdrpKnownDllPath, NULL);
//             NtClose(SymLinkHandle);
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("LDR: %s - failed call to ZwQuerySymbolicLinkObject with status %x\n", "", Status);
//                 return Status;
//             }
//         }
//     }
//
//     /* Check if we failed */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Assume System32 */
//         LdrpKnownDllObjectDirectory = NULL;
//         RtlInitUnicodeString(&LdrpKnownDllPath, StringBuffer);
//         LdrpKnownDllPath.Length -= sizeof(WCHAR);
//     }
//
//     /* If we have process parameters, get the default path and current path */
//     if (ProcessParameters)
//     {
//         /* Check if we have a Dll Path */
//         if (ProcessParameters->DllPath.Length)
//         {
//             /* Get the path */
//             LdrpDefaultPath = *(PUNICODE_STRING)&ProcessParameters->DllPath;
//         }
//         else
//         {
//             /* We need a valid path */
//             DPRINT1("No valid DllPath was given!\n");
//             LdrpInitFailure(STATUS_INVALID_PARAMETER);
//         }
//
//         /* Set the current directory */
//         CurrentDirectory = ProcessParameters->CurrentDirectory.DosPath;
//
//         /* Check if it's empty or invalid */
//         if ((!CurrentDirectory.Buffer) ||
//             (CurrentDirectory.Buffer[0] == UNICODE_NULL) ||
//             (!CurrentDirectory.Length))
//         {
//             /* Allocate space for the buffer */
//             CurrentDirectory.Buffer = RtlAllocateHeap(Peb->ProcessHeap,
//                                                       0,
//                                                       3 * sizeof(WCHAR) +
//                                                       sizeof(UNICODE_NULL));
//             if (!CurrentDirectory.Buffer)
//             {
//                 DPRINT1("LDR: LdrpInitializeProcess - unable to allocate current working directory buffer\n");
//                 // FIXME: And what?
//             }
//
//             /* Copy the drive of the system root */
//             RtlMoveMemory(CurrentDirectory.Buffer,
//                           SharedUserData->NtSystemRoot,
//                           3 * sizeof(WCHAR));
//             CurrentDirectory.Buffer[3] = UNICODE_NULL;
//             CurrentDirectory.Length = 3 * sizeof(WCHAR);
//             CurrentDirectory.MaximumLength = CurrentDirectory.Length + sizeof(WCHAR);
//
//             FreeCurDir = TRUE;
//             DPRINT("Using dynamically allocd curdir\n");
//         }
//         else
//         {
//             /* Use the local buffer */
//             DPRINT("Using local system root\n");
//         }
//     }
//
//     DPRINT1("LdrpInitializeProcess: setting up loader data\n");
//     /* Setup Loader Data */
//     Peb->Ldr = &PebLdr;
//     InitializeListHead(&PebLdr.InLoadOrderModuleList);
//     InitializeListHead(&PebLdr.InMemoryOrderModuleList);
//     InitializeListHead(&PebLdr.InInitializationOrderModuleList);
//     PebLdr.Length = sizeof(PEB_LDR_DATA);
//     PebLdr.Initialized = TRUE;
//
//     /* Allocate a data entry for the Image */
//     LdrpImageEntry = LdrpAllocateDataTableEntry(Peb->ImageBaseAddress);
//
//     /* Set it up */
//     LdrpImageEntry->EntryPoint = LdrpFetchAddressOfEntryPoint(LdrpImageEntry->DllBase);
//     LdrpImageEntry->LoadCount = -1;
//     LdrpImageEntry->EntryPointActivationContext = 0;
//     LdrpImageEntry->FullDllName = ImageFileName;
//
//     if (IsDotNetImage)
//         LdrpImageEntry->Flags = LDRP_COR_IMAGE;
//     else
//         LdrpImageEntry->Flags = 0;
//
//     /* Check if the name is empty */
//     if (!ImageFileName.Buffer[0])
//     {
//         /* Use the same Base name */
//         LdrpImageEntry->BaseDllName = LdrpImageEntry->FullDllName;
//     }
//     else
//     {
//         /* Find the last slash */
//         Current = ImageFileName.Buffer;
//         while (*Current)
//         {
//             if (*Current++ == '\\')
//             {
//                 /* Set this path */
//                 NtDllName = Current;
//             }
//         }
//
//         /* Did we find anything? */
//         if (!NtDllName)
//         {
//             /* Use the same Base name */
//             LdrpImageEntry->BaseDllName = LdrpImageEntry->FullDllName;
//         }
//         else
//         {
//             /* Setup the name */
//             LdrpImageEntry->BaseDllName.Length = (USHORT)((ULONG_PTR)ImageFileName.Buffer + ImageFileName.Length - (ULONG_PTR)NtDllName);
//             LdrpImageEntry->BaseDllName.MaximumLength = LdrpImageEntry->BaseDllName.Length + sizeof(WCHAR);
//             LdrpImageEntry->BaseDllName.Buffer = (PWSTR)((ULONG_PTR)ImageFileName.Buffer +
//                                                      (ImageFileName.Length - LdrpImageEntry->BaseDllName.Length));
//         }
//     }
//
//     /* Processing done, insert it */
//     LdrpInsertMemoryTableEntry(LdrpImageEntry);
//     LdrpImageEntry->Flags |= LDRP_ENTRY_PROCESSED;
//
//     /* Now add an entry for NTDLL */
//     NtLdrEntry = LdrpAllocateDataTableEntry(SystemArgument1);
//     NtLdrEntry->Flags = LDRP_IMAGE_DLL;
//     NtLdrEntry->EntryPoint = LdrpFetchAddressOfEntryPoint(NtLdrEntry->DllBase);
//     NtLdrEntry->LoadCount = -1;
//     NtLdrEntry->EntryPointActivationContext = 0;
//
//     NtLdrEntry->FullDllName.Length = FullPath.Length;
//     NtLdrEntry->FullDllName.MaximumLength = FullPath.MaximumLength;
//     NtLdrEntry->FullDllName.Buffer = StringBuffer;
//     RtlAppendUnicodeStringToString(&NtLdrEntry->FullDllName, &NtDllString);
//
//     NtLdrEntry->BaseDllName.Length = NtDllString.Length;
//     NtLdrEntry->BaseDllName.MaximumLength = NtDllString.MaximumLength;
//     NtLdrEntry->BaseDllName.Buffer = NtDllString.Buffer;
//
//     /* Processing done, insert it */
//     LdrpNtDllDataTableEntry = NtLdrEntry;
//     LdrpInsertMemoryTableEntry(NtLdrEntry);
//
//     /* Let the world know */
//     if (ShowSnaps)
//     {
//         DPRINT1("LDR: NEW PROCESS\n");
//         DPRINT1("     Image Path: %wZ (%wZ)\n", &LdrpImageEntry->FullDllName, &LdrpImageEntry->BaseDllName);
//         DPRINT1("     Current Directory: %wZ\n", &CurrentDirectory);
//         DPRINT1("     Search Path: %wZ\n", &LdrpDefaultPath);
//     }
//
//     /* Link the Init Order List */
//     InsertHeadList(&Peb->Ldr->InInitializationOrderModuleList,
//                    &LdrpNtDllDataTableEntry->InInitializationOrderLinks);
//
//     DPRINT1("LdrpInitializeProcess: init actctx\n");
//     /* Initialize Wine's active context implementation for the current process */
//     RtlpInitializeActCtx(&OldShimData);
//
//     DPRINT1("LdrpInitializeProcess: setting curdir\n");
//     /* Set the current directory */
//     Status = RtlSetCurrentDirectory_U(&CurrentDirectory);
//     if (!NT_SUCCESS(Status))
//     {
//         /* We failed, check if we should free it */
//         if (FreeCurDir) RtlFreeUnicodeString(&CurrentDirectory);
//
//         /* Set it to the NT Root */
//         CurrentDirectory = NtSystemRoot;
//         RtlSetCurrentDirectory_U(&CurrentDirectory);
//     }
//     else
//     {
//         /* We're done with it, free it */
//         if (FreeCurDir) RtlFreeUnicodeString(&CurrentDirectory);
//     }
//
//     DPRINT1("LdrpInitializeProcess: curdir done, status=0x%lx\n", Status);
//     /* Check if we should look for a .local file */
//     if (ProcessParameters && !(ProcessParameters->Flags & RTL_USER_PROCESS_PARAMETERS_LOCAL_DLL_PATH))
//     {
//         LdrpInitializeDotLocalSupport(ProcessParameters);
//     }
//
//     DPRINT1("LdrpInitializeProcess: dotlocal done\n");
//     /* Check if the Application Verifier was enabled */
//     if (Peb->NtGlobalFlag & FLG_APPLICATION_VERIFIER)
//     {
//         Status = AVrfInitializeVerifier();
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("LDR: AVrfInitializeVerifier failed (ntstatus 0x%x)\n", Status);
//             return Status;
//         }
//
//     }
//
//     if (IsDotNetImage)
//     {
//         /* FIXME */
//         DPRINT1("We don't support .NET applications yet\n");
//     }
//
//     if (NtHeader->OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_WINDOWS_GUI ||
//         NtHeader->OptionalHeader.Subsystem == IMAGE_SUBSYSTEM_WINDOWS_CUI)
//     {
//         PVOID Kernel32BaseAddress;
//         PVOID FunctionAddress;
//
//         Status = LdrLoadDll(NULL, NULL, &Kernel32String, &Kernel32BaseAddress);
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (ShowSnaps)
//                 DPRINT1("LDR: Unable to load %wZ, Status=0x%08lx\n", &Kernel32String, Status);
//             return Status;
//         }
//
//         Status = LdrGetProcedureAddress(Kernel32BaseAddress,
//                                         &BaseProcessInitPostImportName,
//                                         0,
//                                         &FunctionAddress);
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (ShowSnaps)
//                 DPRINT1("LDR: Unable to find post-import process init function, Status=0x%08lx\n", Status);
//             return Status;
//         }
//         Kernel32ProcessInitPostImportFunction = FunctionAddress;
//
//         Status = LdrGetProcedureAddress(Kernel32BaseAddress,
//                                         &BaseQueryModuleDataName,
//                                         0,
//                                         &FunctionAddress);
//
//         if (!NT_SUCCESS(Status))
//         {
//             if (ShowSnaps)
//                 DPRINT1("LDR: Unable to find BaseQueryModuleData, Status=0x%08lx\n", Status);
//             return Status;
//         }
//         Kernel32BaseQueryModuleData = FunctionAddress;
//     }
//
//     /* Walk the IAT and load all the DLLs */
//     DPRINT1("LdrpInitializeProcess: Walking imports for '%wZ'\n", &ImagePathName);
//     ImportStatus = LdrpWalkImportDescriptor(LdrpDefaultPath.Buffer, LdrpImageEntry);
//     DPRINT1("LdrpInitializeProcess: Import walk done, status=0x%lx\n", ImportStatus);
//
//     /* Check if relocation is needed */
//     if (Peb->ImageBaseAddress != (PVOID)NtHeader->OptionalHeader.ImageBase)
//     {
//         DPRINT1("LDR: Performing EXE relocation\n");
//
//         /* Change the protection to prepare for relocation */
//         ViewBase = Peb->ImageBaseAddress;
//         Status = LdrpSetProtection(ViewBase, FALSE);
//         if (!NT_SUCCESS(Status)) return Status;
//
//         /* Do the relocation */
//         Status = LdrRelocateImageWithBias(ViewBase,
//                                           0LL,
//                                           NULL,
//                                           STATUS_SUCCESS,
//                                           STATUS_CONFLICTING_ADDRESSES,
//                                           STATUS_INVALID_IMAGE_FORMAT);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("LdrRelocateImageWithBias() failed\n");
//             return Status;
//         }
//
//         /* Check if a start context was provided */
//         if (Context)
//         {
//             DPRINT1("WARNING: Relocated EXE Context");
//             UNIMPLEMENTED; // We should support this
//             return STATUS_INVALID_IMAGE_FORMAT;
//         }
//
//         /* Restore the protection */
//         Status = LdrpSetProtection(ViewBase, TRUE);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Lock the DLLs */
//     ListHead = &Peb->Ldr->InLoadOrderModuleList;
//     NextEntry = ListHead->Flink;
//     while (ListHead != NextEntry)
//     {
//         NtLdrEntry = CONTAINING_RECORD(NextEntry, LDR_DATA_TABLE_ENTRY, InLoadOrderLinks);
//         NtLdrEntry->LoadCount = -1;
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Phase 0 is done */
//     LdrpLdrDatabaseIsSetup = TRUE;
//
//     /* Check whether all static imports were properly loaded and return here */
//     if (!NT_SUCCESS(ImportStatus)) return ImportStatus;
//
//     /* Following two calls are for Vista+ support, required for winesync */
//     /* Initialize the keyed event for condition variables */
//     RtlpInitializeKeyedEvent();
//     RtlpInitializeThreadPooling();
//
//     /* Initialize TLS */
//     Status = LdrpInitializeTls();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LDR: LdrpProcessInitialization failed to initialize TLS slots; status %x\n",
//                 Status);
//         return Status;
//     }
//
//     /* FIXME Mark the DLL Ranges for Stack Traces later */
//
//     /* Notify the debugger now */
//     if (Peb->BeingDebugged)
//     {
//         /* Break */
//         DbgBreakPoint();
//
//         /* Update show snaps again */
//         ShowSnaps = Peb->NtGlobalFlag & FLG_SHOW_LDR_SNAPS;
//     }
//
//     /* Validate the Image for MP Usage */
//     if (LdrpNumberOfProcessors > 1) LdrpValidateImageForMp(LdrpImageEntry);
//
//     /* Check NX options and set them */
//     if (SharedUserData->NXSupportPolicy == NX_SUPPORT_POLICY_ALWAYSON)
//     {
//         ExecuteOptions = MEM_EXECUTE_OPTION_DISABLE |
//                          MEM_EXECUTE_OPTION_DISABLE_THUNK_EMULATION |
//                          MEM_EXECUTE_OPTION_PERMANENT;
//     }
//     else if (SharedUserData->NXSupportPolicy == NX_SUPPORT_POLICY_ALWAYSOFF)
//     {
//         ExecuteOptions = MEM_EXECUTE_OPTION_ENABLE | MEM_EXECUTE_OPTION_PERMANENT;
//     }
//     Status = NtSetInformationProcess(NtCurrentProcess(),
//                                      ProcessExecuteFlags,
//                                      &ExecuteOptions,
//                                      sizeof(ExecuteOptions));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LDR: Could not set process execute flags 0x%x; status %x\n",
//                 ExecuteOptions, Status);
//     }
//
//     // FIXME: Should be done by Application Compatibility features,
//     // by reading the registry, etc...
//     // For now, this is the old code from ntdll!RtlGetVersion().
//     RtlInitEmptyUnicodeString(&Peb->CSDVersion, NULL, 0);
//     if (((Peb->OSCSDVersion >> 8) & 0xFF) != 0)
//     {
//         WCHAR szCSDVersion[128];
//         LONG i;
//         USHORT Length = (USHORT)ARRAYSIZE(szCSDVersion) - 1;
//         i = _snwprintf(szCSDVersion, Length,
//                        L"Service Pack %d",
//                        ((Peb->OSCSDVersion >> 8) & 0xFF));
//         if (i < 0)
//         {
//             /* Null-terminate if it was overflowed */
//             szCSDVersion[Length] = UNICODE_NULL;
//         }
//
//         Length *= sizeof(WCHAR);
//         Peb->CSDVersion.Buffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                                  0,
//                                                  Length + sizeof(UNICODE_NULL));
//         if (Peb->CSDVersion.Buffer)
//         {
//             Peb->CSDVersion.Length = Length;
//             Peb->CSDVersion.MaximumLength = Length + sizeof(UNICODE_NULL);
//
//             RtlCopyMemory(Peb->CSDVersion.Buffer,
//                           szCSDVersion,
//                           Peb->CSDVersion.MaximumLength);
//             Peb->CSDVersion.Buffer[Peb->CSDVersion.Length / sizeof(WCHAR)] = UNICODE_NULL;
//         }
//     }
//
//     /* Check if we had Shim Data */
//     if (OldShimData)
//     {
//         /* Load the Shim Engine */
//         Peb->AppCompatInfo = NULL;
//         LdrpLoadShimEngine(OldShimData, &ImagePathName, OldShimData);
//     }
//     else
//     {
//         /* Check for Application Compatibility Goo */
//         //LdrQueryApplicationCompatibilityGoo(hKey);
//         DPRINT("Querying app compat hacks is missing!\n");
//     }
//
//     /*
//      * FIXME: Check for special images, SecuROM, SafeDisc and other NX-
//      * incompatible images.
//      */
//
//     /* Now call the Init Routines */
//     Status = LdrpRunInitializeRoutines(Context);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("LDR: LdrpProcessInitialization failed running initialization routines; status %x\n",
//                 Status);
//         return Status;
//     }
//
//     /* Notify Shim Engine */
//     if (g_ShimsEnabled)
//     {
//         VOID(NTAPI *SE_InstallAfterInit)(PUNICODE_STRING, PVOID);
//         SE_InstallAfterInit = RtlDecodeSystemPointer(g_pfnSE_InstallAfterInit);
//         SE_InstallAfterInit(&ImagePathName, OldShimData);
//     }
//
//     /* Check if we have a user-defined Post Process Routine */
//     if (NT_SUCCESS(Status) && Peb->PostProcessInitRoutine)
//     {
//         /* Call it */
//         Peb->PostProcessInitRoutine();
//     }
//
//     /* Close the key if we have one opened */
//     if (OptionsKey) NtClose(OptionsKey);
//
//     /* Return status */
//     return Status;
// }
//
// VOID
// NTAPI
// LdrpInitFailure(NTSTATUS Status)
// {
//     ULONG Response;
//     PPEB Peb = NtCurrentPeb();
//
//     /* Print a debug message */
//     DPRINT1("LDR: Process initialization failure for %wZ; NTSTATUS = %08lx\n",
//             &Peb->ProcessParameters->ImagePathName, Status);
//
//     /* Raise a hard error */
//     if (!LdrpFatalHardErrorCount)
//     {
//         ZwRaiseHardError(STATUS_APP_INIT_FAILURE, 1, 0, (PULONG_PTR)&Status, OptionOk, &Response);
//     }
// }
//
// VOID
// NTAPI
// LdrpInit(PCONTEXT Context,
//          PVOID SystemArgument1,
//          PVOID SystemArgument2)
// {
//     LARGE_INTEGER Timeout;
//     PTEB Teb = NtCurrentTeb();
//     NTSTATUS Status, LoaderStatus = STATUS_SUCCESS;
//     MEMORY_BASIC_INFORMATION MemoryBasicInfo;
//     PPEB Peb = NtCurrentPeb();
//
//     DPRINT("LdrpInit() %p/%p\n",
//         NtCurrentTeb()->RealClientId.UniqueProcess,
//         NtCurrentTeb()->RealClientId.UniqueThread);
//
// #ifdef _WIN64
//     /* Set the SList header usage */
//     RtlpUse16ByteSLists = SharedUserData->ProcessorFeatures[PF_COMPARE_EXCHANGE128];
// #endif /* _WIN64 */
//
//     /* Check if we have a deallocation stack */
//     if (!Teb->DeallocationStack)
//     {
//         /* We don't, set one */
//         Status = NtQueryVirtualMemory(NtCurrentProcess(),
//                                       Teb->NtTib.StackLimit,
//                                       MemoryBasicInformation,
//                                       &MemoryBasicInfo,
//                                       sizeof(MEMORY_BASIC_INFORMATION),
//                                       NULL);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Fail */
//             LdrpInitFailure(Status);
//             RtlRaiseStatus(Status);
//             return;
//         }
//
//         /* Set the stack */
//         Teb->DeallocationStack = MemoryBasicInfo.AllocationBase;
//     }
//
//     /* Now check if the process is already being initialized */
//     while (_InterlockedCompareExchange(&LdrpProcessInitialized,
//                                       1,
//                                       0) == 1)
//     {
//         /* Set the timeout to 30 milliseconds */
//         Timeout.QuadPart = Int32x32To64(30, -10000);
//
//         /* Make sure the status hasn't changed */
//         while (LdrpProcessInitialized == 1)
//         {
//             /* Do the wait */
//             ZwDelayExecution(FALSE, &Timeout);
//         }
//     }
//
//     /* Check if we have already setup LDR data */
//     if (!Peb->Ldr)
//     {
//         /* Setup the Loader Lock */
//         Peb->LoaderLock = &LdrpLoaderLock;
//
//         /* Let other code know we're initializing */
//         LdrpInLdrInit = TRUE;
//
//         /* Protect with SEH */
//         _SEH2_TRY
//         {
//             /* Initialize the Process */
//             LoaderStatus = LdrpInitializeProcess(Context,
//                                                  SystemArgument1);
//
//             /* Check for success and if MinimumStackCommit was requested */
//             if (NT_SUCCESS(LoaderStatus) && Peb->MinimumStackCommit)
//             {
//                 /* Enforce the limit */
//                 //LdrpTouchThreadStack(Peb->MinimumStackCommit);
//                 UNIMPLEMENTED;
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Fail with the SEH error */
//             LoaderStatus = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* We're not initializing anymore */
//         LdrpInLdrInit = FALSE;
//
//         /* Check if init worked */
//         if (NT_SUCCESS(LoaderStatus))
//         {
//             /* Set the process as Initialized */
//             _InterlockedIncrement(&LdrpProcessInitialized);
//         }
//     }
//     else
//     {
//         /* Loader data is there... is this a fork() ? */
//         if(Peb->InheritedAddressSpace)
//         {
//             /* Handle the fork() */
//             //LoaderStatus = LdrpForkProcess();
//             LoaderStatus = STATUS_NOT_IMPLEMENTED;
//             UNIMPLEMENTED;
//         }
//         else
//         {
//             /* This is a new thread initializing */
//             LdrpInitializeThread(Context);
//         }
//     }
//
//     /* All done, test alert the thread */
//     NtTestAlert();
//
//     /* Return */
//     if (!NT_SUCCESS(LoaderStatus))
//     {
//         /* Fail */
//         LdrpInitFailure(LoaderStatus);
//         RtlRaiseStatus(LoaderStatus);
//     }
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
