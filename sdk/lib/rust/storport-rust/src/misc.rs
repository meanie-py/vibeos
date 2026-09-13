use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CM_FULL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_RESOURCE_LIST, _KWAIT_REASON_Executive, GUID, _IO_STATUS_BLOCK, IoBuildSynchronousFsdRequest, IoGetDeviceProperty, KEVENT, KeInitializeEvent, KeWaitForSingleObject, MAPPED_ADDRESS, _EVENT_TYPE_NotificationEvent, PBUS_INTERFACE_STANDARD, PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST, PDEVICE_OBJECT, PGUID, PINTERFACE, PIO_STACK_LOCATION, PIRP, PKAFFINITY, PKIRQL, PMAPPED_ADDRESS, PPHYSICAL_ADDRESS, RtlCompareMemory};
use ntkrnl_sys::inline::tier1::{IoGetNextIrpStackLocation, IoSkipCurrentIrpStackLocation};
use ntkrnl_sys::ddk::{PCSTR, CCHAR, INT, INTERFACE_TYPE, KINTERRUPT_MODE, KPROCESSOR_MODE};
use core::ffi::c_void;

extern "C" {
    fn DbgPrint(Format: *const ::core::ffi::c_char, ...) -> ::core::ffi::c_int;
}

// ---- file-scope preamble (translated) ----
pub const NDEBUG: () = ();

#[no_mangle]
pub unsafe extern "system" fn ForwardIrpAndForget(
    mut LowerDevice: PDEVICE_OBJECT,
    mut Irp: PIRP,
) -> NTSTATUS {
    if LowerDevice.is_null() {
        RtlAssert(
            b"LowerDevice\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            b"misc.c\0" as *const u8 as *const ::core::ffi::c_char as PVOID,
            24 as ULONG,
            ::core::ptr::null_mut::<CHAR>(),
        );
    } else {
    };
    IoSkipCurrentIrpStackLocation(Irp);
    return IoCallDriver(LowerDevice, Irp);
}
#[no_mangle]
pub unsafe extern "C" fn GetBusInterface(mut DeviceObject: PDEVICE_OBJECT) -> INTERFACE_TYPE {
    let mut Guid: GUID = GUID {
        Data1: 0,
        Data2: 0,
        Data3: 0,
        Data4: [0; 8],
    };
    let mut Length: ULONG = 0;
    let mut Status: NTSTATUS = 0;
    Status = IoGetDeviceProperty(
        DeviceObject,
        DevicePropertyBusTypeGuid,
        core::mem::size_of::<GUID>() as ULONG,
        core::ptr::addr_of_mut!(Guid) as PVOID,
        core::ptr::addr_of_mut!(Length),
    );
    if !NT_SUCCESS(Status) {
        return InterfaceTypeUndefined;
    }
    if RtlCompareMemory(
        core::ptr::addr_of_mut!(Guid) as *const core::ffi::c_void,
        core::ptr::addr_of!(GUID_BUS_TYPE_PCMCIA) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as _,
    ) as usize
        == core::mem::size_of::<GUID>() as usize
    {
        return PCMCIABus;
    } else if RtlCompareMemory(
        core::ptr::addr_of_mut!(Guid) as *const core::ffi::c_void,
        core::ptr::addr_of!(GUID_BUS_TYPE_PCI) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as _,
    ) as usize
        == core::mem::size_of::<GUID>() as usize
    {
        return PCIBus;
    } else if RtlCompareMemory(
        core::ptr::addr_of_mut!(Guid) as *const core::ffi::c_void,
        core::ptr::addr_of!(GUID_BUS_TYPE_ISAPNP) as *const core::ffi::c_void,
        core::mem::size_of::<GUID>() as _,
    ) as usize
        == core::mem::size_of::<GUID>() as usize
    {
        return PNPISABus;
    }
    return InterfaceTypeUndefined;
}
unsafe fn GetResourceListSize(mut ResourceList: PCM_RESOURCE_LIST) -> ULONG {
    let mut Descriptor: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut Size: ULONG;

    if DbgPrint(
        b"(%s:%d) GetResourceListSize(%p)\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
        66 as ::core::ffi::c_int,
        ResourceList,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
            66 as ::core::ffi::c_int,
        );
    }

    Size = core::mem::size_of::<CM_RESOURCE_LIST>() as ULONG;
    if (*ResourceList).Count == 0 {
        if DbgPrint(
            b"(%s:%d) Size: 0x%lx (%u)\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
            71 as ::core::ffi::c_int,
            Size,
            Size,
        ) != 0
        {
            DbgPrint(
                b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
                b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
                71 as ::core::ffi::c_int,
            );
        }
        return Size;
    }

    if DbgPrint(
        b"(%s:%d) ResourceList->Count: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
        75 as ::core::ffi::c_int,
        (*ResourceList).Count,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
            75 as ::core::ffi::c_int,
        );
    }

    Descriptor = (&raw mut (*ResourceList).List as *mut CM_FULL_RESOURCE_DESCRIPTOR)
        .offset(0 as isize) as PCM_FULL_RESOURCE_DESCRIPTOR;

    if DbgPrint(
        b"(%s:%d) PartialResourceList->Count: %lu\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
        79 as ::core::ffi::c_int,
        (*Descriptor).PartialResourceList.Count,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
            79 as ::core::ffi::c_int,
        );
    }

    if (*Descriptor).PartialResourceList.Count > 1 {
        Size = (Size as ::core::ffi::c_ulonglong).wrapping_add(
            (((*Descriptor).PartialResourceList.Count as ::core::ffi::c_ulong)
                .wrapping_sub(1 as ::core::ffi::c_ulong) as usize)
                .wrapping_mul(core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>() as usize)
                as ::core::ffi::c_ulonglong,
        ) as ULONG;
    }

    if DbgPrint(
        b"(%s:%d) Size: 0x%lx (%u)\n\0" as *const u8 as *const ::core::ffi::c_char,
        b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
        85 as ::core::ffi::c_int,
        Size,
        Size,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0" as *const u8 as *const ::core::ffi::c_char,
            b"misc.c\0" as *const u8 as *const ::core::ffi::c_char,
            85 as ::core::ffi::c_int,
        );
    }

    return Size;
}
#[no_mangle]
pub unsafe extern "C" fn CopyResourceList(
    mut PoolType: POOL_TYPE,
    mut Source: PCM_RESOURCE_LIST,
) -> PCM_RESOURCE_LIST {
    const TAG_RESOURCE_LIST: u32 = 0x4C527453; // 'LRtS' drivers/storage/port/storport/precomp.h
    let mut Destination: PCM_RESOURCE_LIST;
    let mut Size: ULONG;
    if DbgPrint(
        b"(%s:%d) CopyResourceList(%lu %p)\n\0".as_ptr() as *const i8,
        b"misc.c\0".as_ptr() as *const i8,
        99 as i32,
        PoolType as u32,
        Source,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"misc.c\0".as_ptr() as *const i8,
            99 as i32,
        );
    }
    Size = GetResourceListSize(Source);
    Destination = ExAllocatePoolWithTag(PoolType, Size as usize, TAG_RESOURCE_LIST) as PCM_RESOURCE_LIST;
    if Destination.is_null() {
        return core::ptr::null_mut();
    }
    RtlCopyMemory(
        Destination as *mut core::ffi::c_void,
        Source as *const core::ffi::c_void,
        Size as usize,
    );
    return Destination;
}

#[no_mangle]
pub unsafe extern "C" fn QueryBusInterface(
    DeviceObject: PDEVICE_OBJECT,
    Guid: PGUID,
    Size: USHORT,
    Version: USHORT,
    Interface: PBUS_INTERFACE_STANDARD,
    InterfaceSpecificData: PVOID,
) -> NTSTATUS {
    const IRP_MN_QUERY_INTERFACE: u32 = 0x08;

    let mut Event: KEVENT = core::mem::zeroed();
    let mut Status: NTSTATUS;
    let mut Irp: PIRP;
    let mut IoStatus: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Stack: PIO_STACK_LOCATION;

    KeInitializeEvent(core::ptr::addr_of_mut!(Event), _EVENT_TYPE_NotificationEvent, 0 as BOOLEAN);

    Irp = IoBuildSynchronousFsdRequest(
        IRP_MJ_PNP as ULONG,
        DeviceObject,
        core::ptr::null_mut(),
        0 as ULONG,
        core::ptr::null_mut(),
        core::ptr::addr_of_mut!(Event),
        core::ptr::addr_of_mut!(IoStatus),
    );
    if Irp.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    Stack = IoGetNextIrpStackLocation(Irp);

    (*Stack).MajorFunction = IRP_MJ_PNP as UCHAR;
    (*Stack).MinorFunction = IRP_MN_QUERY_INTERFACE as UCHAR;
    (*Stack).Parameters.QueryInterface.InterfaceType = Guid as *const GUID;
    (*Stack).Parameters.QueryInterface.Size = Size;
    (*Stack).Parameters.QueryInterface.Version = Version;
    (*Stack).Parameters.QueryInterface.Interface = Interface as PINTERFACE;
    (*Stack).Parameters.QueryInterface.InterfaceSpecificData = InterfaceSpecificData;

    (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_SUPPORTED;

    Status = IoCallDriver(DeviceObject, Irp);
    if Status == STATUS_PENDING {
        KeWaitForSingleObject(
            core::ptr::addr_of_mut!(Event) as PVOID,
            _KWAIT_REASON_Executive,
            0i32 as KPROCESSOR_MODE,
            0 as BOOLEAN,
            core::ptr::null_mut(),
        );

        Status = IoStatus.__bindgen_anon_1.Status;
    }

    return Status;
}
#[no_mangle]
pub unsafe extern "system" fn TranslateResourceListAddress(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut BusType: INTERFACE_TYPE,
    mut SystemIoBusNumber: ULONG,
    mut IoAddress: STOR_PHYSICAL_ADDRESS,
    mut NumberOfBytes: ULONG,
    mut InIoSpace: BOOLEAN,
    mut TranslatedAddress: PPHYSICAL_ADDRESS,
) -> BOOLEAN {
    const CmResourceTypePort: i32 = 1;
    const CmResourceTypeMemory: i32 = 3;

    let mut FullDescriptorA: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut FullDescriptorT: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialDescriptorA: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialDescriptorT: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut i: INT;
    let mut j: INT;

    if DbgPrint(
        b"(%s:%d) TranslateResourceListAddress(%p)\n\0".as_ptr() as *const i8,
        b"misc.c\0".as_ptr() as *const i8,
        185 as i32,
        DeviceExtension,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"misc.c\0".as_ptr() as *const i8,
            185 as i32,
        );
    }

    FullDescriptorA = core::ptr::addr_of_mut!((*(*DeviceExtension).AllocatedResources).List)
        as *mut CM_FULL_RESOURCE_DESCRIPTOR as PCM_FULL_RESOURCE_DESCRIPTOR;
    FullDescriptorT = core::ptr::addr_of_mut!((*(*DeviceExtension).TranslatedResources).List)
        as *mut CM_FULL_RESOURCE_DESCRIPTOR as PCM_FULL_RESOURCE_DESCRIPTOR;

    i = 0 as INT;
    while (i as u32) < (*(*DeviceExtension).AllocatedResources).Count {
        j = 0 as INT;
        while (j as u32) < (*FullDescriptorA).PartialResourceList.Count {
            PartialDescriptorA = (core::ptr::addr_of_mut!(
                (*FullDescriptorA).PartialResourceList.PartialDescriptors
            ) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(j as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
            PartialDescriptorT = (core::ptr::addr_of_mut!(
                (*FullDescriptorT).PartialResourceList.PartialDescriptors
            ) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(j as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

            match (*PartialDescriptorA).Type as i32 {
                CmResourceTypePort => {
                    if DbgPrint(
                        b"(%s:%d) Port: 0x%I64x (0x%lx)\n\0".as_ptr() as *const i8,
                        b"misc.c\0".as_ptr() as *const i8,
                        201 as i32,
                        (*PartialDescriptorA).u.Port.Start.QuadPart,
                        (*PartialDescriptorA).u.Port.Length,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                            b"misc.c\0".as_ptr() as *const i8,
                            201 as i32,
                        );
                    }
                    if InIoSpace as i32 != 0
                        && IoAddress.QuadPart >= (*PartialDescriptorA).u.Port.Start.QuadPart
                        && IoAddress.QuadPart as i64 + NumberOfBytes as i64
                            <= (*PartialDescriptorA).u.Port.Start.QuadPart as i64
                                + (*PartialDescriptorA).u.Port.Length as i64
                    {
                        (*TranslatedAddress).QuadPart = (*PartialDescriptorT).u.Port.Start.QuadPart
                            + (IoAddress.QuadPart - (*PartialDescriptorA).u.Port.Start.QuadPart);
                        return 1 as BOOLEAN;
                    }
                }
                CmResourceTypeMemory => {
                    if DbgPrint(
                        b"(%s:%d) Memory: 0x%I64x (0x%lx)\n\0".as_ptr() as *const i8,
                        b"misc.c\0".as_ptr() as *const i8,
                        215 as i32,
                        (*PartialDescriptorA).u.Memory.Start.QuadPart,
                        (*PartialDescriptorA).u.Memory.Length,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                            b"misc.c\0".as_ptr() as *const i8,
                            215 as i32,
                        );
                    }
                    if InIoSpace == 0
                        && IoAddress.QuadPart >= (*PartialDescriptorA).u.Memory.Start.QuadPart
                        && IoAddress.QuadPart as i64 + NumberOfBytes as i64
                            <= (*PartialDescriptorA).u.Memory.Start.QuadPart as i64
                                + (*PartialDescriptorA).u.Memory.Length as i64
                    {
                        (*TranslatedAddress).QuadPart = (*PartialDescriptorT).u.Memory.Start.QuadPart
                            + (IoAddress.QuadPart - (*PartialDescriptorA).u.Memory.Start.QuadPart);
                        return 1 as BOOLEAN;
                    }
                }
                _ => {}
            }
            j += 1;
        }

        FullDescriptorA = (core::ptr::addr_of_mut!(
            (*FullDescriptorA).PartialResourceList.PartialDescriptors
        ) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset((*FullDescriptorA).PartialResourceList.Count as isize)
            as PCM_FULL_RESOURCE_DESCRIPTOR;
        FullDescriptorT = (core::ptr::addr_of_mut!(
            (*FullDescriptorT).PartialResourceList.PartialDescriptors
        ) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset((*FullDescriptorT).PartialResourceList.Count as isize)
            as PCM_FULL_RESOURCE_DESCRIPTOR;
        i += 1;
    }

    return 0 as BOOLEAN;
}

#[no_mangle]
pub unsafe extern "C" fn GetResourceListInterrupt(
    mut DeviceExtension: PFDO_DEVICE_EXTENSION,
    mut Vector: PULONG,
    mut Irql: PKIRQL,
    mut InterruptMode: *mut KINTERRUPT_MODE,
    mut ShareVector: PBOOLEAN,
    mut Affinity: PKAFFINITY,
) -> NTSTATUS {
    const CmResourceTypeInterrupt: i32 = 2;
    const CM_RESOURCE_INTERRUPT_LATCHED: i32 = 0x0001;
    const TRUE: i32 = 1;
    const FALSE: i32 = 0;

    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut PartialDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR = core::ptr::null_mut();
    let mut i: INT;
    let mut j: INT;

    if DbgPrint(
        b"(%s:%d) GetResourceListInterrupt(%p)\n\0".as_ptr() as *const i8,
        b"misc.c\0".as_ptr() as *const i8,
        254 as i32,
        DeviceExtension,
    ) != 0
    {
        DbgPrint(
            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
            b"misc.c\0".as_ptr() as *const i8,
            254 as i32,
        );
    }

    FullDescriptor = core::ptr::addr_of_mut!((*(*DeviceExtension).TranslatedResources).List)
        as *mut CM_FULL_RESOURCE_DESCRIPTOR as PCM_FULL_RESOURCE_DESCRIPTOR;
    i = 0 as INT;
    while (i as u32) < (*(*DeviceExtension).TranslatedResources).Count {
        j = 0 as INT;
        while (j as u32) < (*FullDescriptor).PartialResourceList.Count {
            PartialDescriptor = (core::ptr::addr_of_mut!(
                (*FullDescriptor).PartialResourceList.PartialDescriptors
            ) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
                .offset(j as isize) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;
            match (*PartialDescriptor).Type as i32 {
                CmResourceTypeInterrupt => {
                    if DbgPrint(
                        b"(%s:%d) Interrupt: Level %lu  Vector %lu\n\0".as_ptr() as *const i8,
                        b"misc.c\0".as_ptr() as *const i8,
                        268 as i32,
                        (*PartialDescriptor).u.Interrupt.Level,
                        (*PartialDescriptor).u.Interrupt.Vector,
                    ) != 0
                    {
                        DbgPrint(
                            b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const i8,
                            b"misc.c\0".as_ptr() as *const i8,
                            268 as i32,
                        );
                    }
                    *Vector = (*PartialDescriptor).u.Interrupt.Vector as u32;
                    *Irql = (*PartialDescriptor).u.Interrupt.Level as KIRQL as u8;
                    *InterruptMode = (if (*PartialDescriptor).Flags as i32
                        & CM_RESOURCE_INTERRUPT_LATCHED
                        != 0
                    {
                        Latched as i32
                    } else {
                        LevelSensitive as i32
                    }) as KINTERRUPT_MODE;
                    *ShareVector = (if (*PartialDescriptor).ShareDisposition as i32
                        == CmResourceShareShared as i32
                    {
                        TRUE
                    } else {
                        FALSE
                    }) as u8;
                    *Affinity = (*PartialDescriptor).u.Interrupt.Affinity;
                    return STATUS_SUCCESS;
                }
                _ => {}
            }
            j += 1;
        }
        FullDescriptor = (core::ptr::addr_of_mut!(
            (*FullDescriptor).PartialResourceList.PartialDescriptors
        ) as *mut CM_PARTIAL_RESOURCE_DESCRIPTOR)
            .offset((*FullDescriptor).PartialResourceList.Count as isize)
            as PCM_FULL_RESOURCE_DESCRIPTOR;
        i += 1;
    }

    return STATUS_NOT_FOUND;
}
#[no_mangle]
pub unsafe extern "system" fn AllocateAddressMapping(
    MappedAddressList: *mut PMAPPED_ADDRESS,
    IoAddress: STOR_PHYSICAL_ADDRESS,
    MappedAddress: PVOID,
    NumberOfBytes: ULONG,
    BusNumber: ULONG,
) -> NTSTATUS {
    let mut Mapping: PMAPPED_ADDRESS = core::ptr::null_mut();

    // DPRINT1("AllocateAddressMapping()\n");

    Mapping = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<MAPPED_ADDRESS>() as SIZE_T,
        TAG_ADDRESS_MAPPING as ULONG,
    ) as PMAPPED_ADDRESS;
    if Mapping.is_null() {
        // DPRINT1("No memory!\n");
        return STATUS_NO_MEMORY;
    }

    RtlZeroMemory(
        Mapping as *mut core::ffi::c_void,
        core::mem::size_of::<MAPPED_ADDRESS>(),
    );

    (*Mapping).NextMappedAddress = *MappedAddressList;
    *MappedAddressList = Mapping;

    (*Mapping).IoAddress = IoAddress;
    (*Mapping).MappedAddress = MappedAddress;
    (*Mapping).NumberOfBytes = NumberOfBytes;
    (*Mapping).BusNumber = BusNumber;

    return STATUS_SUCCESS;
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Storport Driver
//  * LICENSE:     GPL-2.0+ (https://spdx.org/licenses/GPL-2.0+)
//  * PURPOSE:     Storport helper functions
//  * COPYRIGHT:   Copyright 2017 Eric Kohl (eric.kohl@reactos.org)
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
//
// /* FUNCTIONS ******************************************************************/
//
// NTSTATUS
// NTAPI
// ForwardIrpAndForget(
//      PDEVICE_OBJECT LowerDevice,
//      PIRP Irp)
// {
//     ASSERT(LowerDevice);
//
//     IoSkipCurrentIrpStackLocation(Irp);
//     return IoCallDriver(LowerDevice, Irp);
// }
//
//
// INTERFACE_TYPE
// GetBusInterface(
//     PDEVICE_OBJECT DeviceObject)
// {
//     GUID Guid;
//     ULONG Length;
//     NTSTATUS Status;
//
//     Status = IoGetDeviceProperty(DeviceObject,
//                                  DevicePropertyBusTypeGuid,
//                                  sizeof(Guid),
//                                  &Guid,
//                                  &Length);
//     if (!NT_SUCCESS(Status))
//         return InterfaceTypeUndefined;
//
//     if (RtlCompareMemory(&Guid, &GUID_BUS_TYPE_PCMCIA, sizeof(GUID)) == sizeof(GUID))
//         return PCMCIABus;
//     else if (RtlCompareMemory(&Guid, &GUID_BUS_TYPE_PCI, sizeof(GUID)) == sizeof(GUID))
//         return PCIBus;
//     else if (RtlCompareMemory(&Guid, &GUID_BUS_TYPE_ISAPNP, sizeof(GUID)) == sizeof(GUID))
//         return PNPISABus;
//
//     return InterfaceTypeUndefined;
// }
//
//
// static
// ULONG
// GetResourceListSize(
//     PCM_RESOURCE_LIST ResourceList)
// {
//     PCM_FULL_RESOURCE_DESCRIPTOR Descriptor;
//     ULONG Size;
//
//     DPRINT1("GetResourceListSize(%p)\n", ResourceList);
//
//     Size = sizeof(CM_RESOURCE_LIST);
//     if (ResourceList->Count == 0)
//     {
//         DPRINT1("Size: 0x%lx (%u)\n", Size, Size);
//         return Size;
//     }
//
//     DPRINT1("ResourceList->Count: %lu\n", ResourceList->Count);
//
//     Descriptor = &ResourceList->List[0];
//
//     DPRINT1("PartialResourceList->Count: %lu\n", Descriptor->PartialResourceList.Count);
//
//     /* Add the size of the partial descriptors */
//     if (Descriptor->PartialResourceList.Count > 1)
//         Size += (Descriptor->PartialResourceList.Count - 1) * sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR);
//
//     DPRINT1("Size: 0x%lx (%u)\n", Size, Size);
//     return Size;
// }
//
//
// PCM_RESOURCE_LIST
// CopyResourceList(
//     POOL_TYPE PoolType,
//     PCM_RESOURCE_LIST Source)
// {
//     PCM_RESOURCE_LIST Destination;
//     ULONG Size;
//
//     DPRINT1("CopyResourceList(%lu %p)\n",
//             PoolType, Source);
//
//     /* Get the size of the resource list */
//     Size = GetResourceListSize(Source);
//
//     /* Allocate a new buffer */
//     Destination = ExAllocatePoolWithTag(PoolType,
//                                         Size,
//                                         TAG_RESOURCE_LIST);
//     if (Destination == NULL)
//         return NULL;
//
//     /* Copy the resource list */
//     RtlCopyMemory(Destination,
//                   Source,
//                   Size);
//
//     return Destination;
// }
//
//
// NTSTATUS
// QueryBusInterface(
//     PDEVICE_OBJECT DeviceObject,
//     PGUID Guid,
//     USHORT Size,
//     USHORT Version,
//     PBUS_INTERFACE_STANDARD Interface,
//     PVOID InterfaceSpecificData)
// {
//     KEVENT Event;
//     NTSTATUS Status;
//     PIRP Irp;
//     IO_STATUS_BLOCK IoStatus;
//     PIO_STACK_LOCATION Stack;
//
//     KeInitializeEvent(&Event, NotificationEvent, FALSE);
//
//     Irp = IoBuildSynchronousFsdRequest(IRP_MJ_PNP,
//                                        DeviceObject,
//                                        NULL,
//                                        0,
//                                        NULL,
//                                        &Event,
//                                        &IoStatus);
//     if (Irp == NULL)
//         return STATUS_INSUFFICIENT_RESOURCES;
//
//     Stack = IoGetNextIrpStackLocation(Irp);
//
//     Stack->MajorFunction = IRP_MJ_PNP;
//     Stack->MinorFunction = IRP_MN_QUERY_INTERFACE;
//     Stack->Parameters.QueryInterface.InterfaceType = Guid;
//     Stack->Parameters.QueryInterface.Size = Size;
//     Stack->Parameters.QueryInterface.Version = Version;
//     Stack->Parameters.QueryInterface.Interface = (PINTERFACE)Interface;
//     Stack->Parameters.QueryInterface.InterfaceSpecificData = InterfaceSpecificData;
//
//     Irp->IoStatus.Status = STATUS_NOT_SUPPORTED;
//
//     Status = IoCallDriver(DeviceObject, Irp);
//     if (Status == STATUS_PENDING)
//     {
//         KeWaitForSingleObject(&Event, Executive, KernelMode, FALSE, NULL);
//
//         Status=IoStatus.Status;
//     }
//
//     return Status;
// }
//
//
// BOOLEAN
// TranslateResourceListAddress(
//     PFDO_DEVICE_EXTENSION DeviceExtension,
//     INTERFACE_TYPE BusType,
//     ULONG SystemIoBusNumber,
//     STOR_PHYSICAL_ADDRESS IoAddress,
//     ULONG NumberOfBytes,
//     BOOLEAN InIoSpace,
//     PPHYSICAL_ADDRESS TranslatedAddress)
// {
//     PCM_FULL_RESOURCE_DESCRIPTOR FullDescriptorA, FullDescriptorT;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptorA, PartialDescriptorT;
//     INT i, j;
//
//     DPRINT1("TranslateResourceListAddress(%p)\n", DeviceExtension);
//
//     FullDescriptorA = DeviceExtension->AllocatedResources->List;
//     FullDescriptorT = DeviceExtension->TranslatedResources->List;
//     for (i = 0; i < DeviceExtension->AllocatedResources->Count; i++)
//     {
//         for (j = 0; j < FullDescriptorA->PartialResourceList.Count; j++)
//         {
//             PartialDescriptorA = FullDescriptorA->PartialResourceList.PartialDescriptors + j;
//             PartialDescriptorT = FullDescriptorT->PartialResourceList.PartialDescriptors + j;
//
//             switch (PartialDescriptorA->Type)
//             {
//                 case CmResourceTypePort:
//                     DPRINT1("Port: 0x%I64x (0x%lx)\n",
//                             PartialDescriptorA->u.Port.Start.QuadPart,
//                             PartialDescriptorA->u.Port.Length);
//                     if (InIoSpace &&
//                         IoAddress.QuadPart >= PartialDescriptorA->u.Port.Start.QuadPart &&
//                         IoAddress.QuadPart + NumberOfBytes <= PartialDescriptorA->u.Port.Start.QuadPart + PartialDescriptorA->u.Port.Length)
//                     {
//                         TranslatedAddress->QuadPart = PartialDescriptorT->u.Port.Start.QuadPart +
//                                                       (IoAddress.QuadPart - PartialDescriptorA->u.Port.Start.QuadPart);
//                         return TRUE;
//                     }
//                     break;
//
//                 case CmResourceTypeMemory:
//                     DPRINT1("Memory: 0x%I64x (0x%lx)\n",
//                             PartialDescriptorA->u.Memory.Start.QuadPart,
//                             PartialDescriptorA->u.Memory.Length);
//                     if (!InIoSpace &&
//                         IoAddress.QuadPart >= PartialDescriptorA->u.Memory.Start.QuadPart &&
//                         IoAddress.QuadPart + NumberOfBytes <= PartialDescriptorA->u.Memory.Start.QuadPart + PartialDescriptorA->u.Memory.Length)
//                     {
//                         TranslatedAddress->QuadPart = PartialDescriptorT->u.Memory.Start.QuadPart +
//                                                       (IoAddress.QuadPart - PartialDescriptorA->u.Memory.Start.QuadPart);
//                         return TRUE;
//                     }
//                     break;
//             }
//         }
//
//         /* Advance to next CM_FULL_RESOURCE_DESCRIPTOR block in memory. */
//         FullDescriptorA = (PCM_FULL_RESOURCE_DESCRIPTOR)(FullDescriptorA->PartialResourceList.PartialDescriptors +
//                                                          FullDescriptorA->PartialResourceList.Count);
//
//         FullDescriptorT = (PCM_FULL_RESOURCE_DESCRIPTOR)(FullDescriptorT->PartialResourceList.PartialDescriptors +
//                                                          FullDescriptorT->PartialResourceList.Count);
//     }
//
//     return FALSE;
// }
//
//
// NTSTATUS
// GetResourceListInterrupt(
//     PFDO_DEVICE_EXTENSION DeviceExtension,
//     PULONG Vector,
//     PKIRQL Irql,
//     KINTERRUPT_MODE *InterruptMode,
//     PBOOLEAN ShareVector,
//     PKAFFINITY Affinity)
// {
//     PCM_FULL_RESOURCE_DESCRIPTOR FullDescriptor;
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR PartialDescriptor;
//     INT i, j;
//
//     DPRINT1("GetResourceListInterrupt(%p)\n",
//             DeviceExtension);
//
//     FullDescriptor = DeviceExtension->TranslatedResources->List;
//     for (i = 0; i < DeviceExtension->TranslatedResources->Count; i++)
//     {
//         for (j = 0; j < FullDescriptor->PartialResourceList.Count; j++)
//         {
//             PartialDescriptor = FullDescriptor->PartialResourceList.PartialDescriptors + j;
//
//             switch (PartialDescriptor->Type)
//             {
//                 case CmResourceTypeInterrupt:
//                     DPRINT1("Interrupt: Level %lu  Vector %lu\n",
//                             PartialDescriptor->u.Interrupt.Level,
//                             PartialDescriptor->u.Interrupt.Vector);
//
//                     *Vector = PartialDescriptor->u.Interrupt.Vector;
//                     *Irql = (KIRQL)PartialDescriptor->u.Interrupt.Level;
//                     *InterruptMode = (PartialDescriptor->Flags & CM_RESOURCE_INTERRUPT_LATCHED) ? Latched : LevelSensitive;
//                     *ShareVector = (PartialDescriptor->ShareDisposition == CmResourceShareShared) ? TRUE : FALSE;
//                     *Affinity = PartialDescriptor->u.Interrupt.Affinity;
//
//                     return STATUS_SUCCESS;
//             }
//         }
//
//         /* Advance to next CM_FULL_RESOURCE_DESCRIPTOR block in memory. */
//         FullDescriptor = (PCM_FULL_RESOURCE_DESCRIPTOR)(FullDescriptor->PartialResourceList.PartialDescriptors +
//                                                         FullDescriptor->PartialResourceList.Count);
//     }
//
//     return STATUS_NOT_FOUND;
// }
//
//
// NTSTATUS
// AllocateAddressMapping(
//     PMAPPED_ADDRESS *MappedAddressList,
//     STOR_PHYSICAL_ADDRESS IoAddress,
//     PVOID MappedAddress,
//     ULONG NumberOfBytes,
//     ULONG BusNumber)
// {
//     PMAPPED_ADDRESS Mapping;
//
//     DPRINT1("AllocateAddressMapping()\n");
//
//     Mapping = ExAllocatePoolWithTag(NonPagedPool,
//                                     sizeof(MAPPED_ADDRESS),
//                                     TAG_ADDRESS_MAPPING);
//     if (Mapping == NULL)
//     {
//         DPRINT1("No memory!\n");
//         return STATUS_NO_MEMORY;
//     }
//
//     RtlZeroMemory(Mapping, sizeof(MAPPED_ADDRESS));
//
//     Mapping->NextMappedAddress = *MappedAddressList;
//     *MappedAddressList = Mapping;
//
//     Mapping->IoAddress = IoAddress;
//     Mapping->MappedAddress = MappedAddress;
//     Mapping->NumberOfBytes = NumberOfBytes;
//     Mapping->BusNumber = BusNumber;
//
//     return STATUS_SUCCESS;
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
