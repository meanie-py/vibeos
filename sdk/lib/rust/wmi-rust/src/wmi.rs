use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ddk::{
    ExFreePool, IoCreateDriver, ObReferenceObjectByPointer,
    GUID, HANDLE, LPCGUID, LPGUID, PDEVICE_OBJECT, PUNICODE_STRING, TRACEHANDLE,
    WMI_NOTIFICATION_CALLBACK, _DRIVER_OBJECT, _EVENT_TRACE_HEADER, _UNICODE_STRING, va_list,
};
use ntkrnl_sys::ob::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

type PWMI_LOGGER_INFORMATION = PVOID;

#[repr(C)]
enum WMI_CLOCK_TYPE {
    WMICT_DEFAULT,
    WMICT_SYSTEMTIME,
    WMICT_PERFCOUNTER,
    WMICT_PROCESS,
    WMICT_THREAD,
    WMICT_CPUCYCLE,
}

#[repr(C)]
struct WNODE_HEADER {
    BufferSize: ULONG,
    ProviderId: ULONG,
    HistoricalContext: ULONGLONG,
    TimeStamp: LARGE_INTEGER,
    Guid: GUID,
    ClientContext: ULONG,
    Flags: ULONG,
}

type PWNODE_HEADER = *mut WNODE_HEADER;

const WNODE_FLAG_TRACED_GUID: ULONG = 0x00020000;
const WMIGUID_QUERY: ULONG = 0x0001;

extern "system" {
    fn WmipInitializeGuidObjectType() -> NTSTATUS;
}

extern "system" {
    fn WmipDriverEntry(DriverObject: *mut _DRIVER_OBJECT, RegistryPath: *mut _UNICODE_STRING) -> NTSTATUS;
}

extern "system" {
    fn WmipOpenGuidObject(
        DataBlockGuid: LPCGUID,
        DesiredAccess: ULONG,
        AccessMode: i8,
        GuidObjectHandle: *mut HANDLE,
        DataBlockObject: *mut PVOID,
    ) -> NTSTATUS;
    fn WmipQueryRawSMBiosTables(
        InOutBufferSize: *mut ULONG,
        OutBuffer: PVOID,
    ) -> NTSTATUS;
    static WmipGuidObjectType: *mut PVOID;
    static MSSmBios_RawSMBiosTables_GUID: GUID;
}

#[repr(C)]
struct WMIP_GUID_OBJECT {
    Guid: GUID,
}

type PWMIP_GUID_OBJECT = *mut WMIP_GUID_OBJECT;

#[no_mangle]
pub unsafe extern "system" fn IsEqualGUID(guid1: *const GUID, guid2: *const GUID) -> bool {
    core::ptr::eq(guid1 as *const u8, guid2 as *const u8) ||
    ((*guid1).Data1 == (*guid2).Data1 &&
     (*guid1).Data2 == (*guid2).Data2 &&
     (*guid1).Data3 == (*guid2).Data3 &&
     (*guid1).Data4 == (*guid2).Data4)
}

#[no_mangle]
pub unsafe extern "system" fn WmiInitialize() -> BOOLEAN {
    let mut driver_name = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };
    
    static DRIVER_NAME_STR: [u16; 17] = [
        b'\\' as u16, b'D' as u16, b'r' as u16, b'i' as u16, b'v' as u16,
        b'e' as u16, b'r' as u16, b'\\' as u16, b'W' as u16, b'M' as u16,
        b'I' as u16, b'x' as u16, b'W' as u16, b'D' as u16, b'M' as u16,
        0, 0,
    ];
    
    driver_name.Length = 30;
    driver_name.MaximumLength = 34;
    driver_name.Buffer = DRIVER_NAME_STR.as_ptr() as *mut u16;

    let status = WmipInitializeGuidObjectType();
    if status < 0 {
        return 0u8;
    }

    let status = IoCreateDriver(&mut driver_name as *mut _ as *mut _UNICODE_STRING, core::mem::transmute(WmipDriverEntry as usize));
    if status < 0 {
        return 0u8;
    }

    1u8
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIRegistrationControl(
    DeviceObject: PDEVICE_OBJECT,
    Action: ULONG,
) -> NTSTATUS {
    let DeviceObject = opaque_ptr(DeviceObject);
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIAllocateInstanceIds(
    Guid: *mut GUID,
    InstanceCount: ULONG,
    FirstInstanceId: *mut ULONG,
) -> NTSTATUS {
    let Guid = opaque_ptr(Guid);
    let FirstInstanceId = opaque_ptr(FirstInstanceId);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMISuggestInstanceName(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    SymbolicLinkName: PUNICODE_STRING,
    CombineNames: BOOLEAN,
    SuggestedInstanceName: PUNICODE_STRING,
) -> NTSTATUS {
    let PhysicalDeviceObject = opaque_ptr(PhysicalDeviceObject);
    let SymbolicLinkName = opaque_ptr(SymbolicLinkName);
    let SuggestedInstanceName = opaque_ptr(SuggestedInstanceName);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIWriteEvent(WnodeEventItem: PVOID) -> NTSTATUS {
    let WnodeEventItem = opaque_ptr(WnodeEventItem);
    let header = WnodeEventItem as PWNODE_HEADER;

    if header.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    if (*header).Flags & WNODE_FLAG_TRACED_GUID != 0 {
        return STATUS_SUCCESS;
    }

    ExFreePool(WnodeEventItem);

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIOpenBlock(
    DataBlockGuid: LPCGUID,
    DesiredAccess: ULONG,
    DataBlockObject: *mut PVOID,
) -> NTSTATUS {
    let DataBlockObject = opaque_ptr(DataBlockObject);
    let mut guid_object_handle: HANDLE = core::ptr::null_mut();

    let status = WmipOpenGuidObject(
        DataBlockGuid,
        DesiredAccess,
        0i8,
        &mut guid_object_handle,
        DataBlockObject,
    );
    if status < 0 {
        return status;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIQueryAllData(
    DataBlockObject: PVOID,
    InOutBufferSize: *mut ULONG,
    OutBuffer: PVOID,
) -> NTSTATUS {
    let DataBlockObject = opaque_ptr(DataBlockObject);
    let InOutBufferSize = opaque_ptr(InOutBufferSize);
    let OutBuffer = opaque_ptr(OutBuffer);
    let status = ObReferenceObjectByPointer(
        DataBlockObject,
        WMIGUID_QUERY,
        WmipGuidObjectType as *mut _,
        0i8,
    );
    if status < 0 {
        return status;
    }

    let guid_object = DataBlockObject as PWMIP_GUID_OBJECT;

    let status = if IsEqualGUID(
        core::ptr::addr_of!((*guid_object).Guid),
        &MSSmBios_RawSMBiosTables_GUID,
    ) {
        WmipQueryRawSMBiosTables(InOutBufferSize, OutBuffer)
    } else {
        STATUS_NOT_SUPPORTED
    };

    ObDereferenceObject(DataBlockObject);

    status
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIQueryAllDataMultiple(
    DataBlockObjectList: *mut PVOID,
    ObjectCount: ULONG,
    InOutBufferSize: *mut ULONG,
    OutBuffer: PVOID,
) -> NTSTATUS {
    let DataBlockObjectList = opaque_ptr(DataBlockObjectList);
    let InOutBufferSize = opaque_ptr(InOutBufferSize);
    let OutBuffer = opaque_ptr(OutBuffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIQuerySingleInstance(
    DataBlockObject: PVOID,
    InstanceName: PUNICODE_STRING,
    InOutBufferSize: *mut ULONG,
    OutBuffer: PVOID,
) -> NTSTATUS {
    let DataBlockObject = opaque_ptr(DataBlockObject);
    let InstanceName = opaque_ptr(InstanceName);
    let InOutBufferSize = opaque_ptr(InOutBufferSize);
    let OutBuffer = opaque_ptr(OutBuffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIQuerySingleInstanceMultiple(
    DataBlockObjectList: *mut PVOID,
    InstanceNames: PUNICODE_STRING,
    ObjectCount: ULONG,
    InOutBufferSize: *mut ULONG,
    OutBuffer: PVOID,
) -> NTSTATUS {
    let DataBlockObjectList = opaque_ptr(DataBlockObjectList);
    let InstanceNames = opaque_ptr(InstanceNames);
    let InOutBufferSize = opaque_ptr(InOutBufferSize);
    let OutBuffer = opaque_ptr(OutBuffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMISetSingleInstance(
    DataBlockObject: PVOID,
    InstanceName: PUNICODE_STRING,
    Version: ULONG,
    ValueBufferSize: ULONG,
    ValueBuffer: PVOID,
) -> NTSTATUS {
    let DataBlockObject = opaque_ptr(DataBlockObject);
    let InstanceName = opaque_ptr(InstanceName);
    let ValueBuffer = opaque_ptr(ValueBuffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMISetSingleItem(
    DataBlockObject: PVOID,
    InstanceName: PUNICODE_STRING,
    DataItemId: ULONG,
    Version: ULONG,
    ValueBufferSize: ULONG,
    ValueBuffer: PVOID,
) -> NTSTATUS {
    let DataBlockObject = opaque_ptr(DataBlockObject);
    let InstanceName = opaque_ptr(InstanceName);
    let ValueBuffer = opaque_ptr(ValueBuffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIExecuteMethod(
    DataBlockObject: PVOID,
    InstanceName: PUNICODE_STRING,
    MethodId: ULONG,
    InBufferSize: ULONG,
    OutBufferSize: *mut ULONG,
    InOutBuffer: *mut UCHAR,
) -> NTSTATUS {
    let DataBlockObject = opaque_ptr(DataBlockObject);
    let InstanceName = opaque_ptr(InstanceName);
    let OutBufferSize = opaque_ptr(OutBufferSize);
    let InOutBuffer = opaque_ptr(InOutBuffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMISetNotificationCallback(
    Object: PVOID,
    Callback: WMI_NOTIFICATION_CALLBACK,
    Context: PVOID,
) -> NTSTATUS {
    let Object = opaque_ptr(Object);
    let Context = opaque_ptr(Context);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIHandleToInstanceName(
    DataBlockObject: PVOID,
    FileHandle: HANDLE,
    InstanceName: PUNICODE_STRING,
) -> NTSTATUS {
    let DataBlockObject = opaque_ptr(DataBlockObject);
    let FileHandle = opaque_ptr(FileHandle);
    let InstanceName = opaque_ptr(InstanceName);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn IoWMIDeviceObjectToInstanceName(
    DataBlockObject: PVOID,
    DeviceObject: PDEVICE_OBJECT,
    InstanceName: PUNICODE_STRING,
) -> NTSTATUS {
    let DataBlockObject = opaque_ptr(DataBlockObject);
    let DeviceObject = opaque_ptr(DeviceObject);
    let InstanceName = opaque_ptr(InstanceName);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn WmiQueryTraceInformation(
    TraceInformationClass: ULONG,
    TraceInformation: PVOID,
    TraceInformationLength: ULONG,
    RequiredLength: *mut ULONG,
    Buffer: PVOID,
) -> NTSTATUS {
    let TraceInformation = opaque_ptr(TraceInformation);
    let RequiredLength = opaque_ptr(RequiredLength);
    let Buffer = opaque_ptr(Buffer);
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "C" fn WmiTraceMessage(
    LoggerHandle: TRACEHANDLE,
    MessageFlags: ULONG,
    MessageGuid: LPGUID,
    MessageNumber: USHORT,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn WmiTraceMessageVa(
    LoggerHandle: TRACEHANDLE,
    MessageFlags: ULONG,
    MessageGuid: LPGUID,
    MessageNumber: USHORT,
    MessageArgList: va_list,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn WmiFlushTrace(LoggerInfo: PWMI_LOGGER_INFORMATION) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn WmiGetClock(
    ClockType: WMI_CLOCK_TYPE,
    Context: PVOID,
) -> LONGLONG {
    let Context = opaque_ptr(Context);
    STATUS_NOT_IMPLEMENTED as LONGLONG
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn WmiGetClock(
    ClockType: WMI_CLOCK_TYPE,
    Context: PVOID,
) -> LONGLONG {
    let Context = opaque_ptr(Context);
    STATUS_NOT_IMPLEMENTED as LONGLONG
}

#[no_mangle]
pub unsafe extern "system" fn WmiQueryTrace(LoggerInfo: PWMI_LOGGER_INFORMATION) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn WmiStartTrace(LoggerInfo: PWMI_LOGGER_INFORMATION) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn WmiStopTrace(LoggerInfo: PWMI_LOGGER_INFORMATION) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[cfg(target_arch = "x86")]
#[no_mangle]
pub unsafe extern "fastcall" fn WmiTraceFastEvent(Wnode: PWNODE_HEADER) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[cfg(not(target_arch = "x86"))]
#[no_mangle]
pub unsafe extern "C" fn WmiTraceFastEvent(Wnode: PWNODE_HEADER) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn WmiUpdateTrace(LoggerInfo: PWMI_LOGGER_INFORMATION) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}

#[no_mangle]
pub unsafe extern "system" fn NtTraceEvent(
    TraceHandle: ULONG,
    Flags: ULONG,
    TraceHeaderLength: ULONG,
    TraceHeader: *mut _EVENT_TRACE_HEADER,
) -> NTSTATUS {
    let TraceHeader = opaque_ptr(TraceHeader);
    STATUS_NOT_IMPLEMENTED
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * LICENSE:         GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/wmi/wmi.c
//  * PURPOSE:         I/O Windows Management Instrumentation (WMI) Support
//  * PROGRAMMERS:     Alex Ionescu (alex.ionescu@reactos.org)
//  */
//
// /* INCLUDES *****************************************************************/
//
// #include <ntoskrnl.h>
// #define INITGUID
// #include <wmiguid.h>
// #include <wmidata.h>
// #include <wmistr.h>
//
// #include "wmip.h"
//
// #define NDEBUG
// #include <debug.h>
//
// typedef PVOID PWMI_LOGGER_INFORMATION; // FIXME
//
// typedef enum _WMI_CLOCK_TYPE
// {
//     WMICT_DEFAULT,
//     WMICT_SYSTEMTIME,
//     WMICT_PERFCOUNTER,
//     WMICT_PROCESS,
//     WMICT_THREAD,
//     WMICT_CPUCYCLE
// } WMI_CLOCK_TYPE;
//
// /* FUNCTIONS *****************************************************************/
//
// BOOLEAN
// NTAPI
// WmiInitialize(
//     VOID)
// {
//     UNICODE_STRING DriverName = RTL_CONSTANT_STRING(L"\\Driver\\WMIxWDM");
//     NTSTATUS Status;
//
//     /* Initialize the GUID object type */
//     Status = WmipInitializeGuidObjectType();
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("WmipInitializeGuidObjectType() failed: 0x%lx\n", Status);
//         return FALSE;
//     }
//
//     /* Create the WMI driver */
//     Status = IoCreateDriver(&DriverName, WmipDriverEntry);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("Failed to create WMI driver: 0x%lx\n", Status);
//         return FALSE;
//     }
//
//     return TRUE;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIRegistrationControl(IN PDEVICE_OBJECT DeviceObject,
//                          IN ULONG Action)
// {
//     DPRINT("IoWMIRegistrationControl() called for DO %p, requesting %lu action, returning success\n",
//         DeviceObject, Action);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIAllocateInstanceIds(IN GUID *Guid,
//                          IN ULONG InstanceCount,
//                          OUT ULONG *FirstInstanceId)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMISuggestInstanceName(IN PDEVICE_OBJECT PhysicalDeviceObject OPTIONAL,
//                          IN PUNICODE_STRING SymbolicLinkName OPTIONAL,
//                          IN BOOLEAN CombineNames,
//                          OUT PUNICODE_STRING SuggestedInstanceName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIWriteEvent(_Inout_ PVOID WnodeEventItem)
// {
//     PWNODE_HEADER Header = WnodeEventItem;
//
//     if(!Header)
//     {
//         DPRINT1("Got NULL Item!\n");
//         return STATUS_INVALID_PARAMETER;
//     }
//
//     DPRINT1("IoWMIWriteEvent() called for WnodeEventItem %p (Flags = 0x%08lx), returning success\n",
//             WnodeEventItem, Header->Flags);
//
//     if (Header->Flags & WNODE_FLAG_TRACED_GUID)
//     {
//         // Never free WnodeEventItem in this case.
//         DPRINT("IoWMIWriteEvent(): Flags has WNODE_FLAG_TRACED_GUID\n");
//         return STATUS_SUCCESS;
//     }
//
//     /* Free the buffer if we are returning success */
//     ExFreePool(WnodeEventItem);
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIOpenBlock(
//     _In_ LPCGUID DataBlockGuid,
//     _In_ ULONG DesiredAccess,
//     _Out_ PVOID *DataBlockObject)
// {
//     HANDLE GuidObjectHandle;
//     NTSTATUS Status;
//
//     /* Open the GIOD object */
//     Status = WmipOpenGuidObject(DataBlockGuid,
//                                 DesiredAccess,
//                                 KernelMode,
//                                 &GuidObjectHandle,
//                                 DataBlockObject);
//     if (!NT_SUCCESS(Status))
//     {
//         DPRINT1("WmipOpenGuidObject failed: 0x%lx\n", Status);
//         return Status;
//     }
//
//
//     return STATUS_SUCCESS;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIQueryAllData(
//     IN PVOID DataBlockObject,
//     IN OUT ULONG *InOutBufferSize,
//     OUT PVOID OutBuffer)
// {
//     PWMIP_GUID_OBJECT GuidObject;
//     NTSTATUS Status;
//
//
//     Status = ObReferenceObjectByPointer(DataBlockObject,
//                                         WMIGUID_QUERY,
//                                         WmipGuidObjectType,
//                                         KernelMode);
//     if (!NT_SUCCESS(Status))
//     {
//         return Status;
//     }
//
//     GuidObject = DataBlockObject;
//
//     /* Huge HACK! */
//     if (IsEqualGUID(&GuidObject->Guid, &MSSmBios_RawSMBiosTables_GUID))
//     {
//         Status = WmipQueryRawSMBiosTables(InOutBufferSize, OutBuffer);
//     }
//     else
//     {
//         Status = STATUS_NOT_SUPPORTED;
//     }
//
//     ObDereferenceObject(DataBlockObject);
//
//     return Status;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIQueryAllDataMultiple(IN PVOID *DataBlockObjectList,
//                           IN ULONG ObjectCount,
//                           IN OUT ULONG *InOutBufferSize,
//                           OUT PVOID OutBuffer)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIQuerySingleInstance(IN PVOID DataBlockObject,
//                          IN PUNICODE_STRING InstanceName,
//                          IN OUT ULONG *InOutBufferSize,
//                          OUT PVOID OutBuffer)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIQuerySingleInstanceMultiple(IN PVOID *DataBlockObjectList,
//                                  IN PUNICODE_STRING InstanceNames,
//                                  IN ULONG ObjectCount,
//                                  IN OUT ULONG *InOutBufferSize,
//                                  OUT PVOID OutBuffer)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMISetSingleInstance(IN PVOID DataBlockObject,
//                        IN PUNICODE_STRING InstanceName,
//                        IN ULONG Version,
//                        IN ULONG ValueBufferSize,
//                        IN PVOID ValueBuffer)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMISetSingleItem(IN PVOID DataBlockObject,
//                    IN PUNICODE_STRING InstanceName,
//                    IN ULONG DataItemId,
//                    IN ULONG Version,
//                    IN ULONG ValueBufferSize,
//                    IN PVOID ValueBuffer)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIExecuteMethod(IN PVOID DataBlockObject,
//                    IN PUNICODE_STRING InstanceName,
//                    IN ULONG MethodId,
//                    IN ULONG InBufferSize,
//                    IN OUT PULONG OutBufferSize,
//                    IN OUT PUCHAR InOutBuffer)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMISetNotificationCallback(IN PVOID Object,
//                              IN WMI_NOTIFICATION_CALLBACK Callback,
//                              IN PVOID Context)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIHandleToInstanceName(IN PVOID DataBlockObject,
//                           IN HANDLE FileHandle,
//                           OUT PUNICODE_STRING InstanceName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// IoWMIDeviceObjectToInstanceName(IN PVOID DataBlockObject,
//                                 IN PDEVICE_OBJECT DeviceObject,
//                                 OUT PUNICODE_STRING InstanceName)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// WmiQueryTraceInformation(IN TRACE_INFORMATION_CLASS TraceInformationClass,
//                          OUT PVOID TraceInformation,
//                          IN ULONG TraceInformationLength,
//                          OUT PULONG RequiredLength OPTIONAL,
//                          IN PVOID Buffer OPTIONAL)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// __cdecl
// WmiTraceMessage(IN TRACEHANDLE LoggerHandle,
//                 IN ULONG MessageFlags,
//                 IN LPGUID MessageGuid,
//                 IN USHORT MessageNumber,
//                 IN ...)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// WmiTraceMessageVa(IN TRACEHANDLE LoggerHandle,
//                   IN ULONG MessageFlags,
//                   IN LPGUID MessageGuid,
//                   IN USHORT MessageNumber,
//                   IN va_list MessageArgList)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// WmiFlushTrace(IN OUT PWMI_LOGGER_INFORMATION LoggerInfo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// LONG64
// FASTCALL
// WmiGetClock(IN WMI_CLOCK_TYPE ClockType,
//             IN PVOID Context)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// WmiQueryTrace(IN OUT PWMI_LOGGER_INFORMATION LoggerInfo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// WmiStartTrace(IN OUT PWMI_LOGGER_INFORMATION LoggerInfo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// WmiStopTrace(IN PWMI_LOGGER_INFORMATION LoggerInfo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// FASTCALL
// WmiTraceFastEvent(IN PWNODE_HEADER Wnode)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// NTSTATUS
// NTAPI
// WmiUpdateTrace(IN OUT PWMI_LOGGER_INFORMATION LoggerInfo)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*
//  * @unimplemented
//  */
// NTSTATUS
// NTAPI
// NtTraceEvent(IN ULONG TraceHandle,
//              IN ULONG Flags,
//              IN ULONG TraceHeaderLength,
//              IN struct _EVENT_TRACE_HEADER* TraceHeader)
// {
//     UNIMPLEMENTED;
//     return STATUS_NOT_IMPLEMENTED;
// }
//
// /*Eof*/
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
