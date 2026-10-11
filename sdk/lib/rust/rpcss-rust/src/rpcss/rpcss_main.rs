use crate::rpcss::setup::DoRpcSsSetupConfiguration;
use crate::rpcss::irotp::MIDL_user_allocate;
use crate::prelude::*;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut rpcssW: [WCHAR; 6] =
    unsafe { ::core::mem::transmute::<[u8; 12], [WCHAR; 6]>(*b"R\0p\0c\0S\0s\0\0\0") };

static mut exit_event: HANDLE =
    ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void;

static mut service_handle: SERVICE_STATUS_HANDLE =
    ::core::ptr::null::<SERVICE_STATUS_HANDLE__>() as *mut SERVICE_STATUS_HANDLE__;

static mut registered_classes_cs: CRITICAL_SECTION = _RTL_CRITICAL_SECTION {
    DebugInfo: ::core::ptr::null::<_RTL_CRITICAL_SECTION_DEBUG>()
        as *mut _RTL_CRITICAL_SECTION_DEBUG,
    LockCount: -(1 as ::core::ffi::c_int) as LONG,
    RecursionCount: 0 as LONG,
    OwningThread: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    LockSemaphore: ::core::ptr::null::<::core::ffi::c_void>() as *mut ::core::ffi::c_void,
    SpinCount: 0 as ULONG_PTR,
};

static mut registered_classes: list = unsafe {
    list {
        next: &raw const registered_classes as *mut list,
        prev: &raw const registered_classes as *mut list,
    }
};

// #define COBJMACROS: no declaration emitted.
// #ifdef __REACTOS__ includes and ARRAY_SIZE function-like macro: omitted.
// WINE_DEFAULT_DEBUG_CHANNEL(ole): debug-channel macro, omitted.
// extern VOID DoRpcSsSetupConfiguration(VOID): function declaration, omitted (imports are handled by the caller).

#[repr(C)]
pub struct registered_class {
    pub entry: list,
    pub clsid: GUID,
    pub cookie: u32,
    pub object: PMInterfacePointer,
    // `unsigned int single_use : 1;` bitfield, stored as bit 0 of this word
    pub _bitfield_1: u32,
}

impl registered_class {
    /// `single_use : 1`, bit 0 of `_bitfield_1` (the accessor shape bindgen gives a bitfield).
    #[inline]
    pub fn single_use(&self) -> u32 {
        self._bitfield_1 & 1
    }
    #[inline]
    pub fn set_single_use(&mut self, v: u32) {
        self._bitfield_1 = (self._bitfield_1 & !1) | (v & 1);
    }
}

// LIST_INIT(registered_classes) is self-referential ({ &head, &head }) and
// cannot be expressed in a Rust static initializer; the list head is set up
// at runtime to point at itself before first use.
#[no_mangle]
pub unsafe extern "C" fn irpcss_server_register(
    mut h: handle_t,
    mut clsid: *const GUID,
    mut flags: ::core::ffi::c_uint,
    mut object: PMInterfacePointer,
    mut cookie: *mut ::core::ffi::c_uint,
) -> HRESULT {
    let mut entry: *mut registered_class = core::ptr::null_mut();
    static mut next_cookie: LONG = 0;

    entry = calloc(
        (1 as size_t) as _,
        (core::mem::size_of::<registered_class>() as size_t) as _,
    ) as *mut registered_class;
    if entry.is_null() {
        return E_OUTOFMEMORY;
    }

    (*entry).clsid = *clsid;
    (*entry).set_single_use(
        ((flags & (REGCLS_MULTIPLEUSE | REGCLS_MULTI_SEPARATE) as ::core::ffi::c_uint) == 0)
            as ::core::ffi::c_int as ::core::ffi::c_uint,
    );

    (*entry).object = malloc(
        (core::mem::offset_of!(MInterfacePointer, abData) + (*object).ulCntData as usize)
            as size_t,
    ) as PMInterfacePointer;
    if (*entry).object.is_null() {
        free(entry as *mut ::core::ffi::c_void);
        return E_OUTOFMEMORY;
    }

    (*(*entry).object).ulCntData = (*object).ulCntData;
    memcpy(
        &raw mut (*(*entry).object).abData as *mut ::core::ffi::c_void,
        &raw mut (*object).abData as *mut BYTE as *const ::core::ffi::c_void,
        (*object).ulCntData as size_t,
    );

    (*entry).cookie =
        InterlockedIncrement(&raw mut next_cookie) as ::core::ffi::c_uint;
    *cookie = (*entry).cookie;

    EnterCriticalSection(&raw mut registered_classes_cs);
    list_add_tail(&raw mut registered_classes, &raw mut (*entry).entry);
    LeaveCriticalSection(&raw mut registered_classes_cs);

    return S_OK;
}

unsafe extern "C" fn scm_revoke_class(_class: *mut registered_class) {
    list_remove(core::ptr::addr_of_mut!((*_class).entry));
    free(((*_class).object).cast());
    free((_class).cast());
}
#[no_mangle]
pub unsafe extern "C" fn irpcss_server_revoke(h: handle_t, cookie: u32) -> HRESULT {
    let mut cur: *mut registered_class = core::ptr::null_mut();

    EnterCriticalSection(&raw mut registered_classes_cs);

    cur = ((registered_classes.next as *mut u8)
        .sub(core::mem::offset_of!(registered_class, entry)))
        as *mut registered_class;
    while &raw mut (*cur).entry != &raw mut registered_classes {
        if (*cur).cookie == cookie {
            scm_revoke_class(cur);
            break;
        }
        cur = (((*cur).entry.next as *mut u8)
            .sub(core::mem::offset_of!(registered_class, entry)))
            as *mut registered_class;
    }

    LeaveCriticalSection(&raw mut registered_classes_cs);

    return S_OK;
}
#[no_mangle]
pub unsafe extern "C" fn irpcss_get_class_object(
    mut h: handle_t,
    mut clsid: *const GUID,
    mut object: *mut PMInterfacePointer,
) -> HRESULT {
    let mut cur: *mut registered_class = ::core::ptr::null_mut::<registered_class>();
    *object = ::core::ptr::null_mut::<MInterfacePointer>();
    EnterCriticalSection(&raw mut registered_classes_cs);
    cur = (registered_classes.next as *mut ::core::ffi::c_char).offset(
        -(&raw mut (*::core::ptr::null_mut::<registered_class>()).entry as ::core::ffi::c_ulonglong
            as isize),
    ) as *mut registered_class;
    while &raw mut (*cur).entry != &raw mut registered_classes {
        if memcmp(
            clsid as *const ::core::ffi::c_void,
            &raw mut (*cur).clsid as *const ::core::ffi::c_void,
            ::core::mem::size_of::<GUID>() as size_t,
        ) == 0
        {
            *object = MIDL_user_allocate((FIELD_OFFSET!(MInterfacePointer, abData[(*(*cur).object).ulCntData as usize]) as usize) as ::core::ffi::c_ulonglong as LONG
                as SIZE_T) as PMInterfacePointer;
            if !(*object).is_null() {
                (**object).ulCntData = (*(*cur).object).ulCntData;
                memcpy(
                    &raw mut (**object).abData as *mut BYTE as *mut ::core::ffi::c_void,
                    &raw mut (*(*cur).object).abData as *mut BYTE as *const ::core::ffi::c_void,
                    (*(*cur).object).ulCntData as size_t,
                );
            }
            if (*cur).single_use() != 0 {
                scm_revoke_class(cur);
            }
            break;
        } else {
            cur = ((*cur).entry.next as *mut ::core::ffi::c_char).offset(
                -(&raw mut (*::core::ptr::null_mut::<registered_class>()).entry
                    as ::core::ffi::c_ulonglong as isize),
            ) as *mut registered_class;
        }
    }
    LeaveCriticalSection(&raw mut registered_classes_cs);
    return if !(*object).is_null() {
        S_OK
    } else {
        E_NOINTERFACE
    };
}
#[no_mangle]
pub unsafe extern "C" fn irpcss_get_thread_seq_id(h: handle_t, id: *mut DWORD) -> HRESULT {
    static mut thread_seq_id: LONG = 0;
    *id = InterlockedIncrement(core::ptr::addr_of_mut!(thread_seq_id)) as DWORD;
    return S_OK;
}

unsafe extern "C" fn RPCSS_Initialize() -> RPC_STATUS {
    // irot.idl: `#define IROT_PROTSEQ {'n','c','a','l','r','p','c',0}` and
    // `#define IROT_ENDPOINT {'i','r','o','t',0}` -- array initializers, not constants.
    static mut irot_protseq: [u16; 8] =
        ['n' as u16, 'c' as u16, 'a' as u16, 'l' as u16, 'r' as u16, 'p' as u16, 'c' as u16, 0];
    static mut irot_endpoint: [u16; 5] = ['i' as u16, 'r' as u16, 'o' as u16, 't' as u16, 0];
    static mut epm_protseq: [u16; 9] = [0x006Eu16, 0x0063, 0x0061, 0x0063, 0x006E, 0x005F, 0x006E, 0x0070, 0x0000];
    static mut epm_endpoint: [u16; 15] = [
    0x005Cu16, 0x0070, 0x0069, 0x0070, 0x0065, 0x005C, 0x0065, 0x0070, 0x006D, 0x0061,
    0x0070, 0x0070, 0x0065, 0x0072, 0x0000,
];
    static mut epm_protseq_lrpc: [u16; 8] = [0x006Eu16, 0x0063, 0x0061, 0x006C, 0x0072, 0x0070, 0x0063, 0x0000];
    static mut epm_endpoint_lrpc: [u16; 9] = [0x0065u16, 0x0070, 0x006D, 0x0061, 0x0070, 0x0070, 0x0065, 0x0072, 0x0000];
    // irpcss.idl: `{'n','c','a','l','r','p','c',0}` / `{'i','r','p','c','s','s',0}`.
    static mut irpcss_protseq: [u16; 8] =
        ['n' as u16, 'c' as u16, 'a' as u16, 'l' as u16, 'r' as u16, 'p' as u16, 'c' as u16, 0];
    static mut irpcss_endpoint: [u16; 7] =
        ['i' as u16, 'r' as u16, 'p' as u16, 'c' as u16, 's' as u16, 's' as u16, 0];

    #[repr(C)]
    struct protseq_map {
        protseq: *mut u16,
        endpoint: *mut u16,
    }

    let protseqs: [protseq_map; 4] = [
        protseq_map {
            protseq: &raw mut epm_protseq as *mut u16,
            endpoint: &raw mut epm_endpoint as *mut u16,
        },
        protseq_map {
            protseq: &raw mut epm_protseq_lrpc as *mut u16,
            endpoint: &raw mut epm_endpoint_lrpc as *mut u16,
        },
        protseq_map {
            protseq: &raw mut irot_protseq as *mut u16,
            endpoint: &raw mut irot_endpoint as *mut u16,
        },
        protseq_map {
            protseq: &raw mut irpcss_protseq as *mut u16,
            endpoint: &raw mut irpcss_endpoint as *mut u16,
        },
    ];
    let ifspecs: [RPC_IF_HANDLE; 3] = [
        epm_v3_0_s_ifspec,
        Irot_v0_2_s_ifspec,
        Irpcss_v0_0_s_ifspec,
    ];
    let mut status: RPC_STATUS = 0;
    let mut i: i32 = 0;
    let mut j: i32 = 0;

    WINE_TRACE!(b"\n\0".as_ptr() as *const i8);

    'fail: {
        i = 0;
        j = 0;
        while (i as usize) < ARRAY_SIZE!(ifspecs) {
            status = RpcServerRegisterIf(ifspecs[i as usize], core::ptr::null_mut(), core::ptr::null_mut());
            if status != RPC_S_OK {
                break 'fail;
            }
            i += 1;
            j = i;
        }

        i = 0;
        while (i as usize) < ARRAY_SIZE!(protseqs) {
            status = RpcServerUseProtseqEpW(
                protseqs[i as usize].protseq,
                RPC_C_PROTSEQ_MAX_REQS_DEFAULT,
                protseqs[i as usize].endpoint,
                core::ptr::null_mut(),
            );
            if status != RPC_S_OK {
                break 'fail;
            }
            i += 1;
        }

        status = RpcServerListen(1, RPC_C_LISTEN_MAX_CALLS_DEFAULT, TRUE);
        if status != RPC_S_OK {
            break 'fail;
        }

        return RPC_S_OK;
    }

    // fail:
    i = 0;
    while i < j {
        RpcServerUnregisterIf(ifspecs[i as usize], core::ptr::null_mut(), FALSE);
        i += 1;
    }

    return status;
}

unsafe extern "system" fn service_handler(
    mut ctrl: DWORD,
    mut event_type: DWORD,
    mut event_data: LPVOID,
    mut context: LPVOID,
) -> DWORD {
    let mut status: SERVICE_STATUS = core::mem::zeroed();

    status.dwServiceType = SERVICE_WIN32;
    status.dwControlsAccepted = {
        0
    };
    status.dwWin32ExitCode = 0;
    status.dwServiceSpecificExitCode = 0;
    status.dwCheckPoint = 0;
    status.dwWaitHint = 0;

    match ctrl {
        SERVICE_CONTROL_STOP | SERVICE_CONTROL_SHUTDOWN => {
            TRACE!("shutting down\n");
            RpcMgmtStopServerListening(core::ptr::null_mut());
            RpcServerUnregisterIf(
                epm_v3_0_s_ifspec,
                core::ptr::null_mut::<UUID>(),
                TRUE,
            );
            RpcServerUnregisterIf(
                Irot_v0_2_s_ifspec,
                core::ptr::null_mut::<UUID>(),
                TRUE,
            );
            status.dwCurrentState = SERVICE_STOP_PENDING;
            status.dwControlsAccepted = 0;
            SetServiceStatus(service_handle, core::ptr::addr_of_mut!(status));
            SetEvent(exit_event);
            return NO_ERROR;
        }
        _ => {
            FIXME!("got service ctrl %lx\n", ctrl);
            status.dwCurrentState = SERVICE_RUNNING;
            SetServiceStatus(service_handle, core::ptr::addr_of_mut!(status));
            return NO_ERROR;
        }
    };
}

unsafe extern "system" fn ServiceMain(argc: DWORD, argv: *mut LPWSTR) {
    let mut status: SERVICE_STATUS = core::mem::zeroed();
    let mut ret: RPC_STATUS;
    const REACTOS: bool = true;

    ret = RPCSS_Initialize();
    if ret != 0 {
        WARN!("Failed to initialize rpc interfaces, status %ld.\n", ret);
        return;
    }

    exit_event = CreateEventW(
        core::ptr::null_mut(),
        (TRUE) as BOOL,
        (FALSE) as BOOL,
        core::ptr::null_mut(),
    );

    service_handle = RegisterServiceCtrlHandlerExW(
        core::ptr::addr_of_mut!(rpcssW) as *mut WCHAR,
        Some(service_handler),
        core::ptr::null_mut(),
    );
    if service_handle.is_null() {
        return;
    }

    status.dwServiceType = SERVICE_WIN32;
    status.dwCurrentState = SERVICE_RUNNING;
    status.dwControlsAccepted = {
        0
    };
    status.dwWin32ExitCode = 0;
    status.dwServiceSpecificExitCode = 0;
    status.dwCheckPoint = 0;
    status.dwWaitHint = {
        0
    };
    SetServiceStatus(service_handle, core::ptr::addr_of_mut!(status));

    {
        DoRpcSsSetupConfiguration();
    }

    WaitForSingleObject(exit_event, INFINITE);

    status.dwCurrentState = SERVICE_STOPPED;
    status.dwControlsAccepted = 0;
    SetServiceStatus(service_handle, core::ptr::addr_of_mut!(status));
}

#[no_mangle]
pub unsafe extern "C" fn wmain(argc: i32, argv: *mut *mut WCHAR) -> i32 {
    static mut service_table: [SERVICE_TABLE_ENTRYW; 2] = [
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::addr_of!(rpcssW) as LPWSTR,
            lpServiceProc: Some(ServiceMain),
        },
        SERVICE_TABLE_ENTRYW {
            lpServiceName: core::ptr::null_mut(),
            lpServiceProc: None,
        },
    ];
    StartServiceCtrlDispatcherW(core::ptr::addr_of!(service_table) as *const SERVICE_TABLE_ENTRYW);
    return 0;
}
