use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ke::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::ob::*;
use ntkrnl_sys::interlocked::*;

use ntkrnl_sys::ddk::{
    _DEVICE_NODE, DEVICE_NODE, PDEVICE_NODE, PDEVICE_OBJECT, PEXTENDED_DEVOBJ_EXTENSION,
    PUNICODE_STRING, PDEVICETREE_TRAVERSE_CONTEXT, UINT32,
    IopCreateDeviceKeyPath, IopFreeDeviceNode as IopFreeDeviceNodeExtern,
    IopGetDeviceNode as IopGetDeviceNodeExtern, IopTraverseDeviceTree as IopTraverseDeviceTreeExtern,
    RtlInitUnicodeString, RtlCopyUnicodeString, RtlAppendUnicodeStringToString,
    RtlUpcaseUnicodeString, RtlFreeUnicodeString, ZwClose, ZwSetValueKey,
    ExAllocatePool, ExAllocatePoolZero, ExFreePool,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const TAG_IO_DEVNODE: u32 = 0x646F6E44;

#[no_mangle]
#[used]
pub static mut IopRootDeviceNode: PDEVICE_NODE = core::ptr::null_mut();

#[no_mangle]
#[used]
pub static mut IopDeviceTreeLock: KSPIN_LOCK = 0;

#[no_mangle]
#[used]
pub static mut IopNumberDeviceNodes: i32 = 0;

fastcall_fn! {
#[no_mangle]
pub unsafe fn IopGetDeviceNode(
    DeviceObject: PDEVICE_OBJECT,
) -> PDEVICE_NODE {
    let DeviceObject = opaque_ptr(DeviceObject);
    (*((*DeviceObject).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION)).DeviceNode
}
}

#[no_mangle]
pub unsafe extern "C" fn PipAllocateDeviceNode(
    PhysicalDeviceObject: PDEVICE_OBJECT,
) -> PDEVICE_NODE {
    let PhysicalDeviceObject = opaque_ptr(PhysicalDeviceObject);
    let DeviceNode = ExAllocatePoolZero(
        NonPagedPool,
        core::mem::size_of::<DEVICE_NODE>() as _,
        TAG_IO_DEVNODE,
    ) as PDEVICE_NODE;
    if DeviceNode.is_null() {
        return core::ptr::null_mut();
    }

    InterlockedIncrement(core::ptr::addr_of_mut!(IopNumberDeviceNodes));

    (*DeviceNode).InterfaceType = -1i32;
    (*DeviceNode).BusNumber = -1i32 as u32;
    (*DeviceNode).ChildInterfaceType = -1i32;
    (*DeviceNode).ChildBusNumber = -1i32 as u32;
    (*DeviceNode).ChildBusTypeIndex = 0xFFFF;
    (*DeviceNode).State = 0x301;
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceNode).DeviceArbiterList));
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceNode).DeviceTranslatorList));
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceNode).TargetDeviceNotify));
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceNode).DockInfo.ListEntry));
    InitializeListHead(core::ptr::addr_of_mut!((*DeviceNode).PendedSetInterfaceState));

    if !PhysicalDeviceObject.is_null() {
        (*DeviceNode).PhysicalDeviceObject = PhysicalDeviceObject;
        (*((*PhysicalDeviceObject).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION)).DeviceNode = DeviceNode;
        (*PhysicalDeviceObject).Flags &= !0x00000080;
    }

    DeviceNode
}

#[no_mangle]
pub unsafe extern "C" fn PiInsertDevNode(
    DeviceNode: PDEVICE_NODE,
    ParentNode: PDEVICE_NODE,
) {
    let mut oldIrql: KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!(IopDeviceTreeLock), &mut oldIrql as *mut KIRQL);
    (*DeviceNode).Parent = ParentNode;
    (*DeviceNode).Sibling = core::ptr::null_mut();
    if (*ParentNode).LastChild.is_null() {
        (*ParentNode).Child = DeviceNode;
        (*ParentNode).LastChild = DeviceNode;
    } else {
        (*(*ParentNode).LastChild).Sibling = DeviceNode;
        (*ParentNode).LastChild = DeviceNode;
    }
    KeReleaseSpinLock(core::ptr::addr_of_mut!(IopDeviceTreeLock), oldIrql);
    (*DeviceNode).Level = (*ParentNode).Level + 1;
}

#[no_mangle]
pub unsafe extern "C" fn PiSetDevNodeState(
    DeviceNode: PDEVICE_NODE,
    NewState: UINT32,
) -> UINT32 {
    let mut oldIrql: KIRQL = 0;

    KeAcquireSpinLock(core::ptr::addr_of_mut!(IopDeviceTreeLock), &mut oldIrql as *mut KIRQL);

    let prevState = (*DeviceNode).State;
    if prevState != NewState {
        (*DeviceNode).State = NewState;
        (*DeviceNode).PreviousState = prevState;
        (*DeviceNode).StateHistory[(*DeviceNode).StateHistoryEntry as usize] = prevState;
        (*DeviceNode).StateHistoryEntry = ((*DeviceNode).StateHistoryEntry + 1) % 20;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!(IopDeviceTreeLock), oldIrql);

    prevState
}

#[no_mangle]
pub unsafe extern "C" fn PiSetDevNodeProblem(
    DeviceNode: PDEVICE_NODE,
    Problem: UINT32,
) {
    (*DeviceNode).Flags |= 0x00000800;
    (*DeviceNode).Problem = Problem;
}

#[no_mangle]
pub unsafe extern "C" fn PiClearDevNodeProblem(
    DeviceNode: PDEVICE_NODE,
) {
    (*DeviceNode).Flags &= !0x00000800;
    (*DeviceNode).Problem = 0;
}

#[no_mangle]
pub unsafe extern "C" fn IopFreeDeviceNode(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    let mut OldIrql: KIRQL = 0;
    let mut PrevSibling: PDEVICE_NODE = core::ptr::null_mut();

    KeAcquireSpinLock(core::ptr::addr_of_mut!(IopDeviceTreeLock), &mut OldIrql as *mut KIRQL);

    if !(*DeviceNode).Parent.is_null() && (*(*DeviceNode).Parent).Child != DeviceNode {
        PrevSibling = (*(*DeviceNode).Parent).Child;
        while (*PrevSibling).Sibling != DeviceNode {
            PrevSibling = (*PrevSibling).Sibling;
        }
    }

    if !(*DeviceNode).Parent.is_null() {
        if (*(*DeviceNode).Parent).LastChild == DeviceNode {
            (*(*DeviceNode).Parent).LastChild = PrevSibling;
            if !PrevSibling.is_null() {
                (*PrevSibling).Sibling = core::ptr::null_mut();
            }
        }
        if (*(*DeviceNode).Parent).Child == DeviceNode {
            (*(*DeviceNode).Parent).Child = (*DeviceNode).Sibling;
        }
    }

    if !PrevSibling.is_null() {
        (*PrevSibling).Sibling = (*DeviceNode).Sibling;
    }

    KeReleaseSpinLock(core::ptr::addr_of_mut!(IopDeviceTreeLock), OldIrql);

    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*DeviceNode).InstancePath));

    RtlFreeUnicodeString(core::ptr::addr_of_mut!((*DeviceNode).ServiceName));

    if !(*DeviceNode).ResourceList.is_null() {
        ExFreePool((*DeviceNode).ResourceList as *mut core::ffi::c_void);
    }

    crate::pnpres::IopDiscardTranslatedResources(DeviceNode);

    if !(*DeviceNode).ResourceRequirements.is_null() {
        ExFreePool((*DeviceNode).ResourceRequirements as *mut core::ffi::c_void);
    }

    if !(*DeviceNode).BootResources.is_null() {
        ExFreePool((*DeviceNode).BootResources as *mut core::ffi::c_void);
    }

    (*((*(*DeviceNode).PhysicalDeviceObject).DeviceObjectExtension as PEXTENDED_DEVOBJ_EXTENSION)).DeviceNode = core::ptr::null_mut();
    ExFreePoolWithTag(DeviceNode as *mut core::ffi::c_void, TAG_IO_DEVNODE);

    STATUS_SUCCESS
}

unsafe fn IopFindNextDeviceNodeForTraversal(
    Context: PDEVICETREE_TRAVERSE_CONTEXT,
) -> NTSTATUS {
    if !(*(*Context).DeviceNode).Child.is_null() {
        (*Context).DeviceNode = (*(*Context).DeviceNode).Child;
        return STATUS_SUCCESS;
    }

    while (*Context).DeviceNode != (*Context).FirstDeviceNode {
        if !(*(*Context).DeviceNode).Sibling.is_null() {
            (*Context).DeviceNode = (*(*Context).DeviceNode).Sibling;
            return STATUS_SUCCESS;
        }

        (*Context).DeviceNode = (*(*Context).DeviceNode).Parent;
    }

    0xC0000001u32 as NTSTATUS
}

extern "C" {
}

#[no_mangle]
pub unsafe extern "C" fn IopTraverseDeviceTree(
    Context: PDEVICETREE_TRAVERSE_CONTEXT,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let mut DeviceNode: PDEVICE_NODE;

    (*Context).DeviceNode = (*Context).FirstDeviceNode;

    loop {
        DeviceNode = (*Context).DeviceNode;
        if DeviceNode.is_null() {
            return STATUS_SUCCESS;
        }

        if !(*DeviceNode).PhysicalDeviceObject.is_null() {
            ObReferenceObject((*DeviceNode).PhysicalDeviceObject as *mut core::ffi::c_void);
        }

        Status = ((*Context).Action.unwrap())(DeviceNode, (*Context).Context);
        if Status >= 0 {
            Status = IopFindNextDeviceNodeForTraversal(Context);
        }

        if !(*DeviceNode).PhysicalDeviceObject.is_null() {
            ObDereferenceObject((*DeviceNode).PhysicalDeviceObject as *mut core::ffi::c_void);
        }

        if Status < 0 {
            break;
        }
    }

    if Status == 0xC0000001u32 as NTSTATUS {
        Status = STATUS_SUCCESS;
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:     ReactOS Kernel
//  * LICENSE:     GPL-2.0-or-later (https://spdx.org/licenses/GPL-2.0-or-later)
//  * PURPOSE:     PnP manager device tree functions
//  * COPYRIGHT:   Casper S. Hornstrup (chorns@users.sourceforge.net)
//  *              2007 Hervé Poussineau (hpoussin@reactos.org)
//  *              2010-2012 Cameron Gutman (cameron.gutman@reactos.org)
//  */
//
// /* INCLUDES ******************************************************************/
//
// #include <ntoskrnl.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS *******************************************************************/
//
// PDEVICE_NODE IopRootDeviceNode;
// KSPIN_LOCK IopDeviceTreeLock;
//
// LONG IopNumberDeviceNodes;
//
// /* FUNCTIONS *****************************************************************/
//
// PDEVICE_NODE
// FASTCALL
// IopGetDeviceNode(
//     _In_ PDEVICE_OBJECT DeviceObject)
// {
//     return ((PEXTENDED_DEVOBJ_EXTENSION)DeviceObject->DeviceObjectExtension)->DeviceNode;
// }
//
// PDEVICE_NODE
// PipAllocateDeviceNode(
//     _In_opt_ PDEVICE_OBJECT PhysicalDeviceObject)
// {
//     PDEVICE_NODE DeviceNode;
//     PAGED_CODE();
//
//     /* Allocate it */
//     DeviceNode = ExAllocatePoolZero(NonPagedPool, sizeof(DEVICE_NODE), TAG_IO_DEVNODE);
//     if (!DeviceNode)
//     {
//         return NULL;
//     }
//
//     /* Statistics */
//     InterlockedIncrement(&IopNumberDeviceNodes);
//
//     /* Set it up */
//     DeviceNode->InterfaceType = InterfaceTypeUndefined;
//     DeviceNode->BusNumber = -1;
//     DeviceNode->ChildInterfaceType = InterfaceTypeUndefined;
//     DeviceNode->ChildBusNumber = -1;
//     DeviceNode->ChildBusTypeIndex = -1;
//     DeviceNode->State = DeviceNodeUninitialized;
// //    KeInitializeEvent(&DeviceNode->EnumerationMutex, SynchronizationEvent, TRUE);
//     InitializeListHead(&DeviceNode->DeviceArbiterList);
//     InitializeListHead(&DeviceNode->DeviceTranslatorList);
//     InitializeListHead(&DeviceNode->TargetDeviceNotify);
//     InitializeListHead(&DeviceNode->DockInfo.ListEntry);
//     InitializeListHead(&DeviceNode->PendedSetInterfaceState);
//
//     /* Check if there is a PDO */
//     if (PhysicalDeviceObject)
//     {
//         /* Link it and remove the init flag */
//         DeviceNode->PhysicalDeviceObject = PhysicalDeviceObject;
//         ((PEXTENDED_DEVOBJ_EXTENSION)PhysicalDeviceObject->DeviceObjectExtension)->DeviceNode = DeviceNode;
//         PhysicalDeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//     }
//
//     DPRINT("Allocated devnode 0x%p\n", DeviceNode);
//
//     /* Return the node */
//     return DeviceNode;
// }
//
// VOID
// PiInsertDevNode(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ PDEVICE_NODE ParentNode)
// {
//     KIRQL oldIrql;
//
//     ASSERT(DeviceNode->Parent == NULL);
//
//     KeAcquireSpinLock(&IopDeviceTreeLock, &oldIrql);
//     DeviceNode->Parent = ParentNode;
//     DeviceNode->Sibling = NULL;
//     if (ParentNode->LastChild == NULL)
//     {
//         ParentNode->Child = DeviceNode;
//         ParentNode->LastChild = DeviceNode;
//     }
//     else
//     {
//         ParentNode->LastChild->Sibling = DeviceNode;
//         ParentNode->LastChild = DeviceNode;
//     }
//     KeReleaseSpinLock(&IopDeviceTreeLock, oldIrql);
//     DeviceNode->Level = ParentNode->Level + 1;
//
//     DPRINT("Inserted devnode 0x%p to parent 0x%p\n", DeviceNode, ParentNode);
// }
//
// PNP_DEVNODE_STATE
// PiSetDevNodeState(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ PNP_DEVNODE_STATE NewState)
// {
//     KIRQL oldIrql;
//
//     KeAcquireSpinLock(&IopDeviceTreeLock, &oldIrql);
//
//     PNP_DEVNODE_STATE prevState = DeviceNode->State;
//     if (prevState != NewState)
//     {
//         DeviceNode->State = NewState;
//         DeviceNode->PreviousState = prevState;
//         DeviceNode->StateHistory[DeviceNode->StateHistoryEntry++] = prevState;
//         DeviceNode->StateHistoryEntry %= DEVNODE_HISTORY_SIZE;
//     }
//
//     KeReleaseSpinLock(&IopDeviceTreeLock, oldIrql);
//
//     DPRINT("%wZ Changed state 0x%x => 0x%x\n", &DeviceNode->InstancePath, prevState, NewState);
//     return prevState;
// }
//
// VOID
// PiSetDevNodeProblem(
//     _In_ PDEVICE_NODE DeviceNode,
//     _In_ UINT32 Problem)
// {
//     DeviceNode->Flags |= DNF_HAS_PROBLEM;
//     DeviceNode->Problem = Problem;
// }
//
// VOID
// PiClearDevNodeProblem(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     DeviceNode->Flags &= ~DNF_HAS_PROBLEM;
//     DeviceNode->Problem = 0;
// }
//
// /**
//  * @brief      Creates a device node
//  *
//  * @param[in]  ParentNode           Pointer to parent device node
//  * @param[in]  PhysicalDeviceObject Pointer to PDO for device object. Pass NULL to have
//  *                                  the root device node create one (eg. for legacy drivers)
//  * @param[in]  ServiceName          The service (driver) name for a node. Pass NULL
//  *                                  to set UNKNOWN as a service
//  * @param[out] DeviceNode           Pointer to storage for created device node
//  *
//  * @return     Status, indicating the result of an operation
//  */
// #if 0
// NTSTATUS
// IopCreateDeviceNode(
//     _In_ PDEVICE_NODE ParentNode,
//     _In_opt_ PDEVICE_OBJECT PhysicalDeviceObject,
//     _In_opt_ PUNICODE_STRING ServiceName,
//     _Out_ PDEVICE_NODE *DeviceNode)
// {
//     PDEVICE_NODE Node;
//     NTSTATUS Status;
//     UNICODE_STRING FullServiceName;
//     UNICODE_STRING LegacyPrefix = RTL_CONSTANT_STRING(L"LEGACY_");
//     UNICODE_STRING UnknownDeviceName = RTL_CONSTANT_STRING(L"UNKNOWN");
//     UNICODE_STRING KeyName, ClassName;
//     PUNICODE_STRING ServiceName1;
//     ULONG LegacyValue;
//     UNICODE_STRING ClassGUID;
//     HANDLE InstanceHandle;
//
//     DPRINT("ParentNode 0x%p PhysicalDeviceObject 0x%p ServiceName %wZ\n",
//            ParentNode, PhysicalDeviceObject, ServiceName);
//
//     Node = ExAllocatePoolWithTag(NonPagedPool, sizeof(DEVICE_NODE), TAG_IO_DEVNODE);
//     if (!Node)
//     {
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     RtlZeroMemory(Node, sizeof(DEVICE_NODE));
//     InitializeListHead(&Node->TargetDeviceNotify);
//
//     if (!ServiceName)
//         ServiceName1 = &UnknownDeviceName;
//     else
//         ServiceName1 = ServiceName;
//
//     if (!PhysicalDeviceObject)
//     {
//         FullServiceName.MaximumLength = LegacyPrefix.Length + ServiceName1->Length + sizeof(UNICODE_NULL);
//         FullServiceName.Length = 0;
//         FullServiceName.Buffer = ExAllocatePool(PagedPool, FullServiceName.MaximumLength);
//         if (!FullServiceName.Buffer)
//         {
//             ExFreePoolWithTag(Node, TAG_IO_DEVNODE);
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//
//         RtlAppendUnicodeStringToString(&FullServiceName, &LegacyPrefix);
//         RtlAppendUnicodeStringToString(&FullServiceName, ServiceName1);
//         RtlUpcaseUnicodeString(&FullServiceName, &FullServiceName, FALSE);
//
//         Status = PnpRootCreateDevice(&FullServiceName, NULL, &PhysicalDeviceObject, &Node->InstancePath);
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("PnpRootCreateDevice() failed with status 0x%08X\n", Status);
//             ExFreePool(FullServiceName.Buffer);
//             ExFreePoolWithTag(Node, TAG_IO_DEVNODE);
//             return Status;
//         }
//
//         /* Create the device key for legacy drivers */
//         Status = IopCreateDeviceKeyPath(&Node->InstancePath, REG_OPTION_VOLATILE, &InstanceHandle);
//         if (!NT_SUCCESS(Status))
//         {
//             ExFreePool(FullServiceName.Buffer);
//             ExFreePoolWithTag(Node, TAG_IO_DEVNODE);
//             return Status;
//         }
//
//         Node->ServiceName.MaximumLength = ServiceName1->Length + sizeof(UNICODE_NULL);
//         Node->ServiceName.Length = 0;
//         Node->ServiceName.Buffer = ExAllocatePool(PagedPool, Node->ServiceName.MaximumLength);
//         if (!Node->ServiceName.Buffer)
//         {
//             ZwClose(InstanceHandle);
//             ExFreePool(FullServiceName.Buffer);
//             ExFreePoolWithTag(Node, TAG_IO_DEVNODE);
//             return Status;
//         }
//
//         RtlCopyUnicodeString(&Node->ServiceName, ServiceName1);
//
//         if (ServiceName)
//         {
//             RtlInitUnicodeString(&KeyName, L"Service");
//             Status = ZwSetValueKey(InstanceHandle, &KeyName, 0, REG_SZ, ServiceName->Buffer, ServiceName->Length + sizeof(UNICODE_NULL));
//         }
//
//         if (NT_SUCCESS(Status))
//         {
//             RtlInitUnicodeString(&KeyName, L"Legacy");
//             LegacyValue = 1;
//             Status = ZwSetValueKey(InstanceHandle, &KeyName, 0, REG_DWORD, &LegacyValue, sizeof(LegacyValue));
//
//             RtlInitUnicodeString(&KeyName, L"ConfigFlags");
//             LegacyValue = 0;
//             ZwSetValueKey(InstanceHandle, &KeyName, 0, REG_DWORD, &LegacyValue, sizeof(LegacyValue));
//
//             if (NT_SUCCESS(Status))
//             {
//                 RtlInitUnicodeString(&KeyName, L"Class");
//                 RtlInitUnicodeString(&ClassName, L"LegacyDriver");
//                 Status = ZwSetValueKey(InstanceHandle, &KeyName, 0, REG_SZ, ClassName.Buffer, ClassName.Length + sizeof(UNICODE_NULL));
//                 if (NT_SUCCESS(Status))
//                 {
//                     RtlInitUnicodeString(&KeyName, L"ClassGUID");
//                     RtlInitUnicodeString(&ClassGUID, L"{8ECC055D-047F-11D1-A537-0000F8753ED1}");
//                     Status = ZwSetValueKey(InstanceHandle, &KeyName, 0, REG_SZ, ClassGUID.Buffer, ClassGUID.Length + sizeof(UNICODE_NULL));
//                     if (NT_SUCCESS(Status))
//                     {
//                         // FIXME: Retrieve the real "description" by looking at the "DisplayName" string
//                         // of the corresponding CurrentControlSet\Services\xxx entry for this driver.
//                         RtlInitUnicodeString(&KeyName, L"DeviceDesc");
//                         Status = ZwSetValueKey(InstanceHandle, &KeyName, 0, REG_SZ, ServiceName1->Buffer, ServiceName1->Length + sizeof(UNICODE_NULL));
//                     }
//                 }
//             }
//         }
//
//         ZwClose(InstanceHandle);
//         ExFreePool(FullServiceName.Buffer);
//
//         if (!NT_SUCCESS(Status))
//         {
//             ExFreePool(Node->ServiceName.Buffer);
//             ExFreePoolWithTag(Node, TAG_IO_DEVNODE);
//             return Status;
//         }
//
//         IopDeviceNodeSetFlag(Node, DNF_LEGACY_DRIVER);
//         IopDeviceNodeSetFlag(Node, DNF_PROCESSED);
//         IopDeviceNodeSetFlag(Node, DNF_ADDED);
//         IopDeviceNodeSetFlag(Node, DNF_STARTED);
//     }
//
//     Node->PhysicalDeviceObject = PhysicalDeviceObject;
//
//     ((PEXTENDED_DEVOBJ_EXTENSION)PhysicalDeviceObject->DeviceObjectExtension)->DeviceNode = Node;
//
//     if (ParentNode)
//     {
//         KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//         Node->Parent = ParentNode;
//         Node->Sibling = NULL;
//         if (ParentNode->LastChild == NULL)
//         {
//             ParentNode->Child = Node;
//             ParentNode->LastChild = Node;
//         }
//         else
//         {
//             ParentNode->LastChild->Sibling = Node;
//             ParentNode->LastChild = Node;
//         }
//         KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//         Node->Level = ParentNode->Level + 1;
//     }
//
//     PhysicalDeviceObject->Flags &= ~DO_DEVICE_INITIALIZING;
//
//     *DeviceNode = Node;
//
//     return STATUS_SUCCESS;
// }
// #endif
//
// NTSTATUS
// IopFreeDeviceNode(
//     _In_ PDEVICE_NODE DeviceNode)
// {
//     KIRQL OldIrql;
//     PDEVICE_NODE PrevSibling = NULL;
//
//     ASSERT(DeviceNode->PhysicalDeviceObject);
//     /* All children must be deleted before a parent is deleted */
//     ASSERT(DeviceNode->Child == NULL);
//     /* This is the only state where we are allowed to remove the node */
//     ASSERT(DeviceNode->State == DeviceNodeRemoved);
//     /* No notifications should be registered for this device */
//     ASSERT(IsListEmpty(&DeviceNode->TargetDeviceNotify));
//
//     KeAcquireSpinLock(&IopDeviceTreeLock, &OldIrql);
//
//     /* Get previous sibling */
//     if (DeviceNode->Parent && DeviceNode->Parent->Child != DeviceNode)
//     {
//         PrevSibling = DeviceNode->Parent->Child;
//         while (PrevSibling->Sibling != DeviceNode)
//             PrevSibling = PrevSibling->Sibling;
//     }
//
//     /* Unlink from parent if it exists */
//     if (DeviceNode->Parent)
//     {
//         if (DeviceNode->Parent->LastChild == DeviceNode)
//         {
//             DeviceNode->Parent->LastChild = PrevSibling;
//             if (PrevSibling)
//                 PrevSibling->Sibling = NULL;
//         }
//         if (DeviceNode->Parent->Child == DeviceNode)
//             DeviceNode->Parent->Child = DeviceNode->Sibling;
//     }
//
//     /* Unlink from sibling list */
//     if (PrevSibling)
//         PrevSibling->Sibling = DeviceNode->Sibling;
//
//     KeReleaseSpinLock(&IopDeviceTreeLock, OldIrql);
//
//     RtlFreeUnicodeString(&DeviceNode->InstancePath);
//
//     RtlFreeUnicodeString(&DeviceNode->ServiceName);
//
//     if (DeviceNode->ResourceList)
//     {
//         ExFreePool(DeviceNode->ResourceList);
//     }
//
//     if (DeviceNode->ResourceListTranslated)
//     {
//         ExFreePool(DeviceNode->ResourceListTranslated);
//     }
//
//     if (DeviceNode->ResourceRequirements)
//     {
//         ExFreePool(DeviceNode->ResourceRequirements);
//     }
//
//     if (DeviceNode->BootResources)
//     {
//         ExFreePool(DeviceNode->BootResources);
//     }
//
//     ((PEXTENDED_DEVOBJ_EXTENSION)DeviceNode->PhysicalDeviceObject->DeviceObjectExtension)->DeviceNode = NULL;
//     ExFreePoolWithTag(DeviceNode, TAG_IO_DEVNODE);
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// IopFindNextDeviceNodeForTraversal(
//     _In_ PDEVICETREE_TRAVERSE_CONTEXT Context)
// {
//     /* If we have a child, simply go down the tree */
//     if (Context->DeviceNode->Child != NULL)
//     {
//         ASSERT(Context->DeviceNode->Child->Parent == Context->DeviceNode);
//         Context->DeviceNode = Context->DeviceNode->Child;
//         return STATUS_SUCCESS;
//     }
//
//     while (Context->DeviceNode != Context->FirstDeviceNode)
//     {
//         /* All children processed -- go sideways */
//         if (Context->DeviceNode->Sibling != NULL)
//         {
//             ASSERT(Context->DeviceNode->Sibling->Parent == Context->DeviceNode->Parent);
//             Context->DeviceNode = Context->DeviceNode->Sibling;
//             return STATUS_SUCCESS;
//         }
//
//         /* We're the last sibling -- go back up */
//         ASSERT(Context->DeviceNode->Parent->LastChild == Context->DeviceNode);
//         Context->DeviceNode = Context->DeviceNode->Parent;
//
//         /* We already visited the parent and all its children, so keep looking */
//     }
//
//     /* Done with all children of the start node -- stop enumeration */
//     return STATUS_UNSUCCESSFUL;
// }
//
// NTSTATUS
// IopTraverseDeviceTree(
//     _In_ PDEVICETREE_TRAVERSE_CONTEXT Context)
// {
//     NTSTATUS Status;
//     PDEVICE_NODE DeviceNode;
//
//     DPRINT("Context 0x%p\n", Context);
//
//     DPRINT("IopTraverseDeviceTree(DeviceNode 0x%p  FirstDeviceNode 0x%p  Action %p  Context 0x%p)\n",
//            Context->DeviceNode, Context->FirstDeviceNode, Context->Action, Context->Context);
//
//     /* Start from the specified device node */
//     Context->DeviceNode = Context->FirstDeviceNode;
//
//     /* Traverse the device tree */
//     do
//     {
//         DeviceNode = Context->DeviceNode;
//
//         /* HACK: Keep a reference to the PDO so we can keep traversing the tree
//          * if the device is deleted. In a perfect world, children would have to be
//          * deleted before their parents, and we'd restart the traversal after
//          * deleting a device node. */
//         ObReferenceObject(DeviceNode->PhysicalDeviceObject);
//
//         /* Call the action routine */
//         Status = (Context->Action)(DeviceNode, Context->Context);
//         if (NT_SUCCESS(Status))
//         {
//             /* Find next device node */
//             ASSERT(Context->DeviceNode == DeviceNode);
//             Status = IopFindNextDeviceNodeForTraversal(Context);
//         }
//
//         /* We need to either abort or make progress */
//         ASSERT(!NT_SUCCESS(Status) || Context->DeviceNode != DeviceNode);
//
//         ObDereferenceObject(DeviceNode->PhysicalDeviceObject);
//     } while (NT_SUCCESS(Status));
//
//     if (Status == STATUS_UNSUCCESSFUL)
//     {
//         /* The action routine just wanted to terminate the traversal with status
//         code STATUS_SUCCESS */
//         Status = STATUS_SUCCESS;
//     }
//
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
