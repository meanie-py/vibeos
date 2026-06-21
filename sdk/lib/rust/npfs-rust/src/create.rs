//! create.rs — create.c: Pipes Creation. NpFsdCreate / NpFsdCreateNamedPipe dispatch plus
//! the helpers NpCheckForNotify, NpOpenNamedPipeFileSystem, NpOpenNamedPipeRootDirectory,
//! NpCreateClientEnd, NpTranslateAlias, NpCreateExistingNamedPipe, NpCreateNewNamedPipe.
//! Faithful translation of the ReactOS NPFS create path.
#![allow(non_snake_case, non_camel_case_types, dead_code, unused_imports, unused_variables, unused_mut, unused_parens, unused_assignments)]

use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut, null_mut};

use ntkrnl_sys::base::{
    BOOLEAN, InitializeListHead, InsertTailList, IsListEmpty, LONG, NTSTATUS, NT_SUCCESS,
    PVOID, RemoveHeadList, SIZE_T, UCHAR, ULONG, ULONG_PTR, USHORT,
};
use ntkrnl_sys::ddk::*;
use ntkrnl_sys::driver_ddk::IoCompleteRequest;
use ntkrnl_sys::ex::ExFreePoolWithTag;
use ntkrnl_sys::inline::common::{FsRtlEnterFileSystem, FsRtlExitFileSystem};
use ntkrnl_sys::inline::tier1::{IoGetCurrentIrpStackLocation, IoSetCancelRoutine};

use crate::glue::*;
use crate::globals::*;
use crate::helpers::*;
use crate::types::*;
use crate::CONTAINING_RECORD;

// Cross-module calls (see function->module map):
use crate::fileobsup::{NpDecodeFileObject, NpSetFileObject};
use crate::prefxsup::{NpFindPrefix, NpFindRelativePrefix};
use crate::secursup::{NpInitializeSecurity, NpUninitializeSecurity};
use crate::statesup::NpSetConnectedPipeState;
use crate::strucsup::{NpCreateCcb, NpCreateFcb, NpCreateRootDcbCcb, NpDeleteCcb, NpDeleteFcb};
use crate::waitsup::NpCancelWaiter;
use crate::main::NpCompareAliasNames;

// ---- constants NOT in glue.rs (cited from SDK headers; FLAG for promotion) ----
// xdk/iotypes.h: FILE_OPENED 0x00000001
const FILE_OPENED: ULONG = 0x0000_0001;
// xdk/iotypes.h: FILE_CREATED 0x00000002
const FILE_CREATED: ULONG = 0x0000_0002;
// xdk/iotypes.h: FILE_OPEN 0x00000001
const FILE_OPEN: ULONG = 0x0000_0001;
// xdk/iotypes.h: FILE_CREATE 0x00000002
const FILE_CREATE: ULONG = 0x0000_0002;
// xdk/iotypes.h: SL_FORCE_ACCESS_CHECK 0x01
const SL_FORCE_ACCESS_CHECK: UCHAR = 0x01;
// xdk/iotypes.h: IO_NO_INCREMENT 0
const IO_NO_INCREMENT: i8 = 0;
// xdk/setypes.h: MAXIMUM_ALLOWED 0x02000000
const MAXIMUM_ALLOWED: ULONG = 0x0200_0000;
// xdk/iotypes.h: FILE_READ_DATA 0x00000001
const FILE_READ_DATA: ULONG = 0x0000_0001;
// xdk/iotypes.h: FILE_WRITE_DATA 0x00000002
const FILE_WRITE_DATA: ULONG = 0x0000_0002;
// xdk/iotypes.h: FILE_SHARE_READ 0x00000001
const FILE_SHARE_READ: ULONG = 0x0000_0001;
// xdk/iotypes.h: FILE_SHARE_WRITE 0x00000002
const FILE_SHARE_WRITE: ULONG = 0x0000_0002;
// xdk/obtypes.h: OBJ_NAME_PATH_SEPARATOR ((WCHAR)L'\\') == 0x5C
const OBJ_NAME_PATH_SEPARATOR: u16 = b'\\' as u16;

// L"NamedPipe" wide literal for the audit-alarm object type name.
const NAMED_PIPE_W: [u16; 10] = [
    b'N' as u16, b'a' as u16, b'm' as u16, b'e' as u16, b'd' as u16,
    b'P' as u16, b'i' as u16, b'p' as u16, b'e' as u16, 0,
];

pub unsafe extern "system" fn NpCheckForNotify(
    Dcb: PNP_DCB,
    SecondList: BOOLEAN,
    List: *mut LIST_ENTRY,
) {
    let mut NextEntry: *mut LIST_ENTRY;
    let mut ListHead: *mut LIST_ENTRY;
    let mut Irp: PIRP;
    let mut i: ULONG;

    ListHead = addr_of_mut!((*Dcb).NotifyList);
    i = 0;
    while i < 2 {
        // ASSERT(IsListEmpty(ListHead));
        while !IsListEmpty(ListHead) {
            NextEntry = RemoveHeadList(ListHead);

            Irp = irp_from_list_entry(NextEntry);

            // C: if (IoSetCancelRoutine(Irp, NULL)) -> "previous cancel routine non-NULL"
            if IoSetCancelRoutine(Irp, None).is_some() {
                (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
                InsertTailList(List, NextEntry);
            } else {
                InitializeListHead(NextEntry);
            }
        }

        if SecondList == 0 {
            break;
        }
        ListHead = addr_of_mut!((*Dcb).NotifyList2);
        i += 1;
    }
}

pub unsafe extern "system" fn NpOpenNamedPipeFileSystem(
    FileObject: PFILE_OBJECT,
    DesiredAccess: ACCESS_MASK,
) -> IO_STATUS_BLOCK {
    let mut Status: IO_STATUS_BLOCK = core::mem::zeroed();

    NpSetFileObject(FileObject, NpVcb as PVOID, null_mut(), FALSE as ULONG);
    (*NpVcb).ReferenceCount += 1;

    Status.Information = FILE_OPENED as _;
    Status.__bindgen_anon_1.Status = STATUS_SUCCESS;
    Status
}

pub unsafe extern "system" fn NpOpenNamedPipeRootDirectory(
    Dcb: PNP_DCB,
    FileObject: PFILE_OBJECT,
    DesiredAccess: ACCESS_MASK,
    List: *mut LIST_ENTRY,
) -> IO_STATUS_BLOCK {
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut Ccb: PNP_ROOT_DCB_FCB = null_mut();

    IoStatus.__bindgen_anon_1.Status = NpCreateRootDcbCcb(addr_of_mut!(Ccb));
    if NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
        NpSetFileObject(FileObject, Dcb as PVOID, Ccb as PVOID, FALSE as ULONG);
        (*Dcb).CurrentInstances += 1;

        IoStatus.Information = FILE_OPENED as _;
        IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    } else {
        IoStatus.Information = 0;
    }

    IoStatus
}

pub unsafe extern "system" fn NpCreateClientEnd(
    Fcb: PNP_FCB,
    FileObject: PFILE_OBJECT,
    DesiredAccess: ACCESS_MASK,
    mut SecurityQos: PSECURITY_QUALITY_OF_SERVICE,
    AccessState: PACCESS_STATE,
    PreviousMode: KPROCESSOR_MODE,
    Thread: PETHREAD,
    List: *mut LIST_ENTRY,
) -> IO_STATUS_BLOCK {
    let SubjectSecurityContext: PSECURITY_SUBJECT_CONTEXT;
    let mut AccessGranted: BOOLEAN;
    let mut GrantedAccess: ACCESS_MASK = 0;
    let mut Privileges: PPRIVILEGE_SET;
    let mut ObjectTypeName: UNICODE_STRING = core::mem::zeroed();
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let NamedPipeConfiguration: USHORT;
    let mut NextEntry: *mut LIST_ENTRY;
    let ListHead: *mut LIST_ENTRY;
    let mut Ccb: PNP_CCB = null_mut();

    IoStatus.Information = 0;
    Privileges = null_mut();

    NamedPipeConfiguration = (*Fcb).NamedPipeConfiguration;

    SubjectSecurityContext = addr_of_mut!((*AccessState).SubjectSecurityContext);
    SeLockSubjectContext(SubjectSecurityContext);

    AccessGranted = SeAccessCheck(
        (*Fcb).SecurityDescriptor,
        SubjectSecurityContext,
        TRUE,
        DesiredAccess & !4,
        0,
        addr_of_mut!(Privileges),
        IoGetFileObjectGenericMapping(),
        PreviousMode,
        addr_of_mut!(GrantedAccess),
        addr_of_mut!(IoStatus.__bindgen_anon_1.Status),
    );

    if !Privileges.is_null() {
        SeAppendPrivileges(AccessState, Privileges);
        SeFreePrivileges(Privileges);
    }

    if AccessGranted != 0 {
        (*AccessState).PreviouslyGrantedAccess |= GrantedAccess;
        (*AccessState).RemainingDesiredAccess &= !(GrantedAccess | MAXIMUM_ALLOWED);
    }

    ObjectTypeName.Buffer = NAMED_PIPE_W.as_ptr() as *mut u16;
    ObjectTypeName.Length = 18;
    SeOpenObjectAuditAlarm(
        addr_of_mut!(ObjectTypeName),
        null_mut(),
        addr_of_mut!((*FileObject).FileName),
        (*Fcb).SecurityDescriptor,
        AccessState,
        FALSE,
        AccessGranted,
        PreviousMode,
        addr_of_mut!((*AccessState).GenerateOnClose),
    );
    SeUnlockSubjectContext(SubjectSecurityContext);
    if AccessGranted == 0 {
        return IoStatus;
    }

    if ((GrantedAccess & FILE_READ_DATA) != 0
        && (NamedPipeConfiguration as ULONG == FILE_PIPE_INBOUND))
        || ((GrantedAccess & FILE_WRITE_DATA) != 0
            && (NamedPipeConfiguration as ULONG == FILE_PIPE_OUTBOUND))
    {
        IoStatus.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
        return IoStatus;
    }

    if (GrantedAccess & (FILE_READ_DATA | FILE_WRITE_DATA)) == 0 {
        SecurityQos = null_mut();
    }

    ListHead = addr_of_mut!((*Fcb).CcbList);
    NextEntry = (*ListHead).Flink;
    while NextEntry != ListHead {
        Ccb = CONTAINING_RECORD!(NextEntry, NP_CCB, CcbEntry);
        if (*Ccb).NamedPipeState as ULONG == FILE_PIPE_LISTENING_STATE {
            break;
        }

        NextEntry = (*NextEntry).Flink;
    }

    if NextEntry == ListHead {
        IoStatus.__bindgen_anon_1.Status = STATUS_PIPE_NOT_AVAILABLE;
        return IoStatus;
    }

    IoStatus.__bindgen_anon_1.Status = NpInitializeSecurity(Ccb, SecurityQos, Thread);
    if !NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
        return IoStatus;
    }

    IoStatus.__bindgen_anon_1.Status = NpSetConnectedPipeState(Ccb, FileObject, List);
    if !NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
        NpUninitializeSecurity(Ccb);
        return IoStatus;
    }

    (*Ccb).ClientSession = null_mut();
    (*Ccb).Process = IoThreadToProcess(Thread);

    IoStatus.Information = FILE_OPENED as _;
    IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoStatus
}

pub unsafe extern "system" fn NpTranslateAlias(PipeName: PUNICODE_STRING) -> NTSTATUS {
    let mut UpcaseBuffer: [u16; MAX_INDEXED_LENGTH + 1] = [0; MAX_INDEXED_LENGTH + 1];
    let mut UpcaseString: UNICODE_STRING = core::mem::zeroed();
    let mut Length: ULONG;
    let mut CurrentAlias: PNPFS_ALIAS;
    let mut Status: NTSTATUS;
    let mut BufferAllocated: BOOLEAN;
    let BackSlash: BOOLEAN;
    let mut Result: LONG;

    /* Get the pipe name length and check for empty string */
    Length = (*PipeName).Length as ULONG;
    if Length == 0 {
        return STATUS_SUCCESS;
    }

    /* Check if the name starts with a path separator */
    BackSlash = (*(*PipeName).Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR) as BOOLEAN;
    if BackSlash != 0 {
        /* We are only interested in the part after the backslash */
        Length -= size_of::<u16>() as ULONG;
    }

    /* Check if the length is within our indexed list bounds */
    if (Length >= (MIN_INDEXED_LENGTH * size_of::<u16>()) as ULONG)
        && (Length <= (MAX_INDEXED_LENGTH * size_of::<u16>()) as ULONG)
    {
        /* Length is within bounds, use the list by length */
        CurrentAlias = NpAliasListByLength
            [(Length as usize / size_of::<u16>()) - MIN_INDEXED_LENGTH];
    } else {
        /* We use the generic list, search for an entry of the right size */
        CurrentAlias = NpAliasList;
        while !CurrentAlias.is_null() && ((*CurrentAlias).Name.Length as ULONG != Length) {
            /* Check if we went past the desired length */
            if (*CurrentAlias).Name.Length as ULONG > Length {
                /* In this case there is no matching alias, return success */
                return STATUS_SUCCESS;
            }

            /* Go to the next alias in the list */
            CurrentAlias = (*CurrentAlias).Next;
        }
    }

    /* Did we find any alias? */
    if CurrentAlias.is_null() {
        /* Nothing found, no matching alias */
        return STATUS_SUCCESS;
    }

    /* Check whether we can use our stack buffer */
    if Length <= (MAX_INDEXED_LENGTH * size_of::<u16>()) as ULONG {
        /* Initialize the upcased string */
        UpcaseString.Buffer = UpcaseBuffer.as_mut_ptr();
        UpcaseString.MaximumLength = size_of::<[u16; MAX_INDEXED_LENGTH + 1]>() as USHORT;

        /* Upcase the pipe name */
        Status = RtlUpcaseUnicodeString(
            addr_of_mut!(UpcaseString),
            PipeName as PCUNICODE_STRING,
            FALSE,
        );
        // NT_ASSERT(NT_SUCCESS(Status));
        BufferAllocated = FALSE;
    } else {
        /* Upcase the pipe name, allocate the string buffer */
        Status = RtlUpcaseUnicodeString(
            addr_of_mut!(UpcaseString),
            PipeName as PCUNICODE_STRING,
            TRUE,
        );
        if !NT_SUCCESS(Status) {
            return Status;
        }

        BufferAllocated = TRUE;
    }

    /* Did the original name start with a backslash? */
    if BackSlash != 0 {
        /* Skip it for the comparison */
        UpcaseString.Buffer = UpcaseString.Buffer.offset(1);
        UpcaseString.Length -= size_of::<u16>() as USHORT;
    }

    /* Make sure the length matches the "raw" length */
    // NT_ASSERT(UpcaseString.Length == Length);
    // NT_ASSERT(CurrentAlias->Name.Length == Length);

    /* Loop while we have aliases */
    loop {
        /* Compare the names and check if they match */
        Result = NpCompareAliasNames(
            addr_of!(UpcaseString) as PCUNICODE_STRING,
            addr_of!((*CurrentAlias).Name) as PCUNICODE_STRING,
        );
        if Result == 0 {
            /* The names match, use the target name */
            *PipeName = *(*CurrentAlias).TargetName;

            /* Did the original name start with a backslash? */
            if BackSlash == 0 {
                /* It didn't, so skip it in the target name as well */
                (*PipeName).Buffer = (*PipeName).Buffer.offset(1);
                (*PipeName).Length -= size_of::<u16>() as USHORT;
            }
            break;
        }

        /* Check if we went past all string candidates */
        if Result < 0 {
            /* Nothing found, we're done */
            break;
        }

        /* Go to the next alias */
        CurrentAlias = (*CurrentAlias).Next;

        /* Keep looping while we have aliases of the right length */
        if CurrentAlias.is_null() || ((*CurrentAlias).Name.Length as ULONG != Length) {
            break;
        }
    }

    /* Did we allocate a buffer? */
    if BufferAllocated != 0 {
        /* Free the allocated buffer */
        // ASSERT(UpcaseString.Buffer != UpcaseBuffer);
        RtlFreeUnicodeString(addr_of_mut!(UpcaseString));
    }

    STATUS_SUCCESS
}

pub unsafe extern "system" fn NpFsdCreate(DeviceObject: PDEVICE_OBJECT, Irp: PIRP) -> NTSTATUS {
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let IoStack: PIO_STACK_LOCATION;
    let mut FileName: UNICODE_STRING;
    let FileObject: PFILE_OBJECT;
    let RelatedFileObject: PFILE_OBJECT;
    let mut Type: NODE_TYPE_CODE;
    let mut Ccb: PNP_CCB = null_mut();
    let mut Fcb: PNP_FCB = null_mut();
    let mut Dcb: PNP_DCB;
    let DesiredAccess: ACCESS_MASK;
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    let mut Prefix: UNICODE_STRING = core::mem::zeroed();

    InitializeListHead(addr_of_mut!(DeferredList));
    IoStack = IoGetCurrentIrpStackLocation(Irp);
    FileObject = (*IoStack).FileObject;
    RelatedFileObject = (*FileObject).RelatedFileObject;
    FileName = (*FileObject).FileName;
    DesiredAccess =
        (*(*IoStack).Parameters.CreatePipe.SecurityContext).DesiredAccess;

    IoStatus.Information = 0;

    FsRtlEnterFileSystem();
    NpAcquireExclusiveVcb();

    if !RelatedFileObject.is_null() {
        Type = NpDecodeFileObject(
            RelatedFileObject,
            addr_of_mut!(Fcb) as *mut PVOID,
            addr_of_mut!(Ccb),
            null_mut(),
        );
    } else {
        Type = 0;
        Fcb = null_mut();
        Ccb = null_mut();
    }

    'quickie: {
        if FileName.Length != 0 {
            if (FileName.Length as usize == size_of::<u16>())
                && (*FileName.Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR)
                && RelatedFileObject.is_null()
            {
                IoStatus = NpOpenNamedPipeRootDirectory(
                    (*NpVcb).RootDcb,
                    FileObject,
                    DesiredAccess,
                    addr_of_mut!(DeferredList),
                );
                break 'quickie;
            }
        } else if RelatedFileObject.is_null() || (Type == NPFS_NTC_VCB) {
            IoStatus = NpOpenNamedPipeFileSystem(FileObject, DesiredAccess);
            break 'quickie;
        } else if Type == NPFS_NTC_ROOT_DCB {
            IoStatus = NpOpenNamedPipeRootDirectory(
                (*NpVcb).RootDcb,
                FileObject,
                DesiredAccess,
                addr_of_mut!(DeferredList),
            );
            break 'quickie;
        }

        IoStatus.__bindgen_anon_1.Status = NpTranslateAlias(addr_of_mut!(FileName));
        if !NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
            break 'quickie;
        }

        if !RelatedFileObject.is_null() {
            if Type == NPFS_NTC_ROOT_DCB {
                Dcb = Ccb as PNP_DCB;
                IoStatus.__bindgen_anon_1.Status = NpFindRelativePrefix(
                    Dcb,
                    addr_of_mut!(FileName),
                    1,
                    addr_of_mut!(Prefix),
                    addr_of_mut!(Fcb),
                );
                if !NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
                    break 'quickie;
                }
            } else if (Type != NPFS_NTC_CCB) || (FileName.Length != 0) {
                IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                break 'quickie;
            } else {
                Prefix.Length = 0;
            }
        } else {
            if (FileName.Length as usize <= size_of::<u16>())
                || (*FileName.Buffer.offset(0) != OBJ_NAME_PATH_SEPARATOR)
            {
                IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                break 'quickie;
            }

            Fcb = NpFindPrefix(addr_of_mut!(FileName), 1, addr_of_mut!(Prefix));
        }

        if Prefix.Length != 0 {
            IoStatus.__bindgen_anon_1.Status = if (*Fcb).NodeType != NPFS_NTC_FCB {
                STATUS_OBJECT_NAME_NOT_FOUND
            } else {
                STATUS_OBJECT_NAME_INVALID
            };
            break 'quickie;
        }

        if (*Fcb).NodeType != NPFS_NTC_FCB {
            IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
            break 'quickie;
        }

        if (*Fcb).ServerOpenCount == 0 {
            IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_NOT_FOUND;
            break 'quickie;
        }

        let prev_mode: KPROCESSOR_MODE = if ((*IoStack).Flags & SL_FORCE_ACCESS_CHECK) != 0 {
            UserMode
        } else {
            (*Irp).RequestorMode
        };

        IoStatus = NpCreateClientEnd(
            Fcb,
            FileObject,
            DesiredAccess,
            (*(*IoStack).Parameters.CreatePipe.SecurityContext).SecurityQos,
            (*(*IoStack).Parameters.CreatePipe.SecurityContext).AccessState,
            prev_mode,
            (*Irp).Tail.Overlay.Thread,
            addr_of_mut!(DeferredList),
        );
    } // 'quickie

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));
    FsRtlExitFileSystem();

    (*Irp).IoStatus = IoStatus;
    IoCompleteRequest(Irp, IO_NO_INCREMENT);
    IoStatus.__bindgen_anon_1.Status
}

pub unsafe extern "system" fn NpCreateExistingNamedPipe(
    Fcb: PNP_FCB,
    FileObject: PFILE_OBJECT,
    DesiredAccess: ACCESS_MASK,
    AccessState: PACCESS_STATE,
    PreviousMode: KPROCESSOR_MODE,
    Disposition: ULONG,
    ShareAccess: ULONG,
    Parameters: PNAMED_PIPE_CREATE_PARAMETERS,
    Process: PEPROCESS,
    List: *mut LIST_ENTRY,
) -> IO_STATUS_BLOCK {
    let SubjectSecurityContext: PSECURITY_SUBJECT_CONTEXT;
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectTypeName: UNICODE_STRING = core::mem::zeroed();
    let mut GrantedAccess: ACCESS_MASK = 0;
    let mut Ccb: PNP_CCB = null_mut();
    let mut Privileges: PPRIVILEGE_SET;
    let NamedPipeConfiguration: USHORT;
    let mut CheckShareAccess: USHORT;
    let AccessGranted: BOOLEAN;

    Privileges = null_mut();

    NamedPipeConfiguration = (*Fcb).NamedPipeConfiguration;

    SubjectSecurityContext = addr_of_mut!((*AccessState).SubjectSecurityContext);
    SeLockSubjectContext(SubjectSecurityContext);

    IoStatus.Information = 0;

    AccessGranted = SeAccessCheck(
        (*Fcb).SecurityDescriptor,
        SubjectSecurityContext,
        TRUE,
        DesiredAccess | 4,
        0,
        addr_of_mut!(Privileges),
        IoGetFileObjectGenericMapping(),
        PreviousMode,
        addr_of_mut!(GrantedAccess),
        addr_of_mut!(IoStatus.__bindgen_anon_1.Status),
    );

    if !Privileges.is_null() {
        SeAppendPrivileges(AccessState, Privileges);
        SeFreePrivileges(Privileges);
    }

    if AccessGranted != 0 {
        (*AccessState).PreviouslyGrantedAccess |= GrantedAccess;
        (*AccessState).RemainingDesiredAccess &= !(GrantedAccess | MAXIMUM_ALLOWED);
    }

    ObjectTypeName.Buffer = NAMED_PIPE_W.as_ptr() as *mut u16;
    ObjectTypeName.Length = 18;
    SeOpenObjectAuditAlarm(
        addr_of_mut!(ObjectTypeName),
        null_mut(),
        addr_of_mut!((*FileObject).FileName),
        (*Fcb).SecurityDescriptor,
        AccessState,
        FALSE,
        AccessGranted,
        PreviousMode,
        addr_of_mut!((*AccessState).GenerateOnClose),
    );

    SeUnlockSubjectContext(SubjectSecurityContext);
    if AccessGranted == 0 {
        return IoStatus;
    }

    if (*Fcb).CurrentInstances >= (*Fcb).MaximumInstances {
        IoStatus.__bindgen_anon_1.Status = STATUS_INSTANCE_NOT_AVAILABLE;
        return IoStatus;
    }

    if Disposition == FILE_CREATE {
        IoStatus.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
        return IoStatus;
    }

    CheckShareAccess = 0;
    if NamedPipeConfiguration as ULONG == FILE_PIPE_FULL_DUPLEX {
        CheckShareAccess = (FILE_SHARE_READ | FILE_SHARE_WRITE) as USHORT;
    } else if NamedPipeConfiguration as ULONG == FILE_PIPE_OUTBOUND {
        CheckShareAccess = FILE_SHARE_READ as USHORT;
    } else if NamedPipeConfiguration as ULONG == FILE_PIPE_INBOUND {
        CheckShareAccess = FILE_SHARE_WRITE as USHORT;
    }

    if CheckShareAccess as ULONG != ShareAccess {
        IoStatus.__bindgen_anon_1.Status = STATUS_ACCESS_DENIED;
        return IoStatus;
    }

    IoStatus.__bindgen_anon_1.Status = NpCreateCcb(
        Fcb,
        FileObject,
        FILE_PIPE_LISTENING_STATE as UCHAR,
        ((*Parameters).ReadMode & 0xFF) as UCHAR,
        ((*Parameters).CompletionMode & 0xFF) as UCHAR,
        (*Parameters).InboundQuota,
        (*Parameters).OutboundQuota,
        addr_of_mut!(Ccb),
    );
    if !NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
        return IoStatus;
    }

    IoStatus.__bindgen_anon_1.Status = NpCancelWaiter(
        addr_of_mut!((*NpVcb).WaitQueue),
        addr_of_mut!((*Fcb).FullName),
        FALSE as NTSTATUS,
        List,
    );
    if !NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
        (*(*Ccb).Fcb).CurrentInstances -= 1;
        NpDeleteCcb(Ccb, List);
        return IoStatus;
    }

    NpSetFileObject(FileObject, Ccb as PVOID, (*Ccb).NonPagedCcb as PVOID, TRUE as ULONG);
    (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize] = FileObject;
    NpCheckForNotify((*Fcb).ParentDcb as PNP_DCB, 0, List);

    IoStatus.__bindgen_anon_1.Status = STATUS_SUCCESS;
    IoStatus.Information = FILE_OPENED as _;
    IoStatus
}

pub unsafe extern "system" fn NpCreateNewNamedPipe(
    Dcb: PNP_DCB,
    FileObject: PFILE_OBJECT,
    PipeName: UNICODE_STRING,
    DesiredAccess: ACCESS_MASK,
    AccessState: PACCESS_STATE,
    Disposition: USHORT,
    ShareAccess: USHORT,
    Parameters: PNAMED_PIPE_CREATE_PARAMETERS,
    Process: PEPROCESS,
    List: *mut LIST_ENTRY,
    IoStatus: PIO_STATUS_BLOCK,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let NamedPipeConfiguration: USHORT;
    let SecurityContext: PSECURITY_SUBJECT_CONTEXT;
    let mut SecurityDescriptor: PSECURITY_DESCRIPTOR = null_mut();
    let mut CachedSecurityDescriptor: PSECURITY_DESCRIPTOR = null_mut();
    let mut Ccb: PNP_CCB = null_mut();
    let mut Fcb: PNP_FCB = null_mut();
    // mutable local copy of the by-value PipeName (C passes it IN by value)
    let mut PipeName = PipeName;

    'quickie: {
        if ((*Parameters).TimeoutSpecified == 0)
            || ((*Parameters).MaximumInstances == 0)
            || ((*Parameters).DefaultTimeout.QuadPart >= 0)
        {
            Status = STATUS_INVALID_PARAMETER;
            break 'quickie;
        }

        if Disposition as ULONG == FILE_OPEN {
            Status = STATUS_OBJECT_NAME_NOT_FOUND;
            break 'quickie;
        }

        if ShareAccess as ULONG == (FILE_SHARE_READ | FILE_SHARE_WRITE) {
            NamedPipeConfiguration = FILE_PIPE_FULL_DUPLEX as USHORT;
        } else if ShareAccess as ULONG == FILE_SHARE_READ {
            NamedPipeConfiguration = FILE_PIPE_OUTBOUND as USHORT;
        } else if ShareAccess as ULONG == FILE_SHARE_WRITE {
            NamedPipeConfiguration = FILE_PIPE_INBOUND as USHORT;
        } else {
            Status = STATUS_INVALID_PARAMETER;
            break 'quickie;
        }

        if (*Parameters).NamedPipeType == FILE_PIPE_BYTE_STREAM_TYPE
            && (*Parameters).ReadMode == FILE_PIPE_MESSAGE_MODE
        {
            Status = STATUS_INVALID_PARAMETER;
            break 'quickie;
        }

        Status = NpCreateFcb(
            Dcb,
            addr_of_mut!(PipeName),
            (*Parameters).MaximumInstances,
            (*Parameters).DefaultTimeout,
            NamedPipeConfiguration,
            ((*Parameters).NamedPipeType & 0xFFFF) as USHORT,
            addr_of_mut!(Fcb),
        );
        if !NT_SUCCESS(Status) {
            break 'quickie;
        }

        Status = NpCreateCcb(
            Fcb,
            FileObject,
            FILE_PIPE_LISTENING_STATE as UCHAR,
            ((*Parameters).ReadMode & 0xFF) as UCHAR,
            ((*Parameters).CompletionMode & 0xFF) as UCHAR,
            (*Parameters).InboundQuota,
            (*Parameters).OutboundQuota,
            addr_of_mut!(Ccb),
        );
        if !NT_SUCCESS(Status) {
            NpDeleteFcb(Fcb, List);
            break 'quickie;
        }

        SecurityContext = addr_of_mut!((*AccessState).SubjectSecurityContext);
        SeLockSubjectContext(SecurityContext);

        Status = SeAssignSecurity(
            null_mut(),
            (*AccessState).SecurityDescriptor,
            addr_of_mut!(SecurityDescriptor),
            FALSE,
            SecurityContext,
            IoGetFileObjectGenericMapping(),
            PagedPool,
        );
        SeUnlockSubjectContext(SecurityContext);
        if !NT_SUCCESS(Status) {
            NpDeleteCcb(Ccb, List);
            NpDeleteFcb(Fcb, List);
            break 'quickie;
        }

        Status = ObLogSecurityDescriptor(
            SecurityDescriptor,
            addr_of_mut!(CachedSecurityDescriptor),
            1,
        );
        ExFreePoolWithTag(SecurityDescriptor, 0);

        if !NT_SUCCESS(Status) {
            NpDeleteCcb(Ccb, List);
            NpDeleteFcb(Fcb, List);
            break 'quickie;
        }

        (*Fcb).SecurityDescriptor = CachedSecurityDescriptor;

        NpSetFileObject(FileObject, Ccb as PVOID, (*Ccb).NonPagedCcb as PVOID, TRUE as ULONG);
        (*Ccb).FileObject[FILE_PIPE_SERVER_END as usize] = FileObject;

        NpCheckForNotify(Dcb, TRUE, List);

        (*IoStatus).__bindgen_anon_1.Status = STATUS_SUCCESS;
        (*IoStatus).Information = FILE_CREATED as _;

        return STATUS_SUCCESS;
    } // 'quickie

    (*IoStatus).Information = 0;
    (*IoStatus).__bindgen_anon_1.Status = Status;
    Status
}

pub unsafe extern "system" fn NpFsdCreateNamedPipe(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
) -> NTSTATUS {
    let IoStack: PIO_STACK_LOCATION;
    let FileObject: PFILE_OBJECT;
    let RelatedFileObject: PFILE_OBJECT;
    let Disposition: USHORT;
    let ShareAccess: USHORT;
    let Process: PEPROCESS;
    let mut DeferredList: LIST_ENTRY = core::mem::zeroed();
    let mut FileName: UNICODE_STRING;
    let mut Fcb: PNP_FCB = null_mut();
    let mut Prefix: UNICODE_STRING = core::mem::zeroed();
    let Parameters: PNAMED_PIPE_CREATE_PARAMETERS;
    let mut IoStatus: IO_STATUS_BLOCK = core::mem::zeroed();

    InitializeListHead(addr_of_mut!(DeferredList));
    Process = IoGetRequestorProcess(Irp);

    IoStack = IoGetCurrentIrpStackLocation(Irp);
    FileObject = (*IoStack).FileObject;
    RelatedFileObject = (*FileObject).RelatedFileObject;

    Disposition = (((*IoStack).Parameters.CreatePipe.Options >> 24) & 0xFF) as USHORT;
    ShareAccess = ((*IoStack).Parameters.CreatePipe.ShareAccess & 0xFFFF) as USHORT;
    Parameters = (*IoStack).Parameters.CreatePipe.Parameters;

    FileName = (*FileObject).FileName;

    IoStatus.Information = 0;

    FsRtlEnterFileSystem();
    NpAcquireExclusiveVcb();

    'quickie: {
        if !RelatedFileObject.is_null() {
            Fcb = (((*RelatedFileObject).FsContext as ULONG_PTR) & !1) as PNP_FCB;
            if Fcb.is_null()
                || ((*Fcb).NodeType != NPFS_NTC_ROOT_DCB)
                || ((FileName.Length as usize) < size_of::<u16>())
                || (*FileName.Buffer.offset(0) == OBJ_NAME_PATH_SEPARATOR)
            {
                IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                break 'quickie;
            }

            IoStatus.__bindgen_anon_1.Status = NpFindRelativePrefix(
                (*RelatedFileObject).FsContext as PNP_DCB,
                addr_of_mut!(FileName),
                TRUE as ULONG,
                addr_of_mut!(Prefix),
                addr_of_mut!(Fcb),
            );
            if !NT_SUCCESS(IoStatus.__bindgen_anon_1.Status) {
                break 'quickie;
            }
        } else {
            if (FileName.Length as usize <= size_of::<u16>())
                || (*FileName.Buffer.offset(0) != OBJ_NAME_PATH_SEPARATOR)
            {
                IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                break 'quickie;
            }

            Fcb = NpFindPrefix(addr_of_mut!(FileName), 1, addr_of_mut!(Prefix));
        }

        if Prefix.Length != 0 {
            if (*Fcb).NodeType == NPFS_NTC_ROOT_DCB {
                IoStatus.__bindgen_anon_1.Status = NpCreateNewNamedPipe(
                    Fcb as PNP_DCB,
                    FileObject,
                    FileName,
                    (*(*IoStack).Parameters.CreatePipe.SecurityContext).DesiredAccess,
                    (*(*IoStack).Parameters.CreatePipe.SecurityContext).AccessState,
                    Disposition,
                    ShareAccess,
                    Parameters,
                    Process,
                    addr_of_mut!(DeferredList),
                    addr_of_mut!(IoStatus),
                );
                break 'quickie;
            } else {
                IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
                break 'quickie;
            }
        }

        if (*Fcb).NodeType != NPFS_NTC_FCB {
            IoStatus.__bindgen_anon_1.Status = STATUS_OBJECT_NAME_INVALID;
            break 'quickie;
        }

        let prev_mode: KPROCESSOR_MODE = if ((*IoStack).Flags & SL_FORCE_ACCESS_CHECK) != 0 {
            UserMode
        } else {
            (*Irp).RequestorMode
        };

        IoStatus = NpCreateExistingNamedPipe(
            Fcb,
            FileObject,
            (*(*IoStack).Parameters.CreatePipe.SecurityContext).DesiredAccess,
            (*(*IoStack).Parameters.CreatePipe.SecurityContext).AccessState,
            prev_mode,
            Disposition as ULONG,
            ShareAccess as ULONG,
            Parameters,
            Process,
            addr_of_mut!(DeferredList),
        );
    } // 'quickie

    NpReleaseVcb();
    NpCompleteDeferredIrps(addr_of_mut!(DeferredList));
    FsRtlExitFileSystem();

    (*Irp).IoStatus = IoStatus;
    IoCompleteRequest(Irp, IO_NAMED_PIPE_INCREMENT);
    IoStatus.__bindgen_anon_1.Status
}
