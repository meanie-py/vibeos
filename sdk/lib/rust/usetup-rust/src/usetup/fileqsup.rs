use crate::usetup::cabinet::CabinetCleanup;
use crate::usetup::cabinet::CabinetExtractFile;
use crate::usetup::cabinet::CabinetFindFirst;
use crate::usetup::cabinet::CabinetFindNextFileSequential;
use crate::usetup::cabinet::CabinetGetCabinetName;
use crate::usetup::cabinet::CabinetInitialize;
use crate::usetup::cabinet::CabinetOpen;
use crate::usetup::cabinet::CabinetSetCabinetName;
use crate::usetup::cabinet::CabinetSetDestinationPath;
use crate::usetup::cabinet::CabinetSetEventHandlers;
// Deterministic UTF-16 literals, generated from the C by rustconv (--seal-wide); the model never saw the bytes.
use crate::prelude::*;
pub static RUSTCONV_WSTR_6AF5D4B374C5: [u16; 4] = [0x006Eu16, 0x002F, 0x0061, 0x0000];

use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
#[no_mangle]
pub static mut SpFileExports: SPFILE_EXPORTS = unsafe {
    _SPFILE_EXPORTS {
        SpFileQueueOpen: Some(SetupOpenFileQueue),
        SpFileQueueClose: Some(SetupCloseFileQueue),
        SpFileQueueCopy: Some(
            SetupQueueCopyWithCab,
        ),
        SpFileQueueDelete: Some(
            SetupQueueDeleteW,
        ),
        SpFileQueueRename: Some(
            SetupQueueRenameW,
        ),
        SpFileQueueCommit: Some(
            SetupCommitFileQueueW,
        ),
    }
};

#[repr(C)]
pub struct _QUEUEENTRY {
    pub ListEntry: LIST_ENTRY,
    pub SourceCabinet: PWSTR,
    pub SourceRootPath: PWSTR,
    pub SourcePath: PWSTR,
    pub SourceFileName: PWSTR,
    pub TargetDirectory: PWSTR,
    pub TargetFileName: PWSTR,
}
pub type QUEUEENTRY = _QUEUEENTRY;
pub type PQUEUEENTRY = *mut _QUEUEENTRY;

#[repr(C)]
pub struct _FILEQUEUEHEADER {
    pub DeleteQueue: LIST_ENTRY,
    pub DeleteCount: ULONG,
    pub RenameQueue: LIST_ENTRY,
    pub RenameCount: ULONG,
    pub CopyQueue: LIST_ENTRY,
    pub CopyCount: ULONG,
    pub HasCurrentCabinet: BOOLEAN,
    pub CabinetContext: CABINET_CONTEXT,
    pub Search: CAB_SEARCH,
    pub CurrentCabinetName: [WCHAR; (MAX_PATH) as _],
}
pub type FILEQUEUEHEADER = _FILEQUEUEHEADER;
pub type PFILEQUEUEHEADER = *mut _FILEQUEUEHEADER;

unsafe extern "C" fn SetupExtractFile(
    mut QueueHeader: PFILEQUEUEHEADER,
    mut CabinetFileName: PCWSTR,
    mut SourceFileName: PCWSTR,
    mut DestinationPathName: PCWSTR,
) -> NTSTATUS {
    let mut CabStatus: ULONG = 0;
    DPRINT!(
        b"SetupExtractFile(CabinetFileName: '%S', SourceFileName: '%S', DestinationPathName: '%S')\n\0"
            .as_ptr() as *const i8,
        CabinetFileName,
        SourceFileName,
        DestinationPathName,
    );
    if (*QueueHeader).HasCurrentCabinet != 0 {
        DPRINT!(
            b"CurrentCabinetName: '%S'\n\0".as_ptr() as *const i8,
            (*QueueHeader).CurrentCabinetName.as_mut_ptr(),
        );
    }
    if (*QueueHeader).HasCurrentCabinet != 0
        && wcscmp(
            CabinetFileName,
            (*QueueHeader).CurrentCabinetName.as_mut_ptr(),
        ) == 0
    {
        DPRINT!(b"Using same cabinet as last time\n\0".as_ptr() as *const i8);
        CabStatus = CabinetFindNextFileSequential(
            &raw mut (*QueueHeader).CabinetContext,
            SourceFileName,
            &raw mut (*QueueHeader).Search,
        );
        if CabStatus != CAB_STATUS_SUCCESS {
            DPRINT!(b"Sequential miss on file: %S\n\0".as_ptr() as *const i8, SourceFileName);
            CabStatus = CabinetFindFirst(
                &raw mut (*QueueHeader).CabinetContext,
                SourceFileName,
                &raw mut (*QueueHeader).Search,
            );
        }
    } else {
        DPRINT!(b"Using new cabinet\n\0".as_ptr() as *const i8);
        if (*QueueHeader).HasCurrentCabinet != 0 {
            (*QueueHeader).HasCurrentCabinet = (FALSE) as u8;
            CabinetCleanup(&raw mut (*QueueHeader).CabinetContext);
        }
        RtlStringCchCopyW(
            (*QueueHeader).CurrentCabinetName.as_mut_ptr(),
            (ARRAYSIZE!((*QueueHeader).CurrentCabinetName)) as _,
            CabinetFileName,
        );
        CabinetInitialize(&raw mut (*QueueHeader).CabinetContext);
        CabinetSetEventHandlers(
            &raw mut (*QueueHeader).CabinetContext,
            None,
            None,
            None,
            None,
        );
        CabinetSetCabinetName(&raw mut (*QueueHeader).CabinetContext, CabinetFileName);
        CabStatus = CabinetOpen(&raw mut (*QueueHeader).CabinetContext);
        if CabStatus == CAB_STATUS_SUCCESS {
            DPRINT!(b"Opened cabinet %S\n\0".as_ptr() as *const i8, CabinetFileName);
            (*QueueHeader).HasCurrentCabinet = (TRUE) as u8;
        } else {
            DPRINT!(b"Cannot open cabinet (%d)\n\0".as_ptr() as *const i8, CabStatus);
            return STATUS_UNSUCCESSFUL;
        }
        CabStatus = CabinetFindFirst(
            &raw mut (*QueueHeader).CabinetContext,
            SourceFileName,
            &raw mut (*QueueHeader).Search,
        );
    }
    if CabStatus != CAB_STATUS_SUCCESS {
        DPRINT1!(
            b"Unable to find '%S' in cabinet '%S'\n\0".as_ptr() as *const i8,
            SourceFileName,
            CabinetGetCabinetName(&raw mut (*QueueHeader).CabinetContext),
        );
        return STATUS_UNSUCCESSFUL;
    }
    CabinetSetDestinationPath(&raw mut (*QueueHeader).CabinetContext, DestinationPathName);
    CabStatus = CabinetExtractFile(
        &raw mut (*QueueHeader).CabinetContext,
        &raw mut (*QueueHeader).Search,
    );
    if CabStatus != CAB_STATUS_SUCCESS {
        DPRINT!(b"Cannot extract file %S (%d)\n\0".as_ptr() as *const i8, SourceFileName, CabStatus);
        return STATUS_UNSUCCESSFUL;
    }
    return STATUS_SUCCESS;
}

#[no_mangle]
pub unsafe extern "system" fn SetupOpenFileQueue() -> HSPFILEQ {
    let mut QueueHeader: *mut FILEQUEUEHEADER = core::ptr::null_mut();

    QueueHeader = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<FILEQUEUEHEADER>()) as SIZE_T,
    ) as *mut FILEQUEUEHEADER;
    if QueueHeader.is_null() {
        return core::ptr::null_mut();
    }

    RtlZeroMemory!(
        QueueHeader as *mut core::ffi::c_void,
        core::mem::size_of::<FILEQUEUEHEADER>(),
    );

    InitializeListHead(&raw mut (*QueueHeader).DeleteQueue);
    (*QueueHeader).DeleteCount = 0;
    InitializeListHead(&raw mut (*QueueHeader).RenameQueue);
    (*QueueHeader).RenameCount = 0;
    InitializeListHead(&raw mut (*QueueHeader).CopyQueue);
    (*QueueHeader).CopyCount = 0;

    (*QueueHeader).HasCurrentCabinet = (FALSE) as u8;

    QueueHeader as HSPFILEQ
}

unsafe extern "C" fn SetupDeleteQueueEntry(mut Entry: PQUEUEENTRY) {
    if Entry.is_null() {
        return;
    }
    if !(*Entry).SourceCabinet.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).SourceCabinet as PVOID);
    }
    if !(*Entry).SourceRootPath.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).SourceRootPath as PVOID);
    }
    if !(*Entry).SourcePath.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).SourcePath as PVOID);
    }
    if !(*Entry).SourceFileName.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).SourceFileName as PVOID);
    }
    if !(*Entry).TargetDirectory.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).TargetDirectory as PVOID);
    }
    if !(*Entry).TargetFileName.is_null() {
        RtlFreeHeap(ProcessHeap, 0, (*Entry).TargetFileName as PVOID);
    }
    RtlFreeHeap(ProcessHeap, 0, Entry as PVOID);
}

#[no_mangle]
pub unsafe extern "system" fn SetupCloseFileQueue(
    QueueHandle: HSPFILEQ,
) -> BOOL {
    let mut QueueHeader: PFILEQUEUEHEADER;
    let mut ListEntry: PLIST_ENTRY;
    let mut Entry: PQUEUEENTRY;

    if QueueHandle.is_null() {
        return (FALSE) as BOOL;
    }

    QueueHeader = QueueHandle as PFILEQUEUEHEADER;

    /* Delete the delete queue */
    while !IsListEmpty(&mut (*QueueHeader).DeleteQueue) {
        ListEntry = RemoveHeadList(&mut (*QueueHeader).DeleteQueue);
        Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);
        SetupDeleteQueueEntry(Entry);
    }

    /* Delete the rename queue */
    while !IsListEmpty(&mut (*QueueHeader).RenameQueue) {
        ListEntry = RemoveHeadList(&mut (*QueueHeader).RenameQueue);
        Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);
        SetupDeleteQueueEntry(Entry);
    }

    /* Delete the copy queue */
    while !IsListEmpty(&mut (*QueueHeader).CopyQueue) {
        ListEntry = RemoveHeadList(&mut (*QueueHeader).CopyQueue);
        Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);
        SetupDeleteQueueEntry(Entry);
    }

    /* Delete queue header */
    RtlFreeHeap(ProcessHeap, 0, (QueueHeader).cast());

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueCopyWithCab(
    QueueHandle: HSPFILEQ,
    SourceRootPath: PCWSTR,
    SourcePath: PCWSTR,
    SourceFileName: PCWSTR,
    SourceDescription: PCWSTR,
    SourceCabinet: PCWSTR,
    SourceTagFile: PCWSTR,
    TargetDirectory: PCWSTR,
    TargetFileName: PCWSTR,
    CopyStyle: u32,
) -> i32 {
    let mut QueueHeader: *mut FILEQUEUEHEADER;
    let mut Entry: *mut QUEUEENTRY;
    let mut Length: u32;

    if QueueHandle.is_null()
        || SourceRootPath.is_null()
        || SourceFileName.is_null()
        || TargetDirectory.is_null()
    {
        return (FALSE) as i32;
    }

    QueueHeader = QueueHandle as *mut FILEQUEUEHEADER;

    DPRINT!(
        b"SetupQueueCopy(Cab '%S', SrcRootPath '%S', SrcPath '%S', SrcFN '%S' --> DstPath '%S', DstFN '%S')\n\0"
            .as_ptr() as *const i8,
        if !SourceCabinet.is_null() {
            SourceCabinet
        } else {
            RUSTCONV_WSTR_6AF5D4B374C5.as_ptr()
        },
        SourceRootPath,
        SourcePath,
        SourceFileName,
        TargetDirectory,
        TargetFileName,
    );

    Entry = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<QUEUEENTRY>()) as SIZE_T,
    ) as *mut QUEUEENTRY;
    if Entry.is_null() {
        return (FALSE) as i32;
    }

    RtlZeroMemory!(
        Entry as *mut core::ffi::c_void,
        core::mem::size_of::<QUEUEENTRY>(),
    );

    (*Entry).SourceCabinet = core::ptr::null_mut();
    if !SourceCabinet.is_null() {
        Length = wcslen(SourceCabinet) as u32;
        (*Entry).SourceCabinet = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
        ) as *mut u16;
        if (*Entry).SourceCabinet.is_null() {
            RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
            return (FALSE) as i32;
        }
        RtlStringCchCopyW(
            (*Entry).SourceCabinet,
            (Length + 1) as _,
            SourceCabinet,
        );
    }

    Length = wcslen(SourceRootPath) as u32;
    (*Entry).SourceRootPath = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
    ) as *mut u16;
    if (*Entry).SourceRootPath.is_null() {
        if !(*Entry).SourceCabinet.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceCabinet as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as i32;
    }
    RtlStringCchCopyW(
        (*Entry).SourceRootPath,
        (Length + 1) as _,
        SourceRootPath,
    );

    (*Entry).SourcePath = core::ptr::null_mut();
    if !SourcePath.is_null() {
        Length = wcslen(SourcePath) as u32;
        if (Length > 0)
            && (*SourcePath.offset((Length - 1) as isize) as u16)
                == 0x005Cu16
        {
            Length -= 1;
        }
        (*Entry).SourcePath = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
        ) as *mut u16;
        if (*Entry).SourcePath.is_null() {
            if !(*Entry).SourceCabinet.is_null() {
                RtlFreeHeap(
                    ProcessHeap,
                    0,
                    (*Entry).SourceCabinet as *mut core::ffi::c_void,
                );
            }
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceRootPath as *mut core::ffi::c_void,
            );
            RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
            return (FALSE) as i32;
        }
        RtlStringCchCopyW(
            (*Entry).SourcePath,
            (Length + 1) as _,
            SourcePath,
        );
    }

    Length = wcslen(SourceFileName) as u32;
    (*Entry).SourceFileName = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
    ) as *mut u16;
    if (*Entry).SourceFileName.is_null() {
        if !(*Entry).SourcePath.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourcePath as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(
            ProcessHeap,
            0,
            (*Entry).SourceRootPath as *mut core::ffi::c_void,
        );
        if !(*Entry).SourceCabinet.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceCabinet as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as i32;
    }
    RtlStringCchCopyW(
        (*Entry).SourceFileName,
        (Length + 1) as _,
        SourceFileName,
    );

    Length = wcslen(TargetDirectory) as u32;
    if (Length > 0)
        && (*TargetDirectory.offset((Length - 1) as isize) as u16)
            == 0x005Cu16
    {
        Length -= 1;
    }
    (*Entry).TargetDirectory = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
    ) as *mut u16;
    if (*Entry).TargetDirectory.is_null() {
        RtlFreeHeap(
            ProcessHeap,
            0,
            (*Entry).SourceFileName as *mut core::ffi::c_void,
        );
        if !(*Entry).SourcePath.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourcePath as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(
            ProcessHeap,
            0,
            (*Entry).SourceRootPath as *mut core::ffi::c_void,
        );
        if !(*Entry).SourceCabinet.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceCabinet as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as i32;
    }
    RtlStringCchCopyW(
        (*Entry).TargetDirectory,
        (Length + 1) as _,
        TargetDirectory,
    );

    (*Entry).TargetFileName = core::ptr::null_mut();
    if !TargetFileName.is_null() {
        Length = wcslen(TargetFileName) as u32;
        (*Entry).TargetFileName = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as u32)) as SIZE_T,
        ) as *mut u16;
        if (*Entry).TargetFileName.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).TargetDirectory as *mut core::ffi::c_void,
            );
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceFileName as *mut core::ffi::c_void,
            );
            if !(*Entry).SourcePath.is_null() {
                RtlFreeHeap(
                    ProcessHeap,
                    0,
                    (*Entry).SourcePath as *mut core::ffi::c_void,
                );
            }
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceRootPath as *mut core::ffi::c_void,
            );
            if !(*Entry).SourceCabinet.is_null() {
                RtlFreeHeap(
                    ProcessHeap,
                    0,
                    (*Entry).SourceCabinet as *mut core::ffi::c_void,
                );
            }
            RtlFreeHeap(
                ProcessHeap,
                0,
                Entry as *mut core::ffi::c_void,
            );
            return (FALSE) as i32;
        }
        RtlStringCchCopyW(
            (*Entry).TargetFileName,
            (Length + 1) as _,
            TargetFileName,
        );
    }

    InsertTailList(
        &mut (*QueueHeader).CopyQueue as *mut _,
        &mut (*Entry).ListEntry as *mut _,
    );
    (*QueueHeader).CopyCount += 1;

    return (TRUE) as i32;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueDeleteW(
    QueueHandle: HSPFILEQ,
    PathPart1: PCWSTR,
    PathPart2: PCWSTR,
) -> BOOL {
    let mut QueueHeader: PFILEQUEUEHEADER;
    let mut Entry: PQUEUEENTRY;
    let mut Length: ULONG;

    if QueueHandle.is_null() || PathPart1.is_null() {
        return (FALSE) as BOOL;
    }

    QueueHeader = QueueHandle as PFILEQUEUEHEADER;

    DPRINT1!("SetupQueueDeleteW(PathPart1 '%S', PathPart2 '%S')\n", PathPart1, PathPart2);

    Entry = RtlAllocateHeap(ProcessHeap, 0, (core::mem::size_of::<QUEUEENTRY>()) as SIZE_T) as PQUEUEENTRY;
    if Entry.is_null() {
        return (FALSE) as BOOL;
    }

    RtlZeroMemory!(Entry as *mut core::ffi::c_void, core::mem::size_of::<QUEUEENTRY>());

    (*Entry).SourceCabinet = core::ptr::null_mut();
    (*Entry).SourceRootPath = core::ptr::null_mut();
    (*Entry).SourcePath = core::ptr::null_mut();
    (*Entry).SourceFileName = core::ptr::null_mut();

    Length = wcslen(PathPart1) as ULONG;
    (*Entry).TargetDirectory = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * core::mem::size_of::<WCHAR>() as ULONG) as SIZE_T,
    ) as PWSTR;
    if (*Entry).TargetDirectory.is_null() {
        RtlFreeHeap(ProcessHeap, 0, Entry as PVOID);
        return (FALSE) as BOOL;
    }
    RtlStringCchCopyW((*Entry).TargetDirectory, (Length + 1) as _, PathPart1);

    if !PathPart2.is_null() {
        Length = wcslen(PathPart2) as ULONG;
        (*Entry).TargetFileName = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * core::mem::size_of::<WCHAR>() as ULONG) as SIZE_T,
        ) as PWSTR;
        if (*Entry).TargetFileName.is_null() {
            RtlFreeHeap(ProcessHeap, 0, (*Entry).TargetDirectory as PVOID);
            RtlFreeHeap(ProcessHeap, 0, Entry as PVOID);
            return (FALSE) as BOOL;
        }
        RtlStringCchCopyW((*Entry).TargetFileName, (Length + 1) as _, PathPart2);
    }

    InsertTailList(&mut (*QueueHeader).DeleteQueue, &mut (*Entry).ListEntry);
    (*QueueHeader).DeleteCount += 1;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupQueueRenameW(
    QueueHandle: HSPFILEQ,
    SourcePath: PCWSTR,
    SourceFileName: PCWSTR,
    TargetPath: PCWSTR,
    TargetFileName: PCWSTR,
) -> BOOL {
    let mut QueueHeader: PFILEQUEUEHEADER;
    let mut Entry: PQUEUEENTRY;
    let mut Length: ULONG;

    if QueueHandle.is_null() || SourcePath.is_null() || TargetFileName.is_null() {
        return (FALSE) as BOOL;
    }

    QueueHeader = QueueHandle as PFILEQUEUEHEADER;

    DPRINT1!(
        b"SetupQueueRenameW(SrcPath '%S', SrcFN '%S' --> DstPath '%S', DstFN '%S')\n\0"
            .as_ptr() as *const i8,
        SourcePath,
        SourceFileName,
        TargetPath,
        TargetFileName,
    );

    Entry = RtlAllocateHeap(
        ProcessHeap,
        0,
        (core::mem::size_of::<QUEUEENTRY>()) as SIZE_T,
    ) as PQUEUEENTRY;
    if Entry.is_null() {
        return (FALSE) as BOOL;
    }

    RtlZeroMemory!(Entry as *mut core::ffi::c_void, core::mem::size_of::<QUEUEENTRY>());

    (*Entry).SourceCabinet = core::ptr::null_mut();
    (*Entry).SourceRootPath = core::ptr::null_mut();

    Length = (wcslen(SourcePath)) as _;
    if (Length > 0)
        && (*SourcePath.offset((Length - 1) as isize) as u16
            == 0x005Cu16)
    {
        Length -= 1;
    }
    (*Entry).SourcePath = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as ULONG)) as SIZE_T,
    ) as PWSTR;
    if (*Entry).SourcePath.is_null() {
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as BOOL;
    }
    RtlStringCchCopyW((*Entry).SourcePath, (Length + 1) as _, SourcePath);

    (*Entry).SourceFileName = core::ptr::null_mut();
    if !SourceFileName.is_null() {
        Length = (wcslen(SourceFileName)) as _;
        (*Entry).SourceFileName = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as ULONG)) as SIZE_T,
        ) as PWSTR;
        if (*Entry).SourceFileName.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourcePath as *mut core::ffi::c_void,
            );
            RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
            return (FALSE) as BOOL;
        }
        RtlStringCchCopyW((*Entry).SourceFileName, (Length + 1) as _, SourceFileName);
    }

    (*Entry).TargetDirectory = core::ptr::null_mut();
    if !TargetPath.is_null() {
        Length = (wcslen(TargetPath)) as _;
        if (Length > 0)
            && (*TargetPath.offset((Length - 1) as isize) as u16
                == 0x005Cu16)
        {
            Length -= 1;
        }
        (*Entry).TargetDirectory = RtlAllocateHeap(
            ProcessHeap,
            0,
            ((Length + 1) * (core::mem::size_of::<u16>() as ULONG)) as SIZE_T,
        ) as PWSTR;
        if (*Entry).TargetDirectory.is_null() {
            if !(*Entry).SourceFileName.is_null() {
                RtlFreeHeap(
                    ProcessHeap,
                    0,
                    (*Entry).SourceFileName as *mut core::ffi::c_void,
                );
            }
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourcePath as *mut core::ffi::c_void,
            );
            RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
            return (FALSE) as BOOL;
        }
        RtlStringCchCopyW((*Entry).TargetDirectory, (Length + 1) as _, TargetPath);
    }

    Length = (wcslen(TargetFileName)) as _;
    (*Entry).TargetFileName = RtlAllocateHeap(
        ProcessHeap,
        0,
        ((Length + 1) * (core::mem::size_of::<u16>() as ULONG)) as SIZE_T,
    ) as PWSTR;
    if (*Entry).TargetFileName.is_null() {
        if !(*Entry).TargetDirectory.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).TargetDirectory as *mut core::ffi::c_void,
            );
        }
        if !(*Entry).SourceFileName.is_null() {
            RtlFreeHeap(
                ProcessHeap,
                0,
                (*Entry).SourceFileName as *mut core::ffi::c_void,
            );
        }
        RtlFreeHeap(
            ProcessHeap,
            0,
            (*Entry).SourcePath as *mut core::ffi::c_void,
        );
        RtlFreeHeap(ProcessHeap, 0, Entry as *mut core::ffi::c_void);
        return (FALSE) as BOOL;
    }
    RtlStringCchCopyW((*Entry).TargetFileName, (Length + 1) as _, TargetFileName);

    InsertTailList(
        core::ptr::addr_of_mut!((*QueueHeader).RenameQueue),
        core::ptr::addr_of_mut!((*Entry).ListEntry),
    );
    (*QueueHeader).RenameCount += 1;

    return (TRUE) as BOOL;
}

#[no_mangle]
pub unsafe extern "system" fn SetupCommitFileQueueW(
    Owner: HWND,
    QueueHandle: HSPFILEQ,
    MsgHandler: PSP_FILE_CALLBACK_W,
    Context: PVOID,
) -> BOOL {
    let mut Success: BOOL = (TRUE) as i32;
    let mut Result: UINT;
    // C leaves Status uninitialized; the ABORT/SKIP paths of the first queue entry then
    // report it in Win32Error before any operation assigned it (a C bug). Zero here.
    let mut Status: NTSTATUS = 0;
    let mut QueueHeader: PFILEQUEUEHEADER;
    let mut ListEntry: PLIST_ENTRY;
    let mut Entry: PQUEUEENTRY;
    let mut FilePathInfo: FILEPATHS_W = core::mem::zeroed();
    let mut FileSrcPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];
    let mut FileDstPath: [WCHAR; (MAX_PATH) as _] = [0; (MAX_PATH) as _];

    if QueueHandle.is_null() {
        return (FALSE) as BOOL;
    }

    QueueHeader = QueueHandle as PFILEQUEUEHEADER;

    Result = (MsgHandler.unwrap_unchecked())(
        Context,
        SPFILENOTIFY_STARTQUEUE,
        Owner as UINT_PTR,
        0,
    );
    if Result == FILEOP_ABORT {
        return (FALSE) as BOOL;
    }

    'quit: {
        if !IsListEmpty(&mut (*QueueHeader).DeleteQueue) {
            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTSUBQUEUE,
                (FILEOP_DELETE) as _,
                ((*QueueHeader).DeleteCount) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                break 'quit;
            }
        }

        ListEntry = (*QueueHeader).DeleteQueue.Flink;
        while ListEntry != &mut (*QueueHeader).DeleteQueue {
            Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);

            CombinePaths(
                FileDstPath.as_mut_ptr(),
                (ARRAYSIZE!(FileDstPath)) as _,
                2,
                (*Entry).TargetDirectory,
                (*Entry).TargetFileName,
            );

            DPRINT1!(" -----> Delete: '%S'\n", FileDstPath.as_ptr());

            FilePathInfo.Target = FileDstPath.as_ptr();
            FilePathInfo.Source = core::ptr::null();
            FilePathInfo.Win32Error = (STATUS_SUCCESS) as u32;
            FilePathInfo.Flags = 0;

            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTDELETE,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                (FILEOP_DELETE) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                // goto EndDelete
            } else if Result == FILEOP_SKIP {
                // goto EndDelete
            } else {
                'retry_delete: loop {
                    Status = SetupDeleteFile(FileDstPath.as_ptr(), (TRUE) as BOOLEAN);
                    if ((NT_SUCCESS!(Status)) == 0) {
                        FilePathInfo.Win32Error = Status as UINT;
                        Result = (MsgHandler.unwrap_unchecked())(
                            Context,
                            SPFILENOTIFY_DELETEERROR,
                            &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                            0,
                        );
                        if Result == FILEOP_ABORT {
                            Success = (FALSE) as i32;
                            break 'retry_delete;
                        } else if Result == FILEOP_SKIP {
                            break 'retry_delete;
                        } else if Result == FILEOP_RETRY {
                            continue 'retry_delete;
                        }
                        Success = (FALSE) as i32;
                    }
                    break 'retry_delete;
                }
            }

            FilePathInfo.Win32Error = Status as UINT;
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDDELETE,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                0,
            );
            if (Success) as u32 == FALSE {
                break 'quit;
            }

            ListEntry = (*ListEntry).Flink;
        }

        if !IsListEmpty(&mut (*QueueHeader).DeleteQueue) {
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDSUBQUEUE,
                (FILEOP_DELETE) as _,
                0,
            );
        }

        if !IsListEmpty(&mut (*QueueHeader).RenameQueue) {
            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTSUBQUEUE,
                (FILEOP_RENAME) as _,
                ((*QueueHeader).RenameCount) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                break 'quit;
            }
        }

        ListEntry = (*QueueHeader).RenameQueue.Flink;
        while ListEntry != &mut (*QueueHeader).RenameQueue {
            Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);

            CombinePaths(
                FileSrcPath.as_mut_ptr(),
                (ARRAYSIZE!(FileSrcPath)) as _,
                2,
                (*Entry).SourcePath,
                (*Entry).SourceFileName,
            );

            CombinePaths(
                FileDstPath.as_mut_ptr(),
                (ARRAYSIZE!(FileDstPath)) as _,
                2,
                (*Entry).TargetDirectory,
                (*Entry).TargetFileName,
            );

            DPRINT1!(" -----> Rename: '%S' ==> '%S'\n", FileSrcPath.as_ptr(), FileDstPath.as_ptr());

            FilePathInfo.Target = FileDstPath.as_ptr();
            FilePathInfo.Source = FileSrcPath.as_ptr();
            FilePathInfo.Win32Error = (STATUS_SUCCESS) as u32;
            FilePathInfo.Flags = 0;

            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTRENAME,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                (FILEOP_RENAME) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                // goto EndRename
            } else if Result == FILEOP_SKIP {
                // goto EndRename
            } else {
                'retry_rename: loop {
                    Status = SetupMoveFile(
                        FileSrcPath.as_ptr(),
                        FileDstPath.as_ptr(),
                        MOVEFILE_REPLACE_EXISTING | MOVEFILE_COPY_ALLOWED | MOVEFILE_WRITE_THROUGH,
                    );
                    if ((NT_SUCCESS!(Status)) == 0) {
                        FilePathInfo.Win32Error = Status as UINT;
                        Result = (MsgHandler.unwrap_unchecked())(
                            Context,
                            SPFILENOTIFY_RENAMEERROR,
                            &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                            0,
                        );
                        if Result == FILEOP_ABORT {
                            Success = (FALSE) as i32;
                            break 'retry_rename;
                        } else if Result == FILEOP_SKIP {
                            break 'retry_rename;
                        } else if Result == FILEOP_RETRY {
                            continue 'retry_rename;
                        }
                        Success = (FALSE) as i32;
                    }
                    break 'retry_rename;
                }
            }

            FilePathInfo.Win32Error = Status as UINT;
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDRENAME,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                0,
            );
            if (Success) as u32 == FALSE {
                break 'quit;
            }

            ListEntry = (*ListEntry).Flink;
        }

        if !IsListEmpty(&mut (*QueueHeader).RenameQueue) {
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDSUBQUEUE,
                (FILEOP_RENAME) as _,
                0,
            );
        }

        if !IsListEmpty(&mut (*QueueHeader).CopyQueue) {
            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTSUBQUEUE,
                (FILEOP_COPY) as _,
                ((*QueueHeader).CopyCount) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                break 'quit;
            }
        }

        ListEntry = (*QueueHeader).CopyQueue.Flink;
        while ListEntry != &mut (*QueueHeader).CopyQueue {
            Entry = CONTAINING_RECORD!(ListEntry, QUEUEENTRY, ListEntry);

            if (*Entry).SourceCabinet.is_null() {
                CombinePaths(
                    FileSrcPath.as_mut_ptr(),
                    (ARRAYSIZE!(FileSrcPath)) as _,
                    3,
                    (*Entry).SourceRootPath,
                    (*Entry).SourcePath,
                    (*Entry).SourceFileName,
                );
            } else {
                CombinePaths(
                    FileSrcPath.as_mut_ptr(),
                    (ARRAYSIZE!(FileSrcPath)) as _,
                    3,
                    (*Entry).SourceRootPath,
                    (*Entry).SourcePath,
                    (*Entry).SourceCabinet,
                );
            }

            RtlStringCchCopyW(
                FileDstPath.as_mut_ptr(),
                (ARRAYSIZE!(FileDstPath)) as _,
                (*Entry).TargetDirectory,
            );
            if (*Entry).SourceCabinet.is_null() {
                if !(*Entry).TargetFileName.is_null() {
                    ConcatPaths(
                        FileDstPath.as_mut_ptr(),
                        (ARRAYSIZE!(FileDstPath)) as _,
                        1,
                        (*Entry).TargetFileName,
                    );
                } else {
                    ConcatPaths(
                        FileDstPath.as_mut_ptr(),
                        (ARRAYSIZE!(FileDstPath)) as _,
                        1,
                        (*Entry).SourceFileName,
                    );
                }
            } else {
                ConcatPaths(
                    FileDstPath.as_mut_ptr(),
                    (ARRAYSIZE!(FileDstPath)) as _,
                    1,
                    (*Entry).SourceFileName,
                );
            }

            DPRINT!(" -----> Copy: '%S' ==> '%S'\n", FileSrcPath.as_ptr(), FileDstPath.as_ptr());

            FilePathInfo.Target = FileDstPath.as_ptr();
            FilePathInfo.Source = FileSrcPath.as_ptr();
            FilePathInfo.Win32Error = (STATUS_SUCCESS) as u32;
            FilePathInfo.Flags = 0;

            Result = (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_STARTCOPY,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                (FILEOP_COPY) as _,
            );
            if Result == FILEOP_ABORT {
                Success = (FALSE) as i32;
                // goto EndCopy
            } else if Result == FILEOP_SKIP {
                // goto EndCopy
            } else {
                'retry_copy: loop {
                    if !(*Entry).SourceCabinet.is_null() {
                        Status = SetupExtractFile(
                            QueueHeader,
                            FileSrcPath.as_ptr(),
                            (*Entry).SourceFileName,
                            (*Entry).TargetDirectory,
                        );
                    } else {
                        Status = SetupCopyFile(
                            FileSrcPath.as_ptr(),
                            FileDstPath.as_ptr(),
                            (FALSE) as BOOLEAN,
                        );
                    }

                    if ((NT_SUCCESS!(Status)) == 0) {
                        FilePathInfo.Win32Error = Status as UINT;
                        Result = (MsgHandler.unwrap_unchecked())(
                            Context,
                            SPFILENOTIFY_COPYERROR,
                            &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                            core::ptr::null_mut::<c_void>() as UINT_PTR,
                        );
                        if Result == FILEOP_ABORT {
                            Success = (FALSE) as i32;
                            break 'retry_copy;
                        } else if Result == FILEOP_SKIP {
                            break 'retry_copy;
                        } else if Result == FILEOP_RETRY {
                            continue 'retry_copy;
                        } else if Result == FILEOP_NEWPATH {
                            continue 'retry_copy;
                        }
                        Success = (FALSE) as i32;
                    }
                    break 'retry_copy;
                }
            }

            FilePathInfo.Win32Error = Status as UINT;
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDCOPY,
                &mut FilePathInfo as *mut FILEPATHS_W as UINT_PTR,
                0,
            );
            if (Success) as u32 == FALSE {
                break 'quit;
            }

            ListEntry = (*ListEntry).Flink;
        }

        if !IsListEmpty(&mut (*QueueHeader).CopyQueue) {
            (MsgHandler.unwrap_unchecked())(
                Context,
                SPFILENOTIFY_ENDSUBQUEUE,
                (FILEOP_COPY) as _,
                0,
            );
        }
    }

    (MsgHandler.unwrap_unchecked())(
        Context,
        SPFILENOTIFY_ENDQUEUE,
        Success as UINT_PTR,
        0,
    );

    Success
}
