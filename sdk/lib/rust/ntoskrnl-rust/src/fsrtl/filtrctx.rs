use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    ExAcquireFastMutex, ExInitializeFastMutex, ExReleaseFastMutex, FSRTL_PER_FILEOBJECT_CONTEXT,
    FSRTL_PER_STREAM_CONTEXT, IoChangeFileObjectFilterContext, IoGetFileObjectFilterContext,
    LOGICAL, MmIsFileObjectAPagingFile, PFILE_OBJECT, PFSRTL_ADVANCED_FCB_HEADER,
    PFSRTL_PER_FILEOBJECT_CONTEXT, PFSRTL_PER_STREAM_CONTEXT, _FAST_MUTEX, _LIST_ENTRY,
};
use ntkrnl_sys::ex::{ExAllocatePoolWithTag, ExFreePoolWithTag};
use ntkrnl_sys::mm::NonPagedPool;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

type FAST_MUTEX = _FAST_MUTEX;
type LIST_ENTRY = _LIST_ENTRY;

#[repr(C)]
struct FILE_OBJECT_FILTER_CONTEXTS {
    FilterContextsMutex: FAST_MUTEX,
    FilterContexts: LIST_ENTRY,
}

type PFILE_OBJECT_FILTER_CONTEXTS = *mut FILE_OBJECT_FILTER_CONTEXTS;

#[no_mangle]
pub unsafe extern "system" fn FsRtlPTeardownPerFileObjectContexts(FileObject: PFILE_OBJECT) {
    let FileObject = opaque_ptr(FileObject);
    let mut fo_context: PFILE_OBJECT_FILTER_CONTEXTS = core::ptr::null_mut();

    if FileObject.is_null() {
        return;
    }

    fo_context = IoGetFileObjectFilterContext(FileObject) as PFILE_OBJECT_FILTER_CONTEXTS;
    if fo_context.is_null() {
        return;
    }

    let status = IoChangeFileObjectFilterContext(FileObject, fo_context as PVOID, 0u8);
    if status != STATUS_SUCCESS {
        return;
    }

    if !IsListEmpty(core::ptr::addr_of_mut!((*fo_context).FilterContexts) as PLIST_ENTRY) {
        return;
    }

    ExFreePoolWithTag(fo_context as PVOID, u32::from_be_bytes(*b"FOCX"));
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlIsPagingFile(FileObject: PFILE_OBJECT) -> LOGICAL {
    let FileObject = opaque_ptr(FileObject);
    MmIsFileObjectAPagingFile(FileObject) as LOGICAL
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupPerFileObjectContext(
    FileObject: PFILE_OBJECT,
    OwnerId: PVOID,
    InstanceId: PVOID,
) -> PFSRTL_PER_FILEOBJECT_CONTEXT {
    let FileObject = opaque_ptr(FileObject);
    let OwnerId = opaque_ptr(OwnerId);
    let InstanceId = opaque_ptr(InstanceId);
    let mut next_entry: PLIST_ENTRY;
    let mut fo_context: PFILE_OBJECT_FILTER_CONTEXTS = core::ptr::null_mut();
    let mut tmp_per_fo_context: PFSRTL_PER_FILEOBJECT_CONTEXT;
    let mut per_fo_context: PFSRTL_PER_FILEOBJECT_CONTEXT = core::ptr::null_mut();

    if FileObject.is_null() {
        return core::ptr::null_mut();
    }

    fo_context = IoGetFileObjectFilterContext(FileObject) as PFILE_OBJECT_FILTER_CONTEXTS;
    if fo_context.is_null() {
        return core::ptr::null_mut();
    }

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*fo_context).FilterContextsMutex));

    if !IsListEmpty(core::ptr::addr_of_mut!((*fo_context).FilterContexts) as PLIST_ENTRY) {
        next_entry = (*fo_context).FilterContexts.Flink;
        while next_entry != core::ptr::addr_of_mut!((*fo_context).FilterContexts) as PLIST_ENTRY {
            if OwnerId.is_null() && InstanceId.is_null() {
                per_fo_context = next_entry as PFSRTL_PER_FILEOBJECT_CONTEXT;
                break;
            } else {
                tmp_per_fo_context = (next_entry as usize
                    - core::mem::offset_of!(FSRTL_PER_FILEOBJECT_CONTEXT, Links))
                    as PFSRTL_PER_FILEOBJECT_CONTEXT;
                if (!InstanceId.is_null()
                    && (*tmp_per_fo_context).InstanceId == InstanceId
                    && (*tmp_per_fo_context).OwnerId == OwnerId)
                    || (!OwnerId.is_null() && (*tmp_per_fo_context).OwnerId == OwnerId)
                {
                    per_fo_context = tmp_per_fo_context;
                    break;
                }
            }
            next_entry = (*next_entry).Flink;
        }
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*fo_context).FilterContextsMutex));

    per_fo_context
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlLookupPerStreamContextInternal(
    AdvFcbHeader: PFSRTL_ADVANCED_FCB_HEADER,
    OwnerId: PVOID,
    InstanceId: PVOID,
) -> PFSRTL_PER_STREAM_CONTEXT {
    let OwnerId = opaque_ptr(OwnerId);
    let InstanceId = opaque_ptr(InstanceId);
    let mut next_entry: PLIST_ENTRY;
    let mut tmp_per_stream_context: PFSRTL_PER_STREAM_CONTEXT;
    let mut per_stream_context: PFSRTL_PER_STREAM_CONTEXT = core::ptr::null_mut();

    if AdvFcbHeader.is_null() {
        return core::ptr::null_mut();
    }

    ExAcquireFastMutex((*AdvFcbHeader).FastMutex);

    if !IsListEmpty(core::ptr::addr_of_mut!((*AdvFcbHeader).FilterContexts) as PLIST_ENTRY) {
        next_entry = (*AdvFcbHeader).FilterContexts.Flink;
        while next_entry != core::ptr::addr_of_mut!((*AdvFcbHeader).FilterContexts) as PLIST_ENTRY {
            if OwnerId.is_null() && InstanceId.is_null() {
                per_stream_context = next_entry as PFSRTL_PER_STREAM_CONTEXT;
                break;
            } else {
                tmp_per_stream_context = (next_entry as usize
                    - core::mem::offset_of!(FSRTL_PER_STREAM_CONTEXT, Links))
                    as PFSRTL_PER_STREAM_CONTEXT;
                if (!InstanceId.is_null()
                    && (*tmp_per_stream_context).InstanceId == InstanceId
                    && (*tmp_per_stream_context).OwnerId == OwnerId)
                    || (!OwnerId.is_null() && (*tmp_per_stream_context).OwnerId == OwnerId)
                {
                    per_stream_context = tmp_per_stream_context;
                    break;
                }
            }
            next_entry = (*next_entry).Flink;
        }
    }

    ExReleaseFastMutex((*AdvFcbHeader).FastMutex);

    per_stream_context
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInsertPerFileObjectContext(
    FileObject: PFILE_OBJECT,
    Ptr: PFSRTL_PER_FILEOBJECT_CONTEXT,
) -> NTSTATUS {
    let FileObject = opaque_ptr(FileObject);
    let mut fo_context: PFILE_OBJECT_FILTER_CONTEXTS = core::ptr::null_mut();
    let status: NTSTATUS;

    if FileObject.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if ((*FileObject).Flags & 0x00000020) == 0 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    fo_context = IoGetFileObjectFilterContext(FileObject) as PFILE_OBJECT_FILTER_CONTEXTS;
    if fo_context.is_null() {
        fo_context = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<FILE_OBJECT_FILTER_CONTEXTS>() as SIZE_T,
            u32::from_be_bytes(*b"FOCX"),
        ) as PFILE_OBJECT_FILTER_CONTEXTS;
        if fo_context.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        ExInitializeFastMutex(core::ptr::addr_of_mut!((*fo_context).FilterContextsMutex));
        InitializeListHead(core::ptr::addr_of_mut!((*fo_context).FilterContexts) as PLIST_ENTRY);

        status = IoChangeFileObjectFilterContext(FileObject, fo_context as PVOID, 1u8);
        if status < 0 {
            ExFreePoolWithTag(fo_context as PVOID, u32::from_be_bytes(*b"FOCX"));

            fo_context = IoGetFileObjectFilterContext(FileObject) as PFILE_OBJECT_FILTER_CONTEXTS;
            if fo_context.is_null() {
                return STATUS_UNSUCCESSFUL;
            }
        }
    }

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*fo_context).FilterContextsMutex));
    InsertHeadList(
        core::ptr::addr_of_mut!((*fo_context).FilterContexts) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*Ptr).Links) as PLIST_ENTRY,
    );
    ExReleaseFastMutex(core::ptr::addr_of_mut!((*fo_context).FilterContextsMutex));

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlInsertPerStreamContext(
    AdvFcbHeader: PFSRTL_ADVANCED_FCB_HEADER,
    PerStreamContext: PFSRTL_PER_STREAM_CONTEXT,
) -> NTSTATUS {
    if AdvFcbHeader.is_null() || ((*AdvFcbHeader).Flags2 & 0x00000040) == 0 {
        return STATUS_INVALID_DEVICE_REQUEST;
    }

    ExAcquireFastMutex((*AdvFcbHeader).FastMutex);
    InsertHeadList(
        core::ptr::addr_of_mut!((*AdvFcbHeader).FilterContexts) as PLIST_ENTRY,
        core::ptr::addr_of_mut!((*PerStreamContext).Links) as PLIST_ENTRY,
    );
    ExReleaseFastMutex((*AdvFcbHeader).FastMutex);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRemovePerFileObjectContext(
    FileObject: PFILE_OBJECT,
    OwnerId: PVOID,
    InstanceId: PVOID,
) -> PFSRTL_PER_FILEOBJECT_CONTEXT {
    let FileObject = opaque_ptr(FileObject);
    let OwnerId = opaque_ptr(OwnerId);
    let InstanceId = opaque_ptr(InstanceId);
    let mut next_entry: PLIST_ENTRY;
    let mut fo_context: PFILE_OBJECT_FILTER_CONTEXTS = core::ptr::null_mut();
    let mut tmp_per_fo_context: PFSRTL_PER_FILEOBJECT_CONTEXT;
    let mut per_fo_context: PFSRTL_PER_FILEOBJECT_CONTEXT = core::ptr::null_mut();

    if FileObject.is_null() {
        return core::ptr::null_mut();
    }

    fo_context = IoGetFileObjectFilterContext(FileObject) as PFILE_OBJECT_FILTER_CONTEXTS;
    if fo_context.is_null() {
        return core::ptr::null_mut();
    }

    ExAcquireFastMutex(core::ptr::addr_of_mut!((*fo_context).FilterContextsMutex));

    if !IsListEmpty(core::ptr::addr_of_mut!((*fo_context).FilterContexts) as PLIST_ENTRY) {
        next_entry = (*fo_context).FilterContexts.Flink;
        while next_entry != core::ptr::addr_of_mut!((*fo_context).FilterContexts) as PLIST_ENTRY {
            if OwnerId.is_null() && InstanceId.is_null() {
                per_fo_context = next_entry as PFSRTL_PER_FILEOBJECT_CONTEXT;
                break;
            } else {
                tmp_per_fo_context = (next_entry as usize
                    - core::mem::offset_of!(FSRTL_PER_FILEOBJECT_CONTEXT, Links))
                    as PFSRTL_PER_FILEOBJECT_CONTEXT;
                if (!InstanceId.is_null()
                    && (*tmp_per_fo_context).InstanceId == InstanceId
                    && (*tmp_per_fo_context).OwnerId == OwnerId)
                    || (!OwnerId.is_null() && (*tmp_per_fo_context).OwnerId == OwnerId)
                {
                    per_fo_context = tmp_per_fo_context;
                    break;
                }
            }
            next_entry = (*next_entry).Flink;
        }

        if !per_fo_context.is_null() {
            RemoveEntryList(core::ptr::addr_of_mut!((*per_fo_context).Links) as PLIST_ENTRY);
        }
    }

    ExReleaseFastMutex(core::ptr::addr_of_mut!((*fo_context).FilterContextsMutex));

    per_fo_context
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlRemovePerStreamContext(
    AdvFcbHeader: PFSRTL_ADVANCED_FCB_HEADER,
    OwnerId: PVOID,
    InstanceId: PVOID,
) -> PFSRTL_PER_STREAM_CONTEXT {
    let OwnerId = opaque_ptr(OwnerId);
    let InstanceId = opaque_ptr(InstanceId);
    let mut next_entry: PLIST_ENTRY;
    let mut tmp_per_stream_context: PFSRTL_PER_STREAM_CONTEXT;
    let mut per_stream_context: PFSRTL_PER_STREAM_CONTEXT = core::ptr::null_mut();

    if AdvFcbHeader.is_null() || ((*AdvFcbHeader).Flags2 & 0x00000040) == 0 {
        return core::ptr::null_mut();
    }

    ExAcquireFastMutex((*AdvFcbHeader).FastMutex);
    if !IsListEmpty(core::ptr::addr_of_mut!((*AdvFcbHeader).FilterContexts) as PLIST_ENTRY) {
        next_entry = (*AdvFcbHeader).FilterContexts.Flink;
        while next_entry != core::ptr::addr_of_mut!((*AdvFcbHeader).FilterContexts) as PLIST_ENTRY {
            if OwnerId.is_null() && InstanceId.is_null() {
                per_stream_context = next_entry as PFSRTL_PER_STREAM_CONTEXT;
                break;
            } else {
                tmp_per_stream_context = (next_entry as usize
                    - core::mem::offset_of!(FSRTL_PER_STREAM_CONTEXT, Links))
                    as PFSRTL_PER_STREAM_CONTEXT;
                if (!InstanceId.is_null()
                    && (*tmp_per_stream_context).InstanceId == InstanceId
                    && (*tmp_per_stream_context).OwnerId == OwnerId)
                    || (!OwnerId.is_null() && (*tmp_per_stream_context).OwnerId == OwnerId)
                {
                    per_stream_context = tmp_per_stream_context;
                    break;
                }
            }
            next_entry = (*next_entry).Flink;
        }

        if !per_stream_context.is_null() {
            RemoveEntryList(core::ptr::addr_of_mut!((*per_stream_context).Links) as PLIST_ENTRY);
        }
    }
    ExReleaseFastMutex((*AdvFcbHeader).FastMutex);

    per_stream_context
}

#[no_mangle]
pub unsafe extern "system" fn FsRtlTeardownPerStreamContexts(
    AdvFcbHeader: PFSRTL_ADVANCED_FCB_HEADER,
) {
    let mut next_entry: PLIST_ENTRY;
    let mut is_mutex_locked: BOOLEAN = 0u8;
    let per_stream_context: PFSRTL_PER_STREAM_CONTEXT;

    ExAcquireFastMutex((*AdvFcbHeader).FastMutex);
    is_mutex_locked = 1u8;

    while !IsListEmpty(core::ptr::addr_of_mut!((*AdvFcbHeader).FilterContexts) as PLIST_ENTRY) {
        next_entry =
            RemoveHeadList(core::ptr::addr_of_mut!((*AdvFcbHeader).FilterContexts) as PLIST_ENTRY);
        let per_stream_context = (next_entry as usize
            - core::mem::offset_of!(FSRTL_PER_STREAM_CONTEXT, Links))
            as PFSRTL_PER_STREAM_CONTEXT;

        ExReleaseFastMutex((*AdvFcbHeader).FastMutex);
        is_mutex_locked = 0u8;

        if let Some(callback) = (*per_stream_context).FreeCallback {
            callback(per_stream_context as PVOID);
        }

        ExAcquireFastMutex((*AdvFcbHeader).FastMutex);
        is_mutex_locked = 1u8;
    }

    if is_mutex_locked != 0 {
        ExReleaseFastMutex((*AdvFcbHeader).FastMutex);
    }
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/fsrtl/filtrctx.c
//  * PURPOSE:         File Stream Filter Context support for File System Drivers
//  * PROGRAMMERS:     Pierre Schweitzer (pierre.schweitzer@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// typedef struct _FILE_OBJECT_FILTER_CONTEXTS
// {
//     FAST_MUTEX FilterContextsMutex;
//     LIST_ENTRY FilterContexts;
// } FILE_OBJECT_FILTER_CONTEXTS, *PFILE_OBJECT_FILTER_CONTEXTS;
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlPTeardownPerFileObjectContexts(IN PFILE_OBJECT FileObject)
// {
//     PFILE_OBJECT_FILTER_CONTEXTS FOContext = NULL;
//
//     ASSERT(FileObject);
//
//     if (!(FOContext = IoGetFileObjectFilterContext(FileObject)))
//     {
//         return;
//     }
//
//     ASSERT(IoChangeFileObjectFilterContext(FileObject, FOContext, FALSE) == STATUS_SUCCESS);
//     ASSERT(IsListEmpty(&(FOContext->FilterContexts)));
//
//     ExFreePoolWithTag(FOContext, 'FOCX');
// }
//
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*++
//  * @name FsRtlIsPagingFile
//  * @implemented NT 5.2
//  *
//  *     The FsRtlIsPagingFile routine checks if the FileObject is a Paging File.
//  *
//  * @param FileObject
//  *        A pointer to the File Object to be tested.
//  *
//  * @return TRUE if the File is a Paging File, FALSE otherwise.
//  *
//  * @remarks None.
//  *
//  *--*/
// LOGICAL
// NTAPI
// FsRtlIsPagingFile(IN PFILE_OBJECT FileObject)
// {
//     return MmIsFileObjectAPagingFile(FileObject);
// }
//
// /*
//  * @implemented
//  */
// PFSRTL_PER_FILEOBJECT_CONTEXT
// NTAPI
// FsRtlLookupPerFileObjectContext(IN PFILE_OBJECT FileObject,
//                                 IN PVOID OwnerId OPTIONAL,
//                                 IN PVOID InstanceId OPTIONAL)
// {
//     PLIST_ENTRY NextEntry;
//     PFILE_OBJECT_FILTER_CONTEXTS FOContext = NULL;
//     PFSRTL_PER_FILEOBJECT_CONTEXT TmpPerFOContext, PerFOContext = NULL;
//
//     if (!FileObject || !(FOContext = IoGetFileObjectFilterContext(FileObject)))
//     {
//         return NULL;
//     }
//
//     ExAcquireFastMutex(&(FOContext->FilterContextsMutex));
//
//     /* If list is empty, no need to browse it */
//     if (!IsListEmpty(&(FOContext->FilterContexts)))
//     {
//         for (NextEntry = FOContext->FilterContexts.Flink;
//              NextEntry != &(FOContext->FilterContexts);
//              NextEntry = NextEntry->Flink)
//         {
//             /* If we don't have any criteria for search, first entry will be enough */
//             if (!OwnerId && !InstanceId)
//             {
//                 PerFOContext = (PFSRTL_PER_FILEOBJECT_CONTEXT)NextEntry;
//                 break;
//             }
//             /* Else, we've to find something that matches with the parameters. */
//             else
//             {
//                 TmpPerFOContext = CONTAINING_RECORD(NextEntry, FSRTL_PER_FILEOBJECT_CONTEXT, Links);
//                 if ((InstanceId && TmpPerFOContext->InstanceId == InstanceId && TmpPerFOContext->OwnerId == OwnerId) ||
//                     (OwnerId && TmpPerFOContext->OwnerId == OwnerId))
//                 {
//                     PerFOContext = TmpPerFOContext;
//                     break;
//                 }
//             }
//         }
//     }
//
//     ExReleaseFastMutex(&(FOContext->FilterContextsMutex));
//
//     return PerFOContext;
// }
//
// /*
//  * @implemented
//  */
// PFSRTL_PER_STREAM_CONTEXT
// NTAPI
// FsRtlLookupPerStreamContextInternal(IN PFSRTL_ADVANCED_FCB_HEADER AdvFcbHeader,
//                                     IN PVOID OwnerId OPTIONAL,
//                                     IN PVOID InstanceId OPTIONAL)
// {
//     PLIST_ENTRY NextEntry;
//     PFSRTL_PER_STREAM_CONTEXT TmpPerStreamContext, PerStreamContext = NULL;
//
//     ASSERT(AdvFcbHeader);
//     ASSERT(FlagOn(AdvFcbHeader->Flags2, FSRTL_FLAG2_SUPPORTS_FILTER_CONTEXTS));
//
//     ExAcquireFastMutex(AdvFcbHeader->FastMutex);
//
//     /* If list is empty, no need to browse it */
//     if (!IsListEmpty(&(AdvFcbHeader->FilterContexts)))
//     {
//         for (NextEntry = AdvFcbHeader->FilterContexts.Flink;
//              NextEntry != &(AdvFcbHeader->FilterContexts);
//              NextEntry = NextEntry->Flink)
//         {
//             /* If we don't have any criteria for search, first entry will be enough */
//             if (!OwnerId && !InstanceId)
//             {
//                 PerStreamContext = (PFSRTL_PER_STREAM_CONTEXT)NextEntry;
//                 break;
//             }
//             /* Else, we've to find something that matches with the parameters. */
//             else
//             {
//                 TmpPerStreamContext = CONTAINING_RECORD(NextEntry, FSRTL_PER_STREAM_CONTEXT, Links);
//                 if ((InstanceId && TmpPerStreamContext->InstanceId == InstanceId && TmpPerStreamContext->OwnerId == OwnerId) ||
//                     (OwnerId && TmpPerStreamContext->OwnerId == OwnerId))
//                 {
//                     PerStreamContext = TmpPerStreamContext;
//                     break;
//                 }
//             }
//         }
//     }
//
//     ExReleaseFastMutex(AdvFcbHeader->FastMutex);
//
//     return PerStreamContext;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FsRtlInsertPerFileObjectContext(IN PFILE_OBJECT FileObject,
//                                 IN PFSRTL_PER_FILEOBJECT_CONTEXT Ptr)
// {
//     PFILE_OBJECT_FILTER_CONTEXTS FOContext = NULL;
//     NTSTATUS Status;
//
//     if (!FileObject)
//     {
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     if (!(FileObject->Flags & FO_FILE_OBJECT_HAS_EXTENSION))
//     {
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     /* Get filter contexts */
//     FOContext = IoGetFileObjectFilterContext(FileObject);
//     if (!FOContext)
//     {
//         /* If there's none, allocate new structure */
//         FOContext = ExAllocatePoolWithTag(NonPagedPool, sizeof(FILE_OBJECT_FILTER_CONTEXTS), 'FOCX');
//         if (!FOContext)
//         {
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         /* Initialize it */
//         ExInitializeFastMutex(&(FOContext->FilterContextsMutex));
//         InitializeListHead(&(FOContext->FilterContexts));
//
//         /* Set it */
//         Status = IoChangeFileObjectFilterContext(FileObject, FOContext, TRUE);
//         if (!NT_SUCCESS(Status))
//         {
//             /* If it fails, it means that someone else has set it in the meanwhile */
//             ExFreePoolWithTag(FOContext, 'FOCX');
//
//             /* So, we can get it */
//             FOContext = IoGetFileObjectFilterContext(FileObject);
//             if (!FOContext)
//             {
//                 /* If we fall down here, something went very bad. This shouldn't happen */
//                 ASSERT(FALSE);
//                 return STATUS_UNSUCCESSFUL;
//             }
//         }
//     }
//
//     /* Finally, insert */
//     ExAcquireFastMutex(&(FOContext->FilterContextsMutex));
//     InsertHeadList(&(FOContext->FilterContexts), &(Ptr->Links));
//     ExReleaseFastMutex(&(FOContext->FilterContextsMutex));
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// FsRtlInsertPerStreamContext(IN PFSRTL_ADVANCED_FCB_HEADER AdvFcbHeader,
//                             IN PFSRTL_PER_STREAM_CONTEXT PerStreamContext)
// {
//     if (!(AdvFcbHeader) || !(AdvFcbHeader->Flags2 & FSRTL_FLAG2_SUPPORTS_FILTER_CONTEXTS))
//     {
//         return STATUS_INVALID_DEVICE_REQUEST;
//     }
//
//     ExAcquireFastMutex(AdvFcbHeader->FastMutex);
//     InsertHeadList(&(AdvFcbHeader->FilterContexts), &(PerStreamContext->Links));
//     ExReleaseFastMutex(AdvFcbHeader->FastMutex);
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// PFSRTL_PER_FILEOBJECT_CONTEXT
// NTAPI
// FsRtlRemovePerFileObjectContext(IN PFILE_OBJECT FileObject,
//                                 IN PVOID OwnerId OPTIONAL,
//                                 IN PVOID InstanceId OPTIONAL)
// {
//     PLIST_ENTRY NextEntry;
//     PFILE_OBJECT_FILTER_CONTEXTS FOContext = NULL;
//     PFSRTL_PER_FILEOBJECT_CONTEXT TmpPerFOContext, PerFOContext = NULL;
//
//     if (!FileObject || !(FOContext = IoGetFileObjectFilterContext(FileObject)))
//     {
//         return NULL;
//     }
//
//     ExAcquireFastMutex(&(FOContext->FilterContextsMutex));
//
//     /* If list is empty, no need to browse it */
//     if (!IsListEmpty(&(FOContext->FilterContexts)))
//     {
//         for (NextEntry = FOContext->FilterContexts.Flink;
//              NextEntry != &(FOContext->FilterContexts);
//              NextEntry = NextEntry->Flink)
//         {
//             /* If we don't have any criteria for search, first entry will be enough */
//             if (!OwnerId && !InstanceId)
//             {
//                 PerFOContext = (PFSRTL_PER_FILEOBJECT_CONTEXT)NextEntry;
//                 break;
//             }
//             /* Else, we've to find something that matches with the parameters. */
//             else
//             {
//                 TmpPerFOContext = CONTAINING_RECORD(NextEntry, FSRTL_PER_FILEOBJECT_CONTEXT, Links);
//                 if ((InstanceId && TmpPerFOContext->InstanceId == InstanceId && TmpPerFOContext->OwnerId == OwnerId) ||
//                     (OwnerId && TmpPerFOContext->OwnerId == OwnerId))
//                 {
//                     PerFOContext = TmpPerFOContext;
//                     break;
//                 }
//             }
//         }
//
//         /* Finally remove entry from list */
//         if (PerFOContext)
//         {
//             RemoveEntryList(&(PerFOContext->Links));
//         }
//     }
//
//     ExReleaseFastMutex(&(FOContext->FilterContextsMutex));
//
//     return PerFOContext;
// }
//
// /*
//  * @implemented
//  */
// PFSRTL_PER_STREAM_CONTEXT
// NTAPI
// FsRtlRemovePerStreamContext(IN PFSRTL_ADVANCED_FCB_HEADER AdvFcbHeader,
//                             IN PVOID OwnerId OPTIONAL,
//                             IN PVOID InstanceId OPTIONAL)
// {
//     PLIST_ENTRY NextEntry;
//     PFSRTL_PER_STREAM_CONTEXT TmpPerStreamContext, PerStreamContext = NULL;
//
//     if (!(AdvFcbHeader) || !(AdvFcbHeader->Flags2 & FSRTL_FLAG2_SUPPORTS_FILTER_CONTEXTS))
//     {
//         return NULL;
//     }
//
//     ExAcquireFastMutex(AdvFcbHeader->FastMutex);
//     /* If list is empty, no need to browse it */
//     if (!IsListEmpty(&(AdvFcbHeader->FilterContexts)))
//     {
//         for (NextEntry = AdvFcbHeader->FilterContexts.Flink;
//              NextEntry != &(AdvFcbHeader->FilterContexts);
//              NextEntry = NextEntry->Flink)
//         {
//             /* If we don't have any criteria for search, first entry will be enough */
//             if (!OwnerId && !InstanceId)
//             {
//                 PerStreamContext = (PFSRTL_PER_STREAM_CONTEXT)NextEntry;
//                 break;
//             }
//             /* Else, we've to find something that matches with the parameters. */
//             else
//             {
//                 TmpPerStreamContext = CONTAINING_RECORD(NextEntry, FSRTL_PER_STREAM_CONTEXT, Links);
//                 if ((InstanceId && TmpPerStreamContext->InstanceId == InstanceId && TmpPerStreamContext->OwnerId == OwnerId) ||
//                     (OwnerId && TmpPerStreamContext->OwnerId == OwnerId))
//                 {
//                     PerStreamContext = TmpPerStreamContext;
//                     break;
//                 }
//             }
//         }
//
//         /* Finally remove entry from list */
//         if (PerStreamContext)
//         {
//             RemoveEntryList(&(PerStreamContext->Links));
//         }
//     }
//     ExReleaseFastMutex(AdvFcbHeader->FastMutex);
//
//     return PerStreamContext;
//
// }
//
// /*
//  * @implemented
//  */
// VOID
// NTAPI
// FsRtlTeardownPerStreamContexts(IN PFSRTL_ADVANCED_FCB_HEADER AdvFcbHeader)
// {
//     PLIST_ENTRY NextEntry;
//     volatile BOOLEAN IsMutexLocked = FALSE;
//     PFSRTL_PER_STREAM_CONTEXT PerStreamContext;
//
//     _SEH2_TRY
//     {
//         /* Acquire mutex to deal with the list */
//         ExAcquireFastMutex(AdvFcbHeader->FastMutex);
//         IsMutexLocked = TRUE;
//
//         /* While there are items... */
//         while (!IsListEmpty(&(AdvFcbHeader->FilterContexts)))
//         {
//             /* ...remove one */
//             NextEntry = RemoveHeadList(&(AdvFcbHeader->FilterContexts));
//             PerStreamContext = CONTAINING_RECORD(NextEntry, FSRTL_PER_STREAM_CONTEXT, Links);
//
//             /* Release mutex before calling callback */
//             ExReleaseFastMutex(AdvFcbHeader->FastMutex);
//             IsMutexLocked = FALSE;
//
//             /* Call the callback */
//             ASSERT(PerStreamContext->FreeCallback);
//             (*PerStreamContext->FreeCallback)(PerStreamContext);
//
//             /* Relock the list to continue */
//             ExAcquireFastMutex(AdvFcbHeader->FastMutex);
//             IsMutexLocked = TRUE;
//         }
//     }
//     _SEH2_FINALLY
//     {
//         /* If mutex was locked, release */
//         if (IsMutexLocked)
//         {
//             ExReleaseFastMutex(AdvFcbHeader->FastMutex);
//         }
//     }
//     _SEH2_END;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================

// BEGIN OFFSET CHECKS — auto-generated by verify_offsets.py, do not edit
#[cfg(target_pointer_width = "64")]
const _LAYOUT_CHECKS_64: () = {
    assert!(core::mem::size_of::<FILE_OBJECT_FILTER_CONTEXTS>() == 72);
    assert!(core::mem::offset_of!(FILE_OBJECT_FILTER_CONTEXTS, FilterContextsMutex) == 0);
    assert!(core::mem::offset_of!(FILE_OBJECT_FILTER_CONTEXTS, FilterContexts) == 56);
};
#[cfg(target_pointer_width = "32")]
const _LAYOUT_CHECKS_32: () = {
    assert!(core::mem::size_of::<FILE_OBJECT_FILTER_CONTEXTS>() == 40);
    assert!(core::mem::offset_of!(FILE_OBJECT_FILTER_CONTEXTS, FilterContextsMutex) == 0);
    assert!(core::mem::offset_of!(FILE_OBJECT_FILTER_CONTEXTS, FilterContexts) == 32);
};
// END OFFSET CHECKS
