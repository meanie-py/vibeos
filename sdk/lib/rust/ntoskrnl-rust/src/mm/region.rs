use ntkrnl_sys::base::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;

use ntkrnl_sys::ddk::{
    PMM_REGION, PMMSUPPORT, PMM_ALTER_REGION_FUNC,
    _MM_REGION, MM_REGION,
};
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

const TAG_MM_REGION: u32 = u32::from_be_bytes(*b"NGRM"); // C literal 'NGRM' -- ntoskrnl/include/internal/tag.h

/// CONTAINING_RECORD equivalent for MM_REGION
#[inline]
unsafe fn containing_record_region(entry: PLIST_ENTRY) -> PMM_REGION {
    (entry as *mut u8).sub(core::mem::offset_of!(_MM_REGION, RegionListEntry)) as PMM_REGION
}

unsafe fn InsertAfterEntry(Previous: PLIST_ENTRY, Entry: PLIST_ENTRY) {
    (*(*Previous).Flink).Blink = Entry;
    (*Entry).Flink = (*Previous).Flink;
    (*Entry).Blink = Previous;
    (*Previous).Flink = Entry;
}

unsafe fn MmSplitRegion(
    InitialRegion: PMM_REGION,
    InitialBaseAddress: PVOID,
    StartAddress: PVOID,
    Length: SIZE_T,
    NewType: ULONG,
    NewProtect: ULONG,
    AddressSpace: PMMSUPPORT,
    AlterFunc: PMM_ALTER_REGION_FUNC,
) -> PMM_REGION {
    let NewRegion2 = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<MM_REGION>(),
        TAG_MM_REGION,
    ) as PMM_REGION;
    if NewRegion2.is_null() {
        return core::ptr::null_mut();
    }

    let NewRegion1 = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<MM_REGION>(),
        TAG_MM_REGION,
    ) as PMM_REGION;
    if NewRegion1.is_null() {
        ExFreePoolWithTag(NewRegion2 as PVOID, TAG_MM_REGION);
        return core::ptr::null_mut();
    }

    (*NewRegion1).Type = NewType;
    (*NewRegion1).Protect = NewProtect;
    let region_len = (*InitialRegion).Length as usize;
    let mut InternalLength = (InitialBaseAddress as usize + region_len) - StartAddress as usize;
    InternalLength = core::cmp::min(InternalLength, Length);
    (*NewRegion1).Length = InternalLength as _;
    InsertAfterEntry(
        core::ptr::addr_of_mut!((*InitialRegion).RegionListEntry),
        core::ptr::addr_of_mut!((*NewRegion1).RegionListEntry),
    );

    (AlterFunc.unwrap())(
        AddressSpace,
        StartAddress,
        InternalLength as _,
        (*InitialRegion).Type,
        (*InitialRegion).Protect,
        NewType,
        NewProtect,
    );

    if (InitialBaseAddress as usize + region_len) > (StartAddress as usize + Length) {
        (*NewRegion2).Type = (*InitialRegion).Type;
        (*NewRegion2).Protect = (*InitialRegion).Protect;
        (*NewRegion2).Length = ((InitialBaseAddress as usize + region_len)
            - (StartAddress as usize + Length)) as _;
        InsertAfterEntry(
            core::ptr::addr_of_mut!((*NewRegion1).RegionListEntry),
            core::ptr::addr_of_mut!((*NewRegion2).RegionListEntry),
        );
    } else {
        ExFreePoolWithTag(NewRegion2 as PVOID, TAG_MM_REGION);
    }

    if InitialBaseAddress == StartAddress {
        RemoveEntryList(core::ptr::addr_of_mut!((*InitialRegion).RegionListEntry));
        ExFreePoolWithTag(InitialRegion as PVOID, TAG_MM_REGION);
    } else {
        (*InitialRegion).Length = (StartAddress as usize - InitialBaseAddress as usize) as _;
    }

    NewRegion1
}

#[no_mangle]
pub unsafe extern "system" fn MmAlterRegion(
    AddressSpace: PMMSUPPORT,
    BaseAddress: PVOID,
    RegionListHead: PLIST_ENTRY,
    StartAddress: PVOID,
    Length: SIZE_T,
    NewType: ULONG,
    NewProtect: ULONG,
    AlterFunc: PMM_ALTER_REGION_FUNC,
) -> NTSTATUS {
    let BaseAddress = opaque_ptr(BaseAddress);
    let RegionListHead = opaque_ptr(RegionListHead);
    let StartAddress = opaque_ptr(StartAddress);
    let mut InitialBaseAddress: PVOID = core::ptr::null_mut();
    let InitialRegion = MmFindRegion(
        BaseAddress,
        RegionListHead,
        StartAddress,
        core::ptr::addr_of_mut!(InitialBaseAddress),
    );

    let NewRegion: PMM_REGION;
    let mut RemainingLength: SIZE_T;

    if (*InitialRegion).Type != NewType || (*InitialRegion).Protect != NewProtect {
        NewRegion = MmSplitRegion(
            InitialRegion,
            InitialBaseAddress,
            StartAddress,
            Length,
            NewType,
            NewProtect,
            AddressSpace,
            AlterFunc,
        );
        if NewRegion.is_null() {
            return STATUS_NO_MEMORY;
        }
        let new_len = (*NewRegion).Length as usize;
        if new_len < Length {
            RemainingLength = Length - new_len;
        } else {
            RemainingLength = 0;
        }
    } else {
        NewRegion = InitialRegion;
        let new_len = (*NewRegion).Length as usize;
        if (InitialBaseAddress as usize + new_len) < (StartAddress as usize + Length) {
            RemainingLength = (StartAddress as usize + Length) - (InitialBaseAddress as usize + new_len);
        } else {
            RemainingLength = 0;
        }
    }

    let mut CurrentEntry = (*NewRegion).RegionListEntry.Flink;
    let mut CurrentRegion = containing_record_region(CurrentEntry);
    let new_len = (*NewRegion).Length as usize;
    let mut CurrentBaseAddress = (StartAddress as usize + new_len) as PVOID;

    while RemainingLength > 0
        && ((*CurrentRegion).Length as usize) <= RemainingLength
        && CurrentEntry != RegionListHead
    {
        let cur_len = (*CurrentRegion).Length as usize;
        if (*CurrentRegion).Type != NewType || (*CurrentRegion).Protect != NewProtect {
            (AlterFunc.unwrap())(
                AddressSpace,
                CurrentBaseAddress,
                cur_len as _,
                (*CurrentRegion).Type,
                (*CurrentRegion).Protect,
                NewType,
                NewProtect,
            );
        }

        CurrentBaseAddress = (CurrentBaseAddress as usize + cur_len) as PVOID;
        (*NewRegion).Length = ((*NewRegion).Length as usize + cur_len) as _;
        RemainingLength -= cur_len;
        CurrentEntry = (*CurrentEntry).Flink;
        RemoveEntryList(core::ptr::addr_of_mut!((*CurrentRegion).RegionListEntry));
        ExFreePoolWithTag(CurrentRegion as PVOID, TAG_MM_REGION);
        CurrentRegion = containing_record_region(CurrentEntry);
    }

    if RemainingLength > 0 && CurrentEntry != RegionListHead {
        CurrentRegion = containing_record_region(CurrentEntry);
        if (*CurrentRegion).Type != NewType || (*CurrentRegion).Protect != NewProtect {
            (AlterFunc.unwrap())(
                AddressSpace,
                CurrentBaseAddress,
                RemainingLength as _,
                (*CurrentRegion).Type,
                (*CurrentRegion).Protect,
                NewType,
                NewProtect,
            );
        }
        (*NewRegion).Length = ((*NewRegion).Length as usize + RemainingLength) as _;
        (*CurrentRegion).Length = ((*CurrentRegion).Length as usize - RemainingLength) as _;
    }

    if (*NewRegion).RegionListEntry.Flink != RegionListHead {
        CurrentEntry = (*NewRegion).RegionListEntry.Flink;
        CurrentRegion = containing_record_region(CurrentEntry);
        if (*CurrentRegion).Type == (*NewRegion).Type
            && (*CurrentRegion).Protect == (*NewRegion).Protect
        {
            (*NewRegion).Length += (*CurrentRegion).Length;
            RemoveEntryList(core::ptr::addr_of_mut!((*CurrentRegion).RegionListEntry));
            ExFreePoolWithTag(CurrentRegion as PVOID, TAG_MM_REGION);
        }
    }

    if (*NewRegion).RegionListEntry.Blink != RegionListHead {
        CurrentEntry = (*NewRegion).RegionListEntry.Blink;
        CurrentRegion = containing_record_region(CurrentEntry);
        if (*CurrentRegion).Type == (*NewRegion).Type
            && (*CurrentRegion).Protect == (*NewRegion).Protect
        {
            (*NewRegion).Length += (*CurrentRegion).Length;
            RemoveEntryList(core::ptr::addr_of_mut!((*CurrentRegion).RegionListEntry));
            ExFreePoolWithTag(CurrentRegion as PVOID, TAG_MM_REGION);
        }
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn MmInitializeRegion(
    RegionListHead: PLIST_ENTRY,
    Length: SIZE_T,
    Type: ULONG,
    Protect: ULONG,
) {
    let RegionListHead = opaque_ptr(RegionListHead);
    let Region = ExAllocatePoolWithTag(
        NonPagedPool,
        core::mem::size_of::<MM_REGION>(),
        TAG_MM_REGION,
    ) as PMM_REGION;
    if Region.is_null() {
        return;
    }

    (*Region).Type = Type;
    (*Region).Protect = Protect;
    (*Region).Length = Length as _;
    InitializeListHead(RegionListHead);
    InsertHeadList(RegionListHead, core::ptr::addr_of_mut!((*Region).RegionListEntry));
}

#[no_mangle]
pub unsafe extern "system" fn MmFindRegion(
    BaseAddress: PVOID,
    RegionListHead: PLIST_ENTRY,
    Address: PVOID,
    RegionBaseAddress: *mut PVOID,
) -> PMM_REGION {
    let BaseAddress = opaque_ptr(BaseAddress);
    let RegionListHead = opaque_ptr(RegionListHead);
    let Address = opaque_ptr(Address);
    let RegionBaseAddress = opaque_ptr(RegionBaseAddress);
    let mut current_entry = (*RegionListHead).Flink;
    let mut StartAddress = BaseAddress;

    while current_entry != RegionListHead {
        let current = containing_record_region(current_entry);
        let cur_len = (*current).Length as usize;

        if StartAddress <= Address && (StartAddress as usize + cur_len) > Address as usize {
            if !RegionBaseAddress.is_null() {
                *RegionBaseAddress = StartAddress;
            }
            return current;
        }

        current_entry = (*current_entry).Flink;
        StartAddress = (StartAddress as usize + cur_len) as PVOID;
    }

    core::ptr::null_mut()
}
