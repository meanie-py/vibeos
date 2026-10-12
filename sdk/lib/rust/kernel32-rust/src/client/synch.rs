use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;
use crate::k32_externs::{
    BaseFormatObjectAttributes, RtlZeroMemory, RtlCopyMemory,
};
extern "C" {
    // cdecl in C (handle.c: `HANDLE TranslateStdHandle(IN HANDLE)`, no WINAPI).
    // Declaring it extern "system" (stdcall) here mismatched the cdecl callee:
    // on i386 release (no frame ptr) the per-iteration stdcall/cdecl stack-cleanup
    // mismatch drifted ESP inside WaitForMultipleObjectsEx's translate loop,
    // corrupting the ESP-relative handle buffer → garbage handles → STATUS_INVALID_HANDLE.
    fn TranslateStdHandle(hHandle: HANDLE) -> HANDLE;
}
extern "system" {
    fn VerifyConsoleIoHandle(Handle: HANDLE) -> BOOL;
    fn GetConsoleInputWaitHandle() -> HANDLE;
    fn BaseFormatTimeOut(Timeout: PLARGE_INTEGER, dwMilliseconds: DWORD) -> PLARGE_INTEGER;
    fn BasepGetNamedObjectDirectory() -> HANDLE;
}
#[inline]
unsafe fn IsConsoleHandle(h: HANDLE) -> BOOL { ((h as usize & 0x1000_0003) == 0x3) as BOOL }

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _LIST_ENTRY {
    pub Flink: *mut _LIST_ENTRY,
    pub Blink: *mut _LIST_ENTRY,
}

pub type LIST_ENTRY = _LIST_ENTRY;
pub type PLIST_ENTRY = *mut _LIST_ENTRY;

#[repr(C)]
pub struct _RTL_CRITICAL_SECTION {
    pub DebugInfo: *mut _RTL_CRITICAL_SECTION_DEBUG,
    pub LockCount: i32,
    pub RecursionCount: i32,
    pub OwningThread: HANDLE,
    pub LockSemaphore: HANDLE,
    pub SpinCount: usize,
}

pub type RTL_CRITICAL_SECTION = _RTL_CRITICAL_SECTION;
pub type PRTL_CRITICAL_SECTION = *mut _RTL_CRITICAL_SECTION;

#[repr(C)]
pub struct _RTL_CRITICAL_SECTION_DEBUG {
    pub Type: u16,
    pub CreatorBackTraceIndex: u16,
    pub CriticalSection: *mut _RTL_CRITICAL_SECTION,
    pub ProcessLocksList: LIST_ENTRY,
    pub EntryCount: u32,
    pub ContentionCount: u32,
    pub Flags: u32,
    pub CreatorBackTraceIndexHigh: u16,
    pub SpareWORD: u16,
}

pub type RTL_CRITICAL_SECTION_DEBUG = _RTL_CRITICAL_SECTION_DEBUG;
pub type PRTL_CRITICAL_SECTION_DEBUG = *mut _RTL_CRITICAL_SECTION_DEBUG;

pub const NDEBUG: i32 = 1;

pub type LONG = i32;
pub type PLONG = *mut i32;
pub type LPLONG = *mut i32;
pub type BOOL = i32;
pub type DWORD = u32;
pub type LPDWORD = *mut u32;
pub type HANDLE = *mut core::ffi::c_void;
pub type PHANDLE = *mut HANDLE;
pub type LPVOID = *mut core::ffi::c_void;
pub type LPCVOID = *const core::ffi::c_void;
pub type ULONG_PTR = usize;
pub type SIZE_T = usize;
pub type PSIZE_T = *mut usize;
pub type WORD = u16;
pub type UCHAR = u8;
pub type PUCHAR = *mut u8;
pub type PCHAR = *mut i8;
pub type LPCSTR = *const i8;
pub type LPCWSTR = *const u16;
pub type LPWSTR = *mut u16;
pub type LPSTR = *mut i8;
pub type WCHAR = u16;
pub type PWCHAR = *mut u16;
pub type HRESULT = i32;
pub type NTSTATUS = i32;
pub type BOOLEAN = u8;
pub type PBOOLEAN = *mut u8;
pub type VOID = core::ffi::c_void;
pub type PVOID = *mut core::ffi::c_void;
pub type ULONG = u32;
pub type PULONG = *mut u32;
pub type USHORT = u16;
pub type PUSHORT = *mut u16;
pub type UINT = u32;
pub type INT = i32;
pub type PINT = *mut i32;
pub type LPCRITICAL_SECTION = *const _RTL_CRITICAL_SECTION;
pub type LPSECURITY_ATTRIBUTES = *mut _SECURITY_ATTRIBUTES;

#[repr(C)]
pub struct _SECURITY_ATTRIBUTES {
    pub nLength: DWORD,
    pub lpSecurityDescriptor: LPVOID,
    pub bInheritHandle: BOOL,
}

pub type SECURITY_ATTRIBUTES = _SECURITY_ATTRIBUTES;
pub type PSECURITY_ATTRIBUTES = *mut _SECURITY_ATTRIBUTES;

pub const INVALID_HANDLE_VALUE: HANDLE = !0 as HANDLE;
pub const NULL: *mut core::ffi::c_void = core::ptr::null_mut();

pub const FALSE: BOOL = 0;
pub const TRUE: BOOL = 1;

pub const STATUS_WAIT_0: NTSTATUS = 0x00000000;
pub const STATUS_ABANDONED_WAIT_0: NTSTATUS = 0x00000080;
pub const STATUS_USER_APC: NTSTATUS = 0x000000C0;
pub const STATUS_TIMEOUT: NTSTATUS = 0x00000102;
pub const STATUS_PENDING: NTSTATUS = 0x00000103;
pub const STATUS_ALERTED: NTSTATUS = 0x00000101;
pub const STATUS_SUCCESS: NTSTATUS = 0x00000000;

pub const WAIT_OBJECT_0: DWORD = 0;
pub const WAIT_ABANDONED_0: DWORD = 0x00000080;
pub const WAIT_IO_COMPLETION: DWORD = 0x000000C0;
pub const WAIT_TIMEOUT: DWORD = 0x00000102;
pub const WAIT_FAILED: DWORD = 0xFFFFFFFF;

pub const INFINITE: DWORD = 0xFFFFFFFF;

pub const EVENT_ALL_ACCESS: DWORD = 0x001F0003;
pub const MUTEX_ALL_ACCESS: DWORD = 0x001F0001;
pub const SEMAPHORE_ALL_ACCESS: DWORD = 0x001F0003;
pub const TIMER_ALL_ACCESS: DWORD = 0x001F0003;

pub const ERROR_INVALID_HANDLE: DWORD = 6;
pub const ERROR_NOT_ENOUGH_MEMORY: DWORD = 8;
pub const ERROR_ALREADY_EXISTS: DWORD = 183;
pub const ERROR_SUCCESS: DWORD = 0;

pub type WAITORTIMERCALLBACK = Option<unsafe extern "system" fn(PVOID, BOOLEAN)>;
pub type PTIMERAPCROUTINE = Option<unsafe extern "system" fn(PVOID, LARGE_INTEGER, LARGE_INTEGER)>;
pub type PTIMER_APC_ROUTINE = PTIMERAPCROUTINE;

#[no_mangle]
pub unsafe extern "system" fn WaitForSingleObject(
    hHandle: HANDLE,
    dwMilliseconds: DWORD,
) -> DWORD {
    return WaitForSingleObjectEx(hHandle, dwMilliseconds, 0);
}

#[no_mangle]
pub unsafe extern "system" fn WaitForSingleObjectEx(
    hHandle: HANDLE,
    dwMilliseconds: DWORD,
    bAlertable: BOOL,
) -> DWORD {
    let mut TimePtr: PLARGE_INTEGER = core::ptr::null_mut();
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME = core::mem::zeroed();

    /* APCs must execute with the default activation context */
    if bAlertable != 0 {
        /* Setup the frame */
        RtlZeroMemory(&mut ActCtx as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as _);
        ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as _;
        ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER as _;
        RtlActivateActivationContextUnsafeFast(&mut ActCtx, core::ptr::null_mut());
    }

    /* Get real handle */
    let mut hHandle = TranslateStdHandle(hHandle);

    /* Check for console handle */
    if (IsConsoleHandle(hHandle) != 0) && (VerifyConsoleIoHandle(hHandle) != 0) {
        /* Get the real wait handle */
        hHandle = GetConsoleInputWaitHandle();
    }

    /* Convert the timeout */
    TimePtr = BaseFormatTimeOut(&mut Time, dwMilliseconds);

    /* Start wait loop */
    loop {
        /* Do the wait */
        Status = NtWaitForSingleObject(hHandle, bAlertable as BOOLEAN, TimePtr);
        if !NT_SUCCESS(Status) {
            /* The wait failed */
            BaseSetLastNTError(Status);
            Status = WAIT_FAILED as NTSTATUS;
        }
        if !((Status == STATUS_ALERTED) && (bAlertable != 0)) {
            break;
        }
    }

    /* Cleanup the activation context */
    if bAlertable != 0 {
        RtlDeactivateActivationContextUnsafeFast(&mut ActCtx);
    }

    /* Return wait status */
    return Status as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn WaitForMultipleObjects(
    nCount: DWORD,
    lpHandles: *const HANDLE,
    bWaitAll: BOOL,
    dwMilliseconds: DWORD,
) -> DWORD {
    return WaitForMultipleObjectsEx(nCount, lpHandles, bWaitAll, dwMilliseconds, FALSE);
}

#[no_mangle]
pub unsafe extern "system" fn WaitForMultipleObjectsEx(
    nCount: DWORD,
    lpHandles: *const HANDLE,
    bWaitAll: BOOL,
    dwMilliseconds: DWORD,
    bAlertable: BOOL,
) -> DWORD {
    let mut TimePtr: PLARGE_INTEGER = core::ptr::null_mut();
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut HandleBuffer: PHANDLE;
    let mut Handle: [HANDLE; 8] = [core::ptr::null_mut(); 8];
    let mut i: DWORD = 0;
    let mut Status: NTSTATUS = 0;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME = core::mem::zeroed();

    if bAlertable != 0 {
        RtlZeroMemory(
            &mut ActCtx as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as _,
        );
        ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as _;
        ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER as _;
        RtlActivateActivationContextUnsafeFast(&mut ActCtx, core::ptr::null_mut());
    }

    if nCount > 8 {
        HandleBuffer = RtlAllocateHeap(
            RtlGetProcessHeap(),
            0,
            (nCount as SIZE_T).wrapping_mul(core::mem::size_of::<HANDLE>() as SIZE_T) as _,
        ) as PHANDLE;
        if HandleBuffer.is_null() {
            SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
            if bAlertable != 0 {
                RtlDeactivateActivationContextUnsafeFast(&mut ActCtx);
            }
            return WAIT_FAILED;
        }
    } else {
        HandleBuffer = &mut Handle as *mut HANDLE as PHANDLE;
    }

    RtlCopyMemory(
        HandleBuffer as *mut core::ffi::c_void,
        lpHandles as *const core::ffi::c_void,
        (nCount as SIZE_T).wrapping_mul(core::mem::size_of::<HANDLE>() as SIZE_T) as _,
    );

    i = 0;
    while i < nCount {
        *HandleBuffer.offset(i as isize) = TranslateStdHandle(*HandleBuffer.offset(i as isize));
        if IsConsoleHandle(*HandleBuffer.offset(i as isize)) != 0
            && VerifyConsoleIoHandle(*HandleBuffer.offset(i as isize)) != 0
        {
            *HandleBuffer.offset(i as isize) = GetConsoleInputWaitHandle();
        }
        i = i.wrapping_add(1);
    }

    TimePtr = BaseFormatTimeOut(&mut Time, dwMilliseconds);

    loop {
        Status = NtWaitForMultipleObjects(
            nCount,
            HandleBuffer,
            if bWaitAll != 0 { WaitAll } else { WaitAny },
            bAlertable as BOOLEAN,
            TimePtr,
        );
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            Status = WAIT_FAILED as NTSTATUS;
        }
        if !(Status == STATUS_ALERTED && bAlertable != 0) {
            break;
        }
    }

    if HandleBuffer != &mut Handle as *mut HANDLE as PHANDLE {
        RtlFreeHeap(
            RtlGetProcessHeap(),
            0,
            HandleBuffer as *mut core::ffi::c_void,
        );
    }

    if bAlertable != 0 {
        RtlDeactivateActivationContextUnsafeFast(&mut ActCtx);
    }

    return Status as DWORD;
}

#[no_mangle]
pub unsafe extern "system" fn SignalObjectAndWait(
    hObjectToSignal: HANDLE,
    mut hObjectToWaitOn: HANDLE,
    dwMilliseconds: DWORD,
    bAlertable: BOOL,
) -> DWORD {
    let mut TimePtr: PLARGE_INTEGER = core::ptr::null_mut();
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME = core::mem::zeroed();

    /* APCs must execute with the default activation context */
    if bAlertable != 0 {
        /* Setup the frame */
        RtlZeroMemory(&mut ActCtx as *mut _ as *mut core::ffi::c_void, core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as _);
        ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as _;
        ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER as _;
        RtlActivateActivationContextUnsafeFast(&mut ActCtx, core::ptr::null_mut());
    }

    /* Get real handle */
    hObjectToWaitOn = TranslateStdHandle(hObjectToWaitOn);

    /* Check for console handle */
    if (IsConsoleHandle(hObjectToWaitOn) != 0) &&
        (VerifyConsoleIoHandle(hObjectToWaitOn) != 0)
    {
        /* Get the real wait handle */
        hObjectToWaitOn = GetConsoleInputWaitHandle();
    }

    /* Convert the timeout */
    TimePtr = BaseFormatTimeOut(&mut Time, dwMilliseconds);

    /* Start wait loop */
    loop {
        /* Do the wait */
        Status = NtSignalAndWaitForSingleObject(
            hObjectToSignal,
            hObjectToWaitOn,
            bAlertable as BOOLEAN,
            TimePtr,
        );
        if !NT_SUCCESS(Status) {
            /* The wait failed */
            BaseSetLastNTError(Status);
            Status = WAIT_FAILED as NTSTATUS;
        }
        if !((Status == STATUS_ALERTED) && (bAlertable != 0)) {
            break;
        }
    }

    /* Cleanup the activation context */
    if bAlertable != 0 {
        RtlDeactivateActivationContextUnsafeFast(&mut ActCtx);
    }

    /* Return wait status */
    return Status as DWORD;
}
#[no_mangle]
pub unsafe extern "system" fn CreateWaitableTimerW(
    lpTimerAttributes: LPSECURITY_ATTRIBUTES,
    bManualReset: BOOL,
    lpTimerName: LPCWSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = &raw mut LocalAttributes;
    if !lpTimerName.is_null() {
        RtlInitUnicodeString(&raw mut ObjectName, lpTimerName as PCWSTR);
    }
    ObjectAttributes = BaseFormatObjectAttributes(
        &raw mut LocalAttributes,
        lpTimerAttributes as ntdll_sys::nt::LPSECURITY_ATTRIBUTES,
        if !lpTimerName.is_null() { &raw mut ObjectName } else { core::ptr::null_mut() },
    );
    Status = NtCreateTimer(
        &raw mut Handle,
        TIMER_ALL_ACCESS as ACCESS_MASK,
        ObjectAttributes,
        if bManualReset != 0 { NotificationTimer } else { SynchronizationTimer } as TIMER_TYPE,
    );
    if NT_SUCCESS(Status) {
        if Status == STATUS_OBJECT_NAME_EXISTS {
            SetLastError(ERROR_ALREADY_EXISTS as DWORD);
        } else {
            SetLastError(ERROR_SUCCESS as DWORD);
        }
        return Handle;
    }
    BaseSetLastNTError(Status);
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn CreateWaitableTimerA(
    lpTimerAttributes: LPSECURITY_ATTRIBUTES,
    bManualReset: BOOL,
    lpTimerName: LPCSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let UnicodeCache: PUNICODE_STRING;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    if lpTimerName.is_null() {
        return CreateWaitableTimerW(lpTimerAttributes, bManualReset, core::ptr::null());
    }
    UnicodeCache = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    RtlInitAnsiString(&raw mut AnsiName, lpTimerName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return CreateWaitableTimerW(lpTimerAttributes, bManualReset, (*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn OpenWaitableTimerW(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpTimerName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = &raw mut LocalAttributes;

    if lpTimerName.is_null() {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    RtlInitUnicodeString(&raw mut ObjectName, lpTimerName as PCWSTR);

    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = BasepGetNamedObjectDirectory();
    (*ObjectAttributes).Attributes = (if bInheritHandle != 0 {
        0x2 as ::core::ffi::c_long
    } else {
        0 as ::core::ffi::c_long
    }) as ULONG;
    (*ObjectAttributes).ObjectName = &raw mut ObjectName as PUNICODE_STRING;
    (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut() as PVOID;
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut() as PVOID;

    Status = NtOpenTimer(
        &raw mut Handle,
        dwDesiredAccess as ACCESS_MASK,
        ObjectAttributes,
    );

    if !(Status >= 0 as ::core::ffi::c_long) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    Handle
}
#[no_mangle]
pub unsafe extern "system" fn OpenWaitableTimerA(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpTimerName: LPCSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut UnicodeCache: PUNICODE_STRING = core::ptr::null_mut();
    let mut AnsiName: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut::<CHAR>(),
    };
    if lpTimerName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    UnicodeCache = &raw mut (*NtCurrentTeb())
        .StaticUnicodeString as PUNICODE_STRING;
    RtlInitAnsiString(&raw mut AnsiName, lpTimerName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, FALSE as BOOLEAN);
    if Status >= 0 as ::core::ffi::c_long {
        return OpenWaitableTimerW(
            dwDesiredAccess,
            bInheritHandle,
            (*UnicodeCache).Buffer as LPCWSTR,
        );
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    return core::ptr::null_mut::<::core::ffi::c_void>();
}

#[no_mangle]
pub unsafe extern "system" fn SetWaitableTimer(
    hTimer: HANDLE,
    pDueTime: *const LARGE_INTEGER,
    lPeriod: LONG,
    pfnCompletionRoutine: PTIMERAPCROUTINE,
    lpArgToCompletionRoutine: LPVOID,
    fResume: BOOL,
) -> BOOL {
    let mut Status: NTSTATUS;

    Status = NtSetTimer(
        hTimer,
        pDueTime as PLARGE_INTEGER,
        core::mem::transmute::<PTIMERAPCROUTINE, ntdll_sys::nt::PTIMER_APC_ROUTINE>(pfnCompletionRoutine),
        lpArgToCompletionRoutine,
        fResume as BOOLEAN,
        lPeriod,
        core::ptr::null_mut(),
    );
    if NT_SUCCESS(Status) {
        return 1;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CancelWaitableTimer(hTimer: HANDLE) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtCancelTimer(hTimer, core::ptr::null_mut());
    if NT_SUCCESS(Status) {
        return 1;
    }
    BaseSetLastNTError(Status);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn CreateSemaphoreA(
    lpSemaphoreAttributes: LPSECURITY_ATTRIBUTES,
    lInitialCount: LONG,
    lMaximumCount: LONG,
    lpName: LPCSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let UnicodeCache: PUNICODE_STRING;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    if lpName.is_null() {
        return CreateSemaphoreW(lpSemaphoreAttributes, lInitialCount, lMaximumCount, core::ptr::null());
    }
    UnicodeCache = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return CreateSemaphoreW(lpSemaphoreAttributes, lInitialCount, lMaximumCount, (*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn CreateSemaphoreW(
    lpSemaphoreAttributes: LPSECURITY_ATTRIBUTES,
    lInitialCount: LONG,
    lMaximumCount: LONG,
    lpName: LPCWSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = &raw mut LocalAttributes;
    if !lpName.is_null() {
        RtlInitUnicodeString(&raw mut ObjectName, lpName as PCWSTR);
    }
    ObjectAttributes = BaseFormatObjectAttributes(
        &raw mut LocalAttributes,
        lpSemaphoreAttributes as ntdll_sys::nt::LPSECURITY_ATTRIBUTES,
        if !lpName.is_null() { &raw mut ObjectName } else { core::ptr::null_mut() },
    );
    Status = NtCreateSemaphore(
        &raw mut Handle,
        SEMAPHORE_ALL_ACCESS as ACCESS_MASK,
        ObjectAttributes,
        lInitialCount,
        lMaximumCount,
    );
    if NT_SUCCESS(Status) {
        if Status == STATUS_OBJECT_NAME_EXISTS {
            SetLastError(ERROR_ALREADY_EXISTS as DWORD);
        } else {
            SetLastError(ERROR_SUCCESS as DWORD);
        }
        return Handle;
    }
    BaseSetLastNTError(Status);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateSemaphoreExW(
    lpSemaphoreAttributes: LPSECURITY_ATTRIBUTES,
    lInitialCount: LONG,
    lMaximumCount: LONG,
    lpName: LPCWSTR,
    dwFlags: DWORD,
    dwDesiredAccess: DWORD,
) -> HANDLE {
    let _ = dwFlags;
    let Status: NTSTATUS;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = &raw mut LocalAttributes;
    if !lpName.is_null() {
        RtlInitUnicodeString(&raw mut ObjectName, lpName as PCWSTR);
    }
    ObjectAttributes = BaseFormatObjectAttributes(
        &raw mut LocalAttributes,
        lpSemaphoreAttributes as ntdll_sys::nt::LPSECURITY_ATTRIBUTES,
        if !lpName.is_null() { &raw mut ObjectName } else { core::ptr::null_mut() },
    );
    Status = NtCreateSemaphore(
        &raw mut Handle,
        dwDesiredAccess as ACCESS_MASK,
        ObjectAttributes,
        lInitialCount,
        lMaximumCount,
    );
    if NT_SUCCESS(Status) {
        if Status == STATUS_OBJECT_NAME_EXISTS {
            SetLastError(ERROR_ALREADY_EXISTS as DWORD);
        } else {
            SetLastError(ERROR_SUCCESS as DWORD);
        }
        return Handle;
    }
    BaseSetLastNTError(Status);
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateSemaphoreExA(
    lpSemaphoreAttributes: LPSECURITY_ATTRIBUTES,
    lInitialCount: LONG,
    lMaximumCount: LONG,
    lpName: LPCSTR,
    dwFlags: DWORD,
    dwDesiredAccess: DWORD,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut UnicodeCache: PUNICODE_STRING = core::ptr::null_mut();
    let mut AnsiName: ANSI_STRING = ANSI_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    if lpName.is_null() {
        return CreateSemaphoreExW(
            lpSemaphoreAttributes,
            lInitialCount,
            lMaximumCount,
            core::ptr::null(),
            dwFlags,
            dwDesiredAccess,
        );
    }

    UnicodeCache = &mut (*(NtCurrentTeb())).StaticUnicodeString as PUNICODE_STRING;
    RtlInitAnsiString(&mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return CreateSemaphoreExW(
            lpSemaphoreAttributes,
            lInitialCount,
            lMaximumCount,
            (*UnicodeCache).Buffer as LPCWSTR,
            dwFlags,
            dwDesiredAccess,
        );
    }

    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    return core::ptr::null_mut();
}

#[no_mangle]
pub unsafe extern "system" fn OpenSemaphoreA(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let UnicodeCache: PUNICODE_STRING;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    if lpName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    UnicodeCache = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return OpenSemaphoreW(dwDesiredAccess, bInheritHandle, (*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn OpenSemaphoreW(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = &raw mut LocalAttributes;

    if lpName.is_null() {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    RtlInitUnicodeString(&raw mut ObjectName, lpName as PCWSTR);

    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = BasepGetNamedObjectDirectory();
    (*ObjectAttributes).Attributes = (if bInheritHandle != 0 {
        0x2 as ::core::ffi::c_long
    } else {
        0 as ::core::ffi::c_long
    }) as ULONG;
    (*ObjectAttributes).ObjectName = &raw mut ObjectName as PUNICODE_STRING;
    (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut() as PVOID;
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut() as PVOID;

    Status = NtOpenSemaphore(
        &raw mut Handle,
        dwDesiredAccess as ACCESS_MASK,
        ObjectAttributes,
    );

    if !(Status >= 0 as ::core::ffi::c_long) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    return Handle;
}

#[no_mangle]
pub unsafe extern "system" fn ReleaseSemaphore(
    hSemaphore: HANDLE,
    lReleaseCount: LONG,
    lpPreviousCount: LPLONG,
) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtReleaseSemaphore(hSemaphore, lReleaseCount, lpPreviousCount as PLONG);
    if NT_SUCCESS(Status) {
        return 1; // TRUE
    }
    BaseSetLastNTError(Status);
    return 0; // FALSE
}
#[no_mangle]
pub unsafe extern "system" fn CreateMutexA(
    lpMutexAttributes: LPSECURITY_ATTRIBUTES,
    bInitialOwner: BOOL,
    lpName: LPCSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let UnicodeCache: PUNICODE_STRING;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    if lpName.is_null() {
        return CreateMutexW(lpMutexAttributes, bInitialOwner, core::ptr::null());
    }
    UnicodeCache = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return CreateMutexW(lpMutexAttributes, bInitialOwner, (*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateMutexW(
    lpMutexAttributes: LPSECURITY_ATTRIBUTES,
    bInitialOwner: BOOL,
    lpName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = &raw mut LocalAttributes;
    if !lpName.is_null() {
        RtlInitUnicodeString(&raw mut ObjectName, lpName as PCWSTR);
    }
    ObjectAttributes = BaseFormatObjectAttributes(
        &raw mut LocalAttributes,
        lpMutexAttributes as ntdll_sys::nt::LPSECURITY_ATTRIBUTES,
        if !lpName.is_null() {
            &raw mut ObjectName
        } else {
            core::ptr::null_mut()
        },
    );
    Status = NtCreateMutant(
        &raw mut Handle,
        0x1f0001 as ACCESS_MASK,
        ObjectAttributes,
        bInitialOwner as BOOLEAN,
    );
    if Status >= 0 as ::core::ffi::c_long {
        if Status == STATUS_OBJECT_NAME_EXISTS {
            SetLastError(ERROR_ALREADY_EXISTS as DWORD);
        } else {
            SetLastError(ERROR_SUCCESS as DWORD);
        }
        return Handle;
    }
    BaseSetLastNTError(Status);
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn OpenMutexA(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let UnicodeCache: PUNICODE_STRING;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    if lpName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    UnicodeCache = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return OpenMutexW(dwDesiredAccess, bInheritHandle, (*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn OpenMutexW(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = core::ptr::addr_of_mut!(LocalAttributes);

    if lpName.is_null() {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }

    RtlInitUnicodeString(core::ptr::addr_of_mut!(ObjectName), lpName as PCWSTR);

    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = BasepGetNamedObjectDirectory();
    (*ObjectAttributes).Attributes = (if bInheritHandle != 0 {
        0x2 as ::core::ffi::c_long
    } else {
        0 as ::core::ffi::c_long
    }) as ULONG;
    (*ObjectAttributes).ObjectName = core::ptr::addr_of_mut!(ObjectName) as PUNICODE_STRING;
    (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();

    Status = NtOpenMutant(
        core::ptr::addr_of_mut!(Handle),
        dwDesiredAccess as ACCESS_MASK,
        ObjectAttributes,
    );

    if !(Status >= 0 as ::core::ffi::c_long) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    Handle
}
#[no_mangle]
pub unsafe extern "system" fn ReleaseMutex(hMutex: HANDLE) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtReleaseMutant(hMutex, core::ptr::null_mut());
    if NT_SUCCESS(Status) {
        return 1; // TRUE
    }
    BaseSetLastNTError(Status);
    return 0; // FALSE
}
#[no_mangle]
pub unsafe extern "system" fn CreateEventA(
    lpEventAttributes: LPSECURITY_ATTRIBUTES,
    bManualReset: BOOL,
    bInitialState: BOOL,
    lpName: LPCSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let UnicodeCache: PUNICODE_STRING;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    if lpName.is_null() {
        return CreateEventW(lpEventAttributes, bManualReset, bInitialState, core::ptr::null());
    }
    UnicodeCache = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return CreateEventW(lpEventAttributes, bManualReset, bInitialState, (*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn CreateEventW(
    lpEventAttributes: LPSECURITY_ATTRIBUTES,
    bManualReset: BOOL,
    bInitialState: BOOL,
    lpName: LPCWSTR,
) -> HANDLE {
    let mut Status: NTSTATUS = 0;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = _OBJECT_ATTRIBUTES {
        Length: 0,
        RootDirectory: core::ptr::null_mut(),
        ObjectName: core::ptr::null_mut(),
        Attributes: 0,
        SecurityDescriptor: core::ptr::null_mut(),
        SecurityQualityOfService: core::ptr::null_mut(),
    };
    let mut ObjectAttributes: POBJECT_ATTRIBUTES = &raw mut LocalAttributes;
    if !lpName.is_null() {
        RtlInitUnicodeString(&raw mut ObjectName, lpName as PCWSTR);
    }
    ObjectAttributes = BaseFormatObjectAttributes(
        &raw mut LocalAttributes,
        lpEventAttributes as ntdll_sys::nt::LPSECURITY_ATTRIBUTES,
        if !lpName.is_null() {
            &raw mut ObjectName
        } else {
            core::ptr::null_mut()
        },
    );
    Status = NtCreateEvent(
        &raw mut Handle,
        0x1f0003 as ACCESS_MASK,
        ObjectAttributes,
        (if bManualReset != 0 {
            _EVENT_TYPE_NotificationEvent as ::core::ffi::c_int
        } else {
            _EVENT_TYPE_SynchronizationEvent as ::core::ffi::c_int
        }) as EVENT_TYPE,
        bInitialState as BOOLEAN,
    );
    if Status >= 0 as ::core::ffi::c_long {
        if Status == STATUS_OBJECT_NAME_EXISTS {
            SetLastError(ERROR_ALREADY_EXISTS as DWORD);
        } else {
            SetLastError(ERROR_SUCCESS as DWORD);
        }
        return Handle;
    }
    BaseSetLastNTError(Status);
    return core::ptr::null_mut();
}
#[no_mangle]
pub unsafe extern "system" fn OpenEventA(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let UnicodeCache: PUNICODE_STRING;
    let mut AnsiName: ANSI_STRING = core::mem::zeroed();
    if lpName.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return core::ptr::null_mut();
    }
    UnicodeCache = &raw mut (*NtCurrentTeb()).StaticUnicodeString;
    RtlInitAnsiString(&raw mut AnsiName, lpName as PCSZ);
    Status = RtlAnsiStringToUnicodeString(UnicodeCache, &raw mut AnsiName, 0);
    if NT_SUCCESS(Status) {
        return OpenEventW(dwDesiredAccess, bInheritHandle, (*UnicodeCache).Buffer as LPCWSTR);
    }
    if Status == STATUS_BUFFER_OVERFLOW {
        SetLastError(ERROR_FILENAME_EXCED_RANGE as DWORD);
    } else {
        BaseSetLastNTError(Status);
    }
    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn OpenEventW(
    dwDesiredAccess: DWORD,
    bInheritHandle: BOOL,
    lpName: LPCWSTR,
) -> HANDLE {
    let Status: NTSTATUS;
    let mut Handle: HANDLE = core::ptr::null_mut();
    let mut ObjectName: _UNICODE_STRING = core::mem::zeroed();
    let mut LocalAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let ObjectAttributes: POBJECT_ATTRIBUTES = &raw mut LocalAttributes;
    if lpName.is_null() {
        BaseSetLastNTError(STATUS_INVALID_PARAMETER);
        return core::ptr::null_mut();
    }
    RtlInitUnicodeString(&raw mut ObjectName, lpName as PCWSTR);
    (*ObjectAttributes).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*ObjectAttributes).RootDirectory = BasepGetNamedObjectDirectory();
    (*ObjectAttributes).Attributes = if bInheritHandle != 0 { OBJ_INHERIT } else { 0 };
    (*ObjectAttributes).ObjectName = &raw mut ObjectName as PUNICODE_STRING;
    (*ObjectAttributes).SecurityDescriptor = core::ptr::null_mut();
    (*ObjectAttributes).SecurityQualityOfService = core::ptr::null_mut();
    Status = NtOpenEvent(&raw mut Handle, dwDesiredAccess as ACCESS_MASK, ObjectAttributes);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }
    Handle
}

#[no_mangle]
pub unsafe extern "system" fn PulseEvent(hEvent: HANDLE) -> BOOL {
    let mut Status: NTSTATUS;
    Status = NtPulseEvent(hEvent, core::ptr::null_mut());
    if NT_SUCCESS(Status) {
        return 1; // TRUE
    }
    BaseSetLastNTError(Status);
    return 0; // FALSE
}

#[no_mangle]
pub unsafe extern "system" fn ResetEvent(hEvent: HANDLE) -> BOOL {
    let mut Status: NTSTATUS = 0;

    /* Clear the event */
    Status = NtResetEvent(hEvent, core::ptr::null_mut());
    if NT_SUCCESS(Status) {
        return 1; /* TRUE */
    }

    /* If we got here, then we failed */
    BaseSetLastNTError(Status);
    return 0; /* FALSE */
}
#[no_mangle]
pub unsafe extern "system" fn SetEvent(hEvent: HANDLE) -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = NtSetEvent(hEvent, core::ptr::null_mut());
    if NT_SUCCESS(Status) {
        return 1; // TRUE
    }
    BaseSetLastNTError(Status);
    return 0; // FALSE
}
#[no_mangle]
pub unsafe extern "system" fn InitializeCriticalSection(lpCriticalSection: LPCRITICAL_SECTION) {
    let mut Status: NTSTATUS = 0;
    Status = RtlInitializeCriticalSection(lpCriticalSection as *mut ntdll_sys::nt::_RTL_CRITICAL_SECTION);
    if !NT_SUCCESS(Status) {
        RtlRaiseStatus(Status);
    }
}
#[no_mangle]
pub unsafe extern "system" fn InitializeCriticalSectionAndSpinCount(
    lpCriticalSection: LPCRITICAL_SECTION,
    dwSpinCount: DWORD,
) -> BOOL {
    let mut Status: NTSTATUS = 0;
    Status = RtlInitializeCriticalSectionAndSpinCount(
        lpCriticalSection as *mut ntdll_sys::nt::_RTL_CRITICAL_SECTION,
        dwSpinCount,
    );
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn Sleep(
    dwMilliseconds: DWORD,
) {
    SleepEx(dwMilliseconds, 0);
}

#[no_mangle]
pub unsafe extern "system" fn SleepEx(
    dwMilliseconds: DWORD,
    bAlertable: BOOL,
) -> DWORD {
    let mut Time: LARGE_INTEGER = core::mem::zeroed();
    let mut TimePtr: PLARGE_INTEGER = core::ptr::null_mut();
    let mut errCode: NTSTATUS = 0;
    let mut ActCtx: RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME = core::mem::zeroed();

    /* APCs must execute with the default activation context */
    if bAlertable != 0 {
        /* Setup the frame */
        RtlZeroMemory(
            &mut ActCtx as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as _,
        );
        ActCtx.Size = core::mem::size_of::<RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME>() as _;
        ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER as _;
        RtlActivateActivationContextUnsafeFast(&mut ActCtx, core::ptr::null_mut());
    }

    /* Convert the timeout */
    TimePtr = BaseFormatTimeOut(&mut Time, dwMilliseconds);
    if TimePtr.is_null() {
        /* Turn an infinite wait into a really long wait */
        Time.__bindgen_anon_1.LowPart = 0;
        Time.__bindgen_anon_1.HighPart = 0x80000000u32 as i32;
        TimePtr = &mut Time;
    }

    /* Loop the delay while APCs are alerting us */
    loop {
        /* Do the delay */
        errCode = NtDelayExecution(bAlertable as BOOLEAN, TimePtr);
        if !(bAlertable != 0 && errCode == STATUS_ALERTED) {
            break;
        }
    }

    /* Cleanup the activation context */
    if bAlertable != 0 {
        RtlDeactivateActivationContextUnsafeFast(&mut ActCtx);
    }

    /* Return the correct code */
    return if errCode == STATUS_USER_APC {
        WAIT_IO_COMPLETION
    } else {
        0
    };
}
#[no_mangle]
pub unsafe extern "system" fn RegisterWaitForSingleObject(
    phNewWaitObject: PHANDLE,
    hObject: HANDLE,
    Callback: WAITORTIMERCALLBACK,
    Context: PVOID,
    dwMilliseconds: ULONG,
    dwFlags: ULONG,
) -> BOOL {
    let mut Status: NTSTATUS;
    let mut hObject = hObject;

    /* Get real handle */
    hObject = TranslateStdHandle(hObject);

    /* Check for console handle */
    if (hObject as usize & 0x10000003) == 0x3 && VerifyConsoleIoHandle(hObject) != 0 {
        /* Get the real wait handle */
        hObject = GetConsoleInputWaitHandle();
    }

    /* Register the wait now */
    Status = RtlRegisterWait(
        phNewWaitObject,
        hObject,
        core::mem::transmute::<WAITORTIMERCALLBACK, ntdll_sys::nt::WAITORTIMERCALLBACKFUNC>(Callback),
        Context,
        dwMilliseconds,
        dwFlags,
    );
    if !NT_SUCCESS(Status) {
        /* Return failure */
        BaseSetLastNTError(Status);
        return 0i32;
    }

    /* All good */
    1i32
}

#[no_mangle]
pub unsafe extern "system" fn RegisterWaitForSingleObjectEx(
    hObject: HANDLE,
    Callback: WAITORTIMERCALLBACK,
    Context: PVOID,
    dwMilliseconds: ULONG,
    dwFlags: ULONG,
) -> HANDLE {
    let mut Status: NTSTATUS;
    let mut hNewWaitObject: HANDLE = core::ptr::null_mut();

    /* Get real handle */
    let mut hObject = TranslateStdHandle(hObject);

    /* Check for console handle */
    if (hObject as usize & 0x10000003) == 0x3 && VerifyConsoleIoHandle(hObject) != 0 {
        /* Get the real wait handle */
        hObject = GetConsoleInputWaitHandle();
    }

    /* Register the wait */
    Status = RtlRegisterWait(
        &mut hNewWaitObject,
        hObject,
        core::mem::transmute::<WAITORTIMERCALLBACK, ntdll_sys::nt::WAITORTIMERCALLBACKFUNC>(Callback),
        Context,
        dwMilliseconds,
        dwFlags,
    );
    if !NT_SUCCESS(Status) {
        /* Return failure */
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    /* Return the object */
    hNewWaitObject
}
#[no_mangle]
pub unsafe extern "system" fn UnregisterWait(WaitHandle: HANDLE) -> BOOL {
    let mut Status: NTSTATUS = 0;

    /* Check for invalid handle */
    if WaitHandle.is_null() {
        /* Fail */
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    /* Deregister the wait and check status */
    Status = RtlDeregisterWaitEx(WaitHandle, core::ptr::null_mut());
    if !(NT_SUCCESS(Status)) || Status == STATUS_PENDING {
        /* Failure or non-blocking call */
        BaseSetLastNTError(Status);
        return 0;
    }

    /* All good */
    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn UnregisterWaitEx(
    WaitHandle: HANDLE,
    CompletionEvent: HANDLE,
) -> BOOL {
    let Status: NTSTATUS;

    /* Check for invalid handle */
    if WaitHandle.is_null() {
        SetLastError(ERROR_INVALID_HANDLE);
        return 0;
    }

    /* Deregister the wait and check status */
    Status = RtlDeregisterWaitEx(WaitHandle, CompletionEvent);
    if !(NT_SUCCESS(Status))
        || ((CompletionEvent != INVALID_HANDLE_VALUE) && (Status == STATUS_PENDING))
    {
        /* Failure or non-blocking call */
        BaseSetLastNTError(Status);
        return 0;
    }

    /* All good */
    return 1;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Win32 Base API
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            dll/win32/kernel32/client/synch.c
//  * PURPOSE:         Wrappers for the NT Wait Implementation
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// #undef InterlockedIncrement
// #undef InterlockedDecrement
// #undef InterlockedExchange
// #undef InterlockedExchangeAdd
// #undef InterlockedCompareExchange
//
// /* FUNCTIONS *****************************************************************/
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// InterlockedIncrement(IN OUT LONG volatile *lpAddend)
// {
//     return _InterlockedIncrement(lpAddend);
// }
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// InterlockedDecrement(IN OUT LONG volatile *lpAddend)
// {
//     return _InterlockedDecrement(lpAddend);
// }
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// InterlockedExchange(IN OUT LONG volatile *Target,
//                     IN LONG Value)
// {
//     return _InterlockedExchange(Target, Value);
// }
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// InterlockedExchangeAdd(IN OUT LONG volatile *Addend,
//                        IN LONG Value)
// {
//     return _InterlockedExchangeAdd(Addend, Value);
// }
//
// /*
//  * @implemented
//  */
// LONG
// WINAPI
// InterlockedCompareExchange(IN OUT LONG volatile *Destination,
//                            IN LONG Exchange,
//                            IN LONG Comperand)
// {
//     return _InterlockedCompareExchange(Destination, Exchange, Comperand);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// WaitForSingleObject(IN HANDLE hHandle,
//                     IN DWORD dwMilliseconds)
// {
//     /* Call the extended API */
//     return WaitForSingleObjectEx(hHandle, dwMilliseconds, FALSE);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// WaitForSingleObjectEx(IN HANDLE hHandle,
//                       IN DWORD dwMilliseconds,
//                       IN BOOL bAlertable)
// {
//     PLARGE_INTEGER TimePtr;
//     LARGE_INTEGER Time;
//     NTSTATUS Status;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME ActCtx;
//
//     /* APCs must execute with the default activation context */
//     if (bAlertable)
//     {
//         /* Setup the frame */
//         RtlZeroMemory(&ActCtx, sizeof(ActCtx));
//         ActCtx.Size = sizeof(ActCtx);
//         ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
//         RtlActivateActivationContextUnsafeFast(&ActCtx, NULL);
//     }
//
//     /* Get real handle */
//     hHandle = TranslateStdHandle(hHandle);
//
//     /* Check for console handle */
//     if ((IsConsoleHandle(hHandle)) && (VerifyConsoleIoHandle(hHandle)))
//     {
//         /* Get the real wait handle */
//         hHandle = GetConsoleInputWaitHandle();
//     }
//
//     /* Convert the timeout */
//     TimePtr = BaseFormatTimeOut(&Time, dwMilliseconds);
//
//     /* Start wait loop */
//     do
//     {
//         /* Do the wait */
//         Status = NtWaitForSingleObject(hHandle, (BOOLEAN)bAlertable, TimePtr);
//         if (!NT_SUCCESS(Status))
//         {
//             /* The wait failed */
//             BaseSetLastNTError(Status);
//             Status = WAIT_FAILED;
//         }
//     } while ((Status == STATUS_ALERTED) && (bAlertable));
//
//     /* Cleanup the activation context */
//     if (bAlertable) RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//
//     /* Return wait status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// WaitForMultipleObjects(IN DWORD nCount,
//                        IN CONST HANDLE *lpHandles,
//                        IN BOOL bWaitAll,
//                        IN DWORD dwMilliseconds)
// {
//     /* Call the extended API */
//     return WaitForMultipleObjectsEx(nCount,
//                                     lpHandles,
//                                     bWaitAll,
//                                     dwMilliseconds,
//                                     FALSE);
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// WaitForMultipleObjectsEx(IN DWORD nCount,
//                          IN CONST HANDLE *lpHandles,
//                          IN BOOL bWaitAll,
//                          IN DWORD dwMilliseconds,
//                          IN BOOL bAlertable)
// {
//     PLARGE_INTEGER TimePtr;
//     LARGE_INTEGER Time;
//     PHANDLE HandleBuffer;
//     HANDLE Handle[8];
//     DWORD i;
//     NTSTATUS Status;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME ActCtx;
//
//     /* APCs must execute with the default activation context */
//     if (bAlertable)
//     {
//         /* Setup the frame */
//         RtlZeroMemory(&ActCtx, sizeof(ActCtx));
//         ActCtx.Size = sizeof(ActCtx);
//         ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
//         RtlActivateActivationContextUnsafeFast(&ActCtx, NULL);
//     }
//
//     /* Check if we have more handles then we locally optimize */
//     if (nCount > 8)
//     {
//         /* Allocate a buffer for them */
//         HandleBuffer = RtlAllocateHeap(RtlGetProcessHeap(),
//                                        0,
//                                        nCount * sizeof(HANDLE));
//         if (!HandleBuffer)
//         {
//             /* No buffer, fail the wait */
//             SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//             if (bAlertable) RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//             return WAIT_FAILED;
//         }
//     }
//     else
//     {
//         /* Otherwise, use our local buffer */
//         HandleBuffer = Handle;
//     }
//
//     /* Copy the handles into our buffer and loop them all */
//     RtlCopyMemory(HandleBuffer, (LPVOID)lpHandles, nCount * sizeof(HANDLE));
//     for (i = 0; i < nCount; i++)
//     {
//         /* Check what kind of handle this is */
//         HandleBuffer[i] = TranslateStdHandle(HandleBuffer[i]);
//
//         /* Check for console handle */
//         if ((IsConsoleHandle(HandleBuffer[i])) &&
//             (VerifyConsoleIoHandle(HandleBuffer[i])))
//         {
//             /* Get the real wait handle */
//             HandleBuffer[i] = GetConsoleInputWaitHandle();
//         }
//     }
//
//     /* Convert the timeout */
//     TimePtr = BaseFormatTimeOut(&Time, dwMilliseconds);
//
//     /* Start wait loop */
//     do
//     {
//         /* Do the wait */
//         Status = NtWaitForMultipleObjects(nCount,
//                                           HandleBuffer,
//                                           bWaitAll ? WaitAll : WaitAny,
//                                           (BOOLEAN)bAlertable,
//                                           TimePtr);
//         if (!NT_SUCCESS(Status))
//         {
//             /* Wait failed */
//             BaseSetLastNTError(Status);
//             Status = WAIT_FAILED;
//         }
//     } while ((Status == STATUS_ALERTED) && (bAlertable));
//
//     /* Check if we didn't use our local buffer */
//     if (HandleBuffer != Handle)
//     {
//         /* Free the allocated one */
//         RtlFreeHeap(RtlGetProcessHeap(), 0, HandleBuffer);
//     }
//
//     /* Cleanup the activation context */
//     if (bAlertable) RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//
//     /* Return wait status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SignalObjectAndWait(IN HANDLE hObjectToSignal,
//                     IN HANDLE hObjectToWaitOn,
//                     IN DWORD dwMilliseconds,
//                     IN BOOL bAlertable)
// {
//     PLARGE_INTEGER TimePtr;
//     LARGE_INTEGER Time;
//     NTSTATUS Status;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME ActCtx;
//
//     /* APCs must execute with the default activation context */
//     if (bAlertable)
//     {
//         /* Setup the frame */
//         RtlZeroMemory(&ActCtx, sizeof(ActCtx));
//         ActCtx.Size = sizeof(ActCtx);
//         ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
//         RtlActivateActivationContextUnsafeFast(&ActCtx, NULL);
//     }
//
//     /* Get real handle */
//     hObjectToWaitOn = TranslateStdHandle(hObjectToWaitOn);
//
//     /* Check for console handle */
//     if ((IsConsoleHandle(hObjectToWaitOn)) &&
//         (VerifyConsoleIoHandle(hObjectToWaitOn)))
//     {
//         /* Get the real wait handle */
//         hObjectToWaitOn = GetConsoleInputWaitHandle();
//     }
//
//     /* Convert the timeout */
//     TimePtr = BaseFormatTimeOut(&Time, dwMilliseconds);
//
//     /* Start wait loop */
//     do
//     {
//         /* Do the wait */
//         Status = NtSignalAndWaitForSingleObject(hObjectToSignal,
//                                                 hObjectToWaitOn,
//                                                 (BOOLEAN)bAlertable,
//                                                 TimePtr);
//         if (!NT_SUCCESS(Status))
//         {
//             /* The wait failed */
//             BaseSetLastNTError(Status);
//             Status = WAIT_FAILED;
//         }
//     } while ((Status == STATUS_ALERTED) && (bAlertable));
//
//     /* Cleanup the activation context */
//     if (bAlertable) RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//
//     /* Return wait status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateWaitableTimerW(IN LPSECURITY_ATTRIBUTES lpTimerAttributes OPTIONAL,
//                      IN BOOL bManualReset,
//                      IN LPCWSTR lpTimerName OPTIONAL)
// {
//     CreateNtObjectFromWin32Api(WaitableTimer, Timer, TIMER_ALL_ACCESS,
//                                lpTimerAttributes,
//                                lpTimerName,
//                                bManualReset ? NotificationTimer : SynchronizationTimer);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateWaitableTimerA(IN LPSECURITY_ATTRIBUTES lpTimerAttributes OPTIONAL,
//                      IN BOOL bManualReset,
//                      IN LPCSTR lpTimerName OPTIONAL)
// {
//     ConvertWin32AnsiObjectApiToUnicodeApi(WaitableTimer, lpTimerName, lpTimerAttributes, bManualReset);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// OpenWaitableTimerW(IN DWORD dwDesiredAccess,
//                    IN BOOL bInheritHandle,
//                    IN LPCWSTR lpTimerName)
// {
//     OpenNtObjectFromWin32Api(Timer, dwDesiredAccess, bInheritHandle, lpTimerName);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// OpenWaitableTimerA(IN DWORD dwDesiredAccess,
//                    IN BOOL bInheritHandle,
//                    IN LPCSTR lpTimerName)
// {
//     ConvertOpenWin32AnsiObjectApiToUnicodeApi(WaitableTimer, dwDesiredAccess, bInheritHandle, lpTimerName);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// SetWaitableTimer(IN HANDLE hTimer,
//                  IN const LARGE_INTEGER *pDueTime,
//                  IN LONG lPeriod,
//                  IN PTIMERAPCROUTINE pfnCompletionRoutine OPTIONAL,
//                  IN OPTIONAL LPVOID lpArgToCompletionRoutine,
//                  IN BOOL fResume)
// {
//     NTSTATUS Status;
//
//     /* Set the timer */
//     Status = NtSetTimer(hTimer,
//                         (PLARGE_INTEGER)pDueTime,
//                         (PTIMER_APC_ROUTINE)pfnCompletionRoutine,
//                         lpArgToCompletionRoutine,
//                         (BOOLEAN)fResume,
//                         lPeriod,
//                         NULL);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* If we got here, then we failed */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// CancelWaitableTimer(IN HANDLE hTimer)
// {
//     NTSTATUS Status;
//
//     /* Cancel the timer */
//     Status = NtCancelTimer(hTimer, NULL);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* If we got here, then we failed */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateSemaphoreA(IN LPSECURITY_ATTRIBUTES lpSemaphoreAttributes  OPTIONAL,
//                  IN LONG lInitialCount,
//                  IN LONG lMaximumCount,
//                  IN LPCSTR lpName  OPTIONAL)
// {
//     ConvertWin32AnsiObjectApiToUnicodeApi(Semaphore, lpName, lpSemaphoreAttributes, lInitialCount, lMaximumCount);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateSemaphoreW(IN LPSECURITY_ATTRIBUTES lpSemaphoreAttributes  OPTIONAL,
//                  IN LONG lInitialCount,
//                  IN LONG lMaximumCount,
//                  IN LPCWSTR lpName  OPTIONAL)
// {
//     CreateNtObjectFromWin32Api(Semaphore, Semaphore, SEMAPHORE_ALL_ACCESS,
//                                lpSemaphoreAttributes,
//                                lpName,
//                                lInitialCount,
//                                lMaximumCount);
// }
//
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateSemaphoreExW(IN LPSECURITY_ATTRIBUTES lpSemaphoreAttributes  OPTIONAL,
//                    IN LONG lInitialCount,
//                    IN LONG lMaximumCount,
//                    IN LPCWSTR lpName  OPTIONAL,
//                    IN DWORD dwFlags OPTIONAL,
//                    IN DWORD dwDesiredAccess OPTIONAL)
// {
//     CreateNtObjectFromWin32Api(Semaphore, Semaphore, dwDesiredAccess,
//                                lpSemaphoreAttributes,
//                                lpName,
//                                lInitialCount,
//                                lMaximumCount);
// }
//
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateSemaphoreExA(IN LPSECURITY_ATTRIBUTES lpSemaphoreAttributes OPTIONAL,
//                    IN LONG lInitialCount,
//                    IN LONG lMaximumCount,
//                    IN LPCSTR lpName  OPTIONAL,
//                    IN DWORD dwFlags OPTIONAL,
//                    IN DWORD dwDesiredAccess OPTIONAL)
// {
//     ConvertAnsiToUnicodePrologue
//
//     if (!lpName)
//     {
//         return CreateSemaphoreExW(lpSemaphoreAttributes,
//                                   lInitialCount,
//                                   lMaximumCount,
//                                   NULL,
//                                   dwFlags,
//                                   dwDesiredAccess);
//     }
//
//     ConvertAnsiToUnicodeBody(lpName)
//
//     if (NT_SUCCESS(Status))
//     {
//         return CreateSemaphoreExW(lpSemaphoreAttributes,
//                                   lInitialCount,
//                                   lMaximumCount,
//                                   UnicodeCache->Buffer,
//                                   dwFlags,
//                                   dwDesiredAccess);
//     }
//
//     ConvertAnsiToUnicodeEpilogue
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// OpenSemaphoreA(IN DWORD dwDesiredAccess,
//                IN BOOL bInheritHandle,
//                IN LPCSTR lpName)
// {
//     ConvertOpenWin32AnsiObjectApiToUnicodeApi(Semaphore, dwDesiredAccess, bInheritHandle, lpName);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// OpenSemaphoreW(IN DWORD dwDesiredAccess,
//                IN BOOL bInheritHandle,
//                IN LPCWSTR lpName)
// {
//     OpenNtObjectFromWin32Api(Semaphore, dwDesiredAccess, bInheritHandle, lpName);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// ReleaseSemaphore(IN HANDLE hSemaphore,
//                  IN LONG lReleaseCount,
//                  IN LPLONG lpPreviousCount)
// {
//     NTSTATUS Status;
//
//     /* Release the semaphore */
//     Status = NtReleaseSemaphore(hSemaphore, lReleaseCount, lpPreviousCount);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* If we got here, then we failed */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateMutexA(IN LPSECURITY_ATTRIBUTES lpMutexAttributes  OPTIONAL,
//              IN BOOL bInitialOwner,
//              IN LPCSTR lpName  OPTIONAL)
// {
//     ConvertWin32AnsiObjectApiToUnicodeApi(Mutex, lpName, lpMutexAttributes, bInitialOwner);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateMutexW(IN LPSECURITY_ATTRIBUTES lpMutexAttributes  OPTIONAL,
//              IN BOOL bInitialOwner,
//              IN LPCWSTR lpName  OPTIONAL)
// {
//     CreateNtObjectFromWin32Api(Mutex, Mutant, MUTEX_ALL_ACCESS,
//                                lpMutexAttributes,
//                                lpName,
//                                bInitialOwner);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// OpenMutexA(IN DWORD dwDesiredAccess,
//            IN BOOL bInheritHandle,
//            IN LPCSTR lpName)
// {
//     ConvertOpenWin32AnsiObjectApiToUnicodeApi(Mutex, dwDesiredAccess, bInheritHandle, lpName);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// OpenMutexW(IN DWORD dwDesiredAccess,
//            IN BOOL bInheritHandle,
//            IN LPCWSTR lpName)
// {
//     OpenNtObjectFromWin32Api(Mutant, dwDesiredAccess, bInheritHandle, lpName);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// ReleaseMutex(IN HANDLE hMutex)
// {
//     NTSTATUS Status;
//
//     /* Release the mutant */
//     Status = NtReleaseMutant(hMutex, NULL);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* If we got here, then we failed */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateEventA(IN LPSECURITY_ATTRIBUTES lpEventAttributes  OPTIONAL,
//              IN BOOL bManualReset,
//              IN BOOL bInitialState,
//              IN LPCSTR lpName OPTIONAL)
// {
//     ConvertWin32AnsiObjectApiToUnicodeApi(Event, lpName, lpEventAttributes, bManualReset, bInitialState);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// CreateEventW(IN LPSECURITY_ATTRIBUTES lpEventAttributes  OPTIONAL,
//              IN BOOL bManualReset,
//              IN BOOL bInitialState,
//              IN LPCWSTR lpName  OPTIONAL)
// {
//     CreateNtObjectFromWin32Api(Event, Event, EVENT_ALL_ACCESS,
//                                lpEventAttributes,
//                                lpName,
//                                bManualReset ? NotificationEvent : SynchronizationEvent,
//                                bInitialState);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// OpenEventA(IN DWORD dwDesiredAccess,
//            IN BOOL bInheritHandle,
//            IN LPCSTR lpName)
// {
//     ConvertOpenWin32AnsiObjectApiToUnicodeApi(Event, dwDesiredAccess, bInheritHandle, lpName);
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// DECLSPEC_HOTPATCH
// OpenEventW(IN DWORD dwDesiredAccess,
//            IN BOOL bInheritHandle,
//            IN LPCWSTR lpName)
// {
//     OpenNtObjectFromWin32Api(Event, dwDesiredAccess, bInheritHandle, lpName);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// PulseEvent(IN HANDLE hEvent)
// {
//     NTSTATUS Status;
//
//     /* Pulse the event */
//     Status = NtPulseEvent(hEvent, NULL);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* If we got here, then we failed */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// ResetEvent(IN HANDLE hEvent)
// {
//     NTSTATUS Status;
//
//     /* Clear the event */
//     Status = NtResetEvent(hEvent, NULL);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* If we got here, then we failed */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// DECLSPEC_HOTPATCH
// SetEvent(IN HANDLE hEvent)
// {
//     NTSTATUS Status;
//
//     /* Set the event */
//     Status = NtSetEvent(hEvent, NULL);
//     if (NT_SUCCESS(Status)) return TRUE;
//
//     /* If we got here, then we failed */
//     BaseSetLastNTError(Status);
//     return FALSE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// InitializeCriticalSection(OUT LPCRITICAL_SECTION lpCriticalSection)
// {
//     NTSTATUS Status;
//
//     /* Initialize the critical section and raise an exception if we failed */
//     Status = RtlInitializeCriticalSection((PVOID)lpCriticalSection);
//     if (!NT_SUCCESS(Status)) RtlRaiseStatus(Status);
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// InitializeCriticalSectionAndSpinCount(OUT LPCRITICAL_SECTION lpCriticalSection,
//                                       IN DWORD dwSpinCount)
// {
//     NTSTATUS Status;
//
//     /* Initialize the critical section */
//     Status = RtlInitializeCriticalSectionAndSpinCount((PVOID)lpCriticalSection,
//                                                       dwSpinCount);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Set failure code */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* Success */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// VOID
// WINAPI
// DECLSPEC_HOTPATCH
// Sleep(IN DWORD dwMilliseconds)
// {
//     /* Call the new API */
//     SleepEx(dwMilliseconds, FALSE);
// }
//
//
// /*
//  * @implemented
//  */
// DWORD
// WINAPI
// SleepEx(IN DWORD dwMilliseconds,
//         IN BOOL bAlertable)
// {
//     LARGE_INTEGER Time;
//     PLARGE_INTEGER TimePtr;
//     NTSTATUS errCode;
//     RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME ActCtx;
//
//     /* APCs must execute with the default activation context */
//     if (bAlertable)
//     {
//         /* Setup the frame */
//         RtlZeroMemory(&ActCtx, sizeof(ActCtx));
//         ActCtx.Size = sizeof(ActCtx);
//         ActCtx.Format = RTL_CALLER_ALLOCATED_ACTIVATION_CONTEXT_STACK_FRAME_FORMAT_WHISTLER;
//         RtlActivateActivationContextUnsafeFast(&ActCtx, NULL);
//     }
//
//     /* Convert the timeout */
//     TimePtr = BaseFormatTimeOut(&Time, dwMilliseconds);
//     if (!TimePtr)
//     {
//         /* Turn an infinite wait into a really long wait */
//         Time.LowPart = 0;
//         Time.HighPart = 0x80000000;
//         TimePtr = &Time;
//     }
//
//     /* Loop the delay while APCs are alerting us */
//     do
//     {
//         /* Do the delay */
//         errCode = NtDelayExecution((BOOLEAN)bAlertable, TimePtr);
//     }
//     while ((bAlertable) && (errCode == STATUS_ALERTED));
//
//     /* Cleanup the activation context */
//     if (bAlertable) RtlDeactivateActivationContextUnsafeFast(&ActCtx);
//
//     /* Return the correct code */
//     return (errCode == STATUS_USER_APC) ? WAIT_IO_COMPLETION : 0;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// RegisterWaitForSingleObject(OUT PHANDLE phNewWaitObject,
//                             IN HANDLE hObject,
//                             IN WAITORTIMERCALLBACK Callback,
//                             IN PVOID Context,
//                             IN ULONG dwMilliseconds,
//                             IN ULONG dwFlags)
// {
//     NTSTATUS Status;
//
//     /* Get real handle */
//     hObject = TranslateStdHandle(hObject);
//
//     /* Check for console handle */
//     if ((IsConsoleHandle(hObject)) && (VerifyConsoleIoHandle(hObject)))
//     {
//         /* Get the real wait handle */
//         hObject = GetConsoleInputWaitHandle();
//     }
//
//     /* Register the wait now */
//     Status = RtlRegisterWait(phNewWaitObject,
//                              hObject,
//                              Callback,
//                              Context,
//                              dwMilliseconds,
//                              dwFlags);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Return failure */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* All good */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// RegisterWaitForSingleObjectEx(IN HANDLE hObject,
//                               IN WAITORTIMERCALLBACK Callback,
//                               IN PVOID Context,
//                               IN ULONG dwMilliseconds,
//                               IN ULONG dwFlags)
// {
//     NTSTATUS Status;
//     HANDLE hNewWaitObject;
//
//     /* Get real handle */
//     hObject = TranslateStdHandle(hObject);
//
//     /* Check for console handle */
//     if ((IsConsoleHandle(hObject)) && (VerifyConsoleIoHandle(hObject)))
//     {
//         /* Get the real wait handle */
//         hObject = GetConsoleInputWaitHandle();
//     }
//
//     /* Register the wait */
//     Status = RtlRegisterWait(&hNewWaitObject,
//                              hObject,
//                              Callback,
//                              Context,
//                              dwMilliseconds,
//                              dwFlags);
//     if (!NT_SUCCESS(Status))
//     {
//         /* Return failure */
//         BaseSetLastNTError(Status);
//         return NULL;
//     }
//
//     /* Return the object */
//     return hNewWaitObject;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// UnregisterWait(IN HANDLE WaitHandle)
// {
//     NTSTATUS Status;
//
//     /* Check for invalid handle */
//     if (!WaitHandle)
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Deregister the wait and check status */
//     Status = RtlDeregisterWaitEx(WaitHandle, NULL);
//     if (!(NT_SUCCESS(Status)) || (Status == STATUS_PENDING))
//     {
//         /* Failure or non-blocking call */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* All good */
//     return TRUE;
// }
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// UnregisterWaitEx(IN HANDLE WaitHandle,
//                  IN HANDLE CompletionEvent)
// {
//     NTSTATUS Status;
//
//     /* Check for invalid handle */
//     if (!WaitHandle)
//     {
//         /* Fail */
//         SetLastError(ERROR_INVALID_HANDLE);
//         return FALSE;
//     }
//
//     /* Deregister the wait and check status */
//     Status = RtlDeregisterWaitEx(WaitHandle, CompletionEvent);
//     if (!(NT_SUCCESS(Status)) ||
//         ((CompletionEvent != INVALID_HANDLE_VALUE) && (Status == STATUS_PENDING)))
//     {
//         /* Failure or non-blocking call */
//         BaseSetLastNTError(Status);
//         return FALSE;
//     }
//
//     /* All good */
//     return TRUE;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
