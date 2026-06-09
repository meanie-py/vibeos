use crate::shim::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{ANSI_STRING, _CLIENT_ID, LDR_DATA_TABLE_ENTRY, NtAllocateVirtualMemory, NtFreeVirtualMemory, NtOpenProcess, NtQueryInformationProcess, NtReadVirtualMemory, _OBJECT_ATTRIBUTES, PPEB_LDR_DATA, PROCESS_BASIC_INFORMATION, PRTL_DEBUG_INFORMATION, PRTL_PROCESS_HEAPS, PRTL_PROCESS_LOCKS, PRTL_PROCESS_MODULES, PRTL_PROCESS_MODULE_INFORMATION, _PROCESSINFOCLASS_ProcessBasicInformation as ProcessBasicInformation, RTL_DEBUG_INFORMATION, RTL_PROCESS_HEAPS, RTL_PROCESS_LOCKS, RTL_PROCESS_MODULE_INFORMATION, RtlUnicodeStringToAnsiString};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// NDEBUG defined; no runtime declarations needed.
#[no_mangle]
pub unsafe extern "system" fn RtlpDebugBufferCommit(
    Buffer: PRTL_DEBUG_INFORMATION,
    Size: usize,
) -> *mut core::ffi::c_void {
    let mut Remaining: u32 = (*Buffer).CommitSize as u32 - (*Buffer).OffsetFree as u32;
    let Result: *mut core::ffi::c_void;
    let Status;

    if Size > 0x7fffffff {
        return core::ptr::null_mut();
    }

    if (Remaining as usize) < Size {
        let mut Buf: *mut core::ffi::c_void;
        let mut CommitSize: usize;

        Buf = ((*Buffer).ViewBaseClient as usize + (*Buffer).CommitSize as usize)
            as *mut core::ffi::c_void;
        CommitSize = Size - Remaining as usize;

        if CommitSize > 0x7fffffff {
            return core::ptr::null_mut();
        }

        Status = NtAllocateVirtualMemory(
            -1isize as *mut core::ffi::c_void,
            &mut Buf as *mut *mut core::ffi::c_void,
            0,
            core::ptr::addr_of_mut!(CommitSize) as ntkrnl_sys::ddk::PSIZE_T,
            MEM_COMMIT,
            PAGE_READWRITE,
        );
        if !NT_SUCCESS(Status) {
            return core::ptr::null_mut();
        }

        (*Buffer).CommitSize += CommitSize as ntkrnl_sys::ddk::SIZE_T;
        Remaining = (*Buffer).CommitSize as u32 - (*Buffer).OffsetFree as u32;
        if (Remaining as usize) < Size {
            return core::ptr::null_mut();
        }
    }

    Result = ((*Buffer).ViewBaseClient as *mut u8).add((*Buffer).OffsetFree as usize)
        as *mut core::ffi::c_void;
    (*Buffer).OffsetFree += Size as ntkrnl_sys::ddk::ULONG_PTR;

    Result
}

#[no_mangle]
pub unsafe extern "system" fn RtlCreateQueryDebugBuffer(
    Size: ULONG,
    _EventPair: BOOLEAN,
) -> PRTL_DEBUG_INFORMATION {
    let mut Status: NTSTATUS;
    let mut Buf: PRTL_DEBUG_INFORMATION = core::ptr::null_mut();
    let mut AllocationSize: SIZE_T = if Size != 0 {
        Size as SIZE_T
    } else {
        (0x400 * PAGE_SIZE) as SIZE_T
    };
    let mut CommitSize: SIZE_T = core::mem::size_of::<RTL_DEBUG_INFORMATION>() as SIZE_T;

    Status = NtAllocateVirtualMemory(
        -1isize as HANDLE,
        &mut Buf as *mut PRTL_DEBUG_INFORMATION as *mut PVOID,
        0,
        core::ptr::addr_of_mut!(AllocationSize) as ntkrnl_sys::ddk::PSIZE_T,
        MEM_RESERVE,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        return core::ptr::null_mut();
    }

    Status = NtAllocateVirtualMemory(
        -1isize as HANDLE,
        &mut Buf as *mut PRTL_DEBUG_INFORMATION as *mut PVOID,
        0,
        core::ptr::addr_of_mut!(CommitSize) as ntkrnl_sys::ddk::PSIZE_T,
        MEM_COMMIT,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        RtlDestroyQueryDebugBuffer(Buf);
        return core::ptr::null_mut();
    }

    (*Buf).ViewBaseClient = Buf as PVOID;
    (*Buf).ViewSize = AllocationSize as _;
    (*Buf).CommitSize = CommitSize as _;
    (*Buf).OffsetFree = core::mem::size_of::<RTL_DEBUG_INFORMATION>() as _;

    Buf
}

#[no_mangle]
pub unsafe extern "system" fn RtlDestroyQueryDebugBuffer(mut Buf: PRTL_DEBUG_INFORMATION) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS as NTSTATUS;
    let mut ViewSize: SIZE_T = 0;

    if !Buf.is_null() {
        Status = NtFreeVirtualMemory(
            -1isize as *mut core::ffi::c_void,
            core::ptr::addr_of_mut!(Buf) as *mut PVOID,
            core::ptr::addr_of_mut!(ViewSize) as ntkrnl_sys::ddk::PSIZE_T,
            MEM_RELEASE,
        );
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn RtlpQueryRemoteProcessModules(
    ProcessHandle: HANDLE,
    Modules: PRTL_PROCESS_MODULES,
    Size: ULONG,
    ReturnedSize: PULONG,
) -> NTSTATUS {
    let mut pbiInfo: PROCESS_BASIC_INFORMATION = core::mem::zeroed();
    let mut ppldLdrData: PPEB_LDR_DATA = core::ptr::null_mut();
    let mut lmModule: LDR_DATA_TABLE_ENTRY = core::mem::zeroed();
    let pleListHead: PLIST_ENTRY;
    // Filled via NtReadVirtualMemory through a raw pointer below; init so the
    // binding is considered initialized (the C reads Flink into it).
    let mut pleCurEntry: PLIST_ENTRY = core::ptr::null_mut();

    let mut ModulePtr: PRTL_PROCESS_MODULE_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut UsedSize: ULONG = core::mem::size_of::<ULONG>() as ULONG;
    let mut AnsiString: ANSI_STRING = core::mem::zeroed();
    let mut p: PCHAR;

    Status = NtQueryInformationProcess(
        ProcessHandle,
        ProcessBasicInformation,
        &mut pbiInfo as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<PROCESS_BASIC_INFORMATION>() as ULONG,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if Modules.is_null() || Size == 0 {
        Status = STATUS_INFO_LENGTH_MISMATCH;
    } else {
        (*Modules).NumberOfModules = 0;
        ModulePtr = core::ptr::addr_of_mut!((*Modules).Modules[0]);
        Status = STATUS_SUCCESS;
    }

    let _ = Status;

    Status = NtReadVirtualMemory(
        ProcessHandle,
        core::ptr::addr_of_mut!((*pbiInfo.PebBaseAddress).Ldr) as *mut core::ffi::c_void,
        &mut ppldLdrData as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<PPEB_LDR_DATA>() as ntkrnl_sys::ddk::SIZE_T,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    pleListHead = core::ptr::addr_of_mut!((*ppldLdrData).InLoadOrderModuleList) as PLIST_ENTRY;

    Status = NtReadVirtualMemory(
        ProcessHandle,
        core::ptr::addr_of_mut!((*ppldLdrData).InLoadOrderModuleList.Flink) as *mut core::ffi::c_void,
        &mut pleCurEntry as *mut _ as *mut core::ffi::c_void,
        core::mem::size_of::<PLIST_ENTRY>() as ntkrnl_sys::ddk::SIZE_T,
        core::ptr::null_mut(),
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    pleCurEntry = unsafe {
        let mut tmp: PLIST_ENTRY = core::ptr::null_mut();
        core::ptr::copy_nonoverlapping(
            core::ptr::addr_of!((*ppldLdrData).InLoadOrderModuleList.Flink) as *const PLIST_ENTRY,
            &mut tmp,
            0,
        );
        pleCurEntry
    };

    while pleCurEntry != pleListHead {
        let mut Unicode: _UNICODE_STRING = core::mem::zeroed();
        let mut Buffer: [WCHAR; 256 * core::mem::size_of::<WCHAR>()] =
            [0; 256 * core::mem::size_of::<WCHAR>()];

        let containing = (pleCurEntry as usize
            - core::mem::offset_of!(LDR_DATA_TABLE_ENTRY, InLoadOrderLinks))
            as *mut core::ffi::c_void;

        Status = NtReadVirtualMemory(
            ProcessHandle,
            containing,
            &mut lmModule as *mut _ as *mut core::ffi::c_void,
            core::mem::size_of::<LDR_DATA_TABLE_ENTRY>() as ntkrnl_sys::ddk::SIZE_T,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }

        Unicode.Length = lmModule.FullDllName.Length;
        Unicode.MaximumLength = lmModule.FullDllName.MaximumLength;
        Unicode.Buffer = Buffer.as_mut_ptr();

        Status = NtReadVirtualMemory(
            ProcessHandle,
            lmModule.FullDllName.Buffer as *mut core::ffi::c_void,
            Unicode.Buffer as *mut core::ffi::c_void,
            Unicode.Length as ntkrnl_sys::ddk::SIZE_T,
            core::ptr::null_mut(),
        );

        if !NT_SUCCESS(Status) {
            return Status;
        }

        if UsedSize > Size {
            Status = STATUS_INFO_LENGTH_MISMATCH;
        } else if !Modules.is_null() {
            (*ModulePtr).Section = 0 as _;
            (*ModulePtr).MappedBase = core::ptr::null_mut();
            (*ModulePtr).ImageBase = lmModule.DllBase;
            (*ModulePtr).ImageSize = lmModule.SizeOfImage;
            (*ModulePtr).Flags = lmModule.Flags;
            (*ModulePtr).LoadOrderIndex = 0;
            (*ModulePtr).InitOrderIndex = 0;
            (*ModulePtr).LoadCount = lmModule.LoadCount;

            AnsiString.Length = 0;
            AnsiString.MaximumLength = 256;
            AnsiString.Buffer = (*ModulePtr).FullPathName.as_mut_ptr() as *mut _;
            RtlUnicodeStringToAnsiString(&mut AnsiString, &mut Unicode, 0u8);

            p = strrchr((*ModulePtr).FullPathName.as_mut_ptr() as PCHAR, '\\' as i32);
            if !p.is_null() {
                (*ModulePtr).OffsetToFileName =
                    (p as usize - (*ModulePtr).FullPathName.as_ptr() as usize + 1) as USHORT;
            } else {
                (*ModulePtr).OffsetToFileName = 0;
            }

            ModulePtr = ModulePtr.add(1);
            (*Modules).NumberOfModules += 1;
        }
        UsedSize += core::mem::size_of::<RTL_PROCESS_MODULE_INFORMATION>() as ULONG;

        pleCurEntry = lmModule.InLoadOrderLinks.Flink;
    }

    if !ReturnedSize.is_null() {
        core::ptr::write_volatile(ReturnedSize, UsedSize);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn RtlQueryProcessDebugInformation(
    ProcessId: ULONG,
    DebugInfoMask: ULONG,
    Buf: PRTL_DEBUG_INFORMATION,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let Pid = (*NtCurrentTeb()).ClientId.UniqueProcess as usize as ULONG;

    (*Buf).Flags = DebugInfoMask;
    (*Buf).OffsetFree = core::mem::size_of::<RTL_DEBUG_INFORMATION>() as _;

    if ProcessId <= 1 {
        Status = STATUS_ACCESS_VIOLATION;
    } else if Pid == ProcessId {
        if (DebugInfoMask & RTL_DEBUG_QUERY_MODULES) != 0 {
            let mut ReturnSize: ULONG = 0;

            Status = LdrQueryProcessModuleInformation(
                core::ptr::null_mut(),
                0,
                &mut ReturnSize,
            );

            let Mp = RtlpDebugBufferCommit(Buf, ReturnSize as usize) as PRTL_PROCESS_MODULES;

            Status = LdrQueryProcessModuleInformation(Mp, ReturnSize, &mut ReturnSize);
            if !NT_SUCCESS(Status) {
                return Status;
            }

            (*Buf).__bindgen_anon_1.Modules = Mp;
        }

        if (DebugInfoMask & RTL_DEBUG_QUERY_HEAPS) != 0 {
            let Hp = (Buf as *mut u8).add((*Buf).OffsetFree as usize) as PRTL_PROCESS_HEAPS;
            let HSize = core::mem::size_of::<RTL_PROCESS_HEAPS>() as ntkrnl_sys::ddk::ULONG_PTR;
            (*Buf).Heaps = Hp;
            (*Buf).OffsetFree = (*Buf).OffsetFree + HSize;
        }

        if (DebugInfoMask & RTL_DEBUG_QUERY_LOCKS) != 0 {
            let Lp = (Buf as *mut u8).add((*Buf).OffsetFree as usize) as PRTL_PROCESS_LOCKS;
            let LSize = core::mem::size_of::<RTL_PROCESS_LOCKS>() as ntkrnl_sys::ddk::ULONG_PTR;
            (*Buf).Locks = Lp;
            (*Buf).OffsetFree = (*Buf).OffsetFree + LSize;
        }
    } else {
        let mut hProcess: HANDLE = core::ptr::null_mut();
        let mut ClientId: _CLIENT_ID = core::mem::zeroed();
        let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

        (*Buf).TargetProcessHandle = -1isize as HANDLE;

        ClientId.UniqueThread = core::ptr::null_mut();
        ClientId.UniqueProcess = ProcessId as usize as HANDLE;

        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut();
        ObjectAttributes.Attributes = 0;
        ObjectAttributes.ObjectName = core::ptr::null_mut();
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

        Status = NtOpenProcess(
            &mut hProcess,
            PROCESS_ALL_ACCESS,
            &mut ObjectAttributes,
            &mut ClientId,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        if (DebugInfoMask & RTL_DEBUG_QUERY_MODULES) != 0 {
            let mut ReturnSize: ULONG = 0;

            Status = RtlpQueryRemoteProcessModules(
                hProcess,
                core::ptr::null_mut(),
                0,
                &mut ReturnSize,
            );

            let Mp = RtlpDebugBufferCommit(Buf, ReturnSize as usize) as PRTL_PROCESS_MODULES;

            Status = RtlpQueryRemoteProcessModules(hProcess, Mp, ReturnSize, &mut ReturnSize);
            if !NT_SUCCESS(Status) {
                return Status;
            }

            (*Buf).__bindgen_anon_1.Modules = Mp;
        }

        if (DebugInfoMask & RTL_DEBUG_QUERY_HEAPS) != 0 {
            let Hp = (Buf as *mut u8).add((*Buf).OffsetFree as usize) as PRTL_PROCESS_HEAPS;
            let HSize = core::mem::size_of::<RTL_PROCESS_HEAPS>() as ntkrnl_sys::ddk::ULONG_PTR;
            (*Buf).Heaps = Hp;
            (*Buf).OffsetFree = (*Buf).OffsetFree + HSize;
        }

        if (DebugInfoMask & RTL_DEBUG_QUERY_LOCKS) != 0 {
            let Lp = (Buf as *mut u8).add((*Buf).OffsetFree as usize) as PRTL_PROCESS_LOCKS;
            let LSize = core::mem::size_of::<RTL_PROCESS_LOCKS>() as ntkrnl_sys::ddk::ULONG_PTR;
            (*Buf).Locks = Lp;
            (*Buf).OffsetFree = (*Buf).OffsetFree + LSize;
        }
    }

    Status
}
