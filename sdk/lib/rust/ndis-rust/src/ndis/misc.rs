use crate::prelude::*;
use ntkrnl_sys::ddk::{ExAllocatePool, ExFreePool, ExInterlockedAddLargeInteger, ExQueueWorkItem, FILE_STANDARD_INFORMATION, _IO_STATUS_BLOCK, KeGetRecommendedSharedDataAlignment, KeQueryTimeIncrement, LARGE_INTEGER, _OBJECT_ATTRIBUTES, PLARGE_INTEGER, PWORK_QUEUE_ITEM, RtlAppendUnicodeStringToString, RtlCompareString, RtlCompareUnicodeString, ZwClose, ZwCreateFile, ZwQueryInformationFile, ZwReadFile};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
static mut NDIS_FILE_FOLDER: [WCHAR; 30] = unsafe {
    ::core::mem::transmute::<
        [u8; 60],
        [WCHAR; 30],
    >(
        *b"\\\0S\0y\0s\0t\0e\0m\0R\0o\0o\0t\0\\\0S\0y\0s\0t\0e\0m\x003\x002\0\\\0D\0r\0i\0v\0e\0r\0s\0\\\0\0\0",
    )
};

pub static mut CancelId: LONG = 0;

#[repr(C)]
pub struct NDIS_HANDLE_OBJECT {
    pub Mapped: BOOLEAN,
    pub FileLength: ULONG,
    pub MapBuffer: PVOID,
}
pub type PNDIS_HANDLE_OBJECT = *mut NDIS_HANDLE_OBJECT;
#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedAddUlong(
    Addend: PULONG,
    Increment: ULONG,
    SpinLock: PNDIS_SPIN_LOCK,
) {
    ExInterlockedAddUlong(Addend, Increment, core::ptr::addr_of_mut!((*SpinLock).SpinLock));
}
#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedAddLargeInteger(
    Addend: PLARGE_INTEGER,
    Increment: LARGE_INTEGER,
    SpinLock: PNDIS_SPIN_LOCK,
) {
    ExInterlockedAddLargeInteger(Addend, Increment, core::ptr::addr_of_mut!((*SpinLock).SpinLock));
}

#[no_mangle]
pub unsafe extern "system" fn NdisCompareAnsiString(
    String1: PNDIS_ANSI_STRING,
    String2: PNDIS_ANSI_STRING,
    CaseInSensitive: BOOLEAN,
) -> LONG {
    RtlCompareString(String1, String2, CaseInSensitive)
}

#[no_mangle]
pub unsafe extern "system" fn NdisCompareUnicodeString(
    String1: PNDIS_STRING,
    String2: PNDIS_STRING,
    CaseInSensitive: BOOLEAN,
) -> LONG {
    RtlCompareUnicodeString(String1, String2, CaseInSensitive)
}
#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedInsertHeadList(
    ListHead: PLIST_ENTRY,
    ListEntry: PLIST_ENTRY,
    SpinLock: PNDIS_SPIN_LOCK,
) -> PLIST_ENTRY {
    return ExInterlockedInsertHeadList(ListHead, ListEntry, core::ptr::addr_of_mut!((*SpinLock).SpinLock));
}
#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedInsertTailList(ListHead: PLIST_ENTRY, ListEntry: PLIST_ENTRY, SpinLock: PNDIS_SPIN_LOCK) -> PLIST_ENTRY {
    return ExInterlockedInsertTailList(ListHead, ListEntry, core::ptr::addr_of_mut!((*SpinLock).SpinLock));
}
#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedRemoveHeadList(
    ListHead: PLIST_ENTRY,
    SpinLock: PNDIS_SPIN_LOCK,
) -> PLIST_ENTRY {
    return ExInterlockedRemoveHeadList(ListHead, core::ptr::addr_of_mut!((*SpinLock).SpinLock));
}
#[no_mangle]
pub unsafe extern "C" fn NDIS_HANDLE_TO_POBJECT(handle: NDIS_HANDLE) -> PNDIS_HANDLE_OBJECT {
    return handle as PNDIS_HANDLE_OBJECT;
}
#[no_mangle]
pub unsafe extern "C" fn NDIS_POBJECT_TO_HANDLE(obj: PNDIS_HANDLE_OBJECT) -> NDIS_HANDLE {
    return obj as NDIS_HANDLE;
}
#[no_mangle]
pub unsafe extern "system" fn NdisMapFile(
    Status: PNDIS_STATUS,
    MappedBuffer: *mut PVOID,
    FileHandle: NDIS_HANDLE,
) {
    let HandleObject: PNDIS_HANDLE_OBJECT = FileHandle as PNDIS_HANDLE_OBJECT;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"called: FileHandle 0x%x\n\0".as_ptr() as PCSTR,
        FileHandle,
    );

    if (*HandleObject).Mapped != 0 {
        /* If a file already mapped we will return an error code */
        NDIS_DbgPrint!(
            MIN_TRACE,
            b"File already mapped\n\0".as_ptr() as PCSTR,
        );
        *Status = NDIS_STATUS_ALREADY_MAPPED;
        return;
    }

    (*HandleObject).Mapped = TRUE;
    *MappedBuffer = (*HandleObject).MapBuffer;

    /* Set returned status */
    *Status = STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisUnmapFile(mut FileHandle: NDIS_HANDLE) {
    let mut HandleObject: PNDIS_HANDLE_OBJECT = FileHandle as PNDIS_HANDLE_OBJECT;
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);
    (*HandleObject).Mapped = FALSE;
}
#[no_mangle]
pub unsafe extern "system" fn NdisCloseFile(FileHandle: NDIS_HANDLE) {
    let FileHandleObject: PNDIS_HANDLE_OBJECT;

    ASSERT_IRQL!(PASSIVE_LEVEL);

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    FileHandleObject = NDIS_HANDLE_TO_POBJECT(FileHandle);

    if !(*FileHandleObject).MapBuffer.is_null() {
        ExFreePool((*FileHandleObject).MapBuffer);
    }
    ExFreePool(FileHandleObject as PVOID);
}
#[no_mangle]
pub unsafe extern "system" fn NdisOpenFile(
    Status: PNDIS_STATUS,
    FileHandle: PNDIS_HANDLE,
    FileLength: PUINT,
    FileName: PNDIS_STRING,
    HighestAcceptableAddress: NDIS_PHYSICAL_ADDRESS,
) {
    let mut NtFileHandle: HANDLE = core::ptr::null_mut();
    let mut FullFileName: NDIS_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut FileHandleObject: PNDIS_HANDLE_OBJECT = core::ptr::null_mut();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut StandardInfo: FILE_STANDARD_INFORMATION = core::mem::zeroed();
    let mut NtFileLength: UINT = 0;
    let mut NtStatus: NTSTATUS = 0;

    ASSERT_IRQL!(PASSIVE_LEVEL);

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    *Status = NDIS_STATUS_SUCCESS;
    FullFileName.Buffer = core::ptr::null_mut();

    FullFileName.Length = (core::mem::size_of::<[WCHAR; 30]>()
        - (core::mem::size_of_val(&UNICODE_NULL) as usize)) as USHORT;
    FullFileName.MaximumLength = ((*FileName).MaximumLength as i32
        + FullFileName.Length as i32) as USHORT;
    FullFileName.Buffer = ExAllocatePool(
        NonPagedPool,
        FullFileName.MaximumLength as SIZE_T,
    ) as PWCH;

    'cleanup: {
        if FullFileName.Buffer.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as *const i8);
            *Status = NDIS_STATUS_RESOURCES;
            break 'cleanup;
        }

        FileHandleObject = ExAllocatePool(
            NonPagedPool,
            core::mem::size_of::<NDIS_HANDLE_OBJECT>() as SIZE_T,
        ) as PNDIS_HANDLE_OBJECT;
        if FileHandleObject.is_null() {
            NDIS_DbgPrint!(MIN_TRACE, b"Insufficient resources\n\0".as_ptr() as *const i8);
            *Status = NDIS_STATUS_RESOURCES;
            break 'cleanup;
        }
        memset(
            FileHandleObject as *mut ::core::ffi::c_void,
            0,
            core::mem::size_of::<NDIS_HANDLE_OBJECT>() as size_t,
        );

        memmove(
            FullFileName.Buffer as *mut ::core::ffi::c_void,
            (NDIS_FILE_FOLDER).as_ptr() as *const ::core::ffi::c_void,
            FullFileName.Length as size_t,
        );
        NtStatus = RtlAppendUnicodeStringToString(&raw mut FullFileName, FileName as PCUNICODE_STRING);
        if (NT_SUCCESS!(NtStatus) as i32) == 0 {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"RtlAppendUnicodeStringToString failed (%x)\n\0".as_ptr() as *const i8,
                NtStatus,
            );
            *Status = NDIS_STATUS_FAILURE;
            break 'cleanup;
        }

        // InitializeObjectAttributes expanded inline
        ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
        ObjectAttributes.RootDirectory = core::ptr::null_mut::<::core::ffi::c_void>() as HANDLE;
        ObjectAttributes.ObjectName = &raw mut FullFileName as PUNICODE_STRING;
        ObjectAttributes.Attributes =
            (OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE) as ULONG;
        ObjectAttributes.SecurityDescriptor = core::ptr::null_mut::<::core::ffi::c_void>() as PVOID;
        ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut::<::core::ffi::c_void>() as PVOID;

        NtStatus = ZwCreateFile(
            &raw mut NtFileHandle,
            (FILE_READ_DATA | SYNCHRONIZE) as ACCESS_MASK,
            &raw mut ObjectAttributes,
            &raw mut IoStatusBlock,
            core::ptr::null_mut::<LARGE_INTEGER>(), // PLARGE_INTEGER AllocationSize
            0 as ULONG,        // ULONG FileAttributes
            FILE_SHARE_READ as ULONG,        // ULONG ShareAccess
            FILE_OPEN as ULONG,        // ULONG CreateDisposition
            FILE_SYNCHRONOUS_IO_NONALERT as ULONG,        // ULONG CreateOptions
            0 as PVOID,        // PVOID EaBuffer
            0 as ULONG,        // ULONG EaLength
        );

        if (NT_SUCCESS!(NtStatus) as i32) == 0 {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"ZwCreateFile failed (%x) Name %wZ\n\0".as_ptr() as *const i8,
                NtStatus,
                FileName,
            );
            *Status = NDIS_STATUS_FILE_NOT_FOUND;
            break 'cleanup;
        }

        NtStatus = ZwQueryInformationFile(
            NtFileHandle,
            &raw mut IoStatusBlock,
            &raw mut StandardInfo as PVOID,
            core::mem::size_of::<FILE_STANDARD_INFORMATION>() as ULONG,
            FileStandardInformation,
        );
        if (NT_SUCCESS!(NtStatus) as i32) == 0 {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"ZwQueryInformationFile failed (%x) Name %wZ\n\0".as_ptr() as *const i8,
                NtStatus,
                FileName,
            );
            *Status = NDIS_STATUS_ERROR_READING_FILE;
            break 'cleanup;
        }
        if StandardInfo.EndOfFile.u.HighPart != 0
            || StandardInfo.EndOfFile.u.LowPart == 0 as ULONG
        {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"ZwQueryInformationFile failed Name %wZ\n\0".as_ptr() as *const i8,
                FileName,
            );
            *Status = NDIS_STATUS_ERROR_READING_FILE;
            break 'cleanup;
        }
        NtFileLength = StandardInfo.EndOfFile.u.LowPart as UINT;

        (*FileHandleObject).MapBuffer =
            ExAllocatePool(NonPagedPool, NtFileLength as SIZE_T);
        if (*FileHandleObject).MapBuffer.is_null() {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"ExAllocatePool failed Name %wZ\n\0".as_ptr() as *const i8,
                FileName,
            );
            *Status = NDIS_STATUS_ERROR_READING_FILE;
            break 'cleanup;
        }

        NtStatus = ZwReadFile(
            NtFileHandle,
            core::ptr::null_mut(),
            None,
            core::ptr::null_mut(),
            &raw mut IoStatusBlock,
            (*FileHandleObject).MapBuffer,
            NtFileLength as ULONG,
            core::ptr::null_mut::<LARGE_INTEGER>(),
            core::ptr::null_mut::<ULONG>(),
        );
        if (NT_SUCCESS!(NtStatus) as i32) == 0
            || IoStatusBlock.Information != NtFileLength as ULONG_PTR
        {
            NDIS_DbgPrint!(
                MIN_TRACE,
                b"ZwReadFile failed (%x) Name %wZ\n\0".as_ptr() as *const i8,
                NtStatus,
                FileName,
            );
            *Status = NDIS_STATUS_ERROR_READING_FILE;
            break 'cleanup;
        }
    }

    // cleanup:
    if !NtFileHandle.is_null() {
        ZwClose(NtFileHandle);
    }
    if !FullFileName.Buffer.is_null() {
        ExFreePool(FullFileName.Buffer as PVOID);
        FullFileName.Buffer = core::ptr::null_mut();
    }
    if (NT_SUCCESS!(*Status as NTSTATUS) as i32) == 0 {
        if !FileHandleObject.is_null() {
            if !(*FileHandleObject).MapBuffer.is_null() {
                ExFreePool((*FileHandleObject).MapBuffer);
            }
            ExFreePool(FileHandleObject as PVOID);
        }
        *FileHandle = core::ptr::null_mut();
    } else {
        *FileHandle = NDIS_POBJECT_TO_HANDLE(FileHandleObject) as PVOID;
        *FileLength = NtFileLength as ::core::ffi::c_uint;
    }
}
#[no_mangle]
pub unsafe extern "system" fn NdisSystemProcessorCount() -> CCHAR {
    return KeNumberProcessorsValue();
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetCurrentProcessorCounts(
    pIdleCount: PULONG,
    pKernelAndUser: PULONG,
    pIndex: PULONG,
) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    ExGetCurrentProcessorCounts(pIdleCount as PULONG, pKernelAndUser as PULONG, pIndex as PULONG);
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetSystemUpTime(pSystemUpTime: PULONG) {
    let mut Increment: ULONG;
    let mut TickCount: LARGE_INTEGER = core::mem::zeroed();

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    /* Get the increment and current tick count */
    Increment = KeQueryTimeIncrement();
    KeQueryTickCount!(core::ptr::addr_of_mut!(TickCount) as *mut i64);

    /* Convert to milliseconds and return */
    TickCount.QuadPart *= Increment as LONGLONG;
    TickCount.QuadPart /= (10 * 1000) as LONGLONG;
    *pSystemUpTime = TickCount.__bindgen_anon_1.LowPart;
}

#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedDecrement(Addend: PLONG) -> LONG {
    InterlockedDecrement(Addend)
}

#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedIncrement(Addend: PLONG) -> LONG {
    InterlockedIncrement(Addend)
}
#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedPopEntrySList(
    ListHead: PSLIST_HEADER,
    Lock: PKSPIN_LOCK,
) -> PSINGLE_LIST_ENTRY {
    return ExInterlockedPopEntrySList!(ListHead, Lock) as PSINGLE_LIST_ENTRY;
}
#[no_mangle]
pub unsafe extern "system" fn NdisInterlockedPushEntrySList(
    ListHead: PSLIST_HEADER,
    ListEntry: PSINGLE_LIST_ENTRY,
    Lock: PKSPIN_LOCK,
) -> PSINGLE_LIST_ENTRY {
    return ExInterlockedPushEntrySList!(ListHead, ListEntry as PSLIST_ENTRY, Lock) as PSINGLE_LIST_ENTRY;
}
#[no_mangle]
pub unsafe extern "system" fn ndisProcWorkItemHandler(pContext: PVOID) {
    let mut pNdisItem: PNDIS_WORK_ITEM = pContext as PNDIS_WORK_ITEM;

    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    (*pNdisItem).Routine.unwrap_unchecked()(
        pNdisItem as *mut _NDIS_WORK_ITEM,
        (*pNdisItem).Context,
    );
}
#[no_mangle]
pub unsafe extern "system" fn NdisScheduleWorkItem(mut pWorkItem: PNDIS_WORK_ITEM) -> NDIS_STATUS {
    let mut pntWorkItem: PWORK_QUEUE_ITEM =
        &raw mut (*pWorkItem).WrapperReserved as *mut UCHAR as PWORK_QUEUE_ITEM;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Called.\n\0".as_ptr() as *const i8,
    );

    ExInitializeWorkItem!(
        pntWorkItem,
        Some(ndisProcWorkItemHandler),
        pWorkItem as PVOID,
    );
    ExQueueWorkItem(pntWorkItem, DelayedWorkQueue);
    return NDIS_STATUS_SUCCESS;
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetCurrentProcessorCpuUsage(pCpuUsage: PULONG) {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const i8);

    ExGetCurrentProcessorCpuUsage(pCpuUsage);
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetSharedDataAlignment() -> ULONG {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as PCSTR);

    return KeGetRecommendedSharedDataAlignment();
}
#[no_mangle]
pub unsafe extern "system" fn NdisGetVersion() -> UINT {
    NDIS_DbgPrint!(MAX_TRACE, b"Called.\n\0".as_ptr() as *const ::core::ffi::c_char);

    return NDIS_VERSION;
}
#[no_mangle]
pub unsafe extern "system" fn NdisGeneratePartialCancelId() -> UCHAR {
    let mut PartialCancelId: UCHAR;
    PartialCancelId = InterlockedIncrement(core::ptr::addr_of_mut!(CancelId) as *mut i32) as UCHAR;

    NDIS_DbgPrint!(
        MAX_TRACE,
        b"Cancel ID %u\n\0".as_ptr() as PCSTR,
        PartialCancelId as u32,
    );

    return PartialCancelId;
}
