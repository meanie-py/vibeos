use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{IoGetCurrentIrpStackLocation, IoSkipCurrentIrpStackLocation, IoWMIDeviceObjectToProviderId, IoWMIWriteEvent, KeQueryTimeIncrement, LARGE_INTEGER, LPGUID, PDEVICE_OBJECT, PIO_STACK_LOCATION, PIRP, PUNICODE_STRING, RtlInitUnicodeString, RtlTimeToTimeFields, TIME_FIELDS};
use core::ffi::c_void;
use core::ptr;

extern "C" {
    fn PAGED_CODE();
    fn KeQuerySystemTime(CurrentTime: *mut LARGE_INTEGER);
    fn ClassCompleteRequest(DeviceObject: PDEVICE_OBJECT, Irp: PIRP, PriorityBoost: CCHAR);
    fn ClassReleaseRemoveLock(DeviceObject: PDEVICE_OBJECT, Irp: PIRP);
    fn FREE_POOL(Ptr: *mut core::ffi::c_void);
    fn ClassAcquireRemoveLock(DeviceObject: *mut DEVICE_OBJECT, Irp: *mut IRP) -> u32;
    fn TracePrint(level: u32, flags: u32, fmt: *const i8, ...);
    fn _Analysis_assume_(condition: i32);
    fn NT_ASSERT(condition: i32);
}

// ---- file-scope preamble (translated) ----
pub const TIME_STRING_LENGTH: usize = 25;

pub const MSStorageDriver_ClassErrorLogGuid: GUID = GUID {
    Data1: 0xD5A9A51Eu32,
    Data2: 0x03F9u16,
    Data3: 0x404du16,
    Data4: [0x97, 0x22, 0x15, 0xF9, 0x0E, 0xB0, 0x70, 0x38],
};

pub const MSStorageDriver_ClassErrorLogGuid_Index: u32 = 0;

pub const MOFRESOURCENAME: &[u16] = &[
    0x004D, 0x006F, 0x0066, 0x0052, 0x0065, 0x0073, 0x006F, 0x0075, 0x0072, 0x0063, 0x0065,
    0x004E, 0x0061, 0x006D, 0x0065, 0x0000,
];

pub static wmiClassGuids: [GUIDREGINFO; 1] = [
    GUIDREGINFO {
        Guid: MSStorageDriver_ClassErrorLogGuid,
        Flags: 1,
        InstanceCount: 0,
    },
];

pub const NUM_CLASS_WMI_GUIDS: usize = wmiClassGuids.len();

const NonPagedPoolNx: u32 = 512; // 0x200
const CLASS_TAG_WMI: u32 = 0x776D6943; // guess, verify as 'CImw'?
const WNODE_FLAG_SINGLE_INSTANCE: u32 = 0x1;
const WNODE_FLAG_EVENT_ITEM: u32 = 0x100;
const WNODE_FLAG_STATIC_INSTANCE_NAMES: u32 = 0x200;

const MSStorageDriver_ClassErrorLogGuid_Index: u32 = 0; // TODO: actual value
const NUM_ERROR_LOG_ENTRIES: usize = 10; // TODO: actual value
const MSStorageDriver_ClassErrorLog_SIZE: u32 = 0x1000; // placeholder
const NUM_CLASS_WMI_GUIDS: u32 = 1; // matches file-level array size
const STATUS_WMI_INSTANCE_NOT_FOUND: NTSTATUS = 0xC0000299; // actual value
const STATUS_WMI_GUID_NOT_FOUND: NTSTATUS = 0xC0000298; // actual value

// External declarations for unknown functions
#[no_mangle]
pub unsafe extern "C" fn ClassFindGuid(
    GuidList: *mut GUIDREGINFO,
    GuidCount: u32,
    Guid: *const GUID,
    GuidIndex: *mut u32,
) -> u8 {
    PAGED_CODE();

    let mut i: u32 = 0;
    while i < GuidCount {
        let current_guid: *const GUID = core::ptr::addr_of!((*GuidList.add(i as usize)).Guid);
        if IsEqualGUID(Guid, current_guid) {
            *GuidIndex = i;
            return 1u8;
        }
        i += 1;
    }

    return 0u8;
}
#[no_mangle]
pub unsafe extern "C" fn ClassFindInternalGuid(
    Guid: *const GUID,
    GuidIndex: *mut u32,
) -> i32 {
    // PAGED_CODE() – no‑op, omitted

    for i in 0..wmiClassGuids.len() {
        let element = wmiClassGuids.as_ptr().add(i);
        let guid_ptr: *const GUID = core::ptr::addr_of!((*element).Guid);
        if IsEqualGUID(Guid, guid_ptr) {
            *GuidIndex = i as u32;
            return 1;
        }
    }
    return 0;
}

#[no_mangle]
pub unsafe extern "system" fn ClassSystemControl(
    DeviceObject: *mut DEVICE_OBJECT,
    Irp: *mut IRP,
) -> NTSTATUS {
    // Local constants (typical values)
    const IRP_MN_QUERY_ALL_DATA: u8 = 0x00;
    const IRP_MN_QUERY_SINGLE_INSTANCE: u8 = 0x01;
    const IRP_MN_CHANGE_SINGLE_INSTANCE: u8 = 0x02;
    const IRP_MN_CHANGE_SINGLE_ITEM: u8 = 0x03;
    const IRP_MN_ENABLE_EVENTS: u8 = 0x04;
    const IRP_MN_DISABLE_EVENTS: u8 = 0x05;
    const IRP_MN_ENABLE_COLLECTION: u8 = 0x06;
    const IRP_MN_DISABLE_COLLECTION: u8 = 0x07;
    const IRP_MN_EXECUTE_METHOD: u8 = 0x08;
    const IRP_MN_REGINFO: u8 = 0x0B;

    const STATUS_DEVICE_DOES_NOT_EXIST: i32 = 0xC000004E;
    const STATUS_WMI_GUID_NOT_FOUND: i32 = 0xC0000290;
    const STATUS_WMI_INSTANCE_NOT_FOUND: i32 = 0xC0000291;
    const STATUS_INVALID_DEVICE_REQUEST: i32 = 0xC0000010;
    const STATUS_INVALID_BUFFER_SIZE: i32 = 0xC0000006;
    const WNODE_FLAG_STATIC_INSTANCE_NAMES: u32 = 0x1;
    const WMIREG_FLAG_INSTANCE_PDO: u32 = 0x1;
    const WMIREG_FLAG_INSTANCE_LIST: u32 = 0x2;
    const EventGeneration: u32 = 0;
    const DataBlockCollection: u32 = 1;
    const TRACE_LEVEL_WARNING: u32 = 2;
    const TRACE_FLAG_WMI: u32 = 0x40;
    const TRACE_LEVEL_VERBOSE: u32 = 5;

    let commonExtension = (*DeviceObject).DeviceExtension as *mut COMMON_DEVICE_EXTENSION;
    let irpStack = IoGetCurrentIrpStackLocation(Irp);
    let isRemoved: u32;
    let bufferSize: u32;
    let buffer: *mut u8;
    let mut status: NTSTATUS;
    let minorFunction: u8;
    let mut guidIndex: u32 = u32::MAX;
    let mut classWmiInfo: *mut CLASS_WMI_INFO;
    let mut isInternalGuid: u8 = 0;

    PAGED_CODE();

    // Make sure device has not been removed
    isRemoved = ClassAcquireRemoveLock(DeviceObject, Irp);
    if isRemoved != 0 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_DEVICE_DOES_NOT_EXIST;
        ClassReleaseRemoveLock(DeviceObject, Irp);
        ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
        return STATUS_DEVICE_DOES_NOT_EXIST;
    }

    // If the irp is not a WMI irp or it is not targetted at this device
    // or this device has not registered with WMI then just forward it on.
    minorFunction = (*irpStack).MinorFunction as u8;
    let provider_id = (*irpStack).Parameters.WMI.ProviderId;
    if (minorFunction > IRP_MN_EXECUTE_METHOD)
        || (provider_id != (DeviceObject as u64))
        || ((minorFunction != IRP_MN_REGINFO) && ((*commonExtension).GuidCount == 0))
    {
        IoSkipCurrentIrpStackLocation(Irp);
        ClassReleaseRemoveLock(DeviceObject, Irp);
        return IoCallDriver((*commonExtension).LowerDeviceObject, Irp);
    }

    buffer = (*irpStack).Parameters.WMI.Buffer as *mut u8;
    bufferSize = (*irpStack).Parameters.WMI.BufferSize;

    if minorFunction != IRP_MN_REGINFO {
        // Determine if the guid is supported
        let guid_ptr = (*irpStack).Parameters.WMI.DataPath as *mut GUID;
        if !(*commonExtension).GuidRegInfo.is_null()
            && ClassFindGuid(
                (*commonExtension).GuidRegInfo,
                (*commonExtension).GuidCount,
                guid_ptr,
                &mut guidIndex,
            ) != 0
        {
            isInternalGuid = 0;
            status = 0; // STATUS_SUCCESS
        } else if ClassFindInternalGuid(guid_ptr, &mut guidIndex) != 0 {
            isInternalGuid = 1;
            status = 0;
        } else {
            status = STATUS_WMI_GUID_NOT_FOUND;
            TracePrint(
                TRACE_LEVEL_WARNING,
                TRACE_FLAG_WMI,
                b"WMI GUID not found!\0".as_ptr() as *const i8,
            );
        }

        TracePrint(
            TRACE_LEVEL_VERBOSE,
            TRACE_FLAG_WMI,
            b"WMI Find Guid = %x, isInternalGuid = %x\0".as_ptr() as *const i8,
            status,
            isInternalGuid,
        );

        if status >= 0
            && (minorFunction == IRP_MN_QUERY_SINGLE_INSTANCE
                || minorFunction == IRP_MN_CHANGE_SINGLE_INSTANCE
                || minorFunction == IRP_MN_CHANGE_SINGLE_ITEM
                || minorFunction == IRP_MN_EXECUTE_METHOD)
        {
            let header = buffer as *mut WNODE_HEADER;
            if ((*header).Flags & WNODE_FLAG_STATIC_INSTANCE_NAMES) != 0 {
                let single = buffer as *mut WNODE_SINGLE_INSTANCE;
                if (*single).InstanceIndex != 0 {
                    status = STATUS_WMI_INSTANCE_NOT_FOUND;
                }
            } else {
                status = STATUS_WMI_INSTANCE_NOT_FOUND;
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_WMI,
                    b"WMI Instance not found!\0".as_ptr() as *const i8,
                );
            }
        }

        if status < 0 {
            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
            return status;
        }
    }

    let driverExtension = (*commonExtension).DriverExtension;

    classWmiInfo = if (*commonExtension).IsFdo != 0 {
        &mut (*(*driverExtension).InitData.FdoData.ClassWmiInfo) as *mut CLASS_WMI_INFO
        // Actually the C takes address of the field; it's a struct, we can get ptr directly
        &mut (*(*driverExtension).InitData.FdoData.ClassWmiInfo)
    } else {
        &mut (*(*driverExtension).InitData.PdoData.ClassWmiInfo)
    };

    // Irp_MN_REGINFO
    let retSize: u32;
    match minorFunction {
        IRP_MN_REGINFO => {
            let mut guidList: *mut GUIDREGINFO;
            let mut guidCount: u32;
            let mut wmiRegInfo: *mut WMIREGINFO;
            let mut wmiRegGuid: *mut WMIREGGUIDW;
            let mut regPath: *mut _UNICODE_STRING;
            let mut stringPtr: *mut u16;
            let mut bufferNeeded: u32;
            let mut nameFlags: u32 = 0;
            let mut name: _UNICODE_STRING = _UNICODE_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut(),
            };
            let mut mofName: _UNICODE_STRING = _UNICODE_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut(),
            };

            let ClassQueryWmiRegInfoEx: *mut PCLASS_QUERY_WMI_REGINFO_EX = if (*commonExtension).IsFdo != 0 {
                &mut (*driverExtension).ClassFdoQueryWmiRegInfoEx as *mut PCLASS_QUERY_WMI_REGINFO_EX
            } else {
                &mut (*driverExtension).ClassPdoQueryWmiRegInfoEx as *mut PCLASS_QUERY_WMI_REGINFO_EX
            };

            let classWmiInfo_guidreginfo = (*classWmiInfo).GuidRegInfo;
            let classWmiInfo_classquerywmireginfo = (*classWmiInfo).ClassQueryWmiRegInfo;

            if !classWmiInfo_guidreginfo.is_null()
                && !classWmiInfo_classquerywmireginfo.is_null()
                && ClassQueryWmiRegInfoEx.is_null()
            // Actually the condition is (ClassQueryWmiRegInfoEx == NULL)
            {
                status = ((*classWmiInfo).ClassQueryWmiRegInfo)(
                    DeviceObject,
                    &mut nameFlags,
                    &mut name,
                );
                let mof_resource_name: &[u16] = &[
                    0x004D, 0x006F, 0x0066, 0x0052, 0x0065, 0x0073, 0x006F, 0x0075,
                    0x0072, 0x0063, 0x0065, 0x004E, 0x0061, 0x006D, 0x0065, 0x0000,
                ];
                RtlInitUnicodeString(&mut mofName, mof_resource_name.as_ptr());
            } else if !classWmiInfo_guidreginfo.is_null() && !(*ClassQueryWmiRegInfoEx).is_null() {
                status = (*ClassQueryWmiRegInfoEx)(
                    DeviceObject,
                    &mut nameFlags,
                    &mut name,
                    &mut mofName,
                );
            } else {
                RtlInitUnicodeString(&mut mofName, &0u16 as *const u16); // L""
                nameFlags = WMIREG_FLAG_INSTANCE_PDO;
                status = 0;
            }

            if status >= 0
                && (nameFlags & WMIREG_FLAG_INSTANCE_PDO) == 0
                && name.Buffer.is_null()
            {
                status = STATUS_INVALID_DEVICE_REQUEST;
                TracePrint(
                    TRACE_LEVEL_WARNING,
                    TRACE_FLAG_WMI,
                    b"Invalid Device Request!\0".as_ptr() as *const i8,
                );
            }

            if status >= 0 {
                guidList = classWmiInfo_guidreginfo;
                guidCount = if classWmiInfo_guidreginfo.is_null() {
                    0
                } else {
                    (*classWmiInfo).GuidCount
                } + (wmiClassGuids.len() as u32); // NUM_CLASS_WMI_GUIDS

                let nameOffset = core::mem::size_of::<WMIREGINFO>() as u32
                    + guidCount * core::mem::size_of::<WMIREGGUIDW>() as u32;

                let nameInfo: u64;
                let nameSize: u32;
                if (nameFlags & WMIREG_FLAG_INSTANCE_PDO) != 0 {
                    nameSize = 0;
                    nameInfo = if (*commonExtension).IsFdo != 0 {
                        (*(commonExtension as *mut FUNCTIONAL_DEVICE_EXTENSION)).LowerPdo as u64
                    } else {
                        DeviceObject as u64
                    };
                } else {
                    nameFlags |= WMIREG_FLAG_INSTANCE_LIST;
                    nameSize = name.Length as u32 + core::mem::size_of::<u16>() as u32;
                    nameInfo = nameOffset as u64;
                }

                let mofResourceOffset = nameOffset + nameSize;

                regPath = &mut (*driverExtension).RegistryPath as *mut _UNICODE_STRING;

                let registryPathOffset = mofResourceOffset + mofName.Length as u32 + core::mem::size_of::<u16>() as u32;

                bufferNeeded = registryPathOffset + (*regPath).Length as u32;
                bufferNeeded += core::mem::size_of::<u16>() as u32;

                if bufferNeeded <= bufferSize {
                    retSize = bufferNeeded;

                    (*commonExtension).GuidCount = guidCount;
                    (*commonExtension).GuidRegInfo = guidList;

                    wmiRegInfo = buffer as *mut WMIREGINFO;
                    (*wmiRegInfo).BufferSize = bufferNeeded;
                    (*wmiRegInfo).NextWmiRegInfo = 0;
                    (*wmiRegInfo).MofResourceName = mofResourceOffset;
                    (*wmiRegInfo).RegistryPath = registryPathOffset;
                    (*wmiRegInfo).GuidCount = guidCount;

                    // First group of GUIDs
                    for i in 0..(*classWmiInfo).GuidCount {
                        wmiRegGuid = &mut (*wmiRegInfo).WmiRegGuid[i as usize] as *mut WMIREGGUIDW;
                        (*wmiRegGuid).Guid = (*guidList.offset(i as isize)).Guid;
                        (*wmiRegGuid).Flags = (*guidList.offset(i as isize)).Flags | nameFlags;
                        (*wmiRegGuid).InstanceInfo = nameInfo;
                        (*wmiRegGuid).InstanceCount = 1;
                    }
                    // Class-wide GUIDs
                    for i in 0..(wmiClassGuids.len() as u32) {
                        wmiRegGuid = &mut (*wmiRegInfo).WmiRegGuid[(i + (*classWmiInfo).GuidCount) as usize]
                            as *mut WMIREGGUIDW;
                        (*wmiRegGuid).Guid = wmiClassGuids[i as usize].Guid;
                        (*wmiRegGuid).Flags = wmiClassGuids[i as usize].Flags | nameFlags;
                        (*wmiRegGuid).InstanceInfo = nameInfo;
                        (*wmiRegGuid).InstanceCount = 1;
                    }

                    // Instance list (if name supplied)
                    if (nameFlags & WMIREG_FLAG_INSTANCE_LIST) != 0 {
                        bufferNeeded = nameOffset + core::mem::size_of::<u16>() as u32;
                        bufferNeeded += name.Length as u32;

                        if bufferSize >= bufferNeeded {
                            stringPtr = (buffer as *mut u8).offset(nameOffset as isize) as *mut u16;
                            *stringPtr = name.Length;
                            stringPtr = stringPtr.offset(1);
                            core::ptr::copy_nonoverlapping(name.Buffer, stringPtr, (name.Length / 2) as usize);
                        } else {
                            NT_ASSERT(if bufferSize >= bufferNeeded { 1 } else { 0 });
                            status = STATUS_INVALID_BUFFER_SIZE;
                        }
                    }

                    // MOF resource
                    bufferNeeded = mofResourceOffset + core::mem::size_of::<u16>() as u32;
                    bufferNeeded += mofName.Length as u32;

                    if bufferSize >= bufferNeeded {
                        stringPtr = (buffer as *mut u8).offset(mofResourceOffset as isize) as *mut u16;
                        *stringPtr = mofName.Length;
                        stringPtr = stringPtr.offset(1);
                        core::ptr::copy_nonoverlapping(mofName.Buffer, stringPtr, (mofName.Length / 2) as usize);
                    } else {
                        NT_ASSERT(if bufferSize >= bufferNeeded { 1 } else { 0 });
                        status = STATUS_INVALID_BUFFER_SIZE;
                    }

                    // Registry path
                    bufferNeeded = registryPathOffset + core::mem::size_of::<u16>() as u32;
                    bufferNeeded += (*regPath).Length as u32;

                    if bufferSize >= bufferNeeded {
                        stringPtr = (buffer as *mut u8).offset(registryPathOffset as isize) as *mut u16;
                        *stringPtr = (*regPath).Length;
                        stringPtr = stringPtr.offset(1);
                        core::ptr::copy_nonoverlapping(
                            (*regPath).Buffer,
                            stringPtr,
                            ((*regPath).Length / 2) as usize,
                        );
                    } else {
                        NT_ASSERT(if bufferSize >= bufferNeeded { 1 } else { 0 });
                        TracePrint(
                            TRACE_LEVEL_WARNING,
                            TRACE_FLAG_WMI,
                            b"Invalid Buffer Size!\0".as_ptr() as *const i8,
                        );
                        status = STATUS_INVALID_BUFFER_SIZE;
                    }
                } else {
                    *(buffer as *mut u32) = bufferNeeded;
                    retSize = core::mem::size_of::<u32>() as u32;
                }
            } else {
                retSize = 0;
            }

            FREE_POOL(name.Buffer as *mut core::ffi::c_void);

            (*Irp).IoStatus.__bindgen_anon_1.Status = status;
            (*Irp).IoStatus.Information = retSize as usize;
            ClassReleaseRemoveLock(DeviceObject, Irp);
            ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
            return status;
        }

        IRP_MN_QUERY_ALL_DATA => {
            let wnode = buffer as *mut WNODE_ALL_DATA;
            let bufferAvail = if bufferSize < core::mem::size_of::<WNODE_ALL_DATA>() as u32 {
                0
            } else {
                bufferSize - core::mem::size_of::<WNODE_ALL_DATA>() as u32
            };
            (*wnode).DataBlockOffset = core::mem::size_of::<WNODE_ALL_DATA>() as u32;

            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassQueryInternalDataBlock(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    bufferAvail,
                    buffer.offset(core::mem::size_of::<WNODE_ALL_DATA>() as isize),
                );
            } else {
                status = ((*classWmiInfo).ClassQueryWmiDataBlock)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    bufferAvail,
                    buffer.offset(core::mem::size_of::<WNODE_ALL_DATA>() as isize),
                );
            }
        }

        IRP_MN_QUERY_SINGLE_INSTANCE => {
            let wnode = buffer as *mut WNODE_SINGLE_INSTANCE;
            let dataBlockOffset = (*wnode).DataBlockOffset;

            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassQueryInternalDataBlock(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    bufferSize - dataBlockOffset,
                    (wnode as *mut u8).offset(dataBlockOffset as isize),
                );
            } else {
                status = ((*classWmiInfo).ClassQueryWmiDataBlock)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    bufferSize - dataBlockOffset,
                    (wnode as *mut u8).offset(dataBlockOffset as isize),
                );
            }
        }

        IRP_MN_CHANGE_SINGLE_INSTANCE => {
            let wnode = buffer as *mut WNODE_SINGLE_INSTANCE;
            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT,
                );
            } else {
                NT_ASSERT(if guidIndex != u32::MAX { 1 } else { 0 });
                status = ((*classWmiInfo).ClassSetWmiDataBlock)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    (*wnode).SizeDataBlock,
                    (wnode as *mut u8).offset((*wnode).DataBlockOffset as isize),
                );
            }
        }

        IRP_MN_CHANGE_SINGLE_ITEM => {
            let wnode = buffer as *mut WNODE_SINGLE_ITEM;
            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT,
                );
            } else {
                NT_ASSERT(if guidIndex != u32::MAX { 1 } else { 0 });
                status = ((*classWmiInfo).ClassSetWmiDataItem)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    (*wnode).ItemId,
                    (*wnode).SizeDataItem,
                    (wnode as *mut u8).offset((*wnode).DataBlockOffset as isize),
                );
            }
        }

        IRP_MN_EXECUTE_METHOD => {
            let wnode = buffer as *mut WNODE_METHOD_ITEM;
            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT,
                );
            } else {
                NT_ASSERT(if guidIndex != u32::MAX { 1 } else { 0 });
                status = ((*classWmiInfo).ClassExecuteWmiMethod)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    (*wnode).MethodId,
                    (*wnode).SizeDataBlock,
                    bufferSize - (*wnode).DataBlockOffset,
                    buffer.offset((*wnode).DataBlockOffset as isize),
                );
            }
        }

        IRP_MN_ENABLE_EVENTS => {
            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT,
                );
            } else {
                NT_ASSERT(if guidIndex != u32::MAX { 1 } else { 0 });
                status = ((*classWmiInfo).ClassWmiFunctionControl)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    EventGeneration,
                    1,
                );
            }
        }

        IRP_MN_DISABLE_EVENTS => {
            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT,
                );
            } else {
                NT_ASSERT(if guidIndex != u32::MAX { 1 } else { 0 });
                status = ((*classWmiInfo).ClassWmiFunctionControl)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    EventGeneration,
                    0,
                );
            }
        }

        IRP_MN_ENABLE_COLLECTION => {
            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT,
                );
            } else {
                NT_ASSERT(if guidIndex != u32::MAX { 1 } else { 0 });
                status = ((*classWmiInfo).ClassWmiFunctionControl)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    DataBlockCollection,
                    1,
                );
            }
        }

        IRP_MN_DISABLE_COLLECTION => {
            _Analysis_assume_(isInternalGuid as i32);
            if isInternalGuid != 0 {
                status = ClassWmiCompleteRequest(
                    DeviceObject,
                    Irp,
                    STATUS_WMI_GUID_NOT_FOUND,
                    0,
                    IO_NO_INCREMENT,
                );
            } else {
                NT_ASSERT(if guidIndex != u32::MAX { 1 } else { 0 });
                status = ((*classWmiInfo).ClassWmiFunctionControl)(
                    DeviceObject,
                    Irp,
                    guidIndex,
                    DataBlockCollection,
                    0,
                );
            }
        }

        _ => {
            status = STATUS_INVALID_DEVICE_REQUEST;
        }
    }

    status
}
#[no_mangle]
pub unsafe extern "C" fn ClassQueryInternalDataBlock(
    DeviceObject: *mut PDEVICE_OBJECT,
    Irp: *mut PIRP,
    GuidIndex: u32,
    BufferAvail: u32,
    Buffer: *mut u8,
) -> NTSTATUS {
    let mut status: NTSTATUS;
    let mut size_needed: u32 = 0;
    let fdo_ext = (*DeviceObject).DeviceExtension as *mut FUNCTIONAL_DEVICE_EXTENSION;

    if GuidIndex == MSStorageDriver_ClassErrorLogGuid_Index {
        size_needed = MSStorageDriver_ClassErrorLog_SIZE;
        if BufferAvail >= size_needed {
            let error_log = Buffer as *mut MSStorageDriver_ClassErrorLog;
            let fdo_data = (*fdo_ext).PrivateFdoData;
            (*error_log).numEntries = NUM_ERROR_LOG_ENTRIES as u32;

            for i in 0..NUM_ERROR_LOG_ENTRIES {
                let fdo_log_entry = &mut (*fdo_data).ErrorLogs[i];
                let fdo_srb_block = &mut (*fdo_log_entry).Srb;
                let fdo_sense_data = &mut (*fdo_log_entry).SenseData;
                let log_entry = &mut (*error_log).logEntries[i];
                let srb_block = &mut (*log_entry).srb;
                let sense_data = &mut (*log_entry).senseData;

                (*log_entry).tickCount = (*fdo_log_entry).TickCount.QuadPart;
                (*log_entry).portNumber = (*fdo_log_entry).PortNumber;
                (*log_entry).errorPaging = if (*fdo_log_entry).ErrorPaging == 0 { 0 } else { 1 };
                (*log_entry).errorRetried = if (*fdo_log_entry).ErrorRetried == 0 { 0 } else { 1 };
                (*log_entry).errorUnhandled = if (*fdo_log_entry).ErrorUnhandled == 0 { 0 } else { 1 };
                (*log_entry).errorReserved = (*fdo_log_entry).ErrorReserved;

                core::ptr::copy(
                    addr_of!((*fdo_log_entry).Reserved) as *const u8,
                    addr_of_mut!((*log_entry).reserved) as *mut u8,
                    core::mem::size_of::<typeof((*log_entry).reserved)>(),
                );

                ConvertTickToDateTime(
                    (*fdo_log_entry).TickCount,
                    (*log_entry).eventTime.as_mut_ptr(),
                );

                (*srb_block).length = (*fdo_srb_block).Length;
                (*srb_block).function = (*fdo_srb_block).Function;
                (*srb_block).srbStatus = (*fdo_srb_block).SrbStatus;
                (*srb_block).scsiStatus = (*fdo_srb_block).ScsiStatus;
                (*srb_block).pathID = (*fdo_srb_block).PathId;
                (*srb_block).targetID = (*fdo_srb_block).TargetId;
                (*srb_block).lun = (*fdo_srb_block).Lun;
                (*srb_block).queueTag = (*fdo_srb_block).QueueTag;
                (*srb_block).queueAction = (*fdo_srb_block).QueueAction;
                (*srb_block).cdbLength = (*fdo_srb_block).CdbLength;
                (*srb_block).senseInfoBufferLength = (*fdo_srb_block).SenseInfoBufferLength;
                (*srb_block).srbFlags = (*fdo_srb_block).SrbFlags;
                (*srb_block).dataTransferLength = (*fdo_srb_block).DataTransferLength;
                (*srb_block).timeOutValue = (*fdo_srb_block).TimeOutValue;
                (*srb_block).dataBuffer = (*fdo_srb_block).DataBuffer as u64;
                (*srb_block).senseInfoBuffer = (*fdo_srb_block).SenseInfoBuffer as u64;
                (*srb_block).nextSRB = (*fdo_srb_block).NextSrb as u64;
                (*srb_block).originalRequest = (*fdo_srb_block).OriginalRequest as u64;
                (*srb_block).srbExtension = (*fdo_srb_block).SrbExtension as u64;
                (*srb_block).internalStatus = (*fdo_srb_block).InternalStatus;

                if cfg!(target_pointer_width = "64") {
                    (*srb_block).reserved = (*fdo_srb_block).Reserved;
                } else {
                    (*srb_block).reserved = 0;
                }

                core::ptr::copy(
                    addr_of!((*fdo_srb_block).Cdb) as *const u8,
                    addr_of_mut!((*srb_block).cdb) as *mut u8,
                    core::mem::size_of::<typeof((*srb_block).cdb)>(),
                );

                (*sense_data).errorCode = (*fdo_sense_data).ErrorCode;
                (*sense_data).valid = if (*fdo_sense_data).Valid == 0 { 0 } else { 1 };
                (*sense_data).segmentNumber = (*fdo_sense_data).SegmentNumber;
                (*sense_data).senseKey = (*fdo_sense_data).SenseKey;
                (*sense_data).reserved = if (*fdo_sense_data).Reserved == 0 { 0 } else { 1 };
                (*sense_data).incorrectLength = if (*fdo_sense_data).IncorrectLength == 0 { 0 } else { 1 };
                (*sense_data).endOfMedia = if (*fdo_sense_data).EndOfMedia == 0 { 0 } else { 1 };
                (*sense_data).fileMark = if (*fdo_sense_data).FileMark == 0 { 0 } else { 1 };

                core::ptr::copy(
                    addr_of!((*fdo_sense_data).Information) as *const u8,
                    addr_of_mut!((*sense_data).information) as *mut u8,
                    core::mem::size_of::<typeof((*sense_data).information)>(),
                );

                (*sense_data).additionalSenseLength = (*fdo_sense_data).AdditionalSenseLength;

                core::ptr::copy(
                    addr_of!((*fdo_sense_data).CommandSpecificInformation) as *const u8,
                    addr_of_mut!((*sense_data).commandSpecificInformation) as *mut u8,
                    core::mem::size_of::<typeof((*sense_data).commandSpecificInformation)>(),
                );

                (*sense_data).additionalSenseCode = (*fdo_sense_data).AdditionalSenseCode;
                (*sense_data).additionalSenseCodeQualifier = (*fdo_sense_data).AdditionalSenseCodeQualifier;
                (*sense_data).fieldReplaceableUnitCode = (*fdo_sense_data).FieldReplaceableUnitCode;

                core::ptr::copy(
                    addr_of!((*fdo_sense_data).SenseKeySpecific) as *const u8,
                    addr_of_mut!((*sense_data).senseKeySpecific) as *mut u8,
                    core::mem::size_of::<typeof((*sense_data).senseKeySpecific)>(),
                );
            }
            status = STATUS_SUCCESS;
        } else {
            status = STATUS_BUFFER_TOO_SMALL;
        }
    } else if GuidIndex > 0 && GuidIndex < NUM_CLASS_WMI_GUIDS {
        status = STATUS_WMI_INSTANCE_NOT_FOUND;
    } else {
        status = STATUS_WMI_GUID_NOT_FOUND;
    }

    ClassWmiCompleteRequest(
        DeviceObject,
        Irp,
        status,
        size_needed,
        IO_NO_INCREMENT,
    );
    status
}
#[no_mangle]
pub unsafe extern "C" fn ConvertTickToDateTime(
    Tick: LARGE_INTEGER,
    String: *mut u16,
) -> *mut u16 {

    const TIME_STRING_LENGTH: usize = 25;

    // Locals exactly matching C source
    let mut nowTick: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let mut nowTime: LARGE_INTEGER = LARGE_INTEGER { QuadPart: 0 };
    let time: LARGE_INTEGER;
    let maxInc: u32;
    let mut timeFields: TIME_FIELDS = core::mem::zeroed();
    let mut outDateTime: [u16; TIME_STRING_LENGTH + 1] = [0u16; TIME_STRING_LENGTH + 1]; // WCHAR[26]

    // Translate the tick count to a system time
    KeQueryTickCount(&mut nowTick.QuadPart);
    maxInc = KeQueryTimeIncrement();

    // KeQuerySystemTime is a gap macro/function; we declare it as extern.
    KeQuerySystemTime(&mut nowTime);

    time = LARGE_INTEGER {
        QuadPart: nowTime.QuadPart - ((nowTick.QuadPart - Tick.QuadPart) * maxInc as i64),
    };

    RtlTimeToTimeFields(&time, &mut timeFields);

    // Expand RtlStringCbPrintfW(outDateTime, sizeof(outDateTime), 
    //   L"%04d%02d%02d%02d%02d%02d.%03d***+000", ...)
    // This is a verbatim rewrite of the format string, since variadic FFI is not
    // available.  The effect is identical to the C call.
    {
        struct FixedFields {
            year: u32,
            month: u32,
            day: u32,
            hour: u32,
            minute: u32,
            second: u32,
            millis: u32,
        }
        let f = FixedFields {
            year: timeFields.Year as u32,
            month: timeFields.Month as u32,
            day: timeFields.Day as u32,
            hour: timeFields.Hour as u32,
            minute: timeFields.Minute as u32,
            second: timeFields.Second as u32,
            millis: timeFields.Milliseconds as u32,
        };
        let mut pos = 0usize;

        // Helper closure: write `val` as `digits` wide decimal into outDateTime.
        #[inline(always)]
        fn write_fixed_width(buf: &mut [u16; 26], pos: &mut usize, val: u32, digits: usize) {
            for j in (0..digits).rev() {
                let ten_pow = 10u32.pow(j as u32);
                let digit = (val / ten_pow) % 10;
                buf[*pos] = b'0' as u16 + digit as u16;
                *pos += 1;
            }
        }

        write_fixed_width(&mut outDateTime, &mut pos, f.year, 4);
        write_fixed_width(&mut outDateTime, &mut pos, f.month, 2);
        write_fixed_width(&mut outDateTime, &mut pos, f.day, 2);
        write_fixed_width(&mut outDateTime, &mut pos, f.hour, 2);
        write_fixed_width(&mut outDateTime, &mut pos, f.minute, 2);
        write_fixed_width(&mut outDateTime, &mut pos, f.second, 2);

        outDateTime[pos] = b'.' as u16;
        pos += 1;

        write_fixed_width(&mut outDateTime, &mut pos, f.millis, 3);

        // suffix "***+000"
        const SUFFIX: &[u16] = &[
            b'*' as u16,
            b'*' as u16,
            b'*' as u16,
            b'+' as u16,
            b'0' as u16,
            b'0' as u16,
            b'0' as u16,
            0u16,
        ];
        for &c in SUFFIX {
            outDateTime[pos] = c;
            pos += 1;
        }
    }

    // RtlMoveMemory(String, outDateTime, sizeof(WCHAR) * TIME_STRING_LENGTH)
    ptr::copy_nonoverlapping(outDateTime.as_ptr(), String, TIME_STRING_LENGTH);

    String
}

#[no_mangle]
pub unsafe extern "system" fn ClassWmiCompleteRequest(
    DeviceObject: PDEVICE_OBJECT,
    Irp: PIRP,
    mut Status: NTSTATUS,
    BufferUsed: ULONG,
    PriorityBoost: CCHAR,
) -> NTSTATUS {
    let irpStack: PIO_STACK_LOCATION = IoGetCurrentIrpStackLocation(Irp);
    let buffer: *mut u8;
    let retSize: ULONG;
    let minorFunction: u8 = (*irpStack).MinorFunction;
    buffer = (*irpStack).Parameters.WMI.Buffer as *mut u8;

    match minorFunction {
        0x00 /* IRP_MN_QUERY_ALL_DATA */ => {
            let wnode: *mut WNODE_ALL_DATA;
            let wnodeTooSmall: *mut WNODE_TOO_SMALL;
            let bufferNeeded: ULONG;
            wnode = buffer as *mut WNODE_ALL_DATA;
            bufferNeeded = core::mem::size_of::<WNODE_ALL_DATA>() as ULONG + BufferUsed;

            if NT_SUCCESS(Status) {
                retSize = bufferNeeded;
                (*wnode).WnodeHeader.BufferSize = bufferNeeded;
                KeQuerySystemTime(&mut (*wnode).WnodeHeader.TimeStamp);
                (*wnode).WnodeHeader.Flags |= 0x1; // WNODE_FLAG_FIXED_INSTANCE_SIZE
                (*wnode).FixedInstanceSize = BufferUsed;
                (*wnode).InstanceCount = 1;
            } else if Status == STATUS_BUFFER_TOO_SMALL {
                wnodeTooSmall = wnode as *mut WNODE_TOO_SMALL;
                (*wnodeTooSmall).WnodeHeader.BufferSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                (*wnodeTooSmall).WnodeHeader.Flags = 0x2; // WNODE_FLAG_TOO_SMALL
                (*wnodeTooSmall).SizeNeeded = core::mem::size_of::<WNODE_ALL_DATA>() as ULONG + BufferUsed;
                retSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                Status = STATUS_SUCCESS;
            } else {
                retSize = 0;
            }
        },
        0x01 /* IRP_MN_QUERY_SINGLE_INSTANCE */ => {
            let wnode: *mut WNODE_SINGLE_INSTANCE;
            let wnodeTooSmall: *mut WNODE_TOO_SMALL;
            let bufferNeeded: ULONG;
            wnode = buffer as *mut WNODE_SINGLE_INSTANCE;
            bufferNeeded = (*wnode).DataBlockOffset + BufferUsed;

            if NT_SUCCESS(Status) {
                retSize = bufferNeeded;
                (*wnode).WnodeHeader.BufferSize = bufferNeeded;
                KeQuerySystemTime(&mut (*wnode).WnodeHeader.TimeStamp);
                (*wnode).SizeDataBlock = BufferUsed;
            } else if Status == STATUS_BUFFER_TOO_SMALL {
                wnodeTooSmall = wnode as *mut WNODE_TOO_SMALL;
                (*wnodeTooSmall).WnodeHeader.BufferSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                (*wnodeTooSmall).WnodeHeader.Flags = 0x2; // WNODE_FLAG_TOO_SMALL
                (*wnodeTooSmall).SizeNeeded = bufferNeeded;
                retSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                Status = STATUS_SUCCESS;
            } else {
                retSize = 0;
            }
        },
        0x02 /* IRP_MN_EXECUTE_METHOD */ => {
            let wnode: *mut WNODE_METHOD_ITEM;
            let wnodeTooSmall: *mut WNODE_TOO_SMALL;
            let bufferNeeded: ULONG;
            wnode = buffer as *mut WNODE_METHOD_ITEM;
            bufferNeeded = (*wnode).DataBlockOffset + BufferUsed;

            if NT_SUCCESS(Status) {
                retSize = bufferNeeded;
                (*wnode).WnodeHeader.BufferSize = bufferNeeded;
                KeQuerySystemTime(&mut (*wnode).WnodeHeader.TimeStamp);
                (*wnode).SizeDataBlock = BufferUsed;
            } else if Status == STATUS_BUFFER_TOO_SMALL {
                wnodeTooSmall = wnode as *mut WNODE_TOO_SMALL;
                (*wnodeTooSmall).WnodeHeader.BufferSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                (*wnodeTooSmall).WnodeHeader.Flags = 0x2; // WNODE_FLAG_TOO_SMALL
                (*wnodeTooSmall).SizeNeeded = bufferNeeded;
                retSize = core::mem::size_of::<WNODE_TOO_SMALL>() as ULONG;
                Status = STATUS_SUCCESS;
            } else {
                retSize = 0;
            }
        },
        _ => {
            retSize = 0;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    (*Irp).IoStatus.Information = retSize;
    ClassReleaseRemoveLock(DeviceObject, Irp);
    ClassCompleteRequest(DeviceObject, Irp, PriorityBoost);
    return Status;
}

#[no_mangle]
pub unsafe extern "system" fn ClassWmiFireEvent(
    DeviceObject: *mut ntkrnl_sys::ddk::DEVICE_OBJECT,
    Guid: *mut ntkrnl_sys::ddk::GUID,
    InstanceIndex: u32,
    EventDataSize: u32,
    EventData: *mut core::ffi::c_void,
) -> i32 {
    let size_needed: usize;
    let event: *mut WNODE_SINGLE_INSTANCE;
    let status: i32;

    let mut local_event_data_size = EventDataSize;
    if EventData.is_null() {
        local_event_data_size = 0;
    }

    size_needed = core::mem::size_of::<WNODE_SINGLE_INSTANCE>() + local_event_data_size as usize;

    event = ntkrnl_sys::ex::ExAllocatePoolWithTag(NonPagedPoolNx, size_needed, CLASS_TAG_WMI) as *mut WNODE_SINGLE_INSTANCE;
    if !event.is_null() {
        ntkrnl_sys::rtl::RtlZeroMemory(event as *mut core::ffi::c_void, size_needed);

        // copy Guid into WnodeHeader.Guid
        core::ptr::copy_nonoverlapping(
            Guid as *const ntkrnl_sys::ddk::GUID,
            core::ptr::addr_of_mut!((*event).WnodeHeader.Guid),
            1,
        );

        (*event).WnodeHeader.ProviderId = ntkrnl_sys::ddk::IoWMIDeviceObjectToProviderId(DeviceObject);
        (*event).WnodeHeader.BufferSize = size_needed as u32;
        (*event).WnodeHeader.Flags = WNODE_FLAG_SINGLE_INSTANCE | WNODE_FLAG_EVENT_ITEM | WNODE_FLAG_STATIC_INSTANCE_NAMES;

        KeQuerySystemTime(core::ptr::addr_of_mut!((*event).WnodeHeader.TimeStamp));

        (*event).InstanceIndex = InstanceIndex;
        (*event).SizeDataBlock = local_event_data_size;
        (*event).DataBlockOffset = core::mem::size_of::<WNODE_SINGLE_INSTANCE>() as u32;

        if !EventData.is_null() {
            let dest = (event as *mut u8).add(core::mem::size_of::<WNODE_SINGLE_INSTANCE>());
            core::ptr::copy_nonoverlapping(EventData as *const u8, dest, EventDataSize as usize);
        }

        status = ntkrnl_sys::ddk::IoWMIWriteEvent(event as *mut core::ffi::c_void) as i32;
        if status < 0 {
            FREE_POOL(event as *mut core::ffi::c_void);
        }
    } else {
        status = ntkrnl_sys::base::STATUS_INSUFFICIENT_RESOURCES;
    }

    status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*++
//
// Copyright (C) Microsoft Corporation, 1991 - 1999
//
// Module Name:
//
//     classwmi.c
//
// Abstract:
//
//     SCSI class driver routines
//
// Environment:
//
//     kernel mode only
//
// Notes:
//
//
// Revision History:
//
// --*/
//
// #ifndef __REACTOS__
// #include "stddef.h"
// #include "ntddk.h"
// #include "scsi.h"
//
// #include "classpnp.h"
//
// #include "mountdev.h"
//
// #include <stdarg.h>
// #endif
//
// #include "classp.h"
// #include <wmistr.h>
// #include <wmidata.h>
// // #include <classlog.h> __REACTOS__
//
// #ifdef DEBUG_USE_WPP
// #include "classwmi.tmh"
// #endif
//
// #define TIME_STRING_LENGTH      25
//
// BOOLEAN
// ClassFindGuid(
//     PGUIDREGINFO GuidList,
//     ULONG GuidCount,
//     LPGUID Guid,
//     PULONG GuidIndex
//     );
//
// NTSTATUS
// ClassQueryInternalDataBlock(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN ULONG GuidIndex,
//     IN ULONG BufferAvail,
//     OUT PUCHAR Buffer
//     );
//
// PWCHAR
// ConvertTickToDateTime(
//     IN LARGE_INTEGER Tick,
//      PWCHAR String
//     );
//
// BOOLEAN
// ClassFindInternalGuid(
//     LPGUID Guid,
//     PULONG GuidIndex
//     );
//
//
// //
// // This is the name for the MOF resource that must be part of all drivers that
// // register via this interface.
// #define MOFRESOURCENAME L"MofResourceName"
//
// //
// // What can be paged ???
// #ifdef ALLOC_PRAGMA
// #pragma alloc_text(PAGE, ClassSystemControl)
// #pragma alloc_text(PAGE, ClassFindGuid)
// #pragma alloc_text(PAGE, ClassFindInternalGuid)
// #endif
//
// #ifdef __REACTOS__
// #define MSStorageDriver_ClassErrorLogGuid {0xD5A9A51E, 0x03F9, 0x404d, {0x97, 0x22, 0x15, 0xF9, 0x0E, 0xB0, 0x70, 0x38}}
// #endif
//
// //
// // Define WMI interface to all class drivers
// //
// GUIDREGINFO wmiClassGuids[] =
// {
//     {
//         MSStorageDriver_ClassErrorLogGuid, 1, 0
//     }
// };
//
// #define MSStorageDriver_ClassErrorLogGuid_Index     0
// #define NUM_CLASS_WMI_GUIDS     (sizeof(wmiClassGuids) / sizeof(GUIDREGINFO))
//
//
// /*++////////////////////////////////////////////////////////////////////////////
//
// ClassFindGuid()
//
// Routine Description:
//
//     This routine will search the list of guids registered and return
//     the index for the one that was registered.
//
// Arguments:
//
//     GuidList is the list of guids to search
//
//     GuidCount is the count of guids in the list
//
//     Guid is the guid being searched for
//
//     *GuidIndex returns the index to the guid
//
// Return Value:
//
//     TRUE if guid is found else FALSE
//
// --*/
// BOOLEAN
// ClassFindGuid(
//     PGUIDREGINFO GuidList,
//     ULONG GuidCount,
//     LPGUID Guid,
//     PULONG GuidIndex
//     )
// {
//     ULONG i;
//
//     PAGED_CODE();
//
//     for (i = 0; i < GuidCount; i++)
//     {
//         if (IsEqualGUID(Guid, &GuidList[i].Guid))
//         {
//             *GuidIndex = i;
//             return(TRUE);
//         }
//     }
//     return(FALSE);
// } // end ClassFindGuid()
//
// /*++////////////////////////////////////////////////////////////////////////////
//
// ClassFindInternalGuid()
//
// Routine Description:
//
//     This routine will search the list of internal guids registered and return
//     the index for the one that was registered.
//
// Arguments:
//
//     Guid is the guid being searched for
//
//     *GuidIndex returns the index to the guid
//
// Return Value:
//
//     TRUE if guid is found else FALSE
//
// --*/
// BOOLEAN
// ClassFindInternalGuid(
//     LPGUID Guid,
//     PULONG GuidIndex
//     )
// {
//     ULONG i;
//
//     PAGED_CODE();
//
//     for (i = 0; i < NUM_CLASS_WMI_GUIDS; i++)
//     {
//         if (IsEqualGUID(Guid, &wmiClassGuids[i].Guid))
//         {
//             *GuidIndex = i;
//             return(TRUE);
//         }
//     }
//
//     return(FALSE);
// } // end ClassFindGuid()
//
// /*++////////////////////////////////////////////////////////////////////////////
//
// ClassSystemControl()
//
// Routine Description:
//
//     Dispatch routine for IRP_MJ_SYSTEM_CONTROL. This routine will process
//     all wmi requests received, forwarding them if they are not for this
//     driver or determining if the guid is valid and if so passing it to
//     the driver specific function for handing wmi requests.
//
// Arguments:
//
//     DeviceObject - Supplies a pointer to the device object for this request.
//
//     Irp - Supplies the Irp making the request.
//
// Return Value:
//
//     status
//
// --*/
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ClassSystemControl(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp
//     )
// {
//     PCOMMON_DEVICE_EXTENSION commonExtension = DeviceObject->DeviceExtension;
//     PCLASS_DRIVER_EXTENSION driverExtension;
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation(Irp);
//     ULONG isRemoved;
//     ULONG bufferSize;
//     PUCHAR buffer;
//     NTSTATUS status;
//     UCHAR minorFunction;
//     ULONG guidIndex = (ULONG)-1;
//     PCLASS_WMI_INFO classWmiInfo;
//     BOOLEAN isInternalGuid = FALSE;
//
//     PAGED_CODE();
//
//     //
//     // Make sure device has not been removed
//     isRemoved = ClassAcquireRemoveLock(DeviceObject, Irp);
//     if(isRemoved)
//     {
//         Irp->IoStatus.Status = STATUS_DEVICE_DOES_NOT_EXIST;
//         ClassReleaseRemoveLock(DeviceObject, Irp);
//         ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//         return STATUS_DEVICE_DOES_NOT_EXIST;
//     }
//
//     //
//     // If the irp is not a WMI irp or it is not targetted at this device
//     // or this device has not regstered with WMI then just forward it on.
//     minorFunction = irpStack->MinorFunction;
//     if ((minorFunction > IRP_MN_EXECUTE_METHOD) ||
//         (irpStack->Parameters.WMI.ProviderId != (ULONG_PTR)DeviceObject) ||
//         ((minorFunction != IRP_MN_REGINFO) &&
//          (commonExtension->GuidCount == 0)))
//     {
//         //
//         // CONSIDER: Do I need to hang onto lock until IoCallDriver returns ?
//         IoSkipCurrentIrpStackLocation(Irp);
//         ClassReleaseRemoveLock(DeviceObject, Irp);
//         return(IoCallDriver(commonExtension->LowerDeviceObject, Irp));
//     }
//
//     buffer = (PUCHAR)irpStack->Parameters.WMI.Buffer;
//     bufferSize = irpStack->Parameters.WMI.BufferSize;
//
//     if (minorFunction != IRP_MN_REGINFO)
//     {
//         //
//         // For all requests other than query registration info we are passed
//         // a guid. Determine if the guid is one that is supported by the
//         // device.
//         if (commonExtension->GuidRegInfo != NULL &&
//             ClassFindGuid(commonExtension->GuidRegInfo,
//                             commonExtension->GuidCount,
//                             (LPGUID)irpStack->Parameters.WMI.DataPath,
//                             &guidIndex))
//         {
//             isInternalGuid = FALSE;
//             status = STATUS_SUCCESS;
//         } else if (ClassFindInternalGuid((LPGUID)irpStack->Parameters.WMI.DataPath,
//                             &guidIndex)) {
//             isInternalGuid = TRUE;
//             status = STATUS_SUCCESS;
//         } else {
//             status = STATUS_WMI_GUID_NOT_FOUND;
//             TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "WMI GUID not found!"));
//         }
//
//         TracePrint((TRACE_LEVEL_VERBOSE, TRACE_FLAG_WMI, "WMI Find Guid = %x, isInternalGuid = %x", status, isInternalGuid));
//         if (NT_SUCCESS(status) &&
//             ((minorFunction == IRP_MN_QUERY_SINGLE_INSTANCE) ||
//              (minorFunction == IRP_MN_CHANGE_SINGLE_INSTANCE) ||
//              (minorFunction == IRP_MN_CHANGE_SINGLE_ITEM) ||
//              (minorFunction == IRP_MN_EXECUTE_METHOD)))
//         {
//             if ( (((PWNODE_HEADER)buffer)->Flags) &
//                                           WNODE_FLAG_STATIC_INSTANCE_NAMES)
//             {
//                 if ( ((PWNODE_SINGLE_INSTANCE)buffer)->InstanceIndex != 0 )
//                 {
//                     status = STATUS_WMI_INSTANCE_NOT_FOUND;
//                 }
//             } else {
//                 status = STATUS_WMI_INSTANCE_NOT_FOUND;
//                 TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "WMI Instance not found!"));
//             }
//         }
//
//         if (! NT_SUCCESS(status))
//         {
//             Irp->IoStatus.Status = status;
//             ClassReleaseRemoveLock(DeviceObject, Irp);
//             ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//             return(status);
//         }
//     }
//
//     driverExtension = commonExtension->DriverExtension;
//
//     classWmiInfo = commonExtension->IsFdo ?
//                            &driverExtension->InitData.FdoData.ClassWmiInfo :
//                            &driverExtension->InitData.PdoData.ClassWmiInfo;
//     switch(minorFunction)
//     {
//         case IRP_MN_REGINFO:
//         {
//             ULONG guidCount;
//             PGUIDREGINFO guidList;
//             PWMIREGINFOW wmiRegInfo;
//             PWMIREGGUIDW wmiRegGuid;
//             PUNICODE_STRING regPath;
//             PWCHAR stringPtr;
//             ULONG retSize;
//             ULONG registryPathOffset;
//             ULONG mofResourceOffset;
//             ULONG bufferNeeded;
//             ULONG i;
//             ULONG_PTR nameInfo;
//             ULONG nameSize, nameOffset, nameFlags;
//             UNICODE_STRING name, mofName;
//             PCLASS_QUERY_WMI_REGINFO_EX ClassQueryWmiRegInfoEx;
//
//             name.Buffer = NULL;
//             name.Length = 0;
//             name.MaximumLength = 0;
//             nameFlags = 0;
//
//             ClassQueryWmiRegInfoEx = commonExtension->IsFdo ?
//                                driverExtension->ClassFdoQueryWmiRegInfoEx :
//                                driverExtension->ClassPdoQueryWmiRegInfoEx;
//
//             if ((classWmiInfo->GuidRegInfo != NULL) &&
//                 (classWmiInfo->ClassQueryWmiRegInfo != NULL) &&
//                 (ClassQueryWmiRegInfoEx == NULL))
//             {
//                 status = classWmiInfo->ClassQueryWmiRegInfo(
//                                                         DeviceObject,
//                                                         &nameFlags,
//                                                         &name);
//
//                 RtlInitUnicodeString(&mofName, MOFRESOURCENAME);
//
//             } else if ((classWmiInfo->GuidRegInfo != NULL) && (ClassQueryWmiRegInfoEx != NULL)) {
//                 RtlInitUnicodeString(&mofName, L"");
//
//                 status = (*ClassQueryWmiRegInfoEx)(
//                                                     DeviceObject,
//                                                     &nameFlags,
//                                                     &name,
//                                                     &mofName);
//             } else {
//                 RtlInitUnicodeString(&mofName, L"");
//                 nameFlags = WMIREG_FLAG_INSTANCE_PDO;
//                 status = STATUS_SUCCESS;
//             }
//
//             if (NT_SUCCESS(status) &&
//                 (! (nameFlags &  WMIREG_FLAG_INSTANCE_PDO) &&
//                 (name.Buffer == NULL)))
//             {
//                 //
//                 // if PDO flag not specified then an instance name must be
//                 status = STATUS_INVALID_DEVICE_REQUEST;
//                 TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "Invalid Device Request!"));
//             }
//
//             if (NT_SUCCESS(status))
//             {
//                 guidList = classWmiInfo->GuidRegInfo;
//                 guidCount = (classWmiInfo->GuidRegInfo == NULL ? 0 : classWmiInfo->GuidCount) + NUM_CLASS_WMI_GUIDS;
//
//                 nameOffset = sizeof(WMIREGINFO) +
//                                       guidCount * sizeof(WMIREGGUIDW);
//
//                 if (nameFlags & WMIREG_FLAG_INSTANCE_PDO)
//                 {
//                     nameSize = 0;
//                     nameInfo = commonExtension->IsFdo ?
//                                    (ULONG_PTR)((PFUNCTIONAL_DEVICE_EXTENSION)commonExtension)->LowerPdo :
//                                    (ULONG_PTR)DeviceObject;
//                 } else {
//                     nameFlags |= WMIREG_FLAG_INSTANCE_LIST;
//                     nameSize = name.Length + sizeof(USHORT);
//                     nameInfo = nameOffset;
//                 }
//
//                 mofResourceOffset = nameOffset + nameSize;
//
//                 registryPathOffset = mofResourceOffset +
//                                   mofName.Length + sizeof(USHORT);
//
//                 regPath = &driverExtension->RegistryPath;
//
//                 bufferNeeded = registryPathOffset + regPath->Length;
//                 bufferNeeded += sizeof(USHORT);
//
//                 if (bufferNeeded <= bufferSize)
//                 {
//                     retSize = bufferNeeded;
//
//                     commonExtension->GuidCount = guidCount;
//                     commonExtension->GuidRegInfo = guidList;
//
//                     wmiRegInfo = (PWMIREGINFO)buffer;
//                     wmiRegInfo->BufferSize = bufferNeeded;
//                     wmiRegInfo->NextWmiRegInfo = 0;
//                     wmiRegInfo->MofResourceName = mofResourceOffset;
//                     wmiRegInfo->RegistryPath = registryPathOffset;
//                     wmiRegInfo->GuidCount = guidCount;
//
//                     for (i = 0; i < classWmiInfo->GuidCount; i++)
//                     {
//                         wmiRegGuid = &wmiRegInfo->WmiRegGuid[i];
//                         wmiRegGuid->Guid = guidList[i].Guid;
//                         wmiRegGuid->Flags = guidList[i].Flags | nameFlags;
//                         wmiRegGuid->InstanceInfo = nameInfo;
//                         wmiRegGuid->InstanceCount = 1;
//                     }
//                     for (i = 0; i < NUM_CLASS_WMI_GUIDS; i++)
//                     {
//                         wmiRegGuid = &wmiRegInfo->WmiRegGuid[i + classWmiInfo->GuidCount];
//                         wmiRegGuid->Guid = wmiClassGuids[i].Guid;
//                         wmiRegGuid->Flags = wmiClassGuids[i].Flags | nameFlags;
//                         wmiRegGuid->InstanceInfo = nameInfo;
//                         wmiRegGuid->InstanceCount = 1;
//                     }
//
//                     if ( nameFlags &  WMIREG_FLAG_INSTANCE_LIST)
//                     {
//                         bufferNeeded = nameOffset + sizeof(WCHAR);
//                         bufferNeeded += name.Length;
//
//                         if (bufferSize >= bufferNeeded){
//                             stringPtr = (PWCHAR)((PUCHAR)buffer + nameOffset);
//                             *stringPtr++ = name.Length;
//                             RtlCopyMemory(stringPtr, name.Buffer, name.Length);
//                         }
//                         else {
//                             NT_ASSERT(bufferSize >= bufferNeeded);
//                             status = STATUS_INVALID_BUFFER_SIZE;
//                         }
//                     }
//
//                     bufferNeeded = mofResourceOffset + sizeof(WCHAR);
//                     bufferNeeded += mofName.Length;
//
//                     if (bufferSize >= bufferNeeded){
//                         stringPtr = (PWCHAR)((PUCHAR)buffer + mofResourceOffset);
//                         *stringPtr++ = mofName.Length;
//                         RtlCopyMemory(stringPtr, mofName.Buffer, mofName.Length);
//                     }
//                     else {
//                         NT_ASSERT(bufferSize >= bufferNeeded);
//                         status = STATUS_INVALID_BUFFER_SIZE;
//                     }
//
//                     bufferNeeded = registryPathOffset + sizeof(WCHAR);
//                     bufferNeeded += regPath->Length;
//
//                     if (bufferSize >= bufferNeeded){
//                         stringPtr = (PWCHAR)((PUCHAR)buffer + registryPathOffset);
//                         *stringPtr++ = regPath->Length;
//                         RtlCopyMemory(stringPtr,
//                                   regPath->Buffer,
//                                   regPath->Length);
//                     }
//                     else {
//
//                         NT_ASSERT(bufferSize >= bufferNeeded);
//                         TracePrint((TRACE_LEVEL_WARNING, TRACE_FLAG_WMI, "Invalid Buffer Size!"));
//                         status = STATUS_INVALID_BUFFER_SIZE;
//                     }
//
//                 } else {
//                     *((PULONG)buffer) = bufferNeeded;
//                     retSize = sizeof(ULONG);
//                 }
//             } else {
//                 retSize = 0;
//             }
//
//             FREE_POOL(name.Buffer);
//
//             Irp->IoStatus.Status = status;
//             Irp->IoStatus.Information = retSize;
//             ClassReleaseRemoveLock(DeviceObject, Irp);
//             ClassCompleteRequest(DeviceObject, Irp, IO_NO_INCREMENT);
//             return(status);
//         }
//
//         case IRP_MN_QUERY_ALL_DATA:
//         {
//             PWNODE_ALL_DATA wnode;
//             ULONG bufferAvail;
//
//             wnode = (PWNODE_ALL_DATA)buffer;
//
//             if (bufferSize < sizeof(WNODE_ALL_DATA))
//             {
//                 bufferAvail = 0;
//             } else {
//                 bufferAvail = bufferSize - sizeof(WNODE_ALL_DATA);
//             }
//
//             wnode->DataBlockOffset = sizeof(WNODE_ALL_DATA);
//
//             NT_ASSERT(guidIndex != (ULONG)-1);
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassQueryInternalDataBlock(
//                                                 DeviceObject,
//                                                 Irp,
//                                                 guidIndex,
//                                                 bufferAvail,
//                                                 buffer + sizeof(WNODE_ALL_DATA));
//             } else {
//                 status = classWmiInfo->ClassQueryWmiDataBlock(
//                                                  DeviceObject,
//                                                  Irp,
//                                                  guidIndex,
//                                                  bufferAvail,
//                                                  buffer + sizeof(WNODE_ALL_DATA));
//             }
//             break;
//         }
//
//         case IRP_MN_QUERY_SINGLE_INSTANCE:
//         {
//             PWNODE_SINGLE_INSTANCE wnode;
//             ULONG dataBlockOffset;
//
//             wnode = (PWNODE_SINGLE_INSTANCE)buffer;
//
//             dataBlockOffset = wnode->DataBlockOffset;
//
//             NT_ASSERT(guidIndex != (ULONG)-1);
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassQueryInternalDataBlock(
//                                             DeviceObject,
//                                             Irp,
//                                             guidIndex,
//                                             bufferSize - dataBlockOffset,
//                                             (PUCHAR)wnode + dataBlockOffset);
//             } else {
//                 status = classWmiInfo->ClassQueryWmiDataBlock(
//                                               DeviceObject,
//                                               Irp,
//                                               guidIndex,
//                                               bufferSize - dataBlockOffset,
//                                               (PUCHAR)wnode + dataBlockOffset);
//             }
//             break;
//         }
//
//         case IRP_MN_CHANGE_SINGLE_INSTANCE:
//         {
//             PWNODE_SINGLE_INSTANCE wnode;
//
//             wnode = (PWNODE_SINGLE_INSTANCE)buffer;
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassWmiCompleteRequest(DeviceObject,
//                                                 Irp,
//                                                 STATUS_WMI_GUID_NOT_FOUND,
//                                                 0,
//                                                 IO_NO_INCREMENT);
//             } else {
//
//                 NT_ASSERT(guidIndex != (ULONG)-1);
//
//                 status = classWmiInfo->ClassSetWmiDataBlock(
//                                          DeviceObject,
//                                          Irp,
//                                          guidIndex,
//                                          wnode->SizeDataBlock,
//                                          (PUCHAR)wnode + wnode->DataBlockOffset);
//             }
//
//             break;
//         }
//
//         case IRP_MN_CHANGE_SINGLE_ITEM:
//         {
//             PWNODE_SINGLE_ITEM wnode;
//
//             wnode = (PWNODE_SINGLE_ITEM)buffer;
//
//             NT_ASSERT(guidIndex != (ULONG)-1);
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassWmiCompleteRequest(DeviceObject,
//                                                 Irp,
//                                                 STATUS_WMI_GUID_NOT_FOUND,
//                                                 0,
//                                                 IO_NO_INCREMENT);
//             } else {
//
//                 NT_ASSERT(guidIndex != (ULONG)-1);
//
//                 status = classWmiInfo->ClassSetWmiDataItem(
//                                          DeviceObject,
//                                          Irp,
//                                          guidIndex,
//                                          wnode->ItemId,
//                                          wnode->SizeDataItem,
//                                          (PUCHAR)wnode + wnode->DataBlockOffset);
//
//             }
//
//             break;
//         }
//
//         case IRP_MN_EXECUTE_METHOD:
//         {
//             PWNODE_METHOD_ITEM wnode;
//
//             wnode = (PWNODE_METHOD_ITEM)buffer;
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassWmiCompleteRequest(DeviceObject,
//                                                 Irp,
//                                                 STATUS_WMI_GUID_NOT_FOUND,
//                                                 0,
//                                                 IO_NO_INCREMENT);
//             } else {
//
//                 NT_ASSERT(guidIndex != (ULONG)-1);
//
//                 status = classWmiInfo->ClassExecuteWmiMethod(
//                                              DeviceObject,
//                                              Irp,
//                                              guidIndex,
//                                              wnode->MethodId,
//                                              wnode->SizeDataBlock,
//                                              bufferSize - wnode->DataBlockOffset,
//                                              buffer + wnode->DataBlockOffset);
//             }
//
//             break;
//         }
//
//         case IRP_MN_ENABLE_EVENTS:
//         {
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassWmiCompleteRequest(DeviceObject,
//                                                 Irp,
//                                                 STATUS_WMI_GUID_NOT_FOUND,
//                                                 0,
//                                                 IO_NO_INCREMENT);
//             } else {
//
//                 NT_ASSERT(guidIndex != (ULONG)-1);
//
//                 status = classWmiInfo->ClassWmiFunctionControl(
//                                                                DeviceObject,
//                                                                Irp,
//                                                                guidIndex,
//                                                                EventGeneration,
//                                                                TRUE);
//             }
//             break;
//         }
//
//         case IRP_MN_DISABLE_EVENTS:
//         {
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassWmiCompleteRequest(DeviceObject,
//                                                 Irp,
//                                                 STATUS_WMI_GUID_NOT_FOUND,
//                                                 0,
//                                                 IO_NO_INCREMENT);
//             } else {
//
//                 NT_ASSERT(guidIndex != (ULONG)-1);
//
//                 status = classWmiInfo->ClassWmiFunctionControl(
//                                                                DeviceObject,
//                                                                Irp,
//                                                                guidIndex,
//                                                                EventGeneration,
//                                                                FALSE);
//             }
//             break;
//         }
//
//         case IRP_MN_ENABLE_COLLECTION:
//         {
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassWmiCompleteRequest(DeviceObject,
//                                                 Irp,
//                                                 STATUS_WMI_GUID_NOT_FOUND,
//                                                 0,
//                                                 IO_NO_INCREMENT);
//             } else {
//
//                 NT_ASSERT(guidIndex != (ULONG)-1);
//
//                 status = classWmiInfo->ClassWmiFunctionControl(
//                                                              DeviceObject,
//                                                              Irp,
//                                                              guidIndex,
//                                                              DataBlockCollection,
//                                                              TRUE);
//             }
//             break;
//         }
//
//         case IRP_MN_DISABLE_COLLECTION:
//         {
//             _Analysis_assume_(isInternalGuid);
//             if (isInternalGuid)
//             {
//                 status = ClassWmiCompleteRequest(DeviceObject,
//                                                 Irp,
//                                                 STATUS_WMI_GUID_NOT_FOUND,
//                                                 0,
//                                                 IO_NO_INCREMENT);
//             } else {
//
//                 NT_ASSERT(guidIndex != (ULONG)-1);
//
//                 status = classWmiInfo->ClassWmiFunctionControl(
//                                                              DeviceObject,
//                                                              Irp,
//                                                              guidIndex,
//                                                              DataBlockCollection,
//                                                              FALSE);
//             }
//
//             break;
//         }
//
//         default:
//         {
//             status = STATUS_INVALID_DEVICE_REQUEST;
//             break;
//         }
//
//     }
//
//     return(status);
// } // end ClassSystemControl()
//
//
// NTSTATUS
// ClassQueryInternalDataBlock(
//     IN PDEVICE_OBJECT DeviceObject,
//     IN PIRP Irp,
//     IN ULONG GuidIndex,
//     IN ULONG BufferAvail,
//     OUT PUCHAR Buffer
//     )
// /*++
//
// Routine Description:
//
//     This routine allows querying for the contents of an internal WMI
//     data block. When the driver has finished filling the data block it
//     must call ClassWmiCompleteRequest to complete the irp.
//
// Arguments:
//
//     DeviceObject is the device whose data block is being queried
//
//     Irp is the Irp that makes this request
//
//     GuidIndex is the index into the list of guids provided when the
//         device registered
//
//     BufferAvail on has the maximum size available to write the data
//         block.
//
//     Buffer on return is filled with the returned data block
//
//
// Return Value:
//
//     status
//
// --*/
// {
//     NTSTATUS status;
// #ifndef __REACTOS__ // WMI in not a thing on ReactOS yet
//     ULONG sizeNeeded = 0, i;
//     PFUNCTIONAL_DEVICE_EXTENSION fdoExt = DeviceObject->DeviceExtension;
//     if (GuidIndex == MSStorageDriver_ClassErrorLogGuid_Index) {
//
//         //
//         // NOTE - ClassErrorLog is still using SCSI_REQUEST_BLOCK and will not be
//         // updated to support extended SRB until classpnp is updated to send >16
//         // byte CDBs. Extended SRBs will be translated to SCSI_REQUEST_BLOCK.
//         //
//         sizeNeeded = MSStorageDriver_ClassErrorLog_SIZE;
//         if (BufferAvail >= sizeNeeded) {
//             PMSStorageDriver_ClassErrorLog errorLog = (PMSStorageDriver_ClassErrorLog) Buffer;
//             PMSStorageDriver_ClassErrorLogEntry logEntry;
//             PMSStorageDriver_ScsiRequestBlock srbBlock;
//             PMSStorageDriver_SenseData senseData;
//             PCLASS_PRIVATE_FDO_DATA fdoData = fdoExt->PrivateFdoData;
//             PCLASS_ERROR_LOG_DATA fdoLogEntry;
//             PSCSI_REQUEST_BLOCK fdoSRBBlock;
//             PSENSE_DATA fdoSenseData;
//             errorLog->numEntries = NUM_ERROR_LOG_ENTRIES;
//             for (i = 0; i < NUM_ERROR_LOG_ENTRIES; i++) {
//                 fdoLogEntry = &fdoData->ErrorLogs[i];
//                 fdoSRBBlock = &fdoLogEntry->Srb;
//                 fdoSenseData = &fdoLogEntry->SenseData;
//                 logEntry = &errorLog->logEntries[i];
//                 srbBlock = &logEntry->srb;
//                 senseData = &logEntry->senseData;
//                 logEntry->tickCount = fdoLogEntry->TickCount.QuadPart;
//                 logEntry->portNumber = fdoLogEntry->PortNumber;
//                 logEntry->errorPaging = (fdoLogEntry->ErrorPaging == 0 ? FALSE : TRUE);
//                 logEntry->errorRetried = (fdoLogEntry->ErrorRetried == 0 ? FALSE : TRUE);
//                 logEntry->errorUnhandled = (fdoLogEntry->ErrorUnhandled == 0 ? FALSE : TRUE);
//                 logEntry->errorReserved = fdoLogEntry->ErrorReserved;
//                 RtlMoveMemory(logEntry->reserved, fdoLogEntry->Reserved, sizeof(logEntry->reserved));
//                 ConvertTickToDateTime(fdoLogEntry->TickCount, logEntry->eventTime);
//
//                 srbBlock->length = fdoSRBBlock->Length;
//                 srbBlock->function = fdoSRBBlock->Function;
//                 srbBlock->srbStatus = fdoSRBBlock->SrbStatus;
//                 srbBlock->scsiStatus = fdoSRBBlock->ScsiStatus;
//                 srbBlock->pathID = fdoSRBBlock->PathId;
//                 srbBlock->targetID = fdoSRBBlock->TargetId;
//                 srbBlock->lun = fdoSRBBlock->Lun;
//                 srbBlock->queueTag = fdoSRBBlock->QueueTag;
//                 srbBlock->queueAction = fdoSRBBlock->QueueAction;
//                 srbBlock->cdbLength = fdoSRBBlock->CdbLength;
//                 srbBlock->senseInfoBufferLength = fdoSRBBlock->SenseInfoBufferLength;
//                 srbBlock->srbFlags = fdoSRBBlock->SrbFlags;
//                 srbBlock->dataTransferLength = fdoSRBBlock->DataTransferLength;
//                 srbBlock->timeOutValue = fdoSRBBlock->TimeOutValue;
//                 srbBlock->dataBuffer = (ULONGLONG) fdoSRBBlock->DataBuffer;
//                 srbBlock->senseInfoBuffer = (ULONGLONG) fdoSRBBlock->SenseInfoBuffer;
//                 srbBlock->nextSRB = (ULONGLONG) fdoSRBBlock->NextSrb;
//                 srbBlock->originalRequest = (ULONGLONG) fdoSRBBlock->OriginalRequest;
//                 srbBlock->srbExtension = (ULONGLONG) fdoSRBBlock->SrbExtension;
//                 srbBlock->internalStatus = fdoSRBBlock->InternalStatus;
// #if defined(_WIN64)
//                 srbBlock->reserved = fdoSRBBlock->Reserved;
// #else
//                 srbBlock->reserved = 0;
// #endif
//                 RtlMoveMemory(srbBlock->cdb, fdoSRBBlock->Cdb, sizeof(srbBlock->cdb));
//
//                 //
//                 // Note: Sense data has been converted into Fixed format before it was
//                 //       put in the log.  Therefore, no conversion is needed here.
//                 //
//                 senseData->errorCode = fdoSenseData->ErrorCode;
//                 senseData->valid = (fdoSenseData->Valid == 0 ? FALSE : TRUE);
//                 senseData->segmentNumber = fdoSenseData->SegmentNumber;
//                 senseData->senseKey = fdoSenseData->SenseKey;
//                 senseData->reserved = (fdoSenseData->Reserved == 0 ? FALSE : TRUE);
//                 senseData->incorrectLength = (fdoSenseData->IncorrectLength == 0 ? FALSE : TRUE);
//                 senseData->endOfMedia = (fdoSenseData->EndOfMedia == 0 ? FALSE : TRUE);
//                 senseData->fileMark = (fdoSenseData->FileMark == 0 ? FALSE : TRUE);
//                 RtlMoveMemory(senseData->information, fdoSenseData->Information, sizeof(senseData->information));
//                 senseData->additionalSenseLength = fdoSenseData->AdditionalSenseLength;
//                 RtlMoveMemory(senseData->commandSpecificInformation, fdoSenseData->CommandSpecificInformation, sizeof(senseData->commandSpecificInformation));
//                 senseData->additionalSenseCode = fdoSenseData->AdditionalSenseCode;
//                 senseData->additionalSenseCodeQualifier = fdoSenseData->AdditionalSenseCodeQualifier;
//                 senseData->fieldReplaceableUnitCode = fdoSenseData->FieldReplaceableUnitCode;
//                 RtlMoveMemory(senseData->senseKeySpecific, fdoSenseData->SenseKeySpecific, sizeof(senseData->senseKeySpecific));
//             }
//             status = STATUS_SUCCESS;
//         } else {
//             status = STATUS_BUFFER_TOO_SMALL;
//         }
//     } else if (GuidIndex > 0 && GuidIndex < NUM_CLASS_WMI_GUIDS) {
//         status = STATUS_WMI_INSTANCE_NOT_FOUND;
//     } else {
//         status = STATUS_WMI_GUID_NOT_FOUND;
//     }
// #else
//     ULONG sizeNeeded = 0;
//     status = STATUS_WMI_GUID_NOT_FOUND;
// #endif
//     status = ClassWmiCompleteRequest(DeviceObject,
//                                     Irp,
//                                     status,
//                                     sizeNeeded,
//                                     IO_NO_INCREMENT);
//     return status;
// }
//
// PWCHAR
// ConvertTickToDateTime(
//     IN LARGE_INTEGER Tick,
//      PWCHAR String
//     )
//
// /*++
//
// Routine Description:
//
//     This routine converts a tick count to a datetime (MOF) data type
//
// Arguments:
//
//     Tick - The tick count that needs to be converted
//     String - The buffer to hold the time string, must be able to hold WCHAR[25]
//
// Return Value:
//
//     The time string
//
// --*/
//
// {
//     LARGE_INTEGER nowTick, nowTime, time;
//     ULONG maxInc = 0;
//     TIME_FIELDS timeFields = {0};
//     WCHAR outDateTime[TIME_STRING_LENGTH + 1];
//
//     nowTick.QuadPart = 0;
//     nowTime.QuadPart = 0;
//     //
//     // Translate the tick count to a system time
//     //
//     KeQueryTickCount(&nowTick);
//     maxInc = KeQueryTimeIncrement();
//     KeQuerySystemTime(&nowTime);
//     time.QuadPart = nowTime.QuadPart - ((nowTick.QuadPart - Tick.QuadPart) * maxInc);
//
//     RtlTimeToTimeFields(&time, &timeFields);
//
//     //
//     // The buffer String is of size MAX_PATH. Use that to specify the buffer size.
//     //
//     //yyyymmddhhmmss.mmmmmmsutc
//     RtlStringCbPrintfW(outDateTime, sizeof(outDateTime), L"%04d%02d%02d%02d%02d%02d.%03d***+000", timeFields.Year, timeFields.Month, timeFields.Day, timeFields.Hour, timeFields.Minute, timeFields.Second, timeFields.Milliseconds);
//     RtlMoveMemory(String, outDateTime, sizeof(WCHAR) * TIME_STRING_LENGTH);
//     return  String;
// }
//
// /*++////////////////////////////////////////////////////////////////////////////
//
// ClassWmiCompleteRequest()
//
// Routine Description:
//
//
//     This routine will do the work of completing a WMI irp. Depending upon the
//     the WMI request this routine will fixup the returned WNODE appropriately.
//
//     NOTE: This routine assumes that the ClassRemoveLock is held and it will
//           release it.
//
// Arguments:
//
//     DeviceObject - Supplies a pointer to the device object for this request.
//
//     Irp - Supplies the Irp making the request.
//
//     Status - Status to complete the irp with.  STATUS_BUFFER_TOO_SMALL is used
//         to indicate that more buffer is required for the data requested.
//
//     BufferUsed - number of bytes of actual data to return (not including WMI
//         specific structures)
//
//     PriorityBoost - priority boost to pass to ClassCompleteRequest
//
// Return Value:
//
//     status
//
// --*/
// SCSIPORT_API
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ClassWmiCompleteRequest(
//      PDEVICE_OBJECT DeviceObject,
//      PIRP Irp,
//      NTSTATUS Status,
//      ULONG BufferUsed,
//      CCHAR PriorityBoost
//     )
// {
//     PIO_STACK_LOCATION irpStack = IoGetCurrentIrpStackLocation(Irp);
//     PUCHAR buffer;
//     ULONG retSize;
//     UCHAR minorFunction;
//
//     minorFunction = irpStack->MinorFunction;
//     buffer = (PUCHAR)irpStack->Parameters.WMI.Buffer;
//
//     switch(minorFunction)
//     {
//         case IRP_MN_QUERY_ALL_DATA:
//         {
//             PWNODE_ALL_DATA wnode;
//             PWNODE_TOO_SMALL wnodeTooSmall;
//             ULONG bufferNeeded;
//
//             wnode = (PWNODE_ALL_DATA)buffer;
//
//             bufferNeeded = sizeof(WNODE_ALL_DATA) + BufferUsed;
//
//             if (NT_SUCCESS(Status))
//             {
//                 retSize = bufferNeeded;
//                 wnode->WnodeHeader.BufferSize = bufferNeeded;
//                 KeQuerySystemTime(&wnode->WnodeHeader.TimeStamp);
//                 wnode->WnodeHeader.Flags |= WNODE_FLAG_FIXED_INSTANCE_SIZE;
//                 wnode->FixedInstanceSize = BufferUsed;
//                 wnode->InstanceCount = 1;
//
//             } else if (Status == STATUS_BUFFER_TOO_SMALL) {
//                 wnodeTooSmall = (PWNODE_TOO_SMALL)wnode;
//
//                 wnodeTooSmall->WnodeHeader.BufferSize = sizeof(WNODE_TOO_SMALL);
//                 wnodeTooSmall->WnodeHeader.Flags = WNODE_FLAG_TOO_SMALL;
//                 wnodeTooSmall->SizeNeeded = sizeof(WNODE_ALL_DATA) + BufferUsed;
//                 retSize = sizeof(WNODE_TOO_SMALL);
//                 Status = STATUS_SUCCESS;
//             } else {
//                 retSize = 0;
//             }
//             break;
//         }
//
//         case IRP_MN_QUERY_SINGLE_INSTANCE:
//         {
//             PWNODE_SINGLE_INSTANCE wnode;
//             PWNODE_TOO_SMALL wnodeTooSmall;
//             ULONG bufferNeeded;
//
//             wnode = (PWNODE_SINGLE_INSTANCE)buffer;
//
//             bufferNeeded = wnode->DataBlockOffset + BufferUsed;
//
//             if (NT_SUCCESS(Status))
//             {
//                 retSize = bufferNeeded;
//                 wnode->WnodeHeader.BufferSize = bufferNeeded;
//                 KeQuerySystemTime(&wnode->WnodeHeader.TimeStamp);
//                 wnode->SizeDataBlock = BufferUsed;
//
//             } else if (Status == STATUS_BUFFER_TOO_SMALL) {
//                 wnodeTooSmall = (PWNODE_TOO_SMALL)wnode;
//
//                 wnodeTooSmall->WnodeHeader.BufferSize = sizeof(WNODE_TOO_SMALL);
//                 wnodeTooSmall->WnodeHeader.Flags = WNODE_FLAG_TOO_SMALL;
//                 wnodeTooSmall->SizeNeeded = bufferNeeded;
//                 retSize = sizeof(WNODE_TOO_SMALL);
//                 Status = STATUS_SUCCESS;
//             } else {
//                 retSize = 0;
//             }
//             break;
//         }
//
//         case IRP_MN_EXECUTE_METHOD:
//         {
//             PWNODE_METHOD_ITEM wnode;
//             PWNODE_TOO_SMALL wnodeTooSmall;
//             ULONG bufferNeeded;
//
//             wnode = (PWNODE_METHOD_ITEM)buffer;
//
//             bufferNeeded = wnode->DataBlockOffset + BufferUsed;
//
//             if (NT_SUCCESS(Status))
//             {
//                 retSize = bufferNeeded;
//                 wnode->WnodeHeader.BufferSize = bufferNeeded;
//                 KeQuerySystemTime(&wnode->WnodeHeader.TimeStamp);
//                 wnode->SizeDataBlock = BufferUsed;
//
//             } else if (Status == STATUS_BUFFER_TOO_SMALL) {
//                 wnodeTooSmall = (PWNODE_TOO_SMALL)wnode;
//
//                 wnodeTooSmall->WnodeHeader.BufferSize = sizeof(WNODE_TOO_SMALL);
//                 wnodeTooSmall->WnodeHeader.Flags = WNODE_FLAG_TOO_SMALL;
//                 wnodeTooSmall->SizeNeeded = bufferNeeded;
//                 retSize = sizeof(WNODE_TOO_SMALL);
//                 Status = STATUS_SUCCESS;
//             } else {
//                 retSize = 0;
//             }
//             break;
//         }
//
//         default:
//         {
//             //
//             // All other requests don't return any data
//             retSize = 0;
//             break;
//         }
//
//     }
//
//     Irp->IoStatus.Status = Status;
//     Irp->IoStatus.Information = retSize;
//     ClassReleaseRemoveLock(DeviceObject, Irp);
//     ClassCompleteRequest(DeviceObject, Irp, PriorityBoost);
//     return(Status);
// } // end ClassWmiCompleteRequest()
//
// /*++////////////////////////////////////////////////////////////////////////////
//
// ClassWmiFireEvent()
//
// Routine Description:
//
//     This routine will fire a WMI event using the data buffer passed. This
//     routine may be called at or below DPC level
//
// Arguments:
//
//     DeviceObject - Supplies a pointer to the device object for this event
//
//     Guid is pointer to the GUID that represents the event
//
//     InstanceIndex is the index of the instance of the event
//
//     EventDataSize is the number of bytes of data that is being fired with
//        with the event
//
//     EventData is the data that is fired with the events. This may be NULL
//         if there is no data associated with the event
//
//
// Return Value:
//
//     status
//
// --*/
// _IRQL_requires_max_(DISPATCH_LEVEL)
// NTSTATUS
// NTAPI /* ReactOS Change: GCC Does not support STDCALL by default */
// ClassWmiFireEvent(
//      PDEVICE_OBJECT DeviceObject,
//      LPGUID Guid,
//      ULONG InstanceIndex,
//      ULONG EventDataSize,
//      PVOID EventData
//     )
// {
//
//     ULONG sizeNeeded;
//     PWNODE_SINGLE_INSTANCE event;
//     NTSTATUS status;
//
//     if (EventData == NULL)
//     {
//         EventDataSize = 0;
//     }
//
//     sizeNeeded = sizeof(WNODE_SINGLE_INSTANCE) + EventDataSize;
//
//     event = ExAllocatePoolWithTag(NonPagedPoolNx, sizeNeeded, CLASS_TAG_WMI);
//     if (event != NULL)
//     {
//         RtlZeroMemory(event, sizeNeeded);
//         event->WnodeHeader.Guid = *Guid;
//         event->WnodeHeader.ProviderId = IoWMIDeviceObjectToProviderId(DeviceObject);
//         event->WnodeHeader.BufferSize = sizeNeeded;
//         event->WnodeHeader.Flags =  WNODE_FLAG_SINGLE_INSTANCE |
//                                     WNODE_FLAG_EVENT_ITEM |
//                                     WNODE_FLAG_STATIC_INSTANCE_NAMES;
//         KeQuerySystemTime(&event->WnodeHeader.TimeStamp);
//
//         event->InstanceIndex = InstanceIndex;
//         event->SizeDataBlock = EventDataSize;
//         event->DataBlockOffset = sizeof(WNODE_SINGLE_INSTANCE);
//         if (EventData != NULL)
//         {
//             RtlCopyMemory( &event->VariableData, EventData, EventDataSize);
//         }
//
//         status = IoWMIWriteEvent(event);
//         if (! NT_SUCCESS(status))
//         {
//             FREE_POOL(event);
//         }
//     } else {
//         status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     return(status);
// } // end ClassWmiFireEvent()
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
