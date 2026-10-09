use crate::prelude::*;

// c2rust named the dispatch-table struct `AUTH_PACKAGE`; the offset_of! call uses
// the C struct-tag spelling `_AUTH_PACKAGE`.
pub type _AUTH_PACKAGE = AUTH_PACKAGE;

// Rust-sibling LSA logon-session + credential helpers (session.rs, #[no_mangle]).
use crate::lsasrv::session::{
    LsapAddCredential, LsapCreateLogonSession, LsapDeleteCredential, LsapDeleteLogonSession,
    LsapGetCredentials,
};
// Account/builtin domain SIDs — already #[no_mangle] statics in database.rs.
use crate::lsasrv::database::{AccountDomainSid, BuiltinDomainSid};

// Rust siblings (#[no_mangle] in their modules).
use crate::lsasrv::lsasrv::LsaIFree_LSAPR_PRIVILEGE_SET;
use crate::lsasrv::session::LsapSetLogonSessionData;

// Well-known SID globals — now #[no_mangle] siblings in the flipped-to-Rust
// lookup.rs (was extern-to-C lookup.c). Bound here by Rust path.
use crate::lsasrv::lookup::{
    LsapWorldSid, LsapLocalSystemSid, LsapInteractiveSid, LsapNetworkSid,
    LsapServiceSid, LsapBatchSid, LsapAdministratorsSid,
};

// LSA RPC server stubs (lsa_s.h, __stdcall/NTAPI → extern "system") + SAM/Win32
// bridge fns. LsarOpenPolicy/OpenAccount/Close/EnumeratePrivilegesAccount are the
// widl-generated server routines; SamIConnect is the samsrv NTAPI export; EqualSid
// is the Win32 advapi32 export (authpackage.c calls EqualSid, not RtlEqualSid, here).
extern "system" {
    fn LsarOpenPolicy(
        SystemName: PLSAPR_SERVER_NAME,
        ObjectAttributes: PLSAPR_OBJECT_ATTRIBUTES,
        DesiredAccess: ACCESS_MASK,
        PolicyHandle: *mut LSAPR_HANDLE,
    ) -> NTSTATUS;
    fn LsarOpenAccount(
        PolicyHandle: LSAPR_HANDLE,
        AccountSid: PRPC_SID,
        DesiredAccess: ACCESS_MASK,
        AccountHandle: *mut LSAPR_HANDLE,
    ) -> NTSTATUS;
    fn LsarClose(ObjectHandle: *mut LSAPR_HANDLE) -> NTSTATUS;
    fn LsarEnumeratePrivilegesAccount(
        AccountHandle: LSAPR_HANDLE,
        Privileges: *mut PLSAPR_PRIVILEGE_SET,
    ) -> NTSTATUS;
    fn SamIConnect(
        ServerName: PSAMPR_SERVER_NAME,
        ServerHandle: *mut SAMPR_HANDLE,
        DesiredAccess: ACCESS_MASK,
        Trusted: BOOLEAN,
    ) -> NTSTATUS;
    fn EqualSid(pSid1: PSID, pSid2: PSID) -> BOOL;
}

// LsapAppendRidToSid / LsapCreateTokenSd are plain C (cdecl) lsasrv-internal fns.
extern "C" {
    fn LsapAppendRidToSid(SrcSid: PSID, Rid: ULONG) -> PSID;
    fn LsapCreateTokenSd(
        User: *const TOKEN_USER,
        TokenSd: *mut PSECURITY_DESCRIPTOR,
        TokenSdSize: PULONG,
    ) -> NTSTATUS;
}

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut __wine_dbch_lsasrv: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"lsasrv\0\0\0\0\0\0\0\0\0",
        ),
    }
};

static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_lsasrv as *mut __wine_debug_channel };

static mut PackageListHead: LIST_ENTRY = LIST_ENTRY {
    Flink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
    Blink: ::core::ptr::null::<_LIST_ENTRY>() as *mut _LIST_ENTRY,
};

static mut LsapGlobalPackageId: ULONG = 0;

static mut DispatchTable: LSA_DISPATCH_TABLE = LSA_DISPATCH_TABLE {
    CreateLogonSession: None,
    DeleteLogonSession: None,
    AddCredential: None,
    GetCredentials: None,
    DeleteCredential: None,
    AllocateLsaHeap: None,
    FreeLsaHeap: None,
    AllocateClientBuffer: None,
    FreeClientBuffer: None,
    CopyToClientBuffer: None,
    CopyFromClientBuffer: None,
};

static mut SeChangeNotifyPrivilege: LUID = _LUID {
    LowPart: 23 as DWORD,
    HighPart: 0 as LONG,
};

static mut SeCreateGlobalPrivilege: LUID = _LUID {
    LowPart: 30 as DWORD,
    HighPart: 0 as LONG,
};

static mut SeImpersonatePrivilege: LUID = _LUID {
    LowPart: 29 as DWORD,
    HighPart: 0 as LONG,
};

// LSA_TOKEN_INFORMATION_TYPE + LsaTokenInformation{Null,V1} come from the prelude
// (c_uint typedef + consts), so == / != compares and the prelude consts line up.
pub type PLSA_TOKEN_INFORMATION_TYPE = *mut LSA_TOKEN_INFORMATION_TYPE;

#[repr(C)]
pub struct LSA_TOKEN_INFORMATION_NULL {
    pub ExpirationTime: LARGE_INTEGER,
    pub Groups: PTOKEN_GROUPS,
}
pub type PLSA_TOKEN_INFORMATION_NULL = *mut LSA_TOKEN_INFORMATION_NULL;

// LSA_TOKEN_INFORMATION_V1 is the prelude's canonical _LSA_TOKEN_INFORMATION_V1
// (same #[repr(C)] layout). Alias rather than redefine so the file's
// PLSA_TOKEN_INFORMATION_V1 and the bindgen `*mut _LSA_TOKEN_INFORMATION_V1` used
// elsewhere in this file are the *same* type (avoids cross-definition mismatches).
pub type PLSA_TOKEN_INFORMATION_V1 = *mut _LSA_TOKEN_INFORMATION_V1;

pub type PLSA_CLIENT_REQUEST = PVOID;

pub type PLSA_CREATE_LOGON_SESSION = Option<unsafe extern "system" fn(PLUID) -> NTSTATUS>;
pub type PLSA_DELETE_LOGON_SESSION = Option<unsafe extern "system" fn(PLUID) -> NTSTATUS>;
pub type PLSA_ADD_CREDENTIAL = Option<unsafe extern "system" fn(PLUID, ULONG, PLSA_STRING, PLSA_STRING) -> NTSTATUS>;
pub type PLSA_GET_CREDENTIALS = Option<unsafe extern "system" fn(PLUID, ULONG, PULONG, BOOLEAN, PLSA_STRING, PULONG, PLSA_STRING) -> NTSTATUS>;
pub type PLSA_DELETE_CREDENTIAL = Option<unsafe extern "system" fn(PLUID, ULONG, PLSA_STRING) -> NTSTATUS>;
pub type PLSA_ALLOCATE_LSA_HEAP = Option<unsafe extern "system" fn(ULONG) -> PVOID>;
pub type PLSA_FREE_LSA_HEAP = Option<unsafe extern "system" fn(PVOID)>;
pub type PLSA_ALLOCATE_CLIENT_BUFFER = Option<unsafe extern "system" fn(PLSA_CLIENT_REQUEST, ULONG, *mut PVOID) -> NTSTATUS>;
pub type PLSA_FREE_CLIENT_BUFFER = Option<unsafe extern "system" fn(PLSA_CLIENT_REQUEST, PVOID) -> NTSTATUS>;
pub type PLSA_COPY_TO_CLIENT_BUFFER = Option<unsafe extern "system" fn(PLSA_CLIENT_REQUEST, ULONG, PVOID, PVOID) -> NTSTATUS>;
pub type PLSA_COPY_FROM_CLIENT_BUFFER = Option<unsafe extern "system" fn(PLSA_CLIENT_REQUEST, ULONG, PVOID, PVOID) -> NTSTATUS>;

#[repr(C)]
pub struct LSA_DISPATCH_TABLE {
    pub CreateLogonSession: PLSA_CREATE_LOGON_SESSION,
    pub DeleteLogonSession: PLSA_DELETE_LOGON_SESSION,
    pub AddCredential: PLSA_ADD_CREDENTIAL,
    pub GetCredentials: PLSA_GET_CREDENTIALS,
    pub DeleteCredential: PLSA_DELETE_CREDENTIAL,
    pub AllocateLsaHeap: PLSA_ALLOCATE_LSA_HEAP,
    pub FreeLsaHeap: PLSA_FREE_LSA_HEAP,
    pub AllocateClientBuffer: PLSA_ALLOCATE_CLIENT_BUFFER,
    pub FreeClientBuffer: PLSA_FREE_CLIENT_BUFFER,
    pub CopyToClientBuffer: PLSA_COPY_TO_CLIENT_BUFFER,
    pub CopyFromClientBuffer: PLSA_COPY_FROM_CLIENT_BUFFER,
}
pub type PLSA_DISPATCH_TABLE = *mut LSA_DISPATCH_TABLE;

pub type PLSA_AP_INITIALIZE_PACKAGE = Option<unsafe extern "system" fn(ULONG, PLSA_DISPATCH_TABLE, PLSA_STRING, PLSA_STRING, *mut PLSA_STRING) -> NTSTATUS>;
pub type PLSA_AP_CALL_PACKAGE_INTERNAL = Option<unsafe extern "system" fn(PLSA_CLIENT_REQUEST, PVOID, PVOID, ULONG, *mut PVOID, *mut ULONG, *mut NTSTATUS) -> NTSTATUS>;
pub type PLSA_AP_CALL_PACKAGE_PASSTHROUGH = Option<unsafe extern "system" fn(PLSA_CLIENT_REQUEST, PVOID, PVOID, ULONG, *mut PVOID, *mut ULONG, *mut NTSTATUS) -> NTSTATUS>;
pub type PLSA_AP_CALL_PACKAGE_UNTRUSTED = Option<unsafe extern "system" fn(PLSA_CLIENT_REQUEST, PVOID, PVOID, ULONG, *mut PVOID, *mut ULONG, *mut NTSTATUS) -> NTSTATUS>;
pub type PLSA_AP_LOGON_TERMINATED = Option<unsafe extern "system" fn(PLUID)>;
pub type PLSA_AP_LOGON_USER_EX2 = Option<unsafe extern "system" fn(
    PLSA_CLIENT_REQUEST,
    SECURITY_LOGON_TYPE,
    PVOID,
    PVOID,
    ULONG,
    *mut PVOID,
    *mut ULONG,
    PLUID,
    *mut NTSTATUS,
    *mut LSA_TOKEN_INFORMATION_TYPE,
    *mut PVOID,
    *mut PUNICODE_STRING,
    *mut PUNICODE_STRING,
    *mut PUNICODE_STRING,
    PVOID,
    PVOID,
) -> NTSTATUS>;
pub type PLSA_AP_LOGON_USER_EX = Option<unsafe extern "system" fn(
    PLSA_CLIENT_REQUEST,
    SECURITY_LOGON_TYPE,
    PVOID,
    PVOID,
    ULONG,
    *mut PVOID,
    *mut ULONG,
    PLUID,
    *mut NTSTATUS,
    *mut LSA_TOKEN_INFORMATION_TYPE,
    *mut PVOID,
    *mut PUNICODE_STRING,
    *mut PUNICODE_STRING,
    *mut PUNICODE_STRING,
) -> NTSTATUS>;
pub type PLSA_AP_LOGON_USER_INTERNAL = Option<unsafe extern "system" fn(
    PLSA_CLIENT_REQUEST,
    SECURITY_LOGON_TYPE,
    PVOID,
    PVOID,
    ULONG,
    *mut PVOID,
    *mut ULONG,
    PLUID,
    *mut NTSTATUS,
    *mut LSA_TOKEN_INFORMATION_TYPE,
    *mut PVOID,
    *mut PUNICODE_STRING,
    *mut PUNICODE_STRING,
) -> NTSTATUS>;

#[repr(C)]
pub struct AUTH_PACKAGE {
    pub Entry: LIST_ENTRY,
    pub Name: PSTRING,
    pub Id: u32,
    pub ModuleHandle: PVOID,
    pub LsaApInitializePackage: PLSA_AP_INITIALIZE_PACKAGE,
    pub LsaApCallPackage: PLSA_AP_CALL_PACKAGE_INTERNAL,
    pub LsaApCallPackagePassthrough: PLSA_AP_CALL_PACKAGE_PASSTHROUGH,
    pub LsaApCallPackageUntrusted: PLSA_AP_CALL_PACKAGE_UNTRUSTED,
    pub LsaApLogonTerminated: PLSA_AP_LOGON_TERMINATED,
    pub LsaApLogonUserEx2: PLSA_AP_LOGON_USER_EX2,
    pub LsaApLogonUserEx: PLSA_AP_LOGON_USER_EX,
    pub LsaApLogonUser: PLSA_AP_LOGON_USER_INTERNAL,
}
pub type PAUTH_PACKAGE = *mut AUTH_PACKAGE;

pub type PSAMPR_SERVER_NAME = *mut u16;
pub type SAMPR_HANDLE = *mut core::ffi::c_void;

#[repr(C)]
pub struct SAMPR_SID_INFORMATION {
    pub SidPointer: PRPC_SID,
}
pub type PSAMPR_SID_INFORMATION = *mut SAMPR_SID_INFORMATION;

#[repr(C)]
pub struct SAMPR_PSID_ARRAY {
    pub Count: u32,
    pub Sids: PSAMPR_SID_INFORMATION,
}
pub type PSAMPR_PSID_ARRAY = *mut SAMPR_PSID_ARRAY;

unsafe extern "system" fn LsapAddAuthPackage(
    _ValueName: PWSTR,
    _ValueType: ULONG,
    ValueData: PVOID,
    ValueLength: ULONG,
    Context: PVOID,
    _EntryContext: PVOID,
) -> NTSTATUS {
    let Package: PAUTH_PACKAGE;
    let mut PackageName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut ProcName: STRING = STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let Id: PULONG;
    let mut Status: NTSTATUS;

    TRACE!(
        b"LsapAddAuthPackage()\n\0".as_ptr() as *const u8,
    );

    PackageName.Length = (ValueLength as u16)
        .wrapping_sub(core::mem::size_of::<u16>() as u16);
    PackageName.MaximumLength = ValueLength as u16;
    PackageName.Buffer = ValueData as PWSTR;

    Id = Context as PULONG;

    Package = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY as ULONG,
        core::mem::size_of::<AUTH_PACKAGE>() as SIZE_T,
    ) as PAUTH_PACKAGE;
    if Package.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    'done: {
        Status = LdrLoadDll(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!(PackageName),
            core::ptr::addr_of_mut!((*Package).ModuleHandle),
        );
        if !(Status >= 0) {
            TRACE!(
                b"LdrLoadDll failed (Status 0x%08lx)\n\0".as_ptr() as *const u8,
                Status,
            );
            break 'done;
        }

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(ProcName),
            b"LsaApInitializePackage\0".as_ptr() as PCSZ,
        );
        Status = LdrGetProcedureAddress(
            (*Package).ModuleHandle,
            core::ptr::addr_of_mut!(ProcName) as PSTRING,
            0,
            core::ptr::addr_of_mut!((*Package).LsaApInitializePackage) as *mut PVOID,
        );
        if !(Status >= 0) {
            TRACE!(
                b"LdrGetProcedureAddress() failed (Status 0x%08lx)\n\0".as_ptr()
                    as *const u8,
                Status,
            );
            break 'done;
        }

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(ProcName),
            b"LsaApCallPackage\0".as_ptr() as PCSZ,
        );
        Status = LdrGetProcedureAddress(
            (*Package).ModuleHandle,
            core::ptr::addr_of_mut!(ProcName) as PSTRING,
            0,
            core::ptr::addr_of_mut!((*Package).LsaApCallPackage) as *mut PVOID,
        );
        if !(Status >= 0) {
            TRACE!(
                b"LdrGetProcedureAddress() failed (Status 0x%08lx)\n\0".as_ptr()
                    as *const u8,
                Status,
            );
            break 'done;
        }

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(ProcName),
            b"LsaApCallPackagePassthrough\0".as_ptr() as PCSZ,
        );
        Status = LdrGetProcedureAddress(
            (*Package).ModuleHandle,
            core::ptr::addr_of_mut!(ProcName) as PSTRING,
            0,
            core::ptr::addr_of_mut!((*Package).LsaApCallPackagePassthrough)
                as *mut PVOID,
        );
        if !(Status >= 0) {
            TRACE!(
                b"LdrGetProcedureAddress() failed (Status 0x%08lx)\n\0".as_ptr()
                    as *const u8,
                Status,
            );
            break 'done;
        }

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(ProcName),
            b"LsaApCallPackageUntrusted\0".as_ptr() as PCSZ,
        );
        Status = LdrGetProcedureAddress(
            (*Package).ModuleHandle,
            core::ptr::addr_of_mut!(ProcName) as PSTRING,
            0,
            core::ptr::addr_of_mut!((*Package).LsaApCallPackageUntrusted)
                as *mut PVOID,
        );
        if !(Status >= 0) {
            TRACE!(
                b"LdrGetProcedureAddress() failed (Status 0x%08lx)\n\0".as_ptr()
                    as *const u8,
                Status,
            );
            break 'done;
        }

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(ProcName),
            b"LsaApLogonTerminated\0".as_ptr() as PCSZ,
        );
        Status = LdrGetProcedureAddress(
            (*Package).ModuleHandle,
            core::ptr::addr_of_mut!(ProcName) as PSTRING,
            0,
            core::ptr::addr_of_mut!((*Package).LsaApLogonTerminated) as *mut PVOID,
        );
        if !(Status >= 0) {
            TRACE!(
                b"LdrGetProcedureAddress() failed (Status 0x%08lx)\n\0".as_ptr()
                    as *const u8,
                Status,
            );
            break 'done;
        }

        RtlInitAnsiString(
            core::ptr::addr_of_mut!(ProcName),
            b"LsaApLogonUserEx2\0".as_ptr() as PCSZ,
        );
        Status = LdrGetProcedureAddress(
            (*Package).ModuleHandle,
            core::ptr::addr_of_mut!(ProcName) as PSTRING,
            0,
            core::ptr::addr_of_mut!((*Package).LsaApLogonUserEx2) as *mut PVOID,
        );
        if !(Status >= 0) {
            RtlInitAnsiString(
                core::ptr::addr_of_mut!(ProcName),
                b"LsaApLogonUserEx\0".as_ptr() as PCSZ,
            );
            Status = LdrGetProcedureAddress(
                (*Package).ModuleHandle,
                core::ptr::addr_of_mut!(ProcName) as PSTRING,
                0,
                core::ptr::addr_of_mut!((*Package).LsaApLogonUserEx) as *mut PVOID,
            );
            if !(Status >= 0) {
                RtlInitAnsiString(
                    core::ptr::addr_of_mut!(ProcName),
                    b"LsaApLogonUser\0".as_ptr() as PCSZ,
                );
                Status = LdrGetProcedureAddress(
                    (*Package).ModuleHandle,
                    core::ptr::addr_of_mut!(ProcName) as PSTRING,
                    0,
                    core::ptr::addr_of_mut!((*Package).LsaApLogonUser) as *mut PVOID,
                );
                if !(Status >= 0) {
                    TRACE!(
                        b"LdrGetProcedureAddress() failed (Status 0x%08lx)\n\0".as_ptr()
                            as *const u8,
                        Status,
                    );
                    break 'done;
                }
            }
        }

        Status = ((*Package).LsaApInitializePackage).expect("non-null function pointer")(
            *Id,
            core::ptr::addr_of_mut!(DispatchTable),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::addr_of_mut!((*Package).Name) as *mut PLSA_STRING,
        );
        if !(Status >= 0) {
            TRACE!(
                b"Package->LsaApInitializePackage() failed (Status 0x%08lx)\n\0".as_ptr()
                    as *const u8,
                Status,
            );
            break 'done;
        }

        TRACE!(
            b"Package Name: %s\n\0".as_ptr() as *const u8,
            (*(*Package).Name).Buffer,
        );

        (*Package).Id = *Id;
        *Id = (*Id).wrapping_add(1);

        InsertTailList(
            core::ptr::addr_of_mut!(PackageListHead),
            core::ptr::addr_of_mut!((*Package).Entry),
        );
    }

    if !(Status >= 0) {
        if !Package.is_null() {
            if !(*Package).ModuleHandle.is_null() {
                LdrUnloadDll((*Package).ModuleHandle);
            }
            if !(*Package).Name.is_null() {
                if !(*(*Package).Name).Buffer.is_null() {
                    RtlFreeHeap(
                        RtlGetProcessHeap(),
                        0,
                        (*(*Package).Name).Buffer as PVOID,
                    );
                }
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*Package).Name as PVOID);
            }
            RtlFreeHeap(RtlGetProcessHeap(), 0, Package as PVOID);
        }
    }

    Status
}

unsafe fn LsapGetAuthenticationPackage(PackageId: ULONG) -> PAUTH_PACKAGE {
    let mut ListEntry: PLIST_ENTRY = PackageListHead.Flink;
    while ListEntry != core::ptr::addr_of!(PackageListHead) as *mut _ {
        let offset = core::mem::offset_of!(AUTH_PACKAGE, Entry);
        let Package = ((ListEntry as usize) - offset) as PAUTH_PACKAGE;
        if (*Package).Id == PackageId {
            return Package;
        }
        ListEntry = (*ListEntry).Flink;
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn LsapAllocateHeap(Length: ULONG) -> PVOID {
    RtlAllocateHeap(RtlGetProcessHeap(), 0u32, Length as SIZE_T)
}

#[no_mangle]
pub unsafe extern "system" fn LsapAllocateHeapZero(Length: ULONG) -> PVOID {
    RtlAllocateHeap(RtlGetProcessHeap(), HEAP_ZERO_MEMORY, Length as SIZE_T)
}

#[no_mangle]
pub unsafe extern "system" fn LsapFreeHeap(Base: PVOID) {
    RtlFreeHeap(RtlGetProcessHeap(), 0u32, Base);
}

unsafe extern "system" fn LsapAllocateClientBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    LengthRequired: ULONG,
    ClientBaseAddress: *mut PVOID,
) -> NTSTATUS {
    *ClientBaseAddress = core::ptr::null_mut();
    let logon_context: *mut _LSAP_LOGON_CONTEXT = ClientRequest as *mut _LSAP_LOGON_CONTEXT;
    let mut length: SIZE_T = LengthRequired as SIZE_T;
    NtAllocateVirtualMemory(
        (*logon_context).ClientProcessHandle,
        ClientBaseAddress,
        0 as ULONG_PTR,
        &raw mut length,
        MEM_COMMIT as ULONG,
        PAGE_READWRITE as ULONG,
    )
}

unsafe extern "system" fn LsapFreeClientBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    mut ClientBaseAddress: PVOID,
) -> NTSTATUS {
    if ClientBaseAddress.is_null() {
        return STATUS_SUCCESS;
    }
    let logon_context: PLSAP_LOGON_CONTEXT = ClientRequest as PLSAP_LOGON_CONTEXT;
    let mut length: SIZE_T = 0;
    NtFreeVirtualMemory(
        (*logon_context).ClientProcessHandle,
        core::ptr::addr_of_mut!(ClientBaseAddress),
        core::ptr::addr_of_mut!(length),
        MEM_RELEASE as u32,
    )
}

unsafe extern "system" fn LsapCopyToClientBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    Length: ULONG,
    ClientBaseAddress: PVOID,
    BufferToCopy: PVOID,
) -> NTSTATUS {
    let LogonContext = ClientRequest as *mut _LSAP_LOGON_CONTEXT;
    NtWriteVirtualMemory(
        (*LogonContext).ClientProcessHandle,
        ClientBaseAddress,
        BufferToCopy,
        Length as SIZE_T,
        core::ptr::null_mut::<SIZE_T>(),
    )
}

unsafe extern "system" fn LsapCopyFromClientBuffer(
    ClientRequest: PLSA_CLIENT_REQUEST,
    Length: ULONG,
    BufferToCopy: PVOID,
    ClientBaseAddress: PVOID,
) -> NTSTATUS {
    let LogonContext = ClientRequest as PLSAP_LOGON_CONTEXT;
    NtReadVirtualMemory(
        (*LogonContext).ClientProcessHandle,
        ClientBaseAddress,
        BufferToCopy,
        Length as SIZE_T,
        core::ptr::null_mut(),
    )
}
#[no_mangle]
pub unsafe extern "C" fn LsapInitAuthPackages() -> NTSTATUS {
    let name_bytes: [u8; 48] = *b"A\0u\0t\0h\0e\0n\0t\0i\0c\0a\0t\0i\0o\0n\0 \0P\0a\0c\0k\0a\0g\0e\0s\0\0\0";
    let lsa_bytes: [u8; 8] = *b"L\0s\0a\0\0\0";
    // The [u16] name must OUTLIVE AuthPackageTable — its Name field stores a raw pointer
    // that RtlQueryRegistryValues dereferences below. Transmuting inline and calling
    // `.as_ptr()` on the temporary left Name dangling the instant the table initializer
    // ended (dangling_pointers_from_temporaries). Bind it to a function-scoped local.
    let name_wide: [u16; 24] = ::core::mem::transmute::<[u8; 48], [u16; 24]>(name_bytes);

    let mut AuthPackageTable: [_RTL_QUERY_REGISTRY_TABLE; 2] = [
        _RTL_QUERY_REGISTRY_TABLE {
            // RtlQueryRegistryValues invokes QueryRoutine with the NT callback ABI
            // (stdcall on i386, C on amd64/arm64) — exactly what extern "system"
            // lowers to per-arch. rustc keeps "system" distinct from the bindgen
            // field's literal "stdcall"/"C" ABI string, so transmute the (ABI-
            // identical) fn pointer into the field's PRTL_QUERY_REGISTRY_ROUTINE type.
            QueryRoutine: ::core::mem::transmute::<
                unsafe extern "system" fn(PWSTR, ULONG, PVOID, ULONG, PVOID, PVOID) -> NTSTATUS,
                PRTL_QUERY_REGISTRY_ROUTINE,
            >(LsapAddAuthPackage),
            Flags: 0,
            Name: name_wide.as_ptr() as PCWSTR,
            EntryContext: core::ptr::null_mut(),
            DefaultType: REG_NONE as ULONG,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
        _RTL_QUERY_REGISTRY_TABLE {
            QueryRoutine: None,
            Flags: 0,
            Name: core::ptr::null::<u16>(),
            EntryContext: core::ptr::null_mut(),
            DefaultType: REG_NONE as ULONG,
            DefaultData: core::ptr::null_mut(),
            DefaultLength: 0,
        },
    ];

    InitializeListHead(core::ptr::addr_of_mut!(PackageListHead));
    LsapGlobalPackageId = 0;

    DispatchTable.CreateLogonSession = Some(LsapCreateLogonSession as unsafe extern "system" fn(PLUID) -> NTSTATUS);
    DispatchTable.DeleteLogonSession = Some(LsapDeleteLogonSession as unsafe extern "system" fn(PLUID) -> NTSTATUS);
    DispatchTable.AddCredential = Some(
        LsapAddCredential
            as unsafe extern "system" fn(PLUID, ULONG, PLSA_STRING, PLSA_STRING) -> NTSTATUS,
    );
    DispatchTable.GetCredentials = Some(
        LsapGetCredentials
            as unsafe extern "system" fn(
                PLUID,
                ULONG,
                PULONG,
                BOOLEAN,
                PLSA_STRING,
                PULONG,
                PLSA_STRING,
            ) -> NTSTATUS,
    );
    DispatchTable.DeleteCredential =
        Some(LsapDeleteCredential as unsafe extern "system" fn(PLUID, ULONG, PLSA_STRING) -> NTSTATUS);
    DispatchTable.AllocateLsaHeap =
        Some(LsapAllocateHeapZero as unsafe extern "system" fn(ULONG) -> PVOID);
    DispatchTable.FreeLsaHeap = Some(LsapFreeHeap as unsafe extern "system" fn(PVOID) -> ());
    DispatchTable.AllocateClientBuffer = Some(
        LsapAllocateClientBuffer
            as unsafe extern "system" fn(PLSA_CLIENT_REQUEST, ULONG, *mut PVOID) -> NTSTATUS,
    );
    DispatchTable.FreeClientBuffer = Some(
        LsapFreeClientBuffer as unsafe extern "system" fn(PLSA_CLIENT_REQUEST, PVOID) -> NTSTATUS,
    );
    DispatchTable.CopyToClientBuffer = Some(
        LsapCopyToClientBuffer
            as unsafe extern "system" fn(PLSA_CLIENT_REQUEST, ULONG, PVOID, PVOID) -> NTSTATUS,
    );
    DispatchTable.CopyFromClientBuffer = Some(
        LsapCopyFromClientBuffer
            as unsafe extern "system" fn(PLSA_CLIENT_REQUEST, ULONG, PVOID, PVOID) -> NTSTATUS,
    );

    let Status = RtlQueryRegistryValues(
        RTL_REGISTRY_CONTROL,
        ::core::mem::transmute::<[u8; 8], [u16; 4]>(lsa_bytes).as_ptr() as PCWSTR,
        core::ptr::addr_of_mut!(AuthPackageTable) as PRTL_QUERY_REGISTRY_TABLE,
        core::ptr::addr_of_mut!(LsapGlobalPackageId) as PVOID,
        core::ptr::null_mut(),
    );

    Status
}
#[no_mangle]
pub unsafe extern "C" fn LsapLookupAuthenticationPackage(
    RequestMsg: PLSA_API_MSG,
    LogonContext: PLSAP_LOGON_CONTEXT,
) -> NTSTATUS {
ros_dbg_log(
    __WINE_DBCL_TRACE,
    __wine_dbch___default,
    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authpackage.c\0".as_ptr() as *const i8,
    b"LsapLookupAuthenticationPackage\0".as_ptr() as *const i8,
    518,
    b"(%p %p)\n\0".as_ptr() as *const i8,
    RequestMsg,
    LogonContext,
);
let PackageNameLength = (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LookupAuthenticationPackage.__bindgen_anon_1.Request.PackageNameLength;
let PackageName = core::ptr::addr_of_mut!((*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LookupAuthenticationPackage.__bindgen_anon_1.Request.PackageName) as PCHAR;
ros_dbg_log(
    __WINE_DBCL_TRACE,
    __wine_dbch___default,
    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authpackage.c\0".as_ptr() as *const i8,
    b"LsapLookupAuthenticationPackage\0".as_ptr() as *const i8,
    523,
    b"PackageName: %s\n\0".as_ptr() as *const i8,
    PackageName,
);
let mut ListEntry = PackageListHead.Flink as PLIST_ENTRY;
let head_ptr = core::ptr::addr_of_mut!(PackageListHead);
while ListEntry != head_ptr {
    let entry_offset = core::mem::offset_of!(AUTH_PACKAGE, Entry);
    let Package = (ListEntry as usize).wrapping_sub(entry_offset) as PAUTH_PACKAGE;
    if PackageNameLength == (*(*Package).Name).Length as u32
        && _strnicmp(
            PackageName as *const i8,
            (*(*Package).Name).Buffer as *const i8,
            (*(*Package).Name).Length as usize,
        ) == 0
    {
        (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LookupAuthenticationPackage.__bindgen_anon_1.Reply.Package = (*Package).Id;
        return STATUS_SUCCESS;
    }
    ListEntry = (*ListEntry).Flink as PLIST_ENTRY;
}
STATUS_NO_SUCH_PACKAGE
}
#[no_mangle]
pub unsafe extern "C" fn LsapTerminateLogon(LogonId: PLUID) {
    let mut list_entry = PackageListHead.Flink;
    while list_entry != core::ptr::addr_of_mut!(PackageListHead) {
        let offset = core::mem::offset_of!(_AUTH_PACKAGE, Entry);
        let package = (list_entry as *mut u8).offset(-(offset as isize)) as PAUTH_PACKAGE;
        ((*package).LsaApLogonTerminated).expect("non-null function pointer")(LogonId);
        list_entry = (*list_entry).Flink;
    }
}
#[no_mangle]
pub unsafe extern "C" fn LsapCallAuthenticationPackage(
    RequestMsg: *mut LSA_API_MSG,
    LogonContext: *mut LSAP_LOGON_CONTEXT,
) -> NTSTATUS {
    let Package: PAUTH_PACKAGE;
    let mut LocalBuffer: PVOID = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    TRACE!(b"(%p %p)\n\0".as_ptr() as *const i8, RequestMsg, LogonContext);

    let PackageId = (*RequestMsg)
        .__bindgen_anon_1
        .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
        .__bindgen_anon_1
        .Request
        .AuthenticationPackage;

    Package = LsapGetAuthenticationPackage(PackageId);
    if Package.is_null() {
        TRACE!(
            b"LsapGetAuthenticationPackage() failed to find a package\n\0".as_ptr() as *const i8,
        );
        return STATUS_NO_SUCH_PACKAGE;
    }

    if (*RequestMsg)
        .__bindgen_anon_1
        .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
        .__bindgen_anon_1
        .Request
        .SubmitBufferLength
        > 0
    {
        LocalBuffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            (*RequestMsg)
                .__bindgen_anon_1
                .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                .__bindgen_anon_1
                .Request
                .SubmitBufferLength as SIZE_T,
        );
        if LocalBuffer.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = NtReadVirtualMemory(
            (*LogonContext).ClientProcessHandle,
            (*RequestMsg)
                .__bindgen_anon_1
                .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                .__bindgen_anon_1
                .Request
                .ProtocolSubmitBuffer,
            LocalBuffer,
            (*RequestMsg)
                .__bindgen_anon_1
                .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                .__bindgen_anon_1
                .Request
                .SubmitBufferLength as SIZE_T,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"NtReadVirtualMemory() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            RtlFreeHeap(RtlGetProcessHeap(), 0, LocalBuffer);
            return Status;
        }
    }

    if (*LogonContext).TrustedCaller != 0 {
        Status = (*Package)
            .LsaApCallPackage
            .expect("non-null function pointer")(
            LogonContext as PLSA_CLIENT_REQUEST,
            LocalBuffer,
            (*RequestMsg)
                .__bindgen_anon_1
                .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                .__bindgen_anon_1
                .Request
                .ProtocolSubmitBuffer,
            (*RequestMsg)
                .__bindgen_anon_1
                .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                .__bindgen_anon_1
                .Request
                .SubmitBufferLength,
            core::ptr::addr_of_mut!(
                (*RequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                    .__bindgen_anon_1
                    .Reply
                    .ProtocolReturnBuffer
            ),
            core::ptr::addr_of_mut!(
                (*RequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                    .__bindgen_anon_1
                    .Reply
                    .ReturnBufferLength
            ),
            core::ptr::addr_of_mut!(
                (*RequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                    .__bindgen_anon_1
                    .Reply
                    .ProtocolStatus
            ),
        );
    } else {
        Status = (*Package)
            .LsaApCallPackageUntrusted
            .expect("non-null function pointer")(
            LogonContext as PLSA_CLIENT_REQUEST,
            LocalBuffer,
            (*RequestMsg)
                .__bindgen_anon_1
                .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                .__bindgen_anon_1
                .Request
                .ProtocolSubmitBuffer,
            (*RequestMsg)
                .__bindgen_anon_1
                .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                .__bindgen_anon_1
                .Request
                .SubmitBufferLength,
            core::ptr::addr_of_mut!(
                (*RequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                    .__bindgen_anon_1
                    .Reply
                    .ProtocolReturnBuffer
            ),
            core::ptr::addr_of_mut!(
                (*RequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                    .__bindgen_anon_1
                    .Reply
                    .ReturnBufferLength
            ),
            core::ptr::addr_of_mut!(
                (*RequestMsg)
                    .__bindgen_anon_1
                    .__bindgen_anon_1.__bindgen_anon_1.CallAuthenticationPackage
                    .__bindgen_anon_1
                    .Reply
                    .ProtocolStatus
            ),
        );
    }

    if !NT_SUCCESS(Status) {
        TRACE!(
            b"Package->LsaApCallPackage() failed (Status 0x%08lx)\n\0".as_ptr() as *const i8,
            Status,
        );
    }

    if !LocalBuffer.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, LocalBuffer);
    }

    Status
}

unsafe fn LsapCopyLocalGroups(
    LogonContext: PLSAP_LOGON_CONTEXT,
    ClientGroups: PTOKEN_GROUPS,
    ClientGroupsCount: ULONG,
    token_groups: *mut PTOKEN_GROUPS,
) -> NTSTATUS {

    let local_groups_length = core::mem::size_of::<TOKEN_GROUPS>() as ULONG
        + ((ClientGroupsCount - 1) as ULONG)
            * core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG;

    let local_groups: PTOKEN_GROUPS = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        local_groups_length as SIZE_T,
    ) as PTOKEN_GROUPS;
    if local_groups.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    let mut sid_header: PSID = core::ptr::null_mut();
    let mut allocated_sids: ULONG = 0;
    let mut status: NTSTATUS;

    'done: {
        status = NtReadVirtualMemory(
            (*LogonContext).ClientProcessHandle,
            ClientGroups as *mut c_void,
            local_groups as *mut c_void,
            local_groups_length as SIZE_T,
            core::ptr::null_mut(),
        );
        if status < 0 {
            break 'done;
        }

        let sid_header_length = RtlLengthRequiredSid(0);
        sid_header = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY,
            sid_header_length as SIZE_T,
        ) as PSID;
        if sid_header.is_null() {
            status = STATUS_INSUFFICIENT_RESOURCES;
            break 'done;
        }

        let groups_ptr =
            core::ptr::addr_of_mut!((*local_groups).Groups) as *mut SID_AND_ATTRIBUTES;

        for i in 0..ClientGroupsCount {
            let current_group = &mut *groups_ptr.add(i as usize);
            let src_sid = current_group.Sid;

            status = NtReadVirtualMemory(
                (*LogonContext).ClientProcessHandle,
                src_sid as *mut c_void,
                sid_header as *mut c_void,
                sid_header_length as SIZE_T,
                core::ptr::null_mut(),
            );
            if status < 0 {
                break 'done;
            }

            let sid_length = RtlLengthSid(sid_header);

            let dst_sid = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY,
                sid_length as SIZE_T,
            ) as PSID;
            if dst_sid.is_null() {
                status = STATUS_INSUFFICIENT_RESOURCES;
                break 'done;
            }

            status = NtReadVirtualMemory(
                (*LogonContext).ClientProcessHandle,
                src_sid as *mut c_void,
                dst_sid as *mut c_void,
                sid_length as SIZE_T,
                core::ptr::null_mut(),
            );
            if status < 0 {
                RtlFreeHeap(RtlGetProcessHeap(), 0, dst_sid as *mut c_void);
                break 'done;
            }

            current_group.Sid = dst_sid;
            allocated_sids += 1;
        }

        *token_groups = local_groups;
    }

    if !sid_header.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, sid_header as *mut c_void);
    }

    if status < 0 {
        if !local_groups.is_null() {
            let groups_ptr =
                core::ptr::addr_of_mut!((*local_groups).Groups) as *mut SID_AND_ATTRIBUTES;
            for i in 0..allocated_sids {
                let sid = (*groups_ptr.add(i as usize)).Sid;
                RtlFreeHeap(RtlGetProcessHeap(), 0, sid as *mut c_void);
            }
            RtlFreeHeap(RtlGetProcessHeap(), 0, local_groups as *mut c_void);
        }
    }

    status
}

unsafe fn LsapAddLocalGroups(
    TokenInformation: PVOID,
    TokenInformationType: LSA_TOKEN_INFORMATION_TYPE,
    LocalGroups: PTOKEN_GROUPS,
) -> NTSTATUS {
    let TokenInfo1: PLSA_TOKEN_INFORMATION_V1;
    let Groups: PTOKEN_GROUPS;
    let Length: ULONG;
    let mut i: ULONG;
    let mut j: ULONG;

    if LocalGroups.is_null() || (*LocalGroups).GroupCount == 0 {
        return STATUS_SUCCESS;
    }

    if TokenInformationType == LsaTokenInformationV1 {
        TokenInfo1 = TokenInformation as PLSA_TOKEN_INFORMATION_V1;
        if !(*TokenInfo1).Groups.is_null() {
            Length = (core::mem::size_of::<TOKEN_GROUPS>()
                + ((*LocalGroups).GroupCount as usize
                    + (*(*TokenInfo1).Groups).GroupCount as usize
                    - ANYSIZE_ARRAY as usize)
                    * core::mem::size_of::<SID_AND_ATTRIBUTES>()) as ULONG;
            Groups = RtlAllocateHeap(
                RtlGetProcessHeap() as HANDLE,
                HEAP_ZERO_MEMORY,
                Length as SIZE_T,
            ) as PTOKEN_GROUPS;
            if Groups.is_null() {
                ERR!("Group buffer allocation failed!\n");
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            (*Groups).GroupCount =
                (*LocalGroups).GroupCount + (*(*TokenInfo1).Groups).GroupCount;
            let dest_base: *mut SID_AND_ATTRIBUTES =
                core::ptr::addr_of_mut!((*Groups).Groups) as *mut SID_AND_ATTRIBUTES;
            let src_base: *mut SID_AND_ATTRIBUTES =
                core::ptr::addr_of_mut!((*(*TokenInfo1).Groups).Groups)
                    as *mut SID_AND_ATTRIBUTES;
            i = 0;
            while i < (*(*TokenInfo1).Groups).GroupCount {
                let dest = dest_base.add(i as usize);
                let src = src_base.add(i as usize);
                (*dest).Sid = (*src).Sid;
                (*dest).Attributes = (*src).Attributes;
                i = i.wrapping_add(1);
            }
            let local_base: *mut SID_AND_ATTRIBUTES =
                core::ptr::addr_of_mut!((*LocalGroups).Groups) as *mut SID_AND_ATTRIBUTES;
            j = 0;
            while j < (*LocalGroups).GroupCount {
                let dest = dest_base.add(i as usize);
                let src_local = local_base.add(j as usize);
                (*dest).Sid = (*src_local).Sid;
                (*dest).Attributes = (*src_local).Attributes;
                (*src_local).Sid = core::ptr::null_mut();
                i = i.wrapping_add(1);
                j = j.wrapping_add(1);
            }
            RtlFreeHeap(
                RtlGetProcessHeap() as HANDLE,
                0,
                (*TokenInfo1).Groups as PVOID,
            );
            (*TokenInfo1).Groups = Groups;
        } else {
            Length = (core::mem::size_of::<TOKEN_GROUPS>()
                + ((*LocalGroups).GroupCount as usize - ANYSIZE_ARRAY as usize)
                    * core::mem::size_of::<SID_AND_ATTRIBUTES>()) as ULONG;
            Groups = RtlAllocateHeap(
                RtlGetProcessHeap() as HANDLE,
                HEAP_ZERO_MEMORY,
                Length as SIZE_T,
            ) as PTOKEN_GROUPS;
            if Groups.is_null() {
                ERR!("Group buffer allocation failed!\n");
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            (*Groups).GroupCount = (*LocalGroups).GroupCount;
            let dest_base: *mut SID_AND_ATTRIBUTES =
                core::ptr::addr_of_mut!((*Groups).Groups) as *mut SID_AND_ATTRIBUTES;
            let local_base: *mut SID_AND_ATTRIBUTES =
                core::ptr::addr_of_mut!((*LocalGroups).Groups) as *mut SID_AND_ATTRIBUTES;
            i = 0;
            while i < (*LocalGroups).GroupCount {
                let dest = dest_base.add(i as usize);
                let src = local_base.add(i as usize);
                (*dest).Sid = (*src).Sid;
                (*dest).Attributes = (*src).Attributes;
                i = i.wrapping_add(1);
            }
            (*TokenInfo1).Groups = Groups;
        }
    } else {
        FIXME!(
            "TokenInformationType %d is not supported!\n",
            TokenInformationType as u32,
        );
        return STATUS_NOT_IMPLEMENTED;
    }

    STATUS_SUCCESS
}

unsafe fn LsapAddDefaultGroups(
    TokenInformation: PVOID,
    TokenInformationType: LSA_TOKEN_INFORMATION_TYPE,
    LogonType: SECURITY_LOGON_TYPE,
) -> NTSTATUS {
    if TokenInformationType == LsaTokenInformationV1 {
        let token_info1: PLSA_TOKEN_INFORMATION_V1 = TokenInformation as PLSA_TOKEN_INFORMATION_V1;
        let mut groups_ptr: PTOKEN_GROUPS = core::ptr::null_mut();

        if !(*token_info1).Groups.is_null() {
            let group_count = (*(*token_info1).Groups).GroupCount as usize;
            let length = core::mem::size_of::<TOKEN_GROUPS>()
                + (group_count + 2 - (ANYSIZE_ARRAY as usize))
                    * core::mem::size_of::<SID_AND_ATTRIBUTES>();
            groups_ptr = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY as ULONG,
                length as SIZE_T,
            ) as PTOKEN_GROUPS;
            if groups_ptr.is_null() {
                ERR!("Group buffer allocation failed!\n");
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            (*groups_ptr).GroupCount = (*(*token_info1).Groups).GroupCount;
            for i in 0..group_count {
                let src = &raw const (*(*token_info1).Groups).Groups as *const SID_AND_ATTRIBUTES;
                let dst = &raw mut (*groups_ptr).Groups as *mut SID_AND_ATTRIBUTES;
                (*dst.offset(i as isize)).Sid = (*src.offset(i as isize)).Sid;
                (*dst.offset(i as isize)).Attributes = (*src.offset(i as isize)).Attributes;
            }

            RtlFreeHeap(RtlGetProcessHeap(), 0, (*token_info1).Groups as PVOID);
            (*token_info1).Groups = groups_ptr;
        } else {
            let length = core::mem::size_of::<TOKEN_GROUPS>()
                + (2 - ANYSIZE_ARRAY) as usize * core::mem::size_of::<SID_AND_ATTRIBUTES>();
            groups_ptr = RtlAllocateHeap(
                RtlGetProcessHeap(),
                HEAP_ZERO_MEMORY as ULONG,
                length as SIZE_T,
            ) as PTOKEN_GROUPS;
            if groups_ptr.is_null() {
                ERR!("Group buffer allocation failed!\n");
                return STATUS_INSUFFICIENT_RESOURCES;
            }
            (*token_info1).Groups = groups_ptr;
        }

        // Append World SID
        let world_sid_ptr: PSID = LsapWorldSid;
        let sid_length = RtlLengthSid(world_sid_ptr) as usize;
        let groups_array = &raw mut (*groups_ptr).Groups as *mut SID_AND_ATTRIBUTES;
        let idx = (*groups_ptr).GroupCount as isize;
        (*groups_array.offset(idx)).Sid = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY as ULONG,
            sid_length as SIZE_T,
        ) as PSID;
        if (*groups_array.offset(idx)).Sid.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        core::ptr::copy_nonoverlapping(
            world_sid_ptr as *const core::ffi::c_void,
            (*groups_array.offset(idx)).Sid as *mut core::ffi::c_void,
            sid_length,
        );
        (*groups_array.offset(idx)).Attributes =
            (SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY) as u32;
        (*groups_ptr).GroupCount = (*groups_ptr).GroupCount.wrapping_add(1);

        // Append logon type SID
        let src_sid: PSID = match LogonType {
            2 => LsapInteractiveSid,
            3 => LsapNetworkSid,
            4 => LsapBatchSid,
            5 => LsapServiceSid,
            _ => {
                FIXME!(
                    "LogonType %d is not supported!\n",
                    LogonType as core::ffi::c_uint,
                );
                return STATUS_NOT_IMPLEMENTED;
            }
        };

        let sid_length = RtlLengthSid(src_sid) as usize;
        let idx2 = (*groups_ptr).GroupCount as isize;
        (*groups_array.offset(idx2)).Sid = RtlAllocateHeap(
            RtlGetProcessHeap(),
            HEAP_ZERO_MEMORY as ULONG,
            sid_length as SIZE_T,
        ) as PSID;
        if (*groups_array.offset(idx2)).Sid.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        core::ptr::copy_nonoverlapping(
            src_sid as *const core::ffi::c_void,
            (*groups_array.offset(idx2)).Sid as *mut core::ffi::c_void,
            sid_length,
        );
        (*groups_array.offset(idx2)).Attributes =
            (SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY) as u32;
        (*groups_ptr).GroupCount = (*groups_ptr).GroupCount.wrapping_add(1);
    } else {
        FIXME!(
            "TokenInformationType %d is not supported!\n",
            TokenInformationType as core::ffi::c_uint,
        );
        return STATUS_NOT_IMPLEMENTED;
    }

    STATUS_SUCCESS
}

unsafe fn LsapAppendSidToGroups(
    token_groups: *mut PTOKEN_GROUPS,
    DomainSid: PSID,
    RelativeId: ULONG,
) -> NTSTATUS {
    let Groups: PTOKEN_GROUPS;
    let Sid: PSID;
    let Length: ULONG;
    let mut i: ULONG;

    Sid = LsapAppendRidToSid(DomainSid, RelativeId);
    if Sid.is_null() {
        if (*__wine_dbch___default).flags as ::core::ffi::c_int
            & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int)
            != 0
        {
            ros_dbg_log(
                __WINE_DBCL_ERR,
                __wine_dbch___default,
                b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authpackage.c\0"
                    .as_ptr() as *const ::core::ffi::c_char,
                b"LsapAppendSidToGroups\0".as_ptr() as *const ::core::ffi::c_char,
                964 as ::core::ffi::c_int,
                b"Group SID creation failed!\n\0".as_ptr() as *const ::core::ffi::c_char,
            );
        }
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    if (*token_groups).is_null() {
        Length = (::core::mem::size_of::<TOKEN_GROUPS>() as ULONG).wrapping_add(
            ((1 as ULONG).wrapping_sub(ANYSIZE_ARRAY as ULONG))
                .wrapping_mul(::core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG),
        );

        Groups = RtlAllocateHeap(
            RtlGetProcessHeap() as *mut ::core::ffi::c_void,
            HEAP_ZERO_MEMORY as ULONG,
            Length as SIZE_T,
        ) as PTOKEN_GROUPS;

        if Groups.is_null() {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int)
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authpackage.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"LsapAppendSidToGroups\0".as_ptr() as *const ::core::ffi::c_char,
                    976 as ::core::ffi::c_int,
                    b"Group buffer allocation failed!\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Groups).GroupCount = 1 as DWORD;
        (*(core::ptr::addr_of_mut!((*Groups).Groups) as *mut SID_AND_ATTRIBUTES).offset(0))
            .Sid = Sid;
        (*(core::ptr::addr_of_mut!((*Groups).Groups) as *mut SID_AND_ATTRIBUTES).offset(0))
            .Attributes = (SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY)
            as DWORD;

        *token_groups = Groups;
    } else {
        i = 0 as ULONG;
        while i < (**token_groups).GroupCount {
            if RtlEqualSid(
                (*(core::ptr::addr_of_mut!((**token_groups).Groups) as *mut SID_AND_ATTRIBUTES)
                    .offset(i as isize))
                    .Sid,
                Sid,
            ) != 0
            {
                RtlFreeHeap(
                    RtlGetProcessHeap() as HANDLE,
                    0 as ULONG,
                    Sid as PVOID,
                );
                return STATUS_SUCCESS;
            }
            i = i.wrapping_add(1);
        }

        Length = (::core::mem::size_of::<TOKEN_GROUPS>() as ULONG).wrapping_add(
            (((**token_groups).GroupCount as ULONG)
                .wrapping_add(1 as ULONG)
                .wrapping_sub(ANYSIZE_ARRAY as ULONG))
            .wrapping_mul(::core::mem::size_of::<SID_AND_ATTRIBUTES>() as ULONG),
        );

        Groups = RtlAllocateHeap(
            RtlGetProcessHeap() as *mut ::core::ffi::c_void,
            HEAP_ZERO_MEMORY as ULONG,
            Length as SIZE_T,
        ) as PTOKEN_GROUPS;

        if Groups.is_null() {
            if (*__wine_dbch___default).flags as ::core::ffi::c_int
                & ((1 as ::core::ffi::c_int) << __WINE_DBCL_ERR as ::core::ffi::c_int)
                != 0
            {
                ros_dbg_log(
                    __WINE_DBCL_ERR,
                    __wine_dbch___default,
                    b"/home/curi/Projects/os-playground/vibeos-wt-kernel32/dll/win32/lsasrv/authpackage.c\0"
                        .as_ptr() as *const ::core::ffi::c_char,
                    b"LsapAppendSidToGroups\0".as_ptr() as *const ::core::ffi::c_char,
                    1005 as ::core::ffi::c_int,
                    b"Group buffer allocation failed!\n\0".as_ptr() as *const ::core::ffi::c_char,
                );
            }
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*Groups).GroupCount = (**token_groups).GroupCount;

        i = 0 as ULONG;
        while i < (**token_groups).GroupCount {
            let src_sid_ptr = (*(core::ptr::addr_of_mut!((**token_groups).Groups)
                as *mut SID_AND_ATTRIBUTES)
                .offset(i as isize))
            .Sid;
            let src_attr = (*(core::ptr::addr_of_mut!((**token_groups).Groups)
                as *mut SID_AND_ATTRIBUTES)
                .offset(i as isize))
            .Attributes;

            (*(core::ptr::addr_of_mut!((*Groups).Groups) as *mut SID_AND_ATTRIBUTES)
                .offset(i as isize))
                .Sid = src_sid_ptr;
            (*(core::ptr::addr_of_mut!((*Groups).Groups) as *mut SID_AND_ATTRIBUTES)
                .offset(i as isize))
                .Attributes = src_attr;

            i = i.wrapping_add(1);
        }

        (*(core::ptr::addr_of_mut!((*Groups).Groups) as *mut SID_AND_ATTRIBUTES)
            .offset((*Groups).GroupCount as isize))
            .Sid = Sid;
        (*(core::ptr::addr_of_mut!((*Groups).Groups) as *mut SID_AND_ATTRIBUTES)
            .offset((*Groups).GroupCount as isize))
            .Attributes = (SE_GROUP_ENABLED | SE_GROUP_ENABLED_BY_DEFAULT | SE_GROUP_MANDATORY)
            as DWORD;

        (*Groups).GroupCount = (*Groups).GroupCount.wrapping_add(1);

        RtlFreeHeap(
            RtlGetProcessHeap() as HANDLE,
            0 as ULONG,
            *token_groups as PVOID,
        );

        *token_groups = Groups;
    }

    return STATUS_SUCCESS;
}

unsafe fn LsapAddSamGroups(
    TokenInformation: PVOID,
    TokenInformationType: LSA_TOKEN_INFORMATION_TYPE,
) -> NTSTATUS {
    let TokenInfo1: *mut _LSA_TOKEN_INFORMATION_V1;
    let mut ServerHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut BuiltinDomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut AccountDomainHandle: SAMPR_HANDLE = core::ptr::null_mut();
    let mut SidArray: SAMPR_PSID_ARRAY = SAMPR_PSID_ARRAY {
        Count: 0,
        Sids: core::ptr::null_mut(),
    };
    let mut BuiltinMembership: SAMPR_ULONG_ARRAY = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut AccountMembership: SAMPR_ULONG_ARRAY = SAMPR_ULONG_ARRAY {
        Count: 0,
        Element: core::ptr::null_mut(),
    };
    let mut i: u32;
    let mut Status: NTSTATUS;

    if TokenInformationType != LsaTokenInformationV1 {
        return STATUS_SUCCESS;
    }
    TokenInfo1 = TokenInformation as *mut _LSA_TOKEN_INFORMATION_V1;
    SidArray.Count = (*(*TokenInfo1).Groups).GroupCount + 1;
    SidArray.Sids = RtlAllocateHeap(
        RtlGetProcessHeap(),
        HEAP_ZERO_MEMORY,
        (SidArray.Count as SIZE_T) * core::mem::size_of::<PRPC_SID>() as SIZE_T,
    ) as PSAMPR_SID_INFORMATION;
    if SidArray.Sids.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }
    (*SidArray.Sids.add(0)).SidPointer = (*TokenInfo1).User.User.Sid as PRPC_SID;
    i = 0;
    while i < (*(*TokenInfo1).Groups).GroupCount {
        (*SidArray.Sids.add((i + 1) as usize)).SidPointer =
            (*(core::ptr::addr_of_mut!((*(*TokenInfo1).Groups).Groups) as *mut SID_AND_ATTRIBUTES)
                .add(i as usize))
            .Sid as PRPC_SID;
        i += 1;
    }
    BuiltinMembership.Element = core::ptr::null_mut();
    AccountMembership.Element = core::ptr::null_mut();

    'done: {
        Status = SamIConnect(
            core::ptr::null_mut(),
            &raw mut ServerHandle,
            (SAM_SERVER_CONNECT | SAM_SERVER_LOOKUP_DOMAIN) as ACCESS_MASK,
            FALSE as BOOLEAN,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SamIConnect failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_GET_ALIAS_MEMBERSHIP as ACCESS_MASK,
            BuiltinDomainSid as *mut _,
            &raw mut BuiltinDomainHandle,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SamrOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        Status = SamrOpenDomain(
            ServerHandle,
            DOMAIN_GET_ALIAS_MEMBERSHIP as ACCESS_MASK,
            AccountDomainSid as *mut _,
            &raw mut AccountDomainHandle,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SamrOpenDomain failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        Status = SamrGetAliasMembership(
            BuiltinDomainHandle,
            &raw mut SidArray as *mut _,
            &raw mut BuiltinMembership,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SamrGetAliasMembership failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        Status = SamrGetAliasMembership(
            AccountDomainHandle,
            &raw mut SidArray as *mut _,
            &raw mut AccountMembership,
        );
        if !NT_SUCCESS(Status) {
            TRACE!(
                b"SamrGetAliasMembership failed (Status %08lx)\n\0".as_ptr() as *const i8,
                Status,
            );
            break 'done;
        }

        TRACE!(
            b"Builtin Memberships: %lu\n\0".as_ptr() as *const i8,
            BuiltinMembership.Count,
        );
        i = 0;
        while i < BuiltinMembership.Count {
            let elem = *BuiltinMembership.Element.add(i as usize);
            TRACE!(
                b"RID %lu: %lu (0x%lx)\n\0".as_ptr() as *const i8,
                i,
                elem,
                elem,
            );
            Status = LsapAppendSidToGroups(
                &raw mut (*TokenInfo1).Groups,
                BuiltinDomainSid as PSID,
                elem,
            );
            if !NT_SUCCESS(Status) {
                TRACE!(
                    b"LsapAppendSidToGroups failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
                break 'done;
            }
            i += 1;
        }

        TRACE!(
            b"Account Memberships: %lu\n\0".as_ptr() as *const i8,
            AccountMembership.Count,
        );
        i = 0;
        while i < AccountMembership.Count {
            let elem = *AccountMembership.Element.add(i as usize);
            TRACE!(
                b"RID %lu: %lu (0x%lx)\n\0".as_ptr() as *const i8,
                i,
                elem,
                elem,
            );
            Status = LsapAppendSidToGroups(
                &raw mut (*TokenInfo1).Groups,
                AccountDomainSid as PSID,
                elem,
            );
            if !NT_SUCCESS(Status) {
                TRACE!(
                    b"LsapAppendSidToGroups failed (Status %08lx)\n\0".as_ptr() as *const i8,
                    Status,
                );
                break 'done;
            }
            i += 1;
        }
    }

    RtlFreeHeap(RtlGetProcessHeap(), 0, SidArray.Sids as PVOID);
    if !AccountMembership.Element.is_null() {
        SamIFree_SAMPR_ULONG_ARRAY(&raw mut AccountMembership);
    }
    if !BuiltinMembership.Element.is_null() {
        SamIFree_SAMPR_ULONG_ARRAY(&raw mut BuiltinMembership);
    }
    if !AccountDomainHandle.is_null() {
        SamrCloseHandle(&raw mut AccountDomainHandle);
    }
    if !BuiltinDomainHandle.is_null() {
        SamrCloseHandle(&raw mut BuiltinDomainHandle);
    }
    if !ServerHandle.is_null() {
        SamrCloseHandle(&raw mut ServerHandle);
    }
    STATUS_SUCCESS
}

unsafe fn LsapSetTokenOwner(
    TokenInformation: PVOID,
    TokenInformationType: LSA_TOKEN_INFORMATION_TYPE,
) -> NTSTATUS {
    let TokenInfo1: *mut _LSA_TOKEN_INFORMATION_V1;
    let mut OwnerSid: PSID_AND_ATTRIBUTES;
    let Length: u32;

    if TokenInformationType == _LSA_TOKEN_INFORMATION_TYPE_LsaTokenInformationV1 {
        TokenInfo1 = TokenInformation as *mut _LSA_TOKEN_INFORMATION_V1;

        if !(*TokenInfo1).Owner.Owner.is_null() {
            return STATUS_SUCCESS;
        }

        OwnerSid = &raw mut (*TokenInfo1).User.User as PSID_AND_ATTRIBUTES;

        let groups_base = &raw mut (*(*TokenInfo1).Groups).Groups as *mut _SID_AND_ATTRIBUTES;
        let group_count = (*(*TokenInfo1).Groups).GroupCount;
        let mut i: u32 = 0;
        while i < group_count {
            let sid = (*groups_base.add(i as usize)).Sid;
            if EqualSid(sid, LsapAdministratorsSid) != 0 {
                OwnerSid = groups_base.add(i as usize);
                break;
            }
            i += 1;
        }

        Length = RtlLengthSid((*OwnerSid).Sid);
        (*TokenInfo1).Owner.Owner = DispatchTable
            .AllocateLsaHeap
            .expect("non-null function pointer")(Length) as PSID;
        if (*TokenInfo1).Owner.Owner.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        core::ptr::copy_nonoverlapping(
            (*OwnerSid).Sid as *const u8,
            (*TokenInfo1).Owner.Owner as *mut u8,
            Length as usize,
        );
        (*OwnerSid).Attributes |= SE_GROUP_OWNER as u32;
    }

    STATUS_SUCCESS
}

unsafe fn LsapAddTokenDefaultDacl(
    TokenInformation: PVOID,
    TokenInformationType: LSA_TOKEN_INFORMATION_TYPE,
) -> NTSTATUS {
    if TokenInformationType as core::ffi::c_uint
        == _LSA_TOKEN_INFORMATION_TYPE_LsaTokenInformationV1 as core::ffi::c_uint
    {
        let TokenInfo1: PLSA_TOKEN_INFORMATION_V1 =
            TokenInformation as PLSA_TOKEN_INFORMATION_V1;

        if !(*TokenInfo1).DefaultDacl.DefaultDacl.is_null() {
            return STATUS_SUCCESS;
        }

        let Length: ULONG = (core::mem::size_of::<ACL>()
            + 2 * core::mem::size_of::<ACCESS_ALLOWED_ACE>()
            + RtlLengthSid((*TokenInfo1).Owner.Owner) as usize
            + RtlLengthSid(LsapLocalSystemSid) as usize) as ULONG;

        let Dacl: PACL = DispatchTable
            .AllocateLsaHeap
            .expect("non-null function pointer")(Length) as PACL;

        if Dacl.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        RtlCreateAcl(Dacl, Length, ACL_REVISION as ULONG);

        RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            GENERIC_ALL as ACCESS_MASK,
            (*TokenInfo1).Owner.Owner,
        );

        /* SID: S-1-5-18 */
        RtlAddAccessAllowedAce(
            Dacl,
            ACL_REVISION as ULONG,
            GENERIC_ALL as ACCESS_MASK,
            LsapLocalSystemSid,
        );

        (*TokenInfo1).DefaultDacl.DefaultDacl = Dacl;
    }

    STATUS_SUCCESS
}

unsafe fn LsapAddPrivilegeToTokenPrivileges(
    token_privileges: *mut PTOKEN_PRIVILEGES,
    Privilege: PLSAPR_LUID_AND_ATTRIBUTES,
) -> NTSTATUS {
    let status: NTSTATUS = STATUS_SUCCESS;
    let local: PTOKEN_PRIVILEGES;
    if (*token_privileges).is_null() {
        let length = core::mem::size_of::<TOKEN_PRIVILEGES>()
            + ((1usize - ANYSIZE_ARRAY as usize) * core::mem::size_of::<LUID_AND_ATTRIBUTES>());
        local = RtlAllocateHeap(RtlGetProcessHeap(), 0u32, length as SIZE_T) as PTOKEN_PRIVILEGES;
        if local.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        (*local).PrivilegeCount = 1;
        let priv_ptr = (*local).Privileges.as_mut_ptr();
        (*priv_ptr).Luid = core::mem::transmute((*Privilege).Luid);
        (*priv_ptr).Attributes = (*Privilege).Attributes;
    } else {
        let token_priv_count = (**token_privileges).PrivilegeCount;
        let old_priv_ptr = (**token_privileges).Privileges.as_ptr();
        let mut i: u32 = 0;
        while i < token_priv_count {
            let old_luid = &(*old_priv_ptr.add(i as usize)).Luid;
            let priv_luid = &(*Privilege).Luid;
            if old_luid.HighPart == priv_luid.HighPart && old_luid.LowPart == priv_luid.LowPart {
                return STATUS_SUCCESS;
            }
            i = i.wrapping_add(1);
        }
        let length = core::mem::size_of::<TOKEN_PRIVILEGES>()
            + ((token_priv_count as usize + 1usize - ANYSIZE_ARRAY as usize)
                * core::mem::size_of::<LUID_AND_ATTRIBUTES>());
        local = RtlAllocateHeap(RtlGetProcessHeap(), 0u32, length as SIZE_T) as PTOKEN_PRIVILEGES;
        if local.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }
        (*local).PrivilegeCount = token_priv_count + 1;
        let new_priv_ptr = (*local).Privileges.as_mut_ptr();
        i = 0;
        while i < token_priv_count {
            (*new_priv_ptr.add(i as usize)).Luid =
                (*old_priv_ptr.add(i as usize)).Luid;
            (*new_priv_ptr.add(i as usize)).Attributes =
                (*old_priv_ptr.add(i as usize)).Attributes;
            i = i.wrapping_add(1);
        }
        (*new_priv_ptr.add(token_priv_count as usize)).Luid = core::mem::transmute((*Privilege).Luid);
        (*new_priv_ptr.add(token_priv_count as usize)).Attributes = (*Privilege).Attributes;
        RtlFreeHeap(RtlGetProcessHeap(), 0, *token_privileges as _);
    }
    *token_privileges = local;
    status
}

unsafe fn LsapSetPrivileges(
    TokenInformation: PVOID,
    TokenInformationType: LSA_TOKEN_INFORMATION_TYPE,
) -> NTSTATUS {
    let TokenInfo1: PLSA_TOKEN_INFORMATION_V1;
    let mut PolicyHandle: LSAPR_HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut AccountHandle: LSAPR_HANDLE = core::ptr::null_mut::<core::ffi::c_void>();
    let mut Privileges: PLSAPR_PRIVILEGE_SET = core::ptr::null_mut::<_LSAPR_PRIVILEGE_SET>();
    let mut i: ULONG;
    let mut j: ULONG;
    let mut Status: NTSTATUS;

    if TokenInformationType as u32 == LsaTokenInformationV1 as u32 {
        TokenInfo1 = TokenInformation as PLSA_TOKEN_INFORMATION_V1;

        Status = LsarOpenPolicy(
            core::ptr::null_mut::<WCHAR>(),
            core::ptr::null_mut::<core::ffi::c_void>() as PLSAPR_OBJECT_ATTRIBUTES,
            0,
            &raw mut PolicyHandle,
        );
        if Status < 0 {
            return Status;
        }

        i = 0;
        while i < (*(*TokenInfo1).Groups).GroupCount {
            Status = LsarOpenAccount(
                PolicyHandle,
                (&raw mut (*(*TokenInfo1).Groups).Groups as *mut SID_AND_ATTRIBUTES)
                    .offset(i as isize)
                    .as_ref()
                    .unwrap()
                    .Sid as PRPC_SID,
                ACCOUNT_VIEW,
                &raw mut AccountHandle,
            );
            if Status >= 0 {
                Status = LsarEnumeratePrivilegesAccount(AccountHandle, &raw mut Privileges);
                if Status >= 0 {
                    j = 0;
                    while j < (*Privileges).PrivilegeCount {
                        Status = LsapAddPrivilegeToTokenPrivileges(
                            &raw mut (*TokenInfo1).Privileges,
                            (&raw mut (*Privileges).Privilege as *mut LSAPR_LUID_AND_ATTRIBUTES)
                                .offset(j as isize) as PLSAPR_LUID_AND_ATTRIBUTES,
                        );
                        if Status < 0 {
                            LsaIFree_LSAPR_PRIVILEGE_SET(Privileges);
                            LsarClose(&raw mut AccountHandle);
                            LsarClose(&raw mut PolicyHandle);
                            return Status;
                        }
                        j = j.wrapping_add(1);
                    }
                    LsaIFree_LSAPR_PRIVILEGE_SET(Privileges);
                    Privileges = core::ptr::null_mut();
                }
                LsarClose(&raw mut AccountHandle);
            }
            i = i.wrapping_add(1);
        }
        LsarClose(&raw mut PolicyHandle);

        if !(*TokenInfo1).Privileges.is_null() {
            i = 0;
            while i < (*(*TokenInfo1).Privileges).PrivilegeCount {
                let attrib = &raw mut (*(*TokenInfo1).Privileges).Privileges as *mut LUID_AND_ATTRIBUTES;
                let luid = &(*attrib.offset(i as isize)).Luid;
                if (luid.HighPart == SeChangeNotifyPrivilege.HighPart
                    && luid.LowPart == SeChangeNotifyPrivilege.LowPart)
                    || (luid.HighPart == SeCreateGlobalPrivilege.HighPart
                        && luid.LowPart == SeCreateGlobalPrivilege.LowPart)
                    || (luid.HighPart == SeImpersonatePrivilege.HighPart
                        && luid.LowPart == SeImpersonatePrivilege.LowPart)
                {
                    let attrs = &raw mut (*attrib.offset(i as isize)).Attributes;
                    *attrs |= (SE_PRIVILEGE_ENABLED | SE_PRIVILEGE_ENABLED_BY_DEFAULT) as ULONG;
                }
                i = i.wrapping_add(1);
            }
        }
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn LsapLogonUser(
    RequestMsg: PLSA_API_MSG,
    LogonContext: PLSAP_LOGON_CONTEXT,
) -> NTSTATUS {
    let Package: PAUTH_PACKAGE;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Qos: SECURITY_QUALITY_OF_SERVICE = core::mem::zeroed();
    let mut TokenInformationType: LSA_TOKEN_INFORMATION_TYPE = LsaTokenInformationNull;
    let mut TokenInformation: PVOID = core::ptr::null_mut();
    let mut TokenInfo0: PLSA_TOKEN_INFORMATION_NULL;
    let mut TokenInfo1: PLSA_TOKEN_INFORMATION_V1 = core::ptr::null_mut();
    let mut AccountName: PUNICODE_STRING = core::ptr::null_mut();
    let mut AuthenticatingAuthority: PUNICODE_STRING = core::ptr::null_mut();
    let mut MachineName: PUNICODE_STRING = core::ptr::null_mut();
    let mut LocalAuthInfo: PVOID = core::ptr::null_mut();
    let mut LocalGroups: PTOKEN_GROUPS = core::ptr::null_mut();
    let mut TokenHandle: HANDLE = core::ptr::null_mut();
    let mut i: ULONG;
    let PackageId: ULONG;
    let LogonType: SECURITY_LOGON_TYPE;
    let mut Status: NTSTATUS;
    let mut UserName: PUNICODE_STRING = core::ptr::null_mut();
    let mut LogonDomainName: PUNICODE_STRING = core::ptr::null_mut();

    TRACE!(b"LsapLogonUser(%p %p)\n\0".as_ptr() as *const u8, RequestMsg, LogonContext);

    PackageId = (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationPackage;
    LogonType = (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.LogonType;

    Package = LsapGetAuthenticationPackage(PackageId);
    if Package.is_null() {
        ERR!("LsapGetAuthenticationPackage() failed to find a package\n");
        return STATUS_NO_SUCH_PACKAGE;
    }

    if (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformationLength > 0 {
        LocalAuthInfo = RtlAllocateHeap(
            RtlGetProcessHeap() as HANDLE,
            HEAP_ZERO_MEMORY as ULONG,
            (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformationLength as SIZE_T,
        );
        if LocalAuthInfo.is_null() {
            ERR!("RtlAllocateHeap() failed\n");
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = NtReadVirtualMemory(
            (*LogonContext).ClientProcessHandle,
            (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformation,
            LocalAuthInfo,
            (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformationLength as SIZE_T,
            core::ptr::null_mut(),
        );
        if !NT_SUCCESS(Status) {
            ERR!("NtReadVirtualMemory() failed (Status 0x%08lx)\n", Status);
            RtlFreeHeap(RtlGetProcessHeap(), 0, LocalAuthInfo);
            return Status;
        }
    }

    // We'll use a 'done block' to simplify goto. The LocalGroups copy lives
    // INSIDE the block so its failure path can `break 'done Status` — matching
    // the C original's `goto done` (authpackage.c:1466). (It was previously
    // stranded above the block with a no-op `Status = Status`, so a failed
    // LsapCopyLocalGroups silently fell through into LsaApLogonUser* instead of
    // aborting the logon.)
    let result: NTSTATUS = 'done: {
        if (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.LocalGroupsCount > 0 {
            Status = LsapCopyLocalGroups(
                LogonContext,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.LocalGroups as *mut _ as PTOKEN_GROUPS,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.LocalGroupsCount,
                &mut LocalGroups,
            );
            if !NT_SUCCESS(Status) {
                ERR!("LsapCopyLocalGroups failed (Status 0x%08lx)\n", Status);
                break 'done Status;
            }
            else {
                TRACE!(b"GroupCount: %lu\n\0".as_ptr() as *const u8, (*LocalGroups).GroupCount);
            }
        }

        if (*Package).LsaApLogonUserEx2.is_some() {
            Status = (*Package).LsaApLogonUserEx2.unwrap()(
                LogonContext as PLSA_CLIENT_REQUEST,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.LogonType,
                LocalAuthInfo,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformation,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformationLength,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.ProfileBuffer,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.ProfileBufferLength,
                &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.LogonId as *mut _,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.SubStatus,
                &mut TokenInformationType,
                &mut TokenInformation,
                &mut AccountName,
                &mut AuthenticatingAuthority,
                &mut MachineName,
                core::ptr::null_mut(),   /* PrimaryCredentials */
                core::ptr::null_mut(),   /* SupplementalCredentials */
            );
        } else if (*Package).LsaApLogonUserEx.is_some() {
            Status = (*Package).LsaApLogonUserEx.unwrap()(
                LogonContext as PLSA_CLIENT_REQUEST,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.LogonType,
                LocalAuthInfo,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformation,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformationLength,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.ProfileBuffer,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.ProfileBufferLength,
                &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.LogonId as *mut _,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.SubStatus,
                &mut TokenInformationType,
                &mut TokenInformation,
                &mut AccountName,
                &mut AuthenticatingAuthority,
                &mut MachineName,
            );
        } else {
            Status = (*Package).LsaApLogonUser.unwrap()(
                LogonContext as PLSA_CLIENT_REQUEST,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.LogonType,
                LocalAuthInfo,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformation,
                (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.AuthenticationInformationLength,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.ProfileBuffer,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.ProfileBufferLength,
                &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.LogonId as *mut _,
                &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.SubStatus,
                &mut TokenInformationType,
                &mut TokenInformation,
                &mut AccountName,
                &mut AuthenticatingAuthority,
            );
        }

        if !NT_SUCCESS(Status) {
            ERR!("LsaApLogonUser/Ex/2 failed (Status 0x%08lx)\n", Status);
            break 'done Status;
        }

        if !LocalGroups.is_null() && (*LocalGroups).GroupCount > 0 {
            Status = LsapAddLocalGroups(TokenInformation, TokenInformationType, LocalGroups);
            if !NT_SUCCESS(Status) {
                ERR!("LsapAddLocalGroupsToTokenInfo() failed (Status 0x%08lx)\n", Status);
                break 'done Status;
            }
        }

        Status = LsapAddDefaultGroups(TokenInformation, TokenInformationType, LogonType);
        if !NT_SUCCESS(Status) {
            ERR!("LsapAddDefaultGroups() failed (Status 0x%08lx)\n", Status);
            break 'done Status;
        }

        Status = LsapAddSamGroups(TokenInformation, TokenInformationType);
        if !NT_SUCCESS(Status) {
            ERR!("LsapAddSamGroups() failed (Status 0x%08lx)\n", Status);
            break 'done Status;
        }

        Status = LsapSetTokenOwner(TokenInformation, TokenInformationType);
        if !NT_SUCCESS(Status) {
            ERR!("LsapSetTokenOwner() failed (Status 0x%08lx)\n", Status);
            break 'done Status;
        }

        Status = LsapAddTokenDefaultDacl(TokenInformation, TokenInformationType);
        if !NT_SUCCESS(Status) {
            ERR!("LsapAddTokenDefaultDacl() failed (Status 0x%08lx)\n", Status);
            break 'done Status;
        }

        Status = LsapSetPrivileges(TokenInformation, TokenInformationType);
        if !NT_SUCCESS(Status) {
            ERR!("LsapSetPrivileges() failed (Status 0x%08lx)\n", Status);
            break 'done Status;
        }

        if TokenInformationType == LsaTokenInformationNull {
            let mut token_user: TOKEN_USER = core::mem::zeroed();
            let mut token_primary_group: TOKEN_PRIMARY_GROUP = core::mem::zeroed();
            let mut NoGroups: TOKEN_GROUPS = TOKEN_GROUPS { GroupCount: 0, Groups: [SID_AND_ATTRIBUTES { Sid: core::ptr::null_mut(), Attributes: 0 }; 1] };
            let mut NoPrivileges: TOKEN_PRIVILEGES = TOKEN_PRIVILEGES { PrivilegeCount: 0, Privileges: [LUID_AND_ATTRIBUTES { Luid: LUID { LowPart: 0, HighPart: 0 }, Attributes: 0 }; 1] };

            TokenInfo0 = TokenInformation as PLSA_TOKEN_INFORMATION_NULL;
            token_user.User.Sid = LsapWorldSid;
            token_user.User.Attributes = 0;
            token_primary_group.PrimaryGroup = LsapWorldSid;

            Qos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as u32;
            Qos.ImpersonationLevel = SecurityImpersonation;
            Qos.ContextTrackingMode = SECURITY_STATIC_TRACKING as SECURITY_CONTEXT_TRACKING_MODE;
            Qos.EffectiveOnly = 1u8;

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.ObjectName = core::ptr::null_mut();
            ObjectAttributes.Attributes = 0;
            ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
            ObjectAttributes.SecurityQualityOfService = &mut Qos as *mut _ as PVOID;

            NtCreateToken(
                &mut TokenHandle,
                TOKEN_ALL_ACCESS,
                &mut ObjectAttributes,
                TokenImpersonation,
                &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.LogonId as *mut _,
                &mut (*TokenInfo0).ExpirationTime,
                &mut token_user,
                &mut NoGroups,
                &mut NoPrivileges,
                core::ptr::null_mut(),
                &mut token_primary_group,
                core::ptr::null_mut(),
                &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.SourceContext as *mut _,
            );
        } else if TokenInformationType == LsaTokenInformationV1 {
            let mut NoPrivileges: TOKEN_PRIVILEGES = TOKEN_PRIVILEGES { PrivilegeCount: 0, Privileges: [LUID_AND_ATTRIBUTES{ Luid: LUID { LowPart: 0, HighPart: 0 }, Attributes: 0}; 1] };
            let mut TokenSd: PSECURITY_DESCRIPTOR = core::ptr::null_mut();
            let mut TokenSdSize: ULONG = 0;

            TokenInfo1 = TokenInformation as PLSA_TOKEN_INFORMATION_V1;
            Status = LsapCreateTokenSd(&mut (*TokenInfo1).User, &mut TokenSd, &mut TokenSdSize);
            if !NT_SUCCESS(Status) {
                TokenSd = core::ptr::null_mut();
            }

            Qos.Length = core::mem::size_of::<SECURITY_QUALITY_OF_SERVICE>() as u32;
            Qos.ImpersonationLevel = SecurityImpersonation;
            Qos.ContextTrackingMode = SECURITY_DYNAMIC_TRACKING as SECURITY_CONTEXT_TRACKING_MODE;
            Qos.EffectiveOnly = 0u8;

            ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
            ObjectAttributes.RootDirectory = core::ptr::null_mut();
            ObjectAttributes.ObjectName = core::ptr::null_mut();
            ObjectAttributes.Attributes = 0;
            ObjectAttributes.SecurityDescriptor = TokenSd as PVOID;
            ObjectAttributes.SecurityQualityOfService = &mut Qos as *mut _ as PVOID;

            let token_type = if (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.LogonType == Network { TokenImpersonation } else { TokenPrimary };

            Status = NtCreateToken(
                &mut TokenHandle,
                TOKEN_ALL_ACCESS,
                &mut ObjectAttributes,
                token_type,
                &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.LogonId as *mut _,
                &mut (*TokenInfo1).ExpirationTime,
                &mut (*TokenInfo1).User,
                (*TokenInfo1).Groups,
                if (*TokenInfo1).Privileges.is_null() { &mut NoPrivileges } else { &mut *(*TokenInfo1).Privileges },
                &mut (*TokenInfo1).Owner,
                &mut (*TokenInfo1).PrimaryGroup,
                &mut (*TokenInfo1).DefaultDacl,
                &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Request.SourceContext as *mut _,
            );

            RtlFreeHeap(RtlGetProcessHeap(), 0, TokenSd as PVOID);

            if !NT_SUCCESS(Status) {
                ERR!("NtCreateToken failed (Status 0x%08lx)\n", Status);
                break 'done Status;
            }
        } else {
            FIXME!("TokenInformationType %d is not supported!\n", TokenInformationType as u32);
            Status = STATUS_NOT_IMPLEMENTED;
            break 'done Status;
        }

        if LogonType == Interactive || LogonType == Batch || LogonType == Service {
            UserName = &mut (*(LocalAuthInfo as *mut _MSV1_0_INTERACTIVE_LOGON)).UserName;
            LogonDomainName = &mut (*(LocalAuthInfo as *mut _MSV1_0_INTERACTIVE_LOGON)).LogonDomainName;
        } else {
            FIXME!("LogonType %lu is not supported yet!\n", LogonType as u32);
        }

        Status = LsapSetLogonSessionData(
            &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.LogonId as *mut _,
            LogonType as ULONG,
            UserName,
            LogonDomainName,
            (*TokenInfo1).User.User.Sid,
        );
        if !NT_SUCCESS(Status) {
            ERR!("LsapSetLogonSessionData failed (Status 0x%08lx)\n", Status);
            break 'done Status;
        }

        Status = NtDuplicateObject(
            (-1i32 as LONG_PTR) as HANDLE,   // NtCurrentProcess()
            TokenHandle,
            (*LogonContext).ClientProcessHandle,
            &mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.Token,
            0,
            0,
            (DUPLICATE_SAME_ACCESS | DUPLICATE_SAME_ATTRIBUTES | DUPLICATE_CLOSE_SOURCE) as ULONG,
        );
        if !NT_SUCCESS(Status) {
            ERR!("NtDuplicateObject failed (Status 0x%08lx)\n", Status);
        }

        // Success path; Status will be the result of the duplicate call.
        break 'done Status;
    };

    // Post 'done' processing (error path and cleanup)
    let final_status: NTSTATUS;
    if !NT_SUCCESS(result) {
        final_status = result;
    } else {
        final_status = result;
    }

    // Error handling when status is not success
    if !NT_SUCCESS(final_status) {
        (*Package).LsaApLogonTerminated.unwrap()(
            &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.LogonId as *mut _,
        );
        LsapDeleteLogonSession(
            &raw mut (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.LogonId as *mut _,
        );
        LsapFreeClientBuffer(
            LogonContext as PLSA_CLIENT_REQUEST,
            (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.ProfileBuffer,
        );
        (*RequestMsg).__bindgen_anon_1.__bindgen_anon_1.__bindgen_anon_1.LogonUserW.__bindgen_anon_1.Reply.ProfileBuffer = core::ptr::null_mut();
    }

    if !TokenHandle.is_null() {
        NtClose(TokenHandle);
    }

    // Cleanup LocalGroups
    if !LocalGroups.is_null() {
        i = 0;
        while i < (*LocalGroups).GroupCount {
            // TOKEN_GROUPS.Groups / PRIVILEGE_SET.Privilege are FLEXIBLE arrays
            // (`Groups[ANYSIZE_ARRAY]`, `[size_is(GroupCount)]`), so bindgen's
            // `[T; 1]` panics for every index >= 1.
            if !(*(&raw mut (*LocalGroups).Groups[0]).add(i as usize)).Sid.is_null() {
                RtlFreeHeap(RtlGetProcessHeap(), 0, (*(&raw mut (*LocalGroups).Groups[0]).add(i as usize)).Sid as PVOID);
            }
            i += 1;
        }
        RtlFreeHeap(RtlGetProcessHeap(), 0, LocalGroups as PVOID);
    }

    // Cleanup LocalAuthInfo
    if !LocalAuthInfo.is_null() {
        RtlFreeHeap(RtlGetProcessHeap(), 0, LocalAuthInfo);
    }

    // Cleanup TokenInformation
    if !TokenInformation.is_null() {
        if TokenInformationType == LsaTokenInformationNull {
            TokenInfo0 = TokenInformation as PLSA_TOKEN_INFORMATION_NULL;
            if !TokenInfo0.is_null() {
                if !(*TokenInfo0).Groups.is_null() {
                    i = 0;
                    while i < (*(*TokenInfo0).Groups).GroupCount {
                        if !(*(&raw mut (*(*TokenInfo0).Groups).Groups[0]).add(i as usize)).Sid.is_null() {
                            LsapFreeHeap((*(&raw mut (*(*TokenInfo0).Groups).Groups[0]).add(i as usize)).Sid as PVOID);
                        }
                        i += 1;
                    }
                    LsapFreeHeap((*TokenInfo0).Groups as PVOID);
                }
                LsapFreeHeap(TokenInfo0 as PVOID);
            }
        } else if TokenInformationType == LsaTokenInformationV1 {
            TokenInfo1 = TokenInformation as PLSA_TOKEN_INFORMATION_V1;
            if !TokenInfo1.is_null() {
                if !(*TokenInfo1).User.User.Sid.is_null() {
                    LsapFreeHeap((*TokenInfo1).User.User.Sid as PVOID);
                }
                if !(*TokenInfo1).Groups.is_null() {
                    i = 0;
                    while i < (*(*TokenInfo1).Groups).GroupCount {
                        if !(*(&raw mut (*(*TokenInfo1).Groups).Groups[0]).add(i as usize)).Sid.is_null() {
                            LsapFreeHeap((*(&raw mut (*(*TokenInfo1).Groups).Groups[0]).add(i as usize)).Sid as PVOID);
                        }
                        i += 1;
                    }
                    LsapFreeHeap((*TokenInfo1).Groups as PVOID);
                }
                if !(*TokenInfo1).PrimaryGroup.PrimaryGroup.is_null() {
                    LsapFreeHeap((*TokenInfo1).PrimaryGroup.PrimaryGroup as PVOID);
                }
                if !(*TokenInfo1).Privileges.is_null() {
                    LsapFreeHeap((*TokenInfo1).Privileges as PVOID);
                }
                if !(*TokenInfo1).Owner.Owner.is_null() {
                    LsapFreeHeap((*TokenInfo1).Owner.Owner as PVOID);
                }
                if !(*TokenInfo1).DefaultDacl.DefaultDacl.is_null() {
                    LsapFreeHeap((*TokenInfo1).DefaultDacl.DefaultDacl as PVOID);
                }
                LsapFreeHeap(TokenInfo1 as PVOID);
            }
        } else {
            FIXME!("TokenInformationType %d is not supported!\n", TokenInformationType as u32);
        }
    }

    if !AccountName.is_null() {
        if !(*AccountName).Buffer.is_null() {
            LsapFreeHeap((*AccountName).Buffer as PVOID);
        }
        LsapFreeHeap(AccountName as PVOID);
    }
    if !AuthenticatingAuthority.is_null() {
        if !(*AuthenticatingAuthority).Buffer.is_null() {
            LsapFreeHeap((*AuthenticatingAuthority).Buffer as PVOID);
        }
        LsapFreeHeap(AuthenticatingAuthority as PVOID);
    }
    if !MachineName.is_null() {
        if !(*MachineName).Buffer.is_null() {
            LsapFreeHeap((*MachineName).Buffer as PVOID);
        }
        LsapFreeHeap(MachineName as PVOID);
    }

    TRACE!(b"LsapLogonUser done (Status 0x%08lx)\n\0".as_ptr() as *const u8, final_status);

    final_status
}
