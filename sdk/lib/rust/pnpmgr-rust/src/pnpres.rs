use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::{opaque_ptr, opaque_ptr_const};

use ntkrnl_sys::ddk::{
    CM_PARTIAL_RESOURCE_DESCRIPTOR,
    CM_RESOURCE_LIST,
    CmiGetNextResourceDescriptor,
    ExAllocatePool,
    ExFreePool,
    HalAdjustResourceList,
    HalGetInterruptVector,
    HalTranslateBusAddress,
    IO_RESOURCE_LIST,
    IO_STACK_LOCATION,
    _IO_STATUS_BLOCK,
    IoGetDeviceProperty,
    IopInitiatePnpIrp,
    IopOpenRegistryKeyEx,
    KAFFINITY,
    _OBJECT_ATTRIBUTES,
    ObCloseHandle,
    PCM_FULL_RESOURCE_DESCRIPTOR,
    PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    PCM_PARTIAL_RESOURCE_LIST,
    PCM_RESOURCE_LIST,
    PDEVICE_NODE,
    PIO_RESOURCE_DESCRIPTOR,
    PIO_RESOURCE_LIST,
    PIO_RESOURCE_REQUIREMENTS_LIST,
    PKEY_BASIC_INFORMATION,
    PKEY_VALUE_BASIC_INFORMATION,
    PKEY_VALUE_PARTIAL_INFORMATION,
    RtlAppendUnicodeStringToString,
    RtlInitUnicodeString,
    UINT32,
    UINT64,
    ZwClose,
    ZwCreateKey,
    ZwEnumerateKey,
    ZwEnumerateValueKey,
    ZwOpenKey,
    ZwSetValueKey,
    _CM_PARTIAL_RESOURCE_DESCRIPTOR,
    _CM_RESOURCE_LIST,
    _IO_RESOURCE_LIST,
    _UNICODE_STRING,
};
#[inline(always)]
unsafe fn InitializeObjectAttributes(
    p: *mut _OBJECT_ATTRIBUTES,
    n: *mut _UNICODE_STRING,
    a: ULONG,
    r: HANDLE,
    s: PVOID,
) {
    (*p).Length = core::mem::size_of::<_OBJECT_ATTRIBUTES>() as ULONG;
    (*p).RootDirectory = r;
    (*p).Attributes = a;
    (*p).ObjectName = n;
    (*p).SecurityDescriptor = s;
    (*p).SecurityQualityOfService = core::ptr::null_mut();
}

extern "C" {
    fn PiSetDevNodeState(DeviceNode: PDEVICE_NODE, NewState: u32) -> u32;
}
extern "system" {
    fn PnpDetermineResourceListSize(ResourceList: PCM_RESOURCE_LIST) -> ULONG;
}

const STATUS_CONFLICTING_ADDRESSES: NTSTATUS = 0xC0000018u32 as NTSTATUS;

unsafe fn IopGetNextResourceList(ResourceList: *const IO_RESOURCE_LIST) -> PIO_RESOURCE_LIST {
    ntkrnl_sys::ASSERT!((*ResourceList).Count > 0 && (*ResourceList).Count < 1000);
    (core::ptr::addr_of!((*ResourceList).Descriptors[0]).add((*ResourceList).Count as usize)
        as *const u8) as PIO_RESOURCE_LIST
}

unsafe fn IopCheckDescriptorForConflict(
    CmDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    ConflictingDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> BOOLEAN {
    let mut CmList: CM_RESOURCE_LIST = core::mem::zeroed();

    CmList.Count = 1;
    CmList.List[0].InterfaceType = -1i32;
    CmList.List[0].BusNumber = 0;
    CmList.List[0].PartialResourceList.Version = 1;
    CmList.List[0].PartialResourceList.Revision = 1;
    CmList.List[0].PartialResourceList.Count = 1;
    CmList.List[0].PartialResourceList.PartialDescriptors[0] = *CmDesc;

    let Status = IopDetectResourceConflict(
        &mut CmList as *mut _,
        1u8,
        ConflictingDescriptor,
    );
    if Status == STATUS_CONFLICTING_ADDRESSES {
        return 1u8;
    }

    0u8
}

unsafe fn IopFindBusNumberResource(
    IoDesc: PIO_RESOURCE_DESCRIPTOR,
    CmDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> BOOLEAN {
    let mut Start: ULONG;
    let mut ConflictingDesc: CM_PARTIAL_RESOURCE_DESCRIPTOR = core::mem::zeroed();

    ntkrnl_sys::ASSERT!((*IoDesc).Type == (*CmDesc).Type);
    ntkrnl_sys::ASSERT!((*IoDesc).Type == 6); // CmResourceTypeBusNumber

    Start = (*IoDesc).u.BusNumber.MinBusNumber;
    while Start <= (*IoDesc).u.BusNumber.MaxBusNumber - (*IoDesc).u.BusNumber.Length + 1 {
        (*CmDesc).u.BusNumber.Length = (*IoDesc).u.BusNumber.Length;
        (*CmDesc).u.BusNumber.Start = Start;

        if IopCheckDescriptorForConflict(CmDesc, &mut ConflictingDesc) != 0 {
            Start += ConflictingDesc.u.BusNumber.Start + ConflictingDesc.u.BusNumber.Length;
        } else {
            return 1u8;
        }
        Start += 1;
    }

    0u8
}

unsafe fn IopFindMemoryResource(
    IoDesc: PIO_RESOURCE_DESCRIPTOR,
    CmDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> BOOLEAN {
    let mut Start: ULONGLONG;
    let mut ConflictingDesc: CM_PARTIAL_RESOURCE_DESCRIPTOR = core::mem::zeroed();

    ntkrnl_sys::ASSERT!((*IoDesc).Type == (*CmDesc).Type);
    ntkrnl_sys::ASSERT!((*IoDesc).Type == 3); // CmResourceTypeMemory

    if (*IoDesc).u.Memory.Alignment == 0 {
        // Set Alignment to 1 via direct field write
        (*(IoDesc)).u.Memory.Alignment = 1;
    }

    Start = (*IoDesc).u.Memory.MinimumAddress.QuadPart as ULONGLONG;
    while Start
        <= (*IoDesc).u.Memory.MaximumAddress.QuadPart as ULONGLONG
            - (*IoDesc).u.Memory.Length as ULONGLONG
            + 1
    {
        (*CmDesc).u.Memory.Length = (*IoDesc).u.Memory.Length;
        (*CmDesc).u.Memory.Start.QuadPart = Start as LONGLONG;

        if IopCheckDescriptorForConflict(CmDesc, &mut ConflictingDesc) != 0 {
            Start += ConflictingDesc.u.Memory.Start.QuadPart as ULONGLONG
                + ConflictingDesc.u.Memory.Length as ULONGLONG;
        } else {
            return 1u8;
        }
        Start += (*IoDesc).u.Memory.Alignment as ULONGLONG;
    }

    0u8
}

unsafe fn IopFindPortResource(
    IoDesc: PIO_RESOURCE_DESCRIPTOR,
    CmDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> BOOLEAN {
    let mut Start: ULONGLONG;
    let mut ConflictingDesc: CM_PARTIAL_RESOURCE_DESCRIPTOR = core::mem::zeroed();

    ntkrnl_sys::ASSERT!((*IoDesc).Type == (*CmDesc).Type);
    ntkrnl_sys::ASSERT!((*IoDesc).Type == 1); // CmResourceTypePort

    if (*IoDesc).u.Port.Alignment == 0 {
        (*(IoDesc)).u.Port.Alignment = 1;
    }

    Start = (*IoDesc).u.Port.MinimumAddress.QuadPart as ULONGLONG;
    while Start
        <= (*IoDesc).u.Port.MaximumAddress.QuadPart as ULONGLONG
            - (*IoDesc).u.Port.Length as ULONGLONG
            + 1
    {
        (*CmDesc).u.Port.Length = (*IoDesc).u.Port.Length;
        (*CmDesc).u.Port.Start.QuadPart = Start as LONGLONG;

        if IopCheckDescriptorForConflict(CmDesc, &mut ConflictingDesc) != 0 {
            Start += ConflictingDesc.u.Port.Start.QuadPart as ULONGLONG
                + ConflictingDesc.u.Port.Length as ULONGLONG;
        } else {
            return 1u8;
        }
        Start += (*IoDesc).u.Port.Alignment as ULONGLONG;
    }

    0u8
}

unsafe fn IopFindDmaResource(
    IoDesc: PIO_RESOURCE_DESCRIPTOR,
    CmDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> BOOLEAN {
    let mut Channel: ULONG;

    ntkrnl_sys::ASSERT!((*IoDesc).Type == (*CmDesc).Type);
    ntkrnl_sys::ASSERT!((*IoDesc).Type == 4); // CmResourceTypeDma

    Channel = (*IoDesc).u.Dma.MinimumChannel;
    while Channel <= (*IoDesc).u.Dma.MaximumChannel {
        (*CmDesc).u.Dma.Channel = Channel;
        (*CmDesc).u.Dma.Port = 0;

        if IopCheckDescriptorForConflict(CmDesc, core::ptr::null_mut()) == 0 {
            return 1u8;
        }
        Channel += 1;
    }

    0u8
}

/// `CM_RESOURCE_INTERRUPT_MESSAGE` (cmtypes.h).
const CM_RESOURCE_INTERRUPT_MESSAGE: u16 = 0x0002;
// IRQ_DEVICE_POLICY (iotypes.h).
const IRQ_POLICY_ONE_CLOSE_PROCESSOR: i32 = 2;
const IRQ_POLICY_SPECIFIED_PROCESSORS: i32 = 4;

unsafe extern "system" {
    fn HalVibeosMessageAllocate(Token: ULONG, Requested: KAFFINITY, Irql: *mut KIRQL, Affinity: *mut KAFFINITY) -> ULONG;
    fn HalVibeosMessageRelease(Vector: ULONG) -> BOOLEAN;
}

/// The processors a message may be delivered to, from the requirement's affinity
/// policy (which pci.sys fills in from the device's `Affinity Policy` key). With no
/// NUMA nodes every processor is "close", so one close processor is the boot
/// processor and the all-close and machine-default policies are every processor.
unsafe fn message_affinity(IoDesc: PIO_RESOURCE_DESCRIPTOR) -> KAFFINITY {
    let targeted = (*IoDesc).u.Interrupt.TargetedProcessors;
    match (*IoDesc).u.Interrupt.AffinityPolicy as i32 {
        IRQ_POLICY_SPECIFIED_PROCESSORS if targeted != 0 => targeted,
        IRQ_POLICY_ONE_CLOSE_PROCESSOR => 1,
        _ => !0,
    }
}

/// Give the HAL back every message vector in a translated resource list, zeroing each
/// so a list released once (at REMOVE) and freed later releases nothing twice.
pub unsafe fn IopReleaseMessageInterrupts(List: PCM_RESOURCE_LIST) {
    if List.is_null() {
        return;
    }
    let mut full = core::ptr::addr_of_mut!((*List).List[0]);
    for _ in 0..(*List).Count {
        let partial = core::ptr::addr_of_mut!((*full).PartialResourceList);
        let first = core::ptr::addr_of_mut!((*partial).PartialDescriptors[0]);
        for k in 0..(*partial).Count as usize {
            let d = first.add(k);
            if (*d).Type == 2 && (*d).Flags & CM_RESOURCE_INTERRUPT_MESSAGE != 0 && (*d).u.Interrupt.Vector != 0 {
                HalVibeosMessageRelease((*d).u.Interrupt.Vector);
                (*d).u.Interrupt.Vector = 0;
            }
        }
        full = CmiGetNextResourceDescriptor(full);
    }
}

/// Drop a device's translated resources: message vectors back to the HAL, the list
/// freed. Every path that abandons a translation comes here; the C nulled the field
/// (and, on a failed translation, freed it but nulled ResourceList instead), leaking
/// the list or leaving it dangling.
pub unsafe fn IopDiscardTranslatedResources(DeviceNode: PDEVICE_NODE) {
    let list = (*DeviceNode).ResourceListTranslated;
    if list.is_null() {
        return;
    }
    IopReleaseMessageInterrupts(list);
    ExFreePool(list as PVOID);
    (*DeviceNode).ResourceListTranslated = core::ptr::null_mut();
}

unsafe fn IopFindInterruptResource(
    IoDesc: PIO_RESOURCE_DESCRIPTOR,
    CmDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> BOOLEAN {
    let mut Vector: ULONG;

    ntkrnl_sys::ASSERT!((*IoDesc).Type == (*CmDesc).Type);
    ntkrnl_sys::ASSERT!((*IoDesc).Type == 2); // CmResourceTypeInterrupt

    // A message-signaled interrupt is not a line to arbitrate: its vector is a
    // token (pci.sys asks for message k as CM_RESOURCE_INTERRUPT_MESSAGE_TOKEN - k)
    // that the HAL turns into a real vector at translation. Raw form, per
    // cmtypes.h's MessageInterrupt.Raw: MessageCount in the high half of the slot
    // a line keeps its Level in.
    if ((*IoDesc).Flags & CM_RESOURCE_INTERRUPT_MESSAGE) != 0 {
        (*CmDesc).u.Interrupt.Vector = (*IoDesc).u.Interrupt.MinimumVector;
        (*CmDesc).u.Interrupt.Level = 1 << 16;
        (*CmDesc).u.Interrupt.Affinity = message_affinity(IoDesc);
        return 1u8;
    }

    Vector = (*IoDesc).u.Interrupt.MinimumVector;
    while Vector <= (*IoDesc).u.Interrupt.MaximumVector {
        (*CmDesc).u.Interrupt.Vector = Vector;
        (*CmDesc).u.Interrupt.Level = Vector;
        (*CmDesc).u.Interrupt.Affinity = !0;

        if IopCheckDescriptorForConflict(CmDesc, core::ptr::null_mut()) == 0 {
            return 1u8;
        }
        Vector += 1;
    }

    0u8
}

#[no_mangle]
pub unsafe extern "system" fn IopFixupResourceListWithRequirements(
    RequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST,
    ResourceList: *mut PCM_RESOURCE_LIST,
) -> NTSTATUS {
    let ResourceList = opaque_ptr(ResourceList);
    let mut i: ULONG;
    let OldCount: ULONG;
    let mut AlternateRequired: BOOLEAN = 0u8;
    let mut ResList: PIO_RESOURCE_LIST;

    if !(*ResourceList).is_null() {
        OldCount = (**ResourceList).List[0].PartialResourceList.Count;
    } else {
        OldCount = 0;
    }

    ResList = core::ptr::addr_of_mut!((*RequirementsList).List[0]);
    i = 0;
    while i < (*RequirementsList).AlternativeLists {
        let mut ii: ULONG;

        if OldCount == 0 && !(*ResourceList).is_null() {
            ExFreePool(*ResourceList as PVOID);
            *ResourceList = core::ptr::null_mut();
        } else if OldCount != 0 {
            let NewList: PCM_RESOURCE_LIST;

            (**ResourceList).List[0].PartialResourceList.Count = OldCount;

            NewList = ExAllocatePool(
                PagedPool,
                PnpDetermineResourceListSize(*ResourceList) as _,
            ) as PCM_RESOURCE_LIST;
            if NewList.is_null() {
                return STATUS_NO_MEMORY;
            }

            core::ptr::copy_nonoverlapping(
                *ResourceList as *const u8,
                NewList as *mut u8,
                PnpDetermineResourceListSize(*ResourceList) as usize,
            );

            ExFreePool(*ResourceList as PVOID);

            *ResourceList = NewList;
        }

        ii = 0;
        while ii < (*ResList).Count {
            let mut iii: ULONG;
            let PartialList: PCM_PARTIAL_RESOURCE_LIST = if !(*ResourceList).is_null() {
                core::ptr::addr_of_mut!((**ResourceList).List[0].PartialResourceList)
            } else {
                core::ptr::null_mut()
            };
            let IoDesc: PIO_RESOURCE_DESCRIPTOR =
                core::ptr::addr_of_mut!((*ResList).Descriptors[0]).add(ii as usize);
            let mut Matched: BOOLEAN = 0u8;

            if AlternateRequired == 0 && ((*IoDesc).Option & 0x08) != 0 {
                ii += 1;
                continue;
            }

            if AlternateRequired != 0 && ((*IoDesc).Option & 0x08) == 0 {
                break;
            }

            iii = 0;
            while !PartialList.is_null() && iii < (*PartialList).Count && Matched == 0 {
                let CmDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
                    core::ptr::addr_of_mut!((*PartialList).PartialDescriptors[0]).add(iii as usize);

                if (*IoDesc).Type != (*CmDesc).Type {
                    iii += 1;
                    continue;
                }

                match (*IoDesc).Type as u32 {
                    2 => { // CmResourceTypeInterrupt
                        if (*CmDesc).u.Interrupt.Vector >= (*IoDesc).u.Interrupt.MinimumVector
                            && (*CmDesc).u.Interrupt.Vector <= (*IoDesc).u.Interrupt.MaximumVector
                        {
                            Matched = 1u8;
                        }
                    }
                    3 | 1 => { // CmResourceTypeMemory | CmResourceTypePort
                        if (*CmDesc).u.Memory.Length == (*IoDesc).u.Memory.Length
                            && (*CmDesc).u.Memory.Start.QuadPart as ULONGLONG
                                >= (*IoDesc).u.Memory.MinimumAddress.QuadPart as ULONGLONG
                            && ((*CmDesc).u.Memory.Start.QuadPart as ULONGLONG
                                + (*CmDesc).u.Memory.Length as ULONGLONG
                                - 1)
                                <= (*IoDesc).u.Memory.MaximumAddress.QuadPart as ULONGLONG
                        {
                            Matched = 1u8;
                        }
                    }
                    6 => { // CmResourceTypeBusNumber
                        if (*CmDesc).u.BusNumber.Length == (*IoDesc).u.BusNumber.Length
                            && (*CmDesc).u.BusNumber.Start >= (*IoDesc).u.BusNumber.MinBusNumber
                            && (*CmDesc).u.BusNumber.Start + (*CmDesc).u.BusNumber.Length - 1
                                <= (*IoDesc).u.BusNumber.MaxBusNumber
                        {
                            Matched = 1u8;
                        }
                    }
                    4 => { // CmResourceTypeDma
                        if (*CmDesc).u.Dma.Channel >= (*IoDesc).u.Dma.MinimumChannel
                            && (*CmDesc).u.Dma.Channel <= (*IoDesc).u.Dma.MaximumChannel
                        {
                            Matched = 1u8;
                        }
                    }
                    _ => {
                        Matched = 1u8;
                    }
                }
                iii += 1;
            }

            if Matched == 0 {
                let NewList: PCM_RESOURCE_LIST;
                let mut NewDesc: CM_PARTIAL_RESOURCE_DESCRIPTOR = core::mem::zeroed();
                let DescPtr: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
                let mut FoundResource: BOOLEAN = 1u8;

                NewDesc.Type = (*IoDesc).Type;
                NewDesc.Flags = (*IoDesc).Flags;
                NewDesc.ShareDisposition = (*IoDesc).ShareDisposition;

                match (*IoDesc).Type as u32 {
                    2 => { // CmResourceTypeInterrupt
                        if IopFindInterruptResource(IoDesc, &mut NewDesc) == 0 {
                            FoundResource = 0u8;
                        }
                    }
                    1 => { // CmResourceTypePort
                        if IopFindPortResource(IoDesc, &mut NewDesc) == 0 {
                            FoundResource = 0u8;
                        }
                    }
                    3 => { // CmResourceTypeMemory
                        if IopFindMemoryResource(IoDesc, &mut NewDesc) == 0 {
                            FoundResource = 0u8;
                        }
                    }
                    6 => { // CmResourceTypeBusNumber
                        if IopFindBusNumberResource(IoDesc, &mut NewDesc) == 0 {
                            FoundResource = 0u8;
                        }
                    }
                    4 => { // CmResourceTypeDma
                        if IopFindDmaResource(IoDesc, &mut NewDesc) == 0 {
                            FoundResource = 0u8;
                        }
                    }
                    _ => {
                        FoundResource = 0u8;
                    }
                }

                if FoundResource == 0 && (*IoDesc).Option == 0 {
                    break;
                } else if FoundResource == 0 {
                    AlternateRequired = 1u8;
                    ii += 1;
                    continue;
                } else {
                    AlternateRequired = 0u8;
                }

                if PartialList.is_null() {
                    NewList = ExAllocatePool(
                        PagedPool,
                        core::mem::size_of::<CM_RESOURCE_LIST>() as _,
                    ) as PCM_RESOURCE_LIST;
                    if NewList.is_null() {
                        return STATUS_NO_MEMORY;
                    }

                    (*NewList).Count = 1;
                    (*NewList).List[0].InterfaceType = (*RequirementsList).InterfaceType;
                    (*NewList).List[0].BusNumber = (*RequirementsList).BusNumber;
                    (*NewList).List[0].PartialResourceList.Version = 1;
                    (*NewList).List[0].PartialResourceList.Revision = 1;
                    (*NewList).List[0].PartialResourceList.Count = 1;

                    DescPtr = core::ptr::addr_of_mut!(
                        (*NewList).List[0].PartialResourceList.PartialDescriptors[0]
                    );
                } else {
                    NewList = ExAllocatePool(
                        PagedPool,
                        (PnpDetermineResourceListSize(*ResourceList) as usize
                            + core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>()) as _,
                    ) as PCM_RESOURCE_LIST;
                    if NewList.is_null() {
                        return STATUS_NO_MEMORY;
                    }

                    core::ptr::copy_nonoverlapping(
                        *ResourceList as *const u8,
                        NewList as *mut u8,
                        PnpDetermineResourceListSize(*ResourceList) as usize,
                    );

                    DescPtr = core::ptr::addr_of_mut!(
                        (*NewList).List[0].PartialResourceList.PartialDescriptors[0]
                    ).add((*NewList).List[0].PartialResourceList.Count as usize);

                    (*NewList).List[0].PartialResourceList.Count += 1;

                    ExFreePool(*ResourceList as PVOID);
                }

                *DescPtr = NewDesc;

                *ResourceList = NewList;
            }
            ii += 1;
        }

        if AlternateRequired != 0 {
            ResList = IopGetNextResourceList(ResList);
            i += 1;
            continue;
        }

        return STATUS_SUCCESS;
    }

    if !(*ResourceList).is_null() {
        ExFreePool(*ResourceList as PVOID);
        *ResourceList = core::ptr::null_mut();
    }

    STATUS_CONFLICTING_ADDRESSES
}

unsafe fn IopCheckResourceDescriptor(
    ResDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    ResourceList: PCM_RESOURCE_LIST,
    Silent: BOOLEAN,
    ConflictingDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> BOOLEAN {
    let mut i: ULONG;
    let mut ii: ULONG;
    let mut Result: BOOLEAN = 0u8;
    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR;

    // The C code returns `Silent ? Result : FALSE`, so when Silent=FALSE
    // the return value is always FALSE regardless of conflicts found.
    // Short-circuit here to avoid iterating potentially unvalidated registry data
    // that can contain garbage descriptor counts.
    if Silent == 0 {
        return 0u8;
    }

    FullDescriptor = core::ptr::addr_of_mut!((*ResourceList).List[0]);
    i = 0;
    while i < (*ResourceList).Count {
        let ResList: PCM_PARTIAL_RESOURCE_LIST =
            core::ptr::addr_of_mut!((*FullDescriptor).PartialResourceList);
        FullDescriptor = CmiGetNextResourceDescriptor(FullDescriptor);

        ii = 0;
        while ii < (*ResList).Count {
            let ResDesc2: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
                core::ptr::addr_of_mut!((*ResList).PartialDescriptors[0]).add(ii as usize);

            if (*ResDesc).ShareDisposition == 3 && (*ResDesc2).ShareDisposition == 3 { // CmResourceShareShared
                ii += 1;
                continue;
            }

            if (*ResDesc).Type != (*ResDesc2).Type {
                ii += 1;
                continue;
            }

            match (*ResDesc).Type as u32 {
                3 => { // CmResourceTypeMemory
                    let rStart = (*ResDesc).u.Memory.Start.QuadPart as UINT64;
                    let rEnd =
                        (*ResDesc).u.Memory.Start.QuadPart as UINT64 + (*ResDesc).u.Memory.Length as UINT64;
                    let r2Start = (*ResDesc2).u.Memory.Start.QuadPart as UINT64;
                    let r2End = (*ResDesc2).u.Memory.Start.QuadPart as UINT64
                        + (*ResDesc2).u.Memory.Length as UINT64;

                    if rStart < r2End && r2Start < rEnd {
                        Result = 1u8;
                        break;
                    }
                }
                1 => { // CmResourceTypePort
                    let rStart = (*ResDesc).u.Port.Start.QuadPart as UINT64;
                    let rEnd =
                        (*ResDesc).u.Port.Start.QuadPart as UINT64 + (*ResDesc).u.Port.Length as UINT64;
                    let r2Start = (*ResDesc2).u.Port.Start.QuadPart as UINT64;
                    let r2End =
                        (*ResDesc2).u.Port.Start.QuadPart as UINT64 + (*ResDesc2).u.Port.Length as UINT64;

                    if rStart < r2End && r2Start < rEnd {
                        Result = 1u8;
                        break;
                    }
                }
                2 => { // CmResourceTypeInterrupt
                    // Message tokens repeat across devices by design; the HAL hands
                    // each its own vector, so they never conflict.
                    let message = ((*ResDesc).Flags | (*ResDesc2).Flags) & CM_RESOURCE_INTERRUPT_MESSAGE;
                    if message == 0 && (*ResDesc).u.Interrupt.Vector == (*ResDesc2).u.Interrupt.Vector {
                        Result = 1u8;
                        break;
                    }
                }
                6 => { // CmResourceTypeBusNumber
                    let rStart = (*ResDesc).u.BusNumber.Start;
                    let rEnd = (*ResDesc).u.BusNumber.Start + (*ResDesc).u.BusNumber.Length;
                    let r2Start = (*ResDesc2).u.BusNumber.Start;
                    let r2End = (*ResDesc2).u.BusNumber.Start + (*ResDesc2).u.BusNumber.Length;

                    if rStart < r2End && r2Start < rEnd {
                        Result = 1u8;
                        break;
                    }
                }
                4 => { // CmResourceTypeDma
                    if (*ResDesc).u.Dma.Channel == (*ResDesc2).u.Dma.Channel {
                        Result = 1u8;
                        break;
                    }
                }
                _ => {}
            }
            ii += 1;
        }

        if Result != 0 {
            break;
        }
        i += 1;
    }

    if Result != 0 && !ConflictingDescriptor.is_null() {
        core::ptr::copy_nonoverlapping(
            ResDesc as *const u8,
            ConflictingDescriptor as *mut u8,
            core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
        );
    }

    if Silent != 0 {
        Result
    } else {
        0u8
    }
}

unsafe fn IopUpdateControlKeyWithResources(DeviceNode: PDEVICE_NODE) -> NTSTATUS {
    let mut EnumRoot: _UNICODE_STRING = core::mem::zeroed();
    let mut Control: _UNICODE_STRING = core::mem::zeroed();
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut EnumKey: HANDLE = core::ptr::null_mut();
    let mut InstanceKey: HANDLE = core::ptr::null_mut();
    let mut ControlKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    static ENUM_ROOT: [u16; 50] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
        b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16,
        b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16,
        b'S' as u16, b'y' as u16, b's' as u16, b't' as u16, b'e' as u16, b'm' as u16,
        b'\\' as u16, b'C' as u16, b'u' as u16, b'r' as u16, b'r' as u16, b'e' as u16,
        b'n' as u16, b't' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16,
        b'r' as u16, b'o' as u16, b'l' as u16, b'S' as u16, b'e' as u16, b't' as u16,
        b'\\' as u16, b'E' as u16, b'n' as u16, b'u' as u16, b'm' as u16, 0, 0, 0,
    ];
    // \Registry\Machine\System\CurrentControlSet\Enum = 47 chars + NUL
    RtlInitUnicodeString(&mut EnumRoot, ENUM_ROOT.as_ptr());
    static CONTROL: [u16; 8] = [b'C' as u16, b'o' as u16, b'n' as u16, b't' as u16, b'r' as u16, b'o' as u16, b'l' as u16, 0];
    RtlInitUnicodeString(&mut Control, CONTROL.as_ptr());
    static ALLOC_CONFIG: [u16; 12] = [b'A' as u16, b'l' as u16, b'l' as u16, b'o' as u16, b'c' as u16, b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16, b'i' as u16, b'g' as u16, 0];
    RtlInitUnicodeString(&mut ValueName, ALLOC_CONFIG.as_ptr());

    Status = IopOpenRegistryKeyEx(
        &mut EnumKey,
        core::ptr::null_mut(),
        &mut EnumRoot,
        0x0008,
    );
    if Status < 0 {
        return Status;
    }

    Status = IopOpenRegistryKeyEx(
        &mut InstanceKey,
        EnumKey,
        core::ptr::addr_of_mut!((*DeviceNode).InstancePath),
        0x0008,
    );
    ZwClose(EnumKey);

    if Status < 0 {
        return Status;
    }

    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut Control,
        0x00000040 | 0x00000200, // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
        InstanceKey,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut ControlKey,
        0x00000002,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0x00000001,
        core::ptr::null_mut(),
    );
    ZwClose(InstanceKey);

    if Status < 0 {
        return Status;
    }

    Status = ZwSetValueKey(
        ControlKey,
        &mut ValueName,
        0,
        8,
        (*DeviceNode).ResourceList as PVOID,
        PnpDetermineResourceListSize((*DeviceNode).ResourceList),
    );
    ZwClose(ControlKey);

    if Status < 0 {
        return Status;
    }

    STATUS_SUCCESS
}

unsafe fn IopFilterResourceRequirements(DeviceNode: PDEVICE_NODE) -> NTSTATUS {
    let mut Stack: IO_STACK_LOCATION = core::mem::zeroed();
    let mut IoStatusBlock: _IO_STATUS_BLOCK = core::mem::zeroed();
    let Status: NTSTATUS;

    Stack.Parameters.FilterResourceRequirements.IoResourceRequirementList =
        (*DeviceNode).ResourceRequirements;
    Status = IopInitiatePnpIrp(
        (*DeviceNode).PhysicalDeviceObject,
        &mut IoStatusBlock,
        0x0D,
        &mut Stack,
    );
    if Status < 0 && Status != STATUS_NOT_SUPPORTED {
        return Status;
    } else if Status >= 0 && IoStatusBlock.Information != 0 {
        (*DeviceNode).ResourceRequirements =
            IoStatusBlock.Information as PIO_RESOURCE_REQUIREMENTS_LIST;
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IopUpdateResourceMap(
    DeviceNode: PDEVICE_NODE,
    Level1Key: *mut u16,
    Level2Key: *mut u16,
) -> NTSTATUS {
    let Level1Key = opaque_ptr(Level1Key);
    let Level2Key = opaque_ptr(Level2Key);
    let mut Status: NTSTATUS;
    let mut Disposition: ULONG = 0;
    let mut PnpMgrLevel1: HANDLE = core::ptr::null_mut();
    let mut PnpMgrLevel2: HANDLE = core::ptr::null_mut();
    let mut ResourceMapKey: HANDLE = core::ptr::null_mut();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();

    static RESOURCEMAP_KEY: [u16; 39] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
        b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16,
        b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16,
        b'H' as u16, b'A' as u16, b'R' as u16, b'D' as u16, b'W' as u16, b'A' as u16,
        b'R' as u16, b'E' as u16, b'\\' as u16, b'R' as u16, b'E' as u16, b'S' as u16,
        b'O' as u16, b'U' as u16, b'R' as u16, b'C' as u16, b'E' as u16, b'M' as u16,
        b'A' as u16, b'P' as u16, 0,
    ];
    RtlInitUnicodeString(&mut KeyName, RESOURCEMAP_KEY.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        0x00000040 | 0x00000080 | 0x00000200, // OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut ResourceMapKey,
        0xF003F,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0x00000001,
        &mut Disposition,
    );
    if Status < 0 {
        return Status;
    }

    RtlInitUnicodeString(&mut KeyName, Level1Key);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        0x00000040 | 0x00000080 | 0x00000200, // OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE
        ResourceMapKey,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut PnpMgrLevel1,
        0xF003F,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0x00000001,
        &mut Disposition,
    );
    ZwClose(ResourceMapKey);
    if Status < 0 {
        return Status;
    }

    RtlInitUnicodeString(&mut KeyName, Level2Key);
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        0x00000040 | 0x00000080 | 0x00000200, // OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE
        PnpMgrLevel1,
        core::ptr::null_mut(),
    );
    Status = ZwCreateKey(
        &mut PnpMgrLevel2,
        0xF003F,
        &mut ObjectAttributes,
        0,
        core::ptr::null_mut(),
        0x00000001,
        &mut Disposition,
    );
    ZwClose(PnpMgrLevel1);
    if Status < 0 {
        return Status;
    }

    if !(*DeviceNode).ResourceList.is_null() {
        let mut NameU: _UNICODE_STRING = core::mem::zeroed();
        let mut RawSuffix: _UNICODE_STRING = core::mem::zeroed();
        let mut TranslatedSuffix: _UNICODE_STRING = core::mem::zeroed();
        let mut OldLength: ULONG = 0;

        ntkrnl_sys::ASSERT!(!(*DeviceNode).ResourceListTranslated.is_null());

        static TRANSLATED_SUFFIX: [u16; 12] = [b'.' as u16, b'T' as u16, b'r' as u16, b'a' as u16, b'n' as u16, b's' as u16, b'l' as u16, b'a' as u16, b't' as u16, b'e' as u16, b'd' as u16, 0];
        RtlInitUnicodeString(&mut TranslatedSuffix, TRANSLATED_SUFFIX.as_ptr());
        static RAW_SUFFIX: [u16; 5] = [b'.' as u16, b'R' as u16, b'a' as u16, b'w' as u16, 0];
        RtlInitUnicodeString(&mut RawSuffix, RAW_SUFFIX.as_ptr());

        Status = IoGetDeviceProperty(
            (*DeviceNode).PhysicalDeviceObject,
            0xb, // DevicePropertyPhysicalDeviceObjectName
            0,
            core::ptr::null_mut(),
            &mut OldLength,
        );
        if Status == 0x80000005u32 as i32 || Status == 0xC0000023u32 as i32 {
            ntkrnl_sys::ASSERT!(OldLength != 0);

            NameU.Buffer = ExAllocatePool(PagedPool, (OldLength + TranslatedSuffix.Length as ULONG) as _)
                as *mut u16;
            if NameU.Buffer.is_null() {
                ZwClose(PnpMgrLevel2);
                return STATUS_INSUFFICIENT_RESOURCES;
            }

            NameU.Length = 0;
            NameU.MaximumLength = OldLength as u16 + TranslatedSuffix.Length;

            Status = IoGetDeviceProperty(
                (*DeviceNode).PhysicalDeviceObject,
                0xb, // DevicePropertyPhysicalDeviceObjectName
                NameU.MaximumLength as ULONG,
                NameU.Buffer as PVOID,
                &mut OldLength,
            );
            if Status < 0 {
                ZwClose(PnpMgrLevel2);
                ExFreePool(NameU.Buffer as PVOID);
                return Status;
            }
        } else if Status < 0 {
            ZwClose(PnpMgrLevel2);
            return Status;
        } else {
            ntkrnl_sys::ASSERT!(false);
        }

        NameU.Length = OldLength as u16 - core::mem::size_of::<u16>() as u16;

        RtlAppendUnicodeStringToString(&mut NameU, &mut RawSuffix);

        Status = ZwSetValueKey(
            PnpMgrLevel2,
            &mut NameU,
            0,
            8,
            (*DeviceNode).ResourceList as PVOID,
            PnpDetermineResourceListSize((*DeviceNode).ResourceList),
        );
        if Status < 0 {
            ZwClose(PnpMgrLevel2);
            ExFreePool(NameU.Buffer as PVOID);
            return Status;
        }

        NameU.Length = OldLength as u16 - core::mem::size_of::<u16>() as u16;

        RtlAppendUnicodeStringToString(&mut NameU, &mut TranslatedSuffix);

        Status = ZwSetValueKey(
            PnpMgrLevel2,
            &mut NameU,
            0,
            8,
            (*DeviceNode).ResourceListTranslated as PVOID,
            PnpDetermineResourceListSize((*DeviceNode).ResourceListTranslated),
        );
        ZwClose(PnpMgrLevel2);
        ExFreePool(NameU.Buffer as PVOID);

        if Status < 0 {
            return Status;
        }
    } else {
        ZwClose(PnpMgrLevel2);
    }

    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "C" fn IopUpdateResourceMapForPnPDevice(
    DeviceNode: PDEVICE_NODE,
) -> NTSTATUS {
    static PNP_MANAGER: [u16; 12] = [b'P' as u16, b'n' as u16, b'P' as u16, b' ' as u16, b'M' as u16, b'a' as u16, b'n' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, 0];
    static PNPMANAGER: [u16; 11] = [b'P' as u16, b'n' as u16, b'p' as u16, b'M' as u16, b'a' as u16, b'n' as u16, b'a' as u16, b'g' as u16, b'e' as u16, b'r' as u16, 0];
    IopUpdateResourceMap(
        DeviceNode,
        PNP_MANAGER.as_ptr() as *mut u16,
        PNPMANAGER.as_ptr() as *mut u16,
    )
}

unsafe fn IopTranslateDeviceResources(DeviceNode: PDEVICE_NODE) -> NTSTATUS {
    let mut pPartialResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let mut DescriptorRaw: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut DescriptorTranslated: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR;
    let mut i: ULONG;
    let mut j: ULONG;
    let ListSize: ULONG;
    let mut Status: NTSTATUS;

    if (*DeviceNode).ResourceList.is_null() {
        IopDiscardTranslatedResources(DeviceNode);
        return STATUS_SUCCESS;
    }

    ListSize = PnpDetermineResourceListSize((*DeviceNode).ResourceList);

    // A device translated before (assigned again after a failed start) gives its
    // last translation's message vectors back first.
    IopDiscardTranslatedResources(DeviceNode);
    (*DeviceNode).ResourceListTranslated =
        ExAllocatePool(PagedPool, ListSize as _) as PCM_RESOURCE_LIST;
    if (*DeviceNode).ResourceListTranslated.is_null() {
        Status = STATUS_NO_MEMORY;
        return Status;
    }
    core::ptr::copy_nonoverlapping(
        (*DeviceNode).ResourceList as *const u8,
        (*DeviceNode).ResourceListTranslated as *mut u8,
        ListSize as usize,
    );

    FullDescriptor = core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]);
    i = 0;
    while i < (*(*DeviceNode).ResourceList).Count {
        pPartialResourceList = core::ptr::addr_of_mut!((*FullDescriptor).PartialResourceList);
        FullDescriptor = CmiGetNextResourceDescriptor(FullDescriptor);

        j = 0;
        while j < (*pPartialResourceList).Count {
            DescriptorRaw =
                core::ptr::addr_of_mut!((*pPartialResourceList).PartialDescriptors[0]).add(j as usize);

            DescriptorTranslated = (((*DeviceNode).ResourceListTranslated as *mut u8).add(
                (DescriptorRaw as *const u8)
                    .offset_from((*DeviceNode).ResourceList as *const u8)
                    as usize,
            )) as PCM_PARTIAL_RESOURCE_DESCRIPTOR;

            crate::dbg_print!(
                b"PNPRES: xlat desc Type=%d IF=%d Bus=%d Lvl=0x%x Vec=0x%x\n\0",
                (*DescriptorRaw).Type as u32,
                (*core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]).add(i as usize)).InterfaceType,
                (*core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]).add(i as usize)).BusNumber,
                (*DescriptorRaw).u.Interrupt.Level,
                (*DescriptorRaw).u.Interrupt.Vector
            );
            match (*DescriptorRaw).Type as u32 {
                1 => { // CmResourceTypePort
                    let mut AddressSpace: ULONG = 1;
                    if HalTranslateBusAddress(
                        (*core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]).add(i as usize)).InterfaceType,
                        (*core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]).add(i as usize)).BusNumber,
                        (*DescriptorRaw).u.Port.Start,
                        &mut AddressSpace,
                        core::ptr::addr_of_mut!((*DescriptorTranslated).u.Port.Start),
                    ) == 0
                    {
                        Status = 0xC0000001u32 as i32;
                        ExFreePool((*DeviceNode).ResourceList as PVOID);
                        (*DeviceNode).ResourceList = core::ptr::null_mut();
                        IopDiscardTranslatedResources(DeviceNode);
                        return Status;
                    }

                    if AddressSpace == 0 {
                        (*DescriptorRaw).Flags = 0x0000; // CM_RESOURCE_PORT_MEMORY
                        (*DescriptorTranslated).Flags = 0x0000;
                    }
                }
                2 if (*DescriptorRaw).Flags & CM_RESOURCE_INTERRUPT_MESSAGE != 0 => {
                    // A message token: the HAL hands out a vector and the CPU it is
                    // delivered to, from the processors the raw descriptor allows.
                    let mut Irql: KIRQL = 0;
                    (*DescriptorTranslated).u.Interrupt.Vector = HalVibeosMessageAllocate(
                        (*DescriptorRaw).u.Interrupt.Vector,
                        (*DescriptorRaw).u.Interrupt.Affinity,
                        &mut Irql,
                        core::ptr::addr_of_mut!((*DescriptorTranslated).u.Interrupt.Affinity),
                    );
                    (*DescriptorTranslated).u.Interrupt.Level = Irql as ULONG;
                    if (*DescriptorTranslated).u.Interrupt.Vector == 0 {
                        Status = 0xC0000001u32 as i32;
                        ExFreePool((*DeviceNode).ResourceList as PVOID);
                        (*DeviceNode).ResourceList = core::ptr::null_mut();
                        IopDiscardTranslatedResources(DeviceNode);
                        return Status;
                    }
                }
                2 => { // CmResourceTypeInterrupt
                    let mut Irql: KIRQL = 0;
                    (*DescriptorTranslated).u.Interrupt.Vector = HalGetInterruptVector(
                        (*core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]).add(i as usize)).InterfaceType,
                        (*core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]).add(i as usize)).BusNumber,
                        (*DescriptorRaw).u.Interrupt.Level,
                        (*DescriptorRaw).u.Interrupt.Vector,
                        &mut Irql,
                        core::ptr::addr_of_mut!((*DescriptorTranslated).u.Interrupt.Affinity),
                    );
                    (*DescriptorTranslated).u.Interrupt.Level = Irql as ULONG;
                    crate::dbg_print!(
                        b"PNPRES: HalGetIntVec -> Vec=0x%x Irql=%d\n\0",
                        (*DescriptorTranslated).u.Interrupt.Vector,
                        Irql as u32
                    );
                    if (*DescriptorTranslated).u.Interrupt.Vector == 0 {
                        Status = 0xC0000001u32 as i32;
                        ExFreePool((*DeviceNode).ResourceList as PVOID);
                        (*DeviceNode).ResourceList = core::ptr::null_mut();
                        IopDiscardTranslatedResources(DeviceNode);
                        return Status;
                    }
                }
                3 => { // CmResourceTypeMemory
                    let mut AddressSpace: ULONG = 0;
                    if HalTranslateBusAddress(
                        (*core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]).add(i as usize)).InterfaceType,
                        (*core::ptr::addr_of_mut!((*(*DeviceNode).ResourceList).List[0]).add(i as usize)).BusNumber,
                        (*DescriptorRaw).u.Memory.Start,
                        &mut AddressSpace,
                        core::ptr::addr_of_mut!((*DescriptorTranslated).u.Memory.Start),
                    ) == 0
                    {
                        Status = 0xC0000001u32 as i32;
                        ExFreePool((*DeviceNode).ResourceList as PVOID);
                        (*DeviceNode).ResourceList = core::ptr::null_mut();
                        IopDiscardTranslatedResources(DeviceNode);
                        return Status;
                    }

                    if AddressSpace != 0 {
                        ntkrnl_sys::ASSERT!(false);
                    }
                }

                4 | 6 | 129 | 5 => {} // Dma, BusNumber, DevicePrivate, DeviceSpecific
                _ => {
                    Status = STATUS_NOT_IMPLEMENTED;
                    ExFreePool((*DeviceNode).ResourceList as PVOID);
                    (*DeviceNode).ResourceList = core::ptr::null_mut();
                    IopDiscardTranslatedResources(DeviceNode);
                    return Status;
                }
            }
            j += 1;
        }
        i += 1;
    }
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn IopAssignDeviceResources(DeviceNode: PDEVICE_NODE) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let ListSize: ULONG;
    crate::dbg_print!(b"PNPRES: Assign Boot=%p Req=%p\n\0", (*DeviceNode).BootResources, (*DeviceNode).ResourceRequirements);

    Status = IopFilterResourceRequirements(DeviceNode);
    crate::dbg_print!(b"PNPRES: filter=0x%x\n\0", Status);
    if Status < 0 {
        if !(*DeviceNode).ResourceList.is_null() {
            ExFreePool((*DeviceNode).ResourceList as PVOID);
            (*DeviceNode).ResourceList = core::ptr::null_mut();
        }
        IopDiscardTranslatedResources(DeviceNode);
        return Status;
    }

    if (*DeviceNode).BootResources.is_null() && (*DeviceNode).ResourceRequirements.is_null() {
        (*DeviceNode).ResourceList = core::ptr::null_mut();
        IopDiscardTranslatedResources(DeviceNode);
        PiSetDevNodeState(DeviceNode, 0x304);
        (*DeviceNode).Flags |= 0x00000100; // DNF_NO_RESOURCE_REQUIRED

        return STATUS_SUCCESS;
    }

    if !(*DeviceNode).BootResources.is_null() {
        ListSize = PnpDetermineResourceListSize((*DeviceNode).BootResources);

        (*DeviceNode).ResourceList = ExAllocatePool(PagedPool, ListSize as _) as PCM_RESOURCE_LIST;
        if (*DeviceNode).ResourceList.is_null() {
            Status = STATUS_NO_MEMORY;
            if !(*DeviceNode).ResourceList.is_null() {
                ExFreePool((*DeviceNode).ResourceList as PVOID);
                (*DeviceNode).ResourceList = core::ptr::null_mut();
            }
            IopDiscardTranslatedResources(DeviceNode);
            return Status;
        }

        core::ptr::copy_nonoverlapping(
            (*DeviceNode).BootResources as *const u8,
            (*DeviceNode).ResourceList as *mut u8,
            ListSize as usize,
        );

        crate::dbg_print!(b"PNPRES: detecting conflict\n\0");
        Status = IopDetectResourceConflict((*DeviceNode).ResourceList, 0u8, core::ptr::null_mut());
        crate::dbg_print!(b"PNPRES: conflict=0x%x\n\0", Status);
        if Status < 0 {
            ExFreePool((*DeviceNode).ResourceList as PVOID);
            (*DeviceNode).ResourceList = core::ptr::null_mut();
        }
    } else {
        (*DeviceNode).ResourceList = core::ptr::null_mut();
    }

    if (*DeviceNode).ResourceRequirements.is_null() {
        Status = IopTranslateDeviceResources(DeviceNode);
        if Status < 0 {
            (*DeviceNode).Problem = 0x21; // CM_PROB_TRANSLATION_FAILED
            if !(*DeviceNode).ResourceList.is_null() {
                ExFreePool((*DeviceNode).ResourceList as PVOID);
                (*DeviceNode).ResourceList = core::ptr::null_mut();
            }
            IopDiscardTranslatedResources(DeviceNode);
            return Status;
        }

        Status = IopUpdateResourceMapForPnPDevice(DeviceNode);
        if Status < 0 {
            if !(*DeviceNode).ResourceList.is_null() {
                ExFreePool((*DeviceNode).ResourceList as PVOID);
                (*DeviceNode).ResourceList = core::ptr::null_mut();
            }
            IopDiscardTranslatedResources(DeviceNode);
            return Status;
        }

        Status = IopUpdateControlKeyWithResources(DeviceNode);
        if Status < 0 {
            if !(*DeviceNode).ResourceList.is_null() {
                ExFreePool((*DeviceNode).ResourceList as PVOID);
                (*DeviceNode).ResourceList = core::ptr::null_mut();
            }
            IopDiscardTranslatedResources(DeviceNode);
            return Status;
        }

        PiSetDevNodeState(DeviceNode, 0x304);
        return STATUS_SUCCESS;
    }

    HalAdjustResourceList(core::ptr::addr_of_mut!((*DeviceNode).ResourceRequirements));

    crate::dbg_print!(b"PNPRES: calling Fixup\n\0");
    Status = IopFixupResourceListWithRequirements(
        (*DeviceNode).ResourceRequirements,
        core::ptr::addr_of_mut!((*DeviceNode).ResourceList),
    );
    crate::dbg_print!(b"PNPRES: Fixup returned 0x%x\n\0", Status);
    if Status < 0 {
        (*DeviceNode).Problem = 12;
        if !(*DeviceNode).ResourceList.is_null() {
            ExFreePool((*DeviceNode).ResourceList as PVOID);
            (*DeviceNode).ResourceList = core::ptr::null_mut();
        }
        IopDiscardTranslatedResources(DeviceNode);
        return Status;
    }

    ntkrnl_sys::ASSERT!(IopDetectResourceConflict((*DeviceNode).ResourceList, 0u8, core::ptr::null_mut())
            != STATUS_CONFLICTING_ADDRESSES);

    crate::dbg_print!(b"PNPRES: translating\n\0");
    Status = IopTranslateDeviceResources(DeviceNode);
    crate::dbg_print!(b"PNPRES: translate=0x%x\n\0", Status);
    if Status < 0 {
        (*DeviceNode).Problem = 0x21; // CM_PROB_TRANSLATION_FAILED
        if !(*DeviceNode).ResourceList.is_null() {
            ExFreePool((*DeviceNode).ResourceList as PVOID);
            (*DeviceNode).ResourceList = core::ptr::null_mut();
        }
        IopDiscardTranslatedResources(DeviceNode);
        return Status;
    }

    crate::dbg_print!(b"PNPRES: updating resource map\n\0");
    Status = IopUpdateResourceMapForPnPDevice(DeviceNode);
    crate::dbg_print!(b"PNPRES: map=0x%x\n\0", Status);
    if Status < 0 {
        if !(*DeviceNode).ResourceList.is_null() {
            ExFreePool((*DeviceNode).ResourceList as PVOID);
            (*DeviceNode).ResourceList = core::ptr::null_mut();
        }
        IopDiscardTranslatedResources(DeviceNode);
        return Status;
    }

    Status = IopUpdateControlKeyWithResources(DeviceNode);
    if Status < 0 {
        if !(*DeviceNode).ResourceList.is_null() {
            ExFreePool((*DeviceNode).ResourceList as PVOID);
            (*DeviceNode).ResourceList = core::ptr::null_mut();
        }
        IopDiscardTranslatedResources(DeviceNode);
        return Status;
    }

    PiSetDevNodeState(DeviceNode, 0x304);

    STATUS_SUCCESS
}

unsafe fn IopCheckForResourceConflict(
    ResourceList1: PCM_RESOURCE_LIST,
    ResourceList2: PCM_RESOURCE_LIST,
    Silent: BOOLEAN,
    ConflictingDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> BOOLEAN {
    let mut i: ULONG;
    let mut ii: ULONG;
    let mut Result: BOOLEAN = 0u8;
    let mut FullDescriptor: PCM_FULL_RESOURCE_DESCRIPTOR;

    FullDescriptor = core::ptr::addr_of_mut!((*ResourceList1).List[0]);
    i = 0;
    while i < (*ResourceList1).Count {
        let ResList: PCM_PARTIAL_RESOURCE_LIST =
            core::ptr::addr_of_mut!((*FullDescriptor).PartialResourceList);
        FullDescriptor = CmiGetNextResourceDescriptor(FullDescriptor);

        ii = 0;
        while ii < (*ResList).Count {
            let ResDesc: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
                core::ptr::addr_of_mut!((*ResList).PartialDescriptors[0]).add(ii as usize);

            Result = IopCheckResourceDescriptor(ResDesc, ResourceList2, Silent, ConflictingDescriptor);
            if Result != 0 {
                return Result;
            }
            ii += 1;
        }
        i += 1;
    }

    Result
}

#[no_mangle]
pub unsafe extern "system" fn IopDetectResourceConflict(
    ResourceList: PCM_RESOURCE_LIST,
    Silent: BOOLEAN,
    ConflictingDescriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) -> NTSTATUS {
    // When Silent=FALSE, the C code's IopCheckResourceDescriptor always returns FALSE
    // (due to `return Silent ? Result : FALSE` hack), so IopDetectResourceConflict
    // never finds conflicts. Short-circuit to avoid expensive registry enumeration.
    if Silent == 0 {
        return STATUS_SUCCESS;
    }
    let mut ObjectAttributes: _OBJECT_ATTRIBUTES = core::mem::zeroed();
    let mut KeyName: _UNICODE_STRING = core::mem::zeroed();
    let mut ResourceMapKey: HANDLE = core::ptr::null_mut();
    let mut ChildKey2: HANDLE = core::ptr::null_mut();
    let mut ChildKey3: HANDLE = core::ptr::null_mut();
    let mut KeyInformationLength: ULONG;
    let mut RequiredLength: ULONG = 0;
    let mut KeyValueInformationLength: ULONG;
    let mut KeyNameInformationLength: ULONG;
    let mut KeyInformation: PKEY_BASIC_INFORMATION;
    let mut KeyValueInformation: PKEY_VALUE_PARTIAL_INFORMATION;
    let mut KeyNameInformation: PKEY_VALUE_BASIC_INFORMATION;
    let mut ChildKeyIndex1: ULONG = 0;
    let mut ChildKeyIndex2: ULONG;
    let mut ChildKeyIndex3: ULONG;
    let mut Status: NTSTATUS;

    static RESOURCEMAP_KEY2: [u16; 39] = [
        b'\\' as u16, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
        b't' as u16, b'r' as u16, b'y' as u16, b'\\' as u16, b'M' as u16, b'a' as u16,
        b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, b'\\' as u16,
        b'H' as u16, b'A' as u16, b'R' as u16, b'D' as u16, b'W' as u16, b'A' as u16,
        b'R' as u16, b'E' as u16, b'\\' as u16, b'R' as u16, b'E' as u16, b'S' as u16,
        b'O' as u16, b'U' as u16, b'R' as u16, b'C' as u16, b'E' as u16, b'M' as u16,
        b'A' as u16, b'P' as u16, 0,
    ];
    RtlInitUnicodeString(&mut KeyName, RESOURCEMAP_KEY2.as_ptr());
    InitializeObjectAttributes(
        &mut ObjectAttributes,
        &mut KeyName,
        0x00000040 | 0x00000200, // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    );
    Status = ZwOpenKey(&mut ResourceMapKey, 0x0008 | 0x0001, &mut ObjectAttributes);
    if Status < 0 {
        return STATUS_SUCCESS;
    }

    loop {
        Status = ZwEnumerateKey(
            ResourceMapKey,
            ChildKeyIndex1,
            0,
            core::ptr::null_mut(),
            0,
            &mut RequiredLength,
        );
        if Status == 0x8000001Au32 as i32 {
            break;
        } else if Status == 0x80000005u32 as i32 || Status == 0xC0000023u32 as i32 {
            KeyInformationLength = RequiredLength;
            KeyInformation = ExAllocatePoolWithTag(PagedPool, KeyInformationLength as usize, 0x206F4943)
                as PKEY_BASIC_INFORMATION;
            if KeyInformation.is_null() {
                Status = STATUS_INSUFFICIENT_RESOURCES;
                if !ResourceMapKey.is_null() {
                    ObCloseHandle(ResourceMapKey, 0i8);
                }
                if !ChildKey2.is_null() {
                    ObCloseHandle(ChildKey2, 0i8);
                }
                if !ChildKey3.is_null() {
                    ObCloseHandle(ChildKey3, 0i8);
                }
                if Status == 0x8000001Au32 as i32 {
                    Status = STATUS_SUCCESS;
                }
                return Status;
            }

            Status = ZwEnumerateKey(
                ResourceMapKey,
                ChildKeyIndex1,
                0,
                KeyInformation as PVOID,
                KeyInformationLength,
                &mut RequiredLength,
            );
        } else {
            if !ResourceMapKey.is_null() {
                ObCloseHandle(ResourceMapKey, 0i8);
            }
            if !ChildKey2.is_null() {
                ObCloseHandle(ChildKey2, 0i8);
            }
            if !ChildKey3.is_null() {
                ObCloseHandle(ChildKey3, 0i8);
            }
            if Status == 0x8000001Au32 as i32 {
                Status = STATUS_SUCCESS;
            }
            return Status;
        }
        ChildKeyIndex1 += 1;
        if Status < 0 {
            ExFreePoolWithTag(KeyInformation as PVOID, 0x206F4943);
            if !ResourceMapKey.is_null() {
                ObCloseHandle(ResourceMapKey, 0i8);
            }
            if !ChildKey2.is_null() {
                ObCloseHandle(ChildKey2, 0i8);
            }
            if !ChildKey3.is_null() {
                ObCloseHandle(ChildKey3, 0i8);
            }
            if Status == 0x8000001Au32 as i32 {
                Status = STATUS_SUCCESS;
            }
            return Status;
        }

        // Close handle from previous outer loop iteration to prevent leak
        if !ChildKey2.is_null() {
            ObCloseHandle(ChildKey2, 0i8);
            ChildKey2 = core::ptr::null_mut();
        }

        KeyName.Buffer = (*KeyInformation).Name.as_mut_ptr();
        KeyName.MaximumLength = (*KeyInformation).NameLength as u16;
        KeyName.Length = (*KeyInformation).NameLength as u16;
        InitializeObjectAttributes(
            &mut ObjectAttributes,
            &mut KeyName,
            0x00000040 | 0x00000200, // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
            ResourceMapKey,
            core::ptr::null_mut(),
        );
        Status = ZwOpenKey(&mut ChildKey2, 0x0008 | 0x0001, &mut ObjectAttributes);
        ExFreePoolWithTag(KeyInformation as PVOID, 0x206F4943);
        if Status < 0 {
            if !ResourceMapKey.is_null() {
                ObCloseHandle(ResourceMapKey, 0i8);
            }
            if !ChildKey2.is_null() {
                ObCloseHandle(ChildKey2, 0i8);
            }
            if !ChildKey3.is_null() {
                ObCloseHandle(ChildKey3, 0i8);
            }
            if Status == 0x8000001Au32 as i32 {
                Status = STATUS_SUCCESS;
            }
            return Status;
        }

        ChildKeyIndex2 = 0;
        loop {
            Status = ZwEnumerateKey(
                ChildKey2,
                ChildKeyIndex2,
                0,
                core::ptr::null_mut(),
                0,
                &mut RequiredLength,
            );
            if Status == 0x8000001Au32 as i32 {
                break;
            } else if Status == 0xC0000023u32 as i32 {
                KeyInformationLength = RequiredLength;
                KeyInformation =
                    ExAllocatePoolWithTag(PagedPool, KeyInformationLength as usize, 0x206F4943)
                        as PKEY_BASIC_INFORMATION;
                if KeyInformation.is_null() {
                    Status = STATUS_INSUFFICIENT_RESOURCES;
                    if !ResourceMapKey.is_null() {
                        ObCloseHandle(ResourceMapKey, 0i8);
                    }
                    if !ChildKey2.is_null() {
                        ObCloseHandle(ChildKey2, 0i8);
                    }
                    if !ChildKey3.is_null() {
                        ObCloseHandle(ChildKey3, 0i8);
                    }
                    if Status == 0x8000001Au32 as i32 {
                        Status = STATUS_SUCCESS;
                    }
                    return Status;
                }

                Status = ZwEnumerateKey(
                    ChildKey2,
                    ChildKeyIndex2,
                    0,
                    KeyInformation as PVOID,
                    KeyInformationLength,
                    &mut RequiredLength,
                );
            } else {
                if !ResourceMapKey.is_null() {
                    ObCloseHandle(ResourceMapKey, 0i8);
                }
                if !ChildKey2.is_null() {
                    ObCloseHandle(ChildKey2, 0i8);
                }
                if !ChildKey3.is_null() {
                    ObCloseHandle(ChildKey3, 0i8);
                }
                if Status == 0x8000001Au32 as i32 {
                    Status = STATUS_SUCCESS;
                }
                return Status;
            }
            ChildKeyIndex2 += 1;
            if Status < 0 {
                ExFreePoolWithTag(KeyInformation as PVOID, 0x206F4943);
                if !ResourceMapKey.is_null() {
                    ObCloseHandle(ResourceMapKey, 0i8);
                }
                if !ChildKey2.is_null() {
                    ObCloseHandle(ChildKey2, 0i8);
                }
                if !ChildKey3.is_null() {
                    ObCloseHandle(ChildKey3, 0i8);
                }
                if Status == 0x8000001Au32 as i32 {
                    Status = STATUS_SUCCESS;
                }
                return Status;
            }

            // Close handle from previous middle loop iteration to prevent leak
            if !ChildKey3.is_null() {
                ObCloseHandle(ChildKey3, 0i8);
                ChildKey3 = core::ptr::null_mut();
            }

            KeyName.Buffer = (*KeyInformation).Name.as_mut_ptr();
            KeyName.MaximumLength = (*KeyInformation).NameLength as u16;
            KeyName.Length = (*KeyInformation).NameLength as u16;
            InitializeObjectAttributes(
                &mut ObjectAttributes,
                &mut KeyName,
                0x00000040 | 0x00000200, // OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE
                ChildKey2,
                core::ptr::null_mut(),
            );
            Status = ZwOpenKey(&mut ChildKey3, 0x0001, &mut ObjectAttributes);
            ExFreePoolWithTag(KeyInformation as PVOID, 0x206F4943);
            if Status < 0 {
                if !ResourceMapKey.is_null() {
                    ObCloseHandle(ResourceMapKey, 0i8);
                }
                if !ChildKey2.is_null() {
                    ObCloseHandle(ChildKey2, 0i8);
                }
                if !ChildKey3.is_null() {
                    ObCloseHandle(ChildKey3, 0i8);
                }
                if Status == 0x8000001Au32 as i32 {
                    Status = STATUS_SUCCESS;
                }
                return Status;
            }

            ChildKeyIndex3 = 0;
            loop {
                Status = ZwEnumerateValueKey(
                    ChildKey3,
                    ChildKeyIndex3,
                    2,
                    core::ptr::null_mut(),
                    0,
                    &mut RequiredLength,
                );
                if Status == 0x8000001Au32 as i32 {
                    break;
                } else if Status == 0xC0000023u32 as i32 {
                    KeyValueInformationLength = RequiredLength;
                    KeyValueInformation = ExAllocatePoolWithTag(
                        PagedPool,
                        KeyValueInformationLength as usize,
                        0x206F4943,
                    ) as PKEY_VALUE_PARTIAL_INFORMATION;
                    if KeyValueInformation.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        if !ResourceMapKey.is_null() {
                            ObCloseHandle(ResourceMapKey, 0i8);
                        }
                        if !ChildKey2.is_null() {
                            ObCloseHandle(ChildKey2, 0i8);
                        }
                        if !ChildKey3.is_null() {
                            ObCloseHandle(ChildKey3, 0i8);
                        }
                        if Status == 0x8000001Au32 as i32 {
                            Status = STATUS_SUCCESS;
                        }
                        return Status;
                    }

                    Status = ZwEnumerateValueKey(
                        ChildKey3,
                        ChildKeyIndex3,
                        2,
                        KeyValueInformation as PVOID,
                        KeyValueInformationLength,
                        &mut RequiredLength,
                    );
                } else {
                    if !ResourceMapKey.is_null() {
                        ObCloseHandle(ResourceMapKey, 0i8);
                    }
                    if !ChildKey2.is_null() {
                        ObCloseHandle(ChildKey2, 0i8);
                    }
                    if !ChildKey3.is_null() {
                        ObCloseHandle(ChildKey3, 0i8);
                    }
                    if Status == 0x8000001Au32 as i32 {
                        Status = STATUS_SUCCESS;
                    }
                    return Status;
                }
                if Status < 0 {
                    ExFreePoolWithTag(KeyValueInformation as PVOID, 0x206F4943);
                    if !ResourceMapKey.is_null() {
                        ObCloseHandle(ResourceMapKey, 0i8);
                    }
                    if !ChildKey2.is_null() {
                        ObCloseHandle(ChildKey2, 0i8);
                    }
                    if !ChildKey3.is_null() {
                        ObCloseHandle(ChildKey3, 0i8);
                    }
                    if Status == 0x8000001Au32 as i32 {
                        Status = STATUS_SUCCESS;
                    }
                    return Status;
                }

                Status = ZwEnumerateValueKey(
                    ChildKey3,
                    ChildKeyIndex3,
                    0, // KeyValueBasicInformation
                    core::ptr::null_mut(),
                    0,
                    &mut RequiredLength,
                );
                if Status == 0xC0000023u32 as i32 {
                    KeyNameInformationLength = RequiredLength;
                    KeyNameInformation = ExAllocatePoolWithTag(
                        PagedPool,
                        KeyNameInformationLength as usize + core::mem::size_of::<u16>(),
                        0x206F4943,
                    ) as PKEY_VALUE_BASIC_INFORMATION;
                    if KeyNameInformation.is_null() {
                        Status = STATUS_INSUFFICIENT_RESOURCES;
                        if !ResourceMapKey.is_null() {
                            ObCloseHandle(ResourceMapKey, 0i8);
                        }
                        if !ChildKey2.is_null() {
                            ObCloseHandle(ChildKey2, 0i8);
                        }
                        if !ChildKey3.is_null() {
                            ObCloseHandle(ChildKey3, 0i8);
                        }
                        if Status == 0x8000001Au32 as i32 {
                            Status = STATUS_SUCCESS;
                        }
                        return Status;
                    }

                    Status = ZwEnumerateValueKey(
                        ChildKey3,
                        ChildKeyIndex3,
                        0, // KeyValueBasicInformation
                        KeyNameInformation as PVOID,
                        KeyNameInformationLength,
                        &mut RequiredLength,
                    );
                } else {
                    if !ResourceMapKey.is_null() {
                        ObCloseHandle(ResourceMapKey, 0i8);
                    }
                    if !ChildKey2.is_null() {
                        ObCloseHandle(ChildKey2, 0i8);
                    }
                    if !ChildKey3.is_null() {
                        ObCloseHandle(ChildKey3, 0i8);
                    }
                    if Status == 0x8000001Au32 as i32 {
                        Status = STATUS_SUCCESS;
                    }
                    return Status;
                }
                ChildKeyIndex3 += 1;
                if Status < 0 {
                    ExFreePoolWithTag(KeyNameInformation as PVOID, 0x206F4943);
                    if !ResourceMapKey.is_null() {
                        ObCloseHandle(ResourceMapKey, 0i8);
                    }
                    if !ChildKey2.is_null() {
                        ObCloseHandle(ChildKey2, 0i8);
                    }
                    if !ChildKey3.is_null() {
                        ObCloseHandle(ChildKey3, 0i8);
                    }
                    if Status == 0x8000001Au32 as i32 {
                        Status = STATUS_SUCCESS;
                    }
                    return Status;
                }

                *(*KeyNameInformation)
                    .Name
                    .as_mut_ptr()
                    .add(((*KeyNameInformation).NameLength / core::mem::size_of::<u16>() as u32) as usize) =
                    0;

                let name_slice = core::slice::from_raw_parts(
                    (*KeyNameInformation).Name.as_ptr(),
                    (*KeyNameInformation).NameLength as usize / core::mem::size_of::<u16>() + 1 ,
                );
                let mut found_translated = false;
                for window in name_slice.windows(11) {
                    if window
                        == [
                            b'.' as u16,
                            b'T' as u16,
                            b'r' as u16,
                            b'a' as u16,
                            b'n' as u16,
                            b's' as u16,
                            b'l' as u16,
                            b'a' as u16,
                            b't' as u16,
                            b'e' as u16,
                            b'd' as u16,
                        ]
                    {
                        found_translated = true;
                        break;
                    }
                }

                if found_translated {
                    ExFreePoolWithTag(KeyNameInformation as PVOID, 0x206F4943);
                    ExFreePoolWithTag(KeyValueInformation as PVOID, 0x206F4943);
                    continue;
                }

                ExFreePoolWithTag(KeyNameInformation as PVOID, 0x206F4943);

                if IopCheckForResourceConflict(
                    ResourceList,
                    (*KeyValueInformation).Data.as_ptr() as PCM_RESOURCE_LIST,
                    Silent,
                    ConflictingDescriptor,
                ) != 0
                {
                    ExFreePoolWithTag(KeyValueInformation as PVOID, 0x206F4943);
                    Status = STATUS_CONFLICTING_ADDRESSES;
                    if !ResourceMapKey.is_null() {
                        ObCloseHandle(ResourceMapKey, 0i8);
                    }
                    if !ChildKey2.is_null() {
                        ObCloseHandle(ChildKey2, 0i8);
                    }
                    if !ChildKey3.is_null() {
                        ObCloseHandle(ChildKey3, 0i8);
                    }
                    if Status == 0x8000001Au32 as i32 {
                        Status = STATUS_SUCCESS;
                    }
                    return Status;
                }

                ExFreePoolWithTag(KeyValueInformation as PVOID, 0x206F4943);
            }
        }
    }

    if !ResourceMapKey.is_null() {
        ObCloseHandle(ResourceMapKey, 0i8);
    }
    if !ChildKey2.is_null() {
        ObCloseHandle(ChildKey2, 0i8);
    }
    if !ChildKey3.is_null() {
        ObCloseHandle(ChildKey3, 0i8);
    }
    if Status == 0x8000001Au32 as i32 {
        Status = STATUS_SUCCESS;
    }

    crate::dbg_print!(b"PNPRES: DRC done=0x%x\n\0", Status);
    Status
}


// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS Kernel
//  * COPYRIGHT:       GPL - See COPYING in the top level directory
//  * FILE:            ntoskrnl/io/pnpmgr/pnpres.c
//  * PURPOSE:         Resource handling code
//  * PROGRAMMERS:     Cameron Gutman (cameron.gutman@reactos.org)
//  *                  ReactOS Portable Systems Group
//  */
//
// #include <ntoskrnl.h>
//
// #define NDEBUG
// #include <debug.h>
//
// FORCEINLINE
// PIO_RESOURCE_LIST
// IopGetNextResourceList(
//     _In_ const IO_RESOURCE_LIST *ResourceList)
// {
//     ASSERT((ResourceList->Count > 0) && (ResourceList->Count < 1000));
//     return (PIO_RESOURCE_LIST)(
//         &ResourceList->Descriptors[ResourceList->Count]);
// }
//
// static
// BOOLEAN
// IopCheckDescriptorForConflict(
//     PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDesc,
//     OPTIONAL PCM_PARTIAL_RESOURCE_DESCRIPTOR ConflictingDescriptor)
// {
//     CM_RESOURCE_LIST CmList;
//     NTSTATUS Status;
//
//     CmList.Count = 1;
//     CmList.List[0].InterfaceType = InterfaceTypeUndefined;
//     CmList.List[0].BusNumber = 0;
//     CmList.List[0].PartialResourceList.Version = 1;
//     CmList.List[0].PartialResourceList.Revision = 1;
//     CmList.List[0].PartialResourceList.Count = 1;
//     CmList.List[0].PartialResourceList.PartialDescriptors[0] = *CmDesc;
//
//     Status = IopDetectResourceConflict(&CmList, TRUE, ConflictingDescriptor);
//     if (Status == STATUS_CONFLICTING_ADDRESSES)
//         return TRUE;
//
//     return FALSE;
// }
//
// static
// BOOLEAN
// IopFindBusNumberResource(
//     IN PIO_RESOURCE_DESCRIPTOR IoDesc,
//     OUT PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDesc)
// {
//     ULONG Start;
//     CM_PARTIAL_RESOURCE_DESCRIPTOR ConflictingDesc;
//
//     ASSERT(IoDesc->Type == CmDesc->Type);
//     ASSERT(IoDesc->Type == CmResourceTypeBusNumber);
//
//     for (Start = IoDesc->u.BusNumber.MinBusNumber;
//          Start <= IoDesc->u.BusNumber.MaxBusNumber - IoDesc->u.BusNumber.Length + 1;
//          Start++)
//     {
//         CmDesc->u.BusNumber.Length = IoDesc->u.BusNumber.Length;
//         CmDesc->u.BusNumber.Start = Start;
//
//         if (IopCheckDescriptorForConflict(CmDesc, &ConflictingDesc))
//         {
//             Start += ConflictingDesc.u.BusNumber.Start + ConflictingDesc.u.BusNumber.Length;
//         }
//         else
//         {
//             DPRINT1("Satisfying bus number requirement with 0x%x (length: 0x%x)\n", Start, CmDesc->u.BusNumber.Length);
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// static
// BOOLEAN
// IopFindMemoryResource(
//     IN PIO_RESOURCE_DESCRIPTOR IoDesc,
//     OUT PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDesc)
// {
//     ULONGLONG Start;
//     CM_PARTIAL_RESOURCE_DESCRIPTOR ConflictingDesc;
//
//     ASSERT(IoDesc->Type == CmDesc->Type);
//     ASSERT(IoDesc->Type == CmResourceTypeMemory);
//
//     /* HACK */
//     if (IoDesc->u.Memory.Alignment == 0)
//         IoDesc->u.Memory.Alignment = 1;
//
//     for (Start = (ULONGLONG)IoDesc->u.Memory.MinimumAddress.QuadPart;
//          Start <= (ULONGLONG)IoDesc->u.Memory.MaximumAddress.QuadPart - IoDesc->u.Memory.Length + 1;
//          Start += IoDesc->u.Memory.Alignment)
//     {
//         CmDesc->u.Memory.Length = IoDesc->u.Memory.Length;
//         CmDesc->u.Memory.Start.QuadPart = (LONGLONG)Start;
//
//         if (IopCheckDescriptorForConflict(CmDesc, &ConflictingDesc))
//         {
//             Start += (ULONGLONG)ConflictingDesc.u.Memory.Start.QuadPart +
//                      ConflictingDesc.u.Memory.Length;
//         }
//         else
//         {
//             DPRINT1("Satisfying memory requirement with 0x%I64x (length: 0x%x)\n", Start, CmDesc->u.Memory.Length);
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// static
// BOOLEAN
// IopFindPortResource(
//     IN PIO_RESOURCE_DESCRIPTOR IoDesc,
//     OUT PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDesc)
// {
//     ULONGLONG Start;
//     CM_PARTIAL_RESOURCE_DESCRIPTOR ConflictingDesc;
//
//     ASSERT(IoDesc->Type == CmDesc->Type);
//     ASSERT(IoDesc->Type == CmResourceTypePort);
//
//     /* HACK */
//     if (IoDesc->u.Port.Alignment == 0)
//         IoDesc->u.Port.Alignment = 1;
//
//     for (Start = (ULONGLONG)IoDesc->u.Port.MinimumAddress.QuadPart;
//          Start <= (ULONGLONG)IoDesc->u.Port.MaximumAddress.QuadPart - IoDesc->u.Port.Length + 1;
//          Start += IoDesc->u.Port.Alignment)
//     {
//         CmDesc->u.Port.Length = IoDesc->u.Port.Length;
//         CmDesc->u.Port.Start.QuadPart = (LONGLONG)Start;
//
//         if (IopCheckDescriptorForConflict(CmDesc, &ConflictingDesc))
//         {
//             Start += (ULONGLONG)ConflictingDesc.u.Port.Start.QuadPart + ConflictingDesc.u.Port.Length;
//         }
//         else
//         {
//             DPRINT("Satisfying port requirement with 0x%I64x (length: 0x%x)\n", Start, CmDesc->u.Port.Length);
//             return TRUE;
//         }
//     }
//
//     DPRINT1("IopFindPortResource failed!\n");
//     return FALSE;
// }
//
// static
// BOOLEAN
// IopFindDmaResource(
//     IN PIO_RESOURCE_DESCRIPTOR IoDesc,
//     OUT PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDesc)
// {
//     ULONG Channel;
//
//     ASSERT(IoDesc->Type == CmDesc->Type);
//     ASSERT(IoDesc->Type == CmResourceTypeDma);
//
//     for (Channel = IoDesc->u.Dma.MinimumChannel;
//          Channel <= IoDesc->u.Dma.MaximumChannel;
//          Channel++)
//     {
//         CmDesc->u.Dma.Channel = Channel;
//         CmDesc->u.Dma.Port = 0;
//
//         if (!IopCheckDescriptorForConflict(CmDesc, NULL))
//         {
//             DPRINT1("Satisfying DMA requirement with channel 0x%x\n", Channel);
//             return TRUE;
//         }
//     }
//
//     return FALSE;
// }
//
// static
// BOOLEAN
// IopFindInterruptResource(
//     IN PIO_RESOURCE_DESCRIPTOR IoDesc,
//     OUT PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDesc)
// {
//     ULONG Vector;
//
//     ASSERT(IoDesc->Type == CmDesc->Type);
//     ASSERT(IoDesc->Type == CmResourceTypeInterrupt);
//
//     for (Vector = IoDesc->u.Interrupt.MinimumVector;
//          Vector <= IoDesc->u.Interrupt.MaximumVector;
//          Vector++)
//     {
//         CmDesc->u.Interrupt.Vector = Vector;
//         CmDesc->u.Interrupt.Level = Vector;
//         CmDesc->u.Interrupt.Affinity = (KAFFINITY)-1;
//
//         if (!IopCheckDescriptorForConflict(CmDesc, NULL))
//         {
//             DPRINT1("Satisfying interrupt requirement with IRQ 0x%x\n", Vector);
//             return TRUE;
//         }
//     }
//
//     DPRINT1("Failed to satisfy interrupt requirement with IRQ 0x%x-0x%x\n",
//             IoDesc->u.Interrupt.MinimumVector,
//             IoDesc->u.Interrupt.MaximumVector);
//     return FALSE;
// }
//
// NTSTATUS NTAPI
// IopFixupResourceListWithRequirements(
//     IN PIO_RESOURCE_REQUIREMENTS_LIST RequirementsList,
//     OUT PCM_RESOURCE_LIST *ResourceList)
// {
//     ULONG i, OldCount;
//     BOOLEAN AlternateRequired = FALSE;
//     PIO_RESOURCE_LIST ResList;
//
//     /* Save the initial resource count when we got here so we can restore if an alternate fails */
//     if (*ResourceList != NULL)
//         OldCount = (*ResourceList)->List[0].PartialResourceList.Count;
//     else
//         OldCount = 0;
//
//     ResList = &RequirementsList->List[0];
//     for (i = 0; i < RequirementsList->AlternativeLists; i++, ResList = IopGetNextResourceList(ResList))
//     {
//         ULONG ii;
//
//         /* We need to get back to where we were before processing the last alternative list */
//         if (OldCount == 0 && *ResourceList != NULL)
//         {
//             /* Just free it and kill the pointer */
//             ExFreePool(*ResourceList);
//             *ResourceList = NULL;
//         }
//         else if (OldCount != 0)
//         {
//             PCM_RESOURCE_LIST NewList;
//
//             /* Let's resize it */
//             (*ResourceList)->List[0].PartialResourceList.Count = OldCount;
//
//             /* Allocate the new smaller list */
//             NewList = ExAllocatePool(PagedPool, PnpDetermineResourceListSize(*ResourceList));
//             if (!NewList)
//                 return STATUS_NO_MEMORY;
//
//             /* Copy the old stuff back */
//             RtlCopyMemory(NewList, *ResourceList, PnpDetermineResourceListSize(*ResourceList));
//
//             /* Free the old one */
//             ExFreePool(*ResourceList);
//
//             /* Store the pointer to the new one */
//             *ResourceList = NewList;
//         }
//
//         for (ii = 0; ii < ResList->Count; ii++)
//         {
//             ULONG iii;
//             PCM_PARTIAL_RESOURCE_LIST PartialList = (*ResourceList) ? &(*ResourceList)->List[0].PartialResourceList : NULL;
//             PIO_RESOURCE_DESCRIPTOR IoDesc = &ResList->Descriptors[ii];
//             BOOLEAN Matched = FALSE;
//
//             /* Skip alternates if we don't need one */
//             if (!AlternateRequired && (IoDesc->Option & IO_RESOURCE_ALTERNATIVE))
//             {
//                 DPRINT("Skipping unneeded alternate\n");
//                 continue;
//             }
//
//             /* Check if we couldn't satsify a requirement or its alternates */
//             if (AlternateRequired && !(IoDesc->Option & IO_RESOURCE_ALTERNATIVE))
//             {
//                 DPRINT1("Unable to satisfy preferred resource or alternates in list %lu\n", i);
//
//                 /* Break out of this loop and try the next list */
//                 break;
//             }
//
//             for (iii = 0; PartialList && iii < PartialList->Count && !Matched; iii++)
//             {
//                 /* Partial resource descriptors can be of variable size (CmResourceTypeDeviceSpecific),
//                    but only one is allowed and it must be the last one in the list! */
//                 PCM_PARTIAL_RESOURCE_DESCRIPTOR CmDesc = &PartialList->PartialDescriptors[iii];
//
//                 /* First check types */
//                 if (IoDesc->Type != CmDesc->Type)
//                     continue;
//
//                 switch (IoDesc->Type)
//                 {
//                     case CmResourceTypeInterrupt:
//                         /* Make sure it satisfies our vector range */
//                         if (CmDesc->u.Interrupt.Vector >= IoDesc->u.Interrupt.MinimumVector &&
//                             CmDesc->u.Interrupt.Vector <= IoDesc->u.Interrupt.MaximumVector)
//                         {
//                             /* Found it */
//                             Matched = TRUE;
//                         }
//                         else
//                         {
//                             DPRINT("Interrupt - Not a match! 0x%x not inside 0x%x to 0x%x\n",
//                                    CmDesc->u.Interrupt.Vector,
//                                    IoDesc->u.Interrupt.MinimumVector,
//                                    IoDesc->u.Interrupt.MaximumVector);
//                         }
//                         break;
//
//                     case CmResourceTypeMemory:
//                     case CmResourceTypePort:
//                         /* Make sure the length matches and it satisfies our address range */
//                         if (CmDesc->u.Memory.Length == IoDesc->u.Memory.Length &&
//                             (ULONGLONG)CmDesc->u.Memory.Start.QuadPart >= (ULONGLONG)IoDesc->u.Memory.MinimumAddress.QuadPart &&
//                             (ULONGLONG)CmDesc->u.Memory.Start.QuadPart + CmDesc->u.Memory.Length - 1 <= (ULONGLONG)IoDesc->u.Memory.MaximumAddress.QuadPart)
//                         {
//                             /* Found it */
//                             Matched = TRUE;
//                         }
//                         else
//                         {
//                             DPRINT("Memory/Port - Not a match! 0x%I64x with length 0x%x not inside 0x%I64x to 0x%I64x with length 0x%x\n",
//                                    CmDesc->u.Memory.Start.QuadPart,
//                                    CmDesc->u.Memory.Length,
//                                    IoDesc->u.Memory.MinimumAddress.QuadPart,
//                                    IoDesc->u.Memory.MaximumAddress.QuadPart,
//                                    IoDesc->u.Memory.Length);
//                         }
//                         break;
//
//                     case CmResourceTypeBusNumber:
//                         /* Make sure the length matches and it satisfies our bus number range */
//                         if (CmDesc->u.BusNumber.Length == IoDesc->u.BusNumber.Length &&
//                             CmDesc->u.BusNumber.Start >= IoDesc->u.BusNumber.MinBusNumber &&
//                             CmDesc->u.BusNumber.Start + CmDesc->u.BusNumber.Length - 1 <= IoDesc->u.BusNumber.MaxBusNumber)
//                         {
//                             /* Found it */
//                             Matched = TRUE;
//                         }
//                         else
//                         {
//                             DPRINT("Bus Number - Not a match! 0x%x with length 0x%x not inside 0x%x to 0x%x with length 0x%x\n",
//                                    CmDesc->u.BusNumber.Start,
//                                    CmDesc->u.BusNumber.Length,
//                                    IoDesc->u.BusNumber.MinBusNumber,
//                                    IoDesc->u.BusNumber.MaxBusNumber,
//                                    IoDesc->u.BusNumber.Length);
//                         }
//                         break;
//
//                     case CmResourceTypeDma:
//                         /* Make sure it fits in our channel range */
//                         if (CmDesc->u.Dma.Channel >= IoDesc->u.Dma.MinimumChannel &&
//                             CmDesc->u.Dma.Channel <= IoDesc->u.Dma.MaximumChannel)
//                         {
//                             /* Found it */
//                             Matched = TRUE;
//                         }
//                         else
//                         {
//                             DPRINT("DMA - Not a match! 0x%x not inside 0x%x to 0x%x\n",
//                                    CmDesc->u.Dma.Channel,
//                                    IoDesc->u.Dma.MinimumChannel,
//                                    IoDesc->u.Dma.MaximumChannel);
//                         }
//                         break;
//
//                     default:
//                         /* Other stuff is fine */
//                         Matched = TRUE;
//                         break;
//                 }
//             }
//
//             /* Check if we found a matching descriptor */
//             if (!Matched)
//             {
//                 PCM_RESOURCE_LIST NewList;
//                 CM_PARTIAL_RESOURCE_DESCRIPTOR NewDesc;
//                 PCM_PARTIAL_RESOURCE_DESCRIPTOR DescPtr;
//                 BOOLEAN FoundResource = TRUE;
//
//                 /* Setup the new CM descriptor */
//                 NewDesc.Type = IoDesc->Type;
//                 NewDesc.Flags = IoDesc->Flags;
//                 NewDesc.ShareDisposition = IoDesc->ShareDisposition;
//
//                 /* Let'se see if we can find a resource to satisfy this */
//                 switch (IoDesc->Type)
//                 {
//                     case CmResourceTypeInterrupt:
//                         /* Find an available interrupt */
//                         if (!IopFindInterruptResource(IoDesc, &NewDesc))
//                         {
//                             DPRINT1("Failed to find an available interrupt resource (0x%x to 0x%x)\n",
//                                     IoDesc->u.Interrupt.MinimumVector, IoDesc->u.Interrupt.MaximumVector);
//
//                             FoundResource = FALSE;
//                         }
//                         break;
//
//                     case CmResourceTypePort:
//                         /* Find an available port range */
//                         if (!IopFindPortResource(IoDesc, &NewDesc))
//                         {
//                             DPRINT1("Failed to find an available port resource (0x%I64x to 0x%I64x length: 0x%x)\n",
//                                     IoDesc->u.Port.MinimumAddress.QuadPart, IoDesc->u.Port.MaximumAddress.QuadPart,
//                                     IoDesc->u.Port.Length);
//
//                             FoundResource = FALSE;
//                         }
//                         break;
//
//                     case CmResourceTypeMemory:
//                         /* Find an available memory range */
//                         if (!IopFindMemoryResource(IoDesc, &NewDesc))
//                         {
//                             DPRINT1("Failed to find an available memory resource (0x%I64x to 0x%I64x length: 0x%x)\n",
//                                     IoDesc->u.Memory.MinimumAddress.QuadPart, IoDesc->u.Memory.MaximumAddress.QuadPart,
//                                     IoDesc->u.Memory.Length);
//
//                             FoundResource = FALSE;
//                         }
//                         break;
//
//                     case CmResourceTypeBusNumber:
//                         /* Find an available bus address range */
//                         if (!IopFindBusNumberResource(IoDesc, &NewDesc))
//                         {
//                             DPRINT1("Failed to find an available bus number resource (0x%x to 0x%x length: 0x%x)\n",
//                                     IoDesc->u.BusNumber.MinBusNumber, IoDesc->u.BusNumber.MaxBusNumber,
//                                     IoDesc->u.BusNumber.Length);
//
//                             FoundResource = FALSE;
//                         }
//                         break;
//
//                     case CmResourceTypeDma:
//                         /* Find an available DMA channel */
//                         if (!IopFindDmaResource(IoDesc, &NewDesc))
//                         {
//                             DPRINT1("Failed to find an available dma resource (0x%x to 0x%x)\n",
//                                     IoDesc->u.Dma.MinimumChannel, IoDesc->u.Dma.MaximumChannel);
//
//                             FoundResource = FALSE;
//                         }
//                         break;
//
//                     default:
//                         DPRINT1("Unsupported resource type: %x\n", IoDesc->Type);
//                         FoundResource = FALSE;
//                         break;
//                 }
//
//                 /* Check if it's missing and required */
//                 if (!FoundResource && IoDesc->Option == 0)
//                 {
//                     /* Break out of this loop and try the next list */
//                     DPRINT1("Unable to satisfy required resource in list %lu\n", i);
//                     break;
//                 }
//                 else if (!FoundResource)
//                 {
//                     /* Try an alternate for this preferred descriptor */
//                     AlternateRequired = TRUE;
//                     continue;
//                 }
//                 else
//                 {
//                     /* Move on to the next preferred or required descriptor after this one */
//                     AlternateRequired = FALSE;
//                 }
//
//                 /* Figure out what we need */
//                 if (PartialList == NULL)
//                 {
//                     /* We need a new list */
//                     NewList = ExAllocatePool(PagedPool, sizeof(CM_RESOURCE_LIST));
//                     if (!NewList)
//                         return STATUS_NO_MEMORY;
//
//                     /* Set it up */
//                     NewList->Count = 1;
//                     NewList->List[0].InterfaceType = RequirementsList->InterfaceType;
//                     NewList->List[0].BusNumber = RequirementsList->BusNumber;
//                     NewList->List[0].PartialResourceList.Version = 1;
//                     NewList->List[0].PartialResourceList.Revision = 1;
//                     NewList->List[0].PartialResourceList.Count = 1;
//
//                     /* Set our pointer */
//                     DescPtr = &NewList->List[0].PartialResourceList.PartialDescriptors[0];
//                 }
//                 else
//                 {
//                     /* Allocate the new larger list */
//                     NewList = ExAllocatePool(PagedPool, PnpDetermineResourceListSize(*ResourceList) + sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR));
//                     if (!NewList)
//                         return STATUS_NO_MEMORY;
//
//                     /* Copy the old stuff back */
//                     RtlCopyMemory(NewList, *ResourceList, PnpDetermineResourceListSize(*ResourceList));
//
//                     /* Set our pointer */
//                     DescPtr = &NewList->List[0].PartialResourceList.PartialDescriptors[NewList->List[0].PartialResourceList.Count];
//
//                     /* Increment the descriptor count */
//                     NewList->List[0].PartialResourceList.Count++;
//
//                     /* Free the old list */
//                     ExFreePool(*ResourceList);
//                 }
//
//                 /* Copy the descriptor in */
//                 *DescPtr = NewDesc;
//
//                 /* Store the new list */
//                 *ResourceList = NewList;
//             }
//         }
//
//         /* Check if we need an alternate with no resources left */
//         if (AlternateRequired)
//         {
//             DPRINT1("Unable to satisfy preferred resource or alternates in list %lu\n", i);
//
//             /* Try the next alternate list */
//             continue;
//         }
//
//         /* We're done because we satisfied one of the alternate lists */
//         return STATUS_SUCCESS;
//     }
//
//     /* We ran out of alternates */
//     DPRINT1("Out of alternate lists!\n");
//
//     /* Free the list */
//     if (*ResourceList)
//     {
//         ExFreePool(*ResourceList);
//         *ResourceList = NULL;
//     }
//
//     /* Fail */
//     return STATUS_CONFLICTING_ADDRESSES;
// }
//
// static
// BOOLEAN
// IopCheckResourceDescriptor(
//     IN PCM_PARTIAL_RESOURCE_DESCRIPTOR ResDesc,
//     IN PCM_RESOURCE_LIST ResourceList,
//     IN BOOLEAN Silent,
//     OUT OPTIONAL PCM_PARTIAL_RESOURCE_DESCRIPTOR ConflictingDescriptor)
// {
//     ULONG i, ii;
//     BOOLEAN Result = FALSE;
//     PCM_FULL_RESOURCE_DESCRIPTOR FullDescriptor;
//
//     FullDescriptor = &ResourceList->List[0];
//     for (i = 0; i < ResourceList->Count; i++)
//     {
//         PCM_PARTIAL_RESOURCE_LIST ResList = &FullDescriptor->PartialResourceList;
//         FullDescriptor = CmiGetNextResourceDescriptor(FullDescriptor);
//
//         for (ii = 0; ii < ResList->Count; ii++)
//         {
//             /* Partial resource descriptors can be of variable size (CmResourceTypeDeviceSpecific),
//                but only one is allowed and it must be the last one in the list! */
//             PCM_PARTIAL_RESOURCE_DESCRIPTOR ResDesc2 = &ResList->PartialDescriptors[ii];
//
//             /* We don't care about shared resources */
//             if (ResDesc->ShareDisposition == CmResourceShareShared &&
//                 ResDesc2->ShareDisposition == CmResourceShareShared)
//                 continue;
//
//             /* Make sure we're comparing the same types */
//             if (ResDesc->Type != ResDesc2->Type)
//                 continue;
//
//             switch (ResDesc->Type)
//             {
//                 case CmResourceTypeMemory:
//                 {
//                     /* NOTE: ranges are in a form [x1;x2) */
//                     UINT64 rStart = (UINT64)ResDesc->u.Memory.Start.QuadPart;
//                     UINT64 rEnd = (UINT64)ResDesc->u.Memory.Start.QuadPart
//                                   + ResDesc->u.Memory.Length;
//                     UINT64 r2Start = (UINT64)ResDesc2->u.Memory.Start.QuadPart;
//                     UINT64 r2End = (UINT64)ResDesc2->u.Memory.Start.QuadPart
//                                    + ResDesc2->u.Memory.Length;
//
//                     if (rStart < r2End && r2Start < rEnd)
//                     {
//                         if (!Silent)
//                         {
//                             DPRINT1("Resource conflict: Memory (0x%I64x to 0x%I64x vs. 0x%I64x to 0x%I64x)\n",
//                                     rStart, rEnd, r2Start, r2End);
//                         }
//
//                         Result = TRUE;
//
//                         goto ByeBye;
//                     }
//                     break;
//                 }
//                 case CmResourceTypePort:
//                 {
//                     /* NOTE: ranges are in a form [x1;x2) */
//                     UINT64 rStart = (UINT64)ResDesc->u.Port.Start.QuadPart;
//                     UINT64 rEnd = (UINT64)ResDesc->u.Port.Start.QuadPart
//                                   + ResDesc->u.Port.Length;
//                     UINT64 r2Start = (UINT64)ResDesc2->u.Port.Start.QuadPart;
//                     UINT64 r2End = (UINT64)ResDesc2->u.Port.Start.QuadPart
//                                    + ResDesc2->u.Port.Length;
//
//                     if (rStart < r2End && r2Start < rEnd)
//                     {
//                         if (!Silent)
//                         {
//                             DPRINT1("Resource conflict: Port (0x%I64x to 0x%I64x vs. 0x%I64x to 0x%I64x)\n",
//                                     rStart, rEnd, r2Start, r2End);
//                         }
//
//                         Result = TRUE;
//
//                         goto ByeBye;
//                     }
//                     break;
//                 }
//                 case CmResourceTypeInterrupt:
//                 {
//                     if (ResDesc->u.Interrupt.Vector == ResDesc2->u.Interrupt.Vector)
//                     {
//                         if (!Silent)
//                         {
//                             DPRINT1("Resource conflict: IRQ (0x%x 0x%x vs. 0x%x 0x%x)\n",
//                                     ResDesc->u.Interrupt.Vector, ResDesc->u.Interrupt.Level,
//                                     ResDesc2->u.Interrupt.Vector, ResDesc2->u.Interrupt.Level);
//                         }
//
//                         Result = TRUE;
//
//                         goto ByeBye;
//                     }
//                     break;
//                 }
//                 case CmResourceTypeBusNumber:
//                 {
//                     /* NOTE: ranges are in a form [x1;x2) */
//                     UINT32 rStart = ResDesc->u.BusNumber.Start;
//                     UINT32 rEnd = ResDesc->u.BusNumber.Start + ResDesc->u.BusNumber.Length;
//                     UINT32 r2Start = ResDesc2->u.BusNumber.Start;
//                     UINT32 r2End = ResDesc2->u.BusNumber.Start + ResDesc2->u.BusNumber.Length;
//
//                     if (rStart < r2End && r2Start < rEnd)
//                     {
//                         if (!Silent)
//                         {
//                             DPRINT1("Resource conflict: Bus number (0x%x to 0x%x vs. 0x%x to 0x%x)\n",
//                                     rStart, rEnd, r2Start, r2End);
//                         }
//
//                         Result = TRUE;
//
//                         goto ByeBye;
//                     }
//                     break;
//                 }
//                 case CmResourceTypeDma:
//                 {
//                     if (ResDesc->u.Dma.Channel == ResDesc2->u.Dma.Channel)
//                     {
//                         if (!Silent)
//                         {
//                             DPRINT1("Resource conflict: Dma (0x%x 0x%x vs. 0x%x 0x%x)\n",
//                                     ResDesc->u.Dma.Channel, ResDesc->u.Dma.Port,
//                                     ResDesc2->u.Dma.Channel, ResDesc2->u.Dma.Port);
//                         }
//
//                         Result = TRUE;
//
//                         goto ByeBye;
//                     }
//                     break;
//                 }
//             }
//         }
//     }
//
// ByeBye:
//
//     if (Result && ConflictingDescriptor)
//     {
//         RtlCopyMemory(ConflictingDescriptor,
//                       ResDesc,
//                       sizeof(CM_PARTIAL_RESOURCE_DESCRIPTOR));
//     }
//
//     // Hacked, because after fixing resource list parsing
//     // we actually detect resource conflicts
//     return Silent ? Result : FALSE; // Result;
// }
//
// static
// NTSTATUS
// IopUpdateControlKeyWithResources(
//     IN PDEVICE_NODE DeviceNode)
// {
//     UNICODE_STRING EnumRoot = RTL_CONSTANT_STRING(ENUM_ROOT);
//     UNICODE_STRING Control = RTL_CONSTANT_STRING(L"Control");
//     UNICODE_STRING ValueName = RTL_CONSTANT_STRING(L"AllocConfig");
//     HANDLE EnumKey, InstanceKey, ControlKey;
//     NTSTATUS Status;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     /* Open the Enum key */
//     Status = IopOpenRegistryKeyEx(&EnumKey, NULL, &EnumRoot, KEY_ENUMERATE_SUB_KEYS);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Open the instance key (eg. Root\PNP0A03) */
//     Status = IopOpenRegistryKeyEx(&InstanceKey, EnumKey, &DeviceNode->InstancePath, KEY_ENUMERATE_SUB_KEYS);
//     ZwClose(EnumKey);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Create/Open the Control key */
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &Control,
//                                OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                InstanceKey,
//                                NULL);
//     Status = ZwCreateKey(&ControlKey,
//                          KEY_SET_VALUE,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          NULL);
//     ZwClose(InstanceKey);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     /* Write the resource list */
//     Status = ZwSetValueKey(ControlKey,
//                            &ValueName,
//                            0,
//                            REG_RESOURCE_LIST,
//                            DeviceNode->ResourceList,
//                            PnpDetermineResourceListSize(DeviceNode->ResourceList));
//     ZwClose(ControlKey);
//
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     return STATUS_SUCCESS;
// }
//
// static
// NTSTATUS
// IopFilterResourceRequirements(
//     IN PDEVICE_NODE DeviceNode)
// {
//     IO_STACK_LOCATION Stack;
//     IO_STATUS_BLOCK IoStatusBlock;
//     NTSTATUS Status;
//
//     DPRINT("Sending IRP_MN_FILTER_RESOURCE_REQUIREMENTS to device stack\n");
//
//     Stack.Parameters.FilterResourceRequirements.IoResourceRequirementList = DeviceNode->ResourceRequirements;
//     Status = IopInitiatePnpIrp(DeviceNode->PhysicalDeviceObject,
//                                &IoStatusBlock,
//                                IRP_MN_FILTER_RESOURCE_REQUIREMENTS,
//                                &Stack);
//     if (!NT_SUCCESS(Status) && Status != STATUS_NOT_SUPPORTED)
//     {
//         DPRINT1("IopInitiatePnpIrp(IRP_MN_FILTER_RESOURCE_REQUIREMENTS) failed\n");
//         return Status;
//     }
//     else if (NT_SUCCESS(Status) && IoStatusBlock.Information)
//     {
//         DeviceNode->ResourceRequirements = (PIO_RESOURCE_REQUIREMENTS_LIST)IoStatusBlock.Information;
//     }
//
//     return STATUS_SUCCESS;
// }
//
//
// NTSTATUS
// IopUpdateResourceMap(
//     IN PDEVICE_NODE DeviceNode,
//     PWCHAR Level1Key,
//     PWCHAR Level2Key)
// {
//     NTSTATUS Status;
//     ULONG Disposition;
//     HANDLE PnpMgrLevel1, PnpMgrLevel2, ResourceMapKey;
//     UNICODE_STRING KeyName;
//     OBJECT_ATTRIBUTES ObjectAttributes;
//
//     RtlInitUnicodeString(&KeyName,
//                          L"\\Registry\\Machine\\HARDWARE\\RESOURCEMAP");
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE,
//                                NULL,
//                                NULL);
//     Status = ZwCreateKey(&ResourceMapKey,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          &Disposition);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RtlInitUnicodeString(&KeyName, Level1Key);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE,
//                                ResourceMapKey,
//                                NULL);
//     Status = ZwCreateKey(&PnpMgrLevel1,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          &Disposition);
//     ZwClose(ResourceMapKey);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     RtlInitUnicodeString(&KeyName, Level2Key);
//     InitializeObjectAttributes(&ObjectAttributes,
//                                &KeyName,
//                                OBJ_CASE_INSENSITIVE | OBJ_OPENIF | OBJ_KERNEL_HANDLE,
//                                PnpMgrLevel1,
//                                NULL);
//     Status = ZwCreateKey(&PnpMgrLevel2,
//                          KEY_ALL_ACCESS,
//                          &ObjectAttributes,
//                          0,
//                          NULL,
//                          REG_OPTION_VOLATILE,
//                          &Disposition);
//     ZwClose(PnpMgrLevel1);
//     if (!NT_SUCCESS(Status))
//         return Status;
//
//     if (DeviceNode->ResourceList)
//     {
//         UNICODE_STRING NameU;
//         UNICODE_STRING RawSuffix, TranslatedSuffix;
//         ULONG OldLength = 0;
//
//         ASSERT(DeviceNode->ResourceListTranslated);
//
//         RtlInitUnicodeString(&TranslatedSuffix, L".Translated");
//         RtlInitUnicodeString(&RawSuffix, L".Raw");
//
//         Status = IoGetDeviceProperty(DeviceNode->PhysicalDeviceObject,
//                                      DevicePropertyPhysicalDeviceObjectName,
//                                      0,
//                                      NULL,
//                                      &OldLength);
//         if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//         {
//             ASSERT(OldLength);
//
//             NameU.Buffer = ExAllocatePool(PagedPool, OldLength + TranslatedSuffix.Length);
//             if (!NameU.Buffer)
//             {
//                 ZwClose(PnpMgrLevel2);
//                 return STATUS_INSUFFICIENT_RESOURCES;
//             }
//
//             NameU.Length = 0;
//             NameU.MaximumLength = (USHORT)OldLength + TranslatedSuffix.Length;
//
//             Status = IoGetDeviceProperty(DeviceNode->PhysicalDeviceObject,
//                                          DevicePropertyPhysicalDeviceObjectName,
//                                          NameU.MaximumLength,
//                                          NameU.Buffer,
//                                          &OldLength);
//             if (!NT_SUCCESS(Status))
//             {
//                 ZwClose(PnpMgrLevel2);
//                 ExFreePool(NameU.Buffer);
//                 return Status;
//             }
//         }
//         else if (!NT_SUCCESS(Status))
//         {
//             /* Some failure */
//             ZwClose(PnpMgrLevel2);
//             return Status;
//         }
//         else
//         {
//             /* This should never happen */
//             ASSERT(FALSE);
//         }
//
//         NameU.Length = (USHORT)OldLength - sizeof(UNICODE_NULL); /* Remove final NULL */
//
//         RtlAppendUnicodeStringToString(&NameU, &RawSuffix);
//
//         Status = ZwSetValueKey(PnpMgrLevel2,
//                                &NameU,
//                                0,
//                                REG_RESOURCE_LIST,
//                                DeviceNode->ResourceList,
//                                PnpDetermineResourceListSize(DeviceNode->ResourceList));
//         if (!NT_SUCCESS(Status))
//         {
//             ZwClose(PnpMgrLevel2);
//             ExFreePool(NameU.Buffer);
//             return Status;
//         }
//
//         /* "Remove" the suffix by setting the length back to what it used to be */
//         NameU.Length = (USHORT)OldLength - sizeof(UNICODE_NULL); /* Remove final NULL */
//
//         RtlAppendUnicodeStringToString(&NameU, &TranslatedSuffix);
//
//         Status = ZwSetValueKey(PnpMgrLevel2,
//                                &NameU,
//                                0,
//                                REG_RESOURCE_LIST,
//                                DeviceNode->ResourceListTranslated,
//                                PnpDetermineResourceListSize(DeviceNode->ResourceListTranslated));
//         ZwClose(PnpMgrLevel2);
//         ExFreePool(NameU.Buffer);
//
//         if (!NT_SUCCESS(Status))
//             return Status;
//     }
//     else
//     {
//         ZwClose(PnpMgrLevel2);
//     }
//
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// IopUpdateResourceMapForPnPDevice(
//     IN PDEVICE_NODE DeviceNode)
// {
//     return IopUpdateResourceMap(DeviceNode, L"PnP Manager", L"PnpManager");
// }
//
// static
// NTSTATUS
// IopTranslateDeviceResources(
//    IN PDEVICE_NODE DeviceNode)
// {
//    PCM_PARTIAL_RESOURCE_LIST pPartialResourceList;
//    PCM_PARTIAL_RESOURCE_DESCRIPTOR DescriptorRaw, DescriptorTranslated;
//    PCM_FULL_RESOURCE_DESCRIPTOR FullDescriptor;
//    ULONG i, j, ListSize;
//    NTSTATUS Status;
//
//    if (!DeviceNode->ResourceList)
//    {
//       DeviceNode->ResourceListTranslated = NULL;
//       return STATUS_SUCCESS;
//    }
//
//    /* That's easy to translate a resource list. Just copy the
//     * untranslated one and change few fields in the copy
//     */
//    ListSize = PnpDetermineResourceListSize(DeviceNode->ResourceList);
//
//    DeviceNode->ResourceListTranslated = ExAllocatePool(PagedPool, ListSize);
//    if (!DeviceNode->ResourceListTranslated)
//    {
//       Status = STATUS_NO_MEMORY;
//       goto cleanup;
//    }
//    RtlCopyMemory(DeviceNode->ResourceListTranslated, DeviceNode->ResourceList, ListSize);
//
//    FullDescriptor = &DeviceNode->ResourceList->List[0];
//    for (i = 0; i < DeviceNode->ResourceList->Count; i++)
//    {
//       pPartialResourceList = &FullDescriptor->PartialResourceList;
//       FullDescriptor = CmiGetNextResourceDescriptor(FullDescriptor);
//
//       for (j = 0; j < pPartialResourceList->Count; j++)
//       {
//         /* Partial resource descriptors can be of variable size (CmResourceTypeDeviceSpecific),
//            but only one is allowed and it must be the last one in the list! */
//          DescriptorRaw = &pPartialResourceList->PartialDescriptors[j];
//
//          /* Calculate the location of the translated resource descriptor */
//          DescriptorTranslated = (PCM_PARTIAL_RESOURCE_DESCRIPTOR)(
//              (PUCHAR)DeviceNode->ResourceListTranslated +
//              ((PUCHAR)DescriptorRaw - (PUCHAR)DeviceNode->ResourceList));
//
//          switch (DescriptorRaw->Type)
//          {
//             case CmResourceTypePort:
//             {
//                ULONG AddressSpace = 1; /* IO space */
//                if (!HalTranslateBusAddress(
//                   DeviceNode->ResourceList->List[i].InterfaceType,
//                   DeviceNode->ResourceList->List[i].BusNumber,
//                   DescriptorRaw->u.Port.Start,
//                   &AddressSpace,
//                   &DescriptorTranslated->u.Port.Start))
//                {
//                   Status = STATUS_UNSUCCESSFUL;
//                   DPRINT1("Failed to translate port resource (Start: 0x%I64x)\n", DescriptorRaw->u.Port.Start.QuadPart);
//                   goto cleanup;
//                }
//
//                if (AddressSpace == 0)
//                {
//                    DPRINT1("Guessed incorrect address space: 1 -> 0\n");
//
//                    /* FIXME: I think all other CM_RESOURCE_PORT_XXX flags are
//                     * invalid for this state but I'm not 100% sure */
//                    DescriptorRaw->Flags =
//                    DescriptorTranslated->Flags = CM_RESOURCE_PORT_MEMORY;
//                }
//                break;
//             }
//             case CmResourceTypeInterrupt:
//             {
//                KIRQL Irql;
//                DescriptorTranslated->u.Interrupt.Vector = HalGetInterruptVector(
//                   DeviceNode->ResourceList->List[i].InterfaceType,
//                   DeviceNode->ResourceList->List[i].BusNumber,
//                   DescriptorRaw->u.Interrupt.Level,
//                   DescriptorRaw->u.Interrupt.Vector,
//                   &Irql,
//                   &DescriptorTranslated->u.Interrupt.Affinity);
//                DescriptorTranslated->u.Interrupt.Level = Irql;
//                if (!DescriptorTranslated->u.Interrupt.Vector)
//                {
//                    Status = STATUS_UNSUCCESSFUL;
//                    DPRINT1("Failed to translate interrupt resource (Vector: 0x%x | Level: 0x%x)\n", DescriptorRaw->u.Interrupt.Vector,
//                                                                                                    DescriptorRaw->u.Interrupt.Level);
//                    goto cleanup;
//                }
//                break;
//             }
//             case CmResourceTypeMemory:
//             {
//                ULONG AddressSpace = 0; /* Memory space */
//                if (!HalTranslateBusAddress(
//                   DeviceNode->ResourceList->List[i].InterfaceType,
//                   DeviceNode->ResourceList->List[i].BusNumber,
//                   DescriptorRaw->u.Memory.Start,
//                   &AddressSpace,
//                   &DescriptorTranslated->u.Memory.Start))
//                {
//                   Status = STATUS_UNSUCCESSFUL;
//                   DPRINT1("Failed to translate memory resource (Start: 0x%I64x)\n", DescriptorRaw->u.Memory.Start.QuadPart);
//                   goto cleanup;
//                }
//
//                if (AddressSpace != 0)
//                {
//                    DPRINT1("Guessed incorrect address space: 0 -> 1\n");
//
//                    /* This should never happen for memory space */
//                    ASSERT(FALSE);
//                }
//             }
//
//             case CmResourceTypeDma:
//             case CmResourceTypeBusNumber:
//             case CmResourceTypeDevicePrivate:
//             case CmResourceTypeDeviceSpecific:
//                /* Nothing to do */
//                break;
//             default:
//                DPRINT1("Unknown resource descriptor type 0x%x\n", DescriptorRaw->Type);
//                Status = STATUS_NOT_IMPLEMENTED;
//                goto cleanup;
//          }
//       }
//    }
//    return STATUS_SUCCESS;
//
// cleanup:
//    /* Yes! Also delete ResourceList because ResourceList and
//     * ResourceListTranslated should be a pair! */
//    ExFreePool(DeviceNode->ResourceList);
//    DeviceNode->ResourceList = NULL;
//    if (DeviceNode->ResourceListTranslated)
//    {
//       ExFreePool(DeviceNode->ResourceListTranslated);
//       DeviceNode->ResourceList = NULL;
//    }
//    return Status;
// }
//
// NTSTATUS
// NTAPI
// IopAssignDeviceResources(
//    IN PDEVICE_NODE DeviceNode)
// {
//    NTSTATUS Status;
//    ULONG ListSize;
//
//    Status = IopFilterResourceRequirements(DeviceNode);
//    if (!NT_SUCCESS(Status))
//        goto ByeBye;
//
//    if (!DeviceNode->BootResources && !DeviceNode->ResourceRequirements)
//    {
//       /* No resource needed for this device */
//       DeviceNode->ResourceList = NULL;
//       DeviceNode->ResourceListTranslated = NULL;
//       PiSetDevNodeState(DeviceNode, DeviceNodeResourcesAssigned);
//       DeviceNode->Flags |= DNF_NO_RESOURCE_REQUIRED;
//
//       return STATUS_SUCCESS;
//    }
//
//    if (DeviceNode->BootResources)
//    {
//        ListSize = PnpDetermineResourceListSize(DeviceNode->BootResources);
//
//        DeviceNode->ResourceList = ExAllocatePool(PagedPool, ListSize);
//        if (!DeviceNode->ResourceList)
//        {
//            Status = STATUS_NO_MEMORY;
//            goto ByeBye;
//        }
//
//        RtlCopyMemory(DeviceNode->ResourceList, DeviceNode->BootResources, ListSize);
//
//        Status = IopDetectResourceConflict(DeviceNode->ResourceList, FALSE, NULL);
//        if (!NT_SUCCESS(Status))
//        {
//            DPRINT1("Boot resources for %wZ cause a resource conflict!\n", &DeviceNode->InstancePath);
//            ExFreePool(DeviceNode->ResourceList);
//            DeviceNode->ResourceList = NULL;
//        }
//    }
//    else
//    {
//        /* We'll make this from the requirements */
//        DeviceNode->ResourceList = NULL;
//    }
//
//    /* No resources requirements */
//    if (!DeviceNode->ResourceRequirements)
//        goto Finish;
//
//    /* Call HAL to fixup our resource requirements list */
//    HalAdjustResourceList(&DeviceNode->ResourceRequirements);
//
//    /* Add resource requirements that aren't in the list we already got */
//    Status = IopFixupResourceListWithRequirements(DeviceNode->ResourceRequirements,
//                                                  &DeviceNode->ResourceList);
//    if (!NT_SUCCESS(Status))
//    {
//        DPRINT1("Failed to fixup a resource list from supplied resources for %wZ\n", &DeviceNode->InstancePath);
//        DeviceNode->Problem = CM_PROB_NORMAL_CONFLICT;
//        goto ByeBye;
//    }
//
//    /* IopFixupResourceListWithRequirements should NEVER give us a conflicting list */
//    ASSERT(IopDetectResourceConflict(DeviceNode->ResourceList, FALSE, NULL) != STATUS_CONFLICTING_ADDRESSES);
//
// Finish:
//    Status = IopTranslateDeviceResources(DeviceNode);
//    if (!NT_SUCCESS(Status))
//    {
//        DeviceNode->Problem = CM_PROB_TRANSLATION_FAILED;
//        DPRINT1("Failed to translate resources for %wZ\n", &DeviceNode->InstancePath);
//        goto ByeBye;
//    }
//
//    Status = IopUpdateResourceMapForPnPDevice(DeviceNode);
//    if (!NT_SUCCESS(Status))
//        goto ByeBye;
//
//    Status = IopUpdateControlKeyWithResources(DeviceNode);
//    if (!NT_SUCCESS(Status))
//        goto ByeBye;
//
//    PiSetDevNodeState(DeviceNode, DeviceNodeResourcesAssigned);
//
//    return STATUS_SUCCESS;
//
// ByeBye:
//    if (DeviceNode->ResourceList)
//    {
//       ExFreePool(DeviceNode->ResourceList);
//       DeviceNode->ResourceList = NULL;
//    }
//
//    DeviceNode->ResourceListTranslated = NULL;
//
//    return Status;
// }
//
// static
// BOOLEAN
// IopCheckForResourceConflict(
//    IN PCM_RESOURCE_LIST ResourceList1,
//    IN PCM_RESOURCE_LIST ResourceList2,
//    IN BOOLEAN Silent,
//    OUT OPTIONAL PCM_PARTIAL_RESOURCE_DESCRIPTOR ConflictingDescriptor)
// {
//    ULONG i, ii;
//    BOOLEAN Result = FALSE;
//    PCM_FULL_RESOURCE_DESCRIPTOR FullDescriptor;
//
//    FullDescriptor = &ResourceList1->List[0];
//    for (i = 0; i < ResourceList1->Count; i++)
//    {
//       PCM_PARTIAL_RESOURCE_LIST ResList = &FullDescriptor->PartialResourceList;
//       FullDescriptor = CmiGetNextResourceDescriptor(FullDescriptor);
//
//       for (ii = 0; ii < ResList->Count; ii++)
//       {
//         /* Partial resource descriptors can be of variable size (CmResourceTypeDeviceSpecific),
//            but only one is allowed and it must be the last one in the list! */
//          PCM_PARTIAL_RESOURCE_DESCRIPTOR ResDesc = &ResList->PartialDescriptors[ii];
//
//          Result = IopCheckResourceDescriptor(ResDesc,
//                                              ResourceList2,
//                                              Silent,
//                                              ConflictingDescriptor);
//          if (Result) goto ByeBye;
//       }
//    }
//
// ByeBye:
//
//    return Result;
// }
//
// NTSTATUS NTAPI
// IopDetectResourceConflict(
//    IN PCM_RESOURCE_LIST ResourceList,
//    IN BOOLEAN Silent,
//    OUT OPTIONAL PCM_PARTIAL_RESOURCE_DESCRIPTOR ConflictingDescriptor)
// {
//    OBJECT_ATTRIBUTES ObjectAttributes;
//    UNICODE_STRING KeyName;
//    HANDLE ResourceMapKey = NULL, ChildKey2 = NULL, ChildKey3 = NULL;
//    ULONG KeyInformationLength, RequiredLength, KeyValueInformationLength, KeyNameInformationLength;
//    PKEY_BASIC_INFORMATION KeyInformation;
//    PKEY_VALUE_PARTIAL_INFORMATION KeyValueInformation;
//    PKEY_VALUE_BASIC_INFORMATION KeyNameInformation;
//    ULONG ChildKeyIndex1 = 0, ChildKeyIndex2, ChildKeyIndex3;
//    NTSTATUS Status;
//
//    RtlInitUnicodeString(&KeyName, L"\\Registry\\Machine\\HARDWARE\\RESOURCEMAP");
//    InitializeObjectAttributes(&ObjectAttributes,
//                               &KeyName,
//                               OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                               NULL,
//                               NULL);
//    Status = ZwOpenKey(&ResourceMapKey, KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE, &ObjectAttributes);
//    if (!NT_SUCCESS(Status))
//    {
//       /* The key is missing which means we are the first device */
//       return STATUS_SUCCESS;
//    }
//
//    while (TRUE)
//    {
//       Status = ZwEnumerateKey(ResourceMapKey,
//                               ChildKeyIndex1,
//                               KeyBasicInformation,
//                               NULL,
//                               0,
//                               &RequiredLength);
//       if (Status == STATUS_NO_MORE_ENTRIES)
//           break;
//       else if (Status == STATUS_BUFFER_OVERFLOW || Status == STATUS_BUFFER_TOO_SMALL)
//       {
//           KeyInformationLength = RequiredLength;
//           KeyInformation = ExAllocatePoolWithTag(PagedPool,
//                                                  KeyInformationLength,
//                                                  TAG_IO);
//           if (!KeyInformation)
//           {
//               Status = STATUS_INSUFFICIENT_RESOURCES;
//               goto cleanup;
//           }
//
//           Status = ZwEnumerateKey(ResourceMapKey,
//                                   ChildKeyIndex1,
//                                   KeyBasicInformation,
//                                   KeyInformation,
//                                   KeyInformationLength,
//                                   &RequiredLength);
//       }
//       else
//          goto cleanup;
//       ChildKeyIndex1++;
//       if (!NT_SUCCESS(Status))
//       {
//           ExFreePoolWithTag(KeyInformation, TAG_IO);
//           goto cleanup;
//       }
//
//       KeyName.Buffer = KeyInformation->Name;
//       KeyName.MaximumLength = KeyName.Length = (USHORT)KeyInformation->NameLength;
//       InitializeObjectAttributes(&ObjectAttributes,
//                                  &KeyName,
//                                  OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                  ResourceMapKey,
//                                  NULL);
//       Status = ZwOpenKey(&ChildKey2,
//                          KEY_ENUMERATE_SUB_KEYS | KEY_QUERY_VALUE,
//                          &ObjectAttributes);
//       ExFreePoolWithTag(KeyInformation, TAG_IO);
//       if (!NT_SUCCESS(Status))
//           goto cleanup;
//
//       ChildKeyIndex2 = 0;
//       while (TRUE)
//       {
//           Status = ZwEnumerateKey(ChildKey2,
//                                   ChildKeyIndex2,
//                                   KeyBasicInformation,
//                                   NULL,
//                                   0,
//                                   &RequiredLength);
//           if (Status == STATUS_NO_MORE_ENTRIES)
//               break;
//           else if (Status == STATUS_BUFFER_TOO_SMALL)
//           {
//               KeyInformationLength = RequiredLength;
//               KeyInformation = ExAllocatePoolWithTag(PagedPool,
//                                                      KeyInformationLength,
//                                                      TAG_IO);
//               if (!KeyInformation)
//               {
//                   Status = STATUS_INSUFFICIENT_RESOURCES;
//                   goto cleanup;
//               }
//
//               Status = ZwEnumerateKey(ChildKey2,
//                                       ChildKeyIndex2,
//                                       KeyBasicInformation,
//                                       KeyInformation,
//                                       KeyInformationLength,
//                                       &RequiredLength);
//           }
//           else
//               goto cleanup;
//           ChildKeyIndex2++;
//           if (!NT_SUCCESS(Status))
//           {
//               ExFreePoolWithTag(KeyInformation, TAG_IO);
//               goto cleanup;
//           }
//
//           KeyName.Buffer = KeyInformation->Name;
//           KeyName.MaximumLength = KeyName.Length = (USHORT)KeyInformation->NameLength;
//           InitializeObjectAttributes(&ObjectAttributes,
//                                      &KeyName,
//                                      OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
//                                      ChildKey2,
//                                      NULL);
//           Status = ZwOpenKey(&ChildKey3, KEY_QUERY_VALUE, &ObjectAttributes);
//           ExFreePoolWithTag(KeyInformation, TAG_IO);
//           if (!NT_SUCCESS(Status))
//               goto cleanup;
//
//           ChildKeyIndex3 = 0;
//           while (TRUE)
//           {
//               Status = ZwEnumerateValueKey(ChildKey3,
//                                            ChildKeyIndex3,
//                                            KeyValuePartialInformation,
//                                            NULL,
//                                            0,
//                                            &RequiredLength);
//               if (Status == STATUS_NO_MORE_ENTRIES)
//                   break;
//               else if (Status == STATUS_BUFFER_TOO_SMALL)
//               {
//                   KeyValueInformationLength = RequiredLength;
//                   KeyValueInformation = ExAllocatePoolWithTag(PagedPool,
//                                                               KeyValueInformationLength,
//                                                               TAG_IO);
//                   if (!KeyValueInformation)
//                   {
//                       Status = STATUS_INSUFFICIENT_RESOURCES;
//                       goto cleanup;
//                   }
//
//                   Status = ZwEnumerateValueKey(ChildKey3,
//                                                ChildKeyIndex3,
//                                                KeyValuePartialInformation,
//                                                KeyValueInformation,
//                                                KeyValueInformationLength,
//                                                &RequiredLength);
//               }
//               else
//                   goto cleanup;
//               if (!NT_SUCCESS(Status))
//               {
//                   ExFreePoolWithTag(KeyValueInformation, TAG_IO);
//                   goto cleanup;
//               }
//
//               Status = ZwEnumerateValueKey(ChildKey3,
//                                            ChildKeyIndex3,
//                                            KeyValueBasicInformation,
//                                            NULL,
//                                            0,
//                                            &RequiredLength);
//               if (Status == STATUS_BUFFER_TOO_SMALL)
//               {
//                   KeyNameInformationLength = RequiredLength;
//                   KeyNameInformation = ExAllocatePoolWithTag(PagedPool,
//                                                              KeyNameInformationLength + sizeof(WCHAR),
//                                                              TAG_IO);
//                   if (!KeyNameInformation)
//                   {
//                       Status = STATUS_INSUFFICIENT_RESOURCES;
//                       goto cleanup;
//                   }
//
//                   Status = ZwEnumerateValueKey(ChildKey3,
//                                                ChildKeyIndex3,
//                                                KeyValueBasicInformation,
//                                                KeyNameInformation,
//                                                KeyNameInformationLength,
//                                                &RequiredLength);
//               }
//               else
//                   goto cleanup;
//               ChildKeyIndex3++;
//               if (!NT_SUCCESS(Status))
//               {
//                   ExFreePoolWithTag(KeyNameInformation, TAG_IO);
//                   goto cleanup;
//               }
//
//               KeyNameInformation->Name[KeyNameInformation->NameLength / sizeof(WCHAR)] = UNICODE_NULL;
//
//               /* Skip translated entries */
//               if (wcsstr(KeyNameInformation->Name, L".Translated"))
//               {
//                   ExFreePoolWithTag(KeyNameInformation, TAG_IO);
//                   ExFreePoolWithTag(KeyValueInformation, TAG_IO);
//                   continue;
//               }
//
//               ExFreePoolWithTag(KeyNameInformation, TAG_IO);
//
//               if (IopCheckForResourceConflict(ResourceList,
//                                               (PCM_RESOURCE_LIST)KeyValueInformation->Data,
//                                               Silent,
//                                               ConflictingDescriptor))
//               {
//                   ExFreePoolWithTag(KeyValueInformation, TAG_IO);
//                   Status = STATUS_CONFLICTING_ADDRESSES;
//                   goto cleanup;
//               }
//
//               ExFreePoolWithTag(KeyValueInformation, TAG_IO);
//           }
//       }
//    }
//
// cleanup:
//    if (ResourceMapKey != NULL)
//        ObCloseHandle(ResourceMapKey, KernelMode);
//    if (ChildKey2 != NULL)
//        ObCloseHandle(ChildKey2, KernelMode);
//    if (ChildKey3 != NULL)
//        ObCloseHandle(ChildKey3, KernelMode);
//
//    if (Status == STATUS_NO_MORE_ENTRIES)
//        Status = STATUS_SUCCESS;
//
//    return Status;
// }
//
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
