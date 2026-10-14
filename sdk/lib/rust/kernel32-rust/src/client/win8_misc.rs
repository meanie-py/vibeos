//! Win8/Win10-era kernel32 exports that real desktop apps import at load time (Wine
//! dlls/kernelbase semantics): SetFileInformationByHandle, CreateFile2,
//! CreateRemoteThreadEx, Get/Set{Process,Thread}Information, stack limits, unbiased
//! interrupt time, WER runtime-module hooks, WoW64 thread context, XState features,
//! processor-group/NUMA shims, virtual-memory hints, power requests and
//! CompareObjectHandles.
//!
//! Where our kernel lacks the backing Nt* class the Nt call fails and the NTSTATUS is
//! reported through BaseSetLastNTError; nothing here fakes success for a class the
//! system does not implement (the two documented advisory no-ops are called out inline).
use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

const S_OK_: HRESULT = 0;

const ERROR_NOT_SUPPORTED_: u32 = 50;
const ERROR_INVALID_PARAMETER_: u32 = 87;

const STATUS_INFO_LENGTH_MISMATCH_: NTSTATUS = 0xC000_0004_u32 as NTSTATUS;
const STATUS_INVALID_INFO_CLASS_: NTSTATUS = 0xC000_0003_u32 as NTSTATUS;
const STATUS_NOT_IMPLEMENTED_: NTSTATUS = 0xC000_0002_u32 as NTSTATUS;

// NT information classes that exist in Windows but not in our ndk enums (Wine's
// winternl.h values). Our kernel rejects them with STATUS_INVALID_INFO_CLASS, which
// BaseSetLastNTError surfaces to the caller.
const PROCESS_POWER_THROTTLING_STATE: u32 = 77;
const PROCESS_LEAP_SECOND_INFORMATION: u32 = 97;

extern "system" {
    fn RtlDosPathNameToNtPathName_U_WithStatus(
        DosName: PCWSTR,
        NtName: PUNICODE_STRING,
        PartName: *mut PCWSTR,
        RelativeName: *mut c_void,
    ) -> NTSTATUS;
}

#[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
extern "system" {
    fn RtlWow64GetThreadContext(ThreadHandle: HANDLE, Context: *mut c_void) -> NTSTATUS;
    fn RtlWow64SetThreadContext(ThreadHandle: HANDLE, Context: *const c_void) -> NTSTATUS;
}

#[inline]
unsafe fn set_ntstatus(Status: NTSTATUS) -> BOOL {
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }
    TRUE
}

/* ------------------------------------------------------------------------- *
 * SetFileInformationByHandle
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn SetFileInformationByHandle(
    hFile: HANDLE,
    FileInformationClass: FILE_INFO_BY_HANDLE_CLASS,
    lpFileInformation: LPVOID,
    dwBufferSize: DWORD,
) -> BOOL {
    let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();
    let nt_class: _FILE_INFORMATION_CLASS = match FileInformationClass {
        x if x == FileEndOfFileInfo => FileEndOfFileInformation,
        x if x == FileBasicInfo => FileBasicInformation,
        x if x == FileDispositionInfo => FileDispositionInformation,
        x if x == FileDispositionInfoEx => FileDispositionInformationEx,
        // Windows maps this to FileAllocationInformation (Wine leaves it unimplemented).
        x if x == FileAllocationInfo => FileAllocationInformation,
        x if x == FileIoPriorityHintInfo => {
            // Pure hint. Pass it to the kernel as Wine does, but a kernel with no such
            // class must not make a hint fail the caller: accept it as a no-op then.
            let Status = NtSetInformationFile(
                hFile,
                &mut Iosb,
                lpFileInformation,
                dwBufferSize,
                FileIoPriorityHintInformation,
            );
            if Status == STATUS_INVALID_INFO_CLASS_ || Status == STATUS_NOT_IMPLEMENTED_ {
                return TRUE;
            }
            return set_ntstatus(Status);
        }
        x if x == FileRenameInfo => {
            return set_file_rename(hFile, lpFileInformation, dwBufferSize);
        }
        // Wine: FileNameInfo/FileStreamInfo/directory-enumeration/etc. are query-only
        // or unimplemented, and the rest (Standard, Compression, AttributeTag,
        // RemoteProtocol, FileRenameInfoEx, ...) are not settable.
        _ => {
            SetLastError(ERROR_INVALID_PARAMETER_);
            return FALSE;
        }
    };
    if lpFileInformation.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    set_ntstatus(NtSetInformationFile(
        hFile,
        &mut Iosb,
        lpFileInformation,
        dwBufferSize,
        nt_class,
    ))
}

/// FILE_RENAME_INFO has the layout of FILE_RENAME_INFORMATION. Wine converts the DOS
/// FileName to an NT path and re-packs the header; this does the same, but honours
/// FileNameLength instead of assuming a NUL-terminated name, and (a divergence) leaves
/// the name untouched when RootDirectory is set: that name is relative to the root
/// handle, and Wine's DOS-to-NT conversion would turn it into an absolute path that
/// NtSetInformationFile then rejects together with a RootDirectory.
unsafe fn set_file_rename(hFile: HANDLE, Info: LPVOID, Size: DWORD) -> BOOL {
    let name_off = core::mem::offset_of!(FILE_RENAME_INFORMATION, FileName);
    if Info.is_null() || (Size as usize) < name_off {
        BaseSetLastNTError(STATUS_INFO_LENGTH_MISMATCH_);
        return FALSE;
    }
    let src = Info as *const FILE_RENAME_INFORMATION;
    let replace = (*src).ReplaceIfExists;
    let root = (*src).RootDirectory;
    let name_len = (*src).FileNameLength as usize;
    if name_len & 1 != 0 || name_len > Size as usize - name_off {
        BaseSetLastNTError(STATUS_INFO_LENGTH_MISMATCH_);
        return FALSE;
    }
    let src_name = (Info as *const u8).add(name_off);
    let heap = RtlGetProcessHeap();

    let mut nt_name: UNICODE_STRING = core::mem::zeroed();
    let mut have_nt = false;
    let (buf, buf_len): (*const u8, usize) = if root.is_null() {
        // NUL-terminated copy of the DOS name for the converter.
        let tmp = RtlAllocateHeap(heap, 0, (name_len + 2) as SIZE_T) as *mut u8;
        if tmp.is_null() {
            BaseSetLastNTError(0xC000_0017_u32 as NTSTATUS); // STATUS_NO_MEMORY
            return FALSE;
        }
        core::ptr::copy_nonoverlapping(src_name, tmp, name_len);
        *tmp.add(name_len) = 0;
        *tmp.add(name_len + 1) = 0;
        let Status = RtlDosPathNameToNtPathName_U_WithStatus(
            tmp as PCWSTR,
            &mut nt_name,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );
        RtlFreeHeap(heap, 0, tmp as PVOID);
        if !NT_SUCCESS(Status) {
            BaseSetLastNTError(Status);
            return FALSE;
        }
        have_nt = true;
        (nt_name.Buffer as *const u8, nt_name.Length as usize)
    } else {
        (src_name, name_len)
    };

    // sizeof(FILE_RENAME_INFORMATION) already includes one WCHAR for the terminator.
    let total = core::mem::size_of::<FILE_RENAME_INFORMATION>() + buf_len;
    let out = RtlAllocateHeap(heap, 0, total as SIZE_T) as *mut FILE_RENAME_INFORMATION;
    let mut ok = FALSE;
    if out.is_null() {
        BaseSetLastNTError(0xC000_0017_u32 as NTSTATUS);
    } else {
        core::ptr::write_bytes(out as *mut u8, 0, total);
        (*out).ReplaceIfExists = replace;
        (*out).RootDirectory = root;
        (*out).FileNameLength = buf_len as ULONG;
        core::ptr::copy_nonoverlapping(buf, (out as *mut u8).add(name_off), buf_len);
        let mut Iosb: IO_STATUS_BLOCK = core::mem::zeroed();
        let Status = NtSetInformationFile(
            hFile,
            &mut Iosb,
            out as PVOID,
            total as ULONG,
            FileRenameInformation,
        );
        ok = set_ntstatus(Status);
        RtlFreeHeap(heap, 0, out as PVOID);
    }
    if have_nt {
        RtlFreeUnicodeString(&mut nt_name);
    }
    ok
}

/* ------------------------------------------------------------------------- *
 * CreateFile2
 * ------------------------------------------------------------------------- */

/// CREATEFILE2_EXTENDED_PARAMETERS (fileapi.h).
#[repr(C)]
pub struct CREATEFILE2_EXTENDED_PARAMETERS {
    pub dwSize: DWORD,
    pub dwFileAttributes: DWORD,
    pub dwFileFlags: DWORD,
    pub dwSecurityQosFlags: DWORD,
    pub lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
    pub hTemplateFile: HANDLE,
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(core::mem::size_of::<CREATEFILE2_EXTENDED_PARAMETERS>() == 32);
#[cfg(target_pointer_width = "32")]
const _: () = assert!(core::mem::size_of::<CREATEFILE2_EXTENDED_PARAMETERS>() == 24);

#[no_mangle]
pub unsafe extern "system" fn CreateFile2(
    lpFileName: LPCWSTR,
    dwDesiredAccess: DWORD,
    dwShareMode: DWORD,
    dwCreationDisposition: DWORD,
    pCreateExParams: *mut CREATEFILE2_EXTENDED_PARAMETERS,
) -> HANDLE {
    // Same masks as Wine: bits CreateFileW cannot honour are dropped.
    const ATTRIBUTES_MASK: DWORD = FILE_ATTRIBUTE_READONLY
        | FILE_ATTRIBUTE_HIDDEN
        | FILE_ATTRIBUTE_SYSTEM
        | FILE_ATTRIBUTE_ARCHIVE
        | FILE_ATTRIBUTE_NORMAL
        | FILE_ATTRIBUTE_TEMPORARY
        | FILE_ATTRIBUTE_OFFLINE
        | FILE_ATTRIBUTE_ENCRYPTED
        | FILE_ATTRIBUTE_INTEGRITY_STREAM;
    const FLAGS_MASK: DWORD = FILE_FLAG_BACKUP_SEMANTICS
        | FILE_FLAG_DELETE_ON_CLOSE
        | FILE_FLAG_NO_BUFFERING
        | FILE_FLAG_OPEN_NO_RECALL
        | FILE_FLAG_OPEN_REPARSE_POINT
        | FILE_FLAG_OVERLAPPED
        | FILE_FLAG_POSIX_SEMANTICS
        | FILE_FLAG_RANDOM_ACCESS
        | FILE_FLAG_SEQUENTIAL_SCAN
        | FILE_FLAG_WRITE_THROUGH;

    let mut sa: LPSECURITY_ATTRIBUTES = core::ptr::null_mut();
    let mut template: HANDLE = core::ptr::null_mut();
    let mut flags_and_attrs: DWORD = 0;
    if !pCreateExParams.is_null() {
        let p = &*pCreateExParams;
        if p.dwSize as usize != core::mem::size_of::<CREATEFILE2_EXTENDED_PARAMETERS>() {
            SetLastError(ERROR_INVALID_PARAMETER_);
            return INVALID_HANDLE_VALUE;
        }
        sa = p.lpSecurityAttributes;
        template = p.hTemplateFile;
        // dwSecurityQosFlags (SECURITY_SQOS_PRESENT | impersonation level ...) lives in
        // the same dwFlagsAndAttributes word as the attributes and flags.
        flags_and_attrs =
            (p.dwFileAttributes & ATTRIBUTES_MASK) | (p.dwFileFlags & FLAGS_MASK) | p.dwSecurityQosFlags;
    }
    crate::k32_externs::CreateFileW(
        lpFileName,
        dwDesiredAccess,
        dwShareMode,
        sa,
        dwCreationDisposition,
        flags_and_attrs,
        template,
    )
}

/* ------------------------------------------------------------------------- *
 * CreateRemoteThreadEx
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn CreateRemoteThreadEx(
    hProcess: HANDLE,
    lpThreadAttributes: LPSECURITY_ATTRIBUTES,
    dwStackSize: SIZE_T,
    lpStartAddress: LPTHREAD_START_ROUTINE,
    lpParameter: LPVOID,
    dwCreationFlags: DWORD,
    lpAttributeList: LPPROC_THREAD_ATTRIBUTE_LIST,
    lpThreadId: LPDWORD,
) -> HANDLE {
    // The attribute list is accepted and NOT applied (ideal processor, group affinity,
    // etc. are not honoured), exactly like our CreateProcess does with its list.
    let _ = lpAttributeList;
    let stack = if dwStackSize > u32::MAX as SIZE_T { u32::MAX } else { dwStackSize as DWORD };
    crate::client::thread::CreateRemoteThread(
        hProcess,
        lpThreadAttributes,
        stack,
        lpStartAddress,
        lpParameter,
        dwCreationFlags,
        lpThreadId,
    )
}

/* ------------------------------------------------------------------------- *
 * Get/Set{Process,Thread}Information
 * ------------------------------------------------------------------------- */

/// Win32 PROCESS_INFORMATION_CLASS -> NT process class. None: no NT mapping.
fn process_nt_class(Class: u32) -> Option<u32> {
    match Class {
        x if x == ProcessMemoryPriority as u32 => Some(ProcessPagePriority as u32),
        x if x == ProcessPowerThrottling as u32 => Some(PROCESS_POWER_THROTTLING_STATE),
        x if x == ProcessLeapSecondInfo as u32 => Some(PROCESS_LEAP_SECOND_INFORMATION),
        _ => None,
    }
}

/// Win32 THREAD_INFORMATION_CLASS -> NT thread class.
fn thread_nt_class(Class: u32) -> Option<u32> {
    match Class {
        x if x == ThreadMemoryPriority as u32 => Some(ThreadPagePriority as u32),
        x if x == ThreadPowerThrottling as u32 => Some(ThreadPowerThrottlingState as u32),
        _ => None,
    }
}

fn unmapped_class_error(Class: u32, Max: u32) {
    // Known to Windows but with no NT mapping here -> not supported; out of range -> invalid.
    unsafe {
        SetLastError(if Class < Max { ERROR_NOT_SUPPORTED_ } else { ERROR_INVALID_PARAMETER_ });
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetProcessInformation(
    hProcess: HANDLE,
    ProcessInformationClass: PROCESS_INFORMATION_CLASS,
    ProcessInformation: LPVOID,
    ProcessInformationSize: DWORD,
) -> BOOL {
    match process_nt_class(ProcessInformationClass as u32) {
        Some(c) => set_ntstatus(NtSetInformationProcess(
            hProcess,
            c as _PROCESSINFOCLASS,
            ProcessInformation,
            ProcessInformationSize,
        )),
        None => {
            unmapped_class_error(ProcessInformationClass as u32, ProcessInformationClassMax as u32);
            FALSE
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetProcessInformation(
    hProcess: HANDLE,
    ProcessInformationClass: PROCESS_INFORMATION_CLASS,
    ProcessInformation: LPVOID,
    ProcessInformationSize: DWORD,
) -> BOOL {
    match process_nt_class(ProcessInformationClass as u32) {
        Some(c) => set_ntstatus(NtQueryInformationProcess(
            hProcess,
            c as _PROCESSINFOCLASS,
            ProcessInformation,
            ProcessInformationSize,
            core::ptr::null_mut(),
        )),
        // ProcessMachineTypeInfo (Wine: NtQuerySystemInformationEx
        // SystemSupportedProcessorArchitectures) is not available in our kernel.
        None => {
            unmapped_class_error(ProcessInformationClass as u32, ProcessInformationClassMax as u32);
            FALSE
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadInformation(
    hThread: HANDLE,
    ThreadInformationClass: THREAD_INFORMATION_CLASS,
    ThreadInformation: LPVOID,
    ThreadInformationSize: DWORD,
) -> BOOL {
    match thread_nt_class(ThreadInformationClass as u32) {
        Some(c) => set_ntstatus(NtSetInformationThread(
            hThread,
            c as _THREADINFOCLASS,
            ThreadInformation,
            ThreadInformationSize,
        )),
        // ThreadAbsoluteCpuPriority / ThreadDynamicCodePolicy have no NT mapping (Wine
        // rejects them too); report that instead of pretending.
        None => {
            unmapped_class_error(ThreadInformationClass as u32, ThreadInformationClassMax as u32);
            FALSE
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadInformation(
    hThread: HANDLE,
    ThreadInformationClass: THREAD_INFORMATION_CLASS,
    ThreadInformation: LPVOID,
    ThreadInformationSize: DWORD,
) -> BOOL {
    match thread_nt_class(ThreadInformationClass as u32) {
        Some(c) => set_ntstatus(NtQueryInformationThread(
            hThread,
            c as _THREADINFOCLASS,
            ThreadInformation,
            ThreadInformationSize,
            core::ptr::null_mut(),
        )),
        None => {
            unmapped_class_error(ThreadInformationClass as u32, ThreadInformationClassMax as u32);
            FALSE
        }
    }
}

/* ------------------------------------------------------------------------- *
 * Stack limits, interrupt time
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn GetCurrentThreadStackLimits(
    LowLimit: PULONG_PTR,
    HighLimit: PULONG_PTR,
) {
    let teb = NtCurrentTeb();
    *LowLimit = (*teb).DeallocationStack as ULONG_PTR;
    *HighLimit = (*teb).NtTib.StackBase as ULONG_PTR;
}

/// Torn-read-safe KSYSTEM_TIME read of KUSER_SHARED_DATA.InterruptTime (High1/Low/High2).
unsafe fn read_interrupt_time() -> ULONGLONG {
    let t = core::ptr::addr_of!((*SharedUserData).InterruptTime);
    loop {
        let high = core::ptr::read_volatile(core::ptr::addr_of!((*t).High1Time));
        let low = core::ptr::read_volatile(core::ptr::addr_of!((*t).LowPart));
        if high == core::ptr::read_volatile(core::ptr::addr_of!((*t).High2Time)) {
            return ((high as u32 as u64) << 32) | low as u64;
        }
    }
}

// KUSER_SHARED_DATA.InterruptTime is NT's *unbiased* interrupt time (it keeps counting
// 100ns units only while the system runs); our layout has no InterruptTimeBias field,
// so there is nothing to subtract.
#[no_mangle]
pub unsafe extern "system" fn QueryUnbiasedInterruptTime(UnbiasedTime: PULONGLONG) -> BOOL {
    if UnbiasedTime.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    *UnbiasedTime = read_interrupt_time();
    TRUE
}

/// Returns VOID per the SDK; the "precise" variant is the same shared-data read here
/// (the shared page is updated every clock tick, as in Wine's semi-stub).
#[no_mangle]
pub unsafe extern "system" fn QueryUnbiasedInterruptTimePrecise(UnbiasedTime: PULONGLONG) {
    if !UnbiasedTime.is_null() {
        *UnbiasedTime = read_interrupt_time();
    }
}

/* ------------------------------------------------------------------------- *
 * WER runtime exception modules
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn WerRegisterRuntimeExceptionModule(
    _pwszOutOfProcessCallbackDll: PCWSTR,
    _pContext: PVOID,
) -> HRESULT {
    // Wine: stub returning S_OK. There is no WER here to ever call the module back.
    S_OK_
}

#[no_mangle]
pub unsafe extern "system" fn WerUnregisterRuntimeExceptionModule(
    _pwszOutOfProcessCallbackDll: PCWSTR,
    _pContext: PVOID,
) -> HRESULT {
    S_OK_
}

/* ------------------------------------------------------------------------- *
 * WoW64 thread context
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn Wow64GetThreadContext(
    hThread: HANDLE,
    lpContext: *mut c_void,
) -> BOOL {
    #[cfg(target_arch = "x86")]
    {
        // On i386 a WOW64_CONTEXT is the native CONTEXT.
        return set_ntstatus(NtGetContextThread(hThread, lpContext as PCONTEXT));
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        return set_ntstatus(RtlWow64GetThreadContext(hThread, lpContext));
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = (hThread, lpContext);
        set_ntstatus(STATUS_NOT_IMPLEMENTED_)
    }
}

#[no_mangle]
pub unsafe extern "system" fn Wow64SetThreadContext(
    hThread: HANDLE,
    lpContext: *const c_void,
) -> BOOL {
    #[cfg(target_arch = "x86")]
    {
        return set_ntstatus(NtSetContextThread(hThread, lpContext as PCONTEXT));
    }
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    {
        return set_ntstatus(RtlWow64SetThreadContext(hThread, lpContext));
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
    {
        let _ = (hThread, lpContext);
        set_ntstatus(STATUS_NOT_IMPLEMENTED_)
    }
}

/* ------------------------------------------------------------------------- *
 * XState, processor groups, NUMA
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn GetEnabledXStateFeatures() -> DWORD64 {
    // Wine: RtlGetEnabledExtendedFeatures(~0) == KUSER_SHARED_DATA.XState.EnabledFeatures
    // (our ntdll stubs that Rtl function, so read the shared data directly).
    let enabled = core::ptr::read_volatile(core::ptr::addr_of!(
        (*SharedUserData).XState.EnabledFeatures
    ));
    if enabled != 0 {
        return enabled;
    }
    // The kernel does not publish XState: x87 + SSE are the architectural baseline on
    // x86/x64 (XSTATE_MASK_LEGACY); ARM64 has no XSAVE components.
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        3
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        0
    }
}

#[no_mangle]
pub unsafe extern "system" fn GetMaximumProcessorGroupCount() -> WORD {
    1 // single processor group, as GetActiveProcessorGroupCount / GetProcessGroupAffinity
}

#[no_mangle]
pub unsafe extern "system" fn GetThreadIdealProcessorEx(
    hThread: HANDLE,
    lpIdealProcessor: *mut PROCESSOR_NUMBER,
) -> BOOL {
    if lpIdealProcessor.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    // Wine's mapping; fails with the kernel's status if ThreadIdealProcessorEx is unsupported.
    set_ntstatus(NtQueryInformationThread(
        hThread,
        ThreadIdealProcessorEx,
        lpIdealProcessor as PVOID,
        core::mem::size_of::<PROCESSOR_NUMBER>() as ULONG,
        core::ptr::null_mut(),
    ))
}

#[no_mangle]
pub unsafe extern "system" fn SetThreadIdealProcessorEx(
    hThread: HANDLE,
    lpIdealProcessor: *mut PROCESSOR_NUMBER,
    lpPreviousIdealProcessor: *mut PROCESSOR_NUMBER,
) -> BOOL {
    if lpIdealProcessor.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    let want = *lpIdealProcessor;
    let mask = (*BaseStaticServerData).SysInfo.ActiveProcessorsAffinityMask as u64;
    if want.Group != 0 || want.Number >= 64 || (mask >> want.Number) & 1 == 0 {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    if !lpPreviousIdealProcessor.is_null() {
        // Best effort: the kernel may not report the current ideal processor. When it
        // cannot, the previous value is reported as group 0 / processor 0 (the
        // scheduler default), since SetThreadIdealProcessor does not return it either.
        let mut prev: PROCESSOR_NUMBER = core::mem::zeroed();
        if NtQueryInformationThread(
            hThread,
            ThreadIdealProcessorEx,
            &mut prev as *mut _ as PVOID,
            core::mem::size_of::<PROCESSOR_NUMBER>() as ULONG,
            core::ptr::null_mut(),
        ) < 0
        {
            prev = core::mem::zeroed();
        }
        *lpPreviousIdealProcessor = prev;
    }
    if crate::client::thread::SetThreadIdealProcessor(hThread, want.Number as DWORD) == !0u32 {
        return FALSE; // last error already set from the Nt status
    }
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn GetNumaProcessorNodeEx(
    Processor: *mut PROCESSOR_NUMBER,
    NodeNumber: PUSHORT,
) -> BOOL {
    if NodeNumber.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    *NodeNumber = 0xFFFF;
    if Processor.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    let p = *Processor;
    let mask = (*BaseStaticServerData).SysInfo.ActiveProcessorsAffinityMask as u64;
    if p.Group != 0 || p.Number >= 64 || (mask >> p.Number) & 1 == 0 {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    *NodeNumber = 0; // one NUMA node (see GetNumaNodeProcessorMaskEx)
    TRUE
}

/* ------------------------------------------------------------------------- *
 * Virtual memory
 * ------------------------------------------------------------------------- */

#[no_mangle]
pub unsafe extern "system" fn VirtualAllocExNuma(
    hProcess: HANDLE,
    lpAddress: LPVOID,
    dwSize: SIZE_T,
    flAllocationType: DWORD,
    flProtect: DWORD,
    nndPreferred: DWORD,
) -> LPVOID {
    // Single NUMA node: the preferred node is a placement hint with nothing to steer
    // (Wine ignores it too).
    let _ = nndPreferred;
    crate::client::virtmem::VirtualAllocEx(hProcess, lpAddress, dwSize, flAllocationType, flProtect)
}

#[no_mangle]
pub unsafe extern "system" fn PrefetchVirtualMemory(
    hProcess: HANDLE,
    NumberOfEntries: ULONG_PTR,
    VirtualAddresses: PVOID,
    Flags: ULONG,
) -> BOOL {
    let _ = hProcess;
    if Flags != 0 || (NumberOfEntries != 0 && VirtualAddresses.is_null()) {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    // Purely a hint (Windows may ignore it); NtSetInformationVirtualMemory does not exist
    // in our ntdll, and pages are demand-faulted anyway.
    TRUE
}

#[no_mangle]
pub unsafe extern "system" fn DiscardVirtualMemory(
    VirtualAddress: PVOID,
    Size: SIZE_T,
) -> DWORD {
    let mut base = VirtualAddress;
    let mut size = Size;
    let Status = NtAllocateVirtualMemory(
        NtCurrentProcess(),
        &mut base,
        0,
        &mut size,
        MEM_RESET,
        PAGE_NOACCESS,
    );
    // Returns the Win32 error code itself (not BOOL), as Wine/Windows do.
    RtlNtStatusToDosError(Status)
}

/* ------------------------------------------------------------------------- *
 * Power requests
 * ------------------------------------------------------------------------- */

// There is no NtCreatePowerRequest in our kernel. A power request is therefore just a
// unique kernel object handle (a manual-reset event); Set/Clear validate the handle and
// the request type and succeed without touching any power policy.

#[no_mangle]
pub unsafe extern "system" fn PowerCreateRequest(Context: PVOID) -> HANDLE {
    if Context.is_null() {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return INVALID_HANDLE_VALUE;
    }
    let h = crate::client::synch::CreateEventW(core::ptr::null_mut(), TRUE, FALSE, core::ptr::null());
    if h.is_null() {
        return INVALID_HANDLE_VALUE;
    }
    h
}

unsafe fn power_request_check(PowerRequest: HANDLE, Type: u32) -> BOOL {
    // POWER_REQUEST_TYPE: Display, System, AwayMode, Execution.
    if Type > 3 {
        SetLastError(ERROR_INVALID_PARAMETER_);
        return FALSE;
    }
    let mut info: EVENT_BASIC_INFORMATION = core::mem::zeroed();
    set_ntstatus(NtQueryEvent(
        PowerRequest,
        EventBasicInformation,
        &mut info as *mut _ as PVOID,
        core::mem::size_of::<EVENT_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    ))
}

#[no_mangle]
pub unsafe extern "system" fn PowerSetRequest(PowerRequest: HANDLE, RequestType: u32) -> BOOL {
    power_request_check(PowerRequest, RequestType)
}

#[no_mangle]
pub unsafe extern "system" fn PowerClearRequest(PowerRequest: HANDLE, RequestType: u32) -> BOOL {
    power_request_check(PowerRequest, RequestType)
}

/* ------------------------------------------------------------------------- *
 * CompareObjectHandles
 * ------------------------------------------------------------------------- */

extern "system" {
    fn NtCompareObjects(FirstObjectHandle: HANDLE, SecondObjectHandle: HANDLE) -> NTSTATUS;
}

/// Do the two handles name the same object? Over NtCompareObjects (ob-rust): TRUE when
/// they do; FALSE with ERROR_NOT_SAME_OBJECT (STATUS_NOT_SAME_OBJECT's mapping) when they
/// do not, and FALSE with the handle's error when either is invalid.
#[no_mangle]
pub unsafe extern "system" fn CompareObjectHandles(
    hFirstObjectHandle: HANDLE,
    hSecondObjectHandle: HANDLE,
) -> BOOL {
    let Status = NtCompareObjects(hFirstObjectHandle, hSecondObjectHandle);
    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }
    TRUE
}
