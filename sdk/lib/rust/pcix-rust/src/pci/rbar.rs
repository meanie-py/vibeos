// pci/rbar.rs — drivers/bus/pcix/pci/rbar.c (upstream 4b3f691a7ba, #9592): PCI Express
// Resizable BAR support. Hand-ported, $0.
#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::{
    PCI_COMMON_HEADER, PCI_EXPRESS_ENHANCED_CAPABILITY_HEADER, PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    PCSTR, PIO_RESOURCE_DESCRIPTOR, IO_RESOURCE_DESCRIPTOR, RtlCmDecodeMemIoResource,
    RtlFindLeastSignificantBit, RtlFindMostSignificantBit, RtlIoEncodeMemIoResource,
};
use ntkrnl_sys::driver_ddk::DbgPrint;
use core::ffi::c_void;

/* GLOBALS ********************************************************************/

#[inline(always)]
const fn PCI_RBAR_SIZE_TO_LENGTH(Bit: ULONGLONG) -> ULONGLONG {
    1u64 << (Bit + 20)
}

// pci.h function-like macros: one capability and control register pair per BAR.
#[inline(always)]
const fn PCI_RBAR_ENTRY_CAPABILITY(n: ULONG) -> ULONG {
    0x04 + (n * 8)
}
#[inline(always)]
const fn PCI_RBAR_ENTRY_CONTROL(n: ULONG) -> ULONG {
    0x08 + (n * 8)
}

const PCI_COMMON_HEADER_COMMAND_OFFSET: ULONG = core::mem::offset_of!(PCI_COMMON_HEADER, Command) as ULONG;

/* FUNCTIONS ******************************************************************/

/// Records which BARs of a new function can be resized and the sizes each of them
/// supports.
#[no_mangle]
pub unsafe extern "system" fn PciGetResizableBarCapability(PdoExtension: PPCI_PDO_EXTENSION) {
    let State: PPCI_RESIZABLE_BAR_STATE = core::ptr::addr_of_mut!((*PdoExtension).ResizableBarState);
    let mut Header: PCI_EXPRESS_ENHANCED_CAPABILITY_HEADER = core::mem::zeroed();
    let Offset: ULONG;
    let mut Entry: ULONG;
    let mut EntryCount: ULONG;
    let mut Control: ULONG = 0;
    let mut SizeMask: ULONG = 0;
    let mut BarIndex: ULONG;

    PAGED_CODE();

    core::ptr::write_bytes(State, 0, 1);

    /* Bridges keep their windows after the first two BARs, so only endpoints are resized */
    if (*PdoExtension).HeaderType as u32 != PCI_DEVICE_TYPE {
        return;
    }

    Offset = PciReadDeviceExtendedCapability(
        PdoExtension,
        PCI_RBAR_EXTENDED_CAP_ID,
        &mut Header,
        core::mem::size_of::<PCI_EXPRESS_ENHANCED_CAPABILITY_HEADER>() as ULONG,
    );
    if Offset == 0 {
        return;
    }

    /* Only the first control register holds the number of entries */
    PciReadDeviceConfig(
        PdoExtension,
        &mut Control as *mut ULONG as PVOID,
        Offset + PCI_RBAR_ENTRY_CONTROL(0),
        core::mem::size_of::<ULONG>() as ULONG,
    );
    EntryCount = (Control & PCI_RBAR_CONTROL_COUNT_MASK) >> PCI_RBAR_CONTROL_COUNT_SHIFT;
    if EntryCount > PCI_RBAR_MAX_ENTRIES {
        EntryCount = PCI_RBAR_MAX_ENTRIES;
    }

    (*State).CapabilityPtr = Offset as USHORT;

    Entry = 0;
    while Entry < EntryCount {
        PciReadDeviceConfig(
            PdoExtension,
            &mut SizeMask as *mut ULONG as PVOID,
            Offset + PCI_RBAR_ENTRY_CAPABILITY(Entry),
            core::mem::size_of::<ULONG>() as ULONG,
        );
        PciReadDeviceConfig(
            PdoExtension,
            &mut Control as *mut ULONG as PVOID,
            Offset + PCI_RBAR_ENTRY_CONTROL(Entry),
            core::mem::size_of::<ULONG>() as ULONG,
        );

        SizeMask >>= PCI_RBAR_CAPABILITY_SIZES_SHIFT;
        BarIndex = Control & PCI_RBAR_CONTROL_BAR_INDEX_MASK;

        /* Skip entries with no sizes, a BAR that cannot exist, or a BAR already seen */
        if SizeMask != 0
            && BarIndex < PCI_RBAR_MAX_ENTRIES
            && (*State).SizeMask[BarIndex as usize] == 0
        {
            (*State).SizeMask[BarIndex as usize] = SizeMask;
            (*State).EntryIndex[BarIndex as usize] = Entry as UCHAR;

            DbgPrint(
                b"(%s:%d) PCI: BAR %lu of %p is resizable, sizes 0x%08lx\n\0".as_ptr() as PCSTR,
                b"drivers/bus/pcix/pci/rbar.c\0".as_ptr() as PCSTR,
                91,
                BarIndex,
                PdoExtension,
                SizeMask,
            );
        }

        Entry += 1;
    }
}

unsafe fn PciFillResizedBarRequirement(
    Descriptor: PIO_RESOURCE_DESCRIPTOR,
    Limit: PIO_RESOURCE_DESCRIPTOR,
    Length: ULONGLONG,
    Option: UCHAR,
) -> BOOLEAN {
    let Status: NTSTATUS;

    *Descriptor = *Limit;
    (*Descriptor).Option = Option;
    (*Descriptor).ShareDisposition = CmResourceShareDeviceExclusive;

    Status = RtlIoEncodeMemIoResource(
        Descriptor,
        CmResourceTypeMemory as UCHAR,
        Length,
        Length,
        (*Limit).u.Memory.MinimumAddress.QuadPart as ULONGLONG,
        (*Limit).u.Memory.MaximumAddress.QuadPart as ULONGLONG,
    );
    (Status >= 0) as BOOLEAN
}

/// Writes the requirements for the sizes a resizable BAR can grow to. They go ahead of
/// the default requirement, which the caller then lists as the alternative. The largest
/// size is preferred, and the smallest supported size follows it when that is also
/// bigger than the default. `Descriptors` receives up to two descriptors, or is NULL to
/// only count them. Returns the number written, 0 when the BAR cannot grow.
#[no_mangle]
pub unsafe extern "system" fn PciAddResizableBarRequirements(
    PdoExtension: PPCI_PDO_EXTENSION,
    BarIndex: ULONG,
    Limit: PIO_RESOURCE_DESCRIPTOR,
    Descriptors: PIO_RESOURCE_DESCRIPTOR,
) -> ULONG {
    let mut Resized: [IO_RESOURCE_DESCRIPTOR; 2] = core::mem::zeroed();
    let Largest: ULONGLONG;
    let Smallest: ULONGLONG;
    let SizeMask: ULONG;
    let mut Count: ULONG;
    PAGED_CODE();

    /* Only a memory BAR asking for less than 4GB is offered other sizes */
    if BarIndex >= PCI_RBAR_MAX_ENTRIES || (*Limit).Type as u32 != CmResourceTypeMemory {
        return 0;
    }

    SizeMask = (*PdoExtension).ResizableBarState.SizeMask[BarIndex as usize];
    if SizeMask == 0 {
        return 0;
    }

    /* Built even when only counting, so a size that cannot be described is never counted */
    Largest = PCI_RBAR_SIZE_TO_LENGTH(RtlFindMostSignificantBit(SizeMask as ULONGLONG) as ULONGLONG);
    if Largest <= (*Limit).u.Memory.Length as ULONGLONG
        || PciFillResizedBarRequirement(&mut Resized[0], Limit, Largest, IO_RESOURCE_PREFERRED as UCHAR) == 0
    {
        return 0;
    }
    Count = 1;

    Smallest = PCI_RBAR_SIZE_TO_LENGTH(RtlFindLeastSignificantBit(SizeMask as ULONGLONG) as ULONGLONG);
    if Smallest > (*Limit).u.Memory.Length as ULONGLONG
        && Smallest < Largest
        && PciFillResizedBarRequirement(&mut Resized[1], Limit, Smallest, IO_RESOURCE_ALTERNATIVE as UCHAR) != 0
    {
        Count += 1;
    }

    if !Descriptors.is_null() {
        core::ptr::copy_nonoverlapping(Resized.as_ptr(), Descriptors, Count as usize);
    }

    Count
}

unsafe fn PciSelectResizableBarSize(PdoExtension: PPCI_PDO_EXTENSION, BarIndex: ULONG, Length: ULONGLONG) {
    let State: PPCI_RESIZABLE_BAR_STATE = core::ptr::addr_of_mut!((*PdoExtension).ResizableBarState);
    let Megabytes: ULONGLONG;
    let SizeBit: ULONG;
    let mut Control: ULONG = 0;
    let Offset: ULONG;

    /* Only a power of two of at least a megabyte can be specified in the size field */
    Megabytes = Length >> 20;
    if Megabytes == 0 || (Megabytes & (Megabytes - 1)) != 0 || (Length & 0xFFFFF) != 0 {
        return;
    }

    if ((*State).SizeMask[BarIndex as usize] as ULONGLONG & Megabytes) == 0 {
        return;
    }

    SizeBit = RtlFindMostSignificantBit(Megabytes) as ULONG;

    Offset = (*State).CapabilityPtr as ULONG + PCI_RBAR_ENTRY_CONTROL((*State).EntryIndex[BarIndex as usize] as ULONG);
    PciReadDeviceConfig(PdoExtension, &mut Control as *mut ULONG as PVOID, Offset, core::mem::size_of::<ULONG>() as ULONG);
    if ((Control & PCI_RBAR_CONTROL_SIZE_MASK) >> PCI_RBAR_CONTROL_SIZE_SHIFT) == SizeBit {
        return;
    }

    Control &= !PCI_RBAR_CONTROL_SIZE_MASK;
    Control |= SizeBit << PCI_RBAR_CONTROL_SIZE_SHIFT;
    PciWriteDeviceConfig(PdoExtension, &mut Control as *mut ULONG as PVOID, Offset, core::mem::size_of::<ULONG>() as ULONG);

    // DPRINT (compiled out: rbar.c defines NDEBUG)
    // "PCI: BAR %lu of %p resized to %I64u MB\n", BarIndex, PdoExtension, Megabytes
}

/// Sets every resizable BAR to the length it was assigned. The memory decode has to be
/// off, since the size field changes what the BAR decodes. Critical devices call this
/// from the IPI that updates their hardware.
#[no_mangle]
pub unsafe extern "system" fn PciApplyResizableBarSizes(PdoExtension: PPCI_PDO_EXTENSION) {
    let mut Current: PCM_PARTIAL_RESOURCE_DESCRIPTOR;
    let mut Command: USHORT = 0;
    let mut BarIndex: ULONG;

    if (*PdoExtension).ResizableBarState.CapabilityPtr == 0 || (*PdoExtension).Resources.is_null() {
        return;
    }

    /* A function whose decodes could not be turned off keeps the sizes it has */
    PciReadDeviceConfig(
        PdoExtension,
        &mut Command as *mut USHORT as PVOID,
        PCI_COMMON_HEADER_COMMAND_OFFSET,
        core::mem::size_of::<USHORT>() as ULONG,
    );
    if (Command as u32 & PCI_ENABLE_MEMORY_SPACE) != 0 {
        // DPRINT (compiled out: rbar.c defines NDEBUG)
        // "PCI: Memory decode of %p is still on, not resizing BARs\n", PdoExtension
        return;
    }

    BarIndex = 0;
    while BarIndex < PCI_RBAR_MAX_ENTRIES {
        if (*PdoExtension).ResizableBarState.SizeMask[BarIndex as usize] != 0 {
            Current = core::ptr::addr_of_mut!((*(*PdoExtension).Resources).Current[BarIndex as usize]);
            if (*Current).Type as u32 == CmResourceTypeMemory || (*Current).Type as u32 == CmResourceTypeMemoryLarge {
                PciSelectResizableBarSize(
                    PdoExtension,
                    BarIndex,
                    RtlCmDecodeMemIoResource(Current, core::ptr::null_mut()),
                );
            }
        }
        BarIndex += 1;
    }
}
