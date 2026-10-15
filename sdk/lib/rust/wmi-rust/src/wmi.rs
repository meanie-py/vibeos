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
    // There is no WMI data provider support. A WPP trace provider (callback registration) is
    // enabled on the spot when the WPPTRACE boot option asks for it, so its messages reach the
    // debug port; everything else is accepted and left alone.
    if Action & WMIREG_FLAG_CALLBACK != 0 && Action & 0xFF == WMIREG_ACTION_REGISTER && !DeviceObject.is_null() {
        let level = wpp_level();
        if level != 0 {
            wpp_enable(DeviceObject as *mut WppControlBlock, level);
        }
    }
    STATUS_SUCCESS
}

// ---- WPP tracing to the debug port (boot option WPPTRACE[=level]) ----------------------------
//
// Windows delivers a WPP provider's messages to an ETW session; there is none here, so a boot
// option stands one in: each provider that registers is enabled at once, at the given level
// (default 3, warnings) with every flag, under our own logger handle, and WmiTraceMessage
// prints the records. A WPP record carries the message number and its arguments, not the
// format string (that lives in the driver's PDB), so a line is the provider, the number, and
// the arguments, with strings shown as text: drivers that trace preformatted messages read
// as they would in a log.

const WMIREG_ACTION_REGISTER: ULONG = 1;
const WMIREG_FLAG_CALLBACK: ULONG = 0x8000_0000;
const IRP_MN_ENABLE_EVENTS: u8 = 0x04;
const TRACE_ENABLE_FLAGS_CLASS: ULONG = 2;
const TRACE_ENABLE_LEVEL_CLASS: ULONG = 3;
/// Our logger handle: any non-zero value WPP stores and hands back. Its low byte is left
/// zero: the enabled level travels there.
const WPP_LOGGER: ULONGLONG = 0x5750_5054_5241_4300;

/// 0: not read yet; 0xFF: tracing off; otherwise the level.
static WPP_LEVEL: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);

unsafe fn wpp_level() -> u8 {
    use core::sync::atomic::Ordering;
    let v = WPP_LEVEL.load(Ordering::Relaxed);
    if v != 0 {
        return if v == 0xFF { 0 } else { v as u8 };
    }
    let lvl = match ntkrnl_sys::bootopt::boot_option_u32(b"WPPTRACE", 3) {
        Some(n) if n != 0 => n.min(0xFE),
        _ => 0xFF,
    };
    WPP_LEVEL.store(lvl, Ordering::Relaxed);
    if lvl == 0xFF { 0 } else { lvl as u8 }
}

/// WPP_TRACE_CONTROL_BLOCK (km-init.h): what a callback registration points at. Its first
/// member is the callback; providers registered together are chained through Next.
#[repr(C)]
struct WppControlBlock {
    Callback: Option<unsafe extern "system" fn(u8, PVOID, ULONG, PVOID, PVOID, *mut ULONG) -> NTSTATUS>,
    ControlGuid: *const GUID,
    Next: *mut WppControlBlock,
}

unsafe fn wpp_enable(first: *mut WppControlBlock, level: u8) {
    // One registration covers the whole chain: only the first block carries the callback,
    // and it finds each provider by the GUID in the enable request.
    let Some(callback) = (*first).Callback else { return };
    let mut cb = first;
    let mut n = 0;
    while !cb.is_null() && n < 16 {
        let guid = (*cb).ControlGuid;
        if guid.is_null() {
            break;
        }
        let mut wnode: WNODE_HEADER = core::mem::zeroed();
        wnode.BufferSize = core::mem::size_of::<WNODE_HEADER>() as ULONG;
        wnode.HistoricalContext = WPP_LOGGER | level as ULONGLONG;
        wnode.Guid = *guid;
        wnode.Flags = WNODE_FLAG_TRACED_GUID;
        let mut size: ULONG = 0;
        let status = callback(IRP_MN_ENABLE_EVENTS, guid as PVOID, wnode.BufferSize,
            &mut wnode as *mut WNODE_HEADER as PVOID, first as PVOID, &mut size);
        let g = &*guid;
        ntkrnl_sys::driver_ddk::DbgPrint(b"[WPP] provider {%08lx-%04x-%04x} enabled at level %u: %x\n\0".as_ptr() as *const i8,
            g.Data1, g.Data2 as u32, g.Data3 as u32, level as u32, status);
        cb = (*cb).Next;
        n += 1;
    }
}

/// The trace records of the enabled providers. Called from the WmiTraceMessage trampoline
/// (variadic_shims.c) with the argument pairs pulled.
#[no_mangle]
pub unsafe extern "C" fn WmipTraceMessagePairs(
    LoggerHandle: ULONGLONG,
    _MessageFlags: ULONG,
    MessageGuid: *const GUID,
    MessageNumber: USHORT,
    Data: *const *const u8,
    Lengths: *const usize,
    Count: ULONG,
) -> NTSTATUS {
    if LoggerHandle & !0xFF != WPP_LOGGER & !0xFF {
        return STATUS_INVALID_HANDLE;
    }
    let mut line = [0u8; 256];
    let mut n = 0usize;
    let mut put = |b: &[u8], n: &mut usize| {
        for &c in b {
            if *n + 1 < line.len() {
                line[*n] = c;
                *n += 1;
            }
        }
    };
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for i in 0..Count as usize {
        let p = *Data.add(i);
        let len = *Lengths.add(i);
        if p.is_null() {
            continue;
        }
        let bytes = core::slice::from_raw_parts(p, len.min(200));
        put(b" ", &mut n);
        // A NUL-terminated printable string: as text.
        let text = len > 1 && bytes.last() == Some(&0)
            && bytes[..bytes.len() - 1].iter().all(|&c| c == b'\t' || (0x20..0x7F).contains(&c) || c == b'\n' || c == b'\r');
        if text {
            for &c in &bytes[..bytes.len() - 1] {
                if c != b'\n' && c != b'\r' {
                    put(&[c], &mut n);
                }
            }
        } else if len <= 8 {
            // A number: little-endian, as hex.
            let mut v: u64 = 0;
            for (k, &c) in bytes.iter().enumerate() {
                v |= (c as u64) << (8 * k);
            }
            put(b"0x", &mut n);
            let mut started = false;
            for k in (0..16).rev() {
                let d = ((v >> (4 * k)) & 0xF) as usize;
                if d != 0 || started || k == 0 {
                    started = true;
                    put(&[HEX[d]], &mut n);
                }
            }
        } else {
            put(b"<", &mut n);
            for &c in bytes.iter().take(16) {
                put(&[HEX[(c >> 4) as usize], HEX[(c & 0xF) as usize]], &mut n);
            }
            put(b">", &mut n);
        }
    }
    line[n] = 0;
    let g = if MessageGuid.is_null() { 0 } else { (*MessageGuid).Data1 };
    ntkrnl_sys::driver_ddk::DbgPrint(b"[WPP] %08lx#%u:%s\n\0".as_ptr() as *const i8, g, MessageNumber as u32, line.as_ptr());
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
    // Only the enable settings of our own debug-port logger (see wpp_enable): the level is
    // carried in the logger handle's low byte, and every flag is on.
    let wnode = Buffer as *const WNODE_HEADER;
    if wnode.is_null() || (*wnode).HistoricalContext & !0xFF != WPP_LOGGER & !0xFF || TraceInformation.is_null() {
        return STATUS_NOT_IMPLEMENTED;
    }
    let level = ((*wnode).HistoricalContext & 0xFF) as u8;
    match TraceInformationClass {
        TRACE_ENABLE_FLAGS_CLASS if TraceInformationLength >= 4 => {
            *(TraceInformation as *mut ULONG) = 0xFFFF_FFFF;
        }
        TRACE_ENABLE_LEVEL_CLASS if TraceInformationLength >= 1 => {
            *(TraceInformation as *mut u8) = level;
        }
        _ => return STATUS_NOT_IMPLEMENTED,
    }
    if !RequiredLength.is_null() {
        core::ptr::write_volatile(RequiredLength, if TraceInformationClass == TRACE_ENABLE_FLAGS_CLASS { 4 } else { 1 });
    }
    STATUS_SUCCESS
}

// WmiTraceMessage / WmiTraceMessageVa are C (variadic_shims.c): they pull the variadic
// argument pairs and call WmipTraceMessagePairs above.

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
