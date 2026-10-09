use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::macros::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
// PsGetCurrentProcess declared in local extern block returning ddk::PEPROCESS
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    _DEBUG_OBJECT, _DEBUG_EVENT, _DBGKM_MSG, _DBGKM_EXCEPTION, _DBGUI_WAIT_STATE_CHANGE,
    _OBJECT_TYPE_INITIALIZER, _KAPC_STATE, _PEB, _PEB_LDR_DATA,
    _LDR_DATA_TABLE_ENTRY,
    _EPROCESS, _ETHREAD,
    _FAST_MUTEX, _GENERIC_MAPPING, _INFORMATION_CLASS_INFO,
    _UNICODE_STRING, _OBJECT_ATTRIBUTES, _IO_STATUS_BLOCK, _CLIENT_ID,
    POBJECT_TYPE, PDEBUG_OBJECT, PDEBUG_EVENT, PDBGKM_MSG, PDBGUI_WAIT_STATE_CHANGE,
    PEPROCESS, PETHREAD, PPEB, PPEB_LDR_DATA, PLDR_DATA_TABLE_ENTRY,
    PIMAGE_NT_HEADERS, PHANDLE, POBJECT_ATTRIBUTES,
    PCLIENT_ID, PEXCEPTION_RECORD, PPORT_MESSAGE,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE,
    ExAcquireRundownProtection, ExReleaseRundownProtection,
    ExGetPreviousMode, ExInitializeFastMutex, ExAcquireFastMutex, ExReleaseFastMutex,
    KeInitializeEvent, KeStackAttachProcess, KeUnstackDetachProcess,
    KeWaitForSingleObject, LpcRequestWaitReplyPort, MmGetFileNameForAddress,
    ObCloseHandle, ObOpenObjectByPointer, ObDuplicateObject, ObCreateObject,
    ObInsertObject, ObReferenceObjectByHandle, ObCreateObjectType,
    PsGetNextProcessThread, PsGetNextProcess, PsResumeThread, PsSuspendThread,
    PsTerminateProcess,
    ZwOpenFile, ZwFlushInstructionCache,
    RtlImageNtHeader, RtlFreeUnicodeString, RtlInitUnicodeString,
    DEBUGOBJECTINFOCLASS, DEBUG_OBJECT_KILL_PROCESS_ON_EXIT_INFORMATION,
    PDEBUG_OBJECT_KILL_PROCESS_ON_EXIT_INFORMATION,
    DBGKM_APINUMBER,
    _DBGKM_APINUMBER_DbgKmExceptionApi,
    _DBGKM_APINUMBER_DbgKmCreateThreadApi,
    _DBGKM_APINUMBER_DbgKmCreateProcessApi,
    _DBGKM_APINUMBER_DbgKmExitThreadApi,
    _DBGKM_APINUMBER_DbgKmExitProcessApi,
    _DBGKM_APINUMBER_DbgKmLoadDllApi,
    _DBGKM_APINUMBER_DbgKmUnloadDllApi,
};

const TAG_DEBUG_EVENT: u32 = 0x45676244;
const DBGK_ALL_FLAGS: ULONG = 1;
const DBGK_KILL_PROCESS_ON_EXIT: ULONG = 1;

const DEBUG_EVENT_NOWAIT: ULONG = 0x2;
const DEBUG_EVENT_INACTIVE: ULONG = 0x4;
const DEBUG_EVENT_RELEASE: ULONG = 0x8;
const DEBUG_EVENT_PROTECT_FAILED: ULONG = 0x10;
const DEBUG_EVENT_SUSPEND: ULONG = 0x20;
const DEBUG_EVENT_READ: ULONG = 0x1;

const PSF_CREATE_REPORTED_BIT: ULONG = 0x00000001;
const PSF_NO_DEBUG_INHERIT_BIT: ULONG = 0x00000002;
const CT_SKIP_CREATION_MSG_BIT: ULONG = 0x80;

#[allow(non_upper_case_globals)]
const DbgKmExceptionApi: DBGKM_APINUMBER = _DBGKM_APINUMBER_DbgKmExceptionApi;
#[allow(non_upper_case_globals)]
const DbgKmCreateThreadApi: DBGKM_APINUMBER = _DBGKM_APINUMBER_DbgKmCreateThreadApi;
#[allow(non_upper_case_globals)]
const DbgKmCreateProcessApi: DBGKM_APINUMBER = _DBGKM_APINUMBER_DbgKmCreateProcessApi;
#[allow(non_upper_case_globals)]
const DbgKmExitThreadApi: DBGKM_APINUMBER = _DBGKM_APINUMBER_DbgKmExitThreadApi;
#[allow(non_upper_case_globals)]
const DbgKmExitProcessApi: DBGKM_APINUMBER = _DBGKM_APINUMBER_DbgKmExitProcessApi;
#[allow(non_upper_case_globals)]
const DbgKmLoadDllApi: DBGKM_APINUMBER = _DBGKM_APINUMBER_DbgKmLoadDllApi;
#[allow(non_upper_case_globals)]
const DbgKmUnloadDllApi: DBGKM_APINUMBER = _DBGKM_APINUMBER_DbgKmUnloadDllApi;

#[allow(non_upper_case_globals)]
const DbgCreateProcessStateChange: ULONG = 1;
#[allow(non_upper_case_globals)]
const DbgCreateThreadStateChange: ULONG = 2;
#[allow(non_upper_case_globals)]
const DbgExceptionStateChange: ULONG = 3;
#[allow(non_upper_case_globals)]
const DbgBreakpointStateChange: ULONG = 4;
#[allow(non_upper_case_globals)]
const DbgSingleStepStateChange: ULONG = 5;
#[allow(non_upper_case_globals)]
const DbgLoadDllStateChange: ULONG = 6;
#[allow(non_upper_case_globals)]
const DbgUnloadDllStateChange: ULONG = 7;
#[allow(non_upper_case_globals)]
const DbgExitThreadStateChange: ULONG = 8;
#[allow(non_upper_case_globals)]
const DbgExitProcessStateChange: ULONG = 9;

const LPC_DEBUG_EVENT: i16 = 0x08;
const LPC_EXCEPTION: i16 = 0x07;

const DBG_CONTINUE: NTSTATUS = 0x00010002;
const DBG_EXCEPTION_HANDLED: NTSTATUS = 0x00010001;
const DBG_EXCEPTION_NOT_HANDLED: NTSTATUS = 0x80010001u32 as NTSTATUS;
const DBG_TERMINATE_THREAD: NTSTATUS = 0x40010003;
const DBG_TERMINATE_PROCESS: NTSTATUS = 0x40010004;

const STATUS_DEBUGGER_INACTIVE: NTSTATUS = 0xC0000354u32 as NTSTATUS;
const STATUS_PORT_NOT_SET: NTSTATUS = 0xC0000353u32 as NTSTATUS;
const STATUS_BREAKPOINT: NTSTATUS = 0x80000003u32 as NTSTATUS;
const STATUS_SINGLE_STEP: NTSTATUS = 0x80000004u32 as NTSTATUS;
const STATUS_TIMEOUT: NTSTATUS = 0x00000102;
const STATUS_USER_APC: NTSTATUS = 0x000000C0;

const FILE_SHARE_READ: ULONG = 0x00000001;
const FILE_SHARE_WRITE: ULONG = 0x00000002;
const FILE_SHARE_DELETE: ULONG = 0x00000004;
const FILE_SYNCHRONOUS_IO_NONALERT: ULONG = 0x00000020;

const PORT_MAXIMUM_MESSAGE_LENGTH: usize = 512;

extern "system" {
    fn DbgkpSuspendProcess() -> BOOLEAN;
    fn DbgkpResumeProcess();
    fn DbgkpSectionToFileHandle(section: PVOID) -> HANDLE;
    static mut PsThreadType: POBJECT_TYPE;
}
extern "C" {
    fn CONTAINING_RECORD(address: PVOID, type_size: usize, field_offset: usize) -> PVOID; // ntoskrnl shim_exports
    fn DefaultSetInfoBufferCheck(
        OperationClass: ULONG,
        ClassList: *const _INFORMATION_CLASS_INFO,
        ClassListEntries: ULONG,
        Buffer: PVOID,
        BufferLength: ULONG,
        PreviousMode: KPROCESSOR_MODE,
    ) -> NTSTATUS;
    fn PspSetProcessFlag(Process: PEPROCESS, Flags: ULONG) -> ULONG;
    fn PspSetCrossThreadFlag(Thread: PETHREAD, Flags: ULONG) -> ULONG;
    fn NtCurrentProcess() -> HANDLE;
}
extern "system" {
    #[link_name = "IoGetCurrentProcess"]
    fn PsGetCurrentProcess() -> PEPROCESS;
    fn PsGetCurrentThread() -> PETHREAD;
}
extern "system" {
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
}

#[no_mangle]
pub static mut DbgkDebugObjectType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut DbgkpProcessDebugPortMutex: _FAST_MUTEX = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut DbgkpTraceLevel: ULONG = 0;

#[no_mangle]
pub static mut DbgkDebugObjectMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | 0x0001,
    GenericWrite: STANDARD_RIGHTS_WRITE | 0x0002,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: 0x001F0003,
};

static DbgkpDebugObjectInfoClass: [_INFORMATION_CLASS_INFO; 2] = [
    _INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentQUERY: 0,
        RequiredSizeSET: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentSET: 0,
        Flags: 0,
    },
    _INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<DEBUG_OBJECT_KILL_PROCESS_ON_EXIT_INFORMATION>() as USHORT,
        AlignmentQUERY: 0,
        RequiredSizeSET: core::mem::size_of::<ULONG>() as USHORT,
        AlignmentSET: 0,
        Flags: 2,
    },
];

#[no_mangle]
pub unsafe extern "system" fn DbgkpQueueMessage(
    process: PEPROCESS,
    thread: PETHREAD,
    message: PDBGKM_MSG,
    flags: ULONG,
    target_object: PDEBUG_OBJECT,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    let mut debug_event: PDEBUG_EVENT;
    let mut local_debug_event: _DEBUG_EVENT = core::mem::zeroed();
    let mut debug_object: PDEBUG_OBJECT;
    let mut status: NTSTATUS;
    let new_event: BOOLEAN;

    new_event = if flags & DEBUG_EVENT_NOWAIT != 0 { 1u8 } else { 0u8 };

    if new_event != 0 {
        debug_event = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_DEBUG_EVENT>(),
            TAG_DEBUG_EVENT,
        ) as PDEBUG_EVENT;
        if debug_event.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        (*debug_event).Flags = flags | DEBUG_EVENT_INACTIVE;

        ObReferenceObject(thread as PVOID);
        ObReferenceObject(process as PVOID);

        (*debug_event).BackoutThread = PsGetCurrentThread();

        debug_object = target_object;
    } else {
        debug_event = &mut local_debug_event;
        (*debug_event).Flags = flags;

        ExAcquireFastMutex(&raw mut DbgkpProcessDebugPortMutex);

        debug_object = (*(process as *mut _EPROCESS)).DebugPort as PDEBUG_OBJECT;

        match (*message).ApiNumber {
            DbgKmCreateThreadApi | DbgKmCreateProcessApi => {
                if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SkipCreationMsg() != 0 {
                    debug_object = core::ptr::null_mut();
                }
            }
            DbgKmExitThreadApi | DbgKmExitProcessApi => {
                if (*(thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SkipTerminationMsg() != 0 {
                    debug_object = core::ptr::null_mut();
                }
            }
            _ => {}
        }
    }

    KeInitializeEvent(
        core::ptr::addr_of_mut!((*debug_event).ContinueEvent),
        1u32,
        0u8,
    );
    (*debug_event).Process = process;
    (*debug_event).Thread = thread;
    (*debug_event).ApiMsg = *message;
    (*debug_event).ClientId = (*(thread as *mut _ETHREAD)).Cid;

    if debug_object.is_null() {
        status = STATUS_PORT_NOT_SET;
    } else {
        ExAcquireFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

        if (*debug_object).__bindgen_anon_1.__bindgen_anon_1.DebuggerInactive() == 0 {
            InsertTailList(
                core::ptr::addr_of_mut!((*debug_object).EventList) as PLIST_ENTRY,
                core::ptr::addr_of_mut!((*debug_event).EventList) as PLIST_ENTRY,
            );

            if new_event == 0 {
                KeSetEvent(
                    core::ptr::addr_of_mut!((*debug_object).EventsPresent) as PVOID,
                    0,
                    0u8,
                );
            }

            status = STATUS_SUCCESS;
        } else {
            status = STATUS_DEBUGGER_INACTIVE;
        }

        ExReleaseFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));
    }

    if new_event == 0 {
        ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);

        if status >= 0 {
            KeWaitForSingleObject(
                core::ptr::addr_of_mut!((*debug_event).ContinueEvent) as PVOID,
                0,
                0i8,
                0u8,
                core::ptr::null_mut(),
            );

            *message = (*debug_event).ApiMsg;
            status = (*debug_event).Status;
        }
    } else {
        if status < 0 {
            ObDereferenceObject(thread as PVOID);
            ObDereferenceObject(process as PVOID);
            ExFreePoolWithTag(debug_event as PVOID, TAG_DEBUG_EVENT);
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpSendApiMessageLpc(
    message: PDBGKM_MSG,
    port: PVOID,
    suspend_process: BOOLEAN,
) -> NTSTATUS {
    let port = opaque_ptr(port);
    let mut status: NTSTATUS;
    let mut buffer: [u8; PORT_MAXIMUM_MESSAGE_LENGTH] = [0; PORT_MAXIMUM_MESSAGE_LENGTH];
    let mut suspended = 0u8;

    if suspend_process != 0 {
        suspended = DbgkpSuspendProcess();
    }

    (*message).ReturnedStatus = STATUS_PENDING;

    PspSetProcessFlag(PsGetCurrentProcess(), PSF_CREATE_REPORTED_BIT);

    status = LpcRequestWaitReplyPort(
        port,
        message as PPORT_MESSAGE,
        buffer.as_mut_ptr() as PPORT_MESSAGE,
    );

    ZwFlushInstructionCache(NtCurrentProcess(), core::ptr::null_mut(), 0);

    if status >= 0 {
        RtlCopyMemory(
            message as PVOID,
            buffer.as_ptr() as PVOID,
            core::mem::size_of::<_DBGKM_MSG>(),
        );
    }

    if suspended != 0 {
        DbgkpResumeProcess();
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpSendApiMessage(
    api_msg: PDBGKM_MSG,
    suspend_process: BOOLEAN,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut suspended = 0u8;

    if suspend_process != 0 {
        suspended = DbgkpSuspendProcess();
    }

    (*api_msg).ReturnedStatus = STATUS_PENDING;

    PspSetProcessFlag(PsGetCurrentProcess(), PSF_CREATE_REPORTED_BIT);

    status = DbgkpQueueMessage(
        PsGetCurrentProcess(),
        PsGetCurrentThread(),
        api_msg,
        0,
        core::ptr::null_mut(),
    );

    ZwFlushInstructionCache(NtCurrentProcess(), core::ptr::null_mut(), 0);

    if suspended != 0 {
        DbgkpResumeProcess();
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn DbgkCopyProcessDebugPort(
    process: PEPROCESS,
    parent: PEPROCESS,
) {
    let process = opaque_ptr(process);
    let parent = opaque_ptr(parent);
    let mut debug_object: PDEBUG_OBJECT;

    (*(process as *mut _EPROCESS)).DebugPort = core::ptr::null_mut();

    if (*(parent as *mut _EPROCESS)).DebugPort.is_null() {
        return;
    }

    ExAcquireFastMutex(&raw mut DbgkpProcessDebugPortMutex);

    debug_object = (*(parent as *mut _EPROCESS)).DebugPort as PDEBUG_OBJECT;
    if !debug_object.is_null() && (*(process as *mut _EPROCESS)).__bindgen_anon_2.__bindgen_anon_1.NoDebugInherit() == 0 {
        ExAcquireFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

        if (*debug_object).__bindgen_anon_1.__bindgen_anon_1.DebuggerInactive() == 0 {
            ObReferenceObject(debug_object as PVOID);
            (*(process as *mut _EPROCESS)).DebugPort = debug_object as PVOID;
        }

        ExReleaseFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));
    }

    ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);
}

#[no_mangle]
pub unsafe extern "system" fn DbgkForwardException(
    exception_record: PEXCEPTION_RECORD,
    debug_port: BOOLEAN,
    second_chance: BOOLEAN,
) -> BOOLEAN {
    let mut api_message: _DBGKM_MSG = core::mem::zeroed();
    let mut status: NTSTATUS;
    let process = PsGetCurrentProcess();
    let mut port: PVOID;
    let mut use_lpc = 0u8;

    api_message.h.u1.s1.DataLength = (8 + core::mem::size_of::<_DBGKM_EXCEPTION>()) as i16;
    api_message.h.u1.s1.TotalLength = (core::mem::size_of::<_DBGKM_MSG>()) as i16;
    api_message.h.u2.s2.Type = LPC_DEBUG_EVENT;
    api_message.ApiNumber = DbgKmExceptionApi;

    if debug_port != 0 {
        port = if (*(PsGetCurrentThread() as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.HideFromDebugger() != 0 {
            core::ptr::null_mut()
        } else {
            (*(process as *mut _EPROCESS)).DebugPort
        };
    } else {
        port = (*(process as *mut _EPROCESS)).ExceptionPort;
        api_message.h.u2.s2.Type = LPC_EXCEPTION;
        use_lpc = 1u8;
    }

    if port.is_null() {
        return 0u8;
    }

    api_message.__bindgen_anon_1.Exception.ExceptionRecord = *exception_record;
    api_message.__bindgen_anon_1.Exception.FirstChance = if second_chance != 0 { 0 } else { 1 };

    if use_lpc != 0 {
        status = DbgkpSendApiMessageLpc(&mut api_message, port, debug_port);
    } else {
        status = DbgkpSendApiMessage(&mut api_message, debug_port);
    }

    if status < 0
        || (debug_port != 0
            && (api_message.ReturnedStatus < 0
                || api_message.ReturnedStatus == DBG_EXCEPTION_NOT_HANDLED))
    {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpFreeDebugEvent(debug_event: PDEBUG_EVENT) {
    let mut handle: PHANDLE = core::ptr::null_mut();

    match (*debug_event).ApiMsg.ApiNumber {
        DbgKmCreateProcessApi => {
            handle = core::ptr::addr_of_mut!((*debug_event).ApiMsg.__bindgen_anon_1.CreateProcess.FileHandle);
        }
        DbgKmLoadDllApi => {
            handle = core::ptr::addr_of_mut!((*debug_event).ApiMsg.__bindgen_anon_1.LoadDll.FileHandle);
        }
        _ => {}
    }

    if !handle.is_null() && !(*handle).is_null() {
        ObCloseHandle(*handle, 0i8);
    }

    ObDereferenceObject((*debug_event).Process as PVOID);
    ObDereferenceObject((*debug_event).Thread as PVOID);
    ExFreePoolWithTag(debug_event as PVOID, TAG_DEBUG_EVENT);
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpWakeTarget(debug_event: PDEBUG_EVENT) {
    let thread = (*debug_event).Thread;

    if (*debug_event).Flags & DEBUG_EVENT_SUSPEND != 0 {
        PsResumeThread(thread, core::ptr::null_mut());
    }

    if (*debug_event).Flags & DEBUG_EVENT_RELEASE != 0 {
        ExReleaseRundownProtection(core::ptr::addr_of_mut!(
            (*(thread as *mut _ETHREAD)).RundownProtect
        ));
    }

    if (*debug_event).Flags & DEBUG_EVENT_NOWAIT != 0 {
        DbgkpFreeDebugEvent(debug_event);
    } else {
        KeSetEvent(
            core::ptr::addr_of_mut!((*debug_event).ContinueEvent) as PVOID,
            0,
            0u8,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpPostFakeModuleMessages(
    process: PEPROCESS,
    thread: PETHREAD,
    debug_object: PDEBUG_OBJECT,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    let peb: PPEB = (*(process as *mut _EPROCESS)).Peb as PPEB;
    let mut ldr_data: PPEB_LDR_DATA;
    let mut ldr_entry: PLDR_DATA_TABLE_ENTRY;
    let mut list_head: PLIST_ENTRY;
    let mut next_entry: PLIST_ENTRY;
    let mut api_message: _DBGKM_MSG = core::mem::zeroed();
    let mut i: ULONG;
    let mut nt_header: PIMAGE_NT_HEADERS;
    let mut module_name: _UNICODE_STRING = core::mem::zeroed();
    let mut object_attributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut io_status_block: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut full_dll_name: _UNICODE_STRING;

    if peb.is_null() {
        return STATUS_SUCCESS;
    }

    // C: the whole walk is one _SEH2_TRY whose handler is NOTHING, then
    // `return STATUS_SUCCESS` — a bad user pointer just ends the walk. With no SEH,
    // each probe and each read of user memory is a Result, and Err ends the walk.
    macro_rules! probe_or_done {
        ($ptr:expr, $len:expr) => {
            if ntkrnl_sys::probe::probe_user_buffer($ptr as *const core::ffi::c_void, $len, 1, 1).is_err() {
                return STATUS_SUCCESS;
            }
        };
    }
    macro_rules! read_or_done {
        ($ptr:expr) => {
            match ntkrnl_sys::probe::probe_read_generic($ptr, 1, 0) {
                Ok(value) => value,
                Err(_) => return STATUS_SUCCESS,
            }
        };
    }

    probe_or_done!(peb, core::mem::size_of::<_PEB>());
    ldr_data = read_or_done!(core::ptr::addr_of!((*peb).Ldr));
    probe_or_done!(ldr_data, core::mem::size_of::<_PEB_LDR_DATA>());
    list_head = core::ptr::addr_of_mut!((*ldr_data).InLoadOrderModuleList);
    probe_or_done!(list_head, core::mem::size_of::<LIST_ENTRY>());
    next_entry = read_or_done!(core::ptr::addr_of!((*list_head).Flink));

    i = 0;
    while next_entry != list_head && i < 500 {
        probe_or_done!(next_entry, core::mem::size_of::<LIST_ENTRY>());

        if i != 0 {
            ldr_entry = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_LDR_DATA_TABLE_ENTRY>(),
            core::mem::offset_of!(_LDR_DATA_TABLE_ENTRY, InLoadOrderLinks),
        ) as PLDR_DATA_TABLE_ENTRY;
            probe_or_done!(ldr_entry, core::mem::size_of::<_LDR_DATA_TABLE_ENTRY>());

            RtlZeroMemory(
                &mut api_message as *mut _ as PVOID,
                core::mem::size_of::<_DBGKM_MSG>(),
            );
            api_message.ApiNumber = DbgKmLoadDllApi;

            api_message.__bindgen_anon_1.LoadDll.BaseOfDll = read_or_done!(core::ptr::addr_of!((*ldr_entry).DllBase));
            api_message.__bindgen_anon_1.LoadDll.NamePointer = core::ptr::null_mut();

            nt_header = RtlImageNtHeader(api_message.__bindgen_anon_1.LoadDll.BaseOfDll);
            if !nt_header.is_null() {
                api_message.__bindgen_anon_1.LoadDll.DebugInfoFileOffset =
                    (*nt_header).FileHeader.PointerToSymbolTable;
                api_message.__bindgen_anon_1.LoadDll.DebugInfoSize = (*nt_header).FileHeader.NumberOfSymbols;
            }

            full_dll_name = read_or_done!(core::ptr::addr_of!((*ldr_entry).FullDllName));
            probe_or_done!(full_dll_name.Buffer, full_dll_name.MaximumLength as usize);

            status = MmGetFileNameForAddress(nt_header as PVOID, &mut module_name);
            if status >= 0 {
                object_attributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
                object_attributes.RootDirectory = core::ptr::null_mut();
                object_attributes.ObjectName = &mut module_name;
                object_attributes.Attributes = OBJ_FORCE_ACCESS_CHECK | OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE;
                object_attributes.SecurityDescriptor = core::ptr::null_mut();
                object_attributes.SecurityQualityOfService = core::ptr::null_mut();

                status = ZwOpenFile(
                    core::ptr::addr_of_mut!(api_message.__bindgen_anon_1.LoadDll.FileHandle),
                    GENERIC_READ | SYNCHRONIZE,
                    &mut object_attributes,
                    &mut io_status_block,
                    FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
                    FILE_SYNCHRONOUS_IO_NONALERT,
                );
                if status < 0 {
                    api_message.__bindgen_anon_1.LoadDll.FileHandle = core::ptr::null_mut();
                }

                RtlFreeUnicodeString(&mut module_name);
            }

            status = DbgkpQueueMessage(
                process,
                thread,
                &mut api_message,
                DEBUG_EVENT_NOWAIT,
                debug_object,
            );
            if status < 0 {
                if !api_message.__bindgen_anon_1.LoadDll.FileHandle.is_null() {
                    ObCloseHandle(api_message.__bindgen_anon_1.LoadDll.FileHandle, 0i8);
                }
            }
        }

        next_entry = read_or_done!(core::ptr::addr_of!((*next_entry).Flink));
        i += 1;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpPostFakeThreadMessages(
    process: PEPROCESS,
    debug_object: PDEBUG_OBJECT,
    start_thread: PETHREAD,
    first_thread: *mut PETHREAD,
    last_thread: *mut PETHREAD,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let start_thread = opaque_ptr(start_thread);
    let first_thread = opaque_ptr(first_thread);
    let last_thread = opaque_ptr(last_thread);
    let mut p_first_thread: PETHREAD = core::ptr::null_mut();
    let mut this_thread: PETHREAD;
    let mut old_thread: PETHREAD = core::ptr::null_mut();
    let mut p_last_thread: PETHREAD;
    let mut status = STATUS_UNSUCCESSFUL;
    let mut is_first_thread: BOOLEAN;
    let mut flags: ULONG;
    let mut api_message: _DBGKM_MSG = core::mem::zeroed();
    let mut first: BOOLEAN;
    let mut nt_header: PIMAGE_NT_HEADERS;

    if !start_thread.is_null() {
        is_first_thread = 0u8;
        p_first_thread = start_thread;
        this_thread = start_thread;
        ObReferenceObject(start_thread as PVOID);
    } else {
        this_thread = PsGetNextProcessThread(process, core::ptr::null_mut());
        is_first_thread = 1u8;
    }

    loop {
        if !old_thread.is_null() {
            ObDereferenceObject(old_thread as PVOID);
        }

        p_last_thread = this_thread;
        ObReferenceObject(this_thread as PVOID);

        if ExAcquireRundownProtection(core::ptr::addr_of_mut!(
            (*(this_thread as *mut _ETHREAD)).RundownProtect
        )) != 0
        {
            flags = DEBUG_EVENT_RELEASE | DEBUG_EVENT_NOWAIT;

            if (*(this_thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() == 0 {
                if PsSuspendThread(this_thread, core::ptr::null_mut()) >= 0 {
                    flags |= DEBUG_EVENT_SUSPEND;
                }
            }
        } else {
            flags = DEBUG_EVENT_PROTECT_FAILED | DEBUG_EVENT_NOWAIT;
        }

        RtlZeroMemory(
            &mut api_message as *mut _ as PVOID,
            core::mem::size_of::<_DBGKM_MSG>(),
        );

        if is_first_thread != 0
            && flags & DEBUG_EVENT_PROTECT_FAILED == 0
            && (*(this_thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() == 0
            && (*(this_thread as *mut _ETHREAD)).GrantedAccess != 0
        {
            first = 1u8;
        } else {
            first = 0u8;
        }

        if first != 0 {
            api_message.ApiNumber = DbgKmCreateProcessApi;

            if !(*(process as *mut _EPROCESS)).SectionObject.is_null() {
                api_message.__bindgen_anon_1.CreateProcess.FileHandle =
                    DbgkpSectionToFileHandle((*(process as *mut _EPROCESS)).SectionObject);
            } else {
                api_message.__bindgen_anon_1.CreateProcess.FileHandle = core::ptr::null_mut();
            }

            api_message.__bindgen_anon_1.CreateProcess.BaseOfImage =
                (*(process as *mut _EPROCESS)).SectionBaseAddress;

            nt_header = RtlImageNtHeader((*(process as *mut _EPROCESS)).SectionBaseAddress);
            if !nt_header.is_null() {
                api_message.__bindgen_anon_1.CreateProcess.DebugInfoFileOffset =
                    (*nt_header).FileHeader.PointerToSymbolTable;
                api_message.__bindgen_anon_1.CreateProcess.DebugInfoSize =
                    (*nt_header).FileHeader.NumberOfSymbols;
            }
        } else {
            api_message.ApiNumber = DbgKmCreateThreadApi;
            api_message.__bindgen_anon_1.CreateThread.StartAddress =
                core::mem::transmute((*(this_thread as *mut _ETHREAD)).__bindgen_anon_6.StartAddress);
        }

        status = DbgkpQueueMessage(
            process,
            this_thread,
            &mut api_message,
            flags,
            debug_object,
        );
        if status < 0 {
            if flags & DEBUG_EVENT_SUSPEND != 0 {
                PsResumeThread(this_thread, core::ptr::null_mut());
            }

            if flags & DEBUG_EVENT_RELEASE != 0 {
                ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                    (*(this_thread as *mut _ETHREAD)).RundownProtect
                ));
            }

            if api_message.ApiNumber == DbgKmCreateProcessApi
                && !api_message.__bindgen_anon_1.CreateProcess.FileHandle.is_null()
            {
                ObCloseHandle(api_message.__bindgen_anon_1.CreateProcess.FileHandle, 0i8);
            }

            ObDereferenceObject(this_thread as PVOID);
            break;
        }

        if first != 0 {
            is_first_thread = 0u8;
            ObReferenceObject(this_thread as PVOID);
            p_first_thread = this_thread;
        }

        this_thread = PsGetNextProcessThread(process, this_thread);
        old_thread = p_last_thread;

        if this_thread.is_null() {
            break;
        }
    }

    if status < 0 {
        if !p_first_thread.is_null() {
            ObDereferenceObject(p_first_thread as PVOID);
        }
        ObDereferenceObject(p_last_thread as PVOID);
        return status;
    }

    if p_first_thread.is_null() {
        // Upstream ReactOS leaks the last-thread reference here (dbgkobj.c:
        // `if (!pFirstThread) return STATUS_UNSUCCESSFUL;` without releasing
        // the loop's pLastThread ref, which is only handed to the caller on
        // the success path below) — found by scan_ob_refcount.py.
        if !p_last_thread.is_null() {
            ObDereferenceObject(p_last_thread as PVOID);
        }
        return STATUS_UNSUCCESSFUL;
    }

    *first_thread = p_first_thread;
    *last_thread = p_last_thread;
    status
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpPostFakeProcessCreateMessages(
    process: PEPROCESS,
    debug_object: PDEBUG_OBJECT,
    last_thread: *mut PETHREAD,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let last_thread = opaque_ptr(last_thread);
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();
    let mut first_thread: PETHREAD = core::ptr::null_mut();
    let mut final_thread: PETHREAD = core::ptr::null_mut();
    let mut return_thread: PETHREAD = core::ptr::null_mut();
    let mut status: NTSTATUS;

    KeStackAttachProcess(
        core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Pcb),
        &mut apc_state,
    );

    status = DbgkpPostFakeThreadMessages(
        process,
        debug_object,
        core::ptr::null_mut(),
        &mut first_thread,
        &mut final_thread,
    );
    if status >= 0 {
        status = DbgkpPostFakeModuleMessages(process, first_thread, debug_object);
        if status < 0 {
            ObDereferenceObject(final_thread as PVOID);
        } else {
            return_thread = final_thread;
        }

        ObDereferenceObject(first_thread as PVOID);
    }

    KeUnstackDetachProcess(&mut apc_state);

    *last_thread = return_thread;
    status
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpConvertKernelToUserStateChange(
    wait_state_change: PDBGUI_WAIT_STATE_CHANGE,
    debug_event: PDEBUG_EVENT,
) {
    (*wait_state_change).AppClientId = (*debug_event).ClientId;

    match (*debug_event).ApiMsg.ApiNumber {
        DbgKmCreateProcessApi => {
            (*wait_state_change).NewState = DbgCreateProcessStateChange;
            (*wait_state_change).StateInfo.CreateProcessInfo.NewProcess =
                (*debug_event).ApiMsg.__bindgen_anon_1.CreateProcess;
            (*debug_event).ApiMsg.__bindgen_anon_1.CreateProcess.FileHandle = core::ptr::null_mut();
        }
        DbgKmCreateThreadApi => {
            (*wait_state_change).NewState = DbgCreateThreadStateChange;
            (*wait_state_change)
                .StateInfo
                .CreateThread
                .NewThread
                .StartAddress = (*debug_event).ApiMsg.__bindgen_anon_1.CreateThread.StartAddress;
            (*wait_state_change)
                .StateInfo
                .CreateThread
                .NewThread
                .SubSystemKey = (*debug_event).ApiMsg.__bindgen_anon_1.CreateThread.SubSystemKey;
        }
        DbgKmExceptionApi => {
            if (*debug_event).ApiMsg.__bindgen_anon_1.Exception.ExceptionRecord.ExceptionCode as u32
                == STATUS_BREAKPOINT as u32
            {
                (*wait_state_change).NewState = DbgBreakpointStateChange;
            } else if (*debug_event).ApiMsg.__bindgen_anon_1.Exception.ExceptionRecord.ExceptionCode as u32
                == STATUS_SINGLE_STEP as u32
            {
                (*wait_state_change).NewState = DbgSingleStepStateChange;
            } else {
                (*wait_state_change).NewState = DbgExceptionStateChange;
            }

            (*wait_state_change).StateInfo.Exception.ExceptionRecord =
                (*debug_event).ApiMsg.__bindgen_anon_1.Exception.ExceptionRecord;
            (*wait_state_change).StateInfo.Exception.FirstChance =
                (*debug_event).ApiMsg.__bindgen_anon_1.Exception.FirstChance;
        }
        DbgKmExitProcessApi => {
            (*wait_state_change).NewState = DbgExitProcessStateChange;
            (*wait_state_change).StateInfo.ExitProcess.ExitStatus =
                (*debug_event).ApiMsg.__bindgen_anon_1.ExitProcess.ExitStatus;
        }
        DbgKmExitThreadApi => {
            (*wait_state_change).NewState = DbgExitThreadStateChange;
            (*wait_state_change).StateInfo.ExitThread.ExitStatus =
                (*debug_event).ApiMsg.__bindgen_anon_1.ExitThread.ExitStatus;
        }
        DbgKmLoadDllApi => {
            (*wait_state_change).NewState = DbgLoadDllStateChange;
            (*wait_state_change).StateInfo.LoadDll = (*debug_event).ApiMsg.__bindgen_anon_1.LoadDll;
            (*debug_event).ApiMsg.__bindgen_anon_1.LoadDll.FileHandle = core::ptr::null_mut();
        }
        DbgKmUnloadDllApi => {
            (*wait_state_change).NewState = DbgUnloadDllStateChange;
            (*wait_state_change).StateInfo.UnloadDll.BaseAddress =
                (*debug_event).ApiMsg.__bindgen_anon_1.UnloadDll.BaseAddress;
        }
        _ => {}
    }
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpMarkProcessPeb(process: PEPROCESS) {
    let process = opaque_ptr(process);
    let mut apc_state: _KAPC_STATE = core::mem::zeroed();

    if ExAcquireRundownProtection(core::ptr::addr_of_mut!(
        (*(process as *mut _EPROCESS)).RundownProtect
    )) == 0
    {
        return;
    }

    if !(*(process as *mut _EPROCESS)).Peb.is_null() {
        KeStackAttachProcess(
            core::ptr::addr_of_mut!((*(process as *mut _EPROCESS)).Pcb),
            &mut apc_state,
        );

        ExAcquireFastMutex(&raw mut DbgkpProcessDebugPortMutex);

        (*((*(process as *mut _EPROCESS)).Peb as *mut _PEB)).BeingDebugged =
            if (*(process as *mut _EPROCESS)).DebugPort.is_null() {
                0u8
            } else {
                1u8
            };

        ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);

        KeUnstackDetachProcess(&mut apc_state);
    }

    ExReleaseRundownProtection(core::ptr::addr_of_mut!(
        (*(process as *mut _EPROCESS)).RundownProtect
    ));
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpOpenHandles(
    wait_state_change: PDBGUI_WAIT_STATE_CHANGE,
    process: PEPROCESS,
    thread: PETHREAD,
) {
    let process = opaque_ptr(process);
    let thread = opaque_ptr(thread);
    let mut status: NTSTATUS;
    let mut handle: HANDLE = core::ptr::null_mut();
    let mut dup_handle: PHANDLE;

    match (*wait_state_change).NewState {
        DbgCreateThreadStateChange => {
            status = ObOpenObjectByPointer(
                thread as PVOID,
                0,
                core::ptr::null_mut(),
                THREAD_ALL_ACCESS,
                PsThreadType,
                0i8,
                &mut handle,
            );
            if status >= 0 {
                (*wait_state_change)
                    .StateInfo
                    .CreateThread
                    .HandleToThread = handle;
            }
            return;
        }
        DbgCreateProcessStateChange => {
            status = ObOpenObjectByPointer(
                thread as PVOID,
                0,
                core::ptr::null_mut(),
                THREAD_ALL_ACCESS,
                PsThreadType,
                0i8,
                &mut handle,
            );
            if status >= 0 {
                (*wait_state_change)
                    .StateInfo
                    .CreateProcessInfo
                    .HandleToThread = handle;
            }

            status = ObOpenObjectByPointer(
                process as PVOID,
                0,
                core::ptr::null_mut(),
                PROCESS_ALL_ACCESS,
                PsProcessType,
                0i8,
                &mut handle,
            );
            if status >= 0 {
                (*wait_state_change)
                    .StateInfo
                    .CreateProcessInfo
                    .HandleToProcess = handle;
            }

            dup_handle = core::ptr::addr_of_mut!(
                (*wait_state_change)
                    .StateInfo
                    .CreateProcessInfo
                    .NewProcess
                    .FileHandle
            );
        }
        DbgLoadDllStateChange => {
            dup_handle = core::ptr::addr_of_mut!((*wait_state_change).StateInfo.LoadDll.FileHandle);
        }
        _ => return,
    }

    handle = *dup_handle;
    if !handle.is_null() {
        status = ObDuplicateObject(
            PsGetCurrentProcess(),
            handle,
            PsGetCurrentProcess(),
            dup_handle,
            0,
            0,
            DUPLICATE_SAME_ACCESS,
            0i8,
        );
        if status < 0 {
            *dup_handle = core::ptr::null_mut();
        }

        ObCloseHandle(handle, 0i8);
    }
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpDeleteObject(debug_object: PVOID) {}
#[no_mangle]
pub unsafe extern "system" fn DbgkpCloseObject(
    _owner_process: PEPROCESS,
    object_body: PVOID,
    _granted_access: ACCESS_MASK,
    _handle_count: ULONG,
    system_handle_count: ULONG,
) {
    let _owner_process = opaque_ptr(_owner_process);
    let object_body = opaque_ptr(object_body);
    let debug_object = object_body as PDEBUG_OBJECT;
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut debug_port_cleared: BOOLEAN;
    let mut debug_event_list: PLIST_ENTRY;
    let mut debug_event: PDEBUG_EVENT;

    if system_handle_count > 1 {
        return;
    }

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

    (*debug_object).__bindgen_anon_1.__bindgen_anon_1.set_DebuggerInactive(1);

    debug_event_list = (*debug_object).EventList.Flink;
    InitializeListHead(core::ptr::addr_of_mut!((*debug_object).EventList));

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

    KeSetEvent(
        core::ptr::addr_of_mut!((*debug_object).EventsPresent) as PVOID,
        0,
        0u8,
    );

    loop {
        process = PsGetNextProcess(process);
        if process.is_null() {
            break;
        }

        if (*(process as *mut _EPROCESS)).DebugPort == debug_object as PVOID {
            ExAcquireFastMutex(&raw mut DbgkpProcessDebugPortMutex);

            debug_port_cleared = 0u8;
            if (*(process as *mut _EPROCESS)).DebugPort == debug_object as PVOID {
                (*(process as *mut _EPROCESS)).DebugPort = core::ptr::null_mut();
                debug_port_cleared = 1u8;
            }

            ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);

            if debug_port_cleared != 0 {
                DbgkpMarkProcessPeb(process);

                if (*debug_object).__bindgen_anon_1.__bindgen_anon_1.KillProcessOnExit() != 0 {
                    PsTerminateProcess(process, STATUS_DEBUGGER_INACTIVE);
                }

                ObDereferenceObject(debug_object as PVOID);
            }
        }
    }

    while debug_event_list != core::ptr::addr_of_mut!((*debug_object).EventList) {
        debug_event = CONTAINING_RECORD(
            debug_event_list as PVOID,
            core::mem::size_of::<_DEBUG_EVENT>(),
            core::mem::offset_of!(_DEBUG_EVENT, EventList),
        ) as PDEBUG_EVENT;

        debug_event_list = (*debug_event_list).Flink;

        (*debug_event).Status = STATUS_DEBUGGER_INACTIVE;
        DbgkpWakeTarget(debug_event);
    }
}

#[no_mangle]
pub unsafe extern "system" fn DbgkpSetProcessDebugObject(
    process: PEPROCESS,
    debug_object: PDEBUG_OBJECT,
    msg_status: NTSTATUS,
    last_thread: PETHREAD,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let last_thread = opaque_ptr(last_thread);
    let mut status: NTSTATUS;
    let mut temp_list: LIST_ENTRY = core::mem::zeroed();
    let mut global_held = 0u8;
    let mut do_set_event: BOOLEAN;
    let mut this_thread: PETHREAD;
    let mut first_thread: PETHREAD = core::ptr::null_mut();
    let mut next_entry: PLIST_ENTRY;
    let mut debug_event: PDEBUG_EVENT;
    let mut event_thread: PETHREAD;
    let mut last_thread_local = last_thread;

    InitializeListHead(&mut temp_list);

    if msg_status >= 0 {
        status = STATUS_SUCCESS;
    } else {
        last_thread_local = core::ptr::null_mut();
        status = msg_status;
    }

    if status >= 0 {
        'thread_scan: loop {
            global_held = 1u8;
            ExAcquireFastMutex(&raw mut DbgkpProcessDebugPortMutex);

            if !(*(process as *mut _EPROCESS)).DebugPort.is_null() {
                status = STATUS_PORT_ALREADY_SET;
            } else {
                (*(process as *mut _EPROCESS)).DebugPort = debug_object as PVOID;
                ObReferenceObject(last_thread_local as PVOID);

                this_thread = PsGetNextProcessThread(process, last_thread_local);
                if !this_thread.is_null() {
                    (*(process as *mut _EPROCESS)).DebugPort = core::ptr::null_mut();
                    ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);
                    global_held = 0u8;

                    ObDereferenceObject(last_thread_local as PVOID);

                    status = DbgkpPostFakeThreadMessages(
                        process,
                        debug_object,
                        this_thread,
                        &mut first_thread,
                        &mut last_thread_local,
                    );
                    if status < 0 {
                        last_thread_local = core::ptr::null_mut();
                    } else {
                        ObDereferenceObject(first_thread as PVOID);
                        continue 'thread_scan;
                    }
                }
            }
            break;
        }
    }

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

    if status >= 0 {
        if (*debug_object).__bindgen_anon_1.__bindgen_anon_1.DebuggerInactive() != 0 {
            (*(process as *mut _EPROCESS)).DebugPort = core::ptr::null_mut();
            status = STATUS_DEBUGGER_INACTIVE;
        } else {
            PspSetProcessFlag(
                process,
                PSF_NO_DEBUG_INHERIT_BIT | PSF_CREATE_REPORTED_BIT,
            );

            ObReferenceObject(debug_object as PVOID);
        }
    }

    do_set_event = 1u8;
    next_entry = (*debug_object).EventList.Flink;
    while next_entry != core::ptr::addr_of_mut!((*debug_object).EventList) {
        debug_event = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_DEBUG_EVENT>(),
            core::mem::offset_of!(_DEBUG_EVENT, EventList),
        ) as PDEBUG_EVENT;
        next_entry = (*next_entry).Flink;

        if (*debug_event).Flags & DEBUG_EVENT_INACTIVE != 0
            && (*debug_event).BackoutThread == PsGetCurrentThread()
        {
            event_thread = (*debug_event).Thread;

            if msg_status == STATUS_SUCCESS
                && (*(event_thread as *mut _ETHREAD)).GrantedAccess != 0
                && (*(event_thread as *mut _ETHREAD)).__bindgen_anon_7.__bindgen_anon_1.SystemThread() == 0
            {
                if (*debug_event).Flags & DEBUG_EVENT_PROTECT_FAILED != 0 {
                    PspSetCrossThreadFlag(event_thread, CT_SKIP_CREATION_MSG_BIT);

                    RemoveEntryList(core::ptr::addr_of_mut!((*debug_event).EventList));
                    InsertTailList(
                        &mut temp_list,
                        core::ptr::addr_of_mut!((*debug_event).EventList) as PLIST_ENTRY,
                    );
                } else {
                    if do_set_event != 0 {
                        (*debug_event).Flags &= !DEBUG_EVENT_INACTIVE;
                        KeSetEvent(
                            core::ptr::addr_of_mut!((*debug_object).EventsPresent) as PVOID,
                            0,
                            0u8,
                        );
                        do_set_event = 0u8;
                    }

                    (*debug_event).BackoutThread = core::ptr::null_mut();

                    PspSetCrossThreadFlag(event_thread, CT_SKIP_CREATION_MSG_BIT);
                }
            } else {
                RemoveEntryList(core::ptr::addr_of_mut!((*debug_event).EventList));
                InsertTailList(
                    &mut temp_list,
                    core::ptr::addr_of_mut!((*debug_event).EventList) as PLIST_ENTRY,
                );
            }

            if (*debug_event).Flags & DEBUG_EVENT_RELEASE != 0 {
                (*debug_event).Flags &= !DEBUG_EVENT_RELEASE;
                ExReleaseRundownProtection(core::ptr::addr_of_mut!(
                    (*(event_thread as *mut _ETHREAD)).RundownProtect
                ));
            }
        }
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

    if global_held != 0 {
        ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);
    }

    if !last_thread_local.is_null() {
        ObDereferenceObject(last_thread_local as PVOID);
    }

    while !IsListEmpty(&temp_list) {
        next_entry = RemoveHeadList(&mut temp_list);
        debug_event = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_DEBUG_EVENT>(),
            core::mem::offset_of!(_DEBUG_EVENT, EventList),
        ) as PDEBUG_EVENT;

        DbgkpWakeTarget(debug_event);
    }

    if status >= 0 {
        DbgkpMarkProcessPeb(process);
    }
    status
}

#[no_mangle]
pub unsafe extern "system" fn DbgkClearProcessDebugObject(
    process: PEPROCESS,
    source_debug_object: PDEBUG_OBJECT,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let mut debug_object: PDEBUG_OBJECT;
    let mut debug_event: PDEBUG_EVENT;
    let mut temp_list: LIST_ENTRY = core::mem::zeroed();
    let mut next_entry: PLIST_ENTRY;

    ExAcquireFastMutex(&raw mut DbgkpProcessDebugPortMutex);

    debug_object = (*(process as *mut _EPROCESS)).DebugPort as PDEBUG_OBJECT;

    if !debug_object.is_null()
        && (debug_object == source_debug_object || source_debug_object.is_null())
    {
        (*(process as *mut _EPROCESS)).DebugPort = core::ptr::null_mut();

        ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);
        DbgkpMarkProcessPeb(process);
    } else {
        ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);
        return STATUS_PORT_NOT_SET;
    }

    InitializeListHead(&mut temp_list);

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

    next_entry = (*debug_object).EventList.Flink;
    while next_entry != core::ptr::addr_of_mut!((*debug_object).EventList) {
        debug_event = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_DEBUG_EVENT>(),
            core::mem::offset_of!(_DEBUG_EVENT, EventList),
        ) as PDEBUG_EVENT;
        next_entry = (*next_entry).Flink;

        if (*debug_event).Process == process {
            RemoveEntryList(core::ptr::addr_of_mut!((*debug_event).EventList));
            InsertTailList(
                &mut temp_list,
                core::ptr::addr_of_mut!((*debug_event).EventList) as PLIST_ENTRY,
            );
        }
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

    ObDereferenceObject(debug_object as PVOID);

    while !IsListEmpty(&temp_list) {
        next_entry = RemoveHeadList(&mut temp_list);
        debug_event = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_DEBUG_EVENT>(),
            core::mem::offset_of!(_DEBUG_EVENT, EventList),
        ) as PDEBUG_EVENT;

        (*debug_event).Status = STATUS_DEBUGGER_INACTIVE;
        DbgkpWakeTarget(debug_event);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn DbgkInitialize() {
    let mut object_type_initializer: _OBJECT_TYPE_INITIALIZER = core::mem::zeroed();
    let mut name: _UNICODE_STRING = core::mem::zeroed();

    ExInitializeFastMutex(&raw mut DbgkpProcessDebugPortMutex);

    RtlZeroMemory(
        &mut object_type_initializer as *mut _ as PVOID,
        core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>(),
    );
    RtlInitUnicodeString(&mut name, [
        b'D' as u16, b'e' as u16, b'b' as u16, b'u' as u16, b'g' as u16,
        b'O' as u16, b'b' as u16, b'j' as u16, b'e' as u16, b'c' as u16,
        b't' as u16, 0,
    ].as_ptr());
    object_type_initializer.Length = core::mem::size_of::<_OBJECT_TYPE_INITIALIZER>() as u16;
    object_type_initializer.DefaultNonPagedPoolCharge =
        core::mem::size_of::<_DEBUG_OBJECT>() as ULONG;
    object_type_initializer.GenericMapping = DbgkDebugObjectMapping;
    object_type_initializer.PoolType = NonPagedPool;
    object_type_initializer.ValidAccessMask = 0x001F0003;
    object_type_initializer.SecurityRequired = 1;
    object_type_initializer.CloseProcedure = core::mem::transmute(DbgkpCloseObject as usize);
    object_type_initializer.DeleteProcedure = core::mem::transmute(DbgkpDeleteObject as usize);
    ObCreateObjectType(
        &mut name,
        &mut object_type_initializer,
        core::ptr::null_mut(),
        &raw mut DbgkDebugObjectType,
    );
}

#[no_mangle]
pub unsafe extern "system" fn DbgkOpenProcessDebugPort(
    process: PEPROCESS,
    previous_mode: KPROCESSOR_MODE,
    debug_handle: *mut HANDLE,
) -> NTSTATUS {
    let process = opaque_ptr(process);
    let debug_handle = opaque_ptr(debug_handle);
    let mut debug_object: PDEBUG_OBJECT;
    let mut status: NTSTATUS;

    if (*(process as *mut _EPROCESS)).DebugPort.is_null() {
        return STATUS_PORT_NOT_SET;
    }

    ExAcquireFastMutex(&raw mut DbgkpProcessDebugPortMutex);

    debug_object = (*(process as *mut _EPROCESS)).DebugPort as PDEBUG_OBJECT;
    if !debug_object.is_null() {
        ObReferenceObject(debug_object as PVOID);
    }

    ExReleaseFastMutex(&raw mut DbgkpProcessDebugPortMutex);

    if debug_object.is_null() {
        return STATUS_PORT_NOT_SET;
    }

    status = ObOpenObjectByPointer(
        debug_object as PVOID,
        0,
        core::ptr::null_mut(),
        MAXIMUM_ALLOWED,
        DbgkDebugObjectType,
        previous_mode,
        debug_handle,
    );
    if status < 0 {
        ObDereferenceObject(debug_object as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateDebugObject(
    debug_handle: *mut HANDLE,
    desired_access: ACCESS_MASK,
    object_attributes: POBJECT_ATTRIBUTES,
    flags: ULONG,
) -> NTSTATUS {
    let debug_handle = opaque_ptr(debug_handle);
    let object_attributes = opaque_ptr(object_attributes);
    let previous_mode = ExGetPreviousMode();
    let mut debug_object: PDEBUG_OBJECT = core::ptr::null_mut();
    let mut h_debug: HANDLE = core::ptr::null_mut();
    let mut status: NTSTATUS;

    // C: inside _SEH2_TRY, handler returns the exception code. No SEH here, so
    // the Result probe returns it instead of raising unhandled.
    if previous_mode != 0i8 {
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(debug_handle, 1, previous_mode) {
            return status;
        }
    }

    if flags & !DBGK_ALL_FLAGS != 0 {
        return STATUS_INVALID_PARAMETER;
    }

    status = ObCreateObject(
        previous_mode,
        DbgkDebugObjectType,
        object_attributes,
        previous_mode,
        core::ptr::null_mut(),
        core::mem::size_of::<_DEBUG_OBJECT>() as ULONG,
        0,
        0,
        &mut debug_object as *mut _ as *mut PVOID,
    );
    if status >= 0 {
        ExInitializeFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

        InitializeListHead(core::ptr::addr_of_mut!((*debug_object).EventList));

        KeInitializeEvent(
            core::ptr::addr_of_mut!((*debug_object).EventsPresent),
            0u32,
            0u8,
        );

        (*debug_object).__bindgen_anon_1.Flags = 0;
        if flags & DBGK_KILL_PROCESS_ON_EXIT != 0 {
            (*debug_object).__bindgen_anon_1.__bindgen_anon_1.set_KillProcessOnExit(1);
        }

        status = ObInsertObject(
            debug_object as PVOID,
            core::ptr::null_mut(),
            desired_access,
            0,
            core::ptr::null_mut(),
            &mut h_debug,
        );
        if status >= 0 {
            // C: `*DebugHandle = hDebug` under _SEH2_TRY; the handler takes the code.
            if let Err(code) = ntkrnl_sys::probe::write_back(debug_handle, h_debug) {
                status = code;
            }
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtDebugContinue(
    debug_handle: HANDLE,
    app_client_id: PCLIENT_ID,
    continue_status: NTSTATUS,
) -> NTSTATUS {
    let debug_handle = opaque_ptr(debug_handle);
    let app_client_id = opaque_ptr(app_client_id);
    let previous_mode = ExGetPreviousMode();
    let mut debug_object: PDEBUG_OBJECT = core::ptr::null_mut();
    let mut status: NTSTATUS;
    let mut debug_event: PDEBUG_EVENT = core::ptr::null_mut();
    let mut debug_event_to_wake: PDEBUG_EVENT = core::ptr::null_mut();
    let mut list_head: PLIST_ENTRY;
    let mut next_entry: PLIST_ENTRY;
    let mut needs_wake = 0u8;
    let mut client_id: _CLIENT_ID = core::mem::zeroed();

    if previous_mode != 0i8 {
        // C: probe + capture under _SEH2_TRY, whose handler returns the code.
        client_id = match ntkrnl_sys::probe::probe_read_generic(
            app_client_id as *const _CLIENT_ID,
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            Ok(id) => id,
            Err(status) => return status,
        };
    } else {
        // Kernel-mode caller: the pointer is trusted, but the load is still made
        // fault-tolerant rather than a bugcheck.
        client_id = match ntkrnl_sys::probe::read_back(app_client_id as *const _CLIENT_ID) {
            Ok(id) => id,
            Err(status) => return status,
        };
    }

    if continue_status != DBG_CONTINUE
        && continue_status != DBG_EXCEPTION_HANDLED
        && continue_status != DBG_EXCEPTION_NOT_HANDLED
        && continue_status != DBG_TERMINATE_THREAD
        && continue_status != DBG_TERMINATE_PROCESS
    {
        status = STATUS_INVALID_PARAMETER;
    } else {
        status = ObReferenceObjectByHandle(
            debug_handle,
            0x0001,
            DbgkDebugObjectType,
            previous_mode,
            &mut debug_object as *mut _ as *mut PVOID,
            core::ptr::null_mut(),
        );
        if status >= 0 {
            ExAcquireFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

            list_head = core::ptr::addr_of_mut!((*debug_object).EventList);
            next_entry = (*list_head).Flink;
            while list_head != next_entry {
                debug_event = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_DEBUG_EVENT>(),
            core::mem::offset_of!(_DEBUG_EVENT, EventList),
        ) as PDEBUG_EVENT;

                if (*debug_event).ClientId.UniqueProcess == client_id.UniqueProcess {
                    if needs_wake != 0 {
                        (*debug_event).Flags &= !DEBUG_EVENT_INACTIVE;
                        KeSetEvent(
                            core::ptr::addr_of_mut!((*debug_object).EventsPresent) as PVOID,
                            0,
                            0u8,
                        );
                        break;
                    }

                    if (*debug_event).ClientId.UniqueThread == client_id.UniqueThread
                        && (*debug_event).Flags & DEBUG_EVENT_READ != 0
                    {
                        RemoveEntryList(next_entry);

                        needs_wake = 1u8;
                        debug_event_to_wake = debug_event;
                    }
                }

                next_entry = (*next_entry).Flink;
            }

            ExReleaseFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

            ObDereferenceObject(debug_object as PVOID);

            if needs_wake != 0 {
                (*debug_event_to_wake).ApiMsg.ReturnedStatus = continue_status;
                (*debug_event_to_wake).Status = STATUS_SUCCESS;

                DbgkpWakeTarget(debug_event_to_wake);
            } else {
                status = STATUS_INVALID_PARAMETER;
            }
        }
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtDebugActiveProcess(
    process_handle: HANDLE,
    debug_handle: HANDLE,
) -> NTSTATUS {
    let process_handle = opaque_ptr(process_handle);
    let debug_handle = opaque_ptr(debug_handle);
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut debug_object: PDEBUG_OBJECT = core::ptr::null_mut();
    let previous_mode = ExGetPreviousMode();
    let mut last_thread: PETHREAD = core::ptr::null_mut();
    let mut status: NTSTATUS;

    status = ObReferenceObjectByHandle(
        process_handle,
        PROCESS_SUSPEND_RESUME,
        PsProcessType,
        previous_mode,
        &mut process as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    if process == PsGetCurrentProcess() || process == PsInitialSystemProcess as PEPROCESS {
        ObDereferenceObject(process as PVOID);
        return STATUS_ACCESS_DENIED;
    }

    status = ObReferenceObjectByHandle(
        debug_handle,
        0x0002,
        DbgkDebugObjectType,
        previous_mode,
        &mut debug_object as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        ObDereferenceObject(process as PVOID);
        return status;
    }

    if ExAcquireRundownProtection(core::ptr::addr_of_mut!(
        (*(process as *mut _EPROCESS)).RundownProtect
    )) == 0
    {
        ObDereferenceObject(process as PVOID);
        ObDereferenceObject(debug_object as PVOID);
        return STATUS_PROCESS_IS_TERMINATING;
    }

    status = DbgkpPostFakeProcessCreateMessages(process, debug_object, &mut last_thread);
    status = DbgkpSetProcessDebugObject(process, debug_object, status, last_thread);

    ExReleaseRundownProtection(core::ptr::addr_of_mut!(
        (*(process as *mut _EPROCESS)).RundownProtect
    ));

    ObDereferenceObject(process as PVOID);
    ObDereferenceObject(debug_object as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtRemoveProcessDebug(
    process_handle: HANDLE,
    debug_handle: HANDLE,
) -> NTSTATUS {
    let process_handle = opaque_ptr(process_handle);
    let debug_handle = opaque_ptr(debug_handle);
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut debug_object: PDEBUG_OBJECT = core::ptr::null_mut();
    let previous_mode = ExGetPreviousMode();
    let mut status: NTSTATUS;

    status = ObReferenceObjectByHandle(
        process_handle,
        PROCESS_SUSPEND_RESUME,
        PsProcessType,
        previous_mode,
        &mut process as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    status = ObReferenceObjectByHandle(
        debug_handle,
        0x0002,
        DbgkDebugObjectType,
        previous_mode,
        &mut debug_object as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        ObDereferenceObject(process as PVOID);
        return status;
    }

    status = DbgkClearProcessDebugObject(process, debug_object);

    ObDereferenceObject(process as PVOID);
    ObDereferenceObject(debug_object as PVOID);
    status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetInformationDebugObject(
    debug_handle: HANDLE,
    debug_object_information_class: DEBUGOBJECTINFOCLASS,
    debug_information: PVOID,
    debug_information_length: ULONG,
    return_length: *mut ULONG,
) -> NTSTATUS {
    let debug_handle = opaque_ptr(debug_handle);
    let debug_information = opaque_ptr(debug_information);
    let return_length = opaque_ptr(return_length);
    let mut debug_object: PDEBUG_OBJECT = core::ptr::null_mut();
    let previous_mode = ExGetPreviousMode();
    let mut status: NTSTATUS;
    let debug_info = debug_information as PDEBUG_OBJECT_KILL_PROCESS_ON_EXIT_INFORMATION;

    status = DefaultSetInfoBufferCheck(
        debug_object_information_class,
        DbgkpDebugObjectInfoClass.as_ptr(),
        DbgkpDebugObjectInfoClass.len() as ULONG,
        debug_information,
        debug_information_length,
        previous_mode,
    );
    if status < 0 {
        return status;
    }

    if !return_length.is_null() {
        // C probes this whatever the previous mode (the typed probe.h macros take
        // none), inside _SEH2_TRY whose handler returns the exception code.
        if let Err(status) = ntkrnl_sys::probe::probe_write_generic(return_length, 1, 1i8) {
            return status;
        }
        if let Err(status) = ntkrnl_sys::probe::write_back(
            return_length,
            core::mem::size_of::<DEBUG_OBJECT_KILL_PROCESS_ON_EXIT_INFORMATION>() as ULONG,
        ) {
            return status;
        }
    }

    status = ObReferenceObjectByHandle(
        debug_handle,
        0x0001,
        DbgkDebugObjectType,
        previous_mode,
        &mut debug_object as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status >= 0 {
        ExAcquireFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

        if (*debug_info).KillProcessOnExit != 0 {
            (*debug_object).__bindgen_anon_1.__bindgen_anon_1.set_KillProcessOnExit(1);
        } else {
            (*debug_object).__bindgen_anon_1.__bindgen_anon_1.set_KillProcessOnExit(0);
        }

        ExReleaseFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

        ObDereferenceObject(debug_object as PVOID);
    }

    status
}

#[no_mangle]
pub unsafe extern "system" fn NtWaitForDebugEvent(
    debug_handle: HANDLE,
    alertable: BOOLEAN,
    timeout: *const LARGE_INTEGER,
    state_change: PDBGUI_WAIT_STATE_CHANGE,
) -> NTSTATUS {
    let debug_handle = opaque_ptr(debug_handle);
    let previous_mode = ExGetPreviousMode();
    let mut local_time_out: LARGE_INTEGER = core::mem::zeroed();
    let mut process: PEPROCESS = core::ptr::null_mut();
    let mut start_time: LARGE_INTEGER = core::mem::zeroed();
    let mut thread: PETHREAD = core::ptr::null_mut();
    let mut got_event: BOOLEAN;
    let mut new_time: LARGE_INTEGER = core::mem::zeroed();
    let mut debug_object: PDEBUG_OBJECT = core::ptr::null_mut();
    let mut wait_state_change: _DBGUI_WAIT_STATE_CHANGE = core::mem::zeroed();
    let mut status: NTSTATUS;
    let mut debug_event: PDEBUG_EVENT = core::ptr::null_mut();
    let mut debug_event2: PDEBUG_EVENT;
    let mut list_head: PLIST_ENTRY;
    let mut next_entry: PLIST_ENTRY;
    let mut next_entry2: PLIST_ENTRY;

    RtlZeroMemory(
        &mut wait_state_change as *mut _ as PVOID,
        core::mem::size_of::<_DBGUI_WAIT_STATE_CHANGE>(),
    );
    local_time_out.QuadPart = 0;

    if previous_mode != 0i8 {
        if !timeout.is_null() {
            // C: probe then copy, inside _SEH2_TRY whose handler returns the
            // exception code. The Result probe captures the value in one read.
            match ntkrnl_sys::probe::probe_read_generic(timeout as *const LARGE_INTEGER, 1, previous_mode) {
                Ok(value) => local_time_out = value,
                Err(status) => return status,
            }
        }

        if let Err(status) = ntkrnl_sys::probe::probe_user_buffer(
            state_change as *const core::ffi::c_void,
            core::mem::size_of::<_DBGUI_WAIT_STATE_CHANGE>(),
            core::mem::size_of::<ULONG>(),
            previous_mode,
        ) {
            return status;
        }
    } else {
        if !timeout.is_null() {
            local_time_out = match ntkrnl_sys::probe::read_back(timeout as *const LARGE_INTEGER) {
                Ok(value) => value,
                Err(status) => return status,
            };
        }
    }

    if !timeout.is_null() {
        KeQuerySystemTime(&mut start_time);
    }

    status = ObReferenceObjectByHandle(
        debug_handle,
        0x0001,
        DbgkDebugObjectType,
        previous_mode,
        &mut debug_object as *mut _ as *mut PVOID,
        core::ptr::null_mut(),
    );
    if status < 0 {
        return status;
    }

    process = core::ptr::null_mut();
    thread = core::ptr::null_mut();

    loop {
        status = KeWaitForSingleObject(
            core::ptr::addr_of_mut!((*debug_object).EventsPresent) as PVOID,
            0,
            previous_mode,
            alertable,
            if timeout.is_null() {
                core::ptr::null_mut()
            } else {
                &mut local_time_out as *mut _ as *mut _
            },
        );
        if status < 0
            || status == STATUS_TIMEOUT
            || status == STATUS_ALERTED
            || status == STATUS_USER_APC
        {
            break;
        }

        got_event = 0u8;
        ExAcquireFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));

        if (*debug_object).__bindgen_anon_1.__bindgen_anon_1.DebuggerInactive() != 0 {
            status = STATUS_DEBUGGER_INACTIVE;
        } else {
            list_head = core::ptr::addr_of_mut!((*debug_object).EventList);
            next_entry = (*list_head).Flink;
            while list_head != next_entry {
                debug_event = CONTAINING_RECORD(
            next_entry as PVOID,
            core::mem::size_of::<_DEBUG_EVENT>(),
            core::mem::offset_of!(_DEBUG_EVENT, EventList),
        ) as PDEBUG_EVENT;

                if (*debug_event).Flags & (DEBUG_EVENT_INACTIVE | DEBUG_EVENT_READ) == 0 {
                    got_event = 1u8;

                    next_entry2 = (*debug_object).EventList.Flink;
                    while next_entry2 != next_entry {
                        debug_event2 = CONTAINING_RECORD(
            next_entry2 as PVOID,
            core::mem::size_of::<_DEBUG_EVENT>(),
            core::mem::offset_of!(_DEBUG_EVENT, EventList),
        ) as PDEBUG_EVENT;

                        if (*debug_event2).ClientId.UniqueProcess
                            == (*debug_event).ClientId.UniqueProcess
                        {
                            (*debug_event).Flags |= DEBUG_EVENT_INACTIVE;
                            (*debug_event).BackoutThread = core::ptr::null_mut();
                            got_event = 0u8;
                            break;
                        }

                        next_entry2 = (*next_entry2).Flink;
                    }

                    if got_event != 0 {
                        break;
                    }
                }

                next_entry = (*next_entry).Flink;
            }

            if got_event != 0 {
                process = (*debug_event).Process;
                thread = (*debug_event).Thread;
                ObReferenceObject(process as PVOID);
                ObReferenceObject(thread as PVOID);

                DbgkpConvertKernelToUserStateChange(&mut wait_state_change, debug_event);

                (*debug_event).Flags |= DEBUG_EVENT_READ;
            } else {
                KeClearEvent(core::ptr::addr_of_mut!((*debug_object).EventsPresent) as PVOID);
            }

            status = STATUS_SUCCESS;
        }

        ExReleaseFastMutex(core::ptr::addr_of_mut!((*debug_object).Mutex));
        if status < 0 {
            break;
        }

        if got_event == 0 {
            if local_time_out.QuadPart < 0 {
                KeQuerySystemTime(&mut new_time);

                local_time_out.QuadPart += new_time.QuadPart - start_time.QuadPart;
                start_time = new_time;

                if local_time_out.QuadPart >= 0 {
                    status = STATUS_TIMEOUT;
                    break;
                }
            }
        } else {
            DbgkpOpenHandles(&mut wait_state_change, process, thread);
            ObDereferenceObject(process as PVOID);
            ObDereferenceObject(thread as PVOID);
            break;
        }
    }

    ObDereferenceObject(debug_object as PVOID);

    // C: `*StateChange = WaitStateChange` under _SEH2_TRY; the handler takes the code.
    if let Err(code) = ntkrnl_sys::probe::write_back(state_change, wait_state_change) {
        status = code;
    }

    status
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/dbgk/dbgkobj.c
//  * PURPOSE:         User-Mode Debugging Support, Debug Object Management.
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// POBJECT_TYPE DbgkDebugObjectType;
// FAST_MUTEX DbgkpProcessDebugPortMutex;
// ULONG DbgkpTraceLevel = 0;
//
// GENERIC_MAPPING DbgkDebugObjectMapping =
// {
//     STANDARD_RIGHTS_READ    | DEBUG_OBJECT_WAIT_STATE_CHANGE,
//     STANDARD_RIGHTS_WRITE   | DEBUG_OBJECT_ADD_REMOVE_PROCESS,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     DEBUG_OBJECT_ALL_ACCESS
// };
//
// static const INFORMATION_CLASS_INFO DbgkpDebugObjectInfoClass[] =
// {
//     /* DebugObjectUnusedInformation */
//     IQS_SAME(ULONG, ULONG, 0),
//     /* DebugObjectKillProcessOnExitInformation */
//     IQS_SAME(DEBUG_OBJECT_KILL_PROCESS_ON_EXIT_INFORMATION, ULONG, ICIF_SET),
// };
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// NTSTATUS
// NTAPI
// DbgkpQueueMessage(IN PEPROCESS Process,
//                   IN PETHREAD Thread,
//                   IN PDBGKM_MSG Message,
//                   IN ULONG Flags,
//                   IN PDEBUG_OBJECT TargetObject OPTIONAL)
// {
//     PDEBUG_EVENT DebugEvent;
//     DEBUG_EVENT LocalDebugEvent;
//     PDEBUG_OBJECT DebugObject;
//     NTSTATUS Status;
//     BOOLEAN NewEvent;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_MESSAGE_DEBUG,
//               "Process: %p Thread: %p Message: %p Flags: %lx\n",
//               Process, Thread, Message, Flags);
//
//     /* Check if we have to allocate a debug event */
//     NewEvent = (Flags & DEBUG_EVENT_NOWAIT) ? TRUE : FALSE;
//     if (NewEvent)
//     {
//         /* Allocate it */
//         DebugEvent = ExAllocatePoolWithTag(NonPagedPool,
//                                            sizeof(DEBUG_EVENT),
//                                            TAG_DEBUG_EVENT);
//         if (!DebugEvent) return STATUS_INSUFFICIENT_RESOURCES;
//
//         /* Set flags */
//         DebugEvent->Flags = Flags | DEBUG_EVENT_INACTIVE;
//
//         /* Reference the thread and process */
//         ObReferenceObject(Thread);
//         ObReferenceObject(Process);
//
//         /* Set the current thread */
//         DebugEvent->BackoutThread = PsGetCurrentThread();
//
//         /* Set the debug object */
//         DebugObject = TargetObject;
//     }
//     else
//     {
//         /* Use the debug event on the stack */
//         DebugEvent = &LocalDebugEvent;
//         DebugEvent->Flags = Flags;
//
//         /* Acquire the port lock */
//         ExAcquireFastMutex(&DbgkpProcessDebugPortMutex);
//
//         /* Get the debug object */
//         DebugObject = Process->DebugPort;
//
//         /* Check what kind of API message this is */
//         switch (Message->ApiNumber)
//         {
//             /* Process or thread creation */
//             case DbgKmCreateThreadApi:
//             case DbgKmCreateProcessApi:
//
//                 /* Make sure we're not skipping creation messages */
//                 if (Thread->SkipCreationMsg) DebugObject = NULL;
//                 break;
//
//             /* Process or thread exit */
//             case DbgKmExitThreadApi:
//             case DbgKmExitProcessApi:
//
//                 /* Make sure we're not skipping exit messages */
//                 if (Thread->SkipTerminationMsg) DebugObject = NULL;
//
//             /* No special handling for other messages */
//             default:
//                 break;
//         }
//     }
//
//     /* Setup the Debug Event */
//     KeInitializeEvent(&DebugEvent->ContinueEvent, SynchronizationEvent, FALSE);
//     DebugEvent->Process = Process;
//     DebugEvent->Thread = Thread;
//     DebugEvent->ApiMsg = *Message;
//     DebugEvent->ClientId = Thread->Cid;
//
//     /* Check if we have a port object */
//     if (!DebugObject)
//     {
//         /* Fail */
//         Status = STATUS_PORT_NOT_SET;
//     }
//     else
//     {
//         /* Acquire the debug object mutex */
//         ExAcquireFastMutex(&DebugObject->Mutex);
//
//         /* Check if a debugger is active */
//         if (!DebugObject->DebuggerInactive)
//         {
//             /* Add the event into the object's list */
//             DBGKTRACE(DBGK_MESSAGE_DEBUG, "Inserting: %p %d\n",
//                       DebugEvent, Message->ApiNumber);
//             InsertTailList(&DebugObject->EventList, &DebugEvent->EventList);
//
//             /* Check if we have to signal it */
//             if (!NewEvent)
//             {
//                 /* Signal it */
//                 KeSetEvent(&DebugObject->EventsPresent,
//                            IO_NO_INCREMENT,
//                            FALSE);
//             }
//
//             /* Set success */
//             Status = STATUS_SUCCESS;
//         }
//         else
//         {
//             /* No debugger */
//             Status = STATUS_DEBUGGER_INACTIVE;
//         }
//
//         /* Release the object lock */
//         ExReleaseFastMutex(&DebugObject->Mutex);
//     }
//
//     /* Check if we had acquired the port lock */
//     if (!NewEvent)
//     {
//         /* Release it */
//         ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
//
//         /* Check if we got here through success */
//         if (NT_SUCCESS(Status))
//         {
//             /* Wait on the continue event */
//             KeWaitForSingleObject(&DebugEvent->ContinueEvent,
//                                   Executive,
//                                   KernelMode,
//                                   FALSE,
//                                   NULL);
//
//             /* Copy API Message back */
//             *Message = DebugEvent->ApiMsg;
//
//             /* Set return status */
//             Status = DebugEvent->Status;
//         }
//     }
//     else
//     {
//         /* Check if we failed */
//         if (!NT_SUCCESS(Status))
//         {
//             /* Dereference the process and thread */
//             ObDereferenceObject(Thread);
//             ObDereferenceObject(Process);
//
//             /* Free the debug event */
//             ExFreePoolWithTag(DebugEvent, TAG_DEBUG_EVENT);
//         }
//     }
//
//     /* Return status */
//     DBGKTRACE(DBGK_MESSAGE_DEBUG, "Status: %lx\n", Status);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// DbgkpSendApiMessageLpc(IN OUT PDBGKM_MSG Message,
//                        IN PVOID Port,
//                        IN BOOLEAN SuspendProcess)
// {
//     NTSTATUS Status;
//     UCHAR Buffer[PORT_MAXIMUM_MESSAGE_LENGTH];
//     BOOLEAN Suspended = FALSE;
//     PAGED_CODE();
//
//     /* Suspend process if required */
//     if (SuspendProcess) Suspended = DbgkpSuspendProcess();
//
//     /* Set return status */
//     Message->ReturnedStatus = STATUS_PENDING;
//
//     /* Set create process reported state */
//     PspSetProcessFlag(PsGetCurrentProcess(), PSF_CREATE_REPORTED_BIT);
//
//     /* Send the LPC command */
//     Status = LpcRequestWaitReplyPort(Port,
//                                      (PPORT_MESSAGE)Message,
//                                      (PPORT_MESSAGE)&Buffer[0]);
//
//     /* Flush the instruction cache */
//     ZwFlushInstructionCache(NtCurrentProcess(), NULL, 0);
//
//     /* Copy the buffer back */
//     if (NT_SUCCESS(Status)) RtlCopyMemory(Message, Buffer, sizeof(DBGKM_MSG));
//
//     /* Resume the process if it was suspended */
//     if (Suspended) DbgkpResumeProcess();
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// DbgkpSendApiMessage(IN OUT PDBGKM_MSG ApiMsg,
//                     IN BOOLEAN SuspendProcess)
// {
//     NTSTATUS Status;
//     BOOLEAN Suspended = FALSE;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_MESSAGE_DEBUG, "ApiMsg: %p SuspendProcess: %lx\n", ApiMsg, SuspendProcess);
//
//     /* Suspend process if required */
//     if (SuspendProcess) Suspended = DbgkpSuspendProcess();
//
//     /* Set return status */
//     ApiMsg->ReturnedStatus = STATUS_PENDING;
//
//     /* Set create process reported state */
//     PspSetProcessFlag(PsGetCurrentProcess(), PSF_CREATE_REPORTED_BIT);
//
//     /* Send the LPC command */
//     Status = DbgkpQueueMessage(PsGetCurrentProcess(),
//                                PsGetCurrentThread(),
//                                ApiMsg,
//                                0,
//                                NULL);
//
//     /* Flush the instruction cache */
//     ZwFlushInstructionCache(NtCurrentProcess(), NULL, 0);
//
//     /* Resume the process if it was suspended */
//     if (Suspended) DbgkpResumeProcess();
//     return Status;
// }
//
// VOID
// NTAPI
// DbgkCopyProcessDebugPort(IN PEPROCESS Process,
//                          IN PEPROCESS Parent)
// {
//     PDEBUG_OBJECT DebugObject;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_PROCESS_DEBUG, "Process: %p Parent: %p\n", Process, Parent);
//
//     /* Clear this process's port */
//     Process->DebugPort = NULL;
//
//     /* Check if the parent has one */
//     if (!Parent->DebugPort) return;
//
//     /* It does, acquire the mutex */
//     ExAcquireFastMutex(&DbgkpProcessDebugPortMutex);
//
//     /* Make sure it still has one, and that we should inherit */
//     DebugObject = Parent->DebugPort;
//     if ((DebugObject) && !(Process->NoDebugInherit))
//     {
//         /* Acquire the debug object's lock */
//         ExAcquireFastMutex(&DebugObject->Mutex);
//
//         /* Make sure the debugger is active */
//         if (!DebugObject->DebuggerInactive)
//         {
//             /* Reference the object and set it */
//             ObReferenceObject(DebugObject);
//             Process->DebugPort = DebugObject;
//         }
//
//         /* Release the debug object */
//         ExReleaseFastMutex(&DebugObject->Mutex);
//     }
//
//     /* Release the port mutex */
//     ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
// }
//
// BOOLEAN
// NTAPI
// DbgkForwardException(IN PEXCEPTION_RECORD ExceptionRecord,
//                      IN BOOLEAN DebugPort,
//                      IN BOOLEAN SecondChance)
// {
//     DBGKM_MSG ApiMessage;
//     PDBGKM_EXCEPTION DbgKmException = &ApiMessage.Exception;
//     NTSTATUS Status;
//     PEPROCESS Process = PsGetCurrentProcess();
//     PVOID Port;
//     BOOLEAN UseLpc = FALSE;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_EXCEPTION_DEBUG,
//               "ExceptionRecord: %p Port: %u\n", ExceptionRecord, DebugPort);
//
//     /* Setup the API Message */
//     ApiMessage.h.u1.Length = sizeof(DBGKM_MSG) << 16 |
//                              (8 + sizeof(DBGKM_EXCEPTION));
//     ApiMessage.h.u2.ZeroInit = 0;
//     ApiMessage.h.u2.s2.Type = LPC_DEBUG_EVENT;
//     ApiMessage.ApiNumber = DbgKmExceptionApi;
//
//     /* Check if this is to be sent on the debug port */
//     if (DebugPort)
//     {
//         /* Use the debug port, unless the thread is being hidden */
//         Port = PsGetCurrentThread()->HideFromDebugger ?
//                NULL : Process->DebugPort;
//     }
//     else
//     {
//         /* Otherwise, use the exception port */
//         Port = Process->ExceptionPort;
//         ApiMessage.h.u2.ZeroInit = 0;
//         ApiMessage.h.u2.s2.Type = LPC_EXCEPTION;
//         UseLpc = TRUE;
//     }
//
//     /* Break out if there's no port */
//     if (!Port) return FALSE;
//
//     /* Fill out the exception information */
//     DbgKmException->ExceptionRecord = *ExceptionRecord;
//     DbgKmException->FirstChance = !SecondChance;
//
//     /* Check if we should use LPC */
//     if (UseLpc)
//     {
//         /* Send the message on the LPC Port */
//         Status = DbgkpSendApiMessageLpc(&ApiMessage, Port, DebugPort);
//     }
//     else
//     {
//         /* Use native debug object */
//         Status = DbgkpSendApiMessage(&ApiMessage, DebugPort);
//     }
//
//     /* Check if we failed, and for a debug port, also check the return status */
//     if (!(NT_SUCCESS(Status)) ||
//         ((DebugPort) &&
//          (!(NT_SUCCESS(ApiMessage.ReturnedStatus)) ||
//            (ApiMessage.ReturnedStatus == DBG_EXCEPTION_NOT_HANDLED))))
//     {
//         /* Fail */
//         return FALSE;
//     }
//
//     /* Otherwise, we're ok */
//     return TRUE;
// }
//
// VOID
// NTAPI
// DbgkpFreeDebugEvent(IN PDEBUG_EVENT DebugEvent)
// {
//     PHANDLE Handle = NULL;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "DebugEvent: %p\n", DebugEvent);
//
//     /* Check if this event had a file handle */
//     switch (DebugEvent->ApiMsg.ApiNumber)
//     {
//         /* Create process has a handle */
//         case DbgKmCreateProcessApi:
//
//             /* Get the pointer */
//             Handle = &DebugEvent->ApiMsg.CreateProcess.FileHandle;
//             break;
//
//         /* As does DLL load */
//         case DbgKmLoadDllApi:
//
//             /* Get the pointer */
//             Handle = &DebugEvent->ApiMsg.LoadDll.FileHandle;
//
//         default:
//             break;
//     }
//
//     /* Close the handle if it exsts */
//     if ((Handle) && (*Handle)) ObCloseHandle(*Handle, KernelMode);
//
//     /* Dereference process and thread and free the event */
//     ObDereferenceObject(DebugEvent->Process);
//     ObDereferenceObject(DebugEvent->Thread);
//     ExFreePoolWithTag(DebugEvent, TAG_DEBUG_EVENT);
// }
//
// VOID
// NTAPI
// DbgkpWakeTarget(IN PDEBUG_EVENT DebugEvent)
// {
//     PETHREAD Thread = DebugEvent->Thread;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "DebugEvent: %p\n", DebugEvent);
//
//     /* Check if we have to wake the thread */
//     if (DebugEvent->Flags & DEBUG_EVENT_SUSPEND) PsResumeThread(Thread, NULL);
//
//     /* Check if we had locked the thread */
//     if (DebugEvent->Flags & DEBUG_EVENT_RELEASE)
//     {
//         /* Unlock it */
//         ExReleaseRundownProtection(&Thread->RundownProtect);
//     }
//
//     /* Check if we have to wake up the event */
//     if (DebugEvent->Flags & DEBUG_EVENT_NOWAIT)
//     {
//         /* Otherwise, free the debug event */
//         DbgkpFreeDebugEvent(DebugEvent);
//     }
//     else
//     {
//         /* Signal the continue event */
//         KeSetEvent(&DebugEvent->ContinueEvent, IO_NO_INCREMENT, FALSE);
//     }
// }
//
// NTSTATUS
// NTAPI
// DbgkpPostFakeModuleMessages(IN PEPROCESS Process,
//                             IN PETHREAD Thread,
//                             IN PDEBUG_OBJECT DebugObject)
// {
//     PPEB Peb = Process->Peb;
//     PPEB_LDR_DATA LdrData;
//     PLDR_DATA_TABLE_ENTRY LdrEntry;
//     PLIST_ENTRY ListHead, NextEntry;
//     DBGKM_MSG ApiMessage;
//     PDBGKM_LOAD_DLL LoadDll = &ApiMessage.LoadDll;
//     ULONG i;
//     PIMAGE_NT_HEADERS NtHeader;
//     UNICODE_STRING ModuleName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//     UNICODE_STRING FullDllName;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_PROCESS_DEBUG, "Process: %p Thread: %p DebugObject: %p\n",
//               Process, Thread, DebugObject);
//
//     /* Quit if there's no PEB */
//     if (!Peb) return STATUS_SUCCESS;
//
//     /* Accessing user memory, need SEH */
//     _SEH2_TRY
//     {
//         /* Get the Loader Data List */
//         ProbeForRead(Peb, sizeof(*Peb), 1);
//         LdrData = Peb->Ldr;
//         ProbeForRead(LdrData, sizeof(*LdrData), 1);
//         ListHead = &LdrData->InLoadOrderModuleList;
//         ProbeForRead(ListHead, sizeof(*ListHead), 1);
//         NextEntry = ListHead->Flink;
//
//         /* Loop the modules */
//         i = 0;
//         while ((NextEntry != ListHead) && (i < 500))
//         {
//             ProbeForRead(NextEntry, sizeof(*NextEntry), 1);
//             /* Skip the first entry */
//             if (!i)
//             {
//                 /* Go to the next module */
//                 NextEntry = NextEntry->Flink;
//                 i++;
//                 continue;
//             }
//
//             /* Get the entry */
//             LdrEntry = CONTAINING_RECORD(NextEntry,
//                                          LDR_DATA_TABLE_ENTRY,
//                                          InLoadOrderLinks);
//             ProbeForRead(LdrEntry, sizeof(*LdrEntry), 1);
//
//             /* Setup the API Message */
//             RtlZeroMemory(&ApiMessage, sizeof(DBGKM_MSG));
//             ApiMessage.ApiNumber = DbgKmLoadDllApi;
//
//             /* Set base and clear the name */
//             LoadDll->BaseOfDll = LdrEntry->DllBase;
//             LoadDll->NamePointer = NULL;
//
//             /* Get the NT Headers */
//             NtHeader = RtlImageNtHeader(LoadDll->BaseOfDll);
//             if (NtHeader)
//             {
//                 /* Save debug data */
//                 LoadDll->DebugInfoFileOffset = NtHeader->FileHeader.
//                                                PointerToSymbolTable;
//                 LoadDll->DebugInfoSize = NtHeader->FileHeader.NumberOfSymbols;
//             }
//
//             /* Trace */
//             FullDllName = LdrEntry->FullDllName;
//             ProbeForRead(FullDllName.Buffer, FullDllName.MaximumLength, 1);
//             DBGKTRACE(DBGK_PROCESS_DEBUG, "Name: %wZ. Base: %p\n",
//                       &FullDllName, LdrEntry->DllBase);
//
//             /* Get the name of the DLL */
//             Status = MmGetFileNameForAddress(NtHeader, &ModuleName);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Setup the object attributes */
//                 InitializeObjectAttributes(&ObjectAttributes,
//                                            &ModuleName,
//                                            OBJ_FORCE_ACCESS_CHECK |
//                                            OBJ_KERNEL_HANDLE |
//                                            OBJ_CASE_INSENSITIVE,
//                                            NULL,
//                                            NULL);
//
//                 /* Open the file to get a handle to it */
//                 Status = ZwOpenFile(&LoadDll->FileHandle,
//                                     GENERIC_READ | SYNCHRONIZE,
//                                     &ObjectAttributes,
//                                     &IoStatusBlock,
//                                     FILE_SHARE_READ |
//                                     FILE_SHARE_WRITE |
//                                     FILE_SHARE_DELETE,
//                                     FILE_SYNCHRONOUS_IO_NONALERT);
//                 if (!NT_SUCCESS(Status)) LoadDll->FileHandle = NULL;
//
//                 /* Free the name now */
//                 RtlFreeUnicodeString(&ModuleName);
//             }
//
//             /* Send the fake module load message */
//             Status = DbgkpQueueMessage(Process,
//                                        Thread,
//                                        &ApiMessage,
//                                        DEBUG_EVENT_NOWAIT,
//                                        DebugObject);
//             if (!NT_SUCCESS(Status))
//             {
//                 /* Message send failed, close the file handle if we had one */
//                 if (LoadDll->FileHandle) ObCloseHandle(LoadDll->FileHandle,
//                                                        KernelMode);
//             }
//
//             /* Go to the next module */
//             NextEntry = NextEntry->Flink;
//             i++;
//         }
//     }
//     _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//     {
//         NOTHING;
//     }
//     _SEH2_END;
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// DbgkpPostFakeThreadMessages(IN PEPROCESS Process,
//                             IN PDEBUG_OBJECT DebugObject,
//                             IN PETHREAD StartThread,
//                             OUT PETHREAD *FirstThread,
//                             OUT PETHREAD *LastThread)
// {
//     PETHREAD pFirstThread = NULL, ThisThread, OldThread = NULL, pLastThread;
//     NTSTATUS Status = STATUS_UNSUCCESSFUL;
//     BOOLEAN IsFirstThread;
//     ULONG Flags;
//     DBGKM_MSG ApiMessage;
//     PDBGKM_CREATE_THREAD CreateThread = &ApiMessage.CreateThread;
//     PDBGKM_CREATE_PROCESS CreateProcess = &ApiMessage.CreateProcess;
//     BOOLEAN First;
//     PIMAGE_NT_HEADERS NtHeader;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_THREAD_DEBUG, "Process: %p StartThread: %p Object: %p\n",
//               Process, StartThread, DebugObject);
//
//     /* Check if we have a start thread */
//     if (StartThread)
//     {
//         /* Then the one we'll find won't be the first one */
//         IsFirstThread = FALSE;
//         pFirstThread = StartThread;
//         ThisThread = StartThread;
//
//         /* Reference it */
//         ObReferenceObject(StartThread);
//     }
//     else
//     {
//         /* Get the first thread ourselves */
//         ThisThread = PsGetNextProcessThread(Process, NULL);
//         IsFirstThread = TRUE;
//     }
//
//     /* Start thread loop */
//     do
//     {
//         /* Dereference the previous thread if we had one */
//         if (OldThread) ObDereferenceObject(OldThread);
//
//         /* Set this as the last thread and lock it */
//         pLastThread = ThisThread;
//         ObReferenceObject(ThisThread);
//         if (ExAcquireRundownProtection(&ThisThread->RundownProtect))
//         {
//             /* Acquire worked, set flags */
//             Flags = DEBUG_EVENT_RELEASE | DEBUG_EVENT_NOWAIT;
//
//             /* Check if this is a user thread */
//             if (!ThisThread->SystemThread)
//             {
//                 /* Suspend it */
//                 if (NT_SUCCESS(PsSuspendThread(ThisThread, NULL)))
//                 {
//                     /* Remember this */
//                     Flags |= DEBUG_EVENT_SUSPEND;
//                 }
//             }
//         }
//         else
//         {
//             /* Couldn't acquire rundown */
//             Flags = DEBUG_EVENT_PROTECT_FAILED | DEBUG_EVENT_NOWAIT;
//         }
//
//         /* Clear the API Message */
//         RtlZeroMemory(&ApiMessage, sizeof(ApiMessage));
//
//         /* Check if this is the first thread */
//         if ((IsFirstThread) &&
//             !(Flags & DEBUG_EVENT_PROTECT_FAILED) &&
//             !(ThisThread->SystemThread) &&
//             (ThisThread->GrantedAccess))
//         {
//             /* It is, save the flag */
//             First = TRUE;
//         }
//         else
//         {
//             /* It isn't, save the flag */
//             First = FALSE;
//         }
//
//         /* Check if this is the first */
//         if (First)
//         {
//             /* So we'll start with the create process message */
//             ApiMessage.ApiNumber = DbgKmCreateProcessApi;
//
//             /* Get the file handle */
//             if (Process->SectionObject)
//             {
//                 /* Use the section object */
//                 CreateProcess->FileHandle =
//                     DbgkpSectionToFileHandle(Process->SectionObject);
//             }
//             else
//             {
//                 /* Don't return any handle */
//                 CreateProcess->FileHandle = NULL;
//             }
//
//             /* Set the base address */
//             CreateProcess->BaseOfImage = Process->SectionBaseAddress;
//
//             /* Get the NT Header */
//             NtHeader = RtlImageNtHeader(Process->SectionBaseAddress);
//             if (NtHeader)
//             {
//                 /* Fill out data from the header */
//                 CreateProcess->DebugInfoFileOffset = NtHeader->FileHeader.
//                                                      PointerToSymbolTable;
//                 CreateProcess->DebugInfoSize = NtHeader->FileHeader.
//                                                NumberOfSymbols;
//             }
//         }
//         else
//         {
//             /* Otherwise it's a thread message */
//             ApiMessage.ApiNumber = DbgKmCreateThreadApi;
//             CreateThread->StartAddress = ThisThread->StartAddress;
//         }
//
//         /* Trace */
//         DBGKTRACE(DBGK_THREAD_DEBUG, "Thread: %p. First: %lx, OldThread: %p\n",
//                   ThisThread, First, OldThread);
//         DBGKTRACE(DBGK_THREAD_DEBUG, "Start Address: %p\n",
//                   ThisThread->StartAddress);
//
//         /* Queue the message */
//         Status = DbgkpQueueMessage(Process,
//                                    ThisThread,
//                                    &ApiMessage,
//                                    Flags,
//                                    DebugObject);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Resume the thread if it was suspended */
//             if (Flags & DEBUG_EVENT_SUSPEND) PsResumeThread(ThisThread, NULL);
//
//             /* Check if we acquired rundown */
//             if (Flags & DEBUG_EVENT_RELEASE)
//             {
//                 /* Release it */
//                 ExReleaseRundownProtection(&ThisThread->RundownProtect);
//             }
//
//             /* If this was a process create, check if we got a handle */
//             if ((ApiMessage.ApiNumber == DbgKmCreateProcessApi) &&
//                  (CreateProcess->FileHandle))
//             {
//                 /* Close it */
//                 ObCloseHandle(CreateProcess->FileHandle, KernelMode);
//             }
//
//             /* Release our reference and break out */
//             ObDereferenceObject(ThisThread);
//             break;
//         }
//
//         /* Check if this was the first message */
//         if (First)
//         {
//             /* It isn't the first thread anymore */
//             IsFirstThread = FALSE;
//
//             /* Reference this thread and set it as first */
//             ObReferenceObject(ThisThread);
//             pFirstThread = ThisThread;
//         }
//
//         /* Get the next thread */
//         ThisThread = PsGetNextProcessThread(Process, ThisThread);
//         OldThread = pLastThread;
//     } while (ThisThread);
//
//     /* Check the API status */
//     if (!NT_SUCCESS(Status))
//     {
//         /* Dereference and fail */
//         if (pFirstThread) ObDereferenceObject(pFirstThread);
//         ObDereferenceObject(pLastThread);
//         return Status;
//     }
//
//     /* Make sure we have a first thread */
//     if (!pFirstThread) return STATUS_UNSUCCESSFUL;
//
//     /* Return thread pointers */
//     *FirstThread = pFirstThread;
//     *LastThread = pLastThread;
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// DbgkpPostFakeProcessCreateMessages(IN PEPROCESS Process,
//                                    IN PDEBUG_OBJECT DebugObject,
//                                    OUT PETHREAD *LastThread)
// {
//     KAPC_STATE ApcState;
//     PETHREAD FirstThread, FinalThread;
//     PETHREAD ReturnThread = NULL;
//     NTSTATUS Status;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_PROCESS_DEBUG, "Process: %p DebugObject: %p\n",
//               Process, DebugObject);
//
//     /* Attach to the process */
//     KeStackAttachProcess(&Process->Pcb, &ApcState);
//
//     /* Post the fake thread messages */
//     Status = DbgkpPostFakeThreadMessages(Process,
//                                          DebugObject,
//                                          NULL,
//                                          &FirstThread,
//                                          &FinalThread);
//     if (NT_SUCCESS(Status))
//     {
//         /* Send the fake module messages too */
//         Status = DbgkpPostFakeModuleMessages(Process,
//                                              FirstThread,
//                                              DebugObject);
//         if (!NT_SUCCESS(Status))
//         {
//             /* We failed, dereference the final thread */
//             ObDereferenceObject(FinalThread);
//         }
//         else
//         {
//             /* Set the final thread */
//             ReturnThread = FinalThread;
//         }
//
//         /* Dereference the first thread */
//         ObDereferenceObject(FirstThread);
//     }
//
//     /* Detach from the process */
//     KeUnstackDetachProcess(&ApcState);
//
//     /* Return the last thread */
//     *LastThread = ReturnThread;
//     return Status;
// }
//
// VOID
// NTAPI
// DbgkpConvertKernelToUserStateChange(IN PDBGUI_WAIT_STATE_CHANGE WaitStateChange,
//                                     IN PDEBUG_EVENT DebugEvent)
// {
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "DebugEvent: %p\n", DebugEvent);
//
//     /* Start by copying the client ID */
//     WaitStateChange->AppClientId = DebugEvent->ClientId;
//
//     /* Now check which kind of event this was */
//     switch (DebugEvent->ApiMsg.ApiNumber)
//     {
//         /* New process */
//         case DbgKmCreateProcessApi:
//
//             /* Set the right native code */
//             WaitStateChange->NewState = DbgCreateProcessStateChange;
//
//             /* Copy the information */
//             WaitStateChange->StateInfo.CreateProcessInfo.NewProcess =
//                 DebugEvent->ApiMsg.CreateProcess;
//
//             /* Clear the file handle for us */
//             DebugEvent->ApiMsg.CreateProcess.FileHandle = NULL;
//             break;
//
//         /* New thread */
//         case DbgKmCreateThreadApi:
//
//             /* Set the right native code */
//             WaitStateChange->NewState = DbgCreateThreadStateChange;
//
//             /* Copy information */
//             WaitStateChange->StateInfo.CreateThread.NewThread.StartAddress =
//                 DebugEvent->ApiMsg.CreateThread.StartAddress;
//             WaitStateChange->StateInfo.CreateThread.NewThread.SubSystemKey =
//                 DebugEvent->ApiMsg.CreateThread.SubSystemKey;
//             break;
//
//         /* Exception (or breakpoint/step) */
//         case DbgKmExceptionApi:
//
//             /* Look at the exception code */
//             if ((NTSTATUS)DebugEvent->ApiMsg.Exception.ExceptionRecord.ExceptionCode ==
//                 STATUS_BREAKPOINT)
//             {
//                 /* Update this as a breakpoint exception */
//                 WaitStateChange->NewState = DbgBreakpointStateChange;
//             }
//             else if ((NTSTATUS)DebugEvent->ApiMsg.Exception.ExceptionRecord.ExceptionCode ==
//                      STATUS_SINGLE_STEP)
//             {
//                 /* Update this as a single step exception */
//                 WaitStateChange->NewState = DbgSingleStepStateChange;
//             }
//             else
//             {
//                 /* Otherwise, set default exception */
//                 WaitStateChange->NewState = DbgExceptionStateChange;
//             }
//
//             /* Copy the exception record */
//             WaitStateChange->StateInfo.Exception.ExceptionRecord =
//                 DebugEvent->ApiMsg.Exception.ExceptionRecord;
//             /* Copy FirstChance flag */
//             WaitStateChange->StateInfo.Exception.FirstChance =
//                 DebugEvent->ApiMsg.Exception.FirstChance;
//             break;
//
//         /* Process exited */
//         case DbgKmExitProcessApi:
//
//             /* Set the right native code and copy the exit code */
//             WaitStateChange->NewState = DbgExitProcessStateChange;
//             WaitStateChange->StateInfo.ExitProcess.ExitStatus =
//                 DebugEvent->ApiMsg.ExitProcess.ExitStatus;
//             break;
//
//         /* Thread exited */
//         case DbgKmExitThreadApi:
//
//             /* Set the right native code */
//             WaitStateChange->NewState = DbgExitThreadStateChange;
//             WaitStateChange->StateInfo.ExitThread.ExitStatus =
//                 DebugEvent->ApiMsg.ExitThread.ExitStatus;
//             break;
//
//         /* DLL Load */
//         case DbgKmLoadDllApi:
//
//             /* Set the native code */
//             WaitStateChange->NewState = DbgLoadDllStateChange;
//
//             /* Copy the data */
//             WaitStateChange->StateInfo.LoadDll = DebugEvent->ApiMsg.LoadDll;
//
//             /* Clear the file handle for us */
//             DebugEvent->ApiMsg.LoadDll.FileHandle = NULL;
//             break;
//
//         /* DLL Unload */
//         case DbgKmUnloadDllApi:
//
//             /* Set the native code and copy the address */
//             WaitStateChange->NewState = DbgUnloadDllStateChange;
//             WaitStateChange->StateInfo.UnloadDll.BaseAddress =
//                 DebugEvent->ApiMsg.UnloadDll.BaseAddress;
//             break;
//
//         default:
//
//             /* Shouldn't happen */
//             ASSERT(FALSE);
//     }
// }
//
// VOID
// NTAPI
// DbgkpMarkProcessPeb(IN PEPROCESS Process)
// {
//     KAPC_STATE ApcState;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_PROCESS_DEBUG, "Process: %p\n", Process);
//
//     /* Acquire process rundown */
//     if (!ExAcquireRundownProtection(&Process->RundownProtect)) return;
//
//     /* Make sure we have a PEB */
//     if (Process->Peb)
//     {
//         /* Attach to the process */
//         KeStackAttachProcess(&Process->Pcb, &ApcState);
//
//         /* Acquire the debug port mutex */
//         ExAcquireFastMutex(&DbgkpProcessDebugPortMutex);
//
//         /* Set the IsBeingDebugged member of the PEB */
//         Process->Peb->BeingDebugged = (Process->DebugPort) ? TRUE: FALSE;
//
//         /* Release lock */
//         ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
//
//         /* Detach from the process */
//         KeUnstackDetachProcess(&ApcState);
//     }
//
//     /* Release rundown protection */
//     ExReleaseRundownProtection(&Process->RundownProtect);
// }
//
// VOID
// NTAPI
// DbgkpOpenHandles(IN PDBGUI_WAIT_STATE_CHANGE WaitStateChange,
//                  IN PEPROCESS Process,
//                  IN PETHREAD Thread)
// {
//     NTSTATUS Status;
//     HANDLE Handle;
//     PHANDLE DupHandle;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "Process: %p Thread: %p State: %lx\n",
//               Process, Thread, WaitStateChange->NewState);
//
//     /* Check which state this is */
//     switch (WaitStateChange->NewState)
//     {
//         /* New thread */
//         case DbgCreateThreadStateChange:
//
//             /* Get handle to thread */
//             Status = ObOpenObjectByPointer(Thread,
//                                            0,
//                                            NULL,
//                                            THREAD_ALL_ACCESS,
//                                            PsThreadType,
//                                            KernelMode,
//                                            &Handle);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Save the thread handle */
//                 WaitStateChange->
//                     StateInfo.CreateThread.HandleToThread = Handle;
//             }
//             return;
//
//         /* New process */
//         case DbgCreateProcessStateChange:
//
//             /* Get handle to thread */
//             Status = ObOpenObjectByPointer(Thread,
//                                            0,
//                                            NULL,
//                                            THREAD_ALL_ACCESS,
//                                            PsThreadType,
//                                            KernelMode,
//                                            &Handle);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Save the thread handle */
//                 WaitStateChange->
//                     StateInfo.CreateProcessInfo.HandleToThread = Handle;
//             }
//
//             /* Get handle to process */
//             Status = ObOpenObjectByPointer(Process,
//                                            0,
//                                            NULL,
//                                            PROCESS_ALL_ACCESS,
//                                            PsProcessType,
//                                            KernelMode,
//                                            &Handle);
//             if (NT_SUCCESS(Status))
//             {
//                 /* Save the process handle */
//                 WaitStateChange->
//                     StateInfo.CreateProcessInfo.HandleToProcess = Handle;
//             }
//
//             /* Fall through to duplicate file handle */
//             DupHandle = &WaitStateChange->
//                             StateInfo.CreateProcessInfo.NewProcess.FileHandle;
//             break;
//
//         /* DLL Load */
//         case DbgLoadDllStateChange:
//
//             /* Fall through to duplicate file handle */
//             DupHandle = &WaitStateChange->StateInfo.LoadDll.FileHandle;
//             break;
//
//         /* Anything else has no handles */
//         default:
//             return;
//     }
//
//     /* If we got here, then we have to duplicate a handle, possibly */
//     Handle = *DupHandle;
//     if (Handle)
//     {
//         /* Duplicate it */
//         Status = ObDuplicateObject(PsGetCurrentProcess(),
//                                    Handle,
//                                    PsGetCurrentProcess(),
//                                    DupHandle,
//                                    0,
//                                    0,
//                                    DUPLICATE_SAME_ACCESS,
//                                    KernelMode);
//         if (!NT_SUCCESS(Status)) *DupHandle = NULL;
//
//         /* Close the original handle */
//         ObCloseHandle(Handle, KernelMode);
//     }
// }
//
// VOID
// NTAPI
// DbgkpDeleteObject(IN PVOID DebugObject)
// {
//     PAGED_CODE();
//
//     /* Sanity check */
//     ASSERT(IsListEmpty(&((PDEBUG_OBJECT)DebugObject)->EventList));
// }
//
// VOID
// NTAPI
// DbgkpCloseObject(IN PEPROCESS OwnerProcess OPTIONAL,
//                  IN PVOID ObjectBody,
//                  IN ACCESS_MASK GrantedAccess,
//                  IN ULONG HandleCount,
//                  IN ULONG SystemHandleCount)
// {
//     PDEBUG_OBJECT DebugObject = ObjectBody;
//     PEPROCESS Process = NULL;
//     BOOLEAN DebugPortCleared = FALSE;
//     PLIST_ENTRY DebugEventList;
//     PDEBUG_EVENT DebugEvent;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "OwnerProcess: %p DebugObject: %p\n",
//               OwnerProcess, DebugObject);
//
//     /* If this isn't the last handle, do nothing */
//     if (SystemHandleCount > 1) return;
//
//     /* Otherwise, lock the debug object */
//     ExAcquireFastMutex(&DebugObject->Mutex);
//
//     /* Set it as inactive */
//     DebugObject->DebuggerInactive = TRUE;
//
//     /* Remove it from the debug event list */
//     DebugEventList = DebugObject->EventList.Flink;
//     InitializeListHead(&DebugObject->EventList);
//
//     /* Release the lock */
//     ExReleaseFastMutex(&DebugObject->Mutex);
//
//     /* Signal the wait event */
//     KeSetEvent(&DebugObject->EventsPresent, IO_NO_INCREMENT, FALSE);
//
//     /* Start looping each process */
//     while ((Process = PsGetNextProcess(Process)))
//     {
//         /* Check if the process has us as their debug port */
//         if (Process->DebugPort == DebugObject)
//         {
//             /* Acquire the process debug port lock */
//             ExAcquireFastMutex(&DbgkpProcessDebugPortMutex);
//
//             /* Check if it's still us */
//             if (Process->DebugPort == DebugObject)
//             {
//                 /* Clear it and remember */
//                 Process->DebugPort = NULL;
//                 DebugPortCleared = TRUE;
//             }
//
//             /* Release the port lock */
//             ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
//
//             /* Check if we cleared the debug port */
//             if (DebugPortCleared)
//             {
//                 /* Mark this in the PEB */
//                 DbgkpMarkProcessPeb(Process);
//
//                 /* Check if we terminate on exit */
//                 if (DebugObject->KillProcessOnExit)
//                 {
//                     /* Terminate the process */
//                     PsTerminateProcess(Process, STATUS_DEBUGGER_INACTIVE);
//                 }
//
//                 /* Dereference the debug object */
//                 ObDereferenceObject(DebugObject);
//             }
//         }
//     }
//
//     /* Loop debug events */
//     while (DebugEventList != &DebugObject->EventList)
//     {
//         /* Get the debug event */
//         DebugEvent = CONTAINING_RECORD(DebugEventList, DEBUG_EVENT, EventList);
//
//         /* Go to the next entry */
//         DebugEventList = DebugEventList->Flink;
//
//         /* Wake it up */
//         DebugEvent->Status = STATUS_DEBUGGER_INACTIVE;
//         DbgkpWakeTarget(DebugEvent);
//     }
// }
//
// NTSTATUS
// NTAPI
// DbgkpSetProcessDebugObject(IN PEPROCESS Process,
//                            IN PDEBUG_OBJECT DebugObject,
//                            IN NTSTATUS MsgStatus,
//                            IN PETHREAD LastThread)
// {
//     NTSTATUS Status;
//     LIST_ENTRY TempList;
//     BOOLEAN GlobalHeld = FALSE, DoSetEvent = TRUE;
//     PETHREAD ThisThread, FirstThread;
//     PLIST_ENTRY NextEntry;
//     PDEBUG_EVENT DebugEvent;
//     PETHREAD EventThread;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_PROCESS_DEBUG, "Process: %p DebugObject: %p\n",
//               Process, DebugObject);
//
//     /* Initialize the temporary list */
//     InitializeListHead(&TempList);
//
//     /* Check if we have a success message */
//     if (NT_SUCCESS(MsgStatus))
//     {
//         /* Then default to STATUS_SUCCESS */
//         Status = STATUS_SUCCESS;
//     }
//     else
//     {
//         /* No last thread, and set the failure code */
//         LastThread = NULL;
//         Status = MsgStatus;
//     }
//
//     /* Now check what status we have here */
//     if (NT_SUCCESS(Status))
//     {
//         /* Acquire the global lock */
// ThreadScan:
//         GlobalHeld = TRUE;
//         ExAcquireFastMutex(&DbgkpProcessDebugPortMutex);
//
//         /* Check if we already have a port */
//         if (Process->DebugPort)
//         {
//             /* Set failure */
//             Status = STATUS_PORT_ALREADY_SET;
//         }
//         else
//         {
//             /* Otherwise, set the port and reference the thread */
//             Process->DebugPort = DebugObject;
//             ObReferenceObject(LastThread);
//
//             /* Get the next thread */
//             ThisThread  = PsGetNextProcessThread(Process, LastThread);
//             if (ThisThread)
//             {
//                 /* Clear the debug port and release the lock */
//                 Process->DebugPort = NULL;
//                 ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
//                 GlobalHeld = FALSE;
//
//                 /* Dereference the thread */
//                 ObDereferenceObject(LastThread);
//
//                 /* Post fake messages */
//                 Status = DbgkpPostFakeThreadMessages(Process,
//                                                      DebugObject,
//                                                      ThisThread,
//                                                      &FirstThread,
//                                                      &LastThread);
//                 if (!NT_SUCCESS(Status))
//                 {
//                     /* Clear the last thread */
//                     LastThread = NULL;
//                 }
//                 else
//                 {
//                     /* Dereference the first thread and re-acquire the lock */
//                     ObDereferenceObject(FirstThread);
//                     goto ThreadScan;
//                 }
//             }
//         }
//     }
//
//     /* Acquire the debug object's lock */
//     ExAcquireFastMutex(&DebugObject->Mutex);
//
//     /* Check our status here */
//     if (NT_SUCCESS(Status))
//     {
//         /* Check if we're disconnected */
//         if (DebugObject->DebuggerInactive)
//         {
//             /* Set status */
//             Process->DebugPort = NULL;
//             Status = STATUS_DEBUGGER_INACTIVE;
//         }
//         else
//         {
//             /* Set the process flags */
//             PspSetProcessFlag(Process,
//                               PSF_NO_DEBUG_INHERIT_BIT |
//                               PSF_CREATE_REPORTED_BIT);
//
//             /* Reference the debug object */
//             ObReferenceObject(DebugObject);
//         }
//     }
//
//     /* Loop the events list */
//     NextEntry = DebugObject->EventList.Flink;
//     while (NextEntry != &DebugObject->EventList)
//     {
//         /* Get the debug event and go to the next entry */
//         DebugEvent = CONTAINING_RECORD(NextEntry, DEBUG_EVENT, EventList);
//         NextEntry = NextEntry->Flink;
//         DBGKTRACE(DBGK_PROCESS_DEBUG, "DebugEvent: %p Flags: %lx TH: %p/%p\n",
//                   DebugEvent, DebugEvent->Flags,
//                   DebugEvent->BackoutThread, PsGetCurrentThread());
//
//         /* Check for if the debug event queue needs flushing */
//         if ((DebugEvent->Flags & DEBUG_EVENT_INACTIVE) &&
//             (DebugEvent->BackoutThread == PsGetCurrentThread()))
//         {
//             /* Get the event's thread */
//             EventThread = DebugEvent->Thread;
//             DBGKTRACE(DBGK_PROCESS_DEBUG, "EventThread: %p MsgStatus: %lx\n",
//                       EventThread, MsgStatus);
//
//             /* Check if the status is success */
//             if ((MsgStatus == STATUS_SUCCESS) &&
//                 (EventThread->GrantedAccess) &&
//                 (!EventThread->SystemThread))
//             {
//                 /* Check if we couldn't acquire rundown for it */
//                 if (DebugEvent->Flags & DEBUG_EVENT_PROTECT_FAILED)
//                 {
//                     /* Set the skip termination flag */
//                     PspSetCrossThreadFlag(EventThread, CT_SKIP_CREATION_MSG_BIT);
//
//                     /* Insert it into the temp list */
//                     RemoveEntryList(&DebugEvent->EventList);
//                     InsertTailList(&TempList, &DebugEvent->EventList);
//                 }
//                 else
//                 {
//                     /* Do we need to signal the event */
//                     if (DoSetEvent)
//                     {
//                         /* Do it */
//                         DebugEvent->Flags &= ~DEBUG_EVENT_INACTIVE;
//                         KeSetEvent(&DebugObject->EventsPresent,
//                                    IO_NO_INCREMENT,
//                                    FALSE);
//                         DoSetEvent = FALSE;
//                     }
//
//                     /* Clear the backout thread */
//                     DebugEvent->BackoutThread = NULL;
//
//                     /* Set skip flag */
//                     PspSetCrossThreadFlag(EventThread, CT_SKIP_CREATION_MSG_BIT);
//                 }
//             }
//             else
//             {
//                 /* Insert it into the temp list */
//                 RemoveEntryList(&DebugEvent->EventList);
//                 InsertTailList(&TempList, &DebugEvent->EventList);
//             }
//
//             /* Check if the lock is held */
//             if (DebugEvent->Flags & DEBUG_EVENT_RELEASE)
//             {
//                 /* Release it */
//                 DebugEvent->Flags &= ~DEBUG_EVENT_RELEASE;
//                 ExReleaseRundownProtection(&EventThread->RundownProtect);
//             }
//         }
//     }
//
//     /* Release the debug object */
//     ExReleaseFastMutex(&DebugObject->Mutex);
//
//     /* Release the global lock if acquired */
//     if (GlobalHeld) ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
//
//     /* Check if there's a thread to dereference */
//     if (LastThread) ObDereferenceObject(LastThread);
//
//     /* Loop our temporary list */
//     while (!IsListEmpty(&TempList))
//     {
//         /* Remove the event */
//         NextEntry = RemoveHeadList(&TempList);
//         DebugEvent = CONTAINING_RECORD(NextEntry, DEBUG_EVENT, EventList);
//
//         /* Wake it */
//         DbgkpWakeTarget(DebugEvent);
//     }
//
//     /* Check if we got here through success and mark the PEB, then return */
//     if (NT_SUCCESS(Status)) DbgkpMarkProcessPeb(Process);
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// DbgkClearProcessDebugObject(IN PEPROCESS Process,
//                             IN PDEBUG_OBJECT SourceDebugObject OPTIONAL)
// {
//     PDEBUG_OBJECT DebugObject;
//     PDEBUG_EVENT DebugEvent;
//     LIST_ENTRY TempList;
//     PLIST_ENTRY NextEntry;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "Process: %p DebugObject: %p\n",
//               Process, SourceDebugObject);
//
//     /* Acquire the port lock */
//     ExAcquireFastMutex(&DbgkpProcessDebugPortMutex);
//
//     /* Get the Process Debug Object */
//     DebugObject = Process->DebugPort;
//
//     /*
//      * Check if the process had an object and it matches,
//      * or if the process had an object but none was specified
//      * (in which we are called from NtTerminateProcess)
//      */
//     if ((DebugObject) &&
//         ((DebugObject == SourceDebugObject) ||
//          (SourceDebugObject == NULL)))
//     {
//         /* Clear the debug port */
//         Process->DebugPort = NULL;
//
//         /* Release the port lock and remove the PEB flag */
//         ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
//         DbgkpMarkProcessPeb(Process);
//     }
//     else
//     {
//         /* Release the port lock and fail */
//         ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
//         return STATUS_PORT_NOT_SET;
//     }
//
//     /* Initialize the temporary list */
//     InitializeListHead(&TempList);
//
//     /* Acquire the Object */
//     ExAcquireFastMutex(&DebugObject->Mutex);
//
//     /* Loop the events */
//     NextEntry = DebugObject->EventList.Flink;
//     while (NextEntry != &DebugObject->EventList)
//     {
//         /* Get the Event and go to the next entry */
//         DebugEvent = CONTAINING_RECORD(NextEntry, DEBUG_EVENT, EventList);
//         NextEntry = NextEntry->Flink;
//
//         /* Check that it belongs to the specified process */
//         if (DebugEvent->Process == Process)
//         {
//             /* Insert it into the temporary list */
//             RemoveEntryList(&DebugEvent->EventList);
//             InsertTailList(&TempList, &DebugEvent->EventList);
//         }
//     }
//
//     /* Release the Object */
//     ExReleaseFastMutex(&DebugObject->Mutex);
//
//     /* Release the initial reference */
//     ObDereferenceObject(DebugObject);
//
//     /* Loop our temporary list */
//     while (!IsListEmpty(&TempList))
//     {
//         /* Remove the event */
//         NextEntry = RemoveHeadList(&TempList);
//         DebugEvent = CONTAINING_RECORD(NextEntry, DEBUG_EVENT, EventList);
//
//         /* Wake it up */
//         DebugEvent->Status = STATUS_DEBUGGER_INACTIVE;
//         DbgkpWakeTarget(DebugEvent);
//     }
//
//     /* Return Success */
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// DbgkInitialize(VOID)
// {
//     OBJECT_TYPE_INITIALIZER ObjectTypeInitializer;
//     UNICODE_STRING Name;
//     PAGED_CODE();
//
//     /* Initialize the process debug port mutex */
//     ExInitializeFastMutex(&DbgkpProcessDebugPortMutex);
//
//     /* Create the Debug Object Type */
//     RtlZeroMemory(&ObjectTypeInitializer, sizeof(ObjectTypeInitializer));
//     RtlInitUnicodeString(&Name, L"DebugObject");
//     ObjectTypeInitializer.Length = sizeof(ObjectTypeInitializer);
//     ObjectTypeInitializer.DefaultNonPagedPoolCharge = sizeof(DEBUG_OBJECT);
//     ObjectTypeInitializer.GenericMapping = DbgkDebugObjectMapping;
//     ObjectTypeInitializer.PoolType = NonPagedPool;
//     ObjectTypeInitializer.ValidAccessMask = DEBUG_OBJECT_ALL_ACCESS;
//     ObjectTypeInitializer.SecurityRequired = TRUE;
//     ObjectTypeInitializer.CloseProcedure = DbgkpCloseObject;
//     ObjectTypeInitializer.DeleteProcedure = DbgkpDeleteObject;
//     ObCreateObjectType(&Name,
//                        &ObjectTypeInitializer,
//                        NULL,
//                        &DbgkDebugObjectType);
// }
//
// NTSTATUS
// NTAPI
// DbgkOpenProcessDebugPort(IN PEPROCESS Process,
//                          IN KPROCESSOR_MODE PreviousMode,
//                          OUT HANDLE *DebugHandle)
// {
//     PDEBUG_OBJECT DebugObject;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* If there's no debug port, just exit */
//     if (!Process->DebugPort) return STATUS_PORT_NOT_SET;
//
//     /* Otherwise, acquire the lock while we grab the port */
//     ExAcquireFastMutex(&DbgkpProcessDebugPortMutex);
//
//     /* Grab it and reference it if it exists */
//     DebugObject = Process->DebugPort;
//     if (DebugObject) ObReferenceObject(DebugObject);
//
//     /* Release the lock now */
//     ExReleaseFastMutex(&DbgkpProcessDebugPortMutex);
//
//     /* Bail out if it doesn't exist */
//     if (!DebugObject) return STATUS_PORT_NOT_SET;
//
//     /* Now get a handle to it */
//     Status = ObOpenObjectByPointer(DebugObject,
//                                    0,
//                                    NULL,
//                                    MAXIMUM_ALLOWED,
//                                    DbgkDebugObjectType,
//                                    PreviousMode,
//                                    DebugHandle);
//     if (!NT_SUCCESS(Status)) ObDereferenceObject(DebugObject);
//
//     /* Return status */
//     return Status;
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtCreateDebugObject(OUT PHANDLE DebugHandle,
//                     IN ACCESS_MASK DesiredAccess,
//                     IN POBJECT_ATTRIBUTES ObjectAttributes,
//                     IN ULONG Flags)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PDEBUG_OBJECT DebugObject;
//     HANDLE hDebug;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if we were called from user mode*/
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the handle */
//             ProbeForWriteHandle(DebugHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         } _SEH2_END;
//     }
//
//     /* Check for invalid flags */
//     if (Flags & ~DBGK_ALL_FLAGS) return STATUS_INVALID_PARAMETER;
//
//     /* Create the Object */
//     Status = ObCreateObject(PreviousMode,
//                             DbgkDebugObjectType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(DEBUG_OBJECT),
//                             0,
//                             0,
//                             (PVOID*)&DebugObject);
//     if (NT_SUCCESS(Status))
//     {
//         /* Initialize the Debug Object's Fast Mutex */
//         ExInitializeFastMutex(&DebugObject->Mutex);
//
//         /* Initialize the State Event List */
//         InitializeListHead(&DebugObject->EventList);
//
//         /* Initialize the Debug Object's Wait Event */
//         KeInitializeEvent(&DebugObject->EventsPresent,
//                           NotificationEvent,
//                           FALSE);
//
//         /* Set the Flags */
//         DebugObject->Flags = 0;
//         if (Flags & DBGK_KILL_PROCESS_ON_EXIT)
//         {
//             DebugObject->KillProcessOnExit = TRUE;
//         }
//
//         /* Insert it */
//         Status = ObInsertObject((PVOID)DebugObject,
//                                  NULL,
//                                  DesiredAccess,
//                                  0,
//                                  NULL,
//                                  &hDebug);
//         if (NT_SUCCESS(Status))
//         {
//             /* Enter SEH to protect the write */
//             _SEH2_TRY
//             {
//                 /* Return the handle */
//                 *DebugHandle = hDebug;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             } _SEH2_END;
//         }
//     }
//
//     /* Return Status */
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "Handle: %p DebugObject: %p\n",
//               hDebug, DebugObject);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtDebugContinue(IN HANDLE DebugHandle,
//                 IN PCLIENT_ID AppClientId,
//                 IN NTSTATUS ContinueStatus)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PDEBUG_OBJECT DebugObject;
//     NTSTATUS Status;
//     PDEBUG_EVENT DebugEvent = NULL, DebugEventToWake = NULL;
//     PLIST_ENTRY ListHead, NextEntry;
//     BOOLEAN NeedsWake = FALSE;
//     CLIENT_ID ClientId;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "Handle: %p Status: %d\n",
//               DebugHandle, ContinueStatus);
//
//     /* Check if we were called from user mode*/
//     if (PreviousMode != KernelMode)
//     {
//         /* Enter SEH for probing */
//         _SEH2_TRY
//         {
//             /* Probe the handle */
//             ProbeForRead(AppClientId, sizeof(CLIENT_ID), sizeof(ULONG));
//             ClientId = *AppClientId;
//             AppClientId = &ClientId;
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         } _SEH2_END;
//     }
//
//     /* Make sure that the status is valid */
//     if ((ContinueStatus != DBG_CONTINUE) &&
//         (ContinueStatus != DBG_EXCEPTION_HANDLED) &&
//         (ContinueStatus != DBG_EXCEPTION_NOT_HANDLED) &&
//         (ContinueStatus != DBG_TERMINATE_THREAD) &&
//         (ContinueStatus != DBG_TERMINATE_PROCESS))
//     {
//         /* Invalid status */
//         Status = STATUS_INVALID_PARAMETER;
//     }
//     else
//     {
//         /* Get the debug object */
//         Status = ObReferenceObjectByHandle(DebugHandle,
//                                            DEBUG_OBJECT_WAIT_STATE_CHANGE,
//                                            DbgkDebugObjectType,
//                                            PreviousMode,
//                                            (PVOID*)&DebugObject,
//                                            NULL);
//         if (NT_SUCCESS(Status))
//         {
//             /* Acquire the mutex */
//             ExAcquireFastMutex(&DebugObject->Mutex);
//
//             /* Loop the state list */
//             ListHead = &DebugObject->EventList;
//             NextEntry = ListHead->Flink;
//             while (ListHead != NextEntry)
//             {
//                 /* Get the current debug event */
//                 DebugEvent = CONTAINING_RECORD(NextEntry,
//                                                DEBUG_EVENT,
//                                                EventList);
//
//                 /* Compare process ID */
//                 if (DebugEvent->ClientId.UniqueProcess ==
//                     AppClientId->UniqueProcess)
//                 {
//                     /* Check if we already found a match */
//                     if (NeedsWake)
//                     {
//                         /* Wake it up and break out */
//                         DebugEvent->Flags &= ~DEBUG_EVENT_INACTIVE;
//                         KeSetEvent(&DebugObject->EventsPresent,
//                                    IO_NO_INCREMENT,
//                                    FALSE);
//                         break;
//                     }
//
//                     /* Compare thread ID and flag */
//                     if ((DebugEvent->ClientId.UniqueThread ==
//                         AppClientId->UniqueThread) && (DebugEvent->Flags & DEBUG_EVENT_READ))
//                     {
//                         /* Remove the event from the list */
//                         RemoveEntryList(NextEntry);
//
//                         /* Remember who to wake */
//                         NeedsWake = TRUE;
//                         DebugEventToWake = DebugEvent;
//                     }
//                 }
//
//                 /* Go to the next entry */
//                 NextEntry = NextEntry->Flink;
//             }
//
//             /* Release the mutex */
//             ExReleaseFastMutex(&DebugObject->Mutex);
//
//             /* Dereference the object */
//             ObDereferenceObject(DebugObject);
//
//             /* Check if need a wait */
//             if (NeedsWake)
//             {
//                 /* Set the continue status */
//                 DebugEventToWake->ApiMsg.ReturnedStatus = ContinueStatus;
//                 DebugEventToWake->Status = STATUS_SUCCESS;
//
//                 /* Wake the target */
//                 DbgkpWakeTarget(DebugEventToWake);
//             }
//             else
//             {
//                 /* Fail */
//                 Status = STATUS_INVALID_PARAMETER;
//             }
//         }
//     }
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
// NtDebugActiveProcess(IN HANDLE ProcessHandle,
//                      IN HANDLE DebugHandle)
// {
//     PEPROCESS Process;
//     PDEBUG_OBJECT DebugObject;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     PETHREAD LastThread;
//     NTSTATUS Status;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_PROCESS_DEBUG, "Process: %p Handle: %p\n",
//               ProcessHandle, DebugHandle);
//
//     /* Reference the process */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_SUSPEND_RESUME,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Don't allow debugging the current process or the system process */
//     if ((Process == PsGetCurrentProcess()) ||
//          (Process == PsInitialSystemProcess))
//     {
//         /* Dereference and fail */
//         ObDereferenceObject(Process);
//         return STATUS_ACCESS_DENIED;
//     }
//
//     /* Reference the debug object */
//     Status = ObReferenceObjectByHandle(DebugHandle,
//                                        DEBUG_OBJECT_ADD_REMOVE_PROCESS,
//                                        DbgkDebugObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&DebugObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Dereference the process and exit */
//         ObDereferenceObject(Process);
//         return Status;
//     }
//
//     /* Acquire process rundown protection */
//     if (!ExAcquireRundownProtection(&Process->RundownProtect))
//     {
//         /* Dereference the process and debug object and exit */
//         ObDereferenceObject(Process);
//         ObDereferenceObject(DebugObject);
//         return STATUS_PROCESS_IS_TERMINATING;
//     }
//
//     /* Send fake create messages for debuggers to have a consistent state */
//     Status = DbgkpPostFakeProcessCreateMessages(Process,
//                                                 DebugObject,
//                                                 &LastThread);
//     Status = DbgkpSetProcessDebugObject(Process,
//                                         DebugObject,
//                                         Status,
//                                         LastThread);
//
//     /* Release rundown protection */
//     ExReleaseRundownProtection(&Process->RundownProtect);
//
//     /* Dereference the process and debug object and return status */
//     ObDereferenceObject(Process);
//     ObDereferenceObject(DebugObject);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtRemoveProcessDebug(IN HANDLE ProcessHandle,
//                      IN HANDLE DebugHandle)
// {
//     PEPROCESS Process;
//     PDEBUG_OBJECT DebugObject;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_PROCESS_DEBUG, "Process: %p Handle: %p\n",
//               ProcessHandle, DebugHandle);
//
//     /* Reference the process */
//     Status = ObReferenceObjectByHandle(ProcessHandle,
//                                        PROCESS_SUSPEND_RESUME,
//                                        PsProcessType,
//                                        PreviousMode,
//                                        (PVOID*)&Process,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Reference the debug object */
//     Status = ObReferenceObjectByHandle(DebugHandle,
//                                        DEBUG_OBJECT_ADD_REMOVE_PROCESS,
//                                        DbgkDebugObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&DebugObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Dereference the process and exit */
//         ObDereferenceObject(Process);
//         return Status;
//     }
//
//     /* Remove the debug object */
//     Status = DbgkClearProcessDebugObject(Process, DebugObject);
//
//     /* Dereference the process and debug object and return status */
//     ObDereferenceObject(Process);
//     ObDereferenceObject(DebugObject);
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtSetInformationDebugObject(IN HANDLE DebugHandle,
//                             IN DEBUGOBJECTINFOCLASS DebugObjectInformationClass,
//                             IN PVOID DebugInformation,
//                             IN ULONG DebugInformationLength,
//                             OUT PULONG ReturnLength OPTIONAL)
// {
//     PDEBUG_OBJECT DebugObject;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PDEBUG_OBJECT_KILL_PROCESS_ON_EXIT_INFORMATION DebugInfo = DebugInformation;
//     PAGED_CODE();
//
//     /* Check buffers and parameters */
//     Status = DefaultSetInfoBufferCheck(DebugObjectInformationClass,
//                                        DbgkpDebugObjectInfoClass,
//                                        sizeof(DbgkpDebugObjectInfoClass) /
//                                        sizeof(DbgkpDebugObjectInfoClass[0]),
//                                        DebugInformation,
//                                        DebugInformationLength,
//                                        PreviousMode);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Check if the caller wanted the return length */
//     if (ReturnLength)
//     {
//         /* Enter SEH for probe */
//         _SEH2_TRY
//         {
//             /* Return required length to user-mode */
//             ProbeForWriteUlong(ReturnLength);
//             *ReturnLength = sizeof(*DebugInfo);
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(DebugHandle,
//                                        DEBUG_OBJECT_WAIT_STATE_CHANGE,
//                                        DbgkDebugObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&DebugObject,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Acquire the object */
//         ExAcquireFastMutex(&DebugObject->Mutex);
//
//         /* Set the proper flag */
//         if (DebugInfo->KillProcessOnExit)
//         {
//             /* Enable killing the process */
//             DebugObject->KillProcessOnExit = TRUE;
//         }
//         else
//         {
//             /* Disable */
//             DebugObject->KillProcessOnExit = FALSE;
//         }
//
//         /* Release the mutex */
//         ExReleaseFastMutex(&DebugObject->Mutex);
//
//         /* Release the Object */
//         ObDereferenceObject(DebugObject);
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// NtWaitForDebugEvent(IN HANDLE DebugHandle,
//                     IN BOOLEAN Alertable,
//                     IN PLARGE_INTEGER Timeout OPTIONAL,
//                     OUT PDBGUI_WAIT_STATE_CHANGE StateChange)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     LARGE_INTEGER LocalTimeOut;
//     PEPROCESS Process;
//     LARGE_INTEGER StartTime;
//     PETHREAD Thread;
//     BOOLEAN GotEvent;
//     LARGE_INTEGER NewTime;
//     PDEBUG_OBJECT DebugObject;
//     DBGUI_WAIT_STATE_CHANGE WaitStateChange;
//     NTSTATUS Status;
//     PDEBUG_EVENT DebugEvent = NULL, DebugEvent2;
//     PLIST_ENTRY ListHead, NextEntry, NextEntry2;
//     PAGED_CODE();
//     DBGKTRACE(DBGK_OBJECT_DEBUG, "Handle: %p\n", DebugHandle);
//
//     /* Clear the initial wait state change structure and the timeout */
//     RtlZeroMemory(&WaitStateChange, sizeof(WaitStateChange));
//     LocalTimeOut.QuadPart = 0;
//
//     /* Check if we were called from user mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Protect probe in SEH */
//         _SEH2_TRY
//         {
//             /* Check if we came with a timeout */
//             if (Timeout)
//             {
//                 /* Probe it */
//                 ProbeForReadLargeInteger(Timeout);
//
//                 /* Make a local copy */
//                 LocalTimeOut = *Timeout;
//                 Timeout = &LocalTimeOut;
//             }
//
//             /* Probe the state change structure */
//             ProbeForWrite(StateChange, sizeof(*StateChange), sizeof(ULONG));
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//     else
//     {
//         /* Copy directly */
//         if (Timeout) LocalTimeOut = *Timeout;
//     }
//
//     /* If we were passed a timeout, query the current time */
//     if (Timeout) KeQuerySystemTime(&StartTime);
//
//     /* Get the debug object */
//     Status = ObReferenceObjectByHandle(DebugHandle,
//                                        DEBUG_OBJECT_WAIT_STATE_CHANGE,
//                                        DbgkDebugObjectType,
//                                        PreviousMode,
//                                        (PVOID*)&DebugObject,
//                                        NULL);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Clear process and thread */
//     Process = NULL;
//     Thread = NULL;
//
//     /* Wait on the debug object given to us */
//     while (TRUE)
//     {
//         Status = KeWaitForSingleObject(&DebugObject->EventsPresent,
//                                        Executive,
//                                        PreviousMode,
//                                        Alertable,
//                                        Timeout);
//         if (!NT_SUCCESS(Status) ||
//             (Status == STATUS_TIMEOUT) ||
//             (Status == STATUS_ALERTED) ||
//             (Status == STATUS_USER_APC))
//         {
//             /* Break out the wait */
//             break;
//         }
//
//         /* Lock the object */
//         GotEvent = FALSE;
//         ExAcquireFastMutex(&DebugObject->Mutex);
//
//         /* Check if a debugger is connected */
//         if (DebugObject->DebuggerInactive)
//         {
//             /* Not connected */
//             Status = STATUS_DEBUGGER_INACTIVE;
//         }
//         else
//         {
//             /* Loop the events */
//             ListHead = &DebugObject->EventList;
//             NextEntry =  ListHead->Flink;
//             while (ListHead != NextEntry)
//             {
//                 /* Get the debug event */
//                 DebugEvent = CONTAINING_RECORD(NextEntry,
//                                                DEBUG_EVENT,
//                                                EventList);
//                 DBGKTRACE(DBGK_PROCESS_DEBUG, "DebugEvent: %p Flags: %lx\n",
//                           DebugEvent, DebugEvent->Flags);
//
//                 /* Check flags */
//                 if (!(DebugEvent->Flags & (DEBUG_EVENT_INACTIVE | DEBUG_EVENT_READ)))
//                 {
//                     /* We got an event */
//                     GotEvent = TRUE;
//
//                     /* Loop the list internally */
//                     NextEntry2 = DebugObject->EventList.Flink;
//                     while (NextEntry2 != NextEntry)
//                     {
//                         /* Get the debug event */
//                         DebugEvent2 = CONTAINING_RECORD(NextEntry2,
//                                                         DEBUG_EVENT,
//                                                         EventList);
//
//                         /* Try to match process IDs */
//                         if (DebugEvent2->ClientId.UniqueProcess ==
//                             DebugEvent->ClientId.UniqueProcess)
//                         {
//                             /* Found it, break out */
//                             DebugEvent->Flags |= DEBUG_EVENT_INACTIVE;
//                             DebugEvent->BackoutThread = NULL;
//                             GotEvent = FALSE;
//                             break;
//                         }
//
//                         /* Move to the next entry */
//                         NextEntry2 = NextEntry2->Flink;
//                     }
//
//                     /* Check if we still have a valid event */
//                     if (GotEvent) break;
//                 }
//
//                 /* Move to the next entry */
//                 NextEntry = NextEntry->Flink;
//             }
//
//             /* Check if we have an event */
//             if (GotEvent)
//             {
//                 /* Save and reference the process and thread */
//                 Process = DebugEvent->Process;
//                 Thread = DebugEvent->Thread;
//                 ObReferenceObject(Process);
//                 ObReferenceObject(Thread);
//
//                 /* Convert to user-mode structure */
//                 DbgkpConvertKernelToUserStateChange(&WaitStateChange,
//                                                     DebugEvent);
//
//                 /* Set flag */
//                 DebugEvent->Flags |= DEBUG_EVENT_READ;
//             }
//             else
//             {
//                 /* Unsignal the event */
//                 KeClearEvent(&DebugObject->EventsPresent);
//             }
//
//             /* Set success */
//             Status = STATUS_SUCCESS;
//         }
//
//         /* Release the mutex */
//         ExReleaseFastMutex(&DebugObject->Mutex);
//         if (!NT_SUCCESS(Status)) break;
//
//         /* Check if we got an event */
//         if (!GotEvent)
//         {
//             /* Check if we can wait again */
//             if (LocalTimeOut.QuadPart < 0)
//             {
//                 /* Query the new time */
//                 KeQuerySystemTime(&NewTime);
//
//                 /* Substract times */
//                 LocalTimeOut.QuadPart += (NewTime.QuadPart - StartTime.QuadPart);
//                 StartTime = NewTime;
//
//                 /* Check if we've timed out */
//                 if (LocalTimeOut.QuadPart >= 0)
//                 {
//                     /* We have, break out of the loop */
//                     Status = STATUS_TIMEOUT;
//                     break;
//                 }
//             }
//         }
//         else
//         {
//             /* Open the handles and dereference the objects */
//             DbgkpOpenHandles(&WaitStateChange, Process, Thread);
//             ObDereferenceObject(Process);
//             ObDereferenceObject(Thread);
//             break;
//         }
//     }
//
//     /* We're done, dereference the object */
//     ObDereferenceObject(DebugObject);
//
//     /* Protect write with SEH */
//     _SEH2_TRY
//     {
//         /* Return our wait state change structure */
//         *StateChange = WaitStateChange;
//     }
//     _SEH2_EXCEPT(ExSystemExceptionFilter())
//     {
//         /* Get SEH Exception code */
//         Status = _SEH2_GetExceptionCode();
//     }
//     _SEH2_END;
//
//     /* Return status */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
