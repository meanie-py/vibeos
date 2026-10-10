use crate::types::*;
use crate::glue::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::ddk::{CLSID, GUID, IoRegisterDeviceInterface, IoSetDeviceInterfaceState, PDEVICE_OBJECT, PIRP, PUNICODE_STRING, RtlFreeUnicodeString};
use crate::misc::{AllocateItem, FreeItem};
use crate::filterfactory::IKsFilterFactory_Create;
use core::ffi::c_void;

// ---- file-scope preamble (translated) ----
pub const NDEBUG: u32 = 0;
#[no_mangle]
pub unsafe extern "C" fn KspSetDeviceInterfacesState(
    ListHead: PLIST_ENTRY,
    Enable: BOOL,
) -> NTSTATUS {
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut Entry: PLIST_ENTRY;
    let mut SymEntry: PSYMBOLIC_LINK_ENTRY;

    Entry = (*ListHead).Flink;
    while Entry != ListHead {
        SymEntry = core::ptr::with_exposed_provenance_mut::<SYMBOLIC_LINK_ENTRY>((Entry as usize).wrapping_sub(
                core::mem::offset_of!(SYMBOLIC_LINK_ENTRY, Entry) as usize,
            ),
        );
        Status = IoSetDeviceInterfaceState(&raw mut (*SymEntry).SymbolicLink, Enable as BOOLEAN);
        if !NT_SUCCESS(Status) {
            return Status;
        }
        Entry = (*Entry).Flink;
    }
    Status
}
#[no_mangle]
pub unsafe extern "C" fn KspFreeDeviceInterfaces(
    ListHead: PLIST_ENTRY,
) -> NTSTATUS {
    let mut Entry: PLIST_ENTRY;
    let mut SymEntry: PSYMBOLIC_LINK_ENTRY;

    while !IsListEmpty(ListHead) {
        Entry = RemoveHeadList(ListHead);
        SymEntry = core::ptr::with_exposed_provenance_mut::<SYMBOLIC_LINK_ENTRY>((Entry as usize)
                - core::mem::offset_of!(SYMBOLIC_LINK_ENTRY, Entry) as usize,
        );
        RtlFreeUnicodeString(&mut (*SymEntry).SymbolicLink);
        FreeItem(Entry as *mut core::ffi::c_void);
    }

    STATUS_SUCCESS
}
#[no_mangle]
pub unsafe extern "C" fn KspRegisterDeviceInterfaces(
    PhysicalDeviceObject: PDEVICE_OBJECT,
    CategoriesCount: ULONG,
    Categories: *const GUID,
    ReferenceString: PUNICODE_STRING,
    SymbolicLinkList: PLIST_ENTRY,
) -> NTSTATUS {
    let mut Index: ULONG = 0;
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut SymEntry: PSYMBOLIC_LINK_ENTRY;

    while Index < CategoriesCount {
        SymEntry = AllocateItem(NonPagedPool, core::mem::size_of::<SYMBOLIC_LINK_ENTRY>()) as PSYMBOLIC_LINK_ENTRY;
        if SymEntry.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        Status = IoRegisterDeviceInterface(
            PhysicalDeviceObject,
            Categories.offset(Index as isize),
            ReferenceString,
            &mut (*SymEntry).SymbolicLink,
        );

        if !NT_SUCCESS(Status) {
            FreeItem(SymEntry as PVOID);
            return Status;
        }

        core::ptr::copy_nonoverlapping(
            Categories.offset(Index as isize) as *const u8,
            &mut (*SymEntry).DeviceInterfaceClass as *mut _ as *mut u8,
            core::mem::size_of::<CLSID>(),
        );

        InsertTailList(SymbolicLinkList, &mut (*SymEntry).Entry);

        Index = Index.wrapping_add(1);
    }

    Status
}
#[allow(unpredictable_function_pointer_comparisons)] // FNPTR-CMP(audited): intentional identity compare mirroring the C source; revisit -> core::ptr::fn_addr_eq (see docs/engineering/active-lanes/warnings-burndown-plan.md)
#[no_mangle]
pub unsafe extern "C" fn KspSetFilterFactoriesState(
    DeviceHeader: PKSIDEVICE_HEADER,
    NewState: BOOLEAN,
) -> NTSTATUS {
    let mut Entry: PLIST_ENTRY;
    let mut Status: NTSTATUS = STATUS_SUCCESS;

    Entry = (*DeviceHeader).ItemList.Flink;
    while Entry != &mut (*DeviceHeader).ItemList && Status == STATUS_SUCCESS {
        let CreateEntry: PCREATE_ITEM_ENTRY = core::ptr::with_exposed_provenance_mut::<CREATE_ITEM_ENTRY>((Entry as usize).wrapping_sub(
                core::mem::offset_of!(CREATE_ITEM_ENTRY, Entry),
            ),
        );

        if !(*CreateEntry).CreateItem.is_null() {
            if (*(*CreateEntry).CreateItem).Create == cb!(IKsFilterFactory_Create) {
                Status = KsFilterFactorySetDeviceClassesState(
                    (*(*CreateEntry).CreateItem).Context as PKSFILTERFACTORY,
                    NewState,
                );
            }
        }

        Entry = (*Entry).Flink;
    }

    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// #include "precomp.h"
//
// #define NDEBUG
// #include <debug.h>
//
// NTSTATUS
// KspSetDeviceInterfacesState(
//     IN PLIST_ENTRY ListHead,
//     IN BOOL Enable)
// {
//     NTSTATUS Status = STATUS_SUCCESS;
//     PLIST_ENTRY Entry;
//     PSYMBOLIC_LINK_ENTRY SymEntry;
//
//
//     Entry = ListHead->Flink;
//     while(Entry != ListHead)
//     {
//         /* fetch symbolic link entry */
//         SymEntry = (PSYMBOLIC_LINK_ENTRY)CONTAINING_RECORD!(Entry, SYMBOLIC_LINK_ENTRY, Entry);
//         /* set device interface state */
//         Status = IoSetDeviceInterfaceState(&SymEntry->SymbolicLink, Enable);
//
//         DPRINT("KspSetDeviceInterfacesState SymbolicLink '%S' Status %lx\n", SymEntry->SymbolicLink.Buffer, Status, Enable);
//
//         /* check for success */
//         if (!NT_SUCCESS(Status))
//             return Status;
//         /* get next entry */
//         Entry = Entry->Flink;
//     }
//     /* return result */
//     return Status;
// }
//
// NTSTATUS
// KspFreeDeviceInterfaces(
//     IN PLIST_ENTRY ListHead)
// {
//     PLIST_ENTRY Entry;
//     PSYMBOLIC_LINK_ENTRY SymEntry;
//
//     while(!IsListEmpty(ListHead))
//     {
//         /* remove first entry */
//         Entry = RemoveHeadList(ListHead);
//
//         /* fetch symbolic link entry */
//         SymEntry = (PSYMBOLIC_LINK_ENTRY)CONTAINING_RECORD!(Entry, SYMBOLIC_LINK_ENTRY, Entry);
//
//         /* free device interface string */
//         RtlFreeUnicodeString(&SymEntry->SymbolicLink);
//         /* free entry item */
//         FreeItem(Entry);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// KspRegisterDeviceInterfaces(
//     IN PDEVICE_OBJECT PhysicalDeviceObject,
//     IN ULONG CategoriesCount,
//     IN GUID const*Categories,
//     IN PUNICODE_STRING ReferenceString,
//     OUT PLIST_ENTRY SymbolicLinkList)
// {
//     ULONG Index;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PSYMBOLIC_LINK_ENTRY SymEntry;
//
//     for(Index = 0; Index < CategoriesCount; Index++)
//     {
//         /* allocate a symbolic link entry */
//         SymEntry = AllocateItem(NonPagedPool, sizeof(SYMBOLIC_LINK_ENTRY));
//         /* check for success */
//         if (!SymEntry)
//             return STATUS_INSUFFICIENT_RESOURCES;
//
//         /* now register device interface */
//         Status = IoRegisterDeviceInterface(PhysicalDeviceObject,
//                                            &Categories[Index],
//                                            ReferenceString,
//                                            &SymEntry->SymbolicLink);
//
//         if (!NT_SUCCESS(Status))
//         {
//             DPRINT1("Failed to register device interface %x\n", Status);
//
//             /* free entry */
//             FreeItem(SymEntry);
//
//             /* return result */
//             return Status;
//         }
//
//         /* copy device class */
//         RtlMoveMemory(&SymEntry->DeviceInterfaceClass, &Categories[Index], sizeof(CLSID));
//
//         /* insert symbolic link entry */
//         InsertTailList(SymbolicLinkList, &SymEntry->Entry);
//     }
//
//     /* return result */
//     return Status;
// }
//
// NTSTATUS
// KspSetFilterFactoriesState(
//     IN PKSIDEVICE_HEADER DeviceHeader,
//     IN BOOLEAN NewState)
// {
//     PCREATE_ITEM_ENTRY CreateEntry;
//     PLIST_ENTRY Entry;
//     NTSTATUS Status = STATUS_SUCCESS;
//
//     /* grab first device interface */
//     Entry = DeviceHeader->ItemList.Flink;
//     while(Entry != &DeviceHeader->ItemList && Status == STATUS_SUCCESS)
//     {
//         /* grab create entry */
//         CreateEntry = CONTAINING_RECORD!(Entry, CREATE_ITEM_ENTRY, Entry);
//
//         /* sanity check */
//         ASSERT(CreateEntry->CreateItem);
//
//         if (CreateEntry->CreateItem->Create == IKsFilterFactory_Create)
//         {
//             /* found our own filterfactory */
//             Status = KsFilterFactorySetDeviceClassesState((PKSFILTERFACTORY)CreateEntry->CreateItem->Context, NewState);
//         }
//
//         Entry = Entry->Flink;
//     }
//
//     /* store result */
//     return Status;
// }
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
