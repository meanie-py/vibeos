use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::interlocked::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::ex::*;

use ntkrnl_sys::ddk::{
    _IRP, _KQUEUE, _IOP_MINI_COMPLETION_PACKET,
    PDEVICE_OBJECT, PIRP, PKQUEUE, PIOP_MINI_COMPLETION_PACKET,
    POBJECT_TYPE, POBJECT_ATTRIBUTES, PHANDLE,
    ACCESS_MASK, KPROCESSOR_MODE, HANDLE,
    PIO_COMPLETION_ROUTINE, PIO_UNLOAD_SAFE_COMPLETION_CONTEXT,
    PIO_COMPLETION_BASIC_INFORMATION, PIO_STATUS_BLOCK,
    _IO_STATUS_BLOCK, _IO_UNLOAD_SAFE_COMPLETION_CONTEXT,
    PLARGE_INTEGER, LARGE_INTEGER,
    _GENERIC_MAPPING, _INFORMATION_CLASS_INFO,
    _IO_COMPLETION_INFORMATION_CLASS, IO_COMPLETION_BASIC_INFORMATION,
    PNPAGED_LOOKASIDE_LIST,
    _PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList,
    ExGetPreviousMode, ExFreePool,
    IoFreeIrp,
    KeInitializeQueue, KeInsertQueue, KeRundownQueue, KeReadStateQueue, KeRemoveQueue,
    ObCreateObject, ObInsertObject, ObOpenObjectByName, ObReferenceObjectByHandle,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

extern "C" {
    fn DefaultQueryInfoBufferCheck(
        info_class: ULONG,
        info_class_info: *const _INFORMATION_CLASS_INFO,
        info_class_count: ULONG,
        flags: ULONG,
        info_buffer: PVOID,
        info_length: ULONG,
        return_length: *mut ULONG,
        previous_mode_ptr: PVOID,
        previous_mode: KPROCESSOR_MODE,
    ) -> NTSTATUS;
}

/// Local CONTAINING_RECORD macro supporting dotted field paths.
macro_rules! CONTAINING_RECORD {
    ($ptr:expr, $type:ty, $($field:tt).+) => {
        ($ptr as *const u8).sub(core::mem::offset_of!($type, $($field).+)) as *mut $type
    };
}

// IO completion access rights (C macros from ndk/iotypes.h)
const IO_COMPLETION_QUERY_STATE: ACCESS_MASK = 0x0001;
const IO_COMPLETION_MODIFY_STATE: ACCESS_MASK = 0x0002;
const IO_COMPLETION_ALL_ACCESS: ACCESS_MASK = STANDARD_RIGHTS_REQUIRED | SYNCHRONIZE | 0x3;

// ICIF flags (C macros from probe.h)
const ICIF_QUERY: ULONG = 0x0001;
const ICIF_PROBE_READ_WRITE: ULONG = 0x0003;

const STATUS_USER_APC: NTSTATUS = 0x000000C0_u32 as NTSTATUS;

#[no_mangle]
pub static mut IoCompletionType: POBJECT_TYPE = core::ptr::null_mut();

#[no_mangle]
pub static mut IoCompletionPacketLookaside: GENERAL_LOOKASIDE = unsafe { core::mem::zeroed() };

#[no_mangle]
pub static mut IopCompletionMapping: _GENERIC_MAPPING = _GENERIC_MAPPING {
    GenericRead: STANDARD_RIGHTS_READ | IO_COMPLETION_QUERY_STATE,
    GenericWrite: STANDARD_RIGHTS_WRITE | IO_COMPLETION_MODIFY_STATE,
    GenericExecute: STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
    GenericAll: IO_COMPLETION_ALL_ACCESS,
};

static IoCompletionInfoClass: [_INFORMATION_CLASS_INFO; 1] = [
    _INFORMATION_CLASS_INFO {
        RequiredSizeQUERY: core::mem::size_of::<IO_COMPLETION_BASIC_INFORMATION>() as USHORT,
        AlignmentQUERY: core::mem::align_of::<ULONG>() as UCHAR,
        RequiredSizeSET: 0,
        AlignmentSET: 0,
        Flags: ICIF_QUERY as USHORT,
    },
];

const IopCompletionPacketIrp: ULONG = 0x00000000;
const IopCompletionPacketMini: ULONG = 0x00000001;

unsafe extern "system" fn IopUnloadSafeCompletion(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    IopUnloadSafeCompletionInner(DeviceObject, Irp, Context)
}

#[inline(always)]
unsafe fn IopUnloadSafeCompletionInner(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    Context: PVOID,
) -> NTSTATUS {
    let UnsafeContext = Context as PIO_UNLOAD_SAFE_COMPLETION_CONTEXT;

    ObReferenceObject((*UnsafeContext).DeviceObject as PVOID);

    let Status = ((*UnsafeContext).CompletionRoutine.unwrap())(
        DeviceObject,
        Irp,
        (*UnsafeContext).Context,
    );

    ObDereferenceObject((*UnsafeContext).DeviceObject as PVOID);

    ExFreePool(UnsafeContext as PVOID);
    Status
}

unsafe fn IopFreeMiniPacket(Packet: PIOP_MINI_COMPLETION_PACKET) {
    let Prcb = KeGetCurrentPrcb();
    let mut List: PNPAGED_LOOKASIDE_LIST;

    List = (*((*Prcb).PPLookasideList.as_ptr().add(_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize))).P as PNPAGED_LOOKASIDE_LIST;
    (*List).L.TotalFrees += 1;

    if ExQueryDepthSList(core::ptr::addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) >= (*List).L.Depth {
        (*List).L.__bindgen_anon_3.FreeMisses += 1;

        List = (*((*Prcb).PPLookasideList.as_ptr().add(_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize))).L as PNPAGED_LOOKASIDE_LIST;
        (*List).L.TotalFrees += 1;

        if ExQueryDepthSList(core::ptr::addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) >= (*List).L.Depth {
            (*List).L.__bindgen_anon_3.FreeMisses += 1;
            ExFreePool(Packet as PVOID);
            return;
        }
    }

    InterlockedPushEntrySList(
        core::ptr::addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER,
        Packet as PSLIST_ENTRY,
    );
}

#[no_mangle]
pub unsafe extern "system" fn IopDeleteIoCompletion(ObjectBody: PVOID) {
    let ObjectBody = opaque_ptr(ObjectBody);
    let Queue = ObjectBody as PKQUEUE;
    let mut CurrentEntry: PLIST_ENTRY;
    let mut Irp: PIRP;
    let mut Packet: PIOP_MINI_COMPLETION_PACKET;

    let FirstEntry = KeRundownQueue(Queue);
    if !FirstEntry.is_null() {
        CurrentEntry = FirstEntry;
        loop {
            Packet = CONTAINING_RECORD!(
                CurrentEntry,
                _IOP_MINI_COMPLETION_PACKET,
                __bindgen_anon_1.ListEntry
            ) as PIOP_MINI_COMPLETION_PACKET;

            CurrentEntry = (*CurrentEntry).Flink;

            if (*Packet).__bindgen_anon_1.__bindgen_anon_1.PacketType == IopCompletionPacketIrp {
                Irp = CONTAINING_RECORD!(
                    Packet,
                    _IRP,
                    Tail.Overlay.__bindgen_anon_2.ListEntry
                ) as PIRP;
                IoFreeIrp(Irp);
            } else {
                IopFreeMiniPacket(Packet);
            }

            if FirstEntry == CurrentEntry {
                break;
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "system" fn IoSetIoCompletion(
    IoCompletion: PVOID,
    KeyContext: PVOID,
    ApcContext: PVOID,
    IoStatus: NTSTATUS,
    IoStatusInformation: ULONG_PTR,
    Quota: BOOLEAN,
) -> NTSTATUS {
    let IoCompletion = opaque_ptr(IoCompletion);
    let KeyContext = opaque_ptr(KeyContext);
    let ApcContext = opaque_ptr(ApcContext);
    let Queue = IoCompletion as PKQUEUE;
    let mut List: PNPAGED_LOOKASIDE_LIST;
    let Prcb = KeGetCurrentPrcb();
    let mut Packet: PIOP_MINI_COMPLETION_PACKET;

    List = (*((*Prcb).PPLookasideList.as_ptr().add(_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize))).P as PNPAGED_LOOKASIDE_LIST;

    (*List).L.TotalAllocates += 1;
    Packet = InterlockedPopEntrySList(core::ptr::addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) as PIOP_MINI_COMPLETION_PACKET;

    if Packet.is_null() {
        (*List).L.__bindgen_anon_2.AllocateMisses += 1;

        List = (*((*Prcb).PPLookasideList.as_ptr().add(_PP_NPAGED_LOOKASIDE_NUMBER_LookasideCompletionList as usize))).L as PNPAGED_LOOKASIDE_LIST;

        (*List).L.TotalAllocates += 1;
        Packet = InterlockedPopEntrySList(core::ptr::addr_of_mut!((*List).L.__bindgen_anon_1.ListHead) as *mut SLIST_HEADER) as PIOP_MINI_COMPLETION_PACKET;
    }

    if Packet.is_null() {
        (*List).L.__bindgen_anon_2.AllocateMisses += 1;

        Packet = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<_IOP_MINI_COMPLETION_PACKET>(),
            u32::from_le_bytes(*b"COI\0"),
        ) as PIOP_MINI_COMPLETION_PACKET;
    }

    if !Packet.is_null() {
        (*Packet).__bindgen_anon_1.__bindgen_anon_1.PacketType = IopCompletionPacketMini;
        (*Packet).KeyContext = KeyContext;
        (*Packet).ApcContext = ApcContext;
        (*Packet).IoStatus = IoStatus;
        (*Packet).IoStatusInformation = IoStatusInformation as _;

        KeInsertQueue(Queue, core::ptr::addr_of_mut!((*Packet).__bindgen_anon_1.ListEntry) as PLIST_ENTRY);
    } else {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoSetCompletionRoutineEx(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    CompletionRoutine: PIO_COMPLETION_ROUTINE,
    Context: PVOID,
    InvokeOnSuccess: BOOLEAN,
    InvokeOnError: BOOLEAN,
    InvokeOnCancel: BOOLEAN,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    let Irp = opaque_ptr(Irp);
    let Context = opaque_ptr(Context);
    let UnloadContext = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<_IO_UNLOAD_SAFE_COMPLETION_CONTEXT>(),
        u32::from_le_bytes(*b"UsIo"),
    ) as PIO_UNLOAD_SAFE_COMPLETION_CONTEXT;

    if UnloadContext.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    (*UnloadContext).DeviceObject = DeviceObject;
    (*UnloadContext).Context = Context;
    (*UnloadContext).CompletionRoutine = CompletionRoutine;

    ntkrnl_sys::inline::tier1::IoSetCompletionRoutine(
        Irp,
        Some(IopUnloadSafeCompletion),
        UnloadContext as PVOID,
        InvokeOnSuccess,
        InvokeOnError,
        InvokeOnCancel,
    );

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn NtCreateIoCompletion(
    IoCompletionHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
    NumberOfConcurrentThreads: ULONG,
) -> NTSTATUS {
    let IoCompletionHandle = opaque_ptr(IoCompletionHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let mut Queue: PKQUEUE = core::ptr::null_mut();
    let mut hIoCompletionHandle: HANDLE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    if PreviousMode != 0i8 {
        Status = 'seh: {
            if let Err(e) = ntkrnl_sys::probe::probe_user_buffer_write(
                IoCompletionHandle as *const core::ffi::c_void,
                core::mem::size_of::<HANDLE>(),
                core::mem::align_of::<HANDLE>(),
                PreviousMode,
            ) {
                break 'seh e;
            }
            STATUS_SUCCESS
        };
        if Status < 0 {
            return Status;
        }
    }

    Status = ObCreateObject(
        PreviousMode,
        IoCompletionType,
        ObjectAttributes,
        PreviousMode,
        core::ptr::null_mut(),
        core::mem::size_of::<_KQUEUE>() as ULONG,
        0,
        0,
        &mut Queue as *mut PKQUEUE as *mut PVOID,
    );

    if Status >= 0 {
        KeInitializeQueue(Queue, NumberOfConcurrentThreads);

        Status = ObInsertObject(
            Queue as PVOID,
            core::ptr::null_mut(),
            DesiredAccess,
            0,
            core::ptr::null_mut(),
            &mut hIoCompletionHandle,
        );

        if Status >= 0 {
            if let Err(e) = ntkrnl_sys::probe::write_user(
                IoCompletionHandle,
                hIoCompletionHandle,
                PreviousMode,
            ) {
                Status = e;
            }
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtOpenIoCompletion(
    IoCompletionHandle: PHANDLE,
    DesiredAccess: ACCESS_MASK,
    ObjectAttributes: POBJECT_ATTRIBUTES,
) -> NTSTATUS {
    let IoCompletionHandle = opaque_ptr(IoCompletionHandle);
    let ObjectAttributes = opaque_ptr(ObjectAttributes);
    let PreviousMode = ExGetPreviousMode();
    let mut hIoCompletionHandle: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;

    if PreviousMode != 0i8 {
        Status = 'seh: {
            if let Err(e) = ntkrnl_sys::probe::probe_user_buffer_write(
                IoCompletionHandle as *const core::ffi::c_void,
                core::mem::size_of::<HANDLE>(),
                core::mem::align_of::<HANDLE>(),
                PreviousMode,
            ) {
                break 'seh e;
            }
            STATUS_SUCCESS
        };
        if Status < 0 {
            return Status;
        }
    }

    Status = ObOpenObjectByName(
        ObjectAttributes,
        IoCompletionType,
        PreviousMode,
        core::ptr::null_mut(),
        DesiredAccess,
        core::ptr::null_mut(),
        &mut hIoCompletionHandle,
    );

    if Status >= 0 {
        if let Err(e) = ntkrnl_sys::probe::write_user(
            IoCompletionHandle,
            hIoCompletionHandle,
            PreviousMode,
        ) {
            Status = e;
        }
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtQueryIoCompletion(
    IoCompletionHandle: HANDLE,
    IoCompletionInformationClass: _IO_COMPLETION_INFORMATION_CLASS,
    IoCompletionInformation: PVOID,
    IoCompletionInformationLength: ULONG,
    ResultLength: *mut ULONG,
) -> NTSTATUS {
    let IoCompletionHandle = opaque_ptr(IoCompletionHandle);
    let IoCompletionInformation = opaque_ptr(IoCompletionInformation);
    let ResultLength = opaque_ptr(ResultLength);
    let mut Queue: PKQUEUE = core::ptr::null_mut();
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;

    Status = DefaultQueryInfoBufferCheck(
        IoCompletionInformationClass,
        IoCompletionInfoClass.as_ptr(),
        (core::mem::size_of_val(&IoCompletionInfoClass) / core::mem::size_of::<_INFORMATION_CLASS_INFO>()) as ULONG,
        ICIF_PROBE_READ_WRITE,
        IoCompletionInformation,
        IoCompletionInformationLength,
        ResultLength,
        core::ptr::null_mut(),
        PreviousMode,
    );

    if Status < 0 {
        return Status;
    }

    Status = ObReferenceObjectByHandle(
        IoCompletionHandle,
        IO_COMPLETION_QUERY_STATE,
        IoCompletionType,
        PreviousMode,
        &mut Queue as *mut PKQUEUE as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        Status = 'seh: {
            if let Err(e) = ntkrnl_sys::probe::write_user(
                core::ptr::addr_of_mut!((*(IoCompletionInformation as PIO_COMPLETION_BASIC_INFORMATION)).Depth),
                KeReadStateQueue(Queue),
                PreviousMode,
            ) {
                break 'seh e;
            }

            if !ResultLength.is_null() {
                if let Err(e) = ntkrnl_sys::probe::write_user(
                    ResultLength,
                    core::mem::size_of::<IO_COMPLETION_BASIC_INFORMATION>() as ULONG,
                    PreviousMode,
                ) {
                    break 'seh e;
                }
            }
            STATUS_SUCCESS
        };

        ObDereferenceObject(Queue as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtRemoveIoCompletion(
    IoCompletionHandle: HANDLE,
    KeyContext: *mut PVOID,
    ApcContext: *mut PVOID,
    IoStatusBlock: PIO_STATUS_BLOCK,
    Timeout: PLARGE_INTEGER,
) -> NTSTATUS {
    let IoCompletionHandle = opaque_ptr(IoCompletionHandle);
    let KeyContext = opaque_ptr(KeyContext);
    let ApcContext = opaque_ptr(ApcContext);
    let IoStatusBlock = opaque_ptr(IoStatusBlock);
    let Timeout = opaque_ptr(Timeout);
    let mut SafeTimeout: LARGE_INTEGER = core::mem::zeroed();
    let mut Queue: PKQUEUE = core::ptr::null_mut();
    let mut Packet: PIOP_MINI_COMPLETION_PACKET;
    let mut ListEntry: PLIST_ENTRY;
    let PreviousMode = ExGetPreviousMode();
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut Apc: PVOID;
    let mut Key: PVOID;
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut TimeoutPtr = Timeout;

    if PreviousMode != 0i8 {
        Status = 'seh: {
            if let Err(e) = ntkrnl_sys::probe::probe_user_buffer_write(
                KeyContext as *const core::ffi::c_void,
                core::mem::size_of::<PVOID>(),
                core::mem::align_of::<PVOID>(),
                PreviousMode,
            ) {
                break 'seh e;
            }
            if let Err(e) = ntkrnl_sys::probe::probe_user_buffer_write(
                ApcContext as *const core::ffi::c_void,
                core::mem::size_of::<PVOID>(),
                core::mem::align_of::<PVOID>(),
                PreviousMode,
            ) {
                break 'seh e;
            }
            if let Err(e) = ntkrnl_sys::probe::probe_user_buffer_write(
                IoStatusBlock as *const core::ffi::c_void,
                core::mem::size_of::<_IO_STATUS_BLOCK>(),
                core::mem::align_of::<_IO_STATUS_BLOCK>(),
                PreviousMode,
            ) {
                break 'seh e;
            }
            if !Timeout.is_null() {
                match ntkrnl_sys::probe::probe_read_user::<LARGE_INTEGER>(Timeout, PreviousMode) {
                    Ok(val) => SafeTimeout = val,
                    Err(e) => break 'seh e,
                }
                TimeoutPtr = &mut SafeTimeout;
            }
            STATUS_SUCCESS
        };
        if Status < 0 {
            return Status;
        }
    }

    Status = ObReferenceObjectByHandle(
        IoCompletionHandle,
        IO_COMPLETION_MODIFY_STATE,
        IoCompletionType,
        PreviousMode,
        &mut Queue as *mut PKQUEUE as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        ListEntry = KeRemoveQueue(Queue, PreviousMode, TimeoutPtr);

        if (ListEntry as NTSTATUS == STATUS_TIMEOUT) || (ListEntry as NTSTATUS == STATUS_USER_APC) {
            Status = ListEntry as NTSTATUS;
        } else {
            Packet = CONTAINING_RECORD!(
                ListEntry,
                _IOP_MINI_COMPLETION_PACKET,
                __bindgen_anon_1.ListEntry
            ) as PIOP_MINI_COMPLETION_PACKET;

            if (*Packet).__bindgen_anon_1.__bindgen_anon_1.PacketType == IopCompletionPacketIrp {
                Irp = CONTAINING_RECORD!(
                    ListEntry,
                    _IRP,
                    Tail.Overlay.__bindgen_anon_2.ListEntry
                ) as PIRP;

                Key = (*Irp).Tail.CompletionKey;
                Apc = (*Irp).Overlay.AsynchronousParameters.UserApcContext;
                IoStatus = (*Irp).IoStatus;

                IoFreeIrp(Irp);
            } else {
                Key = (*Packet).KeyContext;
                Apc = (*Packet).ApcContext;
                IoStatus.__bindgen_anon_1.Status = (*Packet).IoStatus;
                IoStatus.Information = (*Packet).IoStatusInformation;

                IopFreeMiniPacket(Packet);
            }

            Status = 'seh: {
                if let Err(e) = ntkrnl_sys::probe::write_user(ApcContext, Apc, PreviousMode) {
                    break 'seh e;
                }
                if let Err(e) = ntkrnl_sys::probe::write_user(KeyContext, Key, PreviousMode) {
                    break 'seh e;
                }
                if let Err(e) = ntkrnl_sys::probe::write_user(IoStatusBlock, IoStatus, PreviousMode) {
                    break 'seh e;
                }
                STATUS_SUCCESS
            };
        }

        ObDereferenceObject(Queue as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn NtSetIoCompletion(
    IoCompletionPortHandle: HANDLE,
    CompletionKey: PVOID,
    CompletionContext: PVOID,
    CompletionStatus: NTSTATUS,
    CompletionInformation: ULONG,
) -> NTSTATUS {
    let IoCompletionPortHandle = opaque_ptr(IoCompletionPortHandle);
    let CompletionKey = opaque_ptr(CompletionKey);
    let CompletionContext = opaque_ptr(CompletionContext);
    let mut Status: NTSTATUS;
    let mut Queue: PKQUEUE = core::ptr::null_mut();

    Status = ObReferenceObjectByHandle(
        IoCompletionPortHandle,
        IO_COMPLETION_MODIFY_STATE,
        IoCompletionType,
        ExGetPreviousMode(),
        &mut Queue as *mut PKQUEUE as *mut PVOID,
        core::ptr::null_mut(),
    );

    if Status >= 0 {
        Status = IoSetIoCompletion(
            Queue as PVOID,
            CompletionKey,
            CompletionContext,
            CompletionStatus,
            CompletionInformation as ULONG_PTR,
            1u8,
        );

        ObDereferenceObject(Queue as PVOID);
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/iomgr/iocomp.c
//  * PURPOSE:         I/O Wrappers (called Completion Ports) for Kernel Queues
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  *                  Thomas Weidenmueller (w3seek@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// POBJECT_TYPE IoCompletionType;
//
// GENERAL_LOOKASIDE IoCompletionPacketLookaside;
//
// GENERIC_MAPPING IopCompletionMapping =
// {
//     STANDARD_RIGHTS_READ | IO_COMPLETION_QUERY_STATE,
//     STANDARD_RIGHTS_WRITE | IO_COMPLETION_MODIFY_STATE,
//     STANDARD_RIGHTS_EXECUTE | SYNCHRONIZE,
//     IO_COMPLETION_ALL_ACCESS
// };
//
// static const INFORMATION_CLASS_INFO IoCompletionInfoClass[] =
// {
//      /* IoCompletionBasicInformation */
//     IQS_SAME(IO_COMPLETION_BASIC_INFORMATION, ULONG, ICIF_QUERY),
// };
//
// /* PRIVATE FUNCTIONS *********************************************************/
//
// NTSTATUS
// NTAPI
// IopUnloadSafeCompletion(IN PDEVICE_OBJECT DeviceObject,
//                         IN PIRP Irp,
//                         IN PVOID Context)
// {
//     NTSTATUS Status;
//     PIO_UNLOAD_SAFE_COMPLETION_CONTEXT UnsafeContext = Context;
//
//     /* Reference the device object */
//     ObReferenceObject(UnsafeContext->DeviceObject);
//
//     /* Call the completion routine */
//     Status= UnsafeContext->CompletionRoutine(DeviceObject,
//                                              Irp,
//                                              UnsafeContext->Context);
//
//     /* Dereference the device object */
//     ObDereferenceObject(UnsafeContext->DeviceObject);
//
//     /* Free our context */
//     ExFreePool(UnsafeContext);
//     return Status;
// }
//
// VOID
// NTAPI
// IopFreeMiniPacket(PIOP_MINI_COMPLETION_PACKET Packet)
// {
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PNPAGED_LOOKASIDE_LIST List;
//
//     /* Use the P List */
//     List = (PNPAGED_LOOKASIDE_LIST)Prcb->
//             PPLookasideList[LookasideCompletionList].P;
//     List->L.TotalFrees++;
//
//     /* Check if the Free was within the Depth or not */
//     if (ExQueryDepthSList(&List->L.ListHead) >= List->L.Depth)
//     {
//         /* Let the balancer know */
//         List->L.FreeMisses++;
//
//         /* Use the L List */
//         List = (PNPAGED_LOOKASIDE_LIST)Prcb->
//                 PPLookasideList[LookasideCompletionList].L;
//         List->L.TotalFrees++;
//
//         /* Check if the Free was within the Depth or not */
//         if (ExQueryDepthSList(&List->L.ListHead) >= List->L.Depth)
//         {
//             /* All lists failed, use the pool */
//             List->L.FreeMisses++;
//             ExFreePool(Packet);
//             return;
//         }
//     }
//
//     /* The free was within dhe Depth */
//     InterlockedPushEntrySList(&List->L.ListHead, (PSLIST_ENTRY)Packet);
// }
//
// VOID
// NTAPI
// IopDeleteIoCompletion(PVOID ObjectBody)
// {
//     PKQUEUE Queue = ObjectBody;
//     PLIST_ENTRY FirstEntry;
//     PLIST_ENTRY CurrentEntry;
//     PIRP Irp;
//     PIOP_MINI_COMPLETION_PACKET Packet;
//
//     /* Rundown the Queue */
//     FirstEntry = KeRundownQueue(Queue);
//     if (FirstEntry)
//     {
//         /* Loop the packets */
//         CurrentEntry = FirstEntry;
//         do
//         {
//             /* Get the Packet */
//             Packet = CONTAINING_RECORD(CurrentEntry,
//                                        IOP_MINI_COMPLETION_PACKET,
//                                        ListEntry);
//
//             /* Go to next Entry */
//             CurrentEntry = CurrentEntry->Flink;
//
//             /* Check if it's part of an IRP, or a separate packet */
//             if (Packet->PacketType == IopCompletionPacketIrp)
//             {
//                 /* Get the IRP and free it */
//                 Irp = CONTAINING_RECORD(Packet, IRP, Tail.Overlay.ListEntry);
//                 IoFreeIrp(Irp);
//             }
//             else
//             {
//                 /* Use common routine */
//                 IopFreeMiniPacket(Packet);
//             }
//         } while (FirstEntry != CurrentEntry);
//     }
// }
//
// /* PUBLIC FUNCTIONS **********************************************************/
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoSetIoCompletion(IN PVOID IoCompletion,
//                   IN PVOID KeyContext,
//                   IN PVOID ApcContext,
//                   IN NTSTATUS IoStatus,
//                   IN ULONG_PTR IoStatusInformation,
//                   IN BOOLEAN Quota)
// {
//     PKQUEUE Queue = (PKQUEUE)IoCompletion;
//     PNPAGED_LOOKASIDE_LIST List;
//     PKPRCB Prcb = KeGetCurrentPrcb();
//     PIOP_MINI_COMPLETION_PACKET Packet;
//
//     /* Get the P List */
//     List = (PNPAGED_LOOKASIDE_LIST)Prcb->
//             PPLookasideList[LookasideCompletionList].P;
//
//     /* Try to allocate the Packet */
//     List->L.TotalAllocates++;
//     Packet = (PVOID)InterlockedPopEntrySList(&List->L.ListHead);
//
//     /* Check if that failed, use the L list if it did */
//     if (!Packet)
//     {
//         /* Let the balancer know */
//         List->L.AllocateMisses++;
//
//         /* Get L List */
//         List = (PNPAGED_LOOKASIDE_LIST)Prcb->
//                 PPLookasideList[LookasideCompletionList].L;
//
//         /* Try to allocate the Packet */
//         List->L.TotalAllocates++;
//         Packet = (PVOID)InterlockedPopEntrySList(&List->L.ListHead);
//     }
//
//     /* Still failed, use pool */
//     if (!Packet)
//     {
//         /* Let the balancer know */
//         List->L.AllocateMisses++;
//
//         /* Allocate from Nonpaged Pool */
//         Packet = ExAllocatePoolWithTag(NonPagedPool, sizeof(*Packet), IOC_TAG);
//     }
//
//     /* Make sure we have one by now... */
//     if (Packet)
//     {
//         /* Set up the Packet */
//         Packet->PacketType = IopCompletionPacketMini;
//         Packet->KeyContext = KeyContext;
//         Packet->ApcContext = ApcContext;
//         Packet->IoStatus = IoStatus;
//         Packet->IoStatusInformation = IoStatusInformation;
//
//         /* Insert the Queue */
//         KeInsertQueue(Queue, &Packet->ListEntry);
//     }
//     else
//     {
//         /* Out of memory, fail */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Return Success */
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @implemented
//  */
// NTSTATUS
// NTAPI
// IoSetCompletionRoutineEx(IN PDEVICE_OBJECT DeviceObject,
//                          IN PIRP Irp,
//                          IN PIO_COMPLETION_ROUTINE CompletionRoutine,
//                          IN PVOID Context,
//                          IN BOOLEAN InvokeOnSuccess,
//                          IN BOOLEAN InvokeOnError,
//                          IN BOOLEAN InvokeOnCancel)
// {
//     PIO_UNLOAD_SAFE_COMPLETION_CONTEXT UnloadContext;
//
//     /* Allocate the context */
//     UnloadContext = ExAllocatePoolWithTag(NonPagedPool,
//                                           sizeof(*UnloadContext),
//                                           'sUoI');
//     if (!UnloadContext) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Set up the context */
//     UnloadContext->DeviceObject = DeviceObject;
//     UnloadContext->Context = Context;
//     UnloadContext->CompletionRoutine = CompletionRoutine;
//
//     /* Now set the completion routine */
//     IoSetCompletionRoutine(Irp,
//                            IopUnloadSafeCompletion,
//                            UnloadContext,
//                            InvokeOnSuccess,
//                            InvokeOnError,
//                            InvokeOnCancel);
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// NtCreateIoCompletion(OUT PHANDLE IoCompletionHandle,
//                      IN ACCESS_MASK DesiredAccess,
//                      IN POBJECT_ATTRIBUTES ObjectAttributes,
//                      IN ULONG NumberOfConcurrentThreads)
// {
//     PKQUEUE Queue;
//     HANDLE hIoCompletionHandle;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if this was a user-mode call */
//     if (PreviousMode != KernelMode)
//     {
//         /* Wrap probing in SEH */
//         _SEH2_TRY
//         {
//             /* Probe the handle */
//             ProbeForWriteHandle(IoCompletionHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Create the Object */
//     Status = ObCreateObject(PreviousMode,
//                             IoCompletionType,
//                             ObjectAttributes,
//                             PreviousMode,
//                             NULL,
//                             sizeof(KQUEUE),
//                             0,
//                             0,
//                             (PVOID*)&Queue);
//     if (NT_SUCCESS(Status))
//     {
//         /* Initialize the Queue */
//         KeInitializeQueue(Queue, NumberOfConcurrentThreads);
//
//         /* Insert it */
//         Status = ObInsertObject(Queue,
//                                 NULL,
//                                 DesiredAccess,
//                                 0,
//                                 NULL,
//                                 &hIoCompletionHandle);
//         if (NT_SUCCESS(Status))
//         {
//             /* Protect writing the handle in SEH */
//             _SEH2_TRY
//             {
//                 /* Write the handle back */
//                 *IoCompletionHandle = hIoCompletionHandle;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//    }
//
//    /* Return Status */
//    return Status;
// }
//
// NTSTATUS
// NTAPI
// NtOpenIoCompletion(OUT PHANDLE IoCompletionHandle,
//                    IN ACCESS_MASK DesiredAccess,
//                    IN POBJECT_ATTRIBUTES ObjectAttributes)
// {
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     HANDLE hIoCompletionHandle;
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check if this was a user-mode call */
//     if (PreviousMode != KernelMode)
//     {
//         /* Wrap probing in SEH */
//         _SEH2_TRY
//         {
//             /* Probe the handle */
//             ProbeForWriteHandle(IoCompletionHandle);
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Open the Object */
//     Status = ObOpenObjectByName(ObjectAttributes,
//                                 IoCompletionType,
//                                 PreviousMode,
//                                 NULL,
//                                 DesiredAccess,
//                                 NULL,
//                                 &hIoCompletionHandle);
//     if (NT_SUCCESS(Status))
//     {
//         /* Protect writing the handle in SEH */
//         _SEH2_TRY
//         {
//             /* Write the handle back */
//             *IoCompletionHandle = hIoCompletionHandle;
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get the exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtQueryIoCompletion(IN  HANDLE IoCompletionHandle,
//                     IN  IO_COMPLETION_INFORMATION_CLASS IoCompletionInformationClass,
//                     OUT PVOID IoCompletionInformation,
//                     IN  ULONG IoCompletionInformationLength,
//                     OUT PULONG ResultLength OPTIONAL)
// {
//     PKQUEUE Queue;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PAGED_CODE();
//
//     /* Check buffers and parameters */
//     Status = DefaultQueryInfoBufferCheck(IoCompletionInformationClass,
//                                          IoCompletionInfoClass,
//                                          sizeof(IoCompletionInfoClass) /
//                                          sizeof(IoCompletionInfoClass[0]),
//                                          ICIF_PROBE_READ_WRITE,
//                                          IoCompletionInformation,
//                                          IoCompletionInformationLength,
//                                          ResultLength,
//                                          NULL,
//                                          PreviousMode);
//     if (!NT_SUCCESS(Status)) return Status;
//
//     /* Get the Object */
//     Status = ObReferenceObjectByHandle(IoCompletionHandle,
//                                        IO_COMPLETION_QUERY_STATE,
//                                        IoCompletionType,
//                                        PreviousMode,
//                                        (PVOID*)&Queue,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Protect write in SEH */
//         _SEH2_TRY
//         {
//             /* Return Info */
//             ((PIO_COMPLETION_BASIC_INFORMATION)IoCompletionInformation)->
//                 Depth = KeReadStateQueue(Queue);
//
//             /* Return Result Length if needed */
//             if (ResultLength)
//             {
//                 *ResultLength = sizeof(IO_COMPLETION_BASIC_INFORMATION);
//             }
//         }
//         _SEH2_EXCEPT(ExSystemExceptionFilter())
//         {
//             /* Get exception code */
//             Status = _SEH2_GetExceptionCode();
//         }
//         _SEH2_END;
//
//         /* Dereference the queue */
//         ObDereferenceObject(Queue);
//     }
//
//     /* Return Status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtRemoveIoCompletion(IN HANDLE IoCompletionHandle,
//                      OUT PVOID *KeyContext,
//                      OUT PVOID *ApcContext,
//                      OUT PIO_STATUS_BLOCK IoStatusBlock,
//                      IN PLARGE_INTEGER Timeout OPTIONAL)
// {
//     LARGE_INTEGER SafeTimeout;
//     PKQUEUE Queue;
//     PIOP_MINI_COMPLETION_PACKET Packet;
//     PLIST_ENTRY ListEntry;
//     KPROCESSOR_MODE PreviousMode = ExGetPreviousMode();
//     NTSTATUS Status;
//     PIRP Irp;
//     PVOID Apc, Key;
//     IO_STATUS_BLOCK IoStatus;
//     PAGED_CODE();
//
//     /* Check if the call was from user mode */
//     if (PreviousMode != KernelMode)
//     {
//         /* Protect probes in SEH */
//         _SEH2_TRY
//         {
//             /* Probe the pointers */
//             ProbeForWritePointer(KeyContext);
//             ProbeForWritePointer(ApcContext);
//
//             /* Probe the I/O Status Block */
//             ProbeForWriteIoStatusBlock(IoStatusBlock);
//             if (Timeout)
//             {
//                 /* Probe and capture the timeout */
//                 SafeTimeout = ProbeForReadLargeInteger(Timeout);
//                 Timeout = &SafeTimeout;
//             }
//         }
//         _SEH2_EXCEPT(EXCEPTION_EXECUTE_HANDLER)
//         {
//             /* Return the exception code */
//             _SEH2_YIELD(return _SEH2_GetExceptionCode());
//         }
//         _SEH2_END;
//     }
//
//     /* Open the Object */
//     Status = ObReferenceObjectByHandle(IoCompletionHandle,
//                                        IO_COMPLETION_MODIFY_STATE,
//                                        IoCompletionType,
//                                        PreviousMode,
//                                        (PVOID*)&Queue,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Remove queue */
//         ListEntry = KeRemoveQueue(Queue, PreviousMode, Timeout);
//
//         /* If we got a timeout or user_apc back, return the status */
//         if (((NTSTATUS)(ULONG_PTR)ListEntry == STATUS_TIMEOUT) ||
//             ((NTSTATUS)(ULONG_PTR)ListEntry == STATUS_USER_APC))
//         {
//             /* Set this as the status */
//             Status = (NTSTATUS)(ULONG_PTR)ListEntry;
//         }
//         else
//         {
//             /* Get the Packet Data */
//             Packet = CONTAINING_RECORD(ListEntry,
//                                        IOP_MINI_COMPLETION_PACKET,
//                                        ListEntry);
//
//             /* Check if this is piggybacked on an IRP */
//             if (Packet->PacketType == IopCompletionPacketIrp)
//             {
//                 /* Get the IRP */
//                 Irp = CONTAINING_RECORD(ListEntry,
//                                         IRP,
//                                         Tail.Overlay.ListEntry);
//
//                 /* Save values */
//                 Key = Irp->Tail.CompletionKey;
//                 Apc = Irp->Overlay.AsynchronousParameters.UserApcContext;
//                 IoStatus = Irp->IoStatus;
//
//                 /* Free the IRP */
//                 IoFreeIrp(Irp);
//             }
//             else
//             {
//                 /* Save values */
//                 Key = Packet->KeyContext;
//                 Apc = Packet->ApcContext;
//                 IoStatus.Status = Packet->IoStatus;
//                 IoStatus.Information = Packet->IoStatusInformation;
//
//                 /* Free the packet */
//                 IopFreeMiniPacket(Packet);
//             }
//
//             /* Enter SEH to write back the values */
//             _SEH2_TRY
//             {
//                 /* Write the values to caller */
//                 *ApcContext = Apc;
//                 *KeyContext = Key;
//                 *IoStatusBlock = IoStatus;
//             }
//             _SEH2_EXCEPT(ExSystemExceptionFilter())
//             {
//                 /* Get the exception code */
//                 Status = _SEH2_GetExceptionCode();
//             }
//             _SEH2_END;
//         }
//
//         /* Dereference the Object */
//         ObDereferenceObject(Queue);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// NTSTATUS
// NTAPI
// NtSetIoCompletion(IN HANDLE IoCompletionPortHandle,
//                   IN PVOID CompletionKey,
//                   IN PVOID CompletionContext,
//                   IN NTSTATUS CompletionStatus,
//                   IN ULONG CompletionInformation)
// {
//     NTSTATUS Status;
//     PKQUEUE Queue;
//     PAGED_CODE();
//
//     /* Get the Object */
//     Status = ObReferenceObjectByHandle(IoCompletionPortHandle,
//                                        IO_COMPLETION_MODIFY_STATE,
//                                        IoCompletionType,
//                                        ExGetPreviousMode(),
//                                        (PVOID*)&Queue,
//                                        NULL);
//     if (NT_SUCCESS(Status))
//     {
//         /* Set the Completion */
//         Status = IoSetIoCompletion(Queue,
//                                    CompletionKey,
//                                    CompletionContext,
//                                    CompletionStatus,
//                                    CompletionInformation,
//                                    TRUE);
//
//         /* Dereference the object */
//         ObDereferenceObject(Queue);
//     }
//
//     /* Return status */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
