use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{ACCESS_MASK, CreateFileTypeNone, GUID, _IO_STATUS_BLOCK, IoCreateFile, IoGetCurrentIrpStackLocation, LPGUID, LPWSTR, _OBJECT_ATTRIBUTES, PHANDLE, PIO_STACK_LOCATION, PIRP, PKEY_VALUE_PARTIAL_INFORMATION, RtlAppendUnicodeToString};
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const KSSTRING_TopologyNode: [::core::ffi::c_ushort; 39] = unsafe {
    ::core::mem::transmute::<
        [u8; 78],
        [::core::ffi::c_ushort; 39],
    >(
        *b"{\x000\x006\x002\x001\x000\x006\x001\0A\0-\0E\0E\x007\x005\0-\x001\x001\0D\x000\0-\0B\09\x001\x005\0-\x000\x000\0A\x000\0C\09\x002\x002\x003\x001\09\x006\0}\0\0\0",
    )
};

#[repr(C)]
pub struct _TOPO_ENTRY {
    pub TopoId: ULONG,
    pub ParentId: ULONG,
    pub Depth: ULONG,
    pub NodeType: UCHAR,
    pub Reserved: [UCHAR; 3],
}

pub type PTOPO_ENTRY = *mut _TOPO_ENTRY;

#[repr(C)]
pub struct _TOPO_TABLE {
    pub Count: ULONG,
    pub Entries: PTOPO_ENTRY,
}

pub type PTOPO_TABLE = *mut _TOPO_TABLE;

pub const TOPO_ENTRY_TYPE_PROCESSOR: UCHAR = 0;
pub const TOPO_ENTRY_TYPE_CACHE: UCHAR = 1;
pub const TOPO_ENTRY_TYPE_GROUP: UCHAR = 2;
pub const TOPO_ENTRY_TYPE_NODE: UCHAR = 3;
pub const TOPO_ENTRY_TYPE_DIE: UCHAR = 4;
pub const TOPO_ENTRY_TYPE_PACKAGE: UCHAR = 5;
pub const TOPO_ENTRY_TYPE_SYSTEM: UCHAR = 6;

pub const TOPO_FLAG_NONE: ULONG = 0;
pub const TOPO_FLAG_THREAD: ULONG = 0x1;
pub const TOPO_FLAG_SMT: ULONG = 0x2;
pub const TOPO_FLAG_CORE: ULONG = 0x4;
pub const TOPO_FLAG_L1: ULONG = 0x8;
pub const TOPO_FLAG_L2: ULONG = 0x10;
pub const TOPO_FLAG_L3: ULONG = 0x20;
pub const TOPO_FLAG_NODE: ULONG = 0x40;
pub const TOPO_FLAG_DIE: ULONG = 0x80;
pub const TOPO_FLAG_PACKAGE: ULONG = 0x100;
pub const TOPO_FLAG_SYSTEM: ULONG = 0x200;
pub const TOPO_FLAG_ALL: ULONG = 0x3FF;

pub const TOPO_MAX_DEPTH: ULONG = 8;
pub const TOPO_MAX_ENTRIES: ULONG = 256;

#[no_mangle]
pub unsafe extern "system" fn KspCreateObjectType(
    ParentHandle: HANDLE,
    ObjectType: LPWSTR,
    CreateParameters: PVOID,
    CreateParametersSize: UINT,
    DesiredAccess: ACCESS_MASK,
    NodeHandle: PHANDLE,
) -> NTSTATUS {
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut Name: _UNICODE_STRING = _UNICODE_STRING {
        Length: 0,
        MaximumLength: 0,
        Buffer: core::ptr::null_mut(),
    };

    Name.Length = 0;
    Name.MaximumLength = (wcslen(ObjectType) * core::mem::size_of::<WCHAR>() as usize)
        .wrapping_add(CreateParametersSize as usize)
        .wrapping_add(core::mem::size_of::<WCHAR>()) as USHORT;
    Name.MaximumLength = (Name.MaximumLength as usize)
        .wrapping_add(core::mem::size_of::<WCHAR>()) as USHORT;

    Name.Buffer = AllocateItem(NonPagedPool, Name.MaximumLength as usize) as PWCH;
    if Name.Buffer.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    RtlAppendUnicodeToString(&mut Name, ObjectType);
    RtlAppendUnicodeToString(&mut Name, L"\\\0".as_ptr() as PCWSTR);

    core::ptr::copy_nonoverlapping(
        CreateParameters as *const u8,
        Name.Buffer.offset((Name.Length as usize / core::mem::size_of::<WCHAR>()) as isize) as *mut u8,
        CreateParametersSize as usize,
    );
    Name.Length = (Name.Length as u32).wrapping_add(CreateParametersSize) as USHORT;
    *Name.Buffer.offset((Name.Length as i32 / 2) as isize) = 0u16;

    ObjectAttributes.Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32;
    ObjectAttributes.RootDirectory = ParentHandle;
    ObjectAttributes.ObjectName = &mut Name;
    ObjectAttributes.Attributes = (OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE | OBJ_OPENIF) as u32;
    ObjectAttributes.SecurityDescriptor = core::ptr::null_mut();
    ObjectAttributes.SecurityQualityOfService = core::ptr::null_mut();

    let Status = IoCreateFile(
        NodeHandle,
        DesiredAccess,
        &mut ObjectAttributes,
        &mut IoStatusBlock,
        core::ptr::null_mut(),
        0,
        0,
        FILE_OPEN,
        0,
        core::ptr::null_mut(),
        0,
        CreateFileTypeNone,
        core::ptr::null_mut(),
        (IO_NO_PARAMETER_CHECKING | IO_FORCE_ACCESS_CHECK) as u32,
    );

    FreeItem(Name.Buffer as PVOID);
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsCreateTopologyNode(
    ParentHandle: HANDLE,
    NodeCreate: PKSNODE_CREATE,
    DesiredAccess: ACCESS_MASK,
    NodeHandle: PHANDLE,
) -> NTSTATUS {
    return KspCreateObjectType(
        ParentHandle,
        KSSTRING_TopologyNode.as_ptr() as LPWSTR,
        NodeCreate as PVOID,
        core::mem::size_of::<KSNODE_CREATE>() as UINT,
        DesiredAccess,
        NodeHandle,
    );
}

#[no_mangle]
pub unsafe extern "system" fn KsValidateTopologyNodeCreateRequest(
    Irp: PIRP,
    Topology: PKSTOPOLOGY,
    OutNodeCreate: *mut PKSNODE_CREATE,
) -> NTSTATUS {
    let mut NodeCreate: PKSNODE_CREATE = core::ptr::null_mut();
    let mut Size: ULONG = 0;
    let mut Status: NTSTATUS;

    if Topology.is_null() {
        return STATUS_INVALID_PARAMETER;
    }

    Size = core::mem::size_of::<KSNODE_CREATE>() as ULONG;

    Status = KspCopyCreateRequest(
        Irp,
        KSSTRING_TopologyNode.as_ptr() as LPWSTR,
        &mut Size as *mut ULONG,
        &mut NodeCreate as *mut PKSNODE_CREATE as *mut PVOID,
    );

    if !NT_SUCCESS(Status) {
        return Status;
    }

    if (*NodeCreate).CreateFlags != 0
        || ((*NodeCreate).Node >= (*Topology).TopologyNodesCount
            && (*NodeCreate).Node != MAXULONG)
    {
        FreeItem(NodeCreate as PVOID);
        return STATUS_UNSUCCESSFUL;
    }

    *OutNodeCreate = NodeCreate;
    Status
}

#[no_mangle]
pub unsafe extern "system" fn KsTopologyPropertyHandler(
    Irp: PIRP,
    Property: PKSPROPERTY,
    Data: PVOID,
    Topology: *const KSTOPOLOGY,
) -> NTSTATUS {
    let mut Node: *mut KSP_NODE = core::ptr::null_mut();
    let IoStack: PIO_STACK_LOCATION;
    let mut Status: NTSTATUS;
    let mut KeyInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut Guid: LPGUID = core::ptr::null_mut();

    IoStack = IoGetCurrentIrpStackLocation(Irp);

    if (*Property).Flags != KSPROPERTY_TYPE_GET as u32 {
        (*Irp).IoStatus.__bindgen_anon_1.Status = STATUS_NOT_IMPLEMENTED;
        (*Irp).IoStatus.Information = 0;
        return STATUS_NOT_IMPLEMENTED;
    }

    match (*Property).Id {
        0 => {
            return KsHandleSizedListQuery(
                Irp,
                (*Topology).CategoriesCount,
                core::mem::size_of::<GUID>() as u32,
                (*Topology).Categories as *const core::ffi::c_void,
            );
        }
        1 => {
            return KsHandleSizedListQuery(
                Irp,
                (*Topology).TopologyNodesCount,
                core::mem::size_of::<GUID>() as u32,
                (*Topology).TopologyNodes as *const core::ffi::c_void,
            );
        }
        2 => {
            return KsHandleSizedListQuery(
                Irp,
                (*Topology).TopologyConnectionsCount,
                core::mem::size_of::<KSTOPOLOGY_CONNECTION>() as u32,
                (*Topology).TopologyConnections as *const core::ffi::c_void,
            );
        }
        3 => {
            Node = Property as *mut KSP_NODE;
            if (*Node).NodeId >= (*Topology).TopologyNodesCount {
                (*Irp).IoStatus.Information = 0;
                Status = STATUS_INVALID_PARAMETER;
            } else {
                if !IsEqualGUIDAligned(
                    &(*Topology).TopologyNodesNames.offset((*Node).NodeId as isize) as *const _ as *const GUID,
                    &GUID_NULL as *const _ as *const GUID,
                ) {
                    Guid = (*Topology).TopologyNodesNames.offset((*Node).NodeId as isize) as *mut GUID;
                } else {
                    Guid = (*Topology).TopologyNodes.offset((*Node).NodeId as isize) as *mut GUID;
                }

                Status = KspReadMediaCategory(Guid, &mut KeyInfo);
                if !NT_SUCCESS(Status) {
                    (*Irp).IoStatus.Information = 0;
                } else {
                    (*Irp).IoStatus.Information = ((*KeyInfo).DataLength as usize)
                        .wrapping_add(core::mem::size_of::<u16>())
                        as isize;

                    if ((*KeyInfo).DataLength as usize)
                        .wrapping_add(core::mem::size_of::<u16>())
                        > (*IoStack).Parameters.DeviceIoControl.OutputBufferLength as usize
                    {
                        Status = STATUS_BUFFER_OVERFLOW;
                        FreeItem(KeyInfo as *mut core::ffi::c_void);
                    } else {
                        core::ptr::copy_nonoverlapping(
                            &(*KeyInfo).Data as *const _ as *const u8,
                            (*Irp).AssociatedIrp.SystemBuffer as *mut u8,
                            (*KeyInfo).DataLength as usize,
                        );
                        *((*Irp).AssociatedIrp.SystemBuffer as *mut u16)
                            .offset((*KeyInfo).DataLength as isize / 2) = 0u16;
                        FreeItem(KeyInfo as *mut core::ffi::c_void);
                    }
                }
            }
        }
        _ => {
            (*Irp).IoStatus.Information = 0;
            Status = STATUS_NOT_FOUND;
        }
    }

    (*Irp).IoStatus.__bindgen_anon_1.Status = Status;
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * COPYRIGHT:       See COPYING in the top level directory
//  * PROJECT:         ReactOS Kernel Streaming
//  * FILE:            drivers/ksfilter/ks/topoology.c
//  * PURPOSE:         KS Allocator functions
//  * PROGRAMMER:      Johannes Anderwald
//  */
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// NTAPI
// KspCreateObjectType(
//     IN HANDLE ParentHandle,
//     IN LPWSTR ObjectType,
//     PVOID CreateParameters,
//     UINT CreateParametersSize,
//     IN  ACCESS_MASK DesiredAccess,
//     OUT PHANDLE NodeHandle)
// {
//     NTSTATUS Status;
//     IO_STATUS_BLOCK IoStatusBlock;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//     UNICODE_STRING Name;
//
//     /* calculate request length */
//     Name.Length = 0;
//     Name.MaximumLength = (USHORT)(wcslen(ObjectType) * sizeof(WCHAR) + CreateParametersSize +  1 * sizeof(WCHAR));
//     Name.MaximumLength += sizeof(WCHAR);
//     /* acquire request buffer */
//     Name.Buffer = AllocateItem(NonPagedPool, Name.MaximumLength);
//     /* check for success */
//     if (!Name.Buffer)
//     {
//         /* insufficient resources */
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* build a request which looks like {ObjectClass}\CreateParameters
//      * For pins the parent is the reference string used in registration
//      * For clocks it is full path for pin\{ClockGuid}\ClockCreateParams
//      */
//     RtlAppendUnicodeToString(&Name, ObjectType);
//     RtlAppendUnicodeToString(&Name, L"\\");
//     /* append create parameters */
//     RtlMoveMemory(Name.Buffer + (Name.Length / sizeof(WCHAR)), CreateParameters, CreateParametersSize);
//     Name.Length += CreateParametersSize;
//     Name.Buffer[Name.Length / 2] = L'\0';
//
//     InitializeObjectAttributes(&ObjectAttributes, &Name, OBJ_KERNEL_HANDLE | OBJ_CASE_INSENSITIVE | OBJ_OPENIF, ParentHandle, NULL);
//     /* create the instance */
//     Status = IoCreateFile(NodeHandle,
//                           DesiredAccess,
//                           &ObjectAttributes,
//                           &IoStatusBlock,
//                           NULL,
//                           0,
//                           0,
//                           FILE_OPEN,
//                           0,
//                           NULL,
//                           0,
//                           CreateFileTypeNone,
//                           NULL,
//                           IO_NO_PARAMETER_CHECKING | IO_FORCE_ACCESS_CHECK);
//
//     /* free request buffer */
//     FreeItem(Name.Buffer);
//     return Status;
// }
//
//
// /*
//     @implemented
// */
// KSDDKAPI NTSTATUS NTAPI
// KsCreateTopologyNode(
//     IN  HANDLE ParentHandle,
//     IN  PKSNODE_CREATE NodeCreate,
//     IN  ACCESS_MASK DesiredAccess,
//     OUT PHANDLE NodeHandle)
// {
//     return KspCreateObjectType(ParentHandle,
//                                KSSTRING_TopologyNode,
//                                (PVOID)NodeCreate,
//                                sizeof(KSNODE_CREATE),
//                                DesiredAccess,
//                                NodeHandle);
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsValidateTopologyNodeCreateRequest(
//     IN  PIRP Irp,
//     IN  PKSTOPOLOGY Topology,
//     OUT PKSNODE_CREATE* OutNodeCreate)
// {
//     PKSNODE_CREATE NodeCreate;
//     ULONG Size;
//     NTSTATUS Status;
//
//     /* did the caller miss the topology */
//     if (!Topology)
//         return STATUS_INVALID_PARAMETER;
//
//     /* set create param  size */
//     Size = sizeof(KSNODE_CREATE);
//
//     /* fetch create parameters */
//     Status = KspCopyCreateRequest(Irp,
//                                   KSSTRING_TopologyNode,
//                                   &Size,
//                                   (PVOID*)&NodeCreate);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (NodeCreate->CreateFlags != 0 || (NodeCreate->Node >= Topology->TopologyNodesCount && NodeCreate->Node != MAXULONG))
//     {
//         /* invalid node create */
//         FreeItem(NodeCreate);
//         return STATUS_UNSUCCESSFUL;
//     }
//
//     /* store result */
//     *OutNodeCreate = NodeCreate;
//
//     return Status;
// }
//
// /*
//     @implemented
// */
// KSDDKAPI
// NTSTATUS
// NTAPI
// KsTopologyPropertyHandler(
//     IN  PIRP Irp,
//     IN  PKSPROPERTY Property,
//     IN  OUT PVOID Data,
//     IN  const KSTOPOLOGY* Topology)
// {
//     KSP_NODE * Node;
//     PIO_STACK_LOCATION IoStack;
//     NTSTATUS Status;
//     PKEY_VALUE_PARTIAL_INFORMATION KeyInfo;
//     LPGUID Guid;
//
//     IoStack = IoGetCurrentIrpStackLocation(Irp);
//
//     DPRINT("KsTopologyPropertyHandler Irp %p Property %p Data %p Topology %p OutputLength %lu PropertyId %lu\n", Irp, Property, Data, Topology, IoStack->Parameters.DeviceIoControl.OutputBufferLength, Property->Id);
//
//     if (Property->Flags != KSPROPERTY_TYPE_GET)
//     {
//         Irp->IoStatus.Status = STATUS_NOT_IMPLEMENTED;
//         Irp->IoStatus.Information = 0;
//         return STATUS_NOT_IMPLEMENTED;
//     }
//
//     switch(Property->Id)
//     {
//         case KSPROPERTY_TOPOLOGY_CATEGORIES:
//             return KsHandleSizedListQuery(Irp, Topology->CategoriesCount, sizeof(GUID), Topology->Categories);
//
//         case KSPROPERTY_TOPOLOGY_NODES:
//             return KsHandleSizedListQuery(Irp, Topology->TopologyNodesCount, sizeof(GUID), Topology->TopologyNodes);
//
//         case KSPROPERTY_TOPOLOGY_CONNECTIONS:
//             return KsHandleSizedListQuery(Irp, Topology->TopologyConnectionsCount, sizeof(KSTOPOLOGY_CONNECTION), Topology->TopologyConnections);
//
//         case KSPROPERTY_TOPOLOGY_NAME:
//             Node = (KSP_NODE*)Property;
//
//             /* check for invalid node id */
//             if (Node->NodeId >= Topology->TopologyNodesCount)
//             {
//                 /* invalid node id */
//                 Irp->IoStatus.Information = 0;
//                 Status = STATUS_INVALID_PARAMETER;
//                 break;
//             }
//
//             /* check if there is a name supplied */
//             if (!IsEqualGUIDAligned(&Topology->TopologyNodesNames[Node->NodeId], &GUID_NULL))
//             {
//                 /* node name has been supplied */
//                 Guid = (LPGUID)&Topology->TopologyNodesNames[Node->NodeId];
//             }
//             else
//             {
//                 /* fallback to topology node type */
//                 Guid = (LPGUID)&Topology->TopologyNodes[Node->NodeId];
//             }
//
//             /* read topology node name */
//             Status = KspReadMediaCategory(Guid, &KeyInfo);
//             if (!NT_SUCCESS(Status))
//             {
//                 Irp->IoStatus.Information = 0;
//                 break;
//             }
//
//             /* store result size */
//             Irp->IoStatus.Information = KeyInfo->DataLength + sizeof(WCHAR);
//
//             /* check for buffer overflow */
//             if (KeyInfo->DataLength + sizeof(WCHAR) > IoStack->Parameters.DeviceIoControl.OutputBufferLength)
//             {
//                 /* buffer too small */
//                 Status = STATUS_BUFFER_OVERFLOW;
//                 FreeItem(KeyInfo);
//                 break;
//             }
//
//             /* copy result buffer */
//             RtlMoveMemory(Irp->AssociatedIrp.SystemBuffer, &KeyInfo->Data, KeyInfo->DataLength);
//
//             /* zero terminate it */
//             ((LPWSTR)Irp->AssociatedIrp.SystemBuffer)[KeyInfo->DataLength / sizeof(WCHAR)] = L'\0';
//
//             /* free key info */
//             FreeItem(KeyInfo);
//
//             break;
//         default:
//              Irp->IoStatus.Information = 0;
//            Status = STATUS_NOT_FOUND;
//     }
//
//     Irp->IoStatus.Status = Status;
//     return Status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
