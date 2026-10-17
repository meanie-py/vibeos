use ntdll_sys::nt::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::prelude::memcpy;
// The crate's own functions: the definition is the declaration.
use crate::file::create::CreateFileW;
use core::ffi::c_void;
// NLS substrate: winnls.h types/constants + Wine unicode helpers not in ntdll-sys.
use crate::string::nls_subst::*;

// HeapAlloc/HeapFree are pure forwarders to ntdll in kernel32.def (no @N symbol in
// this module); use the nls_subst Rust wrappers over Rtl*Heap, not extern decls.
use crate::string::nls_subst::{HeapAlloc, HeapFree};

// bindgen defines TRUE/FALSE as u32 but BOOL as c_int (i32). The C uses them as
// BOOL return/assign values; shadow with BOOL-typed consts so the casts vanish.
const TRUE: BOOL = 1;
const FALSE: BOOL = 0;

// ---- kernel32 / NT externs nls.c calls (real stdcall @N exports / forwarders) ----
extern "system" {
    fn SetLastError(dwErrCode: DWORD);
    fn GetLocaleInfoW(Locale: LCID, LCType: LCTYPE, lpLCData: LPWSTR, cchData: INT) -> INT;
    fn GetThreadLocale() -> LCID;
    fn ConvertDefaultLocale(Locale: LCID) -> LCID;
    fn GetProcessHeap() -> HANDLE;
    fn GetSystemDirectoryW(lpBuffer: LPWSTR, uSize: UINT) -> UINT;
    fn MapViewOfFile(
        hFileMappingObject: HANDLE,
        dwDesiredAccess: DWORD,
        dwFileOffsetHigh: DWORD,
        dwFileOffsetLow: DWORD,
        dwNumberOfBytesToMap: SIZE_T,
    ) -> LPVOID;
    fn UnmapViewOfFile(lpBaseAddress: LPCVOID) -> BOOL;
    fn lstrcatA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR;
    fn lstrcpyA(lpString1: LPSTR, lpString2: LPCSTR) -> LPSTR;
    fn lstrlenA(lpString: LPCSTR) -> INT;
}
extern "C" {
    fn strlen(s: *const i8) -> usize;
}
extern "C" {
    // hCurrentModule — the kernel32 self HMODULE, exported as data from dllmain.c.
    pub static mut hCurrentModule: HMODULE;
}

// INVALID_HANDLE_VALUE (winbase.h: (HANDLE)(LONG_PTR)-1) and the CreateFile access
// flag FILE_GENERIC_READ (winnt.h), not carried by ntdll-sys.
const INVALID_HANDLE_VALUE: HANDLE = !0usize as HANDLE; // (HANDLE)-1
const FILE_GENERIC_READ: DWORD = 0x00120089; // winnt.h FILE_GENERIC_READ

// Wine `w!("...")` — a compile-time UTF-16, NUL-terminated wide string literal.
// All call sites pass ASCII; .as_ptr() yields the LPCWSTR. Builds a fixed [u16;N+1].
macro_rules! w {
    ($s:literal) => {{
        const __B: &[u8] = $s.as_bytes();
        const __N: usize = __B.len();
        static __W: [u16; __N + 1] = {
            let mut out = [0u16; __N + 1];
            let mut i = 0;
            while i < __N {
                out[i] = __B[i] as u16;
                i += 1;
            }
            out
        };
        &__W
    }};
}

// #define CONTAINING_RECORD(addr, type, field)
//   ((type*)((PCHAR)(addr) - offsetof(type, field)))
macro_rules! CONTAINING_RECORD {
    ($addr:expr, $type:ty, $field:ident) => {
        (($addr as *const u8).wrapping_sub(core::mem::offset_of!($type, $field)) as *mut $type)
    };
}

// #define InitializeObjectAttributes!(p,n,a,r,s) { ... } (winternl.h)
macro_rules! InitializeObjectAttributes {
    ($p:expr, $n:expr, $a:expr, $r:expr, $s:expr $(,)?) => {{
        (*$p).Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
        (*$p).RootDirectory = $r;
        (*$p).Attributes = $a as ULONG;
        (*$p).ObjectName = $n;
        (*$p).SecurityDescriptor = $s;
        (*$p).SecurityQualityOfService = core::ptr::null_mut();
    }};
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const NLS_SECTION_SECURITY_DESCRIPTOR_SIZE: usize =
    (::core::mem::size_of::<_SECURITY_DESCRIPTOR>() as usize)
        .wrapping_add(::core::mem::size_of::<ACL>() as usize)
        .wrapping_add(
            ((8 as ::core::ffi::c_ulonglong as LONG + 8 as ::core::ffi::c_ulonglong as LONG)
                as usize)
                .wrapping_add(
                    (1 as usize).wrapping_mul(::core::mem::size_of::<[DWORD; 1]>() as usize),
                ),
        );

static mut UTF8Length: [::core::ffi::c_char; 128] = [
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    1 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    2 as ::core::ffi::c_int as ::core::ffi::c_char,
    3 as ::core::ffi::c_int as ::core::ffi::c_char,
    3 as ::core::ffi::c_int as ::core::ffi::c_char,
    3 as ::core::ffi::c_int as ::core::ffi::c_char,
    3 as ::core::ffi::c_int as ::core::ffi::c_char,
    3 as ::core::ffi::c_int as ::core::ffi::c_char,
    3 as ::core::ffi::c_int as ::core::ffi::c_char,
    3 as ::core::ffi::c_int as ::core::ffi::c_char,
    3 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
    0 as ::core::ffi::c_int as ::core::ffi::c_char,
];

static mut UTF8Mask: [::core::ffi::c_uchar; 6] = [
    0x7f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x1f as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0xf as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x7 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x3 as ::core::ffi::c_int as ::core::ffi::c_uchar,
    0x1 as ::core::ffi::c_int as ::core::ffi::c_uchar,
];

static mut UTF8LBound: [::core::ffi::c_ulong; 7] = [
    0 as ::core::ffi::c_int as ::core::ffi::c_ulong,
    0x80 as ::core::ffi::c_int as ::core::ffi::c_ulong,
    0x800 as ::core::ffi::c_int as ::core::ffi::c_ulong,
    0x10000 as ::core::ffi::c_int as ::core::ffi::c_ulong,
    0x200000 as ::core::ffi::c_int as ::core::ffi::c_ulong,
    0x2000000 as ::core::ffi::c_int as ::core::ffi::c_ulong,
    0xffffffff as ::core::ffi::c_uint as ::core::ffi::c_ulong,
];

static mut CodePageListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut AnsiCodePage: CODEPAGE_ENTRY = CODEPAGE_ENTRY {
    Entry: LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
    CodePage: 0,
    SectionHandle: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SectionMapping: ::core::ptr::null::<BYTE>() as *mut BYTE,
    CodePageTable: CPTABLEINFO {
        CodePage: 0,
        MaximumCharacterSize: 0,
        DefaultChar: 0,
        UniDefaultChar: 0,
        TransDefaultChar: 0,
        TransUniDefaultChar: 0,
        DBCSCodePage: 0,
        LeadByte: [0; 12],
        MultiByteTable: ::core::ptr::null::<USHORT>() as *mut USHORT,
        WideCharTable: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        DBCSRanges: ::core::ptr::null::<USHORT>() as *mut USHORT,
        DBCSOffsets: ::core::ptr::null::<USHORT>() as *mut USHORT,
    },
};

static mut OemCodePage: CODEPAGE_ENTRY = CODEPAGE_ENTRY {
    Entry: LIST_ENTRY {
        Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
        Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    },
    CodePage: 0,
    SectionHandle: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SectionMapping: ::core::ptr::null::<BYTE>() as *mut BYTE,
    CodePageTable: CPTABLEINFO {
        CodePage: 0,
        MaximumCharacterSize: 0,
        DefaultChar: 0,
        UniDefaultChar: 0,
        TransDefaultChar: 0,
        TransUniDefaultChar: 0,
        DBCSCodePage: 0,
        LeadByte: [0; 12],
        MultiByteTable: ::core::ptr::null::<USHORT>() as *mut USHORT,
        WideCharTable: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
        DBCSRanges: ::core::ptr::null::<USHORT>() as *mut USHORT,
        DBCSOffsets: ::core::ptr::null::<USHORT>() as *mut USHORT,
    },
};

static mut CodePageListLock: RTL_CRITICAL_SECTION = RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: 0,
    RecursionCount: 0,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0,
};
unsafe fn CreateNlsDirectorySecurity(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    DescriptorSize: SIZE_T,
) -> NTSTATUS {
    static mut NtAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 5],
    };
    let mut Status: NTSTATUS = 0;
    let mut AdminsSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut DaclPresent: BOOLEAN = 0;
    let mut DaclDefaulted: BOOLEAN = 0;

    Status = CreateNlsSecurityDescriptor(
        SecurityDescriptor,
        DescriptorSize,
        (DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT) as ULONG,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = RtlAllocateAndInitializeSid(
        &raw mut NtAuthority,
        2 as UCHAR,
        SECURITY_BUILTIN_DOMAIN_RID as ULONG,
        DOMAIN_ALIAS_RID_ADMINS as ULONG,
        0 as ULONG,
        0 as ULONG,
        0 as ULONG,
        0 as ULONG,
        0 as ULONG,
        0 as ULONG,
        &raw mut AdminsSid,
    );
    if !NT_SUCCESS(Status) {
        // Fall through to cleanup
    } else {
        Status = RtlGetDaclSecurityDescriptor(
            SecurityDescriptor,
            &raw mut DaclPresent,
            &raw mut Dacl,
            &raw mut DaclDefaulted,
        );
        if !NT_SUCCESS(Status) || DaclPresent == 0 || Dacl.is_null() {
            // Fall through to cleanup
        } else {
            Status = RtlAddAccessAllowedAce(
                Dacl,
                ACL_REVISION as ULONG,
                DIRECTORY_ALL_ACCESS as ACCESS_MASK,
                AdminsSid,
            );
            if !NT_SUCCESS(Status) {
                // Fall through to cleanup
            }
        }
    }

    RtlFreeSid(AdminsSid);
    return Status;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn NlsInit() -> BOOL {
    let mut Status: NTSTATUS;
    let mut DirName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut SecurityDescriptor: [UCHAR; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE + NLS_SIZEOF_ACE_AND_SIDS(2)] = [0; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE + NLS_SIZEOF_ACE_AND_SIDS(2)];

    InitializeListHead(&raw mut CodePageListHead);
    RtlInitializeCriticalSection(&raw mut CodePageListLock);

    // C: RtlInitUnicodeString(&DirName, L"\\NLS"). DeepSeek emitted a NUL-less
    // temporary [0x5C,0x4E,0x4C,0x53] whose address dangled; use the static w!.
    RtlInitUnicodeString(&raw mut DirName, w!("\\NLS").as_ptr() as PCWSTR);

    Status = CreateNlsDirectorySecurity(
        &raw mut SecurityDescriptor as PSECURITY_DESCRIPTOR,
        core::mem::size_of::<[UCHAR; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE + NLS_SIZEOF_ACE_AND_SIDS(2)]>() as SIZE_T,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        return FALSE;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut() as HANDLE;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_PERMANENT) as ULONG;
    ObjectAttributes.ObjectName = &raw mut DirName as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = &raw mut SecurityDescriptor as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut() as PVOID;

    Status = NtCreateDirectoryObject(
        &raw mut Handle,
        (DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT) as ACCESS_MASK,
        &raw mut ObjectAttributes,
    );
    if NT_SUCCESS(Status) {
        NtClose(Handle);
    }

    AnsiCodePage.SectionHandle = core::ptr::null_mut() as HANDLE;
    AnsiCodePage.SectionMapping = (*((*NtCurrentTeb()).ProcessEnvironmentBlock)).AnsiCodePageData as PBYTE;

    RtlInitCodePageTable(
        AnsiCodePage.SectionMapping as PUSHORT,
        &raw mut AnsiCodePage.CodePageTable,
    );
    AnsiCodePage.CodePage = AnsiCodePage.CodePageTable.CodePage as UINT;

    InsertTailList(&raw mut CodePageListHead, &raw mut AnsiCodePage.Entry);

    OemCodePage.SectionHandle = core::ptr::null_mut() as HANDLE;
    OemCodePage.SectionMapping = (*((*NtCurrentTeb()).ProcessEnvironmentBlock)).OemCodePageData as PBYTE;

    RtlInitCodePageTable(
        OemCodePage.SectionMapping as PUSHORT,
        &raw mut OemCodePage.CodePageTable,
    );
    OemCodePage.CodePage = OemCodePage.CodePageTable.CodePage as UINT;

    InsertTailList(&raw mut CodePageListHead, &raw mut OemCodePage.Entry);

    return TRUE;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NlsInit() -> BOOL {
    let mut Status: NTSTATUS;
    let mut DirName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut SecurityDescriptor: [UCHAR; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE + NLS_SIZEOF_ACE_AND_SIDS(2)] = [0; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE + NLS_SIZEOF_ACE_AND_SIDS(2)];

    InitializeListHead(&raw mut CodePageListHead);
    RtlInitializeCriticalSection(&raw mut CodePageListLock);

    // C: RtlInitUnicodeString(&DirName, L"\\NLS"). DeepSeek emitted a NUL-less
    // temporary [0x5C,0x4E,0x4C,0x53] whose address dangled; use the static w!.
    RtlInitUnicodeString(&raw mut DirName, w!("\\NLS").as_ptr() as PCWSTR);

    Status = CreateNlsDirectorySecurity(
        &raw mut SecurityDescriptor as PSECURITY_DESCRIPTOR,
        core::mem::size_of::<[UCHAR; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE + NLS_SIZEOF_ACE_AND_SIDS(2)]>() as SIZE_T,
    );
    if !NT_SUCCESS(Status) {
        // DPRINT removed
        return FALSE;
    }

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut() as HANDLE;
    ObjectAttributes.Attributes = (OBJ_CASE_INSENSITIVE | OBJ_PERMANENT) as ULONG;
    ObjectAttributes.ObjectName = &raw mut DirName as PUNICODE_STRING;
    ObjectAttributes.SecurityDescriptor = &raw mut SecurityDescriptor as PVOID;
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut() as PVOID;

    Status = NtCreateDirectoryObject(
        &raw mut Handle,
        (DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT) as ACCESS_MASK,
        &raw mut ObjectAttributes,
    );
    if NT_SUCCESS(Status) {
        NtClose(Handle);
    }

    AnsiCodePage.SectionHandle = core::ptr::null_mut() as HANDLE;
    AnsiCodePage.SectionMapping = (*((*NtCurrentTeb()).ProcessEnvironmentBlock)).AnsiCodePageData as PBYTE;

    RtlInitCodePageTable(
        AnsiCodePage.SectionMapping as PUSHORT,
        &raw mut AnsiCodePage.CodePageTable,
    );
    AnsiCodePage.CodePage = AnsiCodePage.CodePageTable.CodePage as UINT;

    InsertTailList(&raw mut CodePageListHead, &raw mut AnsiCodePage.Entry);

    OemCodePage.SectionHandle = core::ptr::null_mut() as HANDLE;
    OemCodePage.SectionMapping = (*((*NtCurrentTeb()).ProcessEnvironmentBlock)).OemCodePageData as PBYTE;

    RtlInitCodePageTable(
        OemCodePage.SectionMapping as PUSHORT,
        &raw mut OemCodePage.CodePageTable,
    );
    OemCodePage.CodePage = OemCodePage.CodePageTable.CodePage as UINT;

    InsertTailList(&raw mut CodePageListHead, &raw mut OemCodePage.Entry);

    return TRUE;
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn NlsUninit() {
    let mut Current: PCODEPAGE_ENTRY = core::ptr::null_mut();
    while !IsListEmpty(&raw mut CodePageListHead as *mut _) {
        Current = (CodePageListHead.Flink as usize)
            .wrapping_sub(core::mem::offset_of!(CODEPAGE_ENTRY, Entry))
            as *mut CODEPAGE_ENTRY as PCODEPAGE_ENTRY;
        if !(*Current).SectionHandle.is_null() {
            UnmapViewOfFile((*Current).SectionMapping as LPCVOID);
            NtClose((*Current).SectionHandle);
        }
        RemoveHeadList(&raw mut CodePageListHead as *mut _);
    }
    RtlDeleteCriticalSection(&raw mut CodePageListLock as *mut _);
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn NlsUninit() {
    let mut Current: PCODEPAGE_ENTRY = core::ptr::null_mut();
    while !IsListEmpty(&raw mut CodePageListHead as *mut _) {
        Current = (CodePageListHead.Flink as usize)
            .wrapping_sub(core::mem::offset_of!(CODEPAGE_ENTRY, Entry))
            as *mut CODEPAGE_ENTRY as PCODEPAGE_ENTRY;
        if !(*Current).SectionHandle.is_null() {
            UnmapViewOfFile((*Current).SectionMapping as LPCVOID);
            NtClose((*Current).SectionHandle);
        }
        RemoveHeadList(&raw mut CodePageListHead as *mut _);
    }
    RtlDeleteCriticalSection(&raw mut CodePageListLock as *mut _);
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetLoadedCodePageEntry(
    CodePage: UINT,
) -> PCODEPAGE_ENTRY {
    let mut CurrentEntry: *mut LIST_ENTRY;
    let mut Current: PCODEPAGE_ENTRY;

    RtlEnterCriticalSection(&raw mut CodePageListLock);
    CurrentEntry = CodePageListHead.Flink;
    while CurrentEntry != &raw mut CodePageListHead {
        Current = CONTAINING_RECORD!(CurrentEntry, CODEPAGE_ENTRY, Entry);
        if (*Current).CodePage == CodePage {
            RtlLeaveCriticalSection(&raw mut CodePageListLock);
            return Current;
        }
        CurrentEntry = (*CurrentEntry).Flink;
    }
    RtlLeaveCriticalSection(&raw mut CodePageListLock);

    return core::ptr::null_mut();
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetLoadedCodePageEntry(
    CodePage: UINT,
) -> PCODEPAGE_ENTRY {
    let mut CurrentEntry: *mut LIST_ENTRY;
    let mut Current: PCODEPAGE_ENTRY;

    RtlEnterCriticalSection(&raw mut CodePageListLock);
    CurrentEntry = CodePageListHead.Flink;
    while CurrentEntry != &raw mut CodePageListHead {
        Current = CONTAINING_RECORD!(CurrentEntry, CODEPAGE_ENTRY, Entry);
        if (*Current).CodePage == CodePage {
            RtlLeaveCriticalSection(&raw mut CodePageListLock);
            return Current;
        }
        CurrentEntry = (*CurrentEntry).Flink;
    }
    RtlLeaveCriticalSection(&raw mut CodePageListLock);

    return core::ptr::null_mut();
}
#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn IntGetCodePageEntry(mut CodePage: UINT) -> PCODEPAGE_ENTRY {
    let mut Status: NTSTATUS;
    let mut SectionName: [CHAR; 40] = [0; 40];
    let mut SectionHandle: HANDLE = INVALID_HANDLE_VALUE;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut SectionMapping: PBYTE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    // NOTE: plain [UCHAR; N] (1-byte aligned), NOT a SECURITY_DESCRIPTOR-aligned union.
    // The aligned-union form (mirroring the C union at nls.c:300-304) cold-HANGS i386 at
    // the registry hive write: the buffer is exactly NLS_SECTION_SECURITY_DESCRIPTOR_SIZE
    // and CreateNlsSecurityDescriptor fills the DACL to its tail, so raising the alignment
    // from 1→4 shifts adjacent stack locals onto the fill edge and corrupts them. x86
    // tolerates the 1-byte alignment for the DWORD field accesses; the array boots ×3
    // (i386/amd64/arm64). Do not "upgrade" this to an aligned union without re-smoking i386.
    let mut SecurityDescriptor: [UCHAR; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE] = [0; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE];
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: [WCHAR; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut FileNamePos: UINT = 0;
    let mut CodePageEntry: PCODEPAGE_ENTRY = core::ptr::null_mut();

    if CodePage == CP_ACP {
        return &raw mut AnsiCodePage;
    } else if CodePage == CP_OEMCP {
        return &raw mut OemCodePage;
    } else if CodePage == CP_THREAD_ACP {
        if GetLocaleInfoW(
            GetThreadLocale(),
            (LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER) as LCTYPE,
            &mut CodePage as *mut UINT as LPWSTR,
            (core::mem::size_of::<UINT>() / core::mem::size_of::<WCHAR>()) as i32,
        ) == 0
        {
            return core::ptr::null_mut();
        }
        if CodePage == 0 {
            return &raw mut AnsiCodePage;
        }
    } else if CodePage == CP_MACCP {
        if GetLocaleInfoW(
            LOCALE_SYSTEM_DEFAULT,
            (LOCALE_IDEFAULTMACCODEPAGE | LOCALE_RETURN_NUMBER) as LCTYPE,
            &mut CodePage as *mut UINT as LPWSTR,
            (core::mem::size_of::<UINT>() / core::mem::size_of::<WCHAR>()) as i32,
        ) == 0
        {
            return core::ptr::null_mut();
        }
    }

    CodePageEntry = IntGetLoadedCodePageEntry(CodePage);
    if !CodePageEntry.is_null() {
        return CodePageEntry;
    }

    RtlEnterCriticalSection(&raw mut CodePageListLock);

    if GetNlsSectionName(
        CodePage,
        10,
        0,
        b"\\Nls\\NlsSectionCP\0".as_ptr() as LPSTR,
        &mut SectionName as *mut [CHAR; 40] as LPSTR,
        core::mem::size_of::<[CHAR; 40]>() as ULONG,
    ) == 0
    {
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    RtlInitAnsiString(&mut AnsiName, &mut SectionName as *mut [CHAR; 40] as *mut CHAR as PCSZ);
    RtlAnsiStringToUnicodeString(&mut UnicodeName, &mut AnsiName, 1);

    Status = CreateNlsSecurityDescriptor(
        &raw mut SecurityDescriptor as PSECURITY_DESCRIPTOR,
        core::mem::size_of::<[UCHAR; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE]>() as SIZE_T,
        SECTION_MAP_READ as ULONG,
    );
    if !NT_SUCCESS(Status) {
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut UnicodeName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        &raw mut SecurityDescriptor as *mut core::ffi::c_void,
    );

    Status = NtOpenSection(
        &mut SectionHandle,
        SECTION_MAP_READ as ACCESS_MASK,
        &mut ObjectAttributes,
    );

    if Status == STATUS_UNSUCCESSFUL
        || Status == STATUS_OBJECT_NAME_NOT_FOUND
        || Status == STATUS_OBJECT_PATH_NOT_FOUND
    {
        FileNamePos = GetSystemDirectoryW(&mut FileName as *mut [WCHAR; MAX_PATH as usize + 1] as LPWSTR, MAX_PATH as UINT);
        if GetCPFileNameFromRegistry(
            CodePage,
            (&mut FileName as *mut WCHAR).offset(FileNamePos as isize).offset(1),
            (MAX_PATH as UINT).wrapping_sub(FileNamePos).wrapping_sub(1) as ULONG,
        ) != 0
        {
            FileName[FileNamePos as usize] = '\\' as WCHAR;
            FileName[MAX_PATH as usize] = 0;
            FileHandle = CreateFileW(
                &mut FileName as *mut [WCHAR; MAX_PATH as usize + 1] as LPCWSTR,
                FILE_GENERIC_READ as DWORD,
                FILE_SHARE_READ as DWORD,
                core::ptr::null_mut(),
                OPEN_EXISTING as DWORD,
                0,
                core::ptr::null_mut(),
            );

            Status = NtCreateSection(
                &mut SectionHandle,
                SECTION_MAP_READ as ACCESS_MASK,
                &mut ObjectAttributes,
                core::ptr::null_mut(),
                PAGE_READONLY as ULONG,
                SEC_COMMIT as ULONG,
                FileHandle,
            );

            if Status == STATUS_OBJECT_NAME_COLLISION {
                NtClose(FileHandle);
                Status = NtOpenSection(
                    &mut SectionHandle,
                    SECTION_MAP_READ as ACCESS_MASK,
                    &mut ObjectAttributes,
                );
            }
        }
    }

    RtlFreeUnicodeString(&mut UnicodeName);

    if !NT_SUCCESS(Status) {
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    SectionMapping = MapViewOfFile(
        SectionHandle,
        FILE_MAP_READ as DWORD,
        0,
        0,
        0,
    ) as PBYTE;
    if SectionMapping.is_null() {
        NtClose(SectionHandle);
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    CodePageEntry = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<CODEPAGE_ENTRY>(),
    ) as PCODEPAGE_ENTRY;
    if CodePageEntry.is_null() {
        NtClose(SectionHandle);
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    (*CodePageEntry).CodePage = CodePage;
    (*CodePageEntry).SectionHandle = SectionHandle;
    (*CodePageEntry).SectionMapping = SectionMapping;

    RtlInitCodePageTable(
        SectionMapping as PUSHORT,
        &mut (*CodePageEntry).CodePageTable,
    );

    InsertTailList(&raw mut CodePageListHead, &mut (*CodePageEntry).Entry);
    RtlLeaveCriticalSection(&raw mut CodePageListLock);

    return CodePageEntry;
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn IntGetCodePageEntry(mut CodePage: UINT) -> PCODEPAGE_ENTRY {
    let mut Status: NTSTATUS;
    let mut SectionName: [CHAR; 40] = [0; 40];
    let mut SectionHandle: HANDLE = INVALID_HANDLE_VALUE;
    let mut FileHandle: HANDLE = core::ptr::null_mut();
    let mut SectionMapping: PBYTE = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    // NOTE: plain [UCHAR; N] (1-byte aligned), NOT a SECURITY_DESCRIPTOR-aligned union.
    // The aligned-union form (mirroring the C union at nls.c:300-304) cold-HANGS i386 at
    // the registry hive write: the buffer is exactly NLS_SECTION_SECURITY_DESCRIPTOR_SIZE
    // and CreateNlsSecurityDescriptor fills the DACL to its tail, so raising the alignment
    // from 1→4 shifts adjacent stack locals onto the fill edge and corrupts them. x86
    // tolerates the 1-byte alignment for the DWORD field accesses; the array boots ×3
    // (i386/amd64/arm64). Do not "upgrade" this to an aligned union without re-smoking i386.
    let mut SecurityDescriptor: [UCHAR; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE] = [0; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE];
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    let mut UnicodeName: _UNICODE_STRING = core::mem::zeroed();
    let mut FileName: [WCHAR; MAX_PATH as usize + 1] = [0; MAX_PATH as usize + 1];
    let mut FileNamePos: UINT = 0;
    let mut CodePageEntry: PCODEPAGE_ENTRY = core::ptr::null_mut();

    if CodePage == CP_ACP {
        return &raw mut AnsiCodePage;
    } else if CodePage == CP_OEMCP {
        return &raw mut OemCodePage;
    } else if CodePage == CP_THREAD_ACP {
        if GetLocaleInfoW(
            GetThreadLocale(),
            (LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER) as LCTYPE,
            &mut CodePage as *mut UINT as LPWSTR,
            (core::mem::size_of::<UINT>() / core::mem::size_of::<WCHAR>()) as i32,
        ) == 0
        {
            return core::ptr::null_mut();
        }
        if CodePage == 0 {
            return &raw mut AnsiCodePage;
        }
    } else if CodePage == CP_MACCP {
        if GetLocaleInfoW(
            LOCALE_SYSTEM_DEFAULT,
            (LOCALE_IDEFAULTMACCODEPAGE | LOCALE_RETURN_NUMBER) as LCTYPE,
            &mut CodePage as *mut UINT as LPWSTR,
            (core::mem::size_of::<UINT>() / core::mem::size_of::<WCHAR>()) as i32,
        ) == 0
        {
            return core::ptr::null_mut();
        }
    }

    CodePageEntry = IntGetLoadedCodePageEntry(CodePage);
    if !CodePageEntry.is_null() {
        return CodePageEntry;
    }

    RtlEnterCriticalSection(&raw mut CodePageListLock);

    if GetNlsSectionName(
        CodePage,
        10,
        0,
        b"\\Nls\\NlsSectionCP\0".as_ptr() as LPSTR,
        &mut SectionName as *mut [CHAR; 40] as LPSTR,
        core::mem::size_of::<[CHAR; 40]>() as ULONG,
    ) == 0
    {
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    RtlInitAnsiString(&mut AnsiName, &mut SectionName as *mut [CHAR; 40] as *mut CHAR as PCSZ);
    RtlAnsiStringToUnicodeString(&mut UnicodeName, &mut AnsiName, 1);

    Status = CreateNlsSecurityDescriptor(
        &raw mut SecurityDescriptor as PSECURITY_DESCRIPTOR,
        core::mem::size_of::<[UCHAR; NLS_SECTION_SECURITY_DESCRIPTOR_SIZE]>() as SIZE_T,
        SECTION_MAP_READ as ULONG,
    );
    if !NT_SUCCESS(Status) {
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    InitializeObjectAttributes!(
        &mut ObjectAttributes,
        &mut UnicodeName,
        OBJ_CASE_INSENSITIVE,
        core::ptr::null_mut(),
        &raw mut SecurityDescriptor as *mut core::ffi::c_void,
    );

    Status = NtOpenSection(
        &mut SectionHandle,
        SECTION_MAP_READ as ACCESS_MASK,
        &mut ObjectAttributes,
    );

    if Status == STATUS_UNSUCCESSFUL
        || Status == STATUS_OBJECT_NAME_NOT_FOUND
        || Status == STATUS_OBJECT_PATH_NOT_FOUND
    {
        FileNamePos = GetSystemDirectoryW(&mut FileName as *mut [WCHAR; MAX_PATH as usize + 1] as LPWSTR, MAX_PATH as UINT);
        if GetCPFileNameFromRegistry(
            CodePage,
            (&mut FileName as *mut WCHAR).offset(FileNamePos as isize).offset(1),
            (MAX_PATH as UINT).wrapping_sub(FileNamePos).wrapping_sub(1) as ULONG,
        ) != 0
        {
            FileName[FileNamePos as usize] = '\\' as WCHAR;
            FileName[MAX_PATH as usize] = 0;
            FileHandle = CreateFileW(
                &mut FileName as *mut [WCHAR; MAX_PATH as usize + 1] as LPCWSTR,
                FILE_GENERIC_READ as DWORD,
                FILE_SHARE_READ as DWORD,
                core::ptr::null_mut(),
                OPEN_EXISTING as DWORD,
                0,
                core::ptr::null_mut(),
            );

            Status = NtCreateSection(
                &mut SectionHandle,
                SECTION_MAP_READ as ACCESS_MASK,
                &mut ObjectAttributes,
                core::ptr::null_mut(),
                PAGE_READONLY as ULONG,
                SEC_COMMIT as ULONG,
                FileHandle,
            );

            if Status == STATUS_OBJECT_NAME_COLLISION {
                NtClose(FileHandle);
                Status = NtOpenSection(
                    &mut SectionHandle,
                    SECTION_MAP_READ as ACCESS_MASK,
                    &mut ObjectAttributes,
                );
            }
        }
    }

    RtlFreeUnicodeString(&mut UnicodeName);

    if !NT_SUCCESS(Status) {
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    SectionMapping = MapViewOfFile(
        SectionHandle,
        FILE_MAP_READ as DWORD,
        0,
        0,
        0,
    ) as PBYTE;
    if SectionMapping.is_null() {
        NtClose(SectionHandle);
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    CodePageEntry = HeapAlloc(
        GetProcessHeap(),
        0,
        core::mem::size_of::<CODEPAGE_ENTRY>(),
    ) as PCODEPAGE_ENTRY;
    if CodePageEntry.is_null() {
        NtClose(SectionHandle);
        RtlLeaveCriticalSection(&raw mut CodePageListLock);
        return core::ptr::null_mut();
    }

    (*CodePageEntry).CodePage = CodePage;
    (*CodePageEntry).SectionHandle = SectionHandle;
    (*CodePageEntry).SectionMapping = SectionMapping;

    RtlInitCodePageTable(
        SectionMapping as PUSHORT,
        &mut (*CodePageEntry).CodePageTable,
    );

    InsertTailList(&raw mut CodePageListHead, &mut (*CodePageEntry).Entry);
    RtlLeaveCriticalSection(&raw mut CodePageListLock);

    return CodePageEntry;
}
unsafe fn IntMultiByteToWideCharUTF8(
    Flags: DWORD,
    mut MultiByteString: LPCSTR,
    MultiByteCount: INT,
    mut WideCharString: LPWSTR,
    mut WideCharCount: INT,
) -> INT {
    let mut MbsEnd: LPCSTR = core::ptr::null();
    let mut MbsPtrSave: LPCSTR = core::ptr::null();
    let mut Char: UCHAR = 0;
    let mut TrailLength: UCHAR = 0;
    // UINT, not WCHAR: a 4-byte UTF-8 sequence decodes above U+FFFF and must
    // survive long enough to be split into a surrogate pair.
    let mut WideChar: UINT = 0;
    let mut Count: LONG = 0;
    let mut CharIsValid: BOOL = 0;
    let mut StringIsValid: BOOL = 1;
    let InvalidChar: WCHAR = 0xFFFD;

    if Flags != 0 && Flags != MB_ERR_INVALID_CHARS {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    if WideCharCount == 0 {
        MbsEnd = MultiByteString.offset(MultiByteCount as isize);
        while MultiByteString < MbsEnd {
            Char = *MultiByteString as UCHAR;
            MultiByteString = MultiByteString.offset(1);
            if (Char as i32) < 0x80 {
                TrailLength = 0;
            } else if (Char as i32) & 0xC0 == 0x80 {
                TrailLength = 0;
                StringIsValid = 0;
            } else {
                TrailLength = UTF8Length[(Char as i32 - 0x80) as usize] as UCHAR;
                if TrailLength == 0 {
                    StringIsValid = 0;
                } else {
                    CharIsValid = 1;
                    MbsPtrSave = MultiByteString;
                    WideChar = (Char as i32 & UTF8Mask[TrailLength as usize] as i32) as UINT;
                    while TrailLength != 0 && MultiByteString < MbsEnd {
                        if (*MultiByteString as i32) & 0xC0 != 0x80 {
                            StringIsValid = 0;
                            CharIsValid = StringIsValid;
                            break;
                        } else {
                            WideChar = ((WideChar as i32) << 6 | *MultiByteString as i32 & 0x7F) as UINT;
                            MultiByteString = MultiByteString.offset(1);
                            TrailLength = TrailLength.wrapping_sub(1);
                        }
                    }
                    if CharIsValid == 0
                        || (WideChar as u32)
                            < UTF8LBound[UTF8Length[(Char as i32 - 0x80) as usize] as usize]
                    {
                        MultiByteString = MbsPtrSave;
                    } else if WideChar > 0xFFFF {
                        /* Above the BMP: encodes as a UTF-16 surrogate PAIR, so it
                         * costs a second code unit. Upstream-Commit: 5635d244fc2 */
                        WideCharCount += 1;
                    }
                }
            }
            WideCharCount += 1;
        }

        if TrailLength != 0 {
            WideCharCount += 1;
            StringIsValid = 0;
        }

        if Flags == MB_ERR_INVALID_CHARS && StringIsValid == 0 {
            SetLastError(ERROR_NO_UNICODE_TRANSLATION);
            return 0;
        }

        return WideCharCount;
    }

    MbsEnd = MultiByteString.offset(MultiByteCount as isize);
    Count = 0;
    while Count < WideCharCount as LONG && MultiByteString < MbsEnd {
        Char = *MultiByteString as UCHAR;
        MultiByteString = MultiByteString.offset(1);
        if (Char as i32) < 0x80 {
            *WideCharString = Char as WCHAR;
            WideCharString = WideCharString.offset(1);
            TrailLength = 0;
        } else if (Char as i32) & 0xC0 == 0x80 {
            *WideCharString = InvalidChar;
            WideCharString = WideCharString.offset(1);
            TrailLength = 0;
            StringIsValid = 0;
        } else {
            TrailLength = UTF8Length[(Char as i32 - 0x80) as usize] as UCHAR;
            if TrailLength == 0 {
                *WideCharString = InvalidChar;
                WideCharString = WideCharString.offset(1);
                StringIsValid = 0;
            } else {
                CharIsValid = 1;
                MbsPtrSave = MultiByteString;
                WideChar = (Char as i32 & UTF8Mask[TrailLength as usize] as i32) as UINT;
                while TrailLength != 0 && MultiByteString < MbsEnd {
                    if (*MultiByteString as i32) & 0xC0 != 0x80 {
                        StringIsValid = 0;
                        CharIsValid = StringIsValid;
                        break;
                    } else {
                        WideChar = ((WideChar as i32) << 6 | *MultiByteString as i32 & 0x7F) as UINT;
                        MultiByteString = MultiByteString.offset(1);
                        TrailLength = TrailLength.wrapping_sub(1);
                    }
                }
                if CharIsValid != 0
                    && UTF8LBound[UTF8Length[(Char as i32 - 0x80) as usize] as usize]
                        <= WideChar as u32
                {
                    if WideChar > 0xFFFF {
                        /* UTF-16 surrogate pair */
                        let Scalar = WideChar - 0x10000;
                        *WideCharString = (0xD800 | (Scalar >> 10)) as WCHAR;
                        WideCharString = WideCharString.offset(1);
                        Count += 1;
                        /* No room for the low half: this is a hard failure, not a
                         * partial conversion -- breaking here would fall through to
                         * the tail and report a success count. */
                        if Count >= WideCharCount as LONG {
                            SetLastError(ERROR_INSUFFICIENT_BUFFER);
                            return 0;
                        }
                        *WideCharString = (0xDC00 | (Scalar & 0x3FF)) as WCHAR;
                        WideCharString = WideCharString.offset(1);
                    } else {
                        *WideCharString = WideChar as WCHAR;
                        WideCharString = WideCharString.offset(1);
                    }
                } else {
                    *WideCharString = InvalidChar;
                    WideCharString = WideCharString.offset(1);
                    MultiByteString = MbsPtrSave;
                    StringIsValid = 0;
                }
            }
        }
        Count += 1;
    }

    if TrailLength != 0 && Count < WideCharCount as LONG && MultiByteString < MbsEnd {
        *WideCharString = InvalidChar;
        WideCharCount += 1;
    }

    if MultiByteString < MbsEnd {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }

    if Flags == MB_ERR_INVALID_CHARS && (StringIsValid == 0 || TrailLength != 0) {
        SetLastError(ERROR_NO_UNICODE_TRANSLATION);
        return 0;
    }

    return Count;
}
unsafe fn IntMultiByteToWideCharCP(
    CodePage: UINT,
    Flags: DWORD,
    mut MultiByteString: LPCSTR,
    mut MultiByteCount: INT,
    mut WideCharString: LPWSTR,
    mut WideCharCount: INT,
) -> INT {
    let mut CodePageEntry: PCODEPAGE_ENTRY;
    let mut CodePageTable: PCPTABLEINFO;
    let mut MultiByteTable: PUSHORT;
    let mut TempString: LPCSTR;
    let mut TempLength: INT;
    let mut WideChar: USHORT;

    CodePageEntry = IntGetCodePageEntry(CodePage);
    if CodePageEntry.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    CodePageTable = &mut (*CodePageEntry).CodePageTable;

    if (Flags & MB_USEGLYPHCHARS) != 0 && *(*CodePageTable).MultiByteTable.offset(256) != 0 {
        MultiByteTable = (*CodePageTable).MultiByteTable.offset(256 + 1);
    } else {
        MultiByteTable = (*CodePageTable).MultiByteTable;
    }

    if (*CodePageTable).DBCSCodePage != 0 {
        let mut Char: UCHAR;
        let mut DBCSOffset: USHORT;
        let MbsEnd: LPCSTR = MultiByteString.offset(MultiByteCount as isize);
        let mut Count: INT;

        if (Flags & MB_ERR_INVALID_CHARS) != 0 {
            TempString = MultiByteString;

            while TempString < MbsEnd {
                // semdiff: intentionally diverges from C — C uses signed `char` (lead/trail bytes)
                // as a codepage-table index, sign-extending bytes >= 0x80 into a negative index;
                // reading `as UCHAR` (zero-extend) is the correct unsigned 0..255 table index.
                DBCSOffset = *(*CodePageTable).DBCSOffsets.offset(*TempString as UCHAR as isize);

                if DBCSOffset != 0 {
                    if TempString.offset(1) == MbsEnd || *TempString.offset(1) == 0 {
                        SetLastError(ERROR_NO_UNICODE_TRANSLATION);
                        return 0;
                    }

                    WideChar = *(*CodePageTable).DBCSOffsets.offset(
                        (DBCSOffset as INT + *TempString.offset(1) as UCHAR as INT) as isize,
                    );

                    if WideChar == (*CodePageTable).UniDefaultChar
                        && (MAKEWORD(*TempString.offset(1) as UCHAR, *TempString as UCHAR) as USHORT)
                            != (*CodePageTable).TransUniDefaultChar
                    {
                        SetLastError(ERROR_NO_UNICODE_TRANSLATION);
                        return 0;
                    }

                    TempString = TempString.offset(1);
                } else {
                    WideChar = *MultiByteTable.offset(*TempString as UCHAR as isize);

                    if (WideChar == (*CodePageTable).UniDefaultChar
                        && *TempString as i32 != (*CodePageTable).TransUniDefaultChar as i32)
                        || (WideChar >= 0xE000 && WideChar <= 0xF8FF)
                    {
                        SetLastError(ERROR_NO_UNICODE_TRANSLATION);
                        return 0;
                    }
                }

                TempString = TempString.offset(1);
            }
        }

        if WideCharCount == 0 {
            while MultiByteString < MbsEnd {
                Char = *MultiByteString as UCHAR;
                MultiByteString = MultiByteString.offset(1);

                DBCSOffset = *(*CodePageTable).DBCSOffsets.offset(Char as isize);

                // DBCS lead byte: consume the trail byte too. Either way this is ONE
                // wide char, so WideCharCount is incremented unconditionally (the C
                // increments every loop iteration — the DeepSeek `continue` here dropped
                // the count for single-byte chars → returned 0, breaking every caller).
                if DBCSOffset != 0 && MultiByteString < MbsEnd {
                    MultiByteString = MultiByteString.offset(1);
                }

                WideCharCount += 1;
            }

            return WideCharCount;
        }

        Count = 0;
        while Count < WideCharCount && MultiByteString < MbsEnd {
            Char = *MultiByteString as UCHAR;
            MultiByteString = MultiByteString.offset(1);

            DBCSOffset = *(*CodePageTable).DBCSOffsets.offset(Char as isize);

            if DBCSOffset == 0 {
                *WideCharString = *MultiByteTable.offset(Char as isize) as WCHAR;
                WideCharString = WideCharString.offset(1);
            } else if MultiByteString == MbsEnd || *MultiByteString == 0 {
                *WideCharString = (*CodePageTable).UniDefaultChar as WCHAR;
                WideCharString = WideCharString.offset(1);
            } else {
                *WideCharString = *(*CodePageTable).DBCSOffsets.offset(
                    (DBCSOffset as INT + *MultiByteString as UCHAR as INT) as isize,
                ) as WCHAR;
                WideCharString = WideCharString.offset(1);
                MultiByteString = MultiByteString.offset(1);
            }

            Count += 1;
        }

        if MultiByteString < MbsEnd {
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return 0;
        }

        return Count;
    } else {
        if (Flags & MB_ERR_INVALID_CHARS) != 0 {
            TempString = MultiByteString;
            TempLength = MultiByteCount;

            while TempLength > 0 {
                WideChar = *MultiByteTable.offset(*TempString as UCHAR as isize);

                if (WideChar == (*CodePageTable).UniDefaultChar
                    && *TempString as i32 != (*CodePageTable).TransUniDefaultChar as i32)
                    || (WideChar >= 0xE000 && WideChar <= 0xF8FF)
                {
                    SetLastError(ERROR_NO_UNICODE_TRANSLATION);
                    return 0;
                }

                TempString = TempString.offset(1);
                TempLength -= 1;
            }
        }

        if WideCharCount == 0 {
            return MultiByteCount;
        }

        TempLength = if WideCharCount < MultiByteCount {
            WideCharCount
        } else {
            MultiByteCount
        };

        while TempLength > 0 {
            *WideCharString = *MultiByteTable.offset(*MultiByteString as UCHAR as isize) as WCHAR;
            WideCharString = WideCharString.offset(1);
            MultiByteString = MultiByteString.offset(1);
            TempLength -= 1;
        }

        if WideCharCount < MultiByteCount {
            MultiByteCount = WideCharCount;
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return 0;
        }

        return MultiByteCount;
    }
}
unsafe fn IntMultiByteToWideCharSYMBOL(
    Flags: DWORD,
    MultiByteString: LPCSTR,
    MultiByteCount: INT,
    WideCharString: LPWSTR,
    WideCharCount: INT,
) -> INT {
    let mut Count: LONG;
    let mut Char: UCHAR;
    let mut WideCharMaxLen: INT;

    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    if WideCharCount == 0 {
        return MultiByteCount;
    }

    WideCharMaxLen = if WideCharCount > MultiByteCount {
        MultiByteCount
    } else {
        WideCharCount
    };

    Count = 0;
    while Count < WideCharMaxLen {
        Char = *MultiByteString.offset(Count as isize) as UCHAR;
        if (Char as i32) < 0x20 {
            *WideCharString.offset(Count as isize) = Char as WCHAR;
        } else {
            *WideCharString.offset(Count as isize) = (Char as i32 + 0xf000) as WCHAR;
        }
        Count += 1;
    }

    if MultiByteCount > WideCharMaxLen {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }

    return WideCharMaxLen;
}
unsafe fn IntWideCharToMultiByteSYMBOL(
    Flags: DWORD,
    WideCharString: LPCWSTR,
    WideCharCount: INT,
    MultiByteString: LPSTR,
    MultiByteCount: INT,
) -> INT {
    let mut Count: LONG;
    let mut MaxLen: INT;
    let mut Char: WCHAR;

    if Flags != 0 {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    if MultiByteCount == 0 {
        return WideCharCount;
    }

    MaxLen = if MultiByteCount > WideCharCount {
        WideCharCount
    } else {
        MultiByteCount
    };
    Count = 0;
    while Count < MaxLen as LONG {
        Char = *WideCharString.offset(Count as isize);
        if (Char as i32) < 0x20 {
            *MultiByteString.offset(Count as isize) = Char as CHAR;
        } else if (Char as i32) >= 0xf020 && (Char as i32) < 0xf100 {
            *MultiByteString.offset(Count as isize) = (Char as i32 - 0xf000) as CHAR;
        } else {
            SetLastError(ERROR_NO_UNICODE_TRANSLATION);
            return 0;
        }
        Count += 1;
    }

    if WideCharCount > MaxLen {
        SetLastError(ERROR_INSUFFICIENT_BUFFER);
        return 0;
    }
    MaxLen
}
unsafe fn IntWideCharToMultiByteUTF8(
    CodePage: UINT,
    Flags: DWORD,
    mut WideCharString: LPCWSTR,
    mut WideCharCount: INT,
    mut MultiByteString: LPSTR,
    MultiByteCount: INT,
    DefaultChar: LPCSTR,
    UsedDefaultChar: LPBOOL,
) -> INT {
    let mut TempLength: INT;
    let mut Char: DWORD;

    if Flags != 0 {
        SetLastError(ERROR_INVALID_FLAGS);
        return 0;
    }

    /* Does caller query for output buffer size? */
    if MultiByteCount == 0 {
        TempLength = 0;
        while WideCharCount != 0 {
            TempLength += 1;
            if *WideCharString as u32 >= 0x80 {
                TempLength += 1;
                if *WideCharString as u32 >= 0x800 {
                    TempLength += 1;
                    if *WideCharString as u32 >= 0xd800
                        && (*WideCharString as u32) < 0xdc00
                        && WideCharCount >= 1
                        && *WideCharString.offset(1) as u32 >= 0xdc00
                        && *WideCharString.offset(1) as u32 <= 0xe000
                    {
                        WideCharCount -= 1;
                        WideCharString = WideCharString.offset(1);
                        TempLength += 1;
                    }
                }
            }
            WideCharCount -= 1;
            WideCharString = WideCharString.offset(1);
        }
        return TempLength;
    }

    TempLength = MultiByteCount;
    while WideCharCount != 0 {
        Char = *WideCharString as DWORD;
        if Char < 0x80 {
            if TempLength == 0 {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                break;
            }
            TempLength -= 1;
            *MultiByteString = Char as CHAR;
            MultiByteString = MultiByteString.offset(1);
        } else if Char < 0x800 {
            /* 0x80-0x7ff: 2 bytes */
            if TempLength < 2 {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                break;
            }
            *MultiByteString.offset(1) = (0x80 | (Char & 0x3f)) as CHAR;
            Char >>= 6;
            *MultiByteString.offset(0) = (0xc0 | Char) as CHAR;
            MultiByteString = MultiByteString.offset(2);
            TempLength -= 2;
        } else if Char >= 0xd800
            && Char < 0xdc00
            && WideCharCount >= 1
            && (*WideCharString.offset(1) as u32) >= 0xdc00
            && (*WideCharString.offset(1) as u32) < 0xe000
        {
            /* surrogate pair 0x10000-0x10ffff: 4 bytes */
            WideCharCount -= 1;
            WideCharString = WideCharString.offset(1);

            if TempLength < 4 {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                break;
            }

            Char = (Char.wrapping_sub(0xd800)) << 10;
            Char |= (*WideCharString as u32).wrapping_sub(0xdc00);
            Char = Char.wrapping_add(0x10000);

            *MultiByteString.offset(3) = (0x80 | (Char & 0x3f)) as CHAR;
            Char >>= 6;
            *MultiByteString.offset(2) = (0x80 | (Char & 0x3f)) as CHAR;
            Char >>= 6;
            *MultiByteString.offset(1) = (0x80 | (Char & 0x3f)) as CHAR;
            Char >>= 6;
            *MultiByteString.offset(0) = (0xf0 | Char) as CHAR;
            MultiByteString = MultiByteString.offset(4);
            TempLength -= 4;
        } else {
            /* 0x800-0xffff: 3 bytes */
            if TempLength < 3 {
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                break;
            }
            *MultiByteString.offset(2) = (0x80 | (Char & 0x3f)) as CHAR;
            Char >>= 6;
            *MultiByteString.offset(1) = (0x80 | (Char & 0x3f)) as CHAR;
            Char >>= 6;
            *MultiByteString.offset(0) = (0xe0 | Char) as CHAR;
            MultiByteString = MultiByteString.offset(3);
            TempLength -= 3;
        }
        WideCharCount -= 1;
        WideCharString = WideCharString.offset(1);
    }

    MultiByteCount - TempLength
}
unsafe fn IntIsValidSBCSMapping(
    CodePageTable: PCPTABLEINFO,
    Flags: DWORD,
    wch: WCHAR,
    ch: UCHAR,
) -> BOOL {
    if (Flags & WC_NO_BEST_FIT_CHARS) != 0 {
        return (*(*CodePageTable).MultiByteTable.add(ch as usize) == wch) as BOOL;
    }

    if ch as i32 != (*CodePageTable).TransDefaultChar as i32 {
        return TRUE;
    }

    return (wch == (*CodePageTable).TransUniDefaultChar) as BOOL;
}
unsafe fn IntIsValidDBCSMapping(
    CodePageTable: PCPTABLEINFO,
    Flags: DWORD,
    wch: WCHAR,
    ch: USHORT,
) -> BOOL {
    if ch as i32 == (*CodePageTable).TransDefaultChar as i32
        && wch as i32 != (*CodePageTable).TransUniDefaultChar as i32
    {
        return 0;
    }

    if Flags & WC_NO_BEST_FIT_CHARS != 0 {
        if ch as i32 & 0xff00 != 0 {
            let uOffset: USHORT = *(*CodePageTable)
                .DBCSOffsets
                .offset((ch as i32 >> 8) as isize);
            return ((*((*CodePageTable).DBCSOffsets.offset(
                (uOffset as i32 + (ch as i32 & 0xff)) as isize,
            )) as i32) == wch as i32) as i32;
        }

        return ((*((*CodePageTable).MultiByteTable.offset(ch as isize)) as i32) == wch as i32) as i32;
    }

    return 1;
}
unsafe fn IntWideCharToMultiByteCP(
    CodePage: UINT,
    Flags: DWORD,
    WideCharString: LPCWSTR,
    WideCharCount: INT,
    MultiByteString: LPSTR,
    MultiByteCount: INT,
    DefaultChar: LPCSTR,
    mut UsedDefaultChar: LPBOOL,
) -> INT {
    let CodePageEntry: PCODEPAGE_ENTRY;
    let CodePageTable: PCPTABLEINFO;
    let TempLength: INT;

    CodePageEntry = IntGetCodePageEntry(CodePage);
    if CodePageEntry.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    CodePageTable = core::ptr::addr_of_mut!((*CodePageEntry).CodePageTable) as PCPTABLEINFO;

    if (*CodePageTable).DBCSCodePage != 0 {
        if Flags != 0 || !DefaultChar.is_null() || !UsedDefaultChar.is_null() {
            let mut TempUsedDefaultChar: BOOL = 0;
            let mut DefChar: USHORT = 0;

            if UsedDefaultChar.is_null() {
                UsedDefaultChar = core::ptr::addr_of_mut!(TempUsedDefaultChar) as LPBOOL;
            }

            *UsedDefaultChar = 0;

            if !DefaultChar.is_null() {
                // semdiff: intentionally diverges from C — C reads DefaultChar (signed char)
                // and sign-extends bytes >= 0x80 in the `<< 8` shift; reading as u8 (zero-extend)
                // is the correct bit pattern for a DBCS character value (no spurious sign bits).
                DefChar = if *DefaultChar.offset(1) as i32 != 0 {
                    ((*DefaultChar as u8 as i32) << 8 | *DefaultChar.offset(1) as u8 as i32) as USHORT
                } else {
                    *DefaultChar as u8 as USHORT
                };
            } else {
                DefChar = (*CodePageTable).TransDefaultChar;
            }

            if MultiByteCount == 0 {
                let mut TempLength: INT = 0;
                let mut WideCharCount = WideCharCount;
                let mut WideCharString = WideCharString;
                while WideCharCount != 0 {
                    let mut uChar: USHORT;

                    if (Flags as u32 & WC_COMPOSITECHECK as u32) != 0 && WideCharCount > 1 {
                        // FIXME: Handle WC_COMPOSITECHECK
                    }

                    uChar = *((*CodePageTable).WideCharTable as PUSHORT).offset(*WideCharString as isize);

                    if IntIsValidDBCSMapping(CodePageTable, Flags, *WideCharString, uChar) == 0 {
                        uChar = DefChar;
                        *UsedDefaultChar = 1;
                    }

                    if uChar as i32 & 0xff00 != 0 {
                        TempLength += 1;
                    }

                    WideCharCount -= 1;
                    WideCharString = WideCharString.offset(1);
                    TempLength += 1;
                }
                return TempLength;
            }

            let mut TempLength = MultiByteCount;
            let mut WideCharCount = WideCharCount;
            let mut WideCharString = WideCharString;
            let mut MultiByteString = MultiByteString;
            while WideCharCount != 0 && TempLength != 0 {
                let mut uChar: USHORT;

                if (Flags as u32 & WC_COMPOSITECHECK as u32) != 0 && WideCharCount > 1 {
                    // FIXME: Handle WC_COMPOSITECHECK
                }

                uChar = *((*CodePageTable).WideCharTable as PUSHORT).offset(*WideCharString as isize);

                if IntIsValidDBCSMapping(CodePageTable, Flags, *WideCharString, uChar) == 0 {
                    uChar = DefChar;
                    *UsedDefaultChar = 1;
                }

                if uChar as i32 & 0xff00 != 0 {
                    if TempLength == 1 {
                        break;
                    }

                    TempLength -= 1;
                    let fresh0 = MultiByteString;
                    MultiByteString = MultiByteString.offset(1);
                    *fresh0 = (uChar as i32 >> 8) as CHAR;
                }

                let fresh1 = MultiByteString;
                MultiByteString = MultiByteString.offset(1);
                *fresh1 = uChar as CHAR;

                TempLength -= 1;
                WideCharString = WideCharString.offset(1);
                WideCharCount -= 1;
            }

            if WideCharCount != 0 {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0;
            }

            return MultiByteCount - TempLength;
        }

        if MultiByteCount == 0 {
            let mut TempLength: INT = 0;
            let mut WideCharCount = WideCharCount;
            let mut WideCharString = WideCharString;
            while WideCharCount != 0 {
                if *((*CodePageTable).WideCharTable as PWCHAR).offset(*WideCharString as isize) as i32 & 0xff00 != 0 {
                    TempLength += 1;
                }
                WideCharCount -= 1;
                WideCharString = WideCharString.offset(1);
                TempLength += 1;
            }
            return TempLength;
        }

        let mut TempLength = MultiByteCount;
        let mut WideCharCount = WideCharCount;
        let mut WideCharString = WideCharString;
        let mut MultiByteString = MultiByteString;
        while WideCharCount != 0 && TempLength != 0 {
            let uChar: USHORT = *((*CodePageTable).WideCharTable as PUSHORT).offset(*WideCharString as isize);

            if uChar as i32 & 0xff00 != 0 {
                if TempLength == 1 {
                    break;
                }

                TempLength -= 1;
                let fresh2 = MultiByteString;
                MultiByteString = MultiByteString.offset(1);
                *fresh2 = (uChar as i32 >> 8) as CHAR;
            }

            let fresh3 = MultiByteString;
            MultiByteString = MultiByteString.offset(1);
            *fresh3 = uChar as CHAR;

            TempLength -= 1;
            WideCharString = WideCharString.offset(1);
            WideCharCount -= 1;
        }

        if WideCharCount != 0 {
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            return 0;
        }

        return MultiByteCount - TempLength;
    } else {
        let nReturn: INT;

        if Flags != 0 || !DefaultChar.is_null() || !UsedDefaultChar.is_null() {
            let mut TempUsedDefaultChar: BOOL = 0;
            let mut DefChar: CHAR = 0;

            if UsedDefaultChar.is_null() {
                UsedDefaultChar = core::ptr::addr_of_mut!(TempUsedDefaultChar) as LPBOOL;
            }

            *UsedDefaultChar = 0;

            if MultiByteCount == 0 {
                let mut TempLength: INT = 0;
                let mut WideCharCount = WideCharCount;
                let mut WideCharString = WideCharString;
                while WideCharCount != 0 {
                    if (Flags as u32 & WC_COMPOSITECHECK as u32) != 0 && WideCharCount > 1 {
                        // FIXME: Handle WC_COMPOSITECHECK
                    }

                    if *UsedDefaultChar == 0 {
                        *UsedDefaultChar = if IntIsValidSBCSMapping(
                            CodePageTable,
                            Flags,
                            *WideCharString,
                            *((*CodePageTable).WideCharTable as PCHAR).offset(*WideCharString as isize) as UCHAR,
                        ) == 0 {
                            1
                        } else {
                            0
                        };
                    }

                    TempLength += 1;
                    WideCharString = WideCharString.offset(1);
                    WideCharCount -= 1;
                }
                return TempLength;
            }

            if !DefaultChar.is_null() {
                DefChar = *DefaultChar;
            } else {
                DefChar = (*CodePageTable).TransDefaultChar as CHAR;
            }

            let mut TempLength = MultiByteCount;
            let mut WideCharCount = WideCharCount;
            let mut WideCharString = WideCharString;
            let mut MultiByteString = MultiByteString;
            while WideCharCount != 0 && TempLength != 0 {
                if (Flags as u32 & WC_COMPOSITECHECK as u32) != 0 && WideCharCount > 1 {
                    // FIXME: Handle WC_COMPOSITECHECK
                }

                *MultiByteString = *((*CodePageTable).WideCharTable as PCHAR).offset(*WideCharString as isize);

                if IntIsValidSBCSMapping(
                    CodePageTable,
                    Flags,
                    *WideCharString,
                    *MultiByteString as UCHAR,
                ) == 0 {
                    *MultiByteString = DefChar;
                    *UsedDefaultChar = 1;
                }

                MultiByteString = MultiByteString.offset(1);
                TempLength -= 1;
                WideCharString = WideCharString.offset(1);
                WideCharCount -= 1;
            }

            if WideCharCount != 0 {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0;
            }

            return MultiByteCount - TempLength;
        }

        if MultiByteCount == 0 {
            return WideCharCount;
        }

        let nReturn: INT;
        let mut WideCharCount = WideCharCount;
        if MultiByteCount < WideCharCount {
            WideCharCount = MultiByteCount;
            SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
            nReturn = 0;
        } else {
            nReturn = WideCharCount;
        }

        let mut TempLength = WideCharCount;
        let mut WideCharString = WideCharString;
        let mut MultiByteString = MultiByteString;
        loop {
            TempLength -= 1;
            if !(TempLength >= 0) {
                break;
            }
            *MultiByteString = *((*CodePageTable).WideCharTable as PCHAR).offset(*WideCharString as isize);
            WideCharString = WideCharString.offset(1);
            MultiByteString = MultiByteString.offset(1);
        }

        return nReturn;
    }
}
unsafe fn IntIsLeadByte(TableInfo: PCPTABLEINFO, Byte: BYTE) -> BOOL {
    let mut i: UINT = 0;

    if (*TableInfo).MaximumCharacterSize == 2 {
        i = 0;
        while i < MAXIMUM_LEADBYTES && (*TableInfo).LeadByte[i as usize] != 0 {
            if Byte >= (*TableInfo).LeadByte[i as usize]
                && Byte <= (*TableInfo).LeadByte[(i.wrapping_add(1)) as usize]
            {
                return 1;
            }
            i = i.wrapping_add(2);
        }
    }

    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetNlsSectionName(
    CodePage: UINT,
    Base: UINT,
    _Unknown: ULONG,
    BaseName: LPSTR,
    Result: LPSTR,
    ResultSize: ULONG,
) -> BOOL {
    let mut Integer: [CHAR; 11] = [0; 11];
    if !NT_SUCCESS(RtlIntegerToChar(
        CodePage as ULONG,
        Base as ULONG,
        core::mem::size_of::<[CHAR; 11]>() as ULONG,
        Integer.as_mut_ptr() as PCHAR,
    )) {
        return 0;
    }
    if strlen(Integer.as_ptr() as *const i8)
        .wrapping_add(strlen(BaseName as *const i8))
        >= ResultSize as usize
    {
        return 0;
    }
    lstrcpyA(Result, BaseName as LPCSTR);
    lstrcatA(Result, Integer.as_ptr() as LPCSTR);
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn GetCPFileNameFromRegistry(
    CodePage: UINT,
    FileName: LPWSTR,
    FileNameSize: ULONG,
) -> BOOL {
    let mut ValueNameBuffer: [WCHAR; 11] = [0; 11];
    let mut KeyName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ValueName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut Status: NTSTATUS = 0;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut Kvpi: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut KvpiSize: DWORD = 0;
    let mut bRetValue: BOOL = 0;

    bRetValue = 0;

    ValueName.Buffer = ValueNameBuffer.as_mut_ptr();
    ValueName.MaximumLength = core::mem::size_of::<[WCHAR; 11]>() as USHORT;

    if !NT_SUCCESS(RtlIntegerToUnicodeString(CodePage as ULONG, 10, &mut ValueName)) {
        return bRetValue;
    }

    RtlInitUnicodeString(
        &mut KeyName,
        w!("\\Registry\\Machine\\System\\CurrentControlSet\\Control\\Nls\\CodePage").as_ptr(),
    );

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = OBJ_CASE_INSENSITIVE;
    ObjectAttributes.ObjectName = &mut KeyName;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenKey(&mut KeyHandle, KEY_READ, &mut ObjectAttributes);
    if !NT_SUCCESS(Status) {
        return bRetValue;
    }

    KvpiSize = (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as usize)
        .wrapping_add((MAX_PATH as usize).wrapping_mul(core::mem::size_of::<WCHAR>() as usize))
        as DWORD;

    Kvpi = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, KvpiSize as usize)
        as PKEY_VALUE_PARTIAL_INFORMATION;
    if Kvpi.is_null() {
        NtClose(KeyHandle);
        return bRetValue;
    }

    Status = NtQueryValueKey(
        KeyHandle,
        &mut ValueName,
        KeyValuePartialInformation,
        Kvpi as PVOID,
        KvpiSize,
        &mut KvpiSize,
    );

    NtClose(KeyHandle);

    if NT_SUCCESS(Status)
        && (*Kvpi).Type == REG_SZ
        && (*Kvpi).DataLength as usize > core::mem::size_of::<WCHAR>() as usize
    {
        bRetValue = 1;
        if !FileName.is_null() {
            lstrcpynW(
                FileName,
                &raw mut (*Kvpi).Data as *mut WCHAR as LPCWSTR,
                (if ((*Kvpi).DataLength as usize)
                    .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                    < FileNameSize as usize
                {
                    ((*Kvpi).DataLength as usize)
                        .wrapping_div(core::mem::size_of::<WCHAR>() as usize)
                } else {
                    FileNameSize as usize
                }) as i32,
            );
        }
    }

    HeapFree(GetProcessHeap(), 0, Kvpi as PVOID);
    return bRetValue;
}

#[no_mangle]
pub unsafe extern "system" fn IsValidCodePage(CodePage: UINT) -> BOOL {
    if CodePage == 0 {
        return 0;
    }
    if CodePage == CP_UTF8 || CodePage == CP_UTF7 {
        return 1;
    }
    if !IntGetLoadedCodePageEntry(CodePage).is_null() {
        return 1;
    }
    return GetCPFileNameFromRegistry(CodePage, core::ptr::null_mut(), 0);
}
unsafe fn utf7_write_w(
    dst: *mut WCHAR,
    dstlen: i32,
    index: *mut i32,
    character: WCHAR,
) -> BOOL {
    if dstlen > 0 {
        if *index >= dstlen {
            return 0;
        }
        *dst.offset(*index as isize) = character;
    }
    *index += 1;
    return 1;
}
unsafe fn Utf7ToWideChar(
    mut src: *const ::core::ffi::c_char,
    srclen: ::core::ffi::c_int,
    mut dst: *mut WCHAR,
    dstlen: ::core::ffi::c_int,
) -> INT {
    static base64_decoding_table: [::core::ffi::c_schar; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1, -1, 63,
        52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1,
        -1,  0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14,
        15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1,
        -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40,
        41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
    ];

    let source_end: *const ::core::ffi::c_char = src.offset(srclen as isize);
    let mut dest_index: ::core::ffi::c_int = 0;
    let mut byte_pair: DWORD = 0;
    let mut offset: ::core::ffi::c_short = 0;

    while src < source_end {
        if *src as ::core::ffi::c_int == '+' as i32 {
            src = src.offset(1);
            if src >= source_end {
                break;
            }

            if *src as ::core::ffi::c_int == '-' as i32 {
                if utf7_write_w(dst, dstlen, &mut dest_index, '+' as WCHAR) == 0 {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                    return 0;
                }
                src = src.offset(1);
                continue;
            }

            loop {
                let mut sextet: ::core::ffi::c_schar = *src as ::core::ffi::c_schar;
                if sextet as ::core::ffi::c_int == '-' as i32 {
                    src = src.offset(1);
                    offset = 0;
                    break;
                }
                if (sextet as ::core::ffi::c_int) < 0 {
                    break;
                }

                sextet = base64_decoding_table[sextet as usize];
                if sextet as ::core::ffi::c_int == -1 {
                    offset = 0;
                    break;
                }

                byte_pair = ((byte_pair as ::core::ffi::c_ulong) << 6 | sextet as ::core::ffi::c_ulong) as DWORD;
                offset = (offset as ::core::ffi::c_int + 6) as ::core::ffi::c_short;

                if offset as ::core::ffi::c_int >= 16 {
                    if utf7_write_w(
                        dst,
                        dstlen,
                        &mut dest_index,
                        ((byte_pair as ::core::ffi::c_ulong >> (offset as ::core::ffi::c_int - 16)) & 0xFFFF) as WCHAR,
                    ) == 0
                    {
                        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                        return 0;
                    }
                    offset = (offset as ::core::ffi::c_int - 16) as ::core::ffi::c_short;
                }

                src = src.offset(1);
                if !(src < source_end) {
                    break;
                }
            }
        } else {
            if utf7_write_w(
                dst,
                dstlen,
                &mut dest_index,
                *src as ::core::ffi::c_uchar as WCHAR,
            ) == 0
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0;
            }
            src = src.offset(1);
        }
    }

    return dest_index as INT;
}

#[no_mangle]
pub unsafe extern "system" fn MultiByteToWideChar(
    CodePage: UINT,
    Flags: DWORD,
    MultiByteString: LPCSTR,
    MultiByteCount: INT,
    WideCharString: LPWSTR,
    WideCharCount: INT,
) -> INT {
    if MultiByteString.is_null()
        || MultiByteCount == 0
        || WideCharCount < 0
        || (WideCharCount != 0
            && (WideCharString.is_null()
                || MultiByteString as *const core::ffi::c_void == WideCharString as *const core::ffi::c_void))
    {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    let mut MultiByteCount = MultiByteCount;
    if MultiByteCount < 0 {
        MultiByteCount = lstrlenA(MultiByteString) + 1;
    }

    match CodePage {
        65001 => {
            return IntMultiByteToWideCharUTF8(
                Flags,
                MultiByteString,
                MultiByteCount,
                WideCharString,
                WideCharCount,
            );
        }
        65000 => {
            if Flags != 0 {
                SetLastError(ERROR_INVALID_FLAGS);
                return 0;
            }
            return Utf7ToWideChar(
                MultiByteString,
                MultiByteCount,
                WideCharString,
                WideCharCount,
            );
        }
        42 => {
            return IntMultiByteToWideCharSYMBOL(
                Flags,
                MultiByteString,
                MultiByteCount,
                WideCharString,
                WideCharCount,
            );
        }
        _ => {
            return IntMultiByteToWideCharCP(
                CodePage,
                Flags,
                MultiByteString,
                MultiByteCount,
                WideCharString,
                WideCharCount,
            );
        }
    }
}
unsafe fn utf7_can_directly_encode(codepoint: WCHAR) -> BOOL {
    static DIRECTLY_ENCODABLE_TABLE: [BOOL; 123] = [
        1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0,
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1,
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
        0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    ];

    if (codepoint as i32) <= 0x7a {
        DIRECTLY_ENCODABLE_TABLE[codepoint as usize]
    } else {
        0
    }
}
unsafe fn utf7_write_c(
    dst: *mut i8,
    dstlen: i32,
    index: *mut i32,
    character: i8,
) -> BOOL {
    if dstlen > 0 {
        if *index >= dstlen {
            return 0;
        }
        *dst.offset(*index as isize) = character;
    }
    *index += 1;
    return 1;
}
unsafe fn WideCharToUtf7(
    src: *const WCHAR,
    srclen: ::core::ffi::c_int,
    dst: *mut ::core::ffi::c_char,
    dstlen: ::core::ffi::c_int,
) -> INT {
    static base64_encoding_table: [::core::ffi::c_char; 65] = unsafe {
        ::core::mem::transmute::<[u8; 65], [::core::ffi::c_char; 65]>(
            *b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/\0",
        )
    };
    let source_end: *const WCHAR = src.offset(srclen as isize);
    let mut dest_index: ::core::ffi::c_int = 0;
    let mut src = src;
    while src < source_end {
        if *src as ::core::ffi::c_int == '+' as i32 {
            if utf7_write_c(
                dst,
                dstlen,
                &raw mut dest_index,
                '+' as ::core::ffi::c_char,
            ) == 0
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0 as INT;
            }
            if utf7_write_c(
                dst,
                dstlen,
                &raw mut dest_index,
                '-' as ::core::ffi::c_char,
            ) == 0
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0 as INT;
            }
            src = src.offset(1);
        } else if utf7_can_directly_encode(*src) != 0 {
            if utf7_write_c(
                dst,
                dstlen,
                &raw mut dest_index,
                *src as ::core::ffi::c_char,
            ) == 0
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0 as INT;
            }
            src = src.offset(1);
        } else {
            let mut offset: ::core::ffi::c_uint = 0;
            let mut byte_pair: DWORD = 0;
            if utf7_write_c(
                dst,
                dstlen,
                &raw mut dest_index,
                '+' as ::core::ffi::c_char,
            ) == 0
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0 as INT;
            }
            while src < source_end && utf7_can_directly_encode(*src) == 0 {
                byte_pair = ((byte_pair as ::core::ffi::c_ulong) << 16
                    | *src as ::core::ffi::c_ulong) as DWORD;
                offset = offset.wrapping_add(16);
                while offset >= 6 {
                    if utf7_write_c(
                        dst,
                        dstlen,
                        &raw mut dest_index,
                        base64_encoding_table[(byte_pair as ::core::ffi::c_ulong
                            >> offset.wrapping_sub(6)
                            & 0x3f)
                            as usize],
                    ) == 0
                    {
                        SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                        return 0 as INT;
                    }
                    offset = offset.wrapping_sub(6);
                }
                src = src.offset(1);
            }
            if offset != 0 {
                if dstlen > 0 && dest_index + 1 >= dstlen {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                    return 0 as INT;
                }
                byte_pair <<= (6 as ::core::ffi::c_uint).wrapping_sub(offset);
                if utf7_write_c(
                    dst,
                    dstlen,
                    &raw mut dest_index,
                    base64_encoding_table[(byte_pair as ::core::ffi::c_ulong & 0x3f) as usize],
                ) == 0
                {
                    SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                    return 0 as INT;
                }
            }
            if utf7_write_c(
                dst,
                dstlen,
                &raw mut dest_index,
                '-' as ::core::ffi::c_char,
            ) == 0
            {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0 as INT;
            }
        }
    }
    return dest_index as INT;
}
#[no_mangle]
pub unsafe extern "C" fn GetLocalisedText(
    mut uID: UINT,
    lpszDest: LPWSTR,
    cchDest: UINT,
    lang: LANGID,
) -> UINT {
    let mut hrsrc: HRSRC = core::ptr::null_mut();
    let mut hmem: HGLOBAL = core::ptr::null_mut();
    let mut lcid: LCID = 0;
    let mut langId: LANGID = 0;
    let mut p: *const WCHAR = core::ptr::null();
    let mut i: UINT = 0;

    if uID == 37 {
        uID = uID.wrapping_mul(100);
    }

    lcid = ConvertDefaultLocale(lang as LCID);
    langId = LANGIDFROMLCID(lcid);

    if (langId as i32 & 0x3ff) == LANG_NEUTRAL as i32 {
        langId = MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US);
    }

    hrsrc = FindResourceExW(
        hCurrentModule,
        RT_STRING as LPWSTR,
        MAKEINTRESOURCEW(((uID >> 4) + 1) as u16),
        langId as WORD,
    );

    if hrsrc.is_null() {
        hrsrc = FindResourceExW(
            hCurrentModule,
            RT_STRING as LPWSTR,
            MAKEINTRESOURCEW(((uID >> 4) + 1) as u16),
            MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US) as WORD,
        );
    }

    if !hrsrc.is_null() {
        hmem = LoadResource(hCurrentModule, hrsrc);
        if !hmem.is_null() {
            p = LockResource(hmem) as *const WCHAR;

            i = 0;
            while i < (uID & 0x0F) {
                p = p.offset((*p as i32 + 1) as isize);
                i = i.wrapping_add(1);
            }

            if cchDest == 0 {
                return (*p as i32 + 1) as UINT;
            }

            if (*p as i32 + 1) as UINT > cchDest {
                SetLastError(ERROR_INSUFFICIENT_BUFFER as DWORD);
                return 0;
            }

            i = *p as UINT;
            if i > 0 {
                memcpy(
                    lpszDest as *mut core::ffi::c_void,
                    p.offset(1) as *const core::ffi::c_void,
                    (i as usize).wrapping_mul(core::mem::size_of::<WCHAR>()),
                );
                *lpszDest.offset(i as isize) = '\0' as u16;
                return i;
            }
        }
    }

    SetLastError(ERROR_INVALID_PARAMETER as DWORD);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn GetCPInfo(
    CodePage: UINT,
    CodePageInfo: LPCPINFO,
) -> BOOL {
    let CodePageEntry: PCODEPAGE_ENTRY;
    if CodePageInfo.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }
    CodePageEntry = IntGetCodePageEntry(CodePage);
    if CodePageEntry.is_null() {
        match CodePage {
            65000 | 65001 => {
                core::ptr::write_bytes(
                    CodePageInfo as *mut core::ffi::c_void,
                    0,
                    core::mem::size_of::<CPINFO>(),
                );
                (*CodePageInfo).DefaultChar[0] = 0x3f as BYTE;
                (*CodePageInfo).DefaultChar[1] = 0 as BYTE;
                (*CodePageInfo).LeadByte[1] = 0 as BYTE;
                (*CodePageInfo).LeadByte[0] = (*CodePageInfo).LeadByte[1];
                (*CodePageInfo).MaxCharSize = if CodePage == 65000 { 5 } else { 4 } as UINT;
                return TRUE;
            }
            _ => {}
        }
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }
    core::ptr::write_bytes(
        CodePageInfo as *mut core::ffi::c_void,
        0,
        core::mem::size_of::<CPINFO>(),
    );
    if (*CodePageEntry).CodePageTable.DefaultChar as i32 & 0xff00 != 0 {
        (*CodePageInfo).DefaultChar[0] = (((*CodePageEntry).CodePageTable.DefaultChar as i32 & 0xff00) >> 8) as BYTE;
        (*CodePageInfo).DefaultChar[1] = ((*CodePageEntry).CodePageTable.DefaultChar as i32 & 0xff) as BYTE;
    } else {
        (*CodePageInfo).DefaultChar[0] = ((*CodePageEntry).CodePageTable.DefaultChar as i32 & 0xff) as BYTE;
        (*CodePageInfo).DefaultChar[1] = 0 as BYTE;
    }
    (*CodePageInfo).MaxCharSize = (*CodePageEntry).CodePageTable.MaximumCharacterSize as UINT;
    if (*CodePageInfo).MaxCharSize == 2 {
        core::ptr::copy_nonoverlapping(
            &raw const (*CodePageEntry).CodePageTable.LeadByte as *const core::ffi::c_void,
            &raw mut (*CodePageInfo).LeadByte as *mut core::ffi::c_void,
            core::mem::size_of::<[BYTE; 12]>(),
        );
    } else {
        (*CodePageInfo).LeadByte[1] = 0 as BYTE;
        (*CodePageInfo).LeadByte[0] = (*CodePageInfo).LeadByte[1];
    }
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetCPInfoExW(
    CodePage: UINT,
    dwFlags: DWORD,
    lpCPInfoEx: LPCPINFOEXW,
) -> BOOL {
    if GetCPInfo(CodePage, lpCPInfoEx as LPCPINFO) == 0 {
        return FALSE;
    }
    match CodePage {
        65000 => {
            (*lpCPInfoEx).CodePage = 65000;
            (*lpCPInfoEx).UnicodeDefaultChar = 0x3f;
            return (GetLocalisedText(
                (*lpCPInfoEx).CodePage,
                &raw mut (*lpCPInfoEx).CodePageName as LPWSTR,
                (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as UINT,
                GetThreadLocale() as LANGID,
            ) != 0) as BOOL;
        }
        65001 => {
            (*lpCPInfoEx).CodePage = 65001;
            (*lpCPInfoEx).UnicodeDefaultChar = 0x3f;
            return (GetLocalisedText(
                (*lpCPInfoEx).CodePage,
                &raw mut (*lpCPInfoEx).CodePageName as LPWSTR,
                (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as UINT,
                GetThreadLocale() as LANGID,
            ) != 0) as BOOL;
        }
        _ => {
            let CodePageEntry: PCODEPAGE_ENTRY = IntGetCodePageEntry(CodePage);
            if CodePageEntry.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
                return FALSE;
            }
            (*lpCPInfoEx).CodePage = (*CodePageEntry).CodePageTable.CodePage as UINT;
            (*lpCPInfoEx).UnicodeDefaultChar = (*CodePageEntry).CodePageTable.UniDefaultChar as WCHAR;
            return (GetLocalisedText(
                (*lpCPInfoEx).CodePage,
                &raw mut (*lpCPInfoEx).CodePageName as LPWSTR,
                (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as UINT,
                GetThreadLocale() as LANGID,
            ) != 0) as BOOL;
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn GetCPInfoExA(
    CodePage: UINT,
    dwFlags: DWORD,
    lpCPInfoEx: LPCPINFOEXA,
) -> BOOL {
    let mut CPInfo: CPINFOEXW = core::mem::zeroed();
    if GetCPInfoExW(CodePage, dwFlags, &mut CPInfo) == 0 {
        return 0;
    }
    core::ptr::copy_nonoverlapping(
        &CPInfo as *const CPINFOEXW as *const u8,
        lpCPInfoEx as *mut u8,
        core::mem::size_of::<CPINFOEXA>(),
    );
    WideCharToMultiByte(
        0, // CP_ACP
        0,
        CPInfo.CodePageName.as_ptr(),
        -1,
        (*lpCPInfoEx).CodePageName.as_mut_ptr(),
        core::mem::size_of_val(&(*lpCPInfoEx).CodePageName) as i32,
        core::ptr::null(),
        core::ptr::null_mut(),
    );
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn WideCharToMultiByte(
    CodePage: UINT,
    Flags: DWORD,
    WideCharString: LPCWSTR,
    mut WideCharCount: INT,
    MultiByteString: LPSTR,
    MultiByteCount: INT,
    DefaultChar: LPCSTR,
    UsedDefaultChar: LPBOOL,
) -> INT {
    if WideCharString.is_null()
        || WideCharCount == 0
        || (MultiByteString.is_null() && MultiByteCount > 0)
        || (WideCharString as PVOID) == (MultiByteString as PVOID)
        || MultiByteCount < 0
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    if WideCharCount < 0 {
        WideCharCount = lstrlenW(WideCharString) + 1;
    }

    match CodePage {
        65001 => {
            if !DefaultChar.is_null() || !UsedDefaultChar.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
                return 0;
            }
            return IntWideCharToMultiByteUTF8(
                CodePage,
                Flags,
                WideCharString,
                WideCharCount,
                MultiByteString,
                MultiByteCount,
                DefaultChar,
                UsedDefaultChar,
            );
        }
        65000 => {
            if !DefaultChar.is_null() || !UsedDefaultChar.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
                return 0;
            }
            if Flags != 0 {
                SetLastError(ERROR_INVALID_FLAGS as DWORD);
                return 0;
            }
            return WideCharToUtf7(
                WideCharString as *const WCHAR,
                WideCharCount,
                MultiByteString as *mut i8,
                MultiByteCount,
            );
        }
        42 => {
            if !DefaultChar.is_null() || !UsedDefaultChar.is_null() {
                SetLastError(ERROR_INVALID_PARAMETER as DWORD);
                return 0;
            }
            return IntWideCharToMultiByteSYMBOL(
                Flags,
                WideCharString,
                WideCharCount,
                MultiByteString,
                MultiByteCount,
            );
        }
        _ => {
            return IntWideCharToMultiByteCP(
                CodePage,
                Flags,
                WideCharString,
                WideCharCount,
                MultiByteString,
                MultiByteCount,
                DefaultChar,
                UsedDefaultChar,
            );
        }
    }
}
#[no_mangle]
pub unsafe extern "system" fn GetACP() -> UINT {
    return AnsiCodePage.CodePageTable.CodePage as UINT;
}
#[no_mangle]
pub unsafe extern "system" fn GetOEMCP() -> UINT {
    return OemCodePage.CodePageTable.CodePage as UINT;
}
#[no_mangle]
pub unsafe extern "system" fn IsDBCSLeadByteEx(CodePage: UINT, TestByte: BYTE) -> BOOL {
    let CodePageEntry: PCODEPAGE_ENTRY;
    CodePageEntry = IntGetCodePageEntry(CodePage);
    if !CodePageEntry.is_null() {
        return IntIsLeadByte(core::ptr::addr_of_mut!((*CodePageEntry).CodePageTable), TestByte);
    }
    SetLastError(ERROR_INVALID_PARAMETER);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn IsDBCSLeadByte(TestByte: BYTE) -> BOOL {
    return IntIsLeadByte(core::ptr::addr_of_mut!(AnsiCodePage.CodePageTable), TestByte);
}

#[no_mangle]
pub unsafe extern "system" fn CreateNlsSecurityDescriptor(
    SecurityDescriptor: PSECURITY_DESCRIPTOR,
    DescriptorSize: SIZE_T,
    AccessMask: ULONG,
) -> NTSTATUS {
    static mut WorldAuthority: SID_IDENTIFIER_AUTHORITY = SID_IDENTIFIER_AUTHORITY {
        Value: [0, 0, 0, 0, 0, 1],
    };
    let mut Status: NTSTATUS = 0;
    let mut WorldSid: PSID = core::ptr::null_mut();
    let mut Dacl: PACL = core::ptr::null_mut();
    let mut DaclSize: ULONG = 0;

    if (DescriptorSize as usize) < NLS_SECTION_SECURITY_DESCRIPTOR_SIZE {
        return STATUS_BUFFER_TOO_SMALL;
    }

    Status = RtlAllocateAndInitializeSid(
        &raw mut WorldAuthority,
        1,
        SECURITY_WORLD_RID,
        0, 0, 0, 0, 0, 0, 0,
        &raw mut WorldSid,
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    Status = RtlCreateSecurityDescriptor(SecurityDescriptor, SECURITY_DESCRIPTOR_REVISION);
    if !NT_SUCCESS(Status) {
        RtlFreeSid(WorldSid);
        return Status;
    }

    Dacl = (SecurityDescriptor as usize)
        .wrapping_add(core::mem::size_of::<_SECURITY_DESCRIPTOR>())
        as PACL;
    DaclSize = (DescriptorSize as usize)
        .wrapping_sub(core::mem::size_of::<_SECURITY_DESCRIPTOR>())
        as ULONG;

    Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION);
    if !NT_SUCCESS(Status) {
        RtlFreeSid(WorldSid);
        return Status;
    }

    Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, AccessMask, WorldSid);
    if !NT_SUCCESS(Status) {
        RtlFreeSid(WorldSid);
        return Status;
    }

    Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor, 1, Dacl, 0);
    if !NT_SUCCESS(Status) {
        RtlFreeSid(WorldSid);
        return Status;
    }

    RtlFreeSid(WorldSid);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn IsValidUILanguage(langid: LANGID) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const ::core::ffi::c_char,
        2475 as ::core::ffi::c_int,
        b"IsValidUILanguage\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const ::core::ffi::c_char,
            2475 as ::core::ffi::c_int,
        );
    }
    return 0 as BOOL;
}
#[no_mangle]
pub unsafe extern "system" fn NlsConvertIntegerToString(
    Value: ULONG,
    Base: ULONG,
    strsize: ULONG,
    str: LPWSTR,
    strsize2: ULONG,
) {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const ::core::ffi::c_char,
        2484 as ::core::ffi::c_int,
        b"NlsConvertIntegerToString\0".as_ptr() as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const ::core::ffi::c_char,
            2484 as ::core::ffi::c_int,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetCPGlobal(CodePage: UINT) -> UINT {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2492i32,
        b"SetCPGlobal\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2492i32,
        );
    }
    return 0u32;
}

#[no_mangle]
pub unsafe extern "system" fn ValidateLCType(
    a1: i32,
    a2: u32,
    a3: i32,
    a4: i32,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as u32);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2503i32,
        b"ValidateLCType\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2503i32,
        );
    }
    return 0i32;
}

#[no_mangle]
pub unsafe extern "system" fn NlsResetProcessLocale() -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2514i32,
        b"NlsResetProcessLocale\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2514i32,
        );
    }
    return TRUE;
}
#[no_mangle]
pub unsafe extern "system" fn GetDefaultSortkeySize(mut lpUnknown: LPVOID) {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2525i32,
        b"GetDefaultSortkeySize\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as PCSTR,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2525i32,
        );
    }
    lpUnknown = core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn GetLinguistLangSize(mut lpUnknown: LPVOID) {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as u32);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2536i32,
        b"GetLinguistLangSize\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2536i32,
        );
    }
    lpUnknown = core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn ValidateLocale(LocaleId: ULONG) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2547i32,
        b"ValidateLocale\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2547i32,
        );
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn NlsGetCacheUpdateCount() -> ULONG {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2558i32,
        b"NlsGetCacheUpdateCount\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2558i32,
        );
    }
    return 0 as ULONG;
}

#[no_mangle]
pub unsafe extern "system" fn IsNLSDefinedString(
    Function: NLS_FUNCTION,
    dwFlags: DWORD,
    lpVersionInformation: LPNLSVERSIONINFO,
    lpString: LPCWSTR,
    cchStr: INT,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2573i32,
        b"IsNLSDefinedString\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2573i32,
        );
    }
    return 1i32;
}

#[no_mangle]
pub unsafe extern "system" fn GetNLSVersion(
    Function: NLS_FUNCTION,
    Locale: LCID,
    lpVersionInformation: LPNLSVERSIONINFO,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0" as *const u8 as PCSTR,
        b"dll/win32/kernel32/winnls/string/nls.c\0"
            as *const u8 as *const ::core::ffi::c_char,
        2586 as ::core::ffi::c_int,
        b"GetNLSVersion\0" as *const u8 as *const ::core::ffi::c_char,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as PCSTR,
            b"dll/win32/kernel32/winnls/string/nls.c\0"
                as *const u8 as *const ::core::ffi::c_char,
            2586 as ::core::ffi::c_int,
        );
    }
    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn GetNLSVersionEx(
    function: NLS_FUNCTION,
    lpLocaleName: LPCWSTR,
    lpVersionInformation: LPNLSVERSIONINFOEX,
) -> BOOL {
    SetLastError(ERROR_CALL_NOT_IMPLEMENTED as DWORD);
    if DbgPrint(
        b"(%s:%d) %s() is unimplemented!()\n\0".as_ptr() as *const i8,
        b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
        2599i32,
        b"GetNLSVersionEx\0".as_ptr() as *const i8,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"dll/win32/kernel32/winnls/string/nls.c\0".as_ptr() as *const i8,
            2599i32,
        );
    }
    return 1i32;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/winnls/string/nls.c
//  * PURPOSE:         National Language Support
//  * PROGRAMMER:      Filip Navara
//  *                  Hartmut Birr
//  *                  Gunnar Andre Dalsnes
//  *                  Thomas Weidenmueller
//  *                  Katayama Hirofumi MZ
//  * UPDATE HISTORY:
//  *                  Created 24/08/2004
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBAL VARIABLES ***********************************************************/
//
// /* Sequence length based on the first character. */
// static const char UTF8Length[128] =
// {
//    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* 0x80 - 0x8F */
//    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* 0x90 - 0x9F */
//    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* 0xA0 - 0xAF */
//    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* 0xB0 - 0xBF */
//    0, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, /* 0xC0 - 0xCF */
//    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, /* 0xD0 - 0xDF */
//    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, /* 0xE0 - 0xEF */
//    3, 3, 3, 3, 3, 3, 3, 3, 0, 0, 0, 0, 0, 0, 0, 0  /* 0xF0 - 0xFF */
// };
//
// /* First byte mask depending on UTF-8 sequence length. */
// static const unsigned char UTF8Mask[6] = {0x7f, 0x1f, 0x0f, 0x07, 0x03, 0x01};
//
// /* UTF-8 length to lower bound */
// static const unsigned long UTF8LBound[] =
//     {0, 0x80, 0x800, 0x10000, 0x200000, 0x2000000, 0xFFFFFFFF};
//
// /* FIXME: Change to HASH table or linear array. */
// static LIST_ENTRY CodePageListHead;
// static CODEPAGE_ENTRY AnsiCodePage;
// static CODEPAGE_ENTRY OemCodePage;
// static RTL_CRITICAL_SECTION CodePageListLock;
//
// /* FORWARD DECLARATIONS *******************************************************/
//
// BOOL WINAPI
// GetNlsSectionName(UINT CodePage, UINT Base, ULONG Unknown,
//                   LPSTR BaseName, LPSTR Result, ULONG ResultSize);
//
// BOOL WINAPI
// GetCPFileNameFromRegistry(UINT CodePage, LPWSTR FileName, ULONG FileNameSize);
//
// NTSTATUS
// WINAPI
// CreateNlsSecurityDescriptor(
//      PSECURITY_DESCRIPTOR SecurityDescriptor,
//      SIZE_T DescriptorSize,
//      ULONG AccessMask);
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /**
//  * @brief
//  * Creates a security descriptor for the NLS object directory.
//  *
//  * @param[out]  SecurityDescriptor
//  * @param[in]   DescriptorSize
//  * Same parameters as for CreateNlsSecurityDescriptor().
//  *
//  * @remark
//  * Everyone (World SID) is given read access to the NLS directory,
//  * whereas Admins are given full access.
//  */
// static NTSTATUS
// CreateNlsDirectorySecurity(
//      PSECURITY_DESCRIPTOR SecurityDescriptor,
//      SIZE_T DescriptorSize)
// {
//     static SID_IDENTIFIER_AUTHORITY NtAuthority = {SECURITY_NT_AUTHORITY};
//     NTSTATUS Status;
//     PSID AdminsSid;
//     PACL Dacl;
//     BOOLEAN DaclPresent, DaclDefaulted;
//
//     /* Give everyone basic directory access */
//     Status = CreateNlsSecurityDescriptor(SecurityDescriptor,
//                                          DescriptorSize,
//                                          DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create basic NLS SD (Status 0x%08x)\n", Status);
//         return Status;
//     }
//
//     /* Create the Admins SID */
//     // NOTE: Win <= 2k3 uses SYSTEM instead (SECURITY_LOCAL_SYSTEM_RID with one SubAuthority)
//     Status = RtlAllocateAndInitializeSid(&NtAuthority,
//                                          2,
//                                          SECURITY_BUILTIN_DOMAIN_RID,
//                                          DOMAIN_ALIAS_RID_ADMINS,
//                                          0, 0, 0, 0, 0, 0,
//                                          &AdminsSid);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create Admins SID (Status 0x%08x)\n", Status);
//         goto Quit;
//     }
//
//     /* Retrieve the DACL from the descriptor */
//     Status = RtlGetDaclSecurityDescriptor(SecurityDescriptor,
//                                           &DaclPresent,
//                                           &Dacl,
//                                           &DaclDefaulted);
//     if (!NT_SUCCESS(Status) || !DaclPresent || !Dacl)
//     {
//         DPRINT1("Failed to get DACL from descriptor (Status 0x%08x)\n", Status);
//         goto Quit;
//     }
//
//     /* Add an allowed access ACE to the Admins SID with full access.
//      * The function verifies the DACL is large enough to accommodate it. */
//     Status = RtlAddAccessAllowedAce(Dacl,
//                                     ACL_REVISION,
//                                     DIRECTORY_ALL_ACCESS,
//                                     AdminsSid);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to add allowed access ACE for Admins SID (Status 0x%08x)\n", Status);
//         goto Quit;
//     }
//
// Quit:
//     RtlFreeSid(AdminsSid);
//     return Status;
// }
//
// /**
//  * @name NlsInit
//  *
//  * Internal NLS related stuff initialization.
//  */
//
// BOOL
// FASTCALL
// NlsInit(VOID)
// {
//     NTSTATUS Status;
//     UNICODE_STRING DirName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     HANDLE Handle;
//     UCHAR SecurityDescriptor[NLS_SECTION_SECURITY_DESCRIPTOR_SIZE +
//                              NLS_SIZEOF_ACE_AND_SIDS(2)];
//
//     InitializeListHead(&CodePageListHead);
//     RtlInitializeCriticalSection(&CodePageListLock);
//
//     /*
//      * FIXME: Eventually this should be done only for the NLS Server
//      * process, but since we don't have anything like that (yet?) we
//      * always try to create the "\NLS" directory here.
//      */
//     RtlInitUnicodeString(&DirName, L"\\NLS");
//
//     /* Create a security descriptor for the NLS directory */
//     Status = CreateNlsDirectorySecurity(&SecurityDescriptor,
//                                         sizeof(SecurityDescriptor));
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create NLS directory security (Status 0x%08x)\n", Status);
//         return FALSE;
//     }
//
//     InitializeObjectAttributes!(&ObjectAttributes,
//                                &DirName,
//                                OBJ_CASE_INSENSITIVE | OBJ_PERMANENT,
//                                NULL,
//                                &SecurityDescriptor);
//
//     Status = NtCreateDirectoryObject(&Handle,
//                                      DIRECTORY_TRAVERSE | DIRECTORY_CREATE_OBJECT,
//                                      &ObjectAttributes);
//     if (NT_SUCCESS(Status))
//     {
//         NtClose(Handle);
//     }
//
//     /* Setup ANSI code page. */
//     AnsiCodePage.SectionHandle = NULL;
//     AnsiCodePage.SectionMapping = NtCurrentTeb()->ProcessEnvironmentBlock->AnsiCodePageData;
//
//     RtlInitCodePageTable((PUSHORT)AnsiCodePage.SectionMapping,
//                          &AnsiCodePage.CodePageTable);
//     AnsiCodePage.CodePage = AnsiCodePage.CodePageTable.CodePage;
//
//     InsertTailList(&CodePageListHead, &AnsiCodePage.Entry);
//
//     /* Setup OEM code page. */
//     OemCodePage.SectionHandle = NULL;
//     OemCodePage.SectionMapping = NtCurrentTeb()->ProcessEnvironmentBlock->OemCodePageData;
//
//     RtlInitCodePageTable((PUSHORT)OemCodePage.SectionMapping,
//                          &OemCodePage.CodePageTable);
//     OemCodePage.CodePage = OemCodePage.CodePageTable.CodePage;
//     InsertTailList(&CodePageListHead, &OemCodePage.Entry);
//
//     return TRUE;
// }
//
// /**
//  * @name NlsUninit
//  *
//  * Internal NLS related stuff uninitialization.
//  */
//
// VOID
// FASTCALL
// NlsUninit(VOID)
// {
//     PCODEPAGE_ENTRY Current;
//
//     /* Delete the code page list. */
//     while (!IsListEmpty(&CodePageListHead))
//     {
//         Current = CONTAINING_RECORD!(CodePageListHead.Flink, CODEPAGE_ENTRY, Entry);
//         if (Current->SectionHandle != NULL)
//         {
//             UnmapViewOfFile(Current->SectionMapping);
//             NtClose(Current->SectionHandle);
//         }
//         RemoveHeadList(&CodePageListHead);
//     }
//     RtlDeleteCriticalSection(&CodePageListLock);
// }
//
// /**
//  * @name IntGetLoadedCodePageEntry
//  *
//  * Internal function to get structure containing a code page information
//  * of code page that is already loaded.
//  *
//  * @param CodePage
//  *        Number of the code page. Special values like CP_OEMCP, CP_ACP
//  *        or CP_UTF8 aren't allowed.
//  *
//  * @return Code page entry or NULL if the specified code page hasn't
//  *         been loaded yet.
//  */
//
// PCODEPAGE_ENTRY
// FASTCALL
// IntGetLoadedCodePageEntry(UINT CodePage)
// {
//     LIST_ENTRY *CurrentEntry;
//     PCODEPAGE_ENTRY Current;
//
//     RtlEnterCriticalSection(&CodePageListLock);
//     for (CurrentEntry = CodePageListHead.Flink;
//          CurrentEntry != &CodePageListHead;
//          CurrentEntry = CurrentEntry->Flink)
//     {
//         Current = CONTAINING_RECORD!(CurrentEntry, CODEPAGE_ENTRY, Entry);
//         if (Current->CodePage == CodePage)
//         {
//             RtlLeaveCriticalSection(&CodePageListLock);
//             return Current;
//         }
//     }
//     RtlLeaveCriticalSection(&CodePageListLock);
//
//     return NULL;
// }
//
// /**
//  * @name IntGetCodePageEntry
//  *
//  * Internal function to get structure containing a code page information.
//  *
//  * @param CodePage
//  *        Number of the code page. Special values like CP_OEMCP, CP_ACP
//  *        or CP_THREAD_ACP are allowed, but CP_UTF[7/8] isn't.
//  *
//  * @return Code page entry.
//  */
//
// PCODEPAGE_ENTRY
// FASTCALL
// IntGetCodePageEntry(UINT CodePage)
// {
//     NTSTATUS Status;
//     CHAR SectionName[40];
//     HANDLE SectionHandle = INVALID_HANDLE_VALUE, FileHandle;
//     PBYTE SectionMapping;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     union
//     {
//         SECURITY_DESCRIPTOR AlignedSd;
//         UCHAR Buffer[NLS_SECTION_SECURITY_DESCRIPTOR_SIZE];
//     } SecurityDescriptor;
//     ANSI_STRING AnsiName;
//     UNICODE_STRING UnicodeName;
//     WCHAR FileName[MAX_PATH + 1];
//     UINT FileNamePos;
//     PCODEPAGE_ENTRY CodePageEntry;
//
//     if (CodePage == CP_ACP)
//     {
//         return &AnsiCodePage;
//     }
//     else if (CodePage == CP_OEMCP)
//     {
//         return &OemCodePage;
//     }
//     else if (CodePage == CP_THREAD_ACP)
//     {
//         if (!GetLocaleInfoW(GetThreadLocale(),
//                             LOCALE_IDEFAULTANSICODEPAGE | LOCALE_RETURN_NUMBER,
//                             (WCHAR *)&CodePage,
//                             sizeof(CodePage) / sizeof(WCHAR)))
//         {
//             /* Last error is set by GetLocaleInfoW. */
//             return NULL;
//         }
//         if (CodePage == 0)
//             return &AnsiCodePage;
//     }
//     else if (CodePage == CP_MACCP)
//     {
//         if (!GetLocaleInfoW(LOCALE_SYSTEM_DEFAULT,
//                             LOCALE_IDEFAULTMACCODEPAGE | LOCALE_RETURN_NUMBER,
//                             (WCHAR *)&CodePage,
//                             sizeof(CodePage) / sizeof(WCHAR)))
//         {
//             /* Last error is set by GetLocaleInfoW. */
//             return NULL;
//         }
//     }
//
//     /* Try searching for loaded page first. */
//     CodePageEntry = IntGetLoadedCodePageEntry(CodePage);
//     if (CodePageEntry != NULL)
//     {
//         return CodePageEntry;
//     }
//
//     /*
//      * Yes, we really want to lock here. Otherwise it can happen that
//      * two parallel requests will try to get the entry for the same
//      * code page and we would load it twice.
//      */
//     RtlEnterCriticalSection(&CodePageListLock);
//
//     /* Generate the section name. */
//     if (!GetNlsSectionName(CodePage,
//                            10,
//                            0,
//                            "\\Nls\\NlsSectionCP",
//                            SectionName,
//                            sizeof(SectionName)))
//     {
//         RtlLeaveCriticalSection(&CodePageListLock);
//         return NULL;
//     }
//
//     RtlInitAnsiString(&AnsiName, SectionName);
//     RtlAnsiStringToUnicodeString(&UnicodeName, &AnsiName, TRUE);
//
//     /*
//      * FIXME: IntGetCodePageEntry should not create any security
//      * descriptor here but instead this responsibility should be
//      * assigned to Base Server API (aka basesrv.dll). That is,
//      * kernel32 must instruct basesrv.dll on creating NLS section
//      * names that do not exist through API message communication.
//      * However since we do not do that, let the kernel32 do the job
//      * by assigning security to NLS section names for the time being...
//      */
//     Status = CreateNlsSecurityDescriptor(&SecurityDescriptor,
//                                          sizeof(SecurityDescriptor),
//                                          SECTION_MAP_READ);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("CreateNlsSecurityDescriptor FAILED! (Status 0x%08x)\n", Status);
//         RtlLeaveCriticalSection(&CodePageListLock);
//         return NULL;
//     }
//
//     InitializeObjectAttributes!(&ObjectAttributes,
//                                &UnicodeName,
//                                OBJ_CASE_INSENSITIVE,
//                                NULL,
//                                &SecurityDescriptor);
//
//     /* Try to open the section first */
//     Status = NtOpenSection(&SectionHandle,
//                            SECTION_MAP_READ,
//                            &ObjectAttributes);
//
//     /* If the section doesn't exist, try to create it. */
//     if (Status == STATUS_UNSUCCESSFUL ||
//         Status == STATUS_OBJECT_NAME_NOT_FOUND ||
//         Status == STATUS_OBJECT_PATH_NOT_FOUND)
//     {
//         FileNamePos = GetSystemDirectoryW(FileName, MAX_PATH);
//         if (GetCPFileNameFromRegistry(CodePage,
//                                       FileName + FileNamePos + 1,
//                                       MAX_PATH - FileNamePos - 1))
//         {
//             FileName[FileNamePos] = L'\\';
//             FileName[MAX_PATH] = 0;
//             FileHandle = CreateFileW(FileName,
//                                      FILE_GENERIC_READ,
//                                      FILE_SHARE_READ,
//                                      NULL,
//                                      OPEN_EXISTING,
//                                      0,
//                                      NULL);
//
//             Status = NtCreateSection(&SectionHandle,
//                                      SECTION_MAP_READ,
//                                      &ObjectAttributes,
//                                      NULL,
//                                      PAGE_READONLY,
//                                      SEC_COMMIT,
//                                      FileHandle);
//
//             /* HACK: Check if another process was faster
//              * and already created this section. See bug 3626 for details */
//             if (Status == STATUS_OBJECT_NAME_COLLISION)
//             {
//                 /* Close the file then */
//                 NtClose(FileHandle);
//
//                 /* And open the section */
//                 Status = NtOpenSection(&SectionHandle,
//                                        SECTION_MAP_READ,
//                                        &ObjectAttributes);
//             }
//         }
//     }
//     RtlFreeUnicodeString(&UnicodeName);
//
//     if (!NT_SUCCESS(Status))
//     {
//         RtlLeaveCriticalSection(&CodePageListLock);
//         return NULL;
//     }
//
//     SectionMapping = MapViewOfFile(SectionHandle, FILE_MAP_READ, 0, 0, 0);
//     if (SectionMapping == NULL)
//     {
//         NtClose(SectionHandle);
//         RtlLeaveCriticalSection(&CodePageListLock);
//         return NULL;
//     }
//
//     CodePageEntry = HeapAlloc(GetProcessHeap(), 0, sizeof(CODEPAGE_ENTRY));
//     if (CodePageEntry == NULL)
//     {
//         NtClose(SectionHandle);
//         RtlLeaveCriticalSection(&CodePageListLock);
//         return NULL;
//     }
//
//     CodePageEntry->CodePage = CodePage;
//     CodePageEntry->SectionHandle = SectionHandle;
//     CodePageEntry->SectionMapping = SectionMapping;
//
//     RtlInitCodePageTable((PUSHORT)SectionMapping, &CodePageEntry->CodePageTable);
//
//     /* Insert the new entry to list and unlock. Uff. */
//     InsertTailList(&CodePageListHead, &CodePageEntry->Entry);
//     RtlLeaveCriticalSection(&CodePageListLock);
//
//     return CodePageEntry;
// }
//
// /**
//  * @name IntMultiByteToWideCharUTF8
//  *
//  * Internal version of MultiByteToWideChar for UTF8.
//  *
//  * @note We use Win10's behaviour due to security reason.
//  *
//  * @see MultiByteToWideChar
//  */
// static
// INT
// WINAPI
// IntMultiByteToWideCharUTF8(DWORD Flags,
//                            LPCSTR MultiByteString,
//                            INT MultiByteCount,
//                            LPWSTR WideCharString,
//                            INT WideCharCount)
// {
//     LPCSTR MbsEnd, MbsPtrSave;
//     UCHAR Char, TrailLength;
//     WCHAR WideChar;
//     LONG Count;
//     BOOL CharIsValid, StringIsValid = TRUE;
//     const WCHAR InvalidChar = 0xFFFD;
//
//     if (Flags != 0 && Flags != MB_ERR_INVALID_CHARS)
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     /* Does caller query for output buffer size? */
//     if (WideCharCount == 0)
//     {
//         /* validate and count the wide characters */
//         MbsEnd = MultiByteString + MultiByteCount;
//         for (; MultiByteString < MbsEnd; WideCharCount++)
//         {
//             Char = *MultiByteString++;
//             if (Char < 0x80)
//             {
//                 TrailLength = 0;
//                 continue;
//             }
//             if ((Char & 0xC0) == 0x80)
//             {
//                 TrailLength = 0;
//                 StringIsValid = FALSE;
//                 continue;
//             }
//
//             TrailLength = UTF8Length[Char - 0x80];
//             if (TrailLength == 0)
//             {
//                 StringIsValid = FALSE;
//                 continue;
//             }
//
//             CharIsValid = TRUE;
//             MbsPtrSave = MultiByteString;
//             WideChar = Char & UTF8Mask[TrailLength];
//
//             while (TrailLength && MultiByteString < MbsEnd)
//             {
//                 if ((*MultiByteString & 0xC0) != 0x80)
//                 {
//                     CharIsValid = StringIsValid = FALSE;
//                     break;
//                 }
//
//                 WideChar = (WideChar << 6) | (*MultiByteString++ & 0x7f);
//                 TrailLength--;
//             }
//
//             if (!CharIsValid || WideChar < UTF8LBound[UTF8Length[Char - 0x80]])
//             {
//                 MultiByteString = MbsPtrSave;
//             }
//         }
//
//         if (TrailLength)
//         {
//             WideCharCount++;
//             StringIsValid = FALSE;
//         }
//
//         if (Flags == MB_ERR_INVALID_CHARS && !StringIsValid)
//         {
//             SetLastError(ERROR_NO_UNICODE_TRANSLATION);
//             return 0;
//         }
//
//         return WideCharCount;
//     }
//
//     /* convert */
//     MbsEnd = MultiByteString + MultiByteCount;
//     for (Count = 0; Count < WideCharCount && MultiByteString < MbsEnd; Count++)
//     {
//         Char = *MultiByteString++;
//         if (Char < 0x80)
//         {
//             *WideCharString++ = Char;
//             TrailLength = 0;
//             continue;
//         }
//         if ((Char & 0xC0) == 0x80)
//         {
//             *WideCharString++ = InvalidChar;
//             TrailLength = 0;
//             StringIsValid = FALSE;
//             continue;
//         }
//
//         TrailLength = UTF8Length[Char - 0x80];
//         if (TrailLength == 0)
//         {
//             *WideCharString++ = InvalidChar;
//             StringIsValid = FALSE;
//             continue;
//         }
//
//         CharIsValid = TRUE;
//         MbsPtrSave = MultiByteString;
//         WideChar = Char & UTF8Mask[TrailLength];
//
//         while (TrailLength && MultiByteString < MbsEnd)
//         {
//             if ((*MultiByteString & 0xC0) != 0x80)
//             {
//                 CharIsValid = StringIsValid = FALSE;
//                 break;
//             }
//
//             WideChar = (WideChar << 6) | (*MultiByteString++ & 0x7f);
//             TrailLength--;
//         }
//
//         if (CharIsValid && UTF8LBound[UTF8Length[Char - 0x80]] <= WideChar)
//         {
//             *WideCharString++ = WideChar;
//         }
//         else
//         {
//             *WideCharString++ = InvalidChar;
//             MultiByteString = MbsPtrSave;
//             StringIsValid = FALSE;
//         }
//     }
//
//     if (TrailLength && Count < WideCharCount && MultiByteString < MbsEnd)
//     {
//         *WideCharString = InvalidChar;
//         WideCharCount++;
//     }
//
//     if (MultiByteString < MbsEnd)
//     {
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return 0;
//     }
//
//     if (Flags == MB_ERR_INVALID_CHARS && (!StringIsValid || TrailLength))
//     {
//         SetLastError(ERROR_NO_UNICODE_TRANSLATION);
//         return 0;
//     }
//
//     return Count;
// }
//
// /**
//  * @name IntMultiByteToWideCharCP
//  *
//  * Internal version of MultiByteToWideChar for code page tables.
//  *
//  * @see MultiByteToWideChar
//  * @todo Handle MB_PRECOMPOSED, MB_COMPOSITE, MB_USEGLYPHCHARS and
//  *       DBCS codepages.
//  */
//
// static
// INT
// WINAPI
// IntMultiByteToWideCharCP(UINT CodePage,
//                          DWORD Flags,
//                          LPCSTR MultiByteString,
//                          INT MultiByteCount,
//                          LPWSTR WideCharString,
//                          INT WideCharCount)
// {
//     PCODEPAGE_ENTRY CodePageEntry;
//     PCPTABLEINFO CodePageTable;
//     PUSHORT MultiByteTable;
//     LPCSTR TempString;
//     INT TempLength;
//     USHORT WideChar;
//
//     /* Get code page table. */
//     CodePageEntry = IntGetCodePageEntry(CodePage);
//     if (CodePageEntry == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     CodePageTable = &CodePageEntry->CodePageTable;
//
//     /* If MB_USEGLYPHCHARS flag present and glyph table present */
//     if ((Flags & MB_USEGLYPHCHARS) && CodePageTable->MultiByteTable[256])
//     {
//         /* Use glyph table */
//         MultiByteTable = CodePageTable->MultiByteTable + 256 + 1;
//     }
//     else
//     {
//         MultiByteTable = CodePageTable->MultiByteTable;
//     }
//
//     /* Different handling for DBCS code pages. */
//     if (CodePageTable->DBCSCodePage)
//     {
//         UCHAR Char;
//         USHORT DBCSOffset;
//         LPCSTR MbsEnd = MultiByteString + MultiByteCount;
//         INT Count;
//
//         if (Flags & MB_ERR_INVALID_CHARS)
//         {
//             TempString = MultiByteString;
//
//             while (TempString < MbsEnd)
//             {
//                 DBCSOffset = CodePageTable->DBCSOffsets[(UCHAR)*TempString];
//
//                 if (DBCSOffset)
//                 {
//                     /* If lead byte is presented, but behind it there is no symbol */
//                     if (((TempString + 1) == MbsEnd) || (*(TempString + 1) == 0))
//                     {
//                         SetLastError(ERROR_NO_UNICODE_TRANSLATION);
//                         return 0;
//                     }
//
//                     WideChar = CodePageTable->DBCSOffsets[DBCSOffset + *(TempString + 1)];
//
//                     if (WideChar == CodePageTable->UniDefaultChar &&
//                         MAKEWORD(*(TempString + 1), *TempString) != CodePageTable->TransUniDefaultChar)
//                     {
//                         SetLastError(ERROR_NO_UNICODE_TRANSLATION);
//                         return 0;
//                     }
//
//                     TempString++;
//                 }
//                 else
//                 {
//                     WideChar = MultiByteTable[(UCHAR)*TempString];
//
//                     if ((WideChar == CodePageTable->UniDefaultChar &&
//                         *TempString != CodePageTable->TransUniDefaultChar) ||
//                         /* "Private Use" characters */
//                         (WideChar >= 0xE000 && WideChar <= 0xF8FF))
//                     {
//                         SetLastError(ERROR_NO_UNICODE_TRANSLATION);
//                         return 0;
//                     }
//                 }
//
//                 TempString++;
//             }
//         }
//
//         /* Does caller query for output buffer size? */
//         if (WideCharCount == 0)
//         {
//             for (; MultiByteString < MbsEnd; WideCharCount++)
//             {
//                 Char = *MultiByteString++;
//
//                 DBCSOffset = CodePageTable->DBCSOffsets[Char];
//
//                 if (!DBCSOffset)
//                     continue;
//
//                 if (MultiByteString < MbsEnd)
//                     MultiByteString++;
//             }
//
//             return WideCharCount;
//         }
//
//         for (Count = 0; Count < WideCharCount && MultiByteString < MbsEnd; Count++)
//         {
//             Char = *MultiByteString++;
//
//             DBCSOffset = CodePageTable->DBCSOffsets[Char];
//
//             if (!DBCSOffset)
//             {
//                 *WideCharString++ = MultiByteTable[Char];
//                 continue;
//             }
//
//             if (MultiByteString == MbsEnd || *MultiByteString == 0)
//             {
//                 *WideCharString++ = CodePageTable->UniDefaultChar;
//             }
//             else
//             {
//                 *WideCharString++ = CodePageTable->DBCSOffsets[DBCSOffset + (UCHAR)*MultiByteString++];
//             }
//         }
//
//         if (MultiByteString < MbsEnd)
//         {
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//             return 0;
//         }
//
//         return Count;
//     }
//     else /* SBCS code page */
//     {
//         /* Check for invalid characters. */
//         if (Flags & MB_ERR_INVALID_CHARS)
//         {
//             for (TempString = MultiByteString, TempLength = MultiByteCount;
//                  TempLength > 0;
//                  TempString++, TempLength--)
//             {
//                 WideChar = MultiByteTable[(UCHAR)*TempString];
//
//                 if ((WideChar == CodePageTable->UniDefaultChar &&
//                     *TempString != CodePageTable->TransUniDefaultChar) ||
//                     /* "Private Use" characters */
//                     (WideChar >= 0xE000 && WideChar <= 0xF8FF))
//                 {
//                     SetLastError(ERROR_NO_UNICODE_TRANSLATION);
//                     return 0;
//                 }
//             }
//         }
//
//         /* Does caller query for output buffer size? */
//         if (WideCharCount == 0)
//             return MultiByteCount;
//
//         /* Fill the WideCharString buffer with what will fit: Verified on WinXP */
//         for (TempLength = (WideCharCount < MultiByteCount) ? WideCharCount : MultiByteCount;
//             TempLength > 0;
//             MultiByteString++, TempLength--)
//         {
//             *WideCharString++ = MultiByteTable[(UCHAR)*MultiByteString];
//         }
//
//         /* Adjust buffer size. Wine trick ;-) */
//         if (WideCharCount < MultiByteCount)
//         {
//             MultiByteCount = WideCharCount;
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//             return 0;
//         }
//         return MultiByteCount;
//     }
// }
//
// /**
//  * @name IntMultiByteToWideCharSYMBOL
//  *
//  * Internal version of MultiByteToWideChar for SYMBOL.
//  *
//  * @see MultiByteToWideChar
//  */
//
// static
// INT
// WINAPI
// IntMultiByteToWideCharSYMBOL(DWORD Flags,
//                              LPCSTR MultiByteString,
//                              INT MultiByteCount,
//                              LPWSTR WideCharString,
//                              INT WideCharCount)
// {
//     LONG Count;
//     UCHAR Char;
//     INT WideCharMaxLen;
//
//
//     if (Flags != 0)
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     if (WideCharCount == 0)
//     {
//         return MultiByteCount;
//     }
//
//     WideCharMaxLen = WideCharCount > MultiByteCount ? MultiByteCount : WideCharCount;
//
//     for (Count = 0; Count < WideCharMaxLen; Count++)
//     {
//         Char = MultiByteString[Count];
//         if ( Char < 0x20 )
//         {
//             WideCharString[Count] = Char;
//         }
//         else
//         {
//             WideCharString[Count] = Char + 0xf000;
//         }
//     }
//     if (MultiByteCount > WideCharMaxLen)
//     {
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return 0;
//     }
//
//     return WideCharMaxLen;
// }
//
// /**
//  * @name IntWideCharToMultiByteSYMBOL
//  *
//  * Internal version of WideCharToMultiByte for SYMBOL.
//  *
//  * @see WideCharToMultiByte
//  */
//
// static INT
// WINAPI
// IntWideCharToMultiByteSYMBOL(DWORD Flags,
//                              LPCWSTR WideCharString,
//                              INT WideCharCount,
//                              LPSTR MultiByteString,
//                              INT MultiByteCount)
// {
//     LONG Count;
//     INT MaxLen;
//     WCHAR Char;
//
//     if (Flags!=0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//
//     if (MultiByteCount == 0)
//     {
//         return WideCharCount;
//     }
//
//     MaxLen = MultiByteCount > WideCharCount ? WideCharCount : MultiByteCount;
//     for (Count = 0; Count < MaxLen; Count++)
//     {
//         Char = WideCharString[Count];
//         if (Char < 0x20)
//         {
//             MultiByteString[Count] = (CHAR)Char;
//         }
//         else
//         {
//             if ((Char >= 0xf020) && (Char < 0xf100))
//             {
//                 MultiByteString[Count] = Char - 0xf000;
//             }
//             else
//             {
//                 SetLastError(ERROR_NO_UNICODE_TRANSLATION);
//                 return 0;
//             }
//         }
//     }
//
//     if (WideCharCount > MaxLen)
//     {
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return 0;
//     }
//     return MaxLen;
// }
//
// /**
//  * @name IntWideCharToMultiByteUTF8
//  *
//  * Internal version of WideCharToMultiByte for UTF8.
//  *
//  * @see WideCharToMultiByte
//  */
//
// static INT
// WINAPI
// IntWideCharToMultiByteUTF8(UINT CodePage,
//                            DWORD Flags,
//                            LPCWSTR WideCharString,
//                            INT WideCharCount,
//                            LPSTR MultiByteString,
//                            INT MultiByteCount,
//                            LPCSTR DefaultChar,
//                            LPBOOL UsedDefaultChar)
// {
//     INT TempLength;
//     DWORD Char;
//
//     if (Flags)
//     {
//         SetLastError(ERROR_INVALID_FLAGS);
//         return 0;
//     }
//
//     /* Does caller query for output buffer size? */
//     if (MultiByteCount == 0)
//     {
//         for (TempLength = 0; WideCharCount;
//             WideCharCount--, WideCharString++)
//         {
//             TempLength++;
//             if (*WideCharString >= 0x80)
//             {
//                 TempLength++;
//                 if (*WideCharString >= 0x800)
//                 {
//                     TempLength++;
//                     if (*WideCharString >= 0xd800 && *WideCharString < 0xdc00 &&
//                         WideCharCount >= 1 &&
//                         WideCharString[1] >= 0xdc00 && WideCharString[1] <= 0xe000)
//                     {
//                         WideCharCount--;
//                         WideCharString++;
//                         TempLength++;
//                     }
//                 }
//             }
//         }
//         return TempLength;
//     }
//
//     for (TempLength = MultiByteCount; WideCharCount; WideCharCount--, WideCharString++)
//     {
//         Char = *WideCharString;
//         if (Char < 0x80)
//         {
//             if (!TempLength)
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 break;
//             }
//             TempLength--;
//             *MultiByteString++ = (CHAR)Char;
//             continue;
//         }
//
//         if (Char < 0x800)  /* 0x80-0x7ff: 2 bytes */
//         {
//             if (TempLength < 2)
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 break;
//             }
//             MultiByteString[1] = 0x80 | (Char & 0x3f); Char >>= 6;
//             MultiByteString[0] = 0xc0 | Char;
//             MultiByteString += 2;
//             TempLength -= 2;
//             continue;
//         }
//
//         /* surrogate pair 0x10000-0x10ffff: 4 bytes */
//         if (Char >= 0xd800 && Char < 0xdc00 &&
//             WideCharCount >= 1 &&
//             WideCharString[1] >= 0xdc00 && WideCharString[1] < 0xe000)
//         {
//             WideCharCount--;
//             WideCharString++;
//
//             if (TempLength < 4)
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 break;
//             }
//
//             Char = (Char - 0xd800) << 10;
//             Char |= *WideCharString - 0xdc00;
//             ASSERT(Char <= 0xfffff);
//             Char += 0x10000;
//             ASSERT(Char <= 0x10ffff);
//
//             MultiByteString[3] = 0x80 | (Char & 0x3f); Char >>= 6;
//             MultiByteString[2] = 0x80 | (Char & 0x3f); Char >>= 6;
//             MultiByteString[1] = 0x80 | (Char & 0x3f); Char >>= 6;
//             MultiByteString[0] = 0xf0 | Char;
//             MultiByteString += 4;
//             TempLength -= 4;
//             continue;
//         }
//
//         /* 0x800-0xffff: 3 bytes */
//         if (TempLength < 3)
//         {
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//             break;
//         }
//         MultiByteString[2] = 0x80 | (Char & 0x3f); Char >>= 6;
//         MultiByteString[1] = 0x80 | (Char & 0x3f); Char >>= 6;
//         MultiByteString[0] = 0xe0 | Char;
//         MultiByteString += 3;
//         TempLength -= 3;
//     }
//
//     return MultiByteCount - TempLength;
// }
//
// /**
//  * @name IsValidSBCSMapping
//  *
//  * Checks if ch (single-byte character) is a valid mapping for wch
//  *
//  * @see IntWideCharToMultiByteCP
//  */
// static
// inline
// BOOL
// IntIsValidSBCSMapping(PCPTABLEINFO CodePageTable, DWORD Flags, WCHAR wch, UCHAR ch)
// {
//     /* If the WC_NO_BEST_FIT_CHARS flag has been specified, the characters need to match exactly. */
//     if (Flags & WC_NO_BEST_FIT_CHARS)
//         return (CodePageTable->MultiByteTable[ch] == wch);
//
//     /* By default, all characters except TransDefaultChar apply as a valid mapping
//        for ch (so also "nearest" characters) */
//     if (ch != CodePageTable->TransDefaultChar)
//         return TRUE;
//
//     /* The only possible left valid mapping is the default character itself */
//     return (wch == CodePageTable->TransUniDefaultChar);
// }
//
// /**
//  * @name IsValidDBCSMapping
//  *
//  * Checks if ch (double-byte character) is a valid mapping for wch
//  *
//  * @see IntWideCharToMultiByteCP
//  */
// static inline BOOL
// IntIsValidDBCSMapping(PCPTABLEINFO CodePageTable, DWORD Flags, WCHAR wch, USHORT ch)
// {
//     /* If ch is the default character, but the wch is not, it can't be a valid mapping */
//     if (ch == CodePageTable->TransDefaultChar && wch != CodePageTable->TransUniDefaultChar)
//         return FALSE;
//
//     /* If the WC_NO_BEST_FIT_CHARS flag has been specified, the characters need to match exactly. */
//     if (Flags & WC_NO_BEST_FIT_CHARS)
//     {
//         if(ch & 0xff00)
//         {
//             USHORT uOffset = CodePageTable->DBCSOffsets[ch >> 8];
//             /* if (!uOffset) return (CodePageTable->MultiByteTable[ch] == wch); */
//             return (CodePageTable->DBCSOffsets[uOffset + (ch & 0xff)] == wch);
//         }
//
//         return (CodePageTable->MultiByteTable[ch] == wch);
//     }
//
//     /* If we're still here, we have a valid mapping */
//     return TRUE;
// }
//
// /**
//  * @name IntWideCharToMultiByteCP
//  *
//  * Internal version of WideCharToMultiByte for code page tables.
//  *
//  * @see WideCharToMultiByte
//  * @todo Handle WC_COMPOSITECHECK
//  */
// static
// INT
// WINAPI
// IntWideCharToMultiByteCP(UINT CodePage,
//                          DWORD Flags,
//                          LPCWSTR WideCharString,
//                          INT WideCharCount,
//                          LPSTR MultiByteString,
//                          INT MultiByteCount,
//                          LPCSTR DefaultChar,
//                          LPBOOL UsedDefaultChar)
// {
//     PCODEPAGE_ENTRY CodePageEntry;
//     PCPTABLEINFO CodePageTable;
//     INT TempLength;
//
//     /* Get code page table. */
//     CodePageEntry = IntGetCodePageEntry(CodePage);
//     if (CodePageEntry == NULL)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     CodePageTable = &CodePageEntry->CodePageTable;
//
//
//     /* Different handling for DBCS code pages. */
//     if (CodePageTable->DBCSCodePage)
//     {
//         /* If Flags, DefaultChar or UsedDefaultChar were given, we have to do some more work */
//         if (Flags || DefaultChar || UsedDefaultChar)
//         {
//             BOOL TempUsedDefaultChar;
//             USHORT DefChar;
//
//             /* If UsedDefaultChar is not set, set it to a temporary value, so we don't have
//                to check on every character */
//             if (!UsedDefaultChar)
//                 UsedDefaultChar = &TempUsedDefaultChar;
//
//             *UsedDefaultChar = FALSE;
//
//             /* Use the CodePage's TransDefaultChar if none was given. Don't modify the DefaultChar pointer here. */
//             if (DefaultChar)
//                 DefChar = DefaultChar[1] ? ((DefaultChar[0] << 8) | DefaultChar[1]) : DefaultChar[0];
//             else
//                 DefChar = CodePageTable->TransDefaultChar;
//
//             /* Does caller query for output buffer size? */
//             if (!MultiByteCount)
//             {
//                 for (TempLength = 0; WideCharCount; WideCharCount--, WideCharString++, TempLength++)
//                 {
//                     USHORT uChar;
//
//                     if ((Flags & WC_COMPOSITECHECK) && WideCharCount > 1)
//                     {
//                         /* FIXME: Handle WC_COMPOSITECHECK */
//                         DPRINT("WC_COMPOSITECHECK flag UNIMPLEMENTED\n");
//                     }
//
//                     uChar = ((PUSHORT) CodePageTable->WideCharTable)[*WideCharString];
//
//                     /* Verify if the mapping is valid for handling DefaultChar and UsedDefaultChar */
//                     if (!IntIsValidDBCSMapping(CodePageTable, Flags, *WideCharString, uChar))
//                     {
//                         uChar = DefChar;
//                         *UsedDefaultChar = TRUE;
//                     }
//
//                     /* Increment TempLength again if this is a double-byte character */
//                     if (uChar & 0xff00)
//                         TempLength++;
//                 }
//
//                 return TempLength;
//             }
//
//             /* Convert the WideCharString to the MultiByteString and verify if the mapping is valid */
//             for (TempLength = MultiByteCount;
//                  WideCharCount && TempLength;
//                  TempLength--, WideCharString++, WideCharCount--)
//             {
//                 USHORT uChar;
//
//                 if ((Flags & WC_COMPOSITECHECK) && WideCharCount > 1)
//                 {
//                     /* FIXME: Handle WC_COMPOSITECHECK */
//                     DPRINT("WC_COMPOSITECHECK flag UNIMPLEMENTED\n");
//                 }
//
//                 uChar = ((PUSHORT)CodePageTable->WideCharTable)[*WideCharString];
//
//                 /* Verify if the mapping is valid for handling DefaultChar and UsedDefaultChar */
//                 if (!IntIsValidDBCSMapping(CodePageTable, Flags, *WideCharString, uChar))
//                 {
//                     uChar = DefChar;
//                     *UsedDefaultChar = TRUE;
//                 }
//
//                 /* Handle double-byte characters */
//                 if (uChar & 0xff00)
//                 {
//                     /* Don't output a partial character */
//                     if (TempLength == 1)
//                         break;
//
//                     TempLength--;
//                     *MultiByteString++ = uChar >> 8;
//                 }
//
//                 *MultiByteString++ = (char)uChar;
//             }
//
//             /* WideCharCount should be 0 if all characters were converted */
//             if (WideCharCount)
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return 0;
//             }
//
//             return MultiByteCount - TempLength;
//         }
//
//         /* Does caller query for output buffer size? */
//         if (!MultiByteCount)
//         {
//             for (TempLength = 0; WideCharCount; WideCharCount--, WideCharString++, TempLength++)
//             {
//                 /* Increment TempLength again if this is a double-byte character */
//                 if (((PWCHAR)CodePageTable->WideCharTable)[*WideCharString] & 0xff00)
//                     TempLength++;
//             }
//
//             return TempLength;
//         }
//
//         /* Convert the WideCharString to the MultiByteString */
//         for (TempLength = MultiByteCount;
//              WideCharCount && TempLength;
//              TempLength--, WideCharString++, WideCharCount--)
//         {
//             USHORT uChar = ((PUSHORT) CodePageTable->WideCharTable)[*WideCharString];
//
//             /* Is this a double-byte character? */
//             if (uChar & 0xff00)
//             {
//                 /* Don't output a partial character */
//                 if (TempLength == 1)
//                     break;
//
//                 TempLength--;
//                 *MultiByteString++ = uChar >> 8;
//             }
//
//             *MultiByteString++ = (char)uChar;
//         }
//
//         /* WideCharCount should be 0 if all characters were converted */
//         if (WideCharCount)
//         {
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//             return 0;
//         }
//
//         return MultiByteCount - TempLength;
//     }
//     else /* SBCS code page */
//     {
//         INT nReturn;
//
//         /* If Flags, DefaultChar or UsedDefaultChar were given, we have to do some more work */
//         if (Flags || DefaultChar || UsedDefaultChar)
//         {
//             BOOL TempUsedDefaultChar;
//             CHAR DefChar;
//
//             /* If UsedDefaultChar is not set, set it to a temporary value, so we don't have
//                to check on every character */
//             if (!UsedDefaultChar)
//                 UsedDefaultChar = &TempUsedDefaultChar;
//
//             *UsedDefaultChar = FALSE;
//
//             /* Does caller query for output buffer size? */
//             if (!MultiByteCount)
//             {
//                 /* Loop through the whole WideCharString and check if we can get a valid mapping for each character */
//                 for (TempLength = 0; WideCharCount; TempLength++, WideCharString++, WideCharCount--)
//                 {
//                     if ((Flags & WC_COMPOSITECHECK) && WideCharCount > 1)
//                     {
//                         /* FIXME: Handle WC_COMPOSITECHECK */
//                         DPRINT("WC_COMPOSITECHECK flag UNIMPLEMENTED\n");
//                     }
//
//                     if (!*UsedDefaultChar)
//                         *UsedDefaultChar = !IntIsValidSBCSMapping(CodePageTable,
//                                                                   Flags,
//                                                                   *WideCharString,
//                                                                   ((PCHAR)CodePageTable->WideCharTable)[*WideCharString]);
//                 }
//
//                 return TempLength;
//             }
//
//             /* Use the CodePage's TransDefaultChar if none was given. Don't modify the DefaultChar pointer here. */
//             if (DefaultChar)
//                 DefChar = *DefaultChar;
//             else
//                 DefChar = (CHAR)CodePageTable->TransDefaultChar;
//
//             /* Convert the WideCharString to the MultiByteString and verify if the mapping is valid */
//             for (TempLength = MultiByteCount;
//                  WideCharCount && TempLength;
//                  MultiByteString++, TempLength--, WideCharString++, WideCharCount--)
//             {
//                 if ((Flags & WC_COMPOSITECHECK) && WideCharCount > 1)
//                 {
//                     /* FIXME: Handle WC_COMPOSITECHECK */
//                     DPRINT("WC_COMPOSITECHECK flag UNIMPLEMENTED\n");
//                 }
//
//                 *MultiByteString = ((PCHAR)CodePageTable->WideCharTable)[*WideCharString];
//
//                 if (!IntIsValidSBCSMapping(CodePageTable, Flags, *WideCharString, *MultiByteString))
//                 {
//                     *MultiByteString = DefChar;
//                     *UsedDefaultChar = TRUE;
//                 }
//             }
//
//             /* WideCharCount should be 0 if all characters were converted */
//             if (WideCharCount)
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return 0;
//             }
//
//             return MultiByteCount - TempLength;
//         }
//
//         /* Does caller query for output buffer size? */
//         if (!MultiByteCount)
//             return WideCharCount;
//
//         /* Is the buffer large enough? */
//         if (MultiByteCount < WideCharCount)
//         {
//             /* Convert the string up to MultiByteCount and return 0 */
//             WideCharCount = MultiByteCount;
//             SetLastError(ERROR_INSUFFICIENT_BUFFER);
//             nReturn = 0;
//         }
//         else
//         {
//             /* Otherwise WideCharCount will be the number of converted characters */
//             nReturn = WideCharCount;
//         }
//
//         /* Convert the WideCharString to the MultiByteString */
//         for (TempLength = WideCharCount; --TempLength >= 0; WideCharString++, MultiByteString++)
//         {
//             *MultiByteString = ((PCHAR)CodePageTable->WideCharTable)[*WideCharString];
//         }
//
//         return nReturn;
//     }
// }
//
// /**
//  * @name IntIsLeadByte
//  *
//  * Internal function to detect if byte is lead byte in specific character
//  * table.
//  */
//
// static BOOL
// WINAPI
// IntIsLeadByte(PCPTABLEINFO TableInfo, BYTE Byte)
// {
//     UINT i;
//
//     if (TableInfo->MaximumCharacterSize == 2)
//     {
//         for (i = 0; i < MAXIMUM_LEADBYTES && TableInfo->LeadByte[i]; i += 2)
//         {
//             if (Byte >= TableInfo->LeadByte[i] && Byte <= TableInfo->LeadByte[i+1])
//                 return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /**
//  * @name GetNlsSectionName
//  *
//  * Construct a name of NLS section.
//  *
//  * @param CodePage
//  *        Code page number.
//  * @param Base
//  *        Integer base used for converting to string. Usually set to 10.
//  * @param Unknown
//  *        As the name suggests the meaning of this parameter is unknown.
//  *        The native version of Kernel32 passes it as the third parameter
//  *        to NlsConvertIntegerToString function, which is used for the
//  *        actual conversion of the code page number.
//  * @param BaseName
//  *        Base name of the section. (ex. "\\Nls\\NlsSectionCP")
//  * @param Result
//  *        Buffer that will hold the constructed name.
//  * @param ResultSize
//  *        Size of the buffer for the result.
//  *
//  * @return TRUE if the buffer was large enough and was filled with
//  *         the requested information, FALSE otherwise.
//  *
//  * @implemented
//  */
//
// BOOL
// WINAPI
// GetNlsSectionName(UINT CodePage,
//                   UINT Base,
//                   ULONG Unknown,
//                   LPSTR BaseName,
//                   LPSTR Result,
//                   ULONG ResultSize)
// {
//     CHAR Integer[11];
//
//     if (!NT_SUCCESS(RtlIntegerToChar(CodePage, Base, sizeof(Integer), Integer)))
//         return FALSE;
//
//     /*
//      * If the name including the terminating NULL character doesn't
//      * fit in the output buffer then fail.
//      */
//     if (strlen(Integer) + strlen(BaseName) >= ResultSize)
//         return FALSE;
//
//     lstrcpyA(Result, BaseName);
//     lstrcatA(Result, Integer);
//
//     return TRUE;
// }
//
// /**
//  * @name GetCPFileNameFromRegistry
//  *
//  * Get file name of code page definition file.
//  *
//  * @param CodePage
//  *        Code page number to get file name of.
//  * @param FileName
//  *        Buffer that is filled with file name of successful return. Can
//  *        be set to NULL.
//  * @param FileNameSize
//  *        Size of the buffer to hold file name in WCHARs.
//  *
//  * @return TRUE if the file name was retrieved, FALSE otherwise.
//  *
//  * @implemented
//  */
//
// BOOL
// WINAPI
// GetCPFileNameFromRegistry(UINT CodePage, LPWSTR FileName, ULONG FileNameSize)
// {
//     WCHAR ValueNameBuffer[11];
//     UNICODE_STRING KeyName, ValueName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     NTSTATUS Status;
//     HANDLE KeyHandle;
//     PKEY_VALUE_PARTIAL_INFORMATION Kvpi;
//     DWORD KvpiSize;
//     BOOL bRetValue;
//
//     bRetValue = FALSE;
//
//     /* Convert the codepage number to string. */
//     ValueName.Buffer = ValueNameBuffer;
//     ValueName.MaximumLength = sizeof(ValueNameBuffer);
//
//     if (!NT_SUCCESS(RtlIntegerToUnicodeString(CodePage, 10, &ValueName)))
//         return bRetValue;
//
//     /* Open the registry key containing file name mappings. */
//     RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\System\\"
//                          L"CurrentControlSet\\Control\\Nls\\CodePage");
//     InitializeObjectAttributes!(&ObjectAttributes, &KeyName, OBJ_CASE_INSENSITIVE,
//                                NULL, NULL);
//     Status = NtOpenKey(&KeyHandle, KEY_READ, &ObjectAttributes);
//     if (!NT_SUCCESS(Status))
//     {
//         return bRetValue;
//     }
//
//     /* Allocate buffer that will be used to query the value data. */
//     KvpiSize = sizeof(KEY_VALUE_PARTIAL_INFORMATION) + (MAX_PATH * sizeof(WCHAR));
//     Kvpi = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, KvpiSize);
//     if (Kvpi == NULL)
//     {
//         NtClose(KeyHandle);
//         return bRetValue;
//     }
//
//     /* Query the file name for our code page. */
//     Status = NtQueryValueKey(KeyHandle, &ValueName, KeyValuePartialInformation,
//                              Kvpi, KvpiSize, &KvpiSize);
//
//     NtClose(KeyHandle);
//
//     /* Check if we succeded and the value is non-empty string. */
//     if (NT_SUCCESS(Status) && Kvpi->Type == REG_SZ &&
//         Kvpi->DataLength > sizeof(WCHAR))
//     {
//         bRetValue = TRUE;
//         if (FileName != NULL)
//         {
//             lstrcpynW(FileName, (WCHAR*)Kvpi->Data,
//                       min(Kvpi->DataLength / sizeof(WCHAR), FileNameSize));
//         }
//     }
//
//     /* free temporary buffer */
//     HeapFree(GetProcessHeap(),0,Kvpi);
//     return bRetValue;
// }
//
// /**
//  * @name IsValidCodePage
//  *
//  * Detect if specified code page is valid and present in the system.
//  *
//  * @param CodePage
//  *        Code page number to query.
//  *
//  * @return TRUE if code page is present.
//  */
//
// BOOL
// WINAPI
// IsValidCodePage(UINT CodePage)
// {
//     if (CodePage == 0) return FALSE;
//     if (CodePage == CP_UTF8 || CodePage == CP_UTF7)
//         return TRUE;
//     if (IntGetLoadedCodePageEntry(CodePage))
//         return TRUE;
//     return GetCPFileNameFromRegistry(CodePage, NULL, 0);
// }
//
// static inline BOOL utf7_write_w(WCHAR *dst, int dstlen, int *index, WCHAR character)
// {
//     if (dstlen > 0)
//     {
//         if (*index >= dstlen)
//             return FALSE;
//
//         dst[*index] = character;
//     }
//
//     (*index)++;
//
//     return TRUE;
// }
//
// static INT Utf7ToWideChar(const char *src, int srclen, WCHAR *dst, int dstlen)
// {
//     static const signed char base64_decoding_table[] =
//     {
//         -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, /* 0x00-0x0F */
//         -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, /* 0x10-0x1F */
//         -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1, -1, 63, /* 0x20-0x2F */
//         52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, /* 0x30-0x3F */
//         -1,  0,  1,  2,  3,  4,  5,  6,  7,  8,  9, 10, 11, 12, 13, 14, /* 0x40-0x4F */
//         15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1, -1, -1, /* 0x50-0x5F */
//         -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, /* 0x60-0x6F */
//         41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1  /* 0x70-0x7F */
//     };
//
//     const char *source_end = src + srclen;
//     int dest_index = 0;
//
//     DWORD byte_pair = 0;
//     short offset = 0;
//
//     while (src < source_end)
//     {
//         if (*src == '+')
//         {
//             src++;
//             if (src >= source_end)
//                 break;
//
//             if (*src == '-')
//             {
//                 /* just a plus sign escaped as +- */
//                 if (!utf7_write_w(dst, dstlen, &dest_index, '+'))
//                 {
//                     SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                     return 0;
//                 }
//                 src++;
//                 continue;
//             }
//
//             do
//             {
//                 signed char sextet = *src;
//                 if (sextet == '-')
//                 {
//                     /* skip over the dash and end base64 decoding
//                      * the current, unfinished byte pair is discarded */
//                     src++;
//                     offset = 0;
//                     break;
//                 }
//                 if (sextet < 0)
//                 {
//                     /* the next character of src is < 0 and therefore not part of a base64 sequence
//                      * the current, unfinished byte pair is NOT discarded in this case
//                      * this is probably a bug in Windows */
//                     break;
//                 }
//
//                 sextet = base64_decoding_table[sextet];
//                 if (sextet == -1)
//                 {
//                     /* -1 means that the next character of src is not part of a base64 sequence
//                      * in other words, all sextets in this base64 sequence have been processed
//                      * the current, unfinished byte pair is discarded */
//                     offset = 0;
//                     break;
//                 }
//
//                 byte_pair = (byte_pair << 6) | sextet;
//                 offset += 6;
//
//                 if (offset >= 16)
//                 {
//                     /* this byte pair is done */
//                     if (!utf7_write_w(dst, dstlen, &dest_index, (byte_pair >> (offset - 16)) & 0xFFFF))
//                     {
//                         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                         return 0;
//                     }
//                     offset -= 16;
//                 }
//
//                 src++;
//             }
//             while (src < source_end);
//         }
//         else
//         {
//             /* we have to convert to unsigned char in case *src < 0 */
//             if (!utf7_write_w(dst, dstlen, &dest_index, (unsigned char)*src))
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return 0;
//             }
//             src++;
//         }
//     }
//
//     return dest_index;
// }
//
// /**
//  * @name MultiByteToWideChar
//  *
//  * Convert a multi-byte string to wide-charater equivalent.
//  *
//  * @param CodePage
//  *        Code page to be used to perform the conversion. It can be also
//  *        one of the special values (CP_ACP for ANSI code page, CP_MACCP
//  *        for Macintosh code page, CP_OEMCP for OEM code page, CP_THREAD_ACP
//  *        for thread active code page, CP_UTF7 or CP_UTF8).
//  * @param Flags
//  *        Additional conversion flags (MB_PRECOMPOSED, MB_COMPOSITE,
//  *        MB_ERR_INVALID_CHARS, MB_USEGLYPHCHARS).
//  * @param MultiByteString
//  *        Input buffer.
//  * @param MultiByteCount
//  *        Size of MultiByteString, or -1 if MultiByteString is NULL
//  *        terminated.
//  * @param WideCharString
//  *        Output buffer.
//  * @param WideCharCount
//  *        Size in WCHARs of WideCharString, or 0 if the caller just wants
//  *        to know how large WideCharString should be for a successful
//  *        conversion.
//  *
//  * @return Zero on error, otherwise the number of WCHARs written
//  *         in the WideCharString buffer.
//  *
//  * @implemented
//  */
// INT
// WINAPI
// MultiByteToWideChar(UINT CodePage,
//                     DWORD Flags,
//                     LPCSTR MultiByteString,
//                     INT MultiByteCount,
//                     LPWSTR WideCharString,
//                     INT WideCharCount)
// {
//     /* Check the parameters. */
//     if (MultiByteString == NULL ||
//         MultiByteCount == 0 || WideCharCount < 0 ||
//         (WideCharCount && (WideCharString == NULL ||
//         (PVOID)MultiByteString == (PVOID)WideCharString)))
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     /* Determine the input string length. */
//     if (MultiByteCount < 0)
//     {
//         MultiByteCount = lstrlenA(MultiByteString) + 1;
//     }
//
//     switch (CodePage)
//     {
//         case CP_UTF8:
//             return IntMultiByteToWideCharUTF8(Flags,
//                                               MultiByteString,
//                                               MultiByteCount,
//                                               WideCharString,
//                                               WideCharCount);
//
//         case CP_UTF7:
//             if (Flags)
//             {
//                 SetLastError(ERROR_INVALID_FLAGS);
//                 return 0;
//             }
//             return Utf7ToWideChar(MultiByteString, MultiByteCount,
//                                   WideCharString, WideCharCount);
//
//         case CP_SYMBOL:
//             return IntMultiByteToWideCharSYMBOL(Flags,
//                                                 MultiByteString,
//                                                 MultiByteCount,
//                                                 WideCharString,
//                                                 WideCharCount);
//         default:
//             return IntMultiByteToWideCharCP(CodePage,
//                                             Flags,
//                                             MultiByteString,
//                                             MultiByteCount,
//                                             WideCharString,
//                                             WideCharCount);
//     }
// }
//
// static inline BOOL utf7_can_directly_encode(WCHAR codepoint)
// {
//     static const BOOL directly_encodable_table[] =
//     {
//         1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 0, /* 0x00 - 0x0F */
//         0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, /* 0x10 - 0x1F */
//         1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 1, 1, /* 0x20 - 0x2F */
//         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, /* 0x30 - 0x3F */
//         0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, /* 0x40 - 0x4F */
//         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, /* 0x50 - 0x5F */
//         0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, /* 0x60 - 0x6F */
//         1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1                 /* 0x70 - 0x7A */
//     };
//
//     return codepoint <= 0x7A ? directly_encodable_table[codepoint] : FALSE;
// }
//
// static inline BOOL utf7_write_c(char *dst, int dstlen, int *index, char character)
// {
//     if (dstlen > 0)
//     {
//         if (*index >= dstlen)
//             return FALSE;
//
//         dst[*index] = character;
//     }
//
//     (*index)++;
//
//     return TRUE;
// }
//
// static INT WideCharToUtf7(const WCHAR *src, int srclen, char *dst, int dstlen)
// {
//     static const char base64_encoding_table[] =
//         "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
//
//     const WCHAR *source_end = src + srclen;
//     int dest_index = 0;
//
//     while (src < source_end)
//     {
//         if (*src == '+')
//         {
//             if (!utf7_write_c(dst, dstlen, &dest_index, '+'))
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return 0;
//             }
//             if (!utf7_write_c(dst, dstlen, &dest_index, '-'))
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return 0;
//             }
//             src++;
//         }
//         else if (utf7_can_directly_encode(*src))
//         {
//             if (!utf7_write_c(dst, dstlen, &dest_index, *src))
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return 0;
//             }
//             src++;
//         }
//         else
//         {
//             unsigned int offset = 0;
//             DWORD byte_pair = 0;
//
//             if (!utf7_write_c(dst, dstlen, &dest_index, '+'))
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return 0;
//             }
//
//             while (src < source_end && !utf7_can_directly_encode(*src))
//             {
//                 byte_pair = (byte_pair << 16) | *src;
//                 offset += 16;
//                 while (offset >= 6)
//                 {
//                     if (!utf7_write_c(dst, dstlen, &dest_index, base64_encoding_table[(byte_pair >> (offset - 6)) & 0x3F]))
//                     {
//                         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                         return 0;
//                     }
//                     offset -= 6;
//                 }
//                 src++;
//             }
//
//             if (offset)
//             {
//                 /* Windows won't create a padded base64 character if there's no room for the - sign
//                  * as well ; this is probably a bug in Windows */
//                 if (dstlen > 0 && dest_index + 1 >= dstlen)
//                 {
//                     SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                     return 0;
//                 }
//
//                 byte_pair <<= (6 - offset);
//                 if (!utf7_write_c(dst, dstlen, &dest_index, base64_encoding_table[byte_pair & 0x3F]))
//                 {
//                     SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                     return 0;
//                 }
//             }
//
//             /* Windows always explicitly terminates the base64 sequence
//                even though RFC 2152 (page 3, rule 2) does not require this */
//             if (!utf7_write_c(dst, dstlen, &dest_index, '-'))
//             {
//                 SetLastError(ERROR_INSUFFICIENT_BUFFER);
//                 return 0;
//             }
//         }
//     }
//
//     return dest_index;
// }
//
// /*
//  * A function similar to LoadStringW, but adapted for usage by GetCPInfoExW
//  * and GetGeoInfoW. It uses the current user localization, otherwise falls back
//  * to English (US). Contrary to LoadStringW which always saves the loaded string
//  * into the user-given buffer, truncating the string if needed, this function
//  * returns instead an ERROR_INSUFFICIENT_BUFFER error code if the user buffer
//  * is not large enough.
//  */
// UINT
// GetLocalisedText(
//     IN UINT uID,
//     IN LPWSTR lpszDest,
//     IN UINT cchDest,
//     IN LANGID lang)
// {
//     HRSRC hrsrc;
//     HGLOBAL hmem;
//     LCID lcid;
//     LANGID langId;
//     const WCHAR *p;
//     UINT i;
//
//     /* See HACK in winnls/lang/xx-XX.rc files */
//     if (uID == 37)
//         uID = uID * 100;
//
//     lcid = ConvertDefaultLocale(lang);
//
//     langId = LANGIDFROMLCID(lcid);
//
//     if (PRIMARYLANGID(langId) == LANG_NEUTRAL)
//         langId = MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US);
//
//     hrsrc = FindResourceExW(hCurrentModule,
//                             (LPWSTR)RT_STRING,
//                             MAKEINTRESOURCEW((uID >> 4) + 1),
//                             langId);
//
//     /* English fallback */
//     if (!hrsrc)
//     {
//         hrsrc = FindResourceExW(hCurrentModule,
//                                 (LPWSTR)RT_STRING,
//                                 MAKEINTRESOURCEW((uID >> 4) + 1),
//                                 MAKELANGID(LANG_ENGLISH, SUBLANG_ENGLISH_US));
//     }
//
//     if (!hrsrc)
//         goto NotFound;
//
//     hmem = LoadResource(hCurrentModule, hrsrc);
//     if (!hmem)
//         goto NotFound;
//
//     p = LockResource(hmem);
//
//     for (i = 0; i < (uID & 0x0F); i++)
//         p += *p + 1;
//
//     /* Needed for GetGeoInfo(): return the needed string size including the NULL terminator */
//     if (cchDest == 0)
//         return *p + 1;
//     /* Needed for GetGeoInfo(): bail out if the user buffer is not large enough */
//     if (*p + 1 > cchDest)
//     {
//         SetLastError(ERROR_INSUFFICIENT_BUFFER);
//         return 0;
//     }
//
//     i = *p;
//     if (i > 0)
//     {
//         memcpy(lpszDest, p + 1, i * sizeof(WCHAR));
//         lpszDest[i] = L'\0';
//         return i;
//     }
//
// NotFound:
//     DPRINT1("Resource not found: uID = %lu\n", uID);
//     SetLastError(ERROR_INVALID_PARAMETER);
//     return 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetCPInfo(UINT CodePage,
//           LPCPINFO CodePageInfo)
// {
//     PCODEPAGE_ENTRY CodePageEntry;
//
//     if (!CodePageInfo)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return FALSE;
//     }
//
//     CodePageEntry = IntGetCodePageEntry(CodePage);
//     if (CodePageEntry == NULL)
//     {
//         switch(CodePage)
//         {
//             case CP_UTF7:
//             case CP_UTF8:
//                 RtlZeroMemory(CodePageInfo, sizeof(*CodePageInfo));
//                 CodePageInfo->DefaultChar[0] = 0x3f;
//                 CodePageInfo->DefaultChar[1] = 0;
//                 CodePageInfo->LeadByte[0] = CodePageInfo->LeadByte[1] = 0;
//                 CodePageInfo->MaxCharSize = (CodePage == CP_UTF7) ? 5 : 4;
//                 return TRUE;
//         }
//
//         DPRINT1("Invalid CP!: %lx\n", CodePage);
//         SetLastError( ERROR_INVALID_PARAMETER );
//         return FALSE;
//     }
//
//     RtlZeroMemory(CodePageInfo, sizeof(*CodePageInfo));
//     if (CodePageEntry->CodePageTable.DefaultChar & 0xff00)
//     {
//         CodePageInfo->DefaultChar[0] = (CodePageEntry->CodePageTable.DefaultChar & 0xff00) >> 8;
//         CodePageInfo->DefaultChar[1] = CodePageEntry->CodePageTable.DefaultChar & 0x00ff;
//     }
//     else
//     {
//         CodePageInfo->DefaultChar[0] = CodePageEntry->CodePageTable.DefaultChar & 0xff;
//         CodePageInfo->DefaultChar[1] = 0;
//     }
//
//     if ((CodePageInfo->MaxCharSize = CodePageEntry->CodePageTable.MaximumCharacterSize) == 2)
//         memcpy(CodePageInfo->LeadByte, CodePageEntry->CodePageTable.LeadByte, sizeof(CodePageInfo->LeadByte));
//     else
//         CodePageInfo->LeadByte[0] = CodePageInfo->LeadByte[1] = 0;
//
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetCPInfoExW(UINT CodePage,
//              DWORD dwFlags,
//              LPCPINFOEXW lpCPInfoEx)
// {
//     if (!GetCPInfo(CodePage, (LPCPINFO)lpCPInfoEx))
//         return FALSE;
//
//     switch(CodePage)
//     {
//         case CP_UTF7:
//         {
//             lpCPInfoEx->CodePage = CP_UTF7;
//             lpCPInfoEx->UnicodeDefaultChar = 0x3f;
//             return GetLocalisedText(lpCPInfoEx->CodePage,
//                                     lpCPInfoEx->CodePageName,
//                                     ARRAYSIZE(lpCPInfoEx->CodePageName),
//                                     GetThreadLocale()) != 0;
//         }
//         break;
//
//         case CP_UTF8:
//         {
//             lpCPInfoEx->CodePage = CP_UTF8;
//             lpCPInfoEx->UnicodeDefaultChar = 0x3f;
//             return GetLocalisedText(lpCPInfoEx->CodePage,
//                                     lpCPInfoEx->CodePageName,
//                                     ARRAYSIZE(lpCPInfoEx->CodePageName),
//                                     GetThreadLocale()) != 0;
//         }
//
//         default:
//         {
//             PCODEPAGE_ENTRY CodePageEntry;
//
//             CodePageEntry = IntGetCodePageEntry(CodePage);
//             if (CodePageEntry == NULL)
//             {
//                 DPRINT1("Could not get CodePage Entry! CodePageEntry = NULL\n");
//                 SetLastError(ERROR_INVALID_PARAMETER);
//                 return FALSE;
//             }
//
//             lpCPInfoEx->CodePage = CodePageEntry->CodePageTable.CodePage;
//             lpCPInfoEx->UnicodeDefaultChar = CodePageEntry->CodePageTable.UniDefaultChar;
//             return GetLocalisedText(lpCPInfoEx->CodePage,
//                                     lpCPInfoEx->CodePageName,
//                                     ARRAYSIZE(lpCPInfoEx->CodePageName),
//                                     GetThreadLocale()) != 0;
//         }
//         break;
//     }
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// GetCPInfoExA(UINT CodePage,
//              DWORD dwFlags,
//              LPCPINFOEXA lpCPInfoEx)
// {
//     CPINFOEXW CPInfo;
//
//     if (!GetCPInfoExW(CodePage, dwFlags, &CPInfo))
//         return FALSE;
//
//     /* the layout is the same except for CodePageName */
//     memcpy(lpCPInfoEx, &CPInfo, sizeof(CPINFOEXA));
//
//     WideCharToMultiByte(CP_ACP,
//                         0,
//                         CPInfo.CodePageName,
//                         -1,
//                         lpCPInfoEx->CodePageName,
//                         sizeof(lpCPInfoEx->CodePageName),
//                         NULL,
//                         NULL);
//     return TRUE;
// }
//
// /**
//  * @name WideCharToMultiByte
//  *
//  * Convert a wide-charater string to closest multi-byte equivalent.
//  *
//  * @param CodePage
//  *        Code page to be used to perform the conversion. It can be also
//  *        one of the special values (CP_ACP for ANSI code page, CP_MACCP
//  *        for Macintosh code page, CP_OEMCP for OEM code page, CP_THREAD_ACP
//  *        for thread active code page, CP_UTF7 or CP_UTF8).
//  * @param Flags
//  *        Additional conversion flags (WC_NO_BEST_FIT_CHARS, WC_COMPOSITECHECK,
//  *        WC_DISCARDNS, WC_SEPCHARS, WC_DEFAULTCHAR).
//  * @param WideCharString
//  *        Points to the wide-character string to be converted.
//  * @param WideCharCount
//  *        Size in WCHARs of WideCharStr, or 0 if the caller just wants to
//  *        know how large WideCharString should be for a successful conversion.
//  * @param MultiByteString
//  *        Points to the buffer to receive the translated string.
//  * @param MultiByteCount
//  *        Specifies the size in bytes of the buffer pointed to by the
//  *        MultiByteString parameter. If this value is zero, the function
//  *        returns the number of bytes required for the buffer.
//  * @param DefaultChar
//  *        Points to the character used if a wide character cannot be
//  *        represented in the specified code page. If this parameter is
//  *        NULL, a system default value is used.
//  * @param UsedDefaultChar
//  *        Points to a flag that indicates whether a default character was
//  *        used. This parameter can be NULL.
//  *
//  * @return Zero on error, otherwise the number of bytes written in the
//  *         MultiByteString buffer. Or the number of bytes needed for
//  *         the MultiByteString buffer if MultiByteCount is zero.
//  *
//  * @implemented
//  */
//
// INT
// WINAPI
// WideCharToMultiByte(UINT CodePage,
//                     DWORD Flags,
//                     LPCWSTR WideCharString,
//                     INT WideCharCount,
//                     LPSTR MultiByteString,
//                     INT MultiByteCount,
//                     LPCSTR DefaultChar,
//                     LPBOOL UsedDefaultChar)
// {
//     /* Check the parameters. */
//     if (WideCharString == NULL ||
//         WideCharCount == 0 ||
//         (MultiByteString == NULL && MultiByteCount > 0) ||
//         (PVOID)WideCharString == (PVOID)MultiByteString ||
//         MultiByteCount < 0)
//     {
//         SetLastError(ERROR_INVALID_PARAMETER);
//         return 0;
//     }
//
//     /* Determine the input string length. */
//     if (WideCharCount < 0)
//     {
//         WideCharCount = lstrlenW(WideCharString) + 1;
//     }
//
//     switch (CodePage)
//     {
//         case CP_UTF8:
//             if (DefaultChar != NULL || UsedDefaultChar != NULL)
//             {
//                 SetLastError(ERROR_INVALID_PARAMETER);
//                 return 0;
//             }
//             return IntWideCharToMultiByteUTF8(CodePage,
//                                               Flags,
//                                               WideCharString,
//                                               WideCharCount,
//                                               MultiByteString,
//                                               MultiByteCount,
//                                               DefaultChar,
//                                               UsedDefaultChar);
//
//         case CP_UTF7:
//             if (DefaultChar != NULL || UsedDefaultChar != NULL)
//             {
//                 SetLastError(ERROR_INVALID_PARAMETER);
//                 return 0;
//             }
//             if (Flags)
//             {
//                 SetLastError(ERROR_INVALID_FLAGS);
//                 return 0;
//             }
//             return WideCharToUtf7(WideCharString, WideCharCount,
//                                   MultiByteString, MultiByteCount);
//
//         case CP_SYMBOL:
//             if ((DefaultChar!=NULL) || (UsedDefaultChar!=NULL))
//             {
//                 SetLastError(ERROR_INVALID_PARAMETER);
//                 return 0;
//             }
//             return IntWideCharToMultiByteSYMBOL(Flags,
//                                                 WideCharString,
//                                                 WideCharCount,
//                                                 MultiByteString,
//                                                 MultiByteCount);
//
//         default:
//             return IntWideCharToMultiByteCP(CodePage,
//                                             Flags,
//                                             WideCharString,
//                                             WideCharCount,
//                                             MultiByteString,
//                                             MultiByteCount,
//                                             DefaultChar,
//                                             UsedDefaultChar);
//    }
// }
//
// /**
//  * @name GetACP
//  *
//  * Get active ANSI code page number.
//  *
//  * @implemented
//  */
//
// UINT
// WINAPI
// GetACP(VOID)
// {
//     return AnsiCodePage.CodePageTable.CodePage;
// }
//
// /**
//  * @name GetOEMCP
//  *
//  * Get active OEM code page number.
//  *
//  * @implemented
//  */
//
// UINT
// WINAPI
// GetOEMCP(VOID)
// {
//     return OemCodePage.CodePageTable.CodePage;
// }
//
// /**
//  * @name IsDBCSLeadByteEx
//  *
//  * Determine if passed byte is lead byte in specified code page.
//  *
//  * @implemented
//  */
//
// BOOL
// WINAPI
// IsDBCSLeadByteEx(UINT CodePage, BYTE TestByte)
// {
//     PCODEPAGE_ENTRY CodePageEntry;
//
//     CodePageEntry = IntGetCodePageEntry(CodePage);
//     if (CodePageEntry != NULL)
//         return IntIsLeadByte(&CodePageEntry->CodePageTable, TestByte);
//
//     SetLastError(ERROR_INVALID_PARAMETER);
//     return FALSE;
// }
//
// /**
//  * @name IsDBCSLeadByteEx
//  *
//  * Determine if passed byte is lead byte in current ANSI code page.
//  *
//  * @implemented
//  */
//
// BOOL
// WINAPI
// IsDBCSLeadByte(BYTE TestByte)
// {
//     return IntIsLeadByte(&AnsiCodePage.CodePageTable, TestByte);
// }
//
// /**
//  * @brief
//  * Creates a security descriptor for each NLS section. Typically used by
//  * BASESRV to give Everyone (World SID) read access to the sections.
//  *
//  * @param[out]  SecurityDescriptor
//  * A pointer to a correctly sized user-allocated buffer, that receives
//  * a security descriptor containing one ACL with one World SID.
//  * Its size should be at least equal to NLS_SECTION_SECURITY_DESCRIPTOR_SIZE.
//  *
//  * @param[in]   DescriptorSize
//  * Size (in bytes) of the user-provided SecurityDescriptor buffer.
//  *
//  * @param[in]   AccessMask
//  * An access mask that grants Everyone an access specific to that mask.
//  *
//  * @return
//  * STATUS_SUCCESS is returned if the function has successfully
//  * created a security descriptor for a NLS section name. Otherwise
//  * a NTSTATUS failure code is returned.
//  *
//  * @remark
//  * This implementation has to be made compatible with NT <= 5.2 in order
//  * to inter-operate with BASESRV. In particular, the security descriptor
//  * is a user-provided buffer correctly sized. The caller is responsible
//  * to submit the exact size of the descriptor.
//  **/
// NTSTATUS
// WINAPI
// CreateNlsSecurityDescriptor(
//      PSECURITY_DESCRIPTOR SecurityDescriptor,
//      SIZE_T DescriptorSize,
//      ULONG AccessMask)
// {
//     static SID_IDENTIFIER_AUTHORITY WorldAuthority = {SECURITY_WORLD_SID_AUTHORITY};
//     NTSTATUS Status;
//     PSID WorldSid;
//     PACL Dacl;
//     ULONG DaclSize;
//
//     if (DescriptorSize < NLS_SECTION_SECURITY_DESCRIPTOR_SIZE)
//     {
//         DPRINT1("Security descriptor size too small\n");
//         return STATUS_BUFFER_TOO_SMALL;
//     }
//
//     /* Create the World SID */
//     Status = RtlAllocateAndInitializeSid(&WorldAuthority,
//                                          1,
//                                          SECURITY_WORLD_RID,
//                                          0, 0, 0, 0, 0, 0, 0,
//                                          &WorldSid);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create World SID (Status 0x%08x)\n", Status);
//         return Status;
//     }
//
//     /* Initialize the security descriptor */
//     Status = RtlCreateSecurityDescriptor(SecurityDescriptor,
//                                          SECURITY_DESCRIPTOR_REVISION);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create security descriptor (Status 0x%08x)\n", Status);
//         goto Quit;
//     }
//
//     /* The DACL follows the security descriptor, and includes the World SID */
//     Dacl = (PACL)((ULONG_PTR)SecurityDescriptor + sizeof(SECURITY_DESCRIPTOR));
//     DaclSize = DescriptorSize - sizeof(SECURITY_DESCRIPTOR);
//
//     /* Create the DACL */
//     Status = RtlCreateAcl(Dacl, DaclSize, ACL_REVISION);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create DACL (Status 0x%08x)\n", Status);
//         goto Quit;
//     }
//
//     /* Add an allowed access ACE to the World SID */
//     Status = RtlAddAccessAllowedAce(Dacl, ACL_REVISION, AccessMask, WorldSid);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to add allowed access ACE for World SID (Status 0x%08x)\n", Status);
//         goto Quit;
//     }
//
//     /* Set the DACL to the descriptor */
//     Status = RtlSetDaclSecurityDescriptor(SecurityDescriptor, TRUE, Dacl, FALSE);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to set DACL into descriptor (Status 0x%08x)\n", Status);
//         goto Quit;
//     }
//
// Quit:
//     RtlFreeSid(WorldSid);
//     return Status;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL WINAPI IsValidUILanguage(LANGID langid)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// VOID WINAPI NlsConvertIntegerToString(ULONG Value,ULONG Base,ULONG strsize, LPWSTR str, ULONG strsize2)
// {
//     STUB;
// }
//
// /*
//  * @unimplemented
//  */
// UINT WINAPI SetCPGlobal(UINT CodePage)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// ValidateLCType(int a1, unsigned int a2, int a3, int a4)
// {
//     STUB;
//     return FALSE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// NlsResetProcessLocale(VOID)
// {
//     STUB;
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// WINAPI
// GetDefaultSortkeySize(LPVOID lpUnknown)
// {
//     STUB;
//     lpUnknown = NULL;
// }
//
// /*
//  * @unimplemented
//  */
// VOID
// WINAPI
// GetLinguistLangSize(LPVOID lpUnknown)
// {
//     STUB;
//     lpUnknown = NULL;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// ValidateLocale(IN ULONG LocaleId)
// {
//     STUB;
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// ULONG
// WINAPI
// NlsGetCacheUpdateCount(VOID)
// {
//     STUB;
//     return 0;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// IsNLSDefinedString(IN NLS_FUNCTION Function,
//                    IN DWORD dwFlags,
//                    IN LPNLSVERSIONINFO lpVersionInformation,
//                    IN LPCWSTR lpString,
//                    IN INT cchStr)
// {
//     STUB;
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// GetNLSVersion(IN NLS_FUNCTION Function,
//               IN LCID Locale,
//               IN OUT LPNLSVERSIONINFO lpVersionInformation)
// {
//     STUB;
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// BOOL
// WINAPI
// GetNLSVersionEx(IN NLS_FUNCTION function,
//                 IN LPCWSTR lpLocaleName,
//                 IN OUT LPNLSVERSIONINFOEX lpVersionInformation)
// {
//     STUB;
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
