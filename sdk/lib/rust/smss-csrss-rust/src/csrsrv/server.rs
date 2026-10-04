use super::srv_inlines::*;
use core::ffi::c_void;
use ntdll_sys::nt::*;
#[no_mangle]
pub static mut CsrLoadedServerDll: [PCSR_SERVER_DLL; CSR_SERVER_DLL_MAX as usize] =
    [core::ptr::null_mut(); CSR_SERVER_DLL_MAX as usize];
#[no_mangle]
pub static mut CsrSrvSharedSectionHeap: PVOID = core::ptr::null_mut();
static mut CsrSrvSharedSectionBase: PVOID = core::ptr::null_mut();
#[no_mangle]
pub static mut CsrSrvSharedStaticServerData: *mut PVOID = core::ptr::null_mut();
static mut CsrSrvSharedSectionSize: ULONG = 0;
static mut CsrSrvSharedSection: HANDLE = core::ptr::null_mut();

// (LLM emitted bogus `#[cfg(target_env="msvc")]` here — it gated the
// static OUT on the gnu targets → "cannot find value" at use sites.
// Also `#[cfg]` is not valid on array-literal elements. `ws03`
// feature is undefined, so this is the not-ws03 variant.)
// PCSR_API_ROUTINE is Option<extern "stdcall" fn>; these fns are
// extern "system" (== stdcall on i386, identical ABI) — Rust's type
// system distinguishes the ABI string, so const-transmute each.
// `fn as usize` is forbidden in const-eval; transmute the fn pointer
// directly (system-ABI fn-ptr -> stdcall-ABI Option<fn-ptr>; same
// size + ABI on i386, const-safe — no ptr->int cast).
type SrvFn = unsafe extern "system" fn(PCSR_API_MESSAGE, *mut CSR_REPLY_CODE) -> NTSTATUS;
static CsrServerApiDispatchTable: [PCSR_API_ROUTINE; 5] = unsafe {
    [
        core::mem::transmute::<SrvFn, PCSR_API_ROUTINE>(CsrSrvClientConnect),
        core::mem::transmute::<SrvFn, PCSR_API_ROUTINE>(CsrSrvUnusedFunction),
        core::mem::transmute::<SrvFn, PCSR_API_ROUTINE>(CsrSrvUnusedFunction),
        core::mem::transmute::<SrvFn, PCSR_API_ROUTINE>(CsrSrvIdentifyAlertableThread),
        core::mem::transmute::<SrvFn, PCSR_API_ROUTINE>(CsrSrvSetPriorityClass),
    ]
};

static CsrServerApiServerValidTable: [BOOLEAN; 5] = [1, 0, 0, 1, 1];

#[cfg(feature = "csr_dbg")]
static CSR_SERVER_API_NAME_TABLE: [*const i8; 5] = [
    b"ClientConnect\0".as_ptr() as *const i8,
    b"ThreadConnect\0".as_ptr() as *const i8,
    b"ProfileControl\0".as_ptr() as *const i8,
    b"IdentifyAlertableThread\0".as_ptr() as *const i8,
    b"SetPriorityClass\0".as_ptr() as *const i8,
];

#[no_mangle]
pub unsafe extern "system" fn CsrServerDllInitialization(
    LoadedServerDll: PCSR_SERVER_DLL,
) -> NTSTATUS {
    (*LoadedServerDll).ApiBase = CSRSRV_FIRST_API_NUMBER;
    (*LoadedServerDll).HighestApiSupported = CsrpMaxApiNumber;
    (*LoadedServerDll).DispatchTable = CsrServerApiDispatchTable.as_ptr() as *mut PCSR_API_ROUTINE;
    (*LoadedServerDll).ValidTable = CsrServerApiServerValidTable.as_ptr() as *mut BOOLEAN;
    #[cfg(feature = "csr_dbg")]
    {
        (*LoadedServerDll).NameTable = CSR_SERVER_API_NAME_TABLE.as_ptr() as *mut *mut i8;
    }
    (*LoadedServerDll).SizeOfProcessData = 0;
    (*LoadedServerDll).ConnectCallback = None;
    (*LoadedServerDll).DisconnectCallback = None;

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrLoadServerDll(
    DllString: *mut i8,
    EntryPoint: *mut i8,
    ServerId: ULONG,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DllName: ANSI_STRING = core::mem::zeroed();
    let mut TempString: _UNICODE_STRING = core::mem::zeroed();
    let mut ErrorString: _UNICODE_STRING = core::mem::zeroed();
    let mut Parameters: [ULONG_PTR; 2] = [0; 2];
    let mut hServerDll: HANDLE = core::ptr::null_mut();
    let Size: ULONG;
    let ServerDll: PCSR_SERVER_DLL;
    let mut EntryPointString: ANSI_STRING = core::mem::zeroed();
    let mut ServerDllInitProcedure: PCSR_SERVER_DLL_INIT_CALLBACK = None;
    let mut Response: ULONG = 0;

    if ServerId >= CSR_SERVER_DLL_MAX {
        return STATUS_TOO_MANY_NAMES;
    }

    if !CsrLoadedServerDll[ServerId as usize].is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    RtlInitAnsiString(&mut DllName, DllString);
    Status = RtlAnsiStringToUnicodeString(&mut TempString, &mut DllName, 1);
    if Status < 0 {
        return Status;
    }

    if ServerId != CSRSRV_SERVERDLL_INDEX {
        Status = LdrLoadDll(
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut TempString,
            &mut hServerDll,
        );
        if Status < 0 {
            static ERROR_PATH: [u16; 18] = [
                b'D' as u16,
                b'e' as u16,
                b'f' as u16,
                b'a' as u16,
                b'u' as u16,
                b'l' as u16,
                b't' as u16,
                b' ' as u16,
                b'L' as u16,
                b'o' as u16,
                b'a' as u16,
                b'd' as u16,
                b' ' as u16,
                b'P' as u16,
                b'a' as u16,
                b't' as u16,
                b'h' as u16,
                0,
            ];

            Parameters[0] = &TempString as *const _ as ULONG_PTR;
            Parameters[1] = &ErrorString as *const _ as ULONG_PTR;
            // RtlInitUnicodeString(&ErrorString, L"Default Load Path"): 17 chars.
            ErrorString.Length = ((ERROR_PATH.len() - 1) * core::mem::size_of::<u16>()) as u16;
            ErrorString.MaximumLength = core::mem::size_of_val(&ERROR_PATH) as u16;
            ErrorString.Buffer = ERROR_PATH.as_ptr() as *mut u16;

            NtRaiseHardError(
                Status,
                2,
                3,
                Parameters.as_mut_ptr(),
                OptionOk,
                &mut Response,
            );
        }

        RtlFreeUnicodeString(&mut TempString);
        crate::dbg_print!(b"[CSLD] after LdrLoadDll st=%lx h=%p\n\0", Status, hServerDll);
        if Status < 0 {
            return Status;
        }
    }

    Size = core::mem::size_of::<CSR_SERVER_DLL>() as ULONG + DllName.MaximumLength as ULONG;
    ServerDll = RtlAllocateHeap(
        CsrHeap,
        HEAP_ZERO_MEMORY,
        (Size as usize).try_into().unwrap(),
    ) as PCSR_SERVER_DLL;
    if ServerDll.is_null() {
        if !hServerDll.is_null() {
            LdrUnloadDll(hServerDll);
        }
        return STATUS_NO_MEMORY;
    }

    (*ServerDll).Length = Size;
    (*ServerDll).SizeOfProcessData = 0;
    (*ServerDll).SharedSection = CsrSrvSharedSectionHeap;
    (*ServerDll).Name.Length = DllName.Length;
    (*ServerDll).Name.MaximumLength = DllName.MaximumLength;
    (*ServerDll).Name.Buffer =
        (ServerDll as usize + core::mem::size_of::<CSR_SERVER_DLL>()) as *mut i8;
    if DllName.Length != 0 {
        core::ptr::copy_nonoverlapping(
            DllName.Buffer as *const u8,
            (*ServerDll).Name.Buffer as *mut u8,
            DllName.Length as usize,
        );
    }
    (*ServerDll).ServerId = ServerId;
    (*ServerDll).ServerHandle = hServerDll;

    if !hServerDll.is_null() {
        RtlInitAnsiString(
            &mut EntryPointString,
            if !EntryPoint.is_null() {
                EntryPoint
            } else {
                b"ServerDllInitialization\0".as_ptr() as *mut i8
            },
        );

        Status = LdrGetProcedureAddress(
            hServerDll,
            &mut EntryPointString,
            0,
            &mut ServerDllInitProcedure as *mut _ as *mut PVOID,
        );
    } else {
        #[cfg(feature = "csr_dbg")]
        RtlInitAnsiString(
            &mut EntryPointString,
            b"CsrServerDllInitialization\0".as_ptr() as *mut i8,
        );
        ServerDllInitProcedure = core::mem::transmute(CsrServerDllInitialization as usize);
        Status = STATUS_SUCCESS;
    }

    if Status >= 0 {
        crate::dbg_print!(b"[CSLD] pre-ServerDllInit proc=%p\n\0", ServerDllInitProcedure.map(|f| f as usize).unwrap_or(0));
        Status = 'seh: {
            if let Some(init_proc) = ServerDllInitProcedure {
                let result = init_proc(ServerDll);
                crate::dbg_print!(b"[CSLD] post-ServerDllInit ret=%lx\n\0", result);
                if result < 0 {
                    break 'seh result;
                }
                result
            } else {
                STATUS_UNSUCCESSFUL
            }
        };

        if Status >= 0 {
            CsrTotalPerProcessDataLength += (*ServerDll).SizeOfProcessData;
            CsrLoadedServerDll[(*ServerDll).ServerId as usize] = ServerDll;

            if (*ServerDll).SharedSection != CsrSrvSharedSectionHeap {
                *CsrSrvSharedStaticServerData.add((*ServerDll).ServerId as usize) =
                    (*ServerDll).SharedSection;
            }
        }
    }

    if Status < 0 {
        if !hServerDll.is_null() {
            LdrUnloadDll(hServerDll);
        }
        RtlFreeHeap(CsrHeap, 0, ServerDll as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrSrvClientConnect(
    ApiMessage: PCSR_API_MESSAGE,
    ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    let ClientConnect = &mut (*ApiMessage)
        .__bindgen_anon_1
        .__bindgen_anon_1
        .Data
        .CsrClientConnect;
    let ServerDll: PCSR_SERVER_DLL;
    let CurrentProcess: PCSR_PROCESS = (*CsrGetClientThread()).Process;

    *ReplyCode = CsrReplyImmediately;

    crate::dbg_print!(
        b"[CSRSRV] CliCon: ServerId=%lu InfoSize=%lu\n\0",
        ClientConnect.ServerId,
        ClientConnect.ConnectionInfoSize
    );

    if ClientConnect.ServerId >= CSR_SERVER_DLL_MAX {
        crate::dbg_print!(b"[CSRSRV] CliCon: ServerId>=MAX rejecting\n\0");
        return STATUS_TOO_MANY_NAMES;
    } else if CsrLoadedServerDll[ClientConnect.ServerId as usize].is_null() {
        crate::dbg_print!(
            b"[CSRSRV] CliCon: ServerDll[%lu] NULL\n\0",
            ClientConnect.ServerId
        );
        return STATUS_INVALID_PARAMETER;
    }

    if CsrValidateMessageBuffer(
        ApiMessage,
        core::ptr::addr_of_mut!(ClientConnect.ConnectionInfo) as *mut PVOID,
        ClientConnect.ConnectionInfoSize,
        core::mem::size_of::<u8>() as ULONG,
    ) == 0
    {
        return STATUS_INVALID_PARAMETER;
    }

    ServerDll = CsrLoadedServerDll[ClientConnect.ServerId as usize];

    let Status = if let Some(callback) = (*ServerDll).ConnectCallback {
        callback(
            CurrentProcess,
            ClientConnect.ConnectionInfo,
            &mut ClientConnect.ConnectionInfoSize,
        )
    } else {
        STATUS_SUCCESS
    };

    Status
}

#[no_mangle]
pub unsafe extern "system" fn CsrSrvCreateSharedSection(ParameterValue: *mut i8) -> NTSTATUS {
    let mut SizeValue = ParameterValue;
    let mut Size: ULONG = 0;
    let mut Status: NTSTATUS;
    let mut SectionSize: LARGE_INTEGER = core::mem::zeroed();
    let mut ViewSize: SIZE_T = 0;
    let Peb = NtCurrentPeb();

    if ParameterValue.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    while *SizeValue != 0 {
        if *SizeValue == b',' as i8 {
            *SizeValue = 0;
            SizeValue = SizeValue.add(1);
            break;
        } else {
            SizeValue = SizeValue.add(1);
        }
    }

    if *SizeValue == 0 {
        return STATUS_INVALID_PARAMETER;
    }

    let mut size_str: ANSI_STRING = core::mem::zeroed();
    RtlInitAnsiString(&mut size_str, SizeValue);
    Status = RtlCharToInteger(SizeValue, 0, &mut Size);
    if Status < 0 {
        return Status;
    }

    CsrSrvSharedSectionSize =
        ((Size * 1024 + CsrNtSysInfo.PageSize - 1) / CsrNtSysInfo.PageSize) * CsrNtSysInfo.PageSize;

    SectionSize.QuadPart = CsrSrvSharedSectionSize as i64;
    Status = NtCreateSection(
        &raw mut CsrSrvSharedSection,
        SECTION_ALL_ACCESS,
        core::ptr::null_mut(),
        &mut SectionSize,
        PAGE_EXECUTE_READWRITE,
        SEC_BASED | SEC_RESERVE,
        core::ptr::null_mut(),
    );
    if Status < 0 {
        return Status;
    }

    Status = NtMapViewOfSection(
        CsrSrvSharedSection,
        NtCurrentProcess(),
        &raw mut CsrSrvSharedSectionBase,
        0,
        0,
        core::ptr::null_mut(),
        &mut ViewSize,
        ViewUnmap,
        MEM_TOP_DOWN,
        PAGE_EXECUTE_READWRITE,
    );
    if Status < 0 {
        NtClose(CsrSrvSharedSection);
        return Status;
    }

    CsrSrvSharedSectionHeap = CsrSrvSharedSectionBase;

    if RtlCreateHeap(
        HEAP_ZERO_MEMORY | HEAP_CLASS_7,
        CsrSrvSharedSectionHeap,
        (CsrSrvSharedSectionSize as usize).try_into().unwrap(),
        PAGE_SIZE as SIZE_T,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
    .is_null()
    {
        NtUnmapViewOfSection(NtCurrentProcess(), CsrSrvSharedSectionBase);
        NtClose(CsrSrvSharedSection);
        return STATUS_NO_MEMORY;
    }

    CsrSrvSharedStaticServerData = RtlAllocateHeap(
        CsrSrvSharedSectionHeap,
        HEAP_ZERO_MEMORY,
        (CSR_SERVER_DLL_MAX as usize * core::mem::size_of::<PVOID>())
            .try_into()
            .unwrap(),
    ) as *mut PVOID;
    if CsrSrvSharedStaticServerData.is_null() {
        return STATUS_NO_MEMORY;
    }

    (*Peb).ReadOnlySharedMemoryBase = CsrSrvSharedSectionBase;
    (*Peb).ReadOnlySharedMemoryHeap = CsrSrvSharedSectionHeap;
    (*Peb).ReadOnlyStaticServerData = CsrSrvSharedStaticServerData as *mut *mut c_void;

    STATUS_SUCCESS
}


/* A WoW64 process has TWO CSR clients: the native 64-bit ntdll, which connects
 * during process creation, and the 32-bit guest's kernel32, which connects again
 * from its own DllMain. The second connect calls CsrSrvAttachSharedSection on a
 * process that ALREADY has the shared section mapped, and because the section is
 * SEC_BASED -- every mapping must land on one system-wide address -- the second
 * NtMapViewOfSection asks for an address its own first mapping occupies and comes
 * back STATUS_CONFLICTING_ADDRESSES. That refusal propagated out as a bare
 * c0000041 at the client, so no 32-bit guest has ever completed a CSR connect.
 *
 * Re-mapping is not what we want even if it were possible: the shared section is
 * a heap whose contents include ABSOLUTE pointers (CsrSrvSharedStaticServerData
 * and everything it points at), so a second view at a different address would
 * hand the client pointers into another address space. The existing mapping is
 * already correct and already at the right address -- the only thing owed is to
 * report it.
 *
 * We must be certain the occupant is OURS before reporting it as the shared heap;
 * treating a stranger's allocation as CSR's would give the client a foreign heap.
 * So verify structurally rather than assume: the section is a heap, so the page at
 * the based address is a HEAP_SEGMENT, and a HEAP_SEGMENT records its own
 * BaseAddress. Signature plus a self-consistent BaseAddress identifies it. */
const CSRP_HEAP_SEGMENT_SIGNATURE: ULONG = 0xffee_ffee; /* rtl-heap/src/types.rs:59 */
/* HEAP_SEGMENT field offsets depend on HEAP_ENTRY, which is 16 bytes on 64-bit
 * and 8 on 32-bit, so they are NOT a shared constant: 64-bit puts
 * SegmentSignature@16 and BaseAddress@48; 32-bit would put them at 8 and 28.
 * Rather than carry two layouts for a path that cannot occur on a 32-bit host --
 * WoW64 means a 32-bit guest under a 64-bit kernel, so there is no second CSR
 * client to collide with -- the check is 64-bit only and says so. A silently
 * wrong offset here would misidentify a heap and is exactly the failure this
 * function exists to prevent. */
#[cfg(target_pointer_width = "64")]
const CSRP_SEG_SIGNATURE_IDX: usize = 16 / 8; /* index into a [ULONGLONG] read */
#[cfg(target_pointer_width = "64")]
const CSRP_SEG_BASEADDRESS_IDX: usize = 48 / 8;

#[cfg(not(target_pointer_width = "64"))]
unsafe fn CsrpSharedSectionAlreadyMappedHere(_ProcessHandle: HANDLE) -> bool {
    false
}

#[cfg(target_pointer_width = "64")]
unsafe fn CsrpSharedSectionAlreadyMappedHere(ProcessHandle: HANDLE) -> bool {
    let mut Hdr: [ULONGLONG; 8] = [0; 8];
    let mut RdLen: SIZE_T = 0;
    let St = NtReadVirtualMemory(
        ProcessHandle,
        CsrSrvSharedSectionBase,
        &raw mut Hdr as PVOID,
        core::mem::size_of_val(&Hdr) as SIZE_T,
        &raw mut RdLen,
    );
    if St < 0 {
        return false;
    }
    let Signature = (Hdr[CSRP_SEG_SIGNATURE_IDX] & 0xffff_ffff) as ULONG;
    let BaseAddress = Hdr[CSRP_SEG_BASEADDRESS_IDX] as PVOID;
    Signature == CSRP_HEAP_SEGMENT_SIGNATURE && BaseAddress == CsrSrvSharedSectionBase
}

#[no_mangle]
pub unsafe extern "system" fn CsrSrvAttachSharedSection(
    CsrProcess: PCSR_PROCESS,
    ConnectInfo: PCSR_API_CONNECTINFO,
) -> NTSTATUS {
    let Status: NTSTATUS;
    let mut ViewSize: SIZE_T = 0;

    if !CsrProcess.is_null() {
        Status = NtMapViewOfSection(
            CsrSrvSharedSection,
            (*CsrProcess).ProcessHandle,
            &raw mut CsrSrvSharedSectionBase,
            0,
            0,
            core::ptr::null_mut(),
            &mut ViewSize,
            ViewUnmap,
            SEC_NO_CHANGE,
            PAGE_EXECUTE_READ,
        );
        if Status == STATUS_CONFLICTING_ADDRESSES && CsrpSharedSectionAlreadyMappedHere((*CsrProcess).ProcessHandle) {
            /* Already mapped, verified ours: the second CSR client in a WoW64
             * process. Fall through and report the existing base. */
        } else if Status < 0 {
            crate::dbg_print!(
                b"[CSRSRV] AttachSharedSection: map at fixed base %p FAILED st=%lx\n\0",
                CsrSrvSharedSectionBase,
                Status
            );
            /* Not our heap (or unreadable) -- a genuine foreign collision, which
             * is a different bug. Name the occupant instead of guessing. */
            let mut Mbi: _MEMORY_BASIC_INFORMATION = core::mem::zeroed();
            let mut RetLen: SIZE_T = 0;
            let QSt = NtQueryVirtualMemory(
                (*CsrProcess).ProcessHandle,
                CsrSrvSharedSectionBase,
                MemoryBasicInformation,
                &raw mut Mbi as PVOID,
                core::mem::size_of::<_MEMORY_BASIC_INFORMATION>() as SIZE_T,
                &raw mut RetLen,
            );
            if QSt >= 0 {
                crate::dbg_print!(
                    b"[CSRSRV]   occupant: alloc=%p size=%p state=%lx protect=%lx type=%lx\n\0",
                    Mbi.AllocationBase,
                    Mbi.RegionSize as PVOID,
                    Mbi.State,
                    Mbi.Protect,
                    Mbi.Type
                );
            }
            return Status;
        }
    }

    (*ConnectInfo).SharedSectionBase = CsrSrvSharedSectionBase;
    (*ConnectInfo).SharedSectionHeap = CsrSrvSharedSectionHeap;
    (*ConnectInfo).SharedStaticServerData = CsrSrvSharedStaticServerData as PVOID;

    STATUS_SUCCESS
}

#[cfg(not(feature = "ws03"))]
#[no_mangle]
pub unsafe extern "system" fn CsrSrvIdentifyAlertableThread(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    let CsrThread = CsrGetClientThread();

    (*CsrThread).Flags |= CsrThreadAlertable;

    STATUS_SUCCESS
}

#[cfg(not(feature = "ws03"))]
#[no_mangle]
pub unsafe extern "system" fn CsrSrvSetPriorityClass(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn CsrSrvUnusedFunction(
    _ApiMessage: PCSR_API_MESSAGE,
    _ReplyCode: *mut CSR_REPLY_CODE,
) -> NTSTATUS {
    STATUS_INVALID_PARAMETER
}

#[no_mangle]
pub unsafe extern "system" fn CsrSetCallingSpooler(_Reserved: ULONG) {}

#[no_mangle]
pub unsafe extern "system" fn CsrUnhandledExceptionFilter(
    ExceptionInfo: PEXCEPTION_POINTERS,
) -> EXCEPTION_DISPOSITION {
    let mut DebuggerInfo: SYSTEM_KERNEL_DEBUGGER_INFORMATION = core::mem::zeroed();
    let mut Result: EXCEPTION_DISPOSITION = EXCEPTION_EXECUTE_HANDLER;
    let mut OldValue: BOOLEAN = 0;
    let mut Status: NTSTATUS;
    let mut ErrorSource: _UNICODE_STRING = core::mem::zeroed();
    let mut ErrorParameters: [ULONG_PTR; 4] = [0; 4];
    let mut Response: ULONG = 0;

    Status = NtQuerySystemInformation(
        SystemKernelDebuggerInformation,
        &mut DebuggerInfo as *mut _ as PVOID,
        core::mem::size_of::<SYSTEM_KERNEL_DEBUGGER_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );

    if ((*NtCurrentPeb()).SessionId != 0)
        && (Status >= 0)
        && (DebuggerInfo.KernelDebuggerEnabled != 0)
    {
        Result = RtlUnhandledExceptionFilter(ExceptionInfo) as u32;
        if Result != EXCEPTION_CONTINUE_EXECUTION.try_into().unwrap() {
            Status = RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, 1, 1, &mut OldValue);

            if Status == STATUS_NO_TOKEN {
                Status = RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE, 1, 0, &mut OldValue);
            }
            if Status < 0 {
                NtTerminateProcess(
                    NtCurrentProcess(),
                    (*(*ExceptionInfo).ExceptionRecord)
                        .ExceptionCode
                        .try_into()
                        .unwrap(),
                );
                return Result;
            }

            static WINDOWS_SUBSYSTEM: [u16; 17] = [
                b'W' as u16,
                b'i' as u16,
                b'n' as u16,
                b'd' as u16,
                b'o' as u16,
                b'w' as u16,
                b's' as u16,
                b' ' as u16,
                b'S' as u16,
                b'u' as u16,
                b'b' as u16,
                b'S' as u16,
                b'y' as u16,
                b's' as u16,
                b't' as u16,
                b'e' as u16,
                b'm' as u16,
            ];
            ErrorSource.Length = 34;
            ErrorSource.MaximumLength = 34;
            ErrorSource.Buffer = WINDOWS_SUBSYSTEM.as_ptr() as *mut u16;

            ErrorParameters[0] = &ErrorSource as *const _ as ULONG_PTR;
            ErrorParameters[1] = (*(*ExceptionInfo).ExceptionRecord).ExceptionCode as ULONG_PTR;
            ErrorParameters[2] = (*(*ExceptionInfo).ExceptionRecord).ExceptionAddress as ULONG_PTR;
            ErrorParameters[3] = (*ExceptionInfo).ContextRecord as ULONG_PTR;

            NtRaiseHardError(
                STATUS_SYSTEM_PROCESS_TERMINATED,
                4,
                1,
                ErrorParameters.as_mut_ptr(),
                OptionShutdownSystem,
                &mut Response,
            );
        }

        NtTerminateProcess(
            NtCurrentProcess(),
            (*(*ExceptionInfo).ExceptionRecord)
                .ExceptionCode
                .try_into()
                .unwrap(),
        );
    }

    Result
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Client/Server Runtime SubSystem
//  * FILE:            subsystems/win32/csrsrv/server.c
//  * PURPOSE:         CSR Server DLL Server Functions
//  * PROGRAMMERS:     Alex Ionescu (alex@relsoft.net)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "srv.h"
//
// #include <ndk/mmfuncs.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* DATA ***********************************************************************/
//
// PCSR_SERVER_DLL CsrLoadedServerDll[CSR_SERVER_DLL_MAX];
// PVOID CsrSrvSharedSectionHeap = NULL;
// PVOID CsrSrvSharedSectionBase = NULL;
// PVOID *CsrSrvSharedStaticServerData = NULL;
// ULONG CsrSrvSharedSectionSize = 0;
// HANDLE CsrSrvSharedSection = NULL;
//
// PCSR_API_ROUTINE CsrServerApiDispatchTable[CsrpMaxApiNumber] =
// {
//     CsrSrvClientConnect,
//     CsrSrvUnusedFunction, // <= WinNT4: CsrSrvThreadConnect
//     CsrSrvUnusedFunction, // <= WinNT4: CsrSrvProfileControl
// #if (NTDDI_VERSION < NTDDI_WS03)
//     CsrSrvIdentifyAlertableThread
//     CsrSrvSetPriorityClass
// #else
//     CsrSrvUnusedFunction, // <= WinXP : CsrSrvIdentifyAlertableThread
//     CsrSrvUnusedFunction  // <= WinXP : CsrSrvSetPriorityClass
// #endif
// };
//
// BOOLEAN CsrServerApiServerValidTable[CsrpMaxApiNumber] =
// {
//     TRUE,
//     FALSE,
//     FALSE,
// #if (NTDDI_VERSION < NTDDI_WS03)
//     TRUE,
//     TRUE
// #else
//     FALSE,
//     FALSE
// #endif
// };
//
// /*
//  * On Windows Server 2003, CSR Servers contain
//  * the API Names Table only in Debug Builds.
//  */
// #ifdef CSR_DBG
// PCHAR CsrServerApiNameTable[CsrpMaxApiNumber] =
// {
//     "ClientConnect",
//     "ThreadConnect",
//     "ProfileControl",
//     "IdentifyAlertableThread",
//     "SetPriorityClass"
// };
// #endif
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// /*++
//  * @name CsrServerDllInitialization
//  * @implemented NT4
//  *
//  * The CsrServerDllInitialization is the initialization routine
//  * for this Server DLL.
//  *
//  * @param LoadedServerDll
//  *        Pointer to the CSR Server DLL structure representing this Server DLL.
//  *
//  * @return STATUS_SUCCESS.
//  *
//  * @remarks None.
//  *
//  *--*/
// CSR_SERVER_DLL_INIT(CsrServerDllInitialization)
// {
//     /* Setup the DLL Object */
//     LoadedServerDll->ApiBase = CSRSRV_FIRST_API_NUMBER;
//     LoadedServerDll->HighestApiSupported = CsrpMaxApiNumber;
//     LoadedServerDll->DispatchTable = CsrServerApiDispatchTable;
//     LoadedServerDll->ValidTable = CsrServerApiServerValidTable;
// #ifdef CSR_DBG
//     LoadedServerDll->NameTable = CsrServerApiNameTable;
// #endif
//     LoadedServerDll->SizeOfProcessData = 0;
//     LoadedServerDll->ConnectCallback = NULL;
//     LoadedServerDll->DisconnectCallback = NULL;
//
//     /* All done */
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrLoadServerDll
//  * @implemented NT4
//  *
//  * The CsrLoadServerDll routine loads a CSR Server DLL and calls its entrypoint.
//  *
//  * @param DllString
//  *        Pointer to the CSR Server DLL to load and call.
//  *
//  * @param EntryPoint
//  *        Pointer to the name of the server's initialization function.
//  *        If this parameter is NULL, the default ServerDllInitialize
//  *        will be assumed.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrLoadServerDll(IN PCHAR DllString,
//                  IN PCHAR EntryPoint OPTIONAL,
//                  IN ULONG ServerId)
// {
//     NTSTATUS Status;
//     ANSI_STRING DllName;
//     UNICODE_STRING TempString, ErrorString;
//     ULONG_PTR Parameters[2];
//     HANDLE hServerDll = NULL;
//     ULONG Size;
//     PCSR_SERVER_DLL ServerDll;
//     STRING EntryPointString;
//     PCSR_SERVER_DLL_INIT_CALLBACK ServerDllInitProcedure;
//     ULONG Response;
//
//     /* Check if it's beyond the maximum we support */
//     if (ServerId >= CSR_SERVER_DLL_MAX) return STATUS_TOO_MANY_NAMES;
//
//     /* Check if it's already been loaded */
//     if (CsrLoadedServerDll[ServerId]) return STATUS_INVALID_PARAMETER;
//
//     /* Convert the name to Unicode */
//     ASSERT(DllString != NULL);
//     RtlInitAnsiString(&DllName, DllString);
//     Status = RtlAnsiStringToUnicodeString(&TempString, &DllName, TRUE);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* If we are loading ourselves, don't actually load us */
//     if (ServerId != CSRSRV_SERVERDLL_INDEX)
//     {
//         /* Load the DLL */
//         Status = LdrLoadDll(NULL, 0, &TempString, &hServerDll);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Setup error parameters */
//             Parameters[0] = (ULONG_PTR)&TempString;
//             Parameters[1] = (ULONG_PTR)&ErrorString;
//             RtlInitUnicodeString(&ErrorString, L"Default Load Path");
//
//             /* Send a hard error */
//             NtRaiseHardError(Status,
//                              2,
//                              3,
//                              Parameters,
//                              OptionOk,
//                              &Response);
//         }
//
//         /* Get rid of the string */
//         RtlFreeUnicodeString(&TempString);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Allocate a CSR DLL Object */
//     Size = sizeof(CSR_SERVER_DLL) + DllName.MaximumLength;
//     ServerDll = RtlAllocateHeap(CsrHeap, HEAP_ZERO_MEMORY, Size);
//     if (!ServerDll)
//     {
//         if (hServerDll) LdrUnloadDll(hServerDll);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Set up the Object */
//     ServerDll->Length = Size;
//     ServerDll->SizeOfProcessData = 0;
//     ServerDll->SharedSection = CsrSrvSharedSectionHeap; // Send to the server dll our shared heap pointer.
//     ServerDll->Name.Length = DllName.Length;
//     ServerDll->Name.MaximumLength = DllName.MaximumLength;
//     ServerDll->Name.Buffer = (PCHAR)(ServerDll + 1);
//     if (DllName.Length)
//     {
//         strncpy(ServerDll->Name.Buffer, DllName.Buffer, DllName.Length);
//     }
//     ServerDll->ServerId = ServerId;
//     ServerDll->ServerHandle = hServerDll;
//
//     /* Now get the entrypoint */
//     if (hServerDll)
//     {
//         /* Initialize a string for the entrypoint, or use the default */
//         RtlInitAnsiString(&mut EntryPointString,
//                           EntryPoint ? EntryPoint : "ServerDllInitialization");
//
//         /* Get a pointer to it */
//         Status = LdrGetProcedureAddress(hServerDll,
//                                         &mut EntryPointString,
//                                         0,
//                                         (PVOID)&ServerDllInitProcedure);
//     }
//     else
//     {
//         /* No handle, so we are loading ourselves */
// #ifdef CSR_DBG
//         RtlInitAnsiString(&mut EntryPointString, "CsrServerDllInitialization");
// #endif
//         ServerDllInitProcedure = CsrServerDllInitialization;
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Check if we got the pointer, and call it */
//     if (NT_SUCCESS(Status))
//     {
//         /* Call the Server DLL entrypoint */
//         _SEH2_TRY
//         {
//             Status = ServerDllInitProcedure(ServerDll);
//         }
//         _SEH2_EXCEPT(CsrUnhandledExceptionFilter(_SEH2_GetExceptionInformation()))
//         {
//             Status = _SEH2_GetExceptionCode();
// #ifdef CSR_DBG
//             DPRINT1("CSRSS: Exception 0x%lx while calling Server DLL entrypoint %Z!%Z()\n",
//                     Status, &DllName, &mut EntryPointString);
// #endif
//         }
//         _SEH2_END;
//
//         if (NT_SUCCESS(Status))
//         {
//             /*
//              * Add this Server's Per-Process Data Size to the total that each
//              * process will need.
//              */
//             CsrTotalPerProcessDataLength += ServerDll->SizeOfProcessData;
//
//             /* Save the pointer in our list */
//             CsrLoadedServerDll[ServerDll->ServerId] = ServerDll;
//
//             /* Does it use our generic heap? */
//             if (ServerDll->SharedSection != CsrSrvSharedSectionHeap)
//             {
//                 /* No, save the pointer to its shared section in our list */
//                 CsrSrvSharedStaticServerData[ServerDll->ServerId] = ServerDll->SharedSection;
//             }
//         }
//     }
//
//     if (!NT_SUCCESS(Status))
//     {
//         /* Server Init failed, unload it */
//         if (hServerDll) LdrUnloadDll(hServerDll);
//
//         /* Delete the Object */
//         RtlFreeHeap(CsrHeap, 0, ServerDll);
//     }
//
//     /* Return to caller */
//     return Status;
// }
//
// /*++
//  * @name CsrSrvClientConnect
//  *
//  * The CsrSrvClientConnect CSR API handles a new connection to a server DLL.
//  *
//  * @param ApiMessage
//  *        Pointer to the CSR API Message for this request.
//  *
//  * @param ReplyCode
//  *        Optional reply to this request.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_INVALID_PARAMETER
//  *         or STATUS_TOO_MANY_NAMES in case of failure.
//  *
//  * @remarks None.
//  *
//  *--*/
// CSR_API(CsrSrvClientConnect)
// {
//     NTSTATUS Status;
//     PCSR_CLIENT_CONNECT ClientConnect = &ApiMessage->Data.CsrClientConnect;
//     PCSR_SERVER_DLL ServerDll;
//     PCSR_PROCESS CurrentProcess = CsrGetClientThread()->Process;
//
//     /* Set default reply */
//     *ReplyCode = CsrReplyImmediately;
//
//     /* Validate the ServerID */
//     if (ClientConnect->ServerId >= CSR_SERVER_DLL_MAX)
//     {
//         return STATUS_TOO_MANY_NAMES;
//     }
//     else if (!CsrLoadedServerDll[ClientConnect->ServerId])
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Validate the Message Buffer */
//     if (!(CsrValidateMessageBuffer(ApiMessage,
//                                    &ClientConnect->ConnectionInfo,
//                                    ClientConnect->ConnectionInfoSize,
//                                    sizeof(BYTE))))
//     {
//         /* Fail due to buffer overflow or other invalid buffer */
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     /* Load the Server DLL */
//     ServerDll = CsrLoadedServerDll[ClientConnect->ServerId];
//
//     /* Check if it has a Connect Callback */
//     if (ServerDll->ConnectCallback)
//     {
//         /* Call the callback */
//         Status = ServerDll->ConnectCallback(CurrentProcess,
//                                             ClientConnect->ConnectionInfo,
//                                             &ClientConnect->ConnectionInfoSize);
//     }
//     else
//     {
//         /* Assume success */
//         Status = STATUS_SUCCESS;
//     }
//
//     /* Return status */
//     return Status;
// }
//
// /*++
//  * @name CsrSrvCreateSharedSection
//  *
//  * The CsrSrvCreateSharedSection creates the Shared Section that all
//  * CSR Server DLLs and Clients can use to share data.
//  *
//  * @param ParameterValue
//  *        Specially formatted string from our registry command-line which
//  *        specifies various arguments for the shared section.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrSrvCreateSharedSection(IN PCHAR ParameterValue)
// {
//     PCHAR SizeValue = ParameterValue;
//     ULONG Size;
//     NTSTATUS Status;
//     LARGE_INTEGER SectionSize;
//     SIZE_T ViewSize = 0;
//     PPEB Peb = NtCurrentPeb();
//
//     /* If there's no parameter, fail */
//     if (!ParameterValue) return STATUS_INVALID_PARAMETER;
//
//     /* Find the first comma, and null terminate */
//     while (*SizeValue)
//     {
//         if (*SizeValue == ',')
//         {
//             *SizeValue++ = ANSI_NULL;
//             break;
//         }
//         else
//         {
//             SizeValue++;
//         }
//     }
//
//     /* Make sure it's valid */
//     if (!*SizeValue) return STATUS_INVALID_PARAMETER;
//
//     /* Convert it to an integer */
//     Status = RtlCharToInteger(SizeValue, 0, &Size);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Multiply by 1024 entries and round to page size */
//     CsrSrvSharedSectionSize = ROUND_UP(Size * 1024, CsrNtSysInfo.PageSize);
//
//     /* Create the Secion */
//     SectionSize.LowPart = CsrSrvSharedSectionSize;
//     SectionSize.HighPart = 0;
//     Status = NtCreateSection(&CsrSrvSharedSection,
//                              SECTION_ALL_ACCESS,
//                              NULL,
//                              &SectionSize,
//                              PAGE_EXECUTE_READWRITE,
//                              SEC_BASED | SEC_RESERVE,
//                              NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Map the section */
//     Status = NtMapViewOfSection(CsrSrvSharedSection,
//                                 NtCurrentProcess(),
//                                 &CsrSrvSharedSectionBase,
//                                 0,
//                                 0,
//                                 NULL,
//                                 &ViewSize,
//                                 ViewUnmap,
//                                 MEM_TOP_DOWN,
//                                 PAGE_EXECUTE_READWRITE);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Fail */
//         NtClose(CsrSrvSharedSection);
//         return Status;
//     }
//
//     /* FIXME: Write the value to registry */
//
//     /* The Heap is the same place as the Base */
//     CsrSrvSharedSectionHeap = CsrSrvSharedSectionBase;
//
//     /* Create the heap */
//     if (!(RtlCreateHeap(HEAP_ZERO_MEMORY | HEAP_CLASS_7,
//                         CsrSrvSharedSectionHeap,
//                         CsrSrvSharedSectionSize,
//                         PAGE_SIZE,
//                         0,
//                         0)))
//     {
//         /* Failure, unmap section and return */
//         NtUnmapViewOfSection(NtCurrentProcess(), CsrSrvSharedSectionBase);
//         NtClose(CsrSrvSharedSection);
//         return STATUS_NO_MEMORY;
//     }
//
//     /* Now allocate space from the heap for the Shared Data */
//     CsrSrvSharedStaticServerData = RtlAllocateHeap(CsrSrvSharedSectionHeap,
//                                                    HEAP_ZERO_MEMORY,
//                                                    CSR_SERVER_DLL_MAX * sizeof(PVOID));
//     if (!CsrSrvSharedStaticServerData) return STATUS_NO_MEMORY;
//
//     /* Write the values to the PEB */
//     Peb->ReadOnlySharedMemoryBase = CsrSrvSharedSectionBase;
//     Peb->ReadOnlySharedMemoryHeap = CsrSrvSharedSectionHeap;
//     Peb->ReadOnlyStaticServerData = CsrSrvSharedStaticServerData;
//
//     /* Return */
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrSrvAttachSharedSection
//  *
//  * The CsrSrvAttachSharedSection maps the CSR Shared Section into a new
//  * CSR Process' address space, and returns the pointers to the section
//  * through the Connection Info structure.
//  *
//  * @param CsrProcess
//  *        Pointer to the CSR Process that is attempting a connection.
//  *
//  * @param ConnectInfo
//  *        Pointer to the CSR Connection Info structure for the incoming
//  *        connection.
//  *
//  * @return STATUS_SUCCESS in case of success, STATUS_UNSUCCESSFUL otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// NTSTATUS
// NTAPI
// CsrSrvAttachSharedSection(IN PCSR_PROCESS CsrProcess OPTIONAL,
//                           OUT PCSR_API_CONNECTINFO ConnectInfo)
// {
//     NTSTATUS Status;
//     SIZE_T ViewSize = 0;
//
//     /* Check if we have a process */
//     if (CsrProcess)
//     {
//         /* Map the section into this process */
//         Status = NtMapViewOfSection(CsrSrvSharedSection,
//                                     CsrProcess->ProcessHandle,
//                                     &CsrSrvSharedSectionBase,
//                                     0,
//                                     0,
//                                     NULL,
//                                     &ViewSize,
//                                     ViewUnmap,
//                                     SEC_NO_CHANGE,
//                                     PAGE_EXECUTE_READ);
//         if (!NT_SUCCESS(Status)) return Status;
//     }
//
//     /* Write the values in the Connection Info structure */
//     ConnectInfo->SharedSectionBase = CsrSrvSharedSectionBase;
//     ConnectInfo->SharedSectionHeap = CsrSrvSharedSectionHeap;
//     ConnectInfo->SharedStaticServerData = CsrSrvSharedStaticServerData;
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// #if (NTDDI_VERSION < NTDDI_WS03)
//
// /*++
//  * @name CsrSrvIdentifyAlertableThread
//  * @implemented NT4, up to WinXP
//  *
//  * The CsrSrvIdentifyAlertableThread CSR API marks a CSR Thread as alertable.
//  *
//  * @param ApiMessage
//  *        Pointer to the CSR API Message for this request.
//  *
//  * @param ReplyCode
//  *        Pointer to an optional reply to this request.
//  *
//  * @return STATUS_SUCCESS.
//  *
//  * @remarks Deprecated.
//  *
//  *--*/
// CSR_API(CsrSrvIdentifyAlertableThread)
// {
//     PCSR_THREAD CsrThread = CsrGetClientThread();
//
//     UNREFERENCED_PARAMETER(ApiMessage);
//     UNREFERENCED_PARAMETER(ReplyCode);
//
//     /* Set the alertable flag */
//     CsrThread->Flags |= CsrThreadAlertable;
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// /*++
//  * @name CsrSrvSetPriorityClass
//  * @implemented NT4, up to WinXP
//  *
//  * The CsrSrvSetPriorityClass CSR API is deprecated.
//  *
//  * @param ApiMessage
//  *        Pointer to the CSR API Message for this request.
//  *
//  * @param ReplyCode
//  *        Pointer to an optional reply to this request.
//  *
//  * @return STATUS_SUCCESS.
//  *
//  * @remarks Deprecated.
//  *
//  *--*/
// CSR_API(CsrSrvSetPriorityClass)
// {
//     UNREFERENCED_PARAMETER(ApiMessage);
//     UNREFERENCED_PARAMETER(ReplyCode);
//
//     /* Deprecated */
//     return STATUS_SUCCESS;
// }
//
// #endif // (NTDDI_VERSION < NTDDI_WS03)
//
// /*++
//  * @name CsrSrvUnusedFunction
//  * @implemented NT4
//  *
//  * The CsrSrvUnusedFunction CSR API is a stub for deprecated APIs.
//  *
//  * @param ApiMessage
//  *        Pointer to the CSR API Message for this request.
//  *
//  * @param ReplyCode
//  *        Pointer to an optional reply to this request.
//  *
//  * @return STATUS_INVALID_PARAMETER.
//  *
//  * @remarks CsrSrvSetPriorityClass does not use this stub because
//  *          it must return success.
//  *
//  *--*/
// CSR_API(CsrSrvUnusedFunction)
// {
//     UNREFERENCED_PARAMETER(ApiMessage);
//     UNREFERENCED_PARAMETER(ReplyCode);
//
//     /* Deprecated */
//     return STATUS_INVALID_PARAMETER;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*++
//  * @name CsrSetCallingSpooler
//  * @implemented NT4
//  *
//  * the CsrSetCallingSpooler routine is deprecated.
//  *
//  * @param Reserved
//  *        Deprecated
//  *
//  * @return None.
//  *
//  * @remarks This routine was used in archaic versions of NT for Printer Drivers.
//  *
//  *--*/
// VOID
// NTAPI
// CsrSetCallingSpooler(ULONG Reserved)
// {
//     /* Deprecated */
//     return;
// }
//
// /*++
//  * @name CsrUnhandledExceptionFilter
//  * @implemented NT5
//  *
//  * The CsrUnhandledExceptionFilter routine handles all exceptions
//  * within SEH-protected blocks.
//  *
//  * @param ExceptionPointers
//  *        System-defined Argument.
//  *
//  * @return EXCEPTION_EXECUTE_HANDLER.
//  *
//  * @remarks None.
//  *
//  *--*/
// EXCEPTION_DISPOSITION
// NTAPI
// CsrUnhandledExceptionFilter(IN PEXCEPTION_POINTERS ExceptionInfo)
// {
//     SYSTEM_KERNEL_DEBUGGER_INFORMATION DebuggerInfo;
//     EXCEPTION_DISPOSITION Result = EXCEPTION_EXECUTE_HANDLER;
//     BOOLEAN OldValue;
//     NTSTATUS Status;
//     UNICODE_STRING ErrorSource;
//     ULONG_PTR ErrorParameters[4];
//     ULONG Response;
//
//     DPRINT1("CsrUnhandledExceptionFilter called\n");
//
//     /* Check if a debugger is installed */
//     Status = NtQuerySystemInformation(SystemKernelDebuggerInformation,
//                                       &DebuggerInfo,
//                                       sizeof(DebuggerInfo),
//                                       NULL);
//
//     /* Check if this is Session 0, and the Debugger is Enabled */
//     if ((NtCurrentPeb()->SessionId != 0) && (NT_SUCCESS(Status)) &&
//         (DebuggerInfo.KernelDebuggerEnabled))
//     {
//         /* Call the Unhandled Exception Filter */
//         Result = RtlUnhandledExceptionFilter(ExceptionInfo) as u32;
//         if (Result != EXCEPTION_CONTINUE_EXECUTION)
//         {
//             /* We're going to raise an error. Get Shutdown Privilege first */
//             Status = RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE,
//                                         TRUE,
//                                         TRUE,
//                                         &OldValue);
//
//             /* Use the Process token if that failed */
//             if (Status == STATUS_NO_TOKEN)
//             {
//                 Status = RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE,
//                                             TRUE,
//                                             FALSE,
//                                             &OldValue);
//             }
//             if (!NT_SUCCESS(Status))
//             {
//                 DPRINT1("CsrUnhandledExceptionFilter(): RtlAdjustPrivilege(SE_SHUTDOWN_PRIVILEGE) failed, Status = 0x%08lx\n", Status);
//                 goto NoPrivilege;
//             }
//
//             /* Initialize our Name String */
//             RtlInitUnicodeString(&ErrorSource, L"Windows SubSystem");
//
//             /* Set the parameters */
//             ErrorParameters[0] = (ULONG_PTR)&ErrorSource;
//             ErrorParameters[1] = ExceptionInfo->ExceptionRecord->ExceptionCode;
//             ErrorParameters[2] = (ULONG_PTR)ExceptionInfo->ExceptionRecord->ExceptionAddress;
//             ErrorParameters[3] = (ULONG_PTR)ExceptionInfo->ContextRecord;
//
//             /* Bugcheck */
//             NtRaiseHardError(STATUS_SYSTEM_PROCESS_TERMINATED,
//                              4,
//                              1,
//                              ErrorParameters,
//                              OptionShutdownSystem,
//                              &Response);
//         }
//
// NoPrivilege:
//         /* Just terminate us */
//         NtTerminateProcess(NtCurrentProcess(),
//                            ExceptionInfo->ExceptionRecord->ExceptionCode);
//     }
//
//     return Result;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
