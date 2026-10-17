use crate::prelude_rpc::*;
// One declaration per external symbol: the variant matching the C prototype.
use crate::rpc_stubs::NetrWkstaUserGetInfo;
use ntdll_sys::seh::rpc_try;
use core::ffi::c_void;

// ---- file-scope preamble (standard netapi32 wine-debug channel; TRACE/FIXME reference it) ----
static mut __wine_dbch___default: *mut __wine_debug_channel =
    unsafe { &raw const __wine_dbch_netapi32 as *mut __wine_debug_channel };

static mut __wine_dbch_netapi32: __wine_debug_channel = unsafe {
    __wine_debug_channel {
        flags: WINE_DBG_DEFAULT_FLAGS,
        name: ::core::mem::transmute::<[u8; 15], [::core::ffi::c_char; 15]>(
            *b"netapi32\0\0\0\0\0\0\0",
        ),
    }
};


// ---- MIDL_user_allocate / MIDL_user_free ----
// wksta_new.c spells these lowercase, but rpcndr.h:93-94 does
//     #define midl_user_free     MIDL_user_free
//     #define midl_user_allocate MIDL_user_allocate
// so the symbol the C actually DEFINES is the UPPERCASE one — and that is exactly what all 7
// WIDL-generated `*_c.c` marshallers call (7 refs each to MIDL_user_allocate/MIDL_user_free).
// The translation kept the lowercase pre-processor spelling and omitted #[no_mangle], so after
// the cohab flip both symbols would have been UNDEFINED at link, breaking every MIDL stub.
// Named + exported to match the C. `__RPC_USER` is __stdcall -> extern "system".
#[no_mangle]
pub unsafe extern "system" fn MIDL_user_allocate(len: SIZE_T) -> *mut core::ffi::c_void {
    HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, len)
}

#[no_mangle]
pub unsafe extern "system" fn MIDL_user_free(ptr: *mut core::ffi::c_void) {
    HeapFree(GetProcessHeap(), 0, ptr);
}

// ---- WKSSVC_IDENTIFY_HANDLE_bind ----
#[no_mangle]
pub unsafe extern "system" fn WKSSVC_IDENTIFY_HANDLE_bind(
    pszSystemName: *const u16,
) -> *mut core::ffi::c_void {
    let mut hBinding: *mut core::ffi::c_void = core::ptr::null_mut();
    // C initialises nothing here, but RpcStringBindingComposeW only WRITES this out-param;
    // Rust still requires the binding be initialised before `&mut` is taken.
    let mut pszStringBinding: *mut u16 = core::ptr::null_mut();
    // Reassigned three times below (the C reuses one `RPC_STATUS status`), so it must be `mut`.
    let mut status: i32;

    // Wide string literals
    let ncacn_np: &[u16] = &[
        0x006e, 0x0063, 0x0061, 0x0063, 0x006e, 0x005f, 0x006e, 0x0070, 0x0000,
    ];
    let pipe_wkssvc: &[u16] = &[
        0x005c, 0x0070, 0x0069, 0x0070, 0x0065, 0x005c, 0x0077, 0x006b, 0x0073, 0x0073, 0x0076,
        0x0063, 0x0000,
    ];

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncacn_np.as_ptr() as *mut u16,
        pszSystemName as *mut u16,
        pipe_wkssvc.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if status != 0 {
        // TRACE equivalent omitted
    }

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != 0 {
        // TRACE equivalent omitted
    }

    hBinding
}

// NOTE: the RPC-runtime prototypes (RpcStringBindingComposeW / RpcBindingFromStringBindingW /
// RpcStringFreeW) used to be re-declared LOCALLY here with `*const u16` parameters, shadowing
// the crate-wide declarations. Two declarations of one C symbol with different signatures is
// exactly the drift verify_ffi.py exists to catch, and this module already took RpcBindingFree
// from the prelude — so the local block is gone and all four now come from prelude_common.

// ---- WKSSVC_IDENTIFY_HANDLE_unbind ----
#[no_mangle]
pub unsafe extern "system" fn WKSSVC_IDENTIFY_HANDLE_unbind(
    _pszSystemName: *mut ::core::ffi::c_void,
    mut hBinding: *mut ::core::ffi::c_void,
) {
    let status = RpcBindingFree(&mut hBinding);
    if status != 0 {
    }
}

// ---- WKSSVC_IMPERSONATE_HANDLE_bind ----
#[no_mangle]
pub unsafe extern "system" fn WKSSVC_IMPERSONATE_HANDLE_bind(
    pszSystemName: LPWSTR,
) -> *mut core::ffi::c_void {
    let mut hBinding: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    let mut status: i32;

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        [110u16, 99, 97, 99, 110, 95, 110, 112, 0].as_ptr() as *mut u16,
        pszSystemName,
        [92u16, 112, 105, 112, 101, 92, 119, 107, 115, 115, 118, 99, 0].as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        return core::ptr::null_mut();
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if status != 0 {}

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != 0 {}

    hBinding
}

// ---- WKSSVC_IMPERSONATE_HANDLE_unbind ----
#[no_mangle]
pub unsafe extern "system" fn WKSSVC_IMPERSONATE_HANDLE_unbind(
    pszSystemName: PWSTR,
    mut hBinding: handle_t,
) {
    let status = RpcBindingFree(&mut hBinding as *mut handle_t);
    let _ = status;
}

// ---- NetpBind ----
unsafe fn NetpBind(
    pszServerName: LPCWSTR,
    pBindingHandle: *mut handle_t,
) -> NET_API_STATUS {
    let mut hBinding: handle_t = core::ptr::null_mut();
    let mut pszStringBinding: LPWSTR = core::ptr::null_mut();
    let mut status: RPC_STATUS = 0;

    *pBindingHandle = core::ptr::null_mut();

    // Wide string literals: L"ncacn_np" and L"\\pipe\\wkssvc"
    let ncacn_np: [u16; 9] = [
        0x006e, 0x0063, 0x0061, 0x0063, 0x006e, 0x005f, 0x006e, 0x0070, 0x0000,
    ];
    let pipe_wkssvc: [u16; 13] = [
        0x005c, 0x0070, 0x0069, 0x0070, 0x0065, 0x005c, 0x0077, 0x006b, 0x0073, 0x0073, 0x0076, 0x0063, 0x0000,
    ];

    status = RpcStringBindingComposeW(
        core::ptr::null_mut(),
        ncacn_np.as_ptr() as *mut u16,
        pszServerName as *mut u16,
        pipe_wkssvc.as_ptr() as *mut u16,
        core::ptr::null_mut(),
        &mut pszStringBinding,
    );
    if status != 0 {
        return NetpNtStatusToApiStatus(status);
    }

    status = RpcBindingFromStringBindingW(pszStringBinding, &mut hBinding);
    if status != 0 {
        /* nothing */
    }

    status = RpcStringFreeW(&mut pszStringBinding);
    if status != 0 {
        /* nothing */
    }

    *pBindingHandle = hBinding;
    NetpNtStatusToApiStatus(status)
}

// ---- NetpUnbind ----
unsafe fn NetpUnbind(BindingHandle: handle_t) -> NET_API_STATUS {
    let status = RpcBindingFree(core::ptr::addr_of_mut!(wkssvc_hBinding));
    if status != 0 {
        return NetpNtStatusToApiStatus(status);
    }
    NERR_Success
}

// ---- NetEnumerateComputerNames ----
#[no_mangle]
pub unsafe extern "system" fn NetEnumerateComputerNames(
    Server: LPCWSTR,
    NameType: NET_COMPUTER_NAME_TYPE,
    Reserved: ULONG,
    EntryCount: PDWORD,
    ComputerNames: *mut *mut PWSTR,
) -> NET_API_STATUS {
    let mut ComputerNameArray: PNET_COMPUTER_NAME_ARRAY = core::ptr::null_mut();
    let mut BufferSize: ULONG;
    let mut i: ULONG;
    let mut NameBuffer: *mut PWSTR = core::ptr::null_mut();
    let mut Ptr: PWSTR;
    // Reassigned by the NetApiBufferAllocate call below (the C reuses one `status`).
    let mut status: NET_API_STATUS;

    // RpcTryExcept body (no SEH handler)
    status = rpc_try(|| NetrEnumerateComputerNames(Server as PWSTR, NameType, Reserved, &mut ComputerNameArray)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    if status == 0 /* NERR_Success */ {
        *EntryCount = (*ComputerNameArray).EntryCount;

        BufferSize = 0;
        i = 0;
        while i < (*ComputerNameArray).EntryCount {
            // wkssvc_c.h:662 — `NET_COMPUTER_NAME_ARRAY { ULONG EntryCount; PUNICODE_STRING
            // ComputerNames; }`. `ComputerNames` is already a POINTER, so the C's
            // `ComputerNames[i]` is plain pointer arithmetic over UNICODE_STRING. The
            // translation invented a `NET_COMPUTER_NAME` element type (no such type exists)
            // and called `.as_mut_ptr()` as if the field were an inline array.
            let len = (*(*ComputerNameArray).ComputerNames.add(i as usize)).Length;
            BufferSize = BufferSize.wrapping_add(len as ULONG + core::mem::size_of::<WCHAR>() as ULONG + core::mem::size_of::<PWSTR>() as ULONG);
            i = i.wrapping_add(1);
        }

        status = NetApiBufferAllocate(BufferSize, &mut NameBuffer as *mut _ as *mut *mut core::ffi::c_void);
        if status == 0 /* NERR_Success */ {
            core::ptr::write_bytes(NameBuffer as *mut u8, 0, BufferSize as usize);

            Ptr = ((NameBuffer as usize)
                + ((*ComputerNameArray).EntryCount as usize) * core::mem::size_of::<PWSTR>())
                as PWSTR;

            i = 0;
            while i < (*ComputerNameArray).EntryCount {
                let name_entry = &*(*ComputerNameArray).ComputerNames.add(i as usize);
                *NameBuffer.add(i as usize) = Ptr;
                core::ptr::copy_nonoverlapping(
                    name_entry.Buffer as *const u8,
                    Ptr as *mut u8,
                    name_entry.Length as usize,
                );
                Ptr = ((Ptr as usize) + (name_entry.Length as usize) + core::mem::size_of::<WCHAR>()) as PWSTR;
                i = i.wrapping_add(1);
            }

            *ComputerNames = NameBuffer;
        }
    }

    status
}

// ---- NetGetJoinInformation ----
#[no_mangle]
pub unsafe extern "system" fn NetGetJoinInformation(
    lpServer: LPCWSTR,
    lpNameBuffer: *mut LPWSTR,
    // wksta_new.c:320 declares this `PNETSETUP_JOIN_STATUS BufferType` — the translation
    // added an extra pointer level.
    BufferType: PNETSETUP_JOIN_STATUS,
) -> NET_API_STATUS {
    if lpNameBuffer.is_null() || BufferType.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    // The translation answered NetSetupUnknownStatus with no name and never asked wkssvc,
    // so every caller (netid's Computer Name page, systeminfo) saw neither the workgroup
    // nor the domain. wksta_new.c:333 forwards to the service; the out-name is published
    // only on success, exactly as there.
    let mut pName: LPWSTR = core::ptr::null_mut();
    // RpcTryExcept body (no SEH handler, as in the other wkssvc wrappers here)
    let status = NetrGetJoinInformation(lpServer as WKSSVC_IMPERSONATE_HANDLE, &mut pName, BufferType);
    if status == NERR_Success {
        *lpNameBuffer = pName;
    }
    status
}

// ---- NetUseAdd ----
#[no_mangle]
pub unsafe extern "system" fn NetUseAdd(
    UncServerName: PWSTR,
    Level: u32,
    Buf: LPBYTE,
    ParmError: LPDWORD,
) -> u32 {
    let status = rpc_try(|| NetrUseAdd(UncServerName, Level, Buf as *mut _, ParmError)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetUseDel ----
#[no_mangle]
pub unsafe extern "system" fn NetUseDel(
    UncServerName: LMSTR,
    UseName: LMSTR,
    ForceCond: DWORD,
) -> NET_API_STATUS {
    let status = rpc_try(|| NetrUseDel(UncServerName, UseName, ForceCond)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetUseEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetUseEnum(
    UncServerName: PWSTR,
    Level: u32,
    BufPtr: *mut *mut u8,
    PreferedMaximumSize: u32,
    EntriesRead: *mut u32,
    TotalEntries: *mut u32,
    ResumeHandle: *mut u32,
) -> u32 {
    let mut UseEnumInfo: USE_ENUM_STRUCT = core::mem::zeroed();
    let mut Container0: USE_INFO_0_CONTAINER = core::mem::zeroed();
    let mut Container1: USE_INFO_1_CONTAINER = core::mem::zeroed();
    let mut Container2: USE_INFO_2_CONTAINER = core::mem::zeroed();
    let mut status: u32;

    UseEnumInfo.Level = Level;
    match Level {
        0 => {
            UseEnumInfo.UseInfo.Level0 = &mut Container0;
            Container0.EntriesRead = 0;
            Container0.Buffer = core::ptr::null_mut();
        }
        1 => {
            UseEnumInfo.UseInfo.Level1 = &mut Container1;
            Container1.EntriesRead = 0;
            Container1.Buffer = core::ptr::null_mut();
        }
        2 => {
            UseEnumInfo.UseInfo.Level2 = &mut Container2;
            Container2.EntriesRead = 0;
            Container2.Buffer = core::ptr::null_mut();
        }
        _ => return ERROR_INVALID_PARAMETER,
    }

    // RPC try block
    {
        status = rpc_try(|| NetrUseEnum(
            UncServerName,
            &mut UseEnumInfo as *mut USE_ENUM_STRUCT,
            PreferedMaximumSize,
            TotalEntries,
            ResumeHandle,
        )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
        if status == 0 || status == ERROR_MORE_DATA {
            match Level {
                0 => {
                    *BufPtr = (*UseEnumInfo.UseInfo.Level0).Buffer as LPBYTE;
                    *EntriesRead = (*UseEnumInfo.UseInfo.Level0).EntriesRead;
                }
                1 => {
                    *BufPtr = (*UseEnumInfo.UseInfo.Level1).Buffer as LPBYTE;
                    *EntriesRead = (*UseEnumInfo.UseInfo.Level1).EntriesRead;
                }
                2 => {
                    *BufPtr = (*UseEnumInfo.UseInfo.Level2).Buffer as LPBYTE;
                    *EntriesRead = (*UseEnumInfo.UseInfo.Level2).EntriesRead;
                }
                _ => {}
            }
        }
    }

    status
}

// ---- NetUseGetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetUseGetInfo(
    UncServerName: *mut u16,
    UseName: *mut u16,
    Level: u32,
    BufPtr: *mut *mut u8,
) -> u32 {
    *BufPtr = core::ptr::null_mut();
    let status = rpc_try(|| NetrUseGetInfo(UncServerName, UseName, Level, BufPtr as *mut _)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetWkstaGetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetWkstaGetInfo(
    servername: LPWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
) -> NET_API_STATUS {
    if bufptr.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    *bufptr = core::ptr::null_mut();
    let status = rpc_try(|| NetrWkstaGetInfo(servername, level, bufptr as *mut _)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetWkstaSetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetWkstaSetInfo(
    servername: *mut u16,
    level: u32,
    buffer: *mut u8,
    parm_err: *mut u32,
) -> u32 {
    // wksta_new.c:916 casts the caller buffer: `(LPWKSTA_INFO)buffer`.
    let status = rpc_try(|| NetrWkstaSetInfo(servername, level, buffer as LPWKSTA_INFO, parm_err)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetWkstaTransportAdd ----
#[no_mangle]
pub unsafe extern "system" fn NetWkstaTransportAdd(
    servername: LPWSTR,
    level: DWORD,
    buf: LPBYTE,
    parm_err: LPDWORD,
) -> NET_API_STATUS {
    // The translation declared a local empty `struct _WKSTA_TRANSPORT_INFO_0;` here, shadowing
    // the real wire type from the island. wksta_new.c:946 simply casts:
    // `(LPWKSTA_TRANSPORT_INFO_0)buf`.
    let status = rpc_try(|| NetrWkstaTransportAdd(
        servername,
        level,
        buf as LPWKSTA_TRANSPORT_INFO_0,
        parm_err,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetWkstaTransportDel ----
#[no_mangle]
pub unsafe extern "system" fn NetWkstaTransportDel(
    servername: LPWSTR,
    transportname: LPWSTR,
    ucond: DWORD,
) -> NET_API_STATUS {
    let status = rpc_try(|| NetrWkstaTransportDel(servername, transportname, ucond)).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetWkstaUserEnum ----
#[no_mangle]
pub unsafe extern "system" fn NetWkstaUserEnum(
    servername: PWSTR,
    level: DWORD,
    bufptr: *mut LPBYTE,
    prefmaxlen: DWORD,
    entriesread: PDWORD,
    totalentries: PDWORD,
    resumehandle: PDWORD,
) -> DWORD {
    let mut UserEnumInfo: WKSTA_USER_ENUM_STRUCT = core::mem::zeroed();
    UserEnumInfo.Level = level;
    match level {
        0 => {
            UserEnumInfo.WkstaUserInfo.Level0.as_mut().EntriesRead = 0;
            UserEnumInfo.WkstaUserInfo.Level0.as_mut().Buffer = core::ptr::null_mut();
        }
        1 => {
            UserEnumInfo.WkstaUserInfo.Level1.as_mut().EntriesRead = 0;
            UserEnumInfo.WkstaUserInfo.Level1.as_mut().Buffer = core::ptr::null_mut();
        }
        _ => return ERROR_INVALID_PARAMETER,
    }

    let status = rpc_try(|| NetrWkstaUserEnum(
        servername,
        &mut UserEnumInfo,
        prefmaxlen,
        totalentries,
        resumehandle,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    if status == 0 || status == ERROR_MORE_DATA {
        match level {
            0 => {
                *bufptr = UserEnumInfo.WkstaUserInfo.Level0.as_ref().Buffer as LPBYTE;
                *entriesread = UserEnumInfo.WkstaUserInfo.Level0.as_ref().EntriesRead;
            }
            1 => {
                *bufptr = UserEnumInfo.WkstaUserInfo.Level1.as_ref().Buffer as LPBYTE;
                *entriesread = UserEnumInfo.WkstaUserInfo.Level1.as_ref().EntriesRead;
            }
            _ => {}
        }
    }

    status
}

// ---- NetWkstaUserGetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetWkstaUserGetInfo(
    reserved: *mut u16,
    level: u32,
    bufptr: *mut *mut u8,
) -> u32 {
    if !reserved.is_null() || bufptr.is_null() {
        return ERROR_INVALID_PARAMETER;
    }
    *bufptr = core::ptr::null_mut();
    extern "system" {
    }
    let status = rpc_try(|| NetrWkstaUserGetInfo(core::ptr::null_mut(), level, bufptr.cast())).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    status
}

// ---- NetWkstaUserSetInfo ----
#[no_mangle]
pub unsafe extern "system" fn NetWkstaUserSetInfo(
    reserved: PWSTR,
    level: u32,
    // `&buf` is taken below (matching the C), so the parameter binding must be `mut`.
    mut buf: *mut u8,
    parm_err: *mut u32,
) -> u32 {
    if !reserved.is_null() {
        return ERROR_INVALID_PARAMETER;
    }

    let status = rpc_try(|| NetrWkstaUserSetInfo(
        core::ptr::null_mut(),
        level,
        // wksta_new.c:1180 — `(LPWKSTA_USER_INFO)&buf`
        core::ptr::addr_of_mut!(buf) as LPWKSTA_USER_INFO,
        parm_err,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    status
}

// ---- NetAddAlternateComputerName ----
#[no_mangle]
pub unsafe extern "system" fn NetAddAlternateComputerName(
    Server: LPCWSTR,
    AlternateName: LPCWSTR,
    DomainAccount: LPCWSTR,
    DomainAccountPassword: LPCWSTR,
    Reserved: ULONG,
) -> NET_API_STATUS {
    let EncryptedPassword: PJOINPR_ENCRYPTED_USER_PASSWORD = core::ptr::null_mut();
    let mut BindingHandle: handle_t = core::ptr::null_mut();
    let mut status = NetpBind(Server, &mut BindingHandle);
    if status != NERR_Success {
        return status;
    }
    // RpcTryExcept
    status = rpc_try(|| NetrAddAlternateComputerName(
        BindingHandle,
        Server as PWSTR,
        AlternateName as PWSTR,
        DomainAccount as PWSTR,
        EncryptedPassword,
        Reserved,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    // RpcExcept(EXCEPTION_EXECUTE_HANDLER) { status = I_RpcMapWin32Status(RpcExceptionCode()); } RpcEndExcept
    NetpUnbind(BindingHandle);
    status
}

// ---- NetGetJoinableOUs ----
#[no_mangle]
pub unsafe extern "system" fn NetGetJoinableOUs(
    lpServer: LPCWSTR,
    lpDomain: LPCWSTR,
    lpAccount: LPCWSTR,
    lpPassword: LPCWSTR,
    OUCount: *mut DWORD,
    OUs: *mut *mut LPWSTR,
) -> NET_API_STATUS {
    let EncryptedPassword: *mut JOINPR_ENCRYPTED_USER_PASSWORD = core::ptr::null_mut();
    let mut BindingHandle: handle_t = core::ptr::null_mut(); // C: `handle_t BindingHandle = NULL`
    let mut status = NetpBind(lpServer, &mut BindingHandle);
    if status != 0 {
        return status;
    }
    // RpcTryExcept body executed directly; handler omitted
    status = rpc_try(|| NetrGetJoinableOUs2(
        BindingHandle,
        lpServer as PWSTR,
        lpDomain as PWSTR,
        lpAccount as PWSTR,
        EncryptedPassword,
        OUCount,
        OUs,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    NetpUnbind(BindingHandle);
    status
}

// ---- NetJoinDomain ----
#[no_mangle]
pub unsafe extern "system" fn NetJoinDomain(
    lpServer: LPCWSTR,
    lpDomain: LPCWSTR,
    lpAccountOU: LPCWSTR,
    lpAccount: LPCWSTR,
    lpPassword: LPCWSTR,
    fJoinOptions: DWORD,
) -> NET_API_STATUS {
    let mut binding_handle: handle_t = core::ptr::null_mut();
    // Reassigned by the NetrJoinDomain2 call below.
    let mut status: NET_API_STATUS;

    status = NetpBind(lpServer, &mut binding_handle);
    if status != 0 {
        return status;
    }

    status = rpc_try(|| NetrJoinDomain2(
        binding_handle,
        lpServer as PWSTR,
        lpDomain as PWSTR,
        lpAccountOU as PWSTR,
        lpAccount as PWSTR,
        core::ptr::null_mut(),
        fJoinOptions,
    )).unwrap_or_else(|code| code as _);

    NetpUnbind(binding_handle);

    status
}

// ---- NetRemoveAlternateComputerName ----
#[no_mangle]
pub unsafe extern "system" fn NetRemoveAlternateComputerName(
    Server: LPCWSTR,
    AlternateName: LPCWSTR,
    DomainAccount: LPCWSTR,
    _DomainAccountPassword: LPCWSTR,
    Reserved: ULONG,
) -> NET_API_STATUS {
    let mut BindingHandle: *mut core::ffi::c_void = core::ptr::null_mut();

    let mut status = NetpBind(Server, &mut BindingHandle);
    if status != 0 {
        return status;
    }

    status = rpc_try(|| NetrRemoveAlternateComputerName(
        BindingHandle,
        Server as *mut u16,
        AlternateName as *mut u16,
        DomainAccount as *mut u16,
        core::ptr::null_mut(),
        Reserved,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    NetpUnbind(BindingHandle);

    status
}

// ---- NetRenameMachineInDomain ----
#[no_mangle]
pub unsafe extern "system" fn NetRenameMachineInDomain(
    lpServer: LPCWSTR,
    lpNewMachineName: LPCWSTR,
    lpAccount: LPCWSTR,
    lpPassword: LPCWSTR,
    fRenameOptions: DWORD,
) -> NET_API_STATUS {
    let mut binding_handle: handle_t = core::ptr::null_mut();
    let status = NetpBind(lpServer, &mut binding_handle);
    if status != 0 {
        return status;
    }
    let status = rpc_try(|| NetrRenameMachineInDomain2(
        binding_handle,
        lpServer as PWSTR,
        lpNewMachineName as PWSTR,
        lpAccount as PWSTR,
        core::ptr::null_mut(),
        fRenameOptions,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    NetpUnbind(binding_handle);
    status
}

// ---- NetSetPrimaryComputerName ----
#[no_mangle]
pub unsafe extern "system" fn NetSetPrimaryComputerName(
    Server: LPCWSTR,
    PrimaryName: LPCWSTR,
    DomainAccount: LPCWSTR,
    DomainAccountPassword: LPCWSTR,
    Reserved: u32,
) -> u32 {
    let EncryptedPassword: *mut u8 = core::ptr::null_mut();
    let mut BindingHandle: handle_t = core::ptr::null_mut();
    let mut status: u32;

    status = NetpBind(Server, &mut BindingHandle as *mut handle_t);
    if status != 0 {
        return status;
    }

    status = rpc_try(|| NetrSetPrimaryComputerName(
        BindingHandle,
        Server as *mut u16,
        PrimaryName as *mut u16,
        DomainAccount as *mut u16,
        EncryptedPassword as *mut _,
        Reserved,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    NetpUnbind(BindingHandle);
    return status;
}

// ---- NetUnjoinDomain ----
#[no_mangle]
pub unsafe extern "system" fn NetUnjoinDomain(
    lpServer: LPCWSTR,
    lpAccount: LPCWSTR,
    lpPassword: LPCWSTR,
    fUnjoinOptions: DWORD,
) -> NET_API_STATUS {
    let _ = lpPassword; // unused
    let encrypted_password: *mut JOINPR_ENCRYPTED_USER_PASSWORD = core::ptr::null_mut();
    let mut binding_handle: handle_t = core::ptr::null_mut();
    let status = NetpBind(lpServer, &mut binding_handle);
    if status != 0 {
        return status;
    }
    // RPC try block (exception handler omitted per translation rules)
    let status = rpc_try(|| NetrUnjoinDomain2(
        binding_handle,
        lpServer as *mut u16,
        lpAccount as *mut u16,
        encrypted_password,
        fUnjoinOptions,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);
    NetpUnbind(binding_handle);
    status
}

// ---- NetValidateName ----
#[no_mangle]
pub unsafe extern "system" fn NetValidateName(
    lpServer: LPCWSTR,
    lpName: LPCWSTR,
    lpAccount: LPCWSTR,
    lpPassword: LPCWSTR,
    NameType: NETSETUP_NAME_TYPE,
) -> NET_API_STATUS {
    // wksta_new.c:827 — `PJOINPR_ENCRYPTED_USER_PASSWORD EncryptedPassword = NULL;`
    let EncryptedPassword: PJOINPR_ENCRYPTED_USER_PASSWORD = core::ptr::null_mut();
    let mut BindingHandle: handle_t = core::ptr::null_mut();
    // Reassigned by the NetrValidateName2 call below.
    let mut status: NET_API_STATUS;

    status = NetpBind(lpServer, &mut BindingHandle);
    if status != 0 {
        return status;
    }

    status = rpc_try(|| NetrValidateName2(
        BindingHandle,
        lpServer as PWSTR,
        lpName as PWSTR,
        lpAccount as PWSTR,
        EncryptedPassword,
        NameType,
    )).unwrap_or_else(|code| I_RpcMapWin32Status(code as _) as _);

    NetpUnbind(BindingHandle);
    status
}
