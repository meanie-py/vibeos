use ntdll_sys::nt::*;
use crate::prelude::*;
use core::ffi::c_void;

// ---- cohab glue (kernel32 Rust port: tlhelp32 FFI surface + intra-DLL imports) ----
use crate::k32_externs::{RtlCopyMemory, RtlZeroMemory, CloseHandle, SetLastError};

// tlhelp32.h constants
pub const MAX_MODULE_NAME32: usize = 255;
pub const LF32_FIXED: DWORD = 0x1;
pub const LF32_FREE: DWORD = 0x2;
pub const LF32_MOVEABLE: DWORD = 0x4;
pub const TH32CS_SNAPHEAPLIST: DWORD = 0x1;
pub const TH32CS_SNAPPROCESS: DWORD = 0x2;
pub const TH32CS_SNAPTHREAD: DWORD = 0x4;
pub const TH32CS_SNAPMODULE: DWORD = 0x8;
pub const TH32CS_INHERIT: DWORD = 0x80000000;
const STATUS_NO_MORE_FILES: NTSTATUS = 0x8000_0006u32 as NTSTATUS;
// BOOL is i32; the nt::* glob exports FALSE/TRUE as u32 — shadow with BOOL-typed consts.
const FALSE: BOOL = 0;
const TRUE: BOOL = 1;

// L"[System Process]" (no_std: no Vec/string literals for UTF-16)
static SYSTEM_PROCESS: [u16; 17] = [
    b'[' as u16, b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16,
    b'm' as u16, b' ' as u16, b'P' as u16, b'r' as u16, b'o' as u16, b'c' as u16,
    b'e' as u16, b's' as u16, b's' as u16, b']' as u16, 0,
];

// tlhelp32.h structures (PE-COFF layout; faithful to sdk/include/psdk/tlhelp32.h)
#[repr(C)]
pub struct tagHEAPLIST32 {
    pub dwSize: DWORD,
    pub th32ProcessID: DWORD,
    pub th32HeapID: DWORD,
    pub dwFlags: DWORD,
}
pub type HEAPLIST32 = tagHEAPLIST32;
pub type PHEAPLIST32 = *mut tagHEAPLIST32;
pub type LPHEAPLIST32 = *mut tagHEAPLIST32;

#[repr(C)]
pub struct tagHEAPENTRY32 {
    pub dwSize: DWORD,
    pub hHandle: HANDLE,
    pub dwAddress: DWORD,
    pub dwBlockSize: DWORD,
    pub dwFlags: DWORD,
    pub dwLockCount: DWORD,
    pub dwResvd: DWORD,
    pub th32ProcessID: DWORD,
    pub th32HeapID: DWORD,
}
pub type HEAPENTRY32 = tagHEAPENTRY32;
pub type PHEAPENTRY32 = *mut tagHEAPENTRY32;
pub type LPHEAPENTRY32 = *mut tagHEAPENTRY32;

#[repr(C)]
pub struct tagPROCESSENTRY32W {
    pub dwSize: DWORD,
    pub cntUsage: DWORD,
    pub th32ProcessID: DWORD,
    pub th32DefaultHeapID: ULONG_PTR,
    pub th32ModuleID: DWORD,
    pub cntThreads: DWORD,
    pub th32ParentProcessID: DWORD,
    pub pcPriClassBase: LONG,
    pub dwFlags: DWORD,
    pub szExeFile: [WCHAR; 260],
}
pub type PROCESSENTRY32W = tagPROCESSENTRY32W;
pub type PPROCESSENTRY32W = *mut tagPROCESSENTRY32W;
pub type LPPROCESSENTRY32W = *mut tagPROCESSENTRY32W;

#[repr(C)]
pub struct tagPROCESSENTRY32 {
    pub dwSize: DWORD,
    pub cntUsage: DWORD,
    pub th32ProcessID: DWORD,
    pub th32DefaultHeapID: ULONG_PTR,
    pub th32ModuleID: DWORD,
    pub cntThreads: DWORD,
    pub th32ParentProcessID: DWORD,
    pub pcPriClassBase: LONG,
    pub dwFlags: DWORD,
    pub szExeFile: [CHAR; 260],
}
pub type PROCESSENTRY32 = tagPROCESSENTRY32;
pub type PPROCESSENTRY32 = *mut tagPROCESSENTRY32;
pub type LPPROCESSENTRY32 = *mut tagPROCESSENTRY32;

#[repr(C)]
pub struct tagMODULEENTRY32W {
    pub dwSize: DWORD,
    pub th32ModuleID: DWORD,
    pub th32ProcessID: DWORD,
    pub GlblcntUsage: DWORD,
    pub ProccntUsage: DWORD,
    pub modBaseAddr: *mut BYTE,
    pub modBaseSize: DWORD,
    pub hModule: HMODULE,
    pub szModule: [WCHAR; MAX_MODULE_NAME32 + 1],
    pub szExePath: [WCHAR; 260],
}
pub type MODULEENTRY32W = tagMODULEENTRY32W;
pub type PMODULEENTRY32W = *mut tagMODULEENTRY32W;
pub type LPMODULEENTRY32W = *mut tagMODULEENTRY32W;

#[repr(C)]
pub struct tagMODULEENTRY32 {
    pub dwSize: DWORD,
    pub th32ModuleID: DWORD,
    pub th32ProcessID: DWORD,
    pub GlblcntUsage: DWORD,
    pub ProccntUsage: DWORD,
    pub modBaseAddr: *mut BYTE,
    pub modBaseSize: DWORD,
    pub hModule: HMODULE,
    pub szModule: [CHAR; MAX_MODULE_NAME32 + 1],
    pub szExePath: [CHAR; 260],
}
pub type MODULEENTRY32 = tagMODULEENTRY32;
pub type PMODULEENTRY32 = *mut tagMODULEENTRY32;
pub type LPMODULEENTRY32 = *mut tagMODULEENTRY32;

#[repr(C)]
pub struct tagTHREADENTRY32 {
    pub dwSize: DWORD,
    pub cntUsage: DWORD,
    pub th32ThreadID: DWORD,
    pub th32OwnerProcessID: DWORD,
    pub tpBasePri: LONG,
    pub tpDeltaPri: LONG,
    pub dwFlags: DWORD,
}
pub type THREADENTRY32 = tagTHREADENTRY32;
pub type PTHREADENTRY32 = *mut tagTHREADENTRY32;
pub type LPTHREADENTRY32 = *mut tagTHREADENTRY32;

// toolhelp.c: `#define OffsetToPtr(Snapshot, Offset) ((ULONG_PTR)((Snapshot) + 1) + (ULONG_PTR)(Offset))`.
// `Snapshot + 1` is pointer arithmetic on PTH32SNAPSHOT: the lists start AFTER the header.
// This used to be `base + offset` (reading the header as entry 0), while the writers below
// computed `base + 1 BYTE + offset` (writing the entries over the header) — so every snapshot
// came back with its counts clobbered and Process32First failed with ERROR_NO_MORE_FILES.
// Typed on purpose: `Snapshot + 1` must step over a whole TH32SNAPSHOT, and a generic or
// `c_void` parameter makes `.add(1)` step one byte instead (it did, in Process32NextW).
unsafe fn OffsetToPtr(snapshot: PTH32SNAPSHOT, offset: ULONG_PTR) -> PVOID {
    (snapshot.add(1) as *const u8).add(offset as usize) as PVOID
}

// intra-DLL kernel32 exports (resolved to #[no_mangle] symbols in sibling modules at link)
extern "system" {
    fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;
    fn ReadProcessMemory(
        hProcess: HANDLE,
        lpBaseAddress: LPCVOID,
        lpBuffer: LPVOID,
        nSize: SIZE_T,
        lpNumberOfBytesRead: *mut SIZE_T,
    ) -> BOOL;
    fn GetCurrentProcessId() -> DWORD;
    fn lstrcpyW(lpString1: LPWSTR, lpString2: LPCWSTR) -> LPWSTR;
    fn MultiByteToWideChar(
        CodePage: UINT,
        Flags: DWORD,
        MultiByteString: LPCSTR,
        MultiByteCount: INT,
        WideCharString: LPWSTR,
        WideCharCount: INT,
    ) -> INT;
    fn WideCharToMultiByte(
        CodePage: UINT,
        Flags: DWORD,
        WideCharString: LPCWSTR,
        WideCharCount: INT,
        MultiByteString: LPSTR,
        MultiByteCount: INT,
        DefaultChar: LPCSTR,
        UsedDefaultChar: LPBOOL,
    ) -> INT;
}

// ---- file-scope preamble (translated) ----
#[repr(C)]
pub struct _RTLP_HEAP_ENTRY {
    pub Size: ULONG,
    pub Flags: USHORT,
    pub Unknown1: USHORT,
    pub Unknown2: ULONG,
    pub Address: PVOID,
}
pub type RTLP_HEAP_ENTRY = _RTLP_HEAP_ENTRY;
pub type PRTLP_HEAP_ENTRY = *mut _RTLP_HEAP_ENTRY;

#[repr(C)]
pub struct _TH32SNAPSHOT {
    pub HeapListCount: ULONG,
    pub HeapListIndex: ULONG,
    pub HeapListOffset: ULONG_PTR,
    pub ModuleListCount: ULONG,
    pub ModuleListIndex: ULONG,
    pub ModuleListOffset: ULONG_PTR,
    pub ProcessListCount: ULONG,
    pub ProcessListIndex: ULONG,
    pub ProcessListOffset: ULONG_PTR,
    pub ThreadListCount: ULONG,
    pub ThreadListIndex: ULONG,
    pub ThreadListOffset: ULONG_PTR,
}
pub type TH32SNAPSHOT = _TH32SNAPSHOT;
pub type PTH32SNAPSHOT = *mut _TH32SNAPSHOT;
unsafe fn TH32FreeAllocatedResources(
    HeapDebug: PRTL_DEBUG_INFORMATION,
    ModuleDebug: PRTL_DEBUG_INFORMATION,
    mut ProcThrdInfo: PVOID,
    mut ProcThrdInfoSize: SIZE_T,
) {
    if !HeapDebug.is_null() {
        RtlDestroyQueryDebugBuffer(HeapDebug);
    }
    if !ModuleDebug.is_null() {
        RtlDestroyQueryDebugBuffer(ModuleDebug);
    }
    if !ProcThrdInfo.is_null() {
        NtFreeVirtualMemory(
            NtCurrentProcess(),
            &raw mut ProcThrdInfo,
            &raw mut ProcThrdInfoSize,
            MEM_RELEASE,
        );
    }
}
unsafe fn TH32CreateSnapshot(
    dwFlags: DWORD,
    th32ProcessID: DWORD,
    HeapDebug: *mut PRTL_DEBUG_INFORMATION,
    ModuleDebug: *mut PRTL_DEBUG_INFORMATION,
    ProcThrdInfo: *mut PVOID,
    ProcThrdInfoSize: *mut SIZE_T,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    *HeapDebug = core::ptr::null_mut();
    *ModuleDebug = core::ptr::null_mut();
    *ProcThrdInfo = core::ptr::null_mut();
    *ProcThrdInfoSize = 0;

    if dwFlags & TH32CS_SNAPHEAPLIST != 0 {
        *HeapDebug = RtlCreateQueryDebugBuffer(0, 0);
        if !(*HeapDebug).is_null() {
            Status = RtlQueryProcessDebugInformation(
                th32ProcessID,
                RTL_DEBUG_QUERY_HEAPS,
                *HeapDebug,
            );
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }
    }

    if dwFlags & TH32CS_SNAPMODULE != 0 && NT_SUCCESS(Status) {
        *ModuleDebug = RtlCreateQueryDebugBuffer(0, 0);
        if !(*ModuleDebug).is_null() {
            Status = RtlQueryProcessDebugInformation(
                th32ProcessID,
                RTL_DEBUG_QUERY_MODULES,
                *ModuleDebug,
            );
        } else {
            Status = STATUS_UNSUCCESSFUL;
        }
    }

    if dwFlags & (TH32CS_SNAPPROCESS | TH32CS_SNAPTHREAD) != 0 && NT_SUCCESS(Status) {
        loop {
            *ProcThrdInfoSize = (*ProcThrdInfoSize).wrapping_add(0x10000);
            Status = NtAllocateVirtualMemory(
                NtCurrentProcess(),
                ProcThrdInfo,
                0,
                ProcThrdInfoSize,
                MEM_COMMIT,
                PAGE_READWRITE,
            );
            if !NT_SUCCESS(Status) {
                break;
            }

            Status = NtQuerySystemInformation(
                SystemProcessInformation,
                *ProcThrdInfo,
                *ProcThrdInfoSize as ULONG,
                core::ptr::null_mut(),
            );
            if Status == STATUS_INFO_LENGTH_MISMATCH {
                NtFreeVirtualMemory(
                    NtCurrentProcess(),
                    ProcThrdInfo,
                    ProcThrdInfoSize,
                    MEM_RELEASE,
                );
                *ProcThrdInfo = core::ptr::null_mut();
            } else {
                break;
            }
        }
    }

    if !NT_SUCCESS(Status) {
        TH32FreeAllocatedResources(
            *HeapDebug,
            *ModuleDebug,
            *ProcThrdInfo,
            *ProcThrdInfoSize,
        );
    }

    Status
}
unsafe fn TH32CreateSnapshotSectionInitialize(
    dwFlags: DWORD,
    th32ProcessID: DWORD,
    HeapDebug: PRTL_DEBUG_INFORMATION,
    ModuleDebug: PRTL_DEBUG_INFORMATION,
    ProcThrdInfo: PVOID,
    SectionHandle: *mut HANDLE,
) -> NTSTATUS {
    let mut ProcessInfo: PSYSTEM_PROCESS_INFORMATION = core::ptr::null_mut();
    let mut HeapListEntry: LPHEAPLIST32 = core::ptr::null_mut();
    let mut ModuleListEntry: LPMODULEENTRY32W = core::ptr::null_mut();
    let mut ProcessListEntry: LPPROCESSENTRY32W = core::ptr::null_mut();
    let mut ThreadListEntry: LPTHREADENTRY32 = core::ptr::null_mut();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut SSize: LARGE_INTEGER = core::mem::zeroed();
    let mut SOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut hSection: HANDLE = core::ptr::null_mut();
    let mut Snapshot: PTH32SNAPSHOT = core::ptr::null_mut();
    let mut DataOffset: ULONG_PTR = 0;
    let mut ViewSize: SIZE_T = 0;
    let mut i: ULONG = 0;
    let mut nProcesses: ULONG = 0;
    let mut nThreads: ULONG = 0;
    let mut nHeaps: ULONG = 0;
    let mut nModules: ULONG = 0;
    let mut RequiredSnapshotSize: ULONG = core::mem::size_of::<TH32SNAPSHOT>() as ULONG;
    let mut hi: PRTL_PROCESS_HEAPS = core::ptr::null_mut();
    let mut mi: PRTL_PROCESS_MODULES = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    if dwFlags & TH32CS_SNAPHEAPLIST != 0 {
        hi = (*HeapDebug).Heaps;
        nHeaps = (*hi).NumberOfHeaps;
        RequiredSnapshotSize = RequiredSnapshotSize.wrapping_add(
            (nHeaps as usize).wrapping_mul(core::mem::size_of::<HEAPLIST32>()) as ULONG,
        );
    }

    if dwFlags & TH32CS_SNAPMODULE != 0 {
        mi = (*ModuleDebug).__bindgen_anon_1.Modules;
        nModules = (*mi).NumberOfModules;
        RequiredSnapshotSize = RequiredSnapshotSize.wrapping_add(
            (nModules as usize).wrapping_mul(core::mem::size_of::<MODULEENTRY32W>()) as ULONG,
        );
    }

    if dwFlags & (TH32CS_SNAPPROCESS | TH32CS_SNAPTHREAD) != 0 {
        let mut ProcOffset: ULONG = 0;

        ProcessInfo = ProcThrdInfo as PSYSTEM_PROCESS_INFORMATION;
        loop {
            ProcessInfo = (ProcessInfo as ULONG_PTR).wrapping_add(ProcOffset as ULONG_PTR)
                as PSYSTEM_PROCESS_INFORMATION;
            nProcesses = nProcesses.wrapping_add(1);
            nThreads = nThreads.wrapping_add((*ProcessInfo).NumberOfThreads);
            ProcOffset = (*ProcessInfo).NextEntryOffset;
            if ProcOffset == 0 {
                break;
            }
        }

        if dwFlags & TH32CS_SNAPPROCESS != 0 {
            RequiredSnapshotSize = RequiredSnapshotSize.wrapping_add(
                (nProcesses as usize).wrapping_mul(core::mem::size_of::<PROCESSENTRY32W>()) as ULONG,
            );
        }
        if dwFlags & TH32CS_SNAPTHREAD != 0 {
            RequiredSnapshotSize = RequiredSnapshotSize.wrapping_add(
                (nThreads as usize).wrapping_mul(core::mem::size_of::<THREADENTRY32>()) as ULONG,
            );
        }
    }

    SSize.QuadPart = RequiredSnapshotSize as LONGLONG;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    ObjectAttributes.RootDirectory = core::ptr::null_mut();
    ObjectAttributes.Attributes = if dwFlags & TH32CS_INHERIT != 0 {
        OBJ_INHERIT
    } else {
        0
    };
    ObjectAttributes.ObjectName = core::ptr::null_mut();
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    Status = NtCreateSection(
        &mut hSection,
        SECTION_ALL_ACCESS,
        &mut ObjectAttributes,
        &mut SSize,
        PAGE_READWRITE,
        SEC_COMMIT,
        core::ptr::null_mut(),
    );
    if !NT_SUCCESS(Status) {
        return Status;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSection,
        NtCurrentProcess(),
        &mut Snapshot as *mut PTH32SNAPSHOT as *mut PVOID,
        0,
        0,
        &mut SOffset,
        &mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE,
    );
    if !NT_SUCCESS(Status) {
        NtClose(hSection);
        return Status;
    }

    RtlZeroMemory(Snapshot as PVOID, core::mem::size_of::<TH32SNAPSHOT>() as SIZE_T);
    DataOffset = 0;

    if dwFlags & TH32CS_SNAPHEAPLIST != 0 {
        (*Snapshot).HeapListCount = nHeaps;
        (*Snapshot).HeapListOffset = DataOffset;
        HeapListEntry = OffsetToPtr(Snapshot, DataOffset) as LPHEAPLIST32;
        i = 0;
        while i < nHeaps {
            (*HeapListEntry).dwSize = core::mem::size_of::<HEAPLIST32>() as DWORD;
            (*HeapListEntry).th32ProcessID = th32ProcessID;
            // RTL_PROCESS_MODULES.Modules is `Modules[1]  /* FIXME: should be
            // Modules[0] */` and RTL_HEAP_INFORMATION_LIST.Heaps is the same shape:
            // flexible arrays bounded by ModulesCount / NumberOfHeaps. `[0]` keeps the
            // element type derived rather than named.
            (*HeapListEntry).th32HeapID = (*(&raw mut (*hi).Heaps[0]).add(i as usize)).BaseAddress as ULONG_PTR as DWORD;
            (*HeapListEntry).dwFlags = (*(&raw mut (*hi).Heaps[0]).add(i as usize)).Flags;

            HeapListEntry = HeapListEntry.offset(1);
            i = i.wrapping_add(1);
        }

        DataOffset = DataOffset.wrapping_add(
            ((*hi).NumberOfHeaps as usize).wrapping_mul(core::mem::size_of::<HEAPLIST32>()) as ULONG_PTR,
        );
    }

    if dwFlags & TH32CS_SNAPMODULE != 0 {
        (*Snapshot).ModuleListCount = nModules;
        (*Snapshot).ModuleListOffset = DataOffset;
        ModuleListEntry = OffsetToPtr(Snapshot, DataOffset) as LPMODULEENTRY32W;
        i = 0;
        while i < nModules {
            (*ModuleListEntry).dwSize = core::mem::size_of::<MODULEENTRY32W>() as DWORD;
            (*ModuleListEntry).th32ModuleID = 1;
            (*ModuleListEntry).th32ProcessID = th32ProcessID;
            (*ModuleListEntry).GlblcntUsage = (*(&raw mut (*mi).Modules[0]).add(i as usize)).LoadCount as DWORD;
            (*ModuleListEntry).ProccntUsage = (*(&raw mut (*mi).Modules[0]).add(i as usize)).LoadCount as DWORD;
            (*ModuleListEntry).modBaseAddr = (*(&raw mut (*mi).Modules[0]).add(i as usize)).ImageBase as *mut BYTE;
            (*ModuleListEntry).modBaseSize = (*(&raw mut (*mi).Modules[0]).add(i as usize)).ImageSize;
            (*ModuleListEntry).hModule = (*(&raw mut (*mi).Modules[0]).add(i as usize)).ImageBase as HMODULE;

            MultiByteToWideChar(
                CP_ACP,
                0,
                &(*(&raw mut (*mi).Modules[0]).add(i as usize)).FullPathName[(*(&raw mut (*mi).Modules[0]).add(i as usize)).OffsetToFileName as usize] as *const CHAR,
                -1,
                &mut (*ModuleListEntry).szModule as *mut WCHAR,
                (core::mem::size_of::<[WCHAR; 256]>() / core::mem::size_of::<WCHAR>()) as i32,
            );

            MultiByteToWideChar(
                CP_ACP,
                0,
                &(*(&raw mut (*mi).Modules[0]).add(i as usize)).FullPathName as *const CHAR,
                -1,
                &mut (*ModuleListEntry).szExePath as *mut WCHAR,
                (core::mem::size_of::<[WCHAR; 260]>() / core::mem::size_of::<WCHAR>()) as i32,
            );

            ModuleListEntry = ModuleListEntry.offset(1);
            i = i.wrapping_add(1);
        }

        DataOffset = DataOffset.wrapping_add(
            ((*mi).NumberOfModules as usize).wrapping_mul(core::mem::size_of::<MODULEENTRY32W>()) as ULONG_PTR,
        );
    }

    if dwFlags & TH32CS_SNAPPROCESS != 0 {
        let mut ProcOffset: ULONG = 0;

        (*Snapshot).ProcessListCount = nProcesses;
        (*Snapshot).ProcessListOffset = DataOffset;
        ProcessListEntry = OffsetToPtr(Snapshot, DataOffset) as LPPROCESSENTRY32W;
        ProcessInfo = ProcThrdInfo as PSYSTEM_PROCESS_INFORMATION;
        loop {
            ProcessInfo = (ProcessInfo as ULONG_PTR).wrapping_add(ProcOffset as ULONG_PTR)
                as PSYSTEM_PROCESS_INFORMATION;

            (*ProcessListEntry).dwSize = core::mem::size_of::<PROCESSENTRY32W>() as DWORD;
            (*ProcessListEntry).cntUsage = 0;
            (*ProcessListEntry).th32ProcessID = (*ProcessInfo).UniqueProcessId as ULONG_PTR as DWORD;
            (*ProcessListEntry).th32DefaultHeapID = 0;
            (*ProcessListEntry).th32ModuleID = 0;
            (*ProcessListEntry).cntThreads = (*ProcessInfo).NumberOfThreads;
            (*ProcessListEntry).th32ParentProcessID = (*ProcessInfo).InheritedFromUniqueProcessId as ULONG_PTR as DWORD;
            (*ProcessListEntry).pcPriClassBase = (*ProcessInfo).BasePriority as LONG;
            (*ProcessListEntry).dwFlags = 0;

            if !(*ProcessInfo).ImageName.Buffer.is_null() {
                let ExeFileLength: ULONG = min(
                    (*ProcessInfo).ImageName.Length as ULONG,
                    (core::mem::size_of::<[WCHAR; 260]>() - core::mem::size_of::<WCHAR>()) as ULONG,
                );
                RtlCopyMemory(
                    &mut (*ProcessListEntry).szExeFile as *mut WCHAR as PVOID,
                    (*ProcessInfo).ImageName.Buffer as PVOID,
                    ExeFileLength as SIZE_T,
                );
                (*ProcessListEntry).szExeFile[(ExeFileLength as usize / core::mem::size_of::<WCHAR>()) as usize] = UNICODE_NULL;
            } else {
                lstrcpyW(
                    &mut (*ProcessListEntry).szExeFile as *mut WCHAR,
                    SYSTEM_PROCESS.as_ptr(),
                );
            }

            ProcessListEntry = ProcessListEntry.offset(1);

            ProcOffset = (*ProcessInfo).NextEntryOffset;
            if ProcOffset == 0 {
                break;
            }
        }

        DataOffset = DataOffset.wrapping_add(
            (nProcesses as usize).wrapping_mul(core::mem::size_of::<PROCESSENTRY32W>()) as ULONG_PTR,
        );
    }

    if dwFlags & TH32CS_SNAPTHREAD != 0 {
        let mut ProcOffset: ULONG = 0;

        (*Snapshot).ThreadListCount = nThreads;
        (*Snapshot).ThreadListOffset = DataOffset;
        ThreadListEntry = OffsetToPtr(Snapshot, DataOffset) as LPTHREADENTRY32;
        ProcessInfo = ProcThrdInfo as PSYSTEM_PROCESS_INFORMATION;
        loop {
            let mut ThreadInfo: PSYSTEM_THREAD_INFORMATION;
            let mut n: ULONG;

            ProcessInfo = (ProcessInfo as ULONG_PTR).wrapping_add(ProcOffset as ULONG_PTR)
                as PSYSTEM_PROCESS_INFORMATION;
            ThreadInfo = (ProcessInfo as ULONG_PTR).wrapping_add(core::mem::size_of::<SYSTEM_PROCESS_INFORMATION>() as ULONG_PTR) as PSYSTEM_THREAD_INFORMATION;

            n = 0;
            while n < (*ProcessInfo).NumberOfThreads {
                (*ThreadListEntry).dwSize = core::mem::size_of::<THREADENTRY32>() as DWORD;
                (*ThreadListEntry).cntUsage = 0;
                (*ThreadListEntry).th32ThreadID = (*ThreadInfo).ClientId.UniqueThread as ULONG_PTR as DWORD;
                (*ThreadListEntry).th32OwnerProcessID = (*ThreadInfo).ClientId.UniqueProcess as ULONG_PTR as DWORD;
                (*ThreadListEntry).tpBasePri = (*ThreadInfo).BasePriority;
                (*ThreadListEntry).tpDeltaPri = 0;
                (*ThreadListEntry).dwFlags = 0;

                ThreadInfo = ThreadInfo.offset(1);
                ThreadListEntry = ThreadListEntry.offset(1);
                n = n.wrapping_add(1);
            }

            ProcOffset = (*ProcessInfo).NextEntryOffset;
            if ProcOffset == 0 {
                break;
            }
        }
    }

    Status = NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);

    if NT_SUCCESS(Status) {
        *SectionHandle = hSection;
    } else {
        NtClose(hSection);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn Heap32First(
    lphe: LPHEAPENTRY32,
    th32ProcessID: DWORD,
    th32HeapID: DWORD,
) -> BOOL {
    let mut DebugInfo: PRTL_DEBUG_INFORMATION = core::ptr::null_mut();
    let mut Heap: PRTL_HEAP_INFORMATION = core::ptr::null_mut();
    let mut Block: PRTLP_HEAP_ENTRY = core::ptr::null_mut();
    let mut LastBlock: PRTLP_HEAP_ENTRY = core::ptr::null_mut();
    let mut i: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    // CHECK_PARAM_SIZE(lphe, sizeof(HEAPENTRY32))
    if lphe.is_null() || (*lphe).dwSize as usize != core::mem::size_of::<HEAPENTRY32>() as usize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }

    DebugInfo = RtlCreateQueryDebugBuffer(0, FALSE as BOOLEAN);
    if DebugInfo.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY as DWORD);
        return FALSE;
    }

    Status = RtlQueryProcessDebugInformation(
        th32ProcessID,
        (RTL_DEBUG_QUERY_HEAPS | RTL_DEBUG_QUERY_HEAP_BLOCKS) as ULONG,
        DebugInfo,
    );

    if NT_SUCCESS(Status) {
        Status = STATUS_NO_MORE_FILES;
        i = 0;
        while i != (*((*DebugInfo).Heaps)).NumberOfHeaps {
            Heap = &mut (*(&raw mut (*((*DebugInfo).Heaps)).Heaps[0]).add(i as usize)) as *mut RTL_HEAP_INFORMATION as PRTL_HEAP_INFORMATION;

            if (*Heap).BaseAddress as ULONG_PTR == th32HeapID as ULONG_PTR {
                (*lphe).hHandle = (*Heap).BaseAddress;
                (*lphe).dwAddress = 0;
                (*lphe).dwBlockSize = 0;
                (*lphe).dwFlags = 0;
                (*lphe).dwLockCount = 0;
                (*lphe).dwResvd = 0;
                (*lphe).th32ProcessID = th32ProcessID;
                (*lphe).th32HeapID = (*Heap).BaseAddress as ULONG_PTR as DWORD;

                Block = (*Heap).Entries as PRTLP_HEAP_ENTRY;
                LastBlock = Block.offset((*Heap).NumberOfEntries as isize);

                while Block != LastBlock && ((*Block).Flags as ULONG & PROCESS_HEAP_UNCOMMITTED_RANGE) != 0 {
                    (*lphe).dwResvd = (*lphe).dwResvd.wrapping_add(1);
                    (*lphe).dwAddress = ((*Block).Address as ULONG_PTR)
                        .wrapping_add((*Heap).EntryOverhead as ULONG_PTR) as DWORD;
                    Block = Block.offset(1);
                }

                if Block != LastBlock && (*lphe).dwResvd != 0 {
                    (*lphe).dwBlockSize = (*Block).Size as DWORD;

                    if ((*Block).Flags & 0x2F1) != 0 {
                        (*lphe).dwFlags = LF32_FIXED as DWORD;
                    } else if ((*Block).Flags & 0x20) != 0 {
                        (*lphe).dwFlags = LF32_MOVEABLE as DWORD;
                    } else if ((*Block).Flags & 0x100) != 0 {
                        (*lphe).dwFlags = LF32_FREE as DWORD;
                    }

                    Status = STATUS_SUCCESS;
                }

                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }

    RtlDestroyQueryDebugBuffer(DebugInfo);

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return FALSE;
    }

    return TRUE;
}

#[no_mangle]
pub unsafe extern "system" fn Heap32Next(lphe: LPHEAPENTRY32) -> BOOL {
    let mut DebugInfo: PRTL_DEBUG_INFORMATION = core::ptr::null_mut();
    let mut Heap: PRTL_HEAP_INFORMATION = core::ptr::null_mut();
    let mut Block: PRTLP_HEAP_ENTRY = core::ptr::null_mut();
    let mut LastBlock: PRTLP_HEAP_ENTRY = core::ptr::null_mut();
    let mut FoundUncommitted: BOOLEAN = 0;
    let mut i: ULONG = 0;
    let mut Status: NTSTATUS = 0;

    if lphe.is_null() || (*lphe).dwSize as usize != core::mem::size_of::<HEAPENTRY32>() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    DebugInfo = RtlCreateQueryDebugBuffer(0, 0);
    if DebugInfo.is_null() {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return 0;
    }

    Status = RtlQueryProcessDebugInformation(
        (*lphe).th32ProcessID,
        (RTL_DEBUG_QUERY_HEAPS | RTL_DEBUG_QUERY_HEAP_BLOCKS) as ULONG,
        DebugInfo,
    );

    if NT_SUCCESS(Status) {
        Status = STATUS_NO_MORE_FILES;
        i = 0;
        while i != (*(*DebugInfo).Heaps).NumberOfHeaps {
            Heap = (&raw mut (*(*DebugInfo).Heaps).Heaps[0]).add(i as usize) as PRTL_HEAP_INFORMATION;

            if (*Heap).BaseAddress as ULONG_PTR == (*lphe).th32HeapID as ULONG_PTR {
                (*lphe).dwResvd = (*lphe).dwResvd.wrapping_add(1);
                if (*lphe).dwResvd < (*Heap).NumberOfEntries {
                    (*lphe).dwFlags = 0;

                    Block = ((*Heap).Entries as PRTLP_HEAP_ENTRY).offset((*lphe).dwResvd as isize);
                    LastBlock = ((*Heap).Entries as PRTLP_HEAP_ENTRY).offset((*Heap).NumberOfEntries as isize);

                    while Block < LastBlock && ((*Block).Flags as ULONG & PROCESS_HEAP_UNCOMMITTED_RANGE) != 0 {
                        (*lphe).dwResvd = (*lphe).dwResvd.wrapping_add(1);
                        (*lphe).dwAddress = ((*Block).Address as ULONG_PTR)
                            .wrapping_add((*Heap).EntryOverhead as ULONG_PTR) as DWORD;
                        FoundUncommitted = 1;
                        Block = Block.offset(1);
                    }

                    if Block < LastBlock {
                        if FoundUncommitted == 0 {
                            (*lphe).dwAddress = ((*lphe).dwAddress as ULONG_PTR)
                                .wrapping_add((*lphe).dwBlockSize as ULONG_PTR) as DWORD;
                        }

                        (*lphe).dwBlockSize = (*Block).Size as DWORD;

                        if ((*Block).Flags as i32 & 0x2F1) != 0 {
                            (*lphe).dwFlags = LF32_FIXED as DWORD;
                        } else if ((*Block).Flags as i32 & 0x20) != 0 {
                            (*lphe).dwFlags = LF32_MOVEABLE as DWORD;
                        } else if ((*Block).Flags as i32 & 0x100) != 0 {
                            (*lphe).dwFlags = LF32_FREE as DWORD;
                        }

                        Status = STATUS_SUCCESS;
                    }
                }
                break;
            } else {
                i = i.wrapping_add(1);
            }
        }
    }

    RtlDestroyQueryDebugBuffer(DebugInfo);

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return 0;
    }

    return 1;
}

#[no_mangle]
pub unsafe extern "system" fn Heap32ListFirst(
    hSnapshot: HANDLE,
    lphl: LPHEAPLIST32,
) -> BOOL {
    let mut Snapshot: PTH32SNAPSHOT = core::ptr::null_mut();
    let mut SOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut ViewSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;

    // CHECK_PARAM_SIZE expansion
    if lphl.is_null() || (*lphl).dwSize as usize != core::mem::size_of::<HEAPLIST32>() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSnapshot,
        NtCurrentProcess(),
        &mut Snapshot as *mut PTH32SNAPSHOT as *mut PVOID,
        0,
        0,
        &mut SOffset,
        &mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE,
    );

    if NT_SUCCESS(Status) {
        let Ret: BOOL;

        if (*Snapshot).HeapListCount > 0 {
            let Entries: LPHEAPLIST32 = OffsetToPtr(Snapshot, (*Snapshot).HeapListOffset) as LPHEAPLIST32;
            (*Snapshot).HeapListIndex = 1;
            RtlCopyMemory(
                lphl as *mut core::ffi::c_void,
                &(*Entries.offset(0)) as *const HEAPLIST32 as *const core::ffi::c_void,
                core::mem::size_of::<HEAPLIST32>() as SIZE_T,
            );
            Ret = 1;
        } else {
            SetLastError(ERROR_NO_MORE_FILES);
            Ret = 0;
        }

        NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);
        return Ret;
    }

    BaseSetLastNTError(Status);
    return 0;
}
#[no_mangle]
pub unsafe extern "system" fn Heap32ListNext(
    hSnapshot: HANDLE,
    lphl: LPHEAPLIST32,
) -> BOOL {
    let mut Snapshot: PTH32SNAPSHOT = core::ptr::null_mut();
    let mut SOffset: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: 0,
    };
    let mut ViewSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;

    // CHECK_PARAM_SIZE expansion
    if lphl.is_null() || (*lphl).dwSize as usize != core::mem::size_of::<HEAPLIST32>() as usize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSnapshot,
        NtCurrentProcess(),
        &mut Snapshot as *mut PTH32SNAPSHOT as *mut PVOID,
        0,
        0,
        &mut SOffset,
        &mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE,
    );

    if NT_SUCCESS(Status) {
        let mut Ret: BOOL = 0;

        if (*Snapshot).HeapListCount > 0
            && (*Snapshot).HeapListIndex < (*Snapshot).HeapListCount
        {
            let Entries: LPHEAPLIST32 = OffsetToPtr(
                Snapshot,
                (*Snapshot).HeapListOffset,
            ) as LPHEAPLIST32;

            let idx = (*Snapshot).HeapListIndex;
            (*Snapshot).HeapListIndex = (*Snapshot).HeapListIndex.wrapping_add(1);

            RtlCopyMemory(
                lphl as *mut core::ffi::c_void,
                &*Entries.add(idx as usize) as *const HEAPLIST32 as *const core::ffi::c_void,
                core::mem::size_of::<HEAPLIST32>() as SIZE_T,
            );
            Ret = TRUE;
        } else {
            SetLastError(ERROR_NO_MORE_FILES as DWORD);
            Ret = FALSE;
        }

        NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);
        return Ret;
    }

    BaseSetLastNTError(Status);
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn Module32First(
    hSnapshot: HANDLE,
    lpme: LPMODULEENTRY32,
) -> BOOL {
    let mut me: MODULEENTRY32W = core::mem::zeroed();
    let mut Ret: BOOL;

    if lpme.is_null() || ((*lpme).dwSize as usize) < core::mem::size_of::<MODULEENTRY32>() as usize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    me.dwSize = core::mem::size_of::<MODULEENTRY32W>() as DWORD;

    Ret = Module32FirstW(hSnapshot, &raw mut me);
    if Ret != 0 {
        (*lpme).th32ModuleID = me.th32ModuleID;
        (*lpme).th32ProcessID = me.th32ProcessID;
        (*lpme).GlblcntUsage = me.GlblcntUsage;
        (*lpme).ProccntUsage = me.ProccntUsage;
        (*lpme).modBaseAddr = me.modBaseAddr;
        (*lpme).modBaseSize = me.modBaseSize;
        (*lpme).hModule = me.hModule;

        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            &raw mut me.szModule as *mut WCHAR as LPCWSTR,
            -1,
            &raw mut (*lpme).szModule as LPSTR,
            core::mem::size_of::<[i8; 256]>() as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            &raw mut me.szExePath as *mut WCHAR as LPCWSTR,
            -1,
            &raw mut (*lpme).szExePath as LPSTR,
            core::mem::size_of::<[i8; 260]>() as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn Module32FirstW(
    hSnapshot: HANDLE,
    lpme: LPMODULEENTRY32W,
) -> BOOL {
    let mut Snapshot: PTH32SNAPSHOT = core::ptr::null_mut();
    let mut SOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut ViewSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;

    // CHECK_PARAM_SIZE expansion
    if lpme.is_null() || (*lpme).dwSize as usize != core::mem::size_of::<MODULEENTRY32W>() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return FALSE;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSnapshot,
        NtCurrentProcess(),
        &mut Snapshot as *mut PTH32SNAPSHOT as *mut PVOID,
        0,
        0,
        &mut SOffset,
        &mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE,
    );

    if NT_SUCCESS(Status) {
        let Ret: BOOL;

        if (*Snapshot).ModuleListCount > 0 {
            let Entries: LPMODULEENTRY32W = OffsetToPtr(Snapshot, (*Snapshot).ModuleListOffset) as LPMODULEENTRY32W;
            (*Snapshot).ModuleListIndex = 1;
            RtlCopyMemory(
                lpme as *mut core::ffi::c_void,
                &(*Entries.offset(0)) as *const MODULEENTRY32W as *const core::ffi::c_void,
                core::mem::size_of::<MODULEENTRY32W>() as SIZE_T,
            );
            Ret = TRUE;
        } else {
            SetLastError(ERROR_NO_MORE_FILES);
            Ret = FALSE;
        }

        NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);
        return Ret;
    }

    BaseSetLastNTError(Status);
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn Module32Next(
    hSnapshot: HANDLE,
    lpme: LPMODULEENTRY32,
) -> BOOL {
    let mut me: MODULEENTRY32W = core::mem::zeroed();
    let mut Ret: BOOL;

    if lpme.is_null() || ((*lpme).dwSize as usize) < core::mem::size_of::<MODULEENTRY32>() {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    me.dwSize = core::mem::size_of::<MODULEENTRY32W>() as DWORD;

    Ret = Module32NextW(hSnapshot, &raw mut me);
    if Ret != 0 {
        (*lpme).th32ModuleID = me.th32ModuleID;
        (*lpme).th32ProcessID = me.th32ProcessID;
        (*lpme).GlblcntUsage = me.GlblcntUsage;
        (*lpme).ProccntUsage = me.ProccntUsage;
        (*lpme).modBaseAddr = me.modBaseAddr;
        (*lpme).modBaseSize = me.modBaseSize;
        (*lpme).hModule = me.hModule;

        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            &raw mut me.szModule as *mut WCHAR as LPCWSTR,
            -1,
            &raw mut (*lpme).szModule as LPSTR,
            core::mem::size_of::<[i8; 256]>() as i32,
            core::ptr::null::<i8>(),
            core::ptr::null_mut::<BOOL>(),
        );
        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            &raw mut me.szExePath as *mut WCHAR as LPCWSTR,
            -1,
            &raw mut (*lpme).szExePath as LPSTR,
            core::mem::size_of::<[i8; 260]>() as i32,
            core::ptr::null::<i8>(),
            core::ptr::null_mut::<BOOL>(),
        );
    }

    Ret
}

#[no_mangle]
pub unsafe extern "system" fn Module32NextW(
    hSnapshot: HANDLE,
    lpme: LPMODULEENTRY32W,
) -> BOOL {
    let mut Snapshot: PTH32SNAPSHOT = core::ptr::null_mut();
    let mut SOffset: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: 0,
    };
    let mut ViewSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;

    if lpme.is_null() || (*lpme).dwSize as usize != core::mem::size_of::<MODULEENTRY32W>() as usize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSnapshot,
        NtCurrentProcess(),
        &raw mut Snapshot as *mut PVOID,
        0,
        0,
        &raw mut SOffset,
        &raw mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE,
    );

    if NT_SUCCESS(Status) {
        let mut Ret: BOOL;

        if (*Snapshot).ModuleListCount > 0
            && (*Snapshot).ModuleListIndex < (*Snapshot).ModuleListCount
        {
            let Entries: LPMODULEENTRY32W = OffsetToPtr(Snapshot, (*Snapshot).ModuleListOffset) as LPMODULEENTRY32W;
            let idx = (*Snapshot).ModuleListIndex;
            (*Snapshot).ModuleListIndex = (*Snapshot).ModuleListIndex.wrapping_add(1);
            RtlCopyMemory(
                lpme as *mut core::ffi::c_void,
                Entries.offset(idx as isize) as *const core::ffi::c_void,
                core::mem::size_of::<MODULEENTRY32W>() as SIZE_T,
            );
            Ret = TRUE;
        } else {
            SetLastError(ERROR_NO_MORE_FILES as DWORD);
            Ret = FALSE;
        }

        NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);
        return Ret;
    }

    BaseSetLastNTError(Status);
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn Process32First(
    hSnapshot: HANDLE,
    lppe: LPPROCESSENTRY32,
) -> BOOL {
    let mut pe: PROCESSENTRY32W = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    if lppe.is_null() || ((*lppe).dwSize as usize) < core::mem::size_of::<PROCESSENTRY32>() as usize
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    pe.dwSize = core::mem::size_of::<PROCESSENTRY32W>() as DWORD;

    Ret = Process32FirstW(hSnapshot, &mut pe);
    if Ret != 0 {
        (*lppe).cntUsage = pe.cntUsage;
        (*lppe).th32ProcessID = pe.th32ProcessID;
        (*lppe).th32DefaultHeapID = pe.th32DefaultHeapID;
        (*lppe).th32ModuleID = pe.th32ModuleID;
        (*lppe).cntThreads = pe.cntThreads;
        (*lppe).th32ParentProcessID = pe.th32ParentProcessID;
        (*lppe).pcPriClassBase = pe.pcPriClassBase;
        (*lppe).dwFlags = pe.dwFlags;

        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            &mut pe.szExeFile as *mut WCHAR as LPCWSTR,
            -1i32,
            &mut (*lppe).szExeFile as LPSTR,
            core::mem::size_of::<[CHAR; 260]>() as i32,
            core::ptr::null::<CHAR>(),
            core::ptr::null_mut::<BOOL>(),
        );
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn Process32FirstW(
    hSnapshot: HANDLE,
    lppe: LPPROCESSENTRY32W,
) -> BOOL {
    let mut Snapshot: PTH32SNAPSHOT = core::ptr::null_mut();
    let mut SOffset: LARGE_INTEGER = LARGE_INTEGER {
        QuadPart: 0,
    };
    let mut ViewSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;

    if lppe.is_null() || (*lppe).dwSize as usize != core::mem::size_of::<PROCESSENTRY32W>() as usize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSnapshot,
        NtCurrentProcess(),
        &raw mut Snapshot as *mut PVOID,
        0,
        0,
        &raw mut SOffset,
        &raw mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE as ULONG,
    );

    if NT_SUCCESS(Status) {
        let Ret: BOOL;

        if (*Snapshot).ProcessListCount > 0 {
            let Entries: LPPROCESSENTRY32W = OffsetToPtr(Snapshot, (*Snapshot).ProcessListOffset) as LPPROCESSENTRY32W;

            (*Snapshot).ProcessListIndex = 1;
            RtlCopyMemory(
                lppe as *mut core::ffi::c_void,
                &*Entries.offset(0) as *const PROCESSENTRY32W as *const core::ffi::c_void,
                core::mem::size_of::<PROCESSENTRY32W>() as SIZE_T,
            );
            Ret = TRUE;
        } else {
            SetLastError(ERROR_NO_MORE_FILES as DWORD);
            Ret = FALSE;
        }

        NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);
        return Ret;
    }

    BaseSetLastNTError(Status);
    return FALSE;
}

#[no_mangle]
pub unsafe extern "system" fn Process32Next(
    hSnapshot: HANDLE,
    lppe: LPPROCESSENTRY32,
) -> BOOL {
    let mut pe: PROCESSENTRY32W = core::mem::zeroed();
    let mut Ret: BOOL = 0;

    if lppe.is_null() || ((*lppe).dwSize as usize) < core::mem::size_of::<PROCESSENTRY32>() as usize
    {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return 0;
    }

    pe.dwSize = core::mem::size_of::<PROCESSENTRY32W>() as DWORD;

    Ret = Process32NextW(hSnapshot, &mut pe);
    if Ret != 0 {
        (*lppe).cntUsage = pe.cntUsage;
        (*lppe).th32ProcessID = pe.th32ProcessID;
        (*lppe).th32DefaultHeapID = pe.th32DefaultHeapID;
        (*lppe).th32ModuleID = pe.th32ModuleID;
        (*lppe).cntThreads = pe.cntThreads;
        (*lppe).th32ParentProcessID = pe.th32ParentProcessID;
        (*lppe).pcPriClassBase = pe.pcPriClassBase;
        (*lppe).dwFlags = pe.dwFlags;

        WideCharToMultiByte(
            CP_ACP as UINT,
            0 as DWORD,
            pe.szExeFile.as_ptr() as LPCWSTR,
            -1,
            (*lppe).szExeFile.as_mut_ptr() as LPSTR,
            core::mem::size_of::<[CHAR; 260]>() as i32,
            core::ptr::null(),
            core::ptr::null_mut(),
        );
    }

    return Ret;
}

#[no_mangle]
pub unsafe extern "system" fn Process32NextW(
    hSnapshot: HANDLE,
    lppe: LPPROCESSENTRY32W,
) -> BOOL {
    let mut Snapshot: PTH32SNAPSHOT;
    let mut SOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ViewSize: SIZE_T;
    let mut Status: NTSTATUS;

    if lppe.is_null() || (*lppe).dwSize as usize != core::mem::size_of::<PROCESSENTRY32W>() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSnapshot,
        NtCurrentProcess(),
        &mut Snapshot as *mut PTH32SNAPSHOT as *mut PVOID,
        0,
        0,
        &mut SOffset,
        &mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE,
    );
    if NT_SUCCESS(Status) {
        let Ret: BOOL;
        if (*Snapshot).ProcessListCount > 0
            && (*Snapshot).ProcessListIndex < (*Snapshot).ProcessListCount
        {
            let Entries: LPPROCESSENTRY32W = OffsetToPtr(
                Snapshot,
                (*Snapshot).ProcessListOffset,
            ) as LPPROCESSENTRY32W;
            let idx = (*Snapshot).ProcessListIndex;
            (*Snapshot).ProcessListIndex = (*Snapshot).ProcessListIndex.wrapping_add(1);
            RtlCopyMemory(
                lppe as *mut ::core::ffi::c_void,
                &*Entries.add(idx as usize) as *const PROCESSENTRY32W as *const ::core::ffi::c_void,
                core::mem::size_of::<PROCESSENTRY32W>() as SIZE_T,
            );
            Ret = 1;
        } else {
            SetLastError(ERROR_NO_MORE_FILES);
            Ret = 0;
        }
        NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);
        return Ret;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn Thread32First(
    hSnapshot: HANDLE,
    lpte: LPTHREADENTRY32,
) -> BOOL {
    let mut Snapshot: PTH32SNAPSHOT = core::ptr::null_mut();
    let mut SOffset: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut ViewSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;

    // CHECK_PARAM_SIZE(lpte, sizeof(THREADENTRY32));
    if lpte.is_null() || (*lpte).dwSize as usize != core::mem::size_of::<THREADENTRY32>() {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSnapshot,
        NtCurrentProcess(),
        &raw mut Snapshot as *mut PVOID,
        0,
        0,
        &raw mut SOffset,
        &raw mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE,
    );
    if NT_SUCCESS(Status) {
        let Ret: BOOL;
        if (*Snapshot).ThreadListCount > 0 {
            let Entries: LPTHREADENTRY32 = OffsetToPtr(Snapshot, (*Snapshot).ThreadListOffset) as LPTHREADENTRY32;
            (*Snapshot).ThreadListIndex = 1;
            RtlCopyMemory(
                lpte as *mut core::ffi::c_void,
                &*Entries.offset(0) as *const THREADENTRY32 as *const core::ffi::c_void,
                core::mem::size_of::<THREADENTRY32>() as SIZE_T,
            );
            Ret = 1;
        } else {
            SetLastError(ERROR_NO_MORE_FILES);
            Ret = 0;
        }
        NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);
        return Ret;
    }

    BaseSetLastNTError(Status);
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn Thread32Next(
    hSnapshot: HANDLE,
    lpte: LPTHREADENTRY32,
) -> BOOL {
    let mut Snapshot: PTH32SNAPSHOT = core::ptr::null_mut();
    let mut SOffset: LARGE_INTEGER = core::mem::zeroed();
    let mut ViewSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;

    if lpte.is_null() || (*lpte).dwSize as usize != core::mem::size_of::<THREADENTRY32>() as usize {
        SetLastError(ERROR_INVALID_PARAMETER as DWORD);
        return FALSE;
    }

    SOffset.QuadPart = 0;
    ViewSize = 0;
    Snapshot = core::ptr::null_mut();

    Status = NtMapViewOfSection(
        hSnapshot,
        NtCurrentProcess(),
        &mut Snapshot as *mut PTH32SNAPSHOT as *mut PVOID,
        0,
        0,
        &mut SOffset,
        &mut ViewSize,
        ViewShare,
        0,
        PAGE_READWRITE,
    );

    if NT_SUCCESS(Status) {
        let Ret: BOOL;

        if (*Snapshot).ThreadListCount > 0
            && (*Snapshot).ThreadListIndex < (*Snapshot).ThreadListCount
        {
            let Entries: LPTHREADENTRY32 = OffsetToPtr(
                Snapshot,
                (*Snapshot).ThreadListOffset,
            ) as LPTHREADENTRY32;

            let fresh3 = (*Snapshot).ThreadListIndex;
            (*Snapshot).ThreadListIndex = (*Snapshot).ThreadListIndex.wrapping_add(1);

            RtlCopyMemory(
                lpte as *mut core::ffi::c_void,
                Entries.offset(fresh3 as isize) as *const core::ffi::c_void,
                core::mem::size_of::<THREADENTRY32>() as SIZE_T,
            );

            Ret = TRUE;
        } else {
            SetLastError(ERROR_NO_MORE_FILES as DWORD);
            Ret = FALSE;
        }

        NtUnmapViewOfSection(NtCurrentProcess(), Snapshot as PVOID);
        return Ret;
    }

    BaseSetLastNTError(Status);
    return FALSE;
}
#[no_mangle]
pub unsafe extern "system" fn Toolhelp32ReadProcessMemory(
    th32ProcessID: DWORD,
    lpBaseAddress: LPCVOID,
    lpBuffer: LPVOID,
    cbRead: SIZE_T,
    lpNumberOfBytesRead: *mut SIZE_T,
) -> BOOL {
    let hProcess: HANDLE = OpenProcess(PROCESS_VM_READ, 0, th32ProcessID);
    if !hProcess.is_null() {
        let Ret: BOOL = ReadProcessMemory(
            hProcess,
            lpBaseAddress,
            lpBuffer,
            cbRead,
            lpNumberOfBytesRead,
        );
        CloseHandle(hProcess);
        return Ret;
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn CreateToolhelp32Snapshot(
    dwFlags: DWORD,
    th32ProcessID: DWORD,
) -> HANDLE {
    let mut HeapDebug: PRTL_DEBUG_INFORMATION = core::ptr::null_mut();
    let mut ModuleDebug: PRTL_DEBUG_INFORMATION = core::ptr::null_mut();
    let mut ProcThrdInfo: PVOID = core::ptr::null_mut();
    let mut ProcThrdInfoSize: SIZE_T = 0;
    let mut Status: NTSTATUS = 0;
    let mut hSnapShotSection: HANDLE = core::ptr::null_mut();

    let mut th32ProcessID = th32ProcessID;
    if th32ProcessID == 0 {
        th32ProcessID = GetCurrentProcessId();
    }

    Status = TH32CreateSnapshot(
        dwFlags,
        th32ProcessID,
        &mut HeapDebug as *mut PRTL_DEBUG_INFORMATION,
        &mut ModuleDebug as *mut PRTL_DEBUG_INFORMATION,
        &mut ProcThrdInfo as *mut PVOID,
        &mut ProcThrdInfoSize as *mut SIZE_T,
    );

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    Status = TH32CreateSnapshotSectionInitialize(
        dwFlags,
        th32ProcessID,
        HeapDebug,
        ModuleDebug,
        ProcThrdInfo,
        &mut hSnapShotSection as *mut HANDLE,
    );

    TH32FreeAllocatedResources(HeapDebug, ModuleDebug, ProcThrdInfo, ProcThrdInfoSize);

    if !NT_SUCCESS(Status) {
        BaseSetLastNTError(Status);
        return core::ptr::null_mut();
    }

    hSnapShotSection
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS system libraries
//  * FILE:            dll/win32/kernel32/client/toolhelp.c
//  * PURPOSE:         Toolhelp functions
//  * PROGRAMMER:      Thomas Weidenmueller <w3seek@reactos.com>
//  *                  Robert Dickenson (robd@mok.lvcm.com)
//  *
//  * NOTES:           Do NOT use the heap functions in here because they
//  *                  adulterate the heap statistics!
//  *
//  * UPDATE HISTORY:
//  *                  10/30/2004 Implemented some parts (w3)
//  *                             Inspired by the book "Windows NT Native API"
//  *                  Created 05 January 2003 (robd)
//  */
//
// #include <k32.h>
//
// #define NDEBUG
// #include <debug.h>
//
// /* INTERNAL DEFINITIONS *******************************************************/
//
// typedef struct _RTLP_HEAP_ENTRY
// {
//     ULONG Size;
//     USHORT Flags;
//     USHORT Unknown1; /* FIXME */
//     ULONG Unknown2; /* FIXME */
//     PVOID Address;
// } RTLP_HEAP_ENTRY, *PRTLP_HEAP_ENTRY;
//
// #define CHECK_PARAM_SIZE(ptr, siz)                                             \
//   if((ptr) == NULL || (ptr)->dwSize != (siz))                                  \
//   {                                                                            \
//     SetLastError(ERROR_INVALID_PARAMETER);                                     \
//     return FALSE;                                                              \
//   }
//
// /*
//  * Tests in win showed that the dwSize field can be greater than the actual size
//  * of the structure for the ansi functions. I found this out by accidently
//  * forgetting to set the dwSize field in a test application and it just didn't
//  * work in ros but in win.
//  */
//
// #define CHECK_PARAM_SIZEA(ptr, siz)                                            \
//   if((ptr) == NULL || (ptr)->dwSize < (siz))                                   \
//   {                                                                            \
//     SetLastError(ERROR_INVALID_PARAMETER);                                     \
//     return FALSE;                                                              \
//   }
//
// #define OffsetToPtr(Snapshot, Offset)                                          \
//   ((ULONG_PTR)((Snapshot) + 1) + (ULONG_PTR)(Offset))
//
// typedef struct _TH32SNAPSHOT
// {
//   /* Heap list */
//   ULONG HeapListCount;
//   ULONG HeapListIndex;
//   ULONG_PTR HeapListOffset;
//   /* Module list */
//   ULONG ModuleListCount;
//   ULONG ModuleListIndex;
//   ULONG_PTR ModuleListOffset;
//   /* Process list */
//   ULONG ProcessListCount;
//   ULONG ProcessListIndex;
//   ULONG_PTR ProcessListOffset;
//   /* Thread list */
//   ULONG ThreadListCount;
//   ULONG ThreadListIndex;
//   ULONG_PTR ThreadListOffset;
// } TH32SNAPSHOT, *PTH32SNAPSHOT;
//
// /* INTERNAL FUNCTIONS *********************************************************/
//
// static VOID
// TH32FreeAllocatedResources(PRTL_DEBUG_INFORMATION HeapDebug,
//                            PRTL_DEBUG_INFORMATION ModuleDebug,
//                            PVOID ProcThrdInfo,
//                            SIZE_T ProcThrdInfoSize)
// {
//   if(HeapDebug != NULL)
//   {
//     RtlDestroyQueryDebugBuffer(HeapDebug);
//   }
//   if(ModuleDebug != NULL)
//   {
//     RtlDestroyQueryDebugBuffer(ModuleDebug);
//   }
//
//   if(ProcThrdInfo != NULL)
//   {
//     NtFreeVirtualMemory(NtCurrentProcess(),
//                         &ProcThrdInfo,
//                         &ProcThrdInfoSize,
//                         MEM_RELEASE);
//   }
// }
//
// static NTSTATUS
// TH32CreateSnapshot(DWORD dwFlags,
//                    DWORD th32ProcessID,
//                    PRTL_DEBUG_INFORMATION *HeapDebug,
//                    PRTL_DEBUG_INFORMATION *ModuleDebug,
//                    PVOID *ProcThrdInfo,
//                    SIZE_T *ProcThrdInfoSize)
// {
//   NTSTATUS Status = STATUS_SUCCESS;
//
//   *HeapDebug = NULL;
//   *ModuleDebug = NULL;
//   *ProcThrdInfo = NULL;
//   *ProcThrdInfoSize = 0;
//
//   /*
//    * Allocate the debug information for a heap snapshot
//    */
//   if(dwFlags & TH32CS_SNAPHEAPLIST)
//   {
//     *HeapDebug = RtlCreateQueryDebugBuffer(0, FALSE);
//     if(*HeapDebug != NULL)
//     {
//       Status = RtlQueryProcessDebugInformation(th32ProcessID,
//                                                RTL_DEBUG_QUERY_HEAPS,
//                                                *HeapDebug);
//     }
//     else
//       Status = STATUS_UNSUCCESSFUL;
//   }
//
//   /*
//    * Allocate the debug information for a module snapshot
//    */
//   if(dwFlags & TH32CS_SNAPMODULE &&
//      NT_SUCCESS(Status))
//   {
//     *ModuleDebug = RtlCreateQueryDebugBuffer(0, FALSE);
//     if(*ModuleDebug != NULL)
//     {
//       Status = RtlQueryProcessDebugInformation(th32ProcessID,
//                                                RTL_DEBUG_QUERY_MODULES,
//                                                *ModuleDebug);
//     }
//     else
//       Status = STATUS_UNSUCCESSFUL;
//   }
//
//   /*
//    * Allocate enough memory for the system's process list
//    */
//
//   if(dwFlags & (TH32CS_SNAPPROCESS | TH32CS_SNAPTHREAD) &&
//      NT_SUCCESS(Status))
//   {
//     for(;;)
//     {
//       (*ProcThrdInfoSize) += 0x10000;
//       Status = NtAllocateVirtualMemory(NtCurrentProcess(),
//                                        ProcThrdInfo,
//                                        0,
//                                        ProcThrdInfoSize,
//                                        MEM_COMMIT,
//                                        PAGE_READWRITE);
//       if(!NT_SUCCESS(Status))
//       {
//         break;
//       }
//
//       Status = NtQuerySystemInformation(SystemProcessInformation,
//                                         *ProcThrdInfo,
//                                         *ProcThrdInfoSize,
//                                         NULL);
//       if(Status == STATUS_INFO_LENGTH_MISMATCH)
//       {
//         NtFreeVirtualMemory(NtCurrentProcess(),
//                             ProcThrdInfo,
//                             ProcThrdInfoSize,
//                             MEM_RELEASE);
//         *ProcThrdInfo = NULL;
//       }
//       else
//       {
//         break;
//       }
//     }
//   }
//
//   /*
//    * Free resources in case of failure!
//    */
//
//   if(!NT_SUCCESS(Status))
//   {
//     TH32FreeAllocatedResources(*HeapDebug,
//                                *ModuleDebug,
//                                *ProcThrdInfo,
//                                *ProcThrdInfoSize);
//   }
//
//   return Status;
// }
//
// static NTSTATUS
// TH32CreateSnapshotSectionInitialize(DWORD dwFlags,
//                                     DWORD th32ProcessID,
//                                     PRTL_DEBUG_INFORMATION HeapDebug,
//                                     PRTL_DEBUG_INFORMATION ModuleDebug,
//                                     PVOID ProcThrdInfo,
//                                     HANDLE *SectionHandle)
// {
//   PSYSTEM_PROCESS_INFORMATION ProcessInfo;
//   LPHEAPLIST32 HeapListEntry;
//   LPMODULEENTRY32W ModuleListEntry;
//   LPPROCESSENTRY32W ProcessListEntry;
//   LPTHREADENTRY32 ThreadListEntry;
//   OBJECT_ATTRIBUTES ObjectAttributes;
//   LARGE_INTEGER SSize, SOffset;
//   HANDLE hSection;
//   PTH32SNAPSHOT Snapshot;
//   ULONG_PTR DataOffset;
//   SIZE_T ViewSize;
//   ULONG i, nProcesses = 0, nThreads = 0, nHeaps = 0, nModules = 0;
//   ULONG RequiredSnapshotSize = sizeof(TH32SNAPSHOT);
//   PRTL_PROCESS_HEAPS hi = NULL;
//   PRTL_PROCESS_MODULES mi = NULL;
//   NTSTATUS Status = STATUS_SUCCESS;
//
//   /*
//    * Determine the required size for the heap snapshot
//    */
//   if(dwFlags & TH32CS_SNAPHEAPLIST)
//   {
//     hi = (PRTL_PROCESS_HEAPS)HeapDebug->Heaps;
//     nHeaps = hi->NumberOfHeaps;
//     RequiredSnapshotSize += nHeaps * sizeof(HEAPLIST32);
//   }
//
//   /*
//    * Determine the required size for the module snapshot
//    */
//   if(dwFlags & TH32CS_SNAPMODULE)
//   {
//     mi = (PRTL_PROCESS_MODULES)ModuleDebug->Modules;
//     nModules = mi->NumberOfModules;
//     RequiredSnapshotSize += nModules * sizeof(MODULEENTRY32W);
//   }
//
//   /*
//    * Determine the required size for the processes and threads snapshot
//    */
//   if(dwFlags & (TH32CS_SNAPPROCESS | TH32CS_SNAPTHREAD))
//   {
//     ULONG ProcOffset = 0;
//
//     ProcessInfo = (PSYSTEM_PROCESS_INFORMATION)ProcThrdInfo;
//     do
//     {
//       ProcessInfo = (PSYSTEM_PROCESS_INFORMATION)((ULONG_PTR)ProcessInfo + ProcOffset);
//       nProcesses++;
//       nThreads += ProcessInfo->NumberOfThreads;
//       ProcOffset = ProcessInfo->NextEntryOffset;
//     } while(ProcOffset != 0);
//
//     if(dwFlags & TH32CS_SNAPPROCESS)
//     {
//       RequiredSnapshotSize += nProcesses * sizeof(PROCESSENTRY32W);
//     }
//     if(dwFlags & TH32CS_SNAPTHREAD)
//     {
//       RequiredSnapshotSize += nThreads * sizeof(THREADENTRY32);
//     }
//   }
//
//   /*
//    * Create and map the section
//    */
//
//   SSize.QuadPart = RequiredSnapshotSize;
//
//   InitializeObjectAttributes(&ObjectAttributes,
//                              NULL,
//                              ((dwFlags & TH32CS_INHERIT) ? OBJ_INHERIT : 0),
// 			     NULL,
// 			     NULL);
//
//   Status = NtCreateSection(&hSection,
//                            SECTION_ALL_ACCESS,
//                            &ObjectAttributes,
//                            &SSize,
//                            PAGE_READWRITE,
//                            SEC_COMMIT,
//                            NULL);
//   if(!NT_SUCCESS(Status))
//   {
//     return Status;
//   }
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSection,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(!NT_SUCCESS(Status))
//   {
//     NtClose(hSection);
//     return Status;
//   }
//
//   RtlZeroMemory(Snapshot, sizeof(TH32SNAPSHOT));
//   DataOffset = 0;
//
//   /*
//    * Initialize the section data and fill it with all the data we collected
//    */
//
//   /* initialize the heap list */
//   if(dwFlags & TH32CS_SNAPHEAPLIST)
//   {
//     Snapshot->HeapListCount = nHeaps;
//     Snapshot->HeapListOffset = DataOffset;
//     HeapListEntry = (LPHEAPLIST32)OffsetToPtr(Snapshot, DataOffset);
//     for(i = 0; i < nHeaps; i++)
//     {
//       HeapListEntry->dwSize = sizeof(HEAPLIST32);
//       HeapListEntry->th32ProcessID = th32ProcessID;
//       HeapListEntry->th32HeapID = (ULONG_PTR)hi->Heaps[i].BaseAddress;
//       HeapListEntry->dwFlags = hi->Heaps[i].Flags;
//
//       HeapListEntry++;
//     }
//
//     DataOffset += hi->NumberOfHeaps * sizeof(HEAPLIST32);
//   }
//
//   /* initialize the module list */
//   if(dwFlags & TH32CS_SNAPMODULE)
//   {
//     Snapshot->ModuleListCount = nModules;
//     Snapshot->ModuleListOffset = DataOffset;
//     ModuleListEntry = (LPMODULEENTRY32W)OffsetToPtr(Snapshot, DataOffset);
//     for(i = 0; i < nModules; i++)
//     {
//       ModuleListEntry->dwSize = sizeof(MODULEENTRY32W);
//       ModuleListEntry->th32ModuleID = 1; /* no longer used, always set to one! */
//       ModuleListEntry->th32ProcessID = th32ProcessID;
//       ModuleListEntry->GlblcntUsage = mi->Modules[i].LoadCount;
//       ModuleListEntry->ProccntUsage = mi->Modules[i].LoadCount;
//       ModuleListEntry->modBaseAddr = (BYTE*)mi->Modules[i].ImageBase;
//       ModuleListEntry->modBaseSize = mi->Modules[i].ImageSize;
//       ModuleListEntry->hModule = (HMODULE)mi->Modules[i].ImageBase;
//
//       MultiByteToWideChar(CP_ACP,
//                           0,
//                           &mi->Modules[i].FullPathName[mi->Modules[i].OffsetToFileName],
//                           -1,
//                           ModuleListEntry->szModule,
//                           sizeof(ModuleListEntry->szModule) / sizeof(ModuleListEntry->szModule[0]));
//
//       MultiByteToWideChar(CP_ACP,
//                           0,
//                           mi->Modules[i].FullPathName,
//                           -1,
//                           ModuleListEntry->szExePath,
//                           sizeof(ModuleListEntry->szExePath) / sizeof(ModuleListEntry->szExePath[0]));
//
//       ModuleListEntry++;
//     }
//
//     DataOffset += mi->NumberOfModules * sizeof(MODULEENTRY32W);
//   }
//
//   /* initialize the process list */
//   if(dwFlags & TH32CS_SNAPPROCESS)
//   {
//     ULONG ProcOffset = 0;
//
//     Snapshot->ProcessListCount = nProcesses;
//     Snapshot->ProcessListOffset = DataOffset;
//     ProcessListEntry = (LPPROCESSENTRY32W)OffsetToPtr(Snapshot, DataOffset);
//     ProcessInfo = (PSYSTEM_PROCESS_INFORMATION)ProcThrdInfo;
//     do
//     {
//       ProcessInfo = (PSYSTEM_PROCESS_INFORMATION)((ULONG_PTR)ProcessInfo + ProcOffset);
//
//       ProcessListEntry->dwSize = sizeof(PROCESSENTRY32W);
//       ProcessListEntry->cntUsage = 0; /* no longer used */
//       ProcessListEntry->th32ProcessID = (ULONG_PTR)ProcessInfo->UniqueProcessId;
//       ProcessListEntry->th32DefaultHeapID = 0; /* no longer used */
//       ProcessListEntry->th32ModuleID = 0; /* no longer used */
//       ProcessListEntry->cntThreads = ProcessInfo->NumberOfThreads;
//       ProcessListEntry->th32ParentProcessID = (ULONG_PTR)ProcessInfo->InheritedFromUniqueProcessId;
//       ProcessListEntry->pcPriClassBase = ProcessInfo->BasePriority;
//       ProcessListEntry->dwFlags = 0; /* no longer used */
//       if(ProcessInfo->ImageName.Buffer != NULL)
//       {
//         ULONG ExeFileLength = min(ProcessInfo->ImageName.Length,
//                                   sizeof(ProcessListEntry->szExeFile) - sizeof(WCHAR));
//         RtlCopyMemory(ProcessListEntry->szExeFile,
//                       ProcessInfo->ImageName.Buffer,
//                       ExeFileLength);
//         ProcessListEntry->szExeFile[ExeFileLength / sizeof(WCHAR)] = UNICODE_NULL;
//       }
//       else
//       {
//         lstrcpyW(ProcessListEntry->szExeFile, L"[System Process]");
//       }
//
//       ProcessListEntry++;
//
//       ProcOffset = ProcessInfo->NextEntryOffset;
//     } while(ProcOffset != 0);
//
//     DataOffset += nProcesses * sizeof(PROCESSENTRY32W);
//   }
//
//   /* initialize the thread list */
//   if(dwFlags & TH32CS_SNAPTHREAD)
//   {
//     ULONG ProcOffset = 0;
//
//     Snapshot->ThreadListCount = nThreads;
//     Snapshot->ThreadListOffset = DataOffset;
//     ThreadListEntry = (LPTHREADENTRY32)OffsetToPtr(Snapshot, DataOffset);
//     ProcessInfo = (PSYSTEM_PROCESS_INFORMATION)ProcThrdInfo;
//     do
//     {
//       PSYSTEM_THREAD_INFORMATION ThreadInfo;
//       ULONG n;
//
//       ProcessInfo = (PSYSTEM_PROCESS_INFORMATION)((ULONG_PTR)ProcessInfo + ProcOffset);
//       ThreadInfo = (PSYSTEM_THREAD_INFORMATION)(ProcessInfo + 1);
//
//       for(n = 0; n < ProcessInfo->NumberOfThreads; n++)
//       {
//         ThreadListEntry->dwSize = sizeof(THREADENTRY32);
//         ThreadListEntry->cntUsage = 0; /* no longer used */
//         ThreadListEntry->th32ThreadID = (ULONG_PTR)ThreadInfo->ClientId.UniqueThread;
//         ThreadListEntry->th32OwnerProcessID = (ULONG_PTR)ThreadInfo->ClientId.UniqueProcess;
//         ThreadListEntry->tpBasePri = ThreadInfo->BasePriority;
//         ThreadListEntry->tpDeltaPri = 0; /* no longer used */
//         ThreadListEntry->dwFlags = 0; /* no longer used */
//
//         ThreadInfo++;
//         ThreadListEntry++;
//       }
//
//       ProcOffset = ProcessInfo->NextEntryOffset;
//     } while(ProcOffset != 0);
//   }
//
//   /*
//    * We're done, unmap the view and return the section handle
//    */
//
//   Status = NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//
//   if(NT_SUCCESS(Status))
//   {
//     *SectionHandle = hSection;
//   }
//   else
//   {
//     NtClose(hSection);
//   }
//
//   return Status;
// }
//
// /* PUBLIC FUNCTIONS ***********************************************************/
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Heap32First(LPHEAPENTRY32 lphe, DWORD th32ProcessID, DWORD th32HeapID)
// {
//   PRTL_DEBUG_INFORMATION DebugInfo;
//   PRTL_HEAP_INFORMATION Heap;
//   PRTLP_HEAP_ENTRY Block, LastBlock;
//   ULONG i;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lphe, sizeof(HEAPENTRY32));
//
//   DebugInfo = RtlCreateQueryDebugBuffer(0,
//                                         FALSE);
//   if (DebugInfo == NULL)
//   {
//     SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//     return FALSE;
//   }
//
//   Status = RtlQueryProcessDebugInformation(th32ProcessID,
//                                            RTL_DEBUG_QUERY_HEAPS | RTL_DEBUG_QUERY_HEAP_BLOCKS,
//                                            DebugInfo);
//
//   if (NT_SUCCESS(Status))
//   {
//     Status = STATUS_NO_MORE_FILES;
//
//     for (i = 0;
//          i != DebugInfo->Heaps->NumberOfHeaps;
//          i++)
//     {
//       Heap = &DebugInfo->Heaps->Heaps[i];
//
//       if ((ULONG_PTR)Heap->BaseAddress == th32HeapID)
//       {
//         lphe->hHandle = (HANDLE)Heap->BaseAddress;
//         lphe->dwAddress = 0;
//         lphe->dwBlockSize = 0;
//         lphe->dwFlags = 0;
//         lphe->dwLockCount = 0;
//         lphe->dwResvd = 0;
//         lphe->th32ProcessID = th32ProcessID;
//         lphe->th32HeapID = (ULONG_PTR)Heap->BaseAddress;
//
//         Block = (PRTLP_HEAP_ENTRY)Heap->Entries;
//         LastBlock = Block + Heap->NumberOfEntries;
//
//         while (Block != LastBlock && (Block->Flags & PROCESS_HEAP_UNCOMMITTED_RANGE))
//         {
//           lphe->dwResvd++;
//           lphe->dwAddress = (ULONG_PTR)((ULONG_PTR)Block->Address + Heap->EntryOverhead);
//           Block++;
//         }
//
//         if (Block != LastBlock && lphe->dwResvd != 0)
//         {
//           lphe->dwBlockSize =  Block->Size;
//
//           if (Block->Flags & 0x2F1) /* FIXME */
//             lphe->dwFlags = LF32_FIXED;
//           else if (Block->Flags & 0x20) /* FIXME */
//             lphe->dwFlags = LF32_MOVEABLE;
//           else if (Block->Flags & 0x100) /* FIXME */
//             lphe->dwFlags = LF32_FREE;
//
//           Status = STATUS_SUCCESS;
//         }
//
//         break;
//       }
//     }
//   }
//
//   RtlDestroyQueryDebugBuffer(DebugInfo);
//
//   if (!NT_SUCCESS(Status))
//   {
//     BaseSetLastNTError(Status);
//     return FALSE;
//   }
//
//   return TRUE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Heap32Next(LPHEAPENTRY32 lphe)
// {
//   PRTL_DEBUG_INFORMATION DebugInfo;
//   PRTL_HEAP_INFORMATION Heap;
//   PRTLP_HEAP_ENTRY Block, LastBlock;
//   BOOLEAN FoundUncommitted = FALSE;
//   ULONG i;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lphe, sizeof(HEAPENTRY32));
//
//   DebugInfo = RtlCreateQueryDebugBuffer(0,
//                                         FALSE);
//   if (DebugInfo == NULL)
//   {
//     SetLastError(ERROR_NOT_ENOUGH_MEMORY);
//     return FALSE;
//   }
//
//   Status = RtlQueryProcessDebugInformation(lphe->th32ProcessID,
//                                            RTL_DEBUG_QUERY_HEAPS | RTL_DEBUG_QUERY_HEAP_BLOCKS,
//                                            DebugInfo);
//
//   if (NT_SUCCESS(Status))
//   {
//     Status = STATUS_NO_MORE_FILES;
//
//     for (i = 0;
//          i != DebugInfo->Heaps->NumberOfHeaps;
//          i++)
//     {
//       Heap = &DebugInfo->Heaps->Heaps[i];
//
//       if ((ULONG_PTR)Heap->BaseAddress == lphe->th32HeapID)
//       {
//         if (++lphe->dwResvd < Heap->NumberOfEntries)
//         {
//           lphe->dwFlags = 0;
//
//           Block = (PRTLP_HEAP_ENTRY)Heap->Entries + lphe->dwResvd;
//           LastBlock = (PRTLP_HEAP_ENTRY)Heap->Entries + Heap->NumberOfEntries;
//
//           while (Block < LastBlock && (Block->Flags & PROCESS_HEAP_UNCOMMITTED_RANGE))
//           {
//             lphe->dwResvd++;
//             lphe->dwAddress = (ULONG_PTR)((ULONG_PTR)Block->Address + Heap->EntryOverhead);
//             FoundUncommitted = TRUE;
//             Block++;
//           }
//
//           if (Block < LastBlock)
//           {
//             if (!FoundUncommitted)
//               lphe->dwAddress += lphe->dwBlockSize;
//
//             lphe->dwBlockSize =  Block->Size;
//
//             if (Block->Flags & 0x2F1) /* FIXME */
//               lphe->dwFlags = LF32_FIXED;
//             else if (Block->Flags & 0x20) /* FIXME */
//               lphe->dwFlags = LF32_MOVEABLE;
//             else if (Block->Flags & 0x100) /* FIXME */
//               lphe->dwFlags = LF32_FREE;
//
//             Status = STATUS_SUCCESS;
//           }
//         }
//
//         break;
//       }
//     }
//   }
//
//   RtlDestroyQueryDebugBuffer(DebugInfo);
//
//   if (!NT_SUCCESS(Status))
//   {
//     BaseSetLastNTError(Status);
//     return FALSE;
//   }
//
//   return TRUE;
//
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Heap32ListFirst(HANDLE hSnapshot, LPHEAPLIST32 lphl)
// {
//   PTH32SNAPSHOT Snapshot;
//   LARGE_INTEGER SOffset;
//   SIZE_T ViewSize;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lphl, sizeof(HEAPLIST32));
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSnapshot,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(NT_SUCCESS(Status))
//   {
//     BOOL Ret;
//
//     if(Snapshot->HeapListCount > 0)
//     {
//       LPHEAPLIST32 Entries = (LPHEAPLIST32)OffsetToPtr(Snapshot, Snapshot->HeapListOffset);
//       Snapshot->HeapListIndex = 1;
//       RtlCopyMemory(lphl, &Entries[0], sizeof(HEAPLIST32));
//       Ret = TRUE;
//     }
//     else
//     {
//       SetLastError(ERROR_NO_MORE_FILES);
//       Ret = FALSE;
//     }
//
//     NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//     return Ret;
//   }
//
//   BaseSetLastNTError(Status);
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Heap32ListNext(HANDLE hSnapshot, LPHEAPLIST32 lphl)
// {
//   PTH32SNAPSHOT Snapshot;
//   LARGE_INTEGER SOffset;
//   SIZE_T ViewSize;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lphl, sizeof(HEAPLIST32));
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSnapshot,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(NT_SUCCESS(Status))
//   {
//     BOOL Ret;
//
//     if(Snapshot->HeapListCount > 0 &&
//        Snapshot->HeapListIndex < Snapshot->HeapListCount)
//     {
//       LPHEAPLIST32 Entries = (LPHEAPLIST32)OffsetToPtr(Snapshot, Snapshot->HeapListOffset);
//       RtlCopyMemory(lphl, &Entries[Snapshot->HeapListIndex++], sizeof(HEAPLIST32));
//       Ret = TRUE;
//     }
//     else
//     {
//       SetLastError(ERROR_NO_MORE_FILES);
//       Ret = FALSE;
//     }
//
//     NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//     return Ret;
//   }
//
//   BaseSetLastNTError(Status);
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Module32First(HANDLE hSnapshot, LPMODULEENTRY32 lpme)
// {
//   MODULEENTRY32W me;
//   BOOL Ret;
//
//   CHECK_PARAM_SIZEA(lpme, sizeof(MODULEENTRY32));
//
//   me.dwSize = sizeof(MODULEENTRY32W);
//
//   Ret = Module32FirstW(hSnapshot, &me);
//   if(Ret)
//   {
//     lpme->th32ModuleID = me.th32ModuleID;
//     lpme->th32ProcessID = me.th32ProcessID;
//     lpme->GlblcntUsage = me.GlblcntUsage;
//     lpme->ProccntUsage = me.ProccntUsage;
//     lpme->modBaseAddr = me.modBaseAddr;
//     lpme->modBaseSize = me.modBaseSize;
//     lpme->hModule = me.hModule;
//
//     WideCharToMultiByte(CP_ACP, 0, me.szModule, -1, lpme->szModule, sizeof(lpme->szModule), 0, 0);
//     WideCharToMultiByte(CP_ACP, 0, me.szExePath, -1, lpme->szExePath, sizeof(lpme->szExePath), 0, 0);
//   }
//
//   return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Module32FirstW(HANDLE hSnapshot, LPMODULEENTRY32W lpme)
// {
//   PTH32SNAPSHOT Snapshot;
//   LARGE_INTEGER SOffset;
//   SIZE_T ViewSize;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lpme, sizeof(MODULEENTRY32W));
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSnapshot,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(NT_SUCCESS(Status))
//   {
//     BOOL Ret;
//
//     if(Snapshot->ModuleListCount > 0)
//     {
//       LPMODULEENTRY32W Entries = (LPMODULEENTRY32W)OffsetToPtr(Snapshot, Snapshot->ModuleListOffset);
//       Snapshot->ModuleListIndex = 1;
//       RtlCopyMemory(lpme, &Entries[0], sizeof(MODULEENTRY32W));
//       Ret = TRUE;
//     }
//     else
//     {
//       SetLastError(ERROR_NO_MORE_FILES);
//       Ret = FALSE;
//     }
//
//     NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//     return Ret;
//   }
//
//   BaseSetLastNTError(Status);
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Module32Next(HANDLE hSnapshot, LPMODULEENTRY32 lpme)
// {
//   MODULEENTRY32W me;
//   BOOL Ret;
//
//   CHECK_PARAM_SIZEA(lpme, sizeof(MODULEENTRY32));
//
//   me.dwSize = sizeof(MODULEENTRY32W);
//
//   Ret = Module32NextW(hSnapshot, &me);
//   if(Ret)
//   {
//     lpme->th32ModuleID = me.th32ModuleID;
//     lpme->th32ProcessID = me.th32ProcessID;
//     lpme->GlblcntUsage = me.GlblcntUsage;
//     lpme->ProccntUsage = me.ProccntUsage;
//     lpme->modBaseAddr = me.modBaseAddr;
//     lpme->modBaseSize = me.modBaseSize;
//     lpme->hModule = me.hModule;
//
//     WideCharToMultiByte(CP_ACP, 0, me.szModule, -1, lpme->szModule, sizeof(lpme->szModule), 0, 0);
//     WideCharToMultiByte(CP_ACP, 0, me.szExePath, -1, lpme->szExePath, sizeof(lpme->szExePath), 0, 0);
//   }
//
//   return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Module32NextW(HANDLE hSnapshot, LPMODULEENTRY32W lpme)
// {
//   PTH32SNAPSHOT Snapshot;
//   LARGE_INTEGER SOffset;
//   SIZE_T ViewSize;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lpme, sizeof(MODULEENTRY32W));
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSnapshot,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(NT_SUCCESS(Status))
//   {
//     BOOL Ret;
//
//     if((Snapshot->ModuleListCount > 0) &&
//        (Snapshot->ModuleListIndex < Snapshot->ModuleListCount))
//     {
//       LPMODULEENTRY32W Entries = (LPMODULEENTRY32W)OffsetToPtr(Snapshot, Snapshot->ModuleListOffset);
//       RtlCopyMemory(lpme, &Entries[Snapshot->ModuleListIndex++], sizeof(MODULEENTRY32W));
//       Ret = TRUE;
//     }
//     else
//     {
//       SetLastError(ERROR_NO_MORE_FILES);
//       Ret = FALSE;
//     }
//
//     NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//     return Ret;
//   }
//
//   BaseSetLastNTError(Status);
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Process32First(HANDLE hSnapshot, LPPROCESSENTRY32 lppe)
// {
//   PROCESSENTRY32W pe;
//   BOOL Ret;
//
//   CHECK_PARAM_SIZEA(lppe, sizeof(PROCESSENTRY32));
//
//   pe.dwSize = sizeof(PROCESSENTRY32W);
//
//   Ret = Process32FirstW(hSnapshot, &pe);
//   if(Ret)
//   {
//     lppe->cntUsage = pe.cntUsage;
//     lppe->th32ProcessID = pe.th32ProcessID;
//     lppe->th32DefaultHeapID = pe.th32DefaultHeapID;
//     lppe->th32ModuleID = pe.th32ModuleID;
//     lppe->cntThreads = pe.cntThreads;
//     lppe->th32ParentProcessID = pe.th32ParentProcessID;
//     lppe->pcPriClassBase = pe.pcPriClassBase;
//     lppe->dwFlags = pe.dwFlags;
//
//     WideCharToMultiByte(CP_ACP, 0, pe.szExeFile, -1, lppe->szExeFile, sizeof(lppe->szExeFile), 0, 0);
//   }
//
//   return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Process32FirstW(HANDLE hSnapshot, LPPROCESSENTRY32W lppe)
// {
//   PTH32SNAPSHOT Snapshot;
//   LARGE_INTEGER SOffset;
//   SIZE_T ViewSize;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lppe, sizeof(PROCESSENTRY32W));
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSnapshot,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(NT_SUCCESS(Status))
//   {
//     BOOL Ret;
//
//     if(Snapshot->ProcessListCount > 0)
//     {
//       LPPROCESSENTRY32W Entries = (LPPROCESSENTRY32W)OffsetToPtr(Snapshot, Snapshot->ProcessListOffset);
//
//       Snapshot->ProcessListIndex = 1;
//       RtlCopyMemory(lppe, &Entries[0], sizeof(PROCESSENTRY32W));
//       Ret = TRUE;
//     }
//     else
//     {
//
//       SetLastError(ERROR_NO_MORE_FILES);
//       Ret = FALSE;
//     }
//
//     NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//     return Ret;
//   }
//
//   BaseSetLastNTError(Status);
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Process32Next(HANDLE hSnapshot, LPPROCESSENTRY32 lppe)
// {
//   PROCESSENTRY32W pe;
//   BOOL Ret;
//
//   CHECK_PARAM_SIZEA(lppe, sizeof(PROCESSENTRY32));
//
//   pe.dwSize = sizeof(PROCESSENTRY32W);
//
//   Ret = Process32NextW(hSnapshot, &pe);
//   if(Ret)
//   {
//     lppe->cntUsage = pe.cntUsage;
//     lppe->th32ProcessID = pe.th32ProcessID;
//     lppe->th32DefaultHeapID = pe.th32DefaultHeapID;
//     lppe->th32ModuleID = pe.th32ModuleID;
//     lppe->cntThreads = pe.cntThreads;
//     lppe->th32ParentProcessID = pe.th32ParentProcessID;
//     lppe->pcPriClassBase = pe.pcPriClassBase;
//     lppe->dwFlags = pe.dwFlags;
//
//     WideCharToMultiByte(CP_ACP, 0, pe.szExeFile, -1, lppe->szExeFile, sizeof(lppe->szExeFile), 0, 0);
//   }
//
//   return Ret;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Process32NextW(HANDLE hSnapshot, LPPROCESSENTRY32W lppe)
// {
//   PTH32SNAPSHOT Snapshot;
//   LARGE_INTEGER SOffset;
//   SIZE_T ViewSize;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lppe, sizeof(PROCESSENTRY32W));
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSnapshot,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(NT_SUCCESS(Status))
//   {
//     BOOL Ret;
//
//     if(Snapshot->ProcessListCount > 0 &&
//        Snapshot->ProcessListIndex < Snapshot->ProcessListCount)
//     {
//       LPPROCESSENTRY32W Entries = (LPPROCESSENTRY32W)OffsetToPtr(Snapshot, Snapshot->ProcessListOffset);
//       RtlCopyMemory(lppe, &Entries[Snapshot->ProcessListIndex++], sizeof(PROCESSENTRY32W));
//       Ret = TRUE;
//     }
//     else
//     {
//       SetLastError(ERROR_NO_MORE_FILES);
//       Ret = FALSE;
//     }
//
//     NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//     return Ret;
//   }
//
//   BaseSetLastNTError(Status);
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Thread32First(HANDLE hSnapshot, LPTHREADENTRY32 lpte)
// {
//   PTH32SNAPSHOT Snapshot;
//   LARGE_INTEGER SOffset;
//   SIZE_T ViewSize;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lpte, sizeof(THREADENTRY32));
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSnapshot,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(NT_SUCCESS(Status))
//   {
//     BOOL Ret;
//
//     if(Snapshot->ThreadListCount > 0)
//     {
//       LPTHREADENTRY32 Entries = (LPTHREADENTRY32)OffsetToPtr(Snapshot, Snapshot->ThreadListOffset);
//       Snapshot->ThreadListIndex = 1;
//       RtlCopyMemory(lpte, &Entries[0], sizeof(THREADENTRY32));
//       Ret = TRUE;
//     }
//     else
//     {
//       SetLastError(ERROR_NO_MORE_FILES);
//       Ret = FALSE;
//     }
//
//     NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//     return Ret;
//   }
//
//   BaseSetLastNTError(Status);
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Thread32Next(HANDLE hSnapshot, LPTHREADENTRY32 lpte)
// {
//   PTH32SNAPSHOT Snapshot;
//   LARGE_INTEGER SOffset;
//   SIZE_T ViewSize;
//   NTSTATUS Status;
//
//   CHECK_PARAM_SIZE(lpte, sizeof(THREADENTRY32));
//
//   SOffset.QuadPart = 0;
//   ViewSize = 0;
//   Snapshot = NULL;
//
//   Status = NtMapViewOfSection(hSnapshot,
//                               NtCurrentProcess(),
//                               (PVOID*)&Snapshot,
//                               0,
//                               0,
//                               &SOffset,
//                               &ViewSize,
//                               ViewShare,
//                               0,
//                               PAGE_READWRITE);
//   if(NT_SUCCESS(Status))
//   {
//     BOOL Ret;
//
//     if(Snapshot->ThreadListCount > 0 &&
//        Snapshot->ThreadListIndex < Snapshot->ThreadListCount)
//     {
//       LPTHREADENTRY32 Entries = (LPTHREADENTRY32)OffsetToPtr(Snapshot, Snapshot->ThreadListOffset);
//       RtlCopyMemory(lpte, &Entries[Snapshot->ThreadListIndex++], sizeof(THREADENTRY32));
//       Ret = TRUE;
//     }
//     else
//     {
//       SetLastError(ERROR_NO_MORE_FILES);
//       Ret = FALSE;
//     }
//
//     NtUnmapViewOfSection(NtCurrentProcess(), (PVOID)Snapshot);
//     return Ret;
//   }
//
//   BaseSetLastNTError(Status);
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// BOOL
// WINAPI
// Toolhelp32ReadProcessMemory(DWORD th32ProcessID,  LPCVOID lpBaseAddress,
//                             LPVOID lpBuffer, SIZE_T cbRead, SIZE_T* lpNumberOfBytesRead)
// {
//   HANDLE hProcess = OpenProcess(PROCESS_VM_READ, FALSE, th32ProcessID);
//   if(hProcess != NULL)
//   {
//     BOOL Ret = ReadProcessMemory(hProcess, lpBaseAddress, lpBuffer, cbRead, lpNumberOfBytesRead);
//     CloseHandle(hProcess);
//     return Ret;
//   }
//
//   return FALSE;
// }
//
//
// /*
//  * @implemented
//  */
// HANDLE
// WINAPI
// CreateToolhelp32Snapshot(DWORD dwFlags, DWORD th32ProcessID)
// {
//   PRTL_DEBUG_INFORMATION HeapDebug, ModuleDebug;
//   PVOID ProcThrdInfo;
//   SIZE_T ProcThrdInfoSize;
//   NTSTATUS Status;
//   HANDLE hSnapShotSection = NULL;
//
//   if(th32ProcessID == 0)
//   {
//     th32ProcessID = GetCurrentProcessId();
//   }
//
//   /*
//    * Get all information required for the snapshot
//    */
//   Status = TH32CreateSnapshot(dwFlags,
//                               th32ProcessID,
//                               &HeapDebug,
//                               &ModuleDebug,
//                               &ProcThrdInfo,
//                               &ProcThrdInfoSize);
//   if(!NT_SUCCESS(Status))
//   {
//     BaseSetLastNTError(Status);
//     return NULL;
//   }
//
//   /*
//    * Create a section handle and initialize the collected information
//    */
//   Status = TH32CreateSnapshotSectionInitialize(dwFlags,
//                                                th32ProcessID,
//                                                HeapDebug,
//                                                ModuleDebug,
//                                                ProcThrdInfo,
//                                                &hSnapShotSection);
//
//   /*
//    * Free the temporarily allocated memory which is no longer needed
//    */
//   TH32FreeAllocatedResources(HeapDebug,
//                              ModuleDebug,
//                              ProcThrdInfo,
//                              ProcThrdInfoSize);
//
//   if(!NT_SUCCESS(Status))
//   {
//     BaseSetLastNTError(Status);
//     return NULL;
//   }
//
//   return hSnapShotSection;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
