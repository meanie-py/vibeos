//! HAL ACPI — table walking, FADT cache, HalAcpiGetTable + HalpSetupAcpiPhase0.
//! Translated from `hal/halx86/acpi/halacpi.c`. Now built for i386 too
//! (task #4 ACPI-HAL backport): provides the i386 ACPI HAL identity +
//! table provider, superseding the legacy halpcat.rs PIC-HAL stubs.
//! Interrupt delivery stays PIC (pic*/timer/portio modules untouched).

use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePoolWithTag, KeBugCheckEx, KeFindConfigurationNextEntry,
    RtlInitUnicodeString,
};
use ntkrnl_sys::hal_ddk::{
    CmResourceTypeInterrupt, HalDisplayString, HalpAllocPhysicalMemory, HalpAssignSlotResources,
    HalpBusType, HalpFindBusAddressTranslation, HalpGetNMICrashFlag, HalpInitializePciStubs,
    HalpMapPhysicalMemory64, HalpRegisterPciDebuggingDeviceInfo, HalpTranslateBusAddress,
    HalpUnmapVirtualAddress, _CONFIGURATION_CLASS_AdapterClass as AdapterClass,
    _CONFIGURATION_TYPE_MultiFunctionAdapter as MultiFunctionAdapter, _INTERFACE_TYPE_Eisa,
    _INTERFACE_TYPE_Internal, _INTERFACE_TYPE_Isa, _INTERFACE_TYPE_MicroChannel,
    _INTERFACE_TYPE_PNPBus, ACPI_BIOS_MULTI_NODE, ACPI_CACHED_TABLE, ACPI_E820_ENTRY,
    ACPI_TMR_VAL_EXT, BOOLEAN, BOOT_TABLE, CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE,
    CM_RESOURCE_PORT_16_BIT_DECODE, DBGP_SIGNATURE, DSDT_SIGNATURE, FADT, FADT_SIGNATURE,
    FAST_MUTEX, INTERFACE_TYPE, IO_RESOURCE_DESCRIPTOR, IO_RESOURCE_LIST,
    IO_RESOURCE_REQUIREMENTS_LIST, LIST_ENTRY, NTSTATUS, PACPI_BIOS_MULTI_NODE, PACPI_CACHED_TABLE,
    PACPI_SRAT, PAGE_SHIFT, PAGE_SIZE, PBOOT_TABLE, PCHAR, PCM_PARTIAL_RESOURCE_LIST,
    PCONFIGURATION_COMPONENT_DATA, PCSTR, PDEBUG_PORT_TABLE, PDESCRIPTION_HEADER, PFADT, PFN_COUNT,
    PHYSICAL_ADDRESS, PIO_RESOURCE_REQUIREMENTS_LIST, PLIST_ENTRY,
    PLOADER_PARAMETER_BLOCK, PLOADER_PARAMETER_EXTENSION, PRSDT, PUNICODE_STRING, PVOID, PWCHAR,
    PXSDT, RSDT, RSDT_SIGNATURE, ULONG, ULONG_PTR, USHORT, XSDT, _UNICODE_STRING,
};
// HARDWARE_PTE is arch-split in bindgen: amd64 = PHARDWARE_PTE (x64 PTE),
// i386 = PHARDWARE_PTE_X86 (32-bit / PAE PTE). Alias so the body stays
// arch-neutral (HalpPteForFlush + HalAddressToPte cast).
#[cfg(target_arch = "x86_64")]
use ntkrnl_sys::hal_ddk::PHARDWARE_PTE;
#[cfg(target_arch = "x86")]
use ntkrnl_sys::hal_ddk::PHARDWARE_PTE_X86 as PHARDWARE_PTE;
// i386: HalpInitializePciBus is owned by bussupp.rs (i386 bus infra); the
// amd64-ACPI local def above is x86_64-gated, so resolve the call through
// the hal_ddk extern (links to bussupp's #[no_mangle] symbol).
#[cfg(target_arch = "x86")]
use ntkrnl_sys::hal_ddk::HalpInitializePciBus;

// MmMapIoSpace / MmUnmapIoSpace bridge — ddk and hal_ddk both declare these
// but with disjoint LARGE_INTEGER types in their signatures.  Wrap so we can
// pass our hal_ddk PHYSICAL_ADDRESS without manual transmutes everywhere.
#[inline(always)]
unsafe fn MmMapIoSpace(addr: PHYSICAL_ADDRESS, size: u64, cache: u32) -> PVOID {
    // ddk MmMapIoSpace takes SIZE_T (u32 on i386, u64 on amd64) — `as _`
    // coerces per-arch from the arch-neutral u64 the HAL body uses.
    ntkrnl_sys::ddk::MmMapIoSpace(core::mem::transmute(addr), size as _, cache)
}
#[inline(always)]
unsafe fn MmUnmapIoSpace(va: PVOID, size: u64) {
    ntkrnl_sys::ddk::MmUnmapIoSpace(va, size as _);
}

// FAST_MUTEX bridge — ddk and hal_ddk define the same struct under separate
// module paths. ExAcquire/Release are exported (amd64 stdcall); only
// ExInitializeFastMutex is FORCEINLINE — inline its body locally.
#[inline(always)]
unsafe fn ExInitializeFastMutex(m: *mut FAST_MUTEX) {
    ntkrnl_sys::inline::ex::ExInitializeFastMutex(m as *mut _);
}
// amd64: ExAcquire/ReleaseFastMutex are real stdcall exports
// (ntoskrnl.spec `-arch=x86_64`). i386: those names are FORCEINLINE in the
// DDK header (not exported); the real i386 export is the FASTCALL
// `Exi{Acquire,Release}FastMutex` (ntoskrnl.spec lines 183-184). Bridge
// per-arch so the symbol resolves on both.
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ExAcquireFastMutex(m: *mut FAST_MUTEX) {
    ntkrnl_sys::ddk::ExAcquireFastMutex(m as *mut _);
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn ExReleaseFastMutex(m: *mut FAST_MUTEX) {
    ntkrnl_sys::ddk::ExReleaseFastMutex(m as *mut _);
}
// extern "fastcall" auto-decorates to `@ExiAcquireFastMutex@4` on i386
// (1 ptr arg = 4 bytes) — exactly the ntoskrnl.spec export. No
// #[link_name] (that would double-decorate).
#[cfg(target_arch = "x86")]
unsafe extern "fastcall" {
    fn ExiAcquireFastMutex(m: *mut core::ffi::c_void);
    fn ExiReleaseFastMutex(m: *mut core::ffi::c_void);
}
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn ExAcquireFastMutex(m: *mut FAST_MUTEX) {
    ExiAcquireFastMutex(m as *mut core::ffi::c_void);
}
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn ExReleaseFastMutex(m: *mut FAST_MUTEX) {
    ExiReleaseFastMutex(m as *mut core::ffi::c_void);
}
use ntkrnl_sys::hal_ddk::HalpReportResourceUsage;
use ntkrnl_sys::hal_ddk::_CM_SHARE_DISPOSITION_CmResourceShareShared as CmResourceShareShared;
use ntkrnl_sys::hal_ddk::_MEMORY_CACHING_TYPE_MmNonCached as MmNonCached;
use ntkrnl_sys::hal_ddk::_POOL_TYPE_NonPagedPool as NonPagedPool;
use ntkrnl_sys::hal_ddk::_POOL_TYPE_PagedPool as PagedPool;
// LIST_ENTRY helpers — `ntkrnl_sys::base` and `hal_ddk` ship the same
// underlying struct under different module paths. Wrap the base helpers to
// take `*mut hal_ddk::LIST_ENTRY` for ergonomics in this file.
#[inline(always)]
unsafe fn InitializeListHead(head: *mut LIST_ENTRY) {
    ntkrnl_sys::base::InitializeListHead(head as *mut _);
}
#[inline(always)]
unsafe fn InsertTailList(head: *mut LIST_ENTRY, entry: *mut LIST_ENTRY) {
    ntkrnl_sys::base::InsertTailList(head as *mut _, entry as *mut _);
}

// RtlZeroMemory is a memset wrapper; use core::ptr::write_bytes.
#[inline(always)]
unsafe fn RtlZeroMemory(dst: *mut u8, size: usize) {
    core::ptr::write_bytes(dst, 0, size);
}

// ---------------------------------------------------------------------------
// Constants — apicp.h-style HAL-private defines, plus a few NT codes.
// ---------------------------------------------------------------------------

// 4-char ACPI signatures.  Bindgen exports many of these but not all.
const XSDT_SIGNATURE: ULONG = u32::from_le_bytes(*b"XSDT");
const SRAT_SIGNATURE: ULONG = u32::from_le_bytes(*b"SRAT");
const BOOT_SIGNATURE: ULONG = u32::from_le_bytes(*b"BOOT");
const SLIT_SIGNATURE: ULONG = u32::from_le_bytes(*b"SLIT");

// TAG_HAL = 'lHaH' / ' laH' — HAL pool tag (matches halp.h).
const TAG_HAL: ULONG = u32::from_be_bytes(*b" laH"); // C literal ' laH' -- hal/halx86/include/hal.h

const STATUS_SUCCESS: NTSTATUS = 0x00000000;
const STATUS_INSUFFICIENT_RESOURCES: NTSTATUS = 0xC000009A_u32 as NTSTATUS;
const STATUS_NOT_FOUND: NTSTATUS = 0xC0000225_u32 as NTSTATUS;
const STATUS_NO_SUCH_DEVICE: NTSTATUS = 0xC000000E_u32 as NTSTATUS;

// Bug check 0x79 (sdk/include/reactos/mc/bugcodes.mc).
const MISMATCHED_HAL: ULONG = 0x79;

// Alternative bus-type names used by sonnet's switch arms.
const PNPBus: i32 = _INTERFACE_TYPE_PNPBus as i32;

// MACHINE_TYPE_* constants (sdk/include/reactos/something).
const MACHINE_TYPE_ISA: u32 = 0;
const MACHINE_TYPE_EISA: u32 = 1;
const MACHINE_TYPE_MCA: u32 = 2;

// HAL-private numeric constants from halp.h.
// Stays 1 page: UEFI leaves almost nothing allocatable below 1MB (measured
// low map: one 2-page FirmwareTemporary run at 0x37000) — a >1-page run
// simply doesn't exist. Only the SIPI target itself must be <1MB; the SMP
// identity-map page-table scratch (HalpApPtScratch below) allocates
// separately with a 4GB cap.
const HALP_LOW_STUB_SIZE_IN_PAGES: PFN_COUNT = 1; // nolint: const-contradiction (deliberate UEFI divergence from halp.h:65/67 (5 amd64 / 3 x86), see above)

// SMP (smp-bringup M2): 3 pages of PDPT/PD/PT scratch smp_processor.rs uses
// to identity-map the low stub (VA==PA) around the SIPI window — the boot
// identity map is long gone by Phase 1. Page-table pages can live anywhere
// <4GB; allocated at Phase 0 alongside the low stub. Zero-cost when unused.
#[unsafe(no_mangle)]
pub static mut HalpApPtScratchPhysicalAddress: PHYSICAL_ADDRESS =
    PHYSICAL_ADDRESS { QuadPart: 0 };
#[unsafe(no_mangle)]
pub static mut HalpApPtScratch: PVOID = core::ptr::null_mut();
const HALP_AP_PT_SCRATCH_PAGES: PFN_COUNT = 3;

// FADT bit constant — 8042 keyboard controller present.
const ACPI_FADT_8042: ULONG = 2;

#[inline(always)]
const fn nt_success(s: NTSTATUS) -> bool {
    s >= 0
}

#[inline(always)]
const fn bytes_to_pages(b: u64) -> PFN_COUNT {
    ((b + 4095) >> 12) as PFN_COUNT
}

// ADDRESS_AND_SIZE_TO_SPAN_PAGES — macro from sdk/include/ntdef.h.
#[inline(always)]
unsafe fn ADDRESS_AND_SIZE_TO_SPAN_PAGES(va: usize, size: usize) -> PFN_COUNT {
    let base = va & 0xFFF;
    bytes_to_pages((base + size) as u64)
}

// CHAR == i8.
#[allow(non_camel_case_types)]
type CHAR = i8;

const _ACPI_LAYOUT_CHECKS_AMD64: () = {
    assert!(core::mem::size_of::<FADT>() == 0x114);
    assert!(core::mem::offset_of!(FADT, Header) == 0x00);
    assert!(core::mem::offset_of!(FADT, dsdt) == 0x28);
    assert!(core::mem::offset_of!(FADT, flags) == 0x70);
    assert!(core::mem::offset_of!(FADT, x_dsdt) == 0x8c);

    assert!(core::mem::size_of::<RSDT>() == 0x28);
    assert!(core::mem::offset_of!(RSDT, Header) == 0x00);
    assert!(core::mem::offset_of!(RSDT, Tables) == 0x24);

    assert!(core::mem::size_of::<XSDT>() == 0x2c);
    assert!(core::mem::offset_of!(XSDT, Header) == 0x00);
    assert!(core::mem::offset_of!(XSDT, Tables) == 0x24);

    assert!(core::mem::size_of::<ACPI_E820_ENTRY>() == 0x18);
    assert!(core::mem::offset_of!(ACPI_E820_ENTRY, Base) == 0x00);
    assert!(core::mem::offset_of!(ACPI_E820_ENTRY, Length) == 0x08);
    assert!(core::mem::offset_of!(ACPI_E820_ENTRY, Type) == 0x10);

    assert!(core::mem::size_of::<ACPI_BIOS_MULTI_NODE>() == 0x28);
    assert!(core::mem::offset_of!(ACPI_BIOS_MULTI_NODE, RsdtAddress) == 0x00);
    assert!(core::mem::offset_of!(ACPI_BIOS_MULTI_NODE, Count) == 0x08);
    assert!(core::mem::offset_of!(ACPI_BIOS_MULTI_NODE, E820Entry) == 0x10);

    // ACPI_CACHED_TABLE is HAL-internal: { Links: LIST_ENTRY, Header: ptr,
    // ... } so its size/Header offset are pointer-width-dependent (amd64
    // LIST_ENTRY=16 → Header@0x10, size 0x38; i386 LIST_ENTRY=8 →
    // Header@0x08, smaller). The amd64 literals are gated to x86_64; the
    // authoritative per-arch layout check is ntkrnl-sys size_check_i386.rs
    // (verify_sizes.py, cross-compiled from C). Links is field 0 on both.
    assert!(core::mem::offset_of!(ACPI_CACHED_TABLE, Links) == 0x00);
    #[cfg(target_arch = "x86_64")]
    assert!(core::mem::size_of::<ACPI_CACHED_TABLE>() == 0x38);
    #[cfg(target_arch = "x86_64")]
    assert!(core::mem::offset_of!(ACPI_CACHED_TABLE, Header) == 0x10);
};

// ---------------------------------------------------------------------------
// Globals — exported with the same C ABI / mangled names.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpAcpiTableCacheList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[unsafe(no_mangle)]
pub static mut HalpAcpiTableCacheLock: FAST_MUTEX = unsafe { core::mem::zeroed() };

#[unsafe(no_mangle)]
pub static mut HalpProcessedACPIPhase0: BOOLEAN = 0;

#[unsafe(no_mangle)]
pub static mut HalpPhysicalMemoryMayAppearAbove4GB: BOOLEAN = 0;

#[unsafe(no_mangle)]
pub static mut HalpFixedAcpiDescTable: FADT = unsafe { core::mem::zeroed() };

#[unsafe(no_mangle)]
pub static mut HalpDebugPortTable: PDEBUG_PORT_TABLE = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpAcpiSrat: PACPI_SRAT = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpSimpleBootFlagTable: PBOOT_TABLE = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpMaxHotPlugMemoryAddress: PHYSICAL_ADDRESS = PHYSICAL_ADDRESS { QuadPart: 0 };

#[unsafe(no_mangle)]
pub static mut HalpLowStubPhysicalAddress: PHYSICAL_ADDRESS = PHYSICAL_ADDRESS { QuadPart: 0 };

#[unsafe(no_mangle)]
pub static mut HalpPteForFlush: PHARDWARE_PTE = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpVirtAddrForFlush: PVOID = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpLowStub: PVOID = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpAcpiMultiNode: PACPI_BIOS_MULTI_NODE = core::ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpAcpiTableMatchList: LIST_ENTRY = LIST_ENTRY {
    Flink: core::ptr::null_mut(),
    Blink: core::ptr::null_mut(),
};

#[unsafe(no_mangle)]
pub static mut HalpInvalidAcpiTable: ULONG = 0;

#[unsafe(no_mangle)]
pub static mut HalpPicVectorRedirect: [ULONG; 16] =
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];

#[unsafe(no_mangle)]
pub static mut HalDisableFirmwareMapper: BOOLEAN = 1; // TRUE

// C exports PWCHAR variables here, not the string arrays themselves.
// Keep the UTF-16 storage private and export pointer-sized globals.
static HW_ID_STRING: [u16; 11] = [
    b'a' as u16,
    b'c' as u16,
    b'p' as u16,
    b'i' as u16,
    b'p' as u16,
    b'i' as u16,
    b'c' as u16,
    b'_' as u16,
    b'u' as u16,
    b'p' as u16,
    0,
];

static HAL_NAME_STRING: [u16; 29] = [
    b'A' as u16,
    b'C' as u16,
    b'P' as u16,
    b'I' as u16,
    b' ' as u16,
    b'C' as u16,
    b'o' as u16,
    b'm' as u16,
    b'p' as u16,
    b'a' as u16,
    b't' as u16,
    b'i' as u16,
    b'b' as u16,
    b'l' as u16,
    b'e' as u16,
    b' ' as u16,
    b'E' as u16,
    b'i' as u16,
    b's' as u16,
    b'a' as u16,
    b'/' as u16,
    b'I' as u16,
    b's' as u16,
    b'a' as u16,
    b' ' as u16,
    b'H' as u16,
    b'A' as u16,
    b'L' as u16,
    0,
];

#[unsafe(no_mangle)]
pub static mut HalHardwareIdString: PWCHAR = HW_ID_STRING.as_ptr() as PWCHAR;

#[unsafe(no_mangle)]
pub static mut HalName: PWCHAR = HAL_NAME_STRING.as_ptr() as PWCHAR;

// ---------------------------------------------------------------------------
// Externs from sibling HAL files.
// ---------------------------------------------------------------------------

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiGetCachedTable(Signature: ULONG) -> PDESCRIPTION_HEADER {
    let list_head: PLIST_ENTRY;
    let mut next_entry: PLIST_ENTRY;
    let mut cached_table: PACPI_CACHED_TABLE;

    list_head = &raw mut HalpAcpiTableCacheList;
    next_entry = (*list_head).Flink;
    while next_entry != list_head {
        cached_table = ((next_entry as ULONG_PTR)
            - core::mem::offset_of!(ACPI_CACHED_TABLE, Links) as ULONG_PTR)
            as PACPI_CACHED_TABLE;

        if (*cached_table).Header.Signature == Signature {
            return &raw mut (*cached_table).Header;
        }

        next_entry = (*next_entry).Flink;
    }

    core::ptr::null_mut()
}

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiCacheTable(TableHeader: PDESCRIPTION_HEADER) {
    let CachedTable: PACPI_CACHED_TABLE = ((TableHeader as usize)
        - core::mem::offset_of!(ACPI_CACHED_TABLE, Header))
        as PACPI_CACHED_TABLE;

    unsafe {
        InsertTailList(
            &raw mut HalpAcpiTableCacheList,
            &raw mut (*CachedTable).Links,
        );
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiCopyBiosTable(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    TableHeader: PDESCRIPTION_HEADER,
) -> PVOID {
    let mut Size: ULONG;
    let mut PageCount: PFN_COUNT;
    let mut PhysAddress: PHYSICAL_ADDRESS;
    let mut CachedTable: PACPI_CACHED_TABLE;
    let CopiedTable: PDESCRIPTION_HEADER;

    Size = (*TableHeader).Length + core::mem::offset_of!(ACPI_CACHED_TABLE, Header) as ULONG;
    if !LoaderBlock.is_null() {
        PageCount = ((Size + 4096 - 1) >> 12) as PFN_COUNT;
        PhysAddress = core::mem::zeroed();
        PhysAddress.QuadPart =
            HalpAllocPhysicalMemory(LoaderBlock, 0x1000000, PageCount as _, 0) as i64;
        if PhysAddress.QuadPart != 0 {
            CachedTable = HalpMapPhysicalMemory64(PhysAddress, PageCount) as _;
        } else {
            CachedTable = core::ptr::null_mut();
        }
    } else {
        CachedTable = ExAllocatePoolWithTag(NonPagedPool, Size as _, TAG_HAL) as _;
    }

    if !CachedTable.is_null() {
        CopiedTable = core::ptr::addr_of_mut!((*CachedTable).Header);
        core::ptr::copy_nonoverlapping(
            TableHeader as *const u8,
            CopiedTable as *mut u8,
            (*TableHeader).Length as usize,
        );
    } else {
        CopiedTable = core::ptr::null_mut();
    }

    CopiedTable as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiGetTableFromBios(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    Signature: ULONG,
) -> PVOID {
    let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let mut Xsdt: PXSDT;
    let mut Rsdt: PRSDT;
    let Fadt: PFADT;
    let mut Header: PDESCRIPTION_HEADER = core::ptr::null_mut();
    let mut TableLength: ULONG;
    let mut CheckSum: CHAR = 0;
    let mut Offset: ULONG;
    let EntryCount: ULONG;
    let mut CurrentEntry: ULONG;
    let mut CurrentByte: PCHAR;
    let PageCount: PFN_COUNT;

    // Should not query the RSDT/XSDT by itself
    if (Signature == RSDT_SIGNATURE) || (Signature == XSDT_SIGNATURE) {
        return core::ptr::null_mut();
    }

    // Special case request for DSDT, because the FADT points to it
    if Signature == DSDT_SIGNATURE {
        // Grab the FADT
        Fadt = unsafe { HalpAcpiGetTable(LoaderBlock, FADT_SIGNATURE) as PFADT };
        if !Fadt.is_null() {
            // Grab the DSDT address and assume 2 pages
            PhysicalAddress.__bindgen_anon_1.HighPart = 0;
            PhysicalAddress.__bindgen_anon_1.LowPart = unsafe { (*Fadt).dsdt };
            TableLength = 2 * PAGE_SIZE;

            // Map it
            if !LoaderBlock.is_null() {
                // Phase 0, use HAL heap
                Header = unsafe {
                    HalpMapPhysicalMemory64(PhysicalAddress, 2u32) as PDESCRIPTION_HEADER
                };
            } else {
                // Phase 1, use Mm
                Header = unsafe {
                    MmMapIoSpace(PhysicalAddress, 2 * PAGE_SIZE as u64, 0) as PDESCRIPTION_HEADER
                };
            }

            // Fail if we couldn't map it
            if Header.is_null() {
                return core::ptr::null_mut();
            }

            // Validate the signature
            if unsafe { (*Header).Signature } != DSDT_SIGNATURE {
                // Fail and unmap
                if !LoaderBlock.is_null() {
                    // Using HAL heap
                    unsafe { HalpUnmapVirtualAddress(Header as PVOID, 2) };
                } else {
                    // Using Mm
                    unsafe { MmUnmapIoSpace(Header as PVOID, 2 * PAGE_SIZE as u64) };
                }

                // Didn't find anything
                return core::ptr::null_mut();
            }
        } else {
            // Couldn't find it
            return core::ptr::null_mut();
        }
    } else {
        // To find tables, we need the RSDT
        Rsdt = unsafe { HalpAcpiGetTable(LoaderBlock, RSDT_SIGNATURE) as PRSDT };
        if !Rsdt.is_null() {
            // Won't be using the XSDT
            Xsdt = core::ptr::null_mut();
        } else {
            // Only other choice is to use the XSDT
            Xsdt = unsafe { HalpAcpiGetTable(LoaderBlock, XSDT_SIGNATURE) as PXSDT };
            if Xsdt.is_null() {
                return core::ptr::null_mut();
            }

            // Won't be using the RSDT
            Rsdt = core::ptr::null_mut();
        }

        // Smallest RSDT/XSDT is one without table entries
        Offset = core::mem::offset_of!(RSDT, Tables) as ULONG;
        if !Xsdt.is_null() {
            // Figure out total size of table and the offset
            TableLength = unsafe { (*Xsdt).Header.Length };
            if TableLength < Offset {
                Offset = unsafe { (*Xsdt).Header.Length };
            }

            // The entries are each 64-bits, so count them
            EntryCount = (TableLength - Offset) / core::mem::size_of::<PHYSICAL_ADDRESS>() as ULONG;
        } else {
            // Figure out total size of table and the offset
            TableLength = unsafe { (*Rsdt).Header.Length };
            if TableLength < Offset {
                Offset = unsafe { (*Rsdt).Header.Length };
            }

            // The entries are each 32-bits, so count them
            EntryCount = (TableLength - Offset) / core::mem::size_of::<ULONG>() as ULONG;
        }

        // Start at the beginning of the array and loop it
        CurrentEntry = 0;
        while CurrentEntry < EntryCount {
            // Are we using the XSDT?
            // Tables is bindgen's 1-element placeholder for the flexible
            // array — must use pointer arithmetic via addr_of! (RSDT/XSDT
            // are #[repr(C, packed)] so we can't take refs to fields).
            if Xsdt.is_null() {
                // Read the 32-bit physical address
                let tables_ptr = unsafe { core::ptr::addr_of!((*Rsdt).Tables) as *const ULONG };
                PhysicalAddress.__bindgen_anon_1.LowPart =
                    unsafe { tables_ptr.add(CurrentEntry as usize).read_unaligned() };
                PhysicalAddress.__bindgen_anon_1.HighPart = 0;
            } else {
                // Read the 64-bit physical address
                let tables_ptr =
                    unsafe { core::ptr::addr_of!((*Xsdt).Tables) as *const PHYSICAL_ADDRESS };
                PhysicalAddress = unsafe { tables_ptr.add(CurrentEntry as usize).read_unaligned() };
            }

            // Had we already mapped a table?
            if !Header.is_null() {
                // Yes, unmap it
                if !LoaderBlock.is_null() {
                    // Using HAL heap
                    unsafe { HalpUnmapVirtualAddress(Header as PVOID, 2) };
                } else {
                    // Using Mm
                    unsafe { MmUnmapIoSpace(Header as PVOID, 2 * PAGE_SIZE as u64) };
                }
            }

            // Now map this table
            if LoaderBlock.is_null() {
                // Phase 1: Use Mm
                Header = unsafe {
                    MmMapIoSpace(PhysicalAddress, 2 * PAGE_SIZE as u64, MmNonCached)
                        as PDESCRIPTION_HEADER
                };
            } else {
                // Phase 0: Use HAL heap
                Header =
                    unsafe { HalpMapPhysicalMemory64(PhysicalAddress, 2) as PDESCRIPTION_HEADER };
            }

            // Check if we mapped it
            if Header.is_null() {
                // Game over
                return core::ptr::null_mut();
            }

            // We found it, break out
            if unsafe { (*Header).Signature } == Signature {
                break;
            }

            CurrentEntry += 1;
        }

        // Did we end up here back at the last entry?
        if CurrentEntry == EntryCount {
            // Yes, unmap the last table we processed
            if !LoaderBlock.is_null() {
                // Using HAL heap
                unsafe { HalpUnmapVirtualAddress(Header as PVOID, 2) };
            } else {
                // Using Mm
                unsafe { MmUnmapIoSpace(Header as PVOID, 2 * PAGE_SIZE as u64) };
            }

            // Didn't find anything
            return core::ptr::null_mut();
        }
    }

    // Past this point, we assume something was found

    // How many pages do we need?
    let PageCount = bytes_to_pages(unsafe { (*Header).Length as u64 });
    if PageCount != 2 {
        // We assumed two, but this is not the case, free the current mapping
        if !LoaderBlock.is_null() {
            // Using HAL heap
            unsafe { HalpUnmapVirtualAddress(Header as PVOID, 2) };
        } else {
            // Using Mm
            unsafe { MmUnmapIoSpace(Header as PVOID, 2 * PAGE_SIZE as u64) };
        }

        // Now map this table using its correct size
        if LoaderBlock.is_null() {
            // Phase 1: Use Mm
            Header = unsafe {
                MmMapIoSpace(
                    PhysicalAddress,
                    (PageCount as u64) << PAGE_SHIFT,
                    MmNonCached,
                ) as PDESCRIPTION_HEADER
            };
        } else {
            // Phase 0: Use HAL heap
            Header = unsafe {
                HalpMapPhysicalMemory64(PhysicalAddress, PageCount) as PDESCRIPTION_HEADER
            };
        }
    }

    // Fail if the remapped failed
    if Header.is_null() {
        return core::ptr::null_mut();
    }

    // All tables in ACPI 3.0 other than the FACP should have correct checksum
    if (unsafe { (*Header).Signature } != FADT_SIGNATURE) || (unsafe { (*Header).Revision } > 2) {
        // Go to the end of the table
        CheckSum = 0;
        CurrentByte = (Header as PCHAR).add(unsafe { (*Header).Length } as usize);
        while CurrentByte != Header as PCHAR {
            CurrentByte = CurrentByte.sub(1);
            // Add this byte
            CheckSum = CheckSum.wrapping_add(unsafe { *CurrentByte });
        }

        // The correct checksum is always 0, anything else is illegal
        if CheckSum != 0 {
            unsafe {
                HalpInvalidAcpiTable = (*Header).Signature;
            }
        }
    }

    // Return the table
    Header as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiGetTable(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    Signature: ULONG,
) -> PVOID {
    let mut PageCount: PFN_COUNT;
    let mut TableAddress: PDESCRIPTION_HEADER;
    let BiosCopy: PDESCRIPTION_HEADER;

    TableAddress = HalpAcpiGetCachedTable(Signature);
    if TableAddress.is_null() {
        TableAddress = HalpAcpiGetTableFromBios(LoaderBlock, Signature) as _;
        if !TableAddress.is_null() {
            BiosCopy = HalpAcpiCopyBiosTable(LoaderBlock, TableAddress) as _;

            PageCount = (((*TableAddress).Length + 4096 - 1) >> 12) as PFN_COUNT;
            if !LoaderBlock.is_null() {
                HalpUnmapVirtualAddress(TableAddress as PVOID, PageCount);
            } else {
                MmUnmapIoSpace(TableAddress as PVOID, (PageCount as u64) << PAGE_SHIFT);
            }

            TableAddress = BiosCopy;
            if !BiosCopy.is_null() {
                HalpAcpiCacheTable(BiosCopy as _);
            }
        }
    }

    TableAddress as PVOID
}

#[no_mangle]
pub unsafe extern "system" fn HalAcpiGetTable(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    Signature: ULONG,
) -> PVOID {
    let TableHeader: PDESCRIPTION_HEADER;

    /* Is this phase0 */
    if !LoaderBlock.is_null() {
        /* Initialize the cache first */
        if !nt_success(HalpAcpiTableCacheInit(LoaderBlock)) {
            return core::ptr::null_mut();
        }
    } else {
        /* Lock the cache */
        ExAcquireFastMutex(&raw mut HalpAcpiTableCacheLock);
    }

    /* Get the table */
    TableHeader = HalpAcpiGetTable(LoaderBlock, Signature) as _;

    /* Release the lock in phase 1 */
    if LoaderBlock.is_null() {
        ExReleaseFastMutex(&raw mut HalpAcpiTableCacheLock);
    }

    /* Return the table */
    return TableHeader as PVOID;
}

#[no_mangle]
pub unsafe extern "system" fn HalpNumaInitializeStaticConfiguration(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    let SratTable: PACPI_SRAT;

    /* Get the SRAT, bail out if it doesn't exist */
    SratTable = HalAcpiGetTable(LoaderBlock, SRAT_SIGNATURE) as _;
    HalpAcpiSrat = SratTable;
    if SratTable.is_null() {
        return;
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetHotPlugMemoryInfo(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let SratTable: PACPI_SRAT;

    /* Get the SRAT, bail out if it doesn't exist */
    SratTable = HalAcpiGetTable(LoaderBlock, SRAT_SIGNATURE) as _;
    HalpAcpiSrat = SratTable;
    if SratTable.is_null() {
        return;
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpDynamicSystemResourceConfiguration(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) {
    /* For this HAL, it means to get hot plug memory information */
    HalpGetHotPlugMemoryInfo(LoaderBlock);
}

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiDetectMachineSpecificActions(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    DescriptionTable: PFADT,
) {
    let _ = LoaderBlock;
    let _ = DescriptionTable;

    if !HalpAcpiTableMatchList.Flink.is_null() {
        // DPRINT1 omitted
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpInitBootTable(LoaderBlock: PLOADER_PARAMETER_BLOCK) {
    let BootTable: PBOOT_TABLE;

    /* Get the boot table */
    BootTable = HalAcpiGetTable(LoaderBlock, BOOT_SIGNATURE) as PBOOT_TABLE;
    HalpSimpleBootFlagTable = BootTable;

    /* Validate it */
    if !BootTable.is_null()
        && (*BootTable).Header.Length >= core::mem::size_of::<BOOT_TABLE>() as ULONG
        && (*BootTable).CMOSIndex >= 9
    {
        /* ACPI Boot table found, but not supported */
    } else {
        /* Invalid or doesn't exist, ignore it */
        HalpSimpleBootFlagTable = core::ptr::null_mut();
    }

    /* Install the end of boot handler */
    // HalEndOfBoot = HalpEndOfBoot;
}

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiFindRsdtPhase0(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
    AcpiMultiNode: *mut PACPI_BIOS_MULTI_NODE,
) -> NTSTATUS {
    unsafe extern "C" {
        fn _stricmp(s1: *const i8, s2: *const i8) -> i32;
    }

    let mut Next: PCONFIGURATION_COMPONENT_DATA = core::ptr::null_mut();
    let mut ComponentEntry: PCONFIGURATION_COMPONENT_DATA;
    let ResourceList: PCM_PARTIAL_RESOURCE_LIST;
    let NodeData: PACPI_BIOS_MULTI_NODE;
    let NodeLength: usize;
    let PageCount: PFN_COUNT;
    let mut MappedAddress: PVOID;
    let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();

    if !HalpAcpiMultiNode.is_null() {
        *AcpiMultiNode = HalpAcpiMultiNode;
        return STATUS_SUCCESS;
    }

    *AcpiMultiNode = core::ptr::null_mut();

    ComponentEntry = KeFindConfigurationNextEntry(
        (*LoaderBlock).ConfigurationRoot as *mut _,
        AdapterClass as u32,
        MultiFunctionAdapter as u32,
        core::ptr::null_mut(),
        &raw mut Next as *mut _,
    ) as *mut _;

    while !ComponentEntry.is_null() {
        if _stricmp(
            (*ComponentEntry).ComponentEntry.Identifier,
            b"ACPI BIOS\0".as_ptr() as *const i8,
        ) == 0
        {
            break;
        }

        Next = ComponentEntry;
        ComponentEntry = KeFindConfigurationNextEntry(
            (*LoaderBlock).ConfigurationRoot as *mut _,
            AdapterClass as u32,
            MultiFunctionAdapter as u32,
            core::ptr::null_mut(),
            &raw mut Next as *mut _,
        ) as *mut _;
    }

    if ComponentEntry.is_null() {
        return STATUS_NOT_FOUND;
    }

    ResourceList = (*ComponentEntry).ConfigurationData as PCM_PARTIAL_RESOURCE_LIST;
    NodeData = ResourceList.add(1) as PACPI_BIOS_MULTI_NODE;

    // `NodeData` lands immediately after a CM_PARTIAL_RESOURCE_LIST inside the
    // loader's configuration data, so it is only 4-byte aligned in practice,
    // while ACPI_BIOS_MULTI_NODE wants 8 (it leads with a PHYSICAL_ADDRESS).
    // A plain `(*NodeData).Count` dereferences the struct pointer and therefore
    // asserts the STRUCT's alignment, not the field's — which is UB here, and
    // rustc's misaligned-pointer check (emitted only under debug-assertions)
    // caught it the first time this crate was built with them on:
    //   "misaligned pointer dereference: address must be a multiple of 0x8 but
    //    is 0xfffff8000010232c"
    // It happens to work on x86, which is why it survived; it is still UB, and
    // `addr_of!` + `read_unaligned` is the correct way to read a field out of a
    // packed/underaligned firmware structure. The byte-wise copy below is
    // alignment-1 and was always fine.
    let node_count = core::ptr::read_unaligned(core::ptr::addr_of!((*NodeData).Count));
    NodeLength = core::mem::size_of::<ACPI_BIOS_MULTI_NODE>()
        // The UEFI loader publishes Count = 0 (no E820 map). In C, `Count - 1` wraps and the
        // multiply comes out at -24, so it copies 16 bytes; in Rust that is an overflow panic
        // under debug-assertions. Divergence: saturate, copying the whole 40-byte node, which
        // is exactly the loader's TableSize, so still in bounds.
        + (node_count as usize).saturating_sub(1) * core::mem::size_of::<ACPI_E820_ENTRY>();

    PageCount = bytes_to_pages(NodeLength as u64);

    PhysicalAddress.QuadPart =
        HalpAllocPhysicalMemory(LoaderBlock, 0x1000000, PageCount as _, 0) as i64;

    if PhysicalAddress.QuadPart != 0 {
        MappedAddress = HalpMapPhysicalMemory64(PhysicalAddress, PageCount);
    } else {
        MappedAddress = core::ptr::null_mut();
    }

    HalpAcpiMultiNode = MappedAddress as _;
    if MappedAddress.is_null() {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    core::ptr::copy_nonoverlapping(NodeData as *const u8, MappedAddress as *mut u8, NodeLength);

    *AcpiMultiNode = HalpAcpiMultiNode;
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiTableCacheInit(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let mut AcpiMultiNode: PACPI_BIOS_MULTI_NODE = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_SUCCESS;
    let mut PhysicalAddress: PHYSICAL_ADDRESS;
    let mut MappedAddress: PVOID;
    let mut TableLength: ULONG;
    let mut Rsdt: PRSDT;
    let LoaderExtension: PLOADER_PARAMETER_EXTENSION;

    unsafe {
        if !(*(&raw mut HalpAcpiTableCacheList)).Flink.is_null() {
            return Status;
        }

        ExInitializeFastMutex(&raw mut HalpAcpiTableCacheLock);
        InitializeListHead(&raw mut HalpAcpiTableCacheList);

        Status = HalpAcpiFindRsdtPhase0(LoaderBlock, &raw mut AcpiMultiNode);
        if !nt_success(Status) {
            return Status;
        }

        PhysicalAddress = core::mem::zeroed();
        PhysicalAddress.QuadPart = (*AcpiMultiNode).RsdtAddress.QuadPart;

        if !LoaderBlock.is_null() {
            MappedAddress = HalpMapPhysicalMemory64(PhysicalAddress, 2);
        } else {
            MappedAddress = MmMapIoSpace(PhysicalAddress, (PAGE_SIZE * 2) as u64, MmNonCached as _);
        }

        Rsdt = MappedAddress as PRSDT;
        if MappedAddress.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        if ((*Rsdt).Header.Signature != RSDT_SIGNATURE)
            && ((*Rsdt).Header.Signature != XSDT_SIGNATURE)
        {
            HalDisplayString(b"Bad RSDT pointer\r\n\0".as_ptr() as *mut _);
            KeBugCheckEx(MISMATCHED_HAL, 4, line!() as ULONG_PTR, 0, 0);
        }

        TableLength = ADDRESS_AND_SIZE_TO_SPAN_PAGES(
            PhysicalAddress.__bindgen_anon_1.LowPart as usize,
            (*Rsdt).Header.Length as usize,
        );
        if TableLength != 2 {
            if LoaderBlock.is_null() {
                MmUnmapIoSpace(MappedAddress, 2 * PAGE_SIZE as u64);
                MappedAddress = MmMapIoSpace(
                    PhysicalAddress,
                    (TableLength as u64) << PAGE_SHIFT,
                    MmNonCached as _,
                );
            } else {
                HalpUnmapVirtualAddress(MappedAddress, 2);
                MappedAddress = HalpMapPhysicalMemory64(PhysicalAddress, TableLength);
            }

            Rsdt = MappedAddress as PRSDT;
            if MappedAddress.is_null() {
                return STATUS_INSUFFICIENT_RESOURCES;
            }
        }

        Rsdt = HalpAcpiCopyBiosTable(LoaderBlock, &raw mut (*Rsdt).Header) as PRSDT;
        if Rsdt.is_null() {
            return STATUS_INSUFFICIENT_RESOURCES;
        }

        if !LoaderBlock.is_null() {
            HalpUnmapVirtualAddress(MappedAddress, TableLength);
            LoaderExtension = (*LoaderBlock).Extension;
        } else {
            MmUnmapIoSpace(MappedAddress, (TableLength as u64) << PAGE_SHIFT);
            LoaderExtension = core::ptr::null_mut();
        }

        HalpAcpiCacheTable(&raw mut (*Rsdt).Header);

        if !LoaderExtension.is_null() && (*LoaderExtension).Size >= 0x58 {
            if !(*LoaderExtension).AcpiTable.is_null() && (*LoaderExtension).AcpiTableSize != 0 {
                // ACPI Table Overrides Not Supported
            }
        }

        Status
    }
}

#[no_mangle]
pub unsafe extern "system" fn HaliAcpiTimerInit(TimerPort: ULONG, TimerValExt: ULONG) {
    let mut timer_port = TimerPort;
    let mut timer_val_ext = TimerValExt;

    if timer_port == 0 {
        timer_port = (*core::ptr::addr_of!(HalpFixedAcpiDescTable)).pm_tmr_blk_io_port;
        timer_val_ext =
            if (*core::ptr::addr_of!(HalpFixedAcpiDescTable)).flags & ACPI_TMR_VAL_EXT != 0 {
                1
            } else {
                0
            };
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpSetupAcpiPhase0(
    LoaderBlock: PLOADER_PARAMETER_BLOCK,
) -> NTSTATUS {
    let mut Status: NTSTATUS;
    let Fadt: PFADT;
    let mut TableLength: ULONG;
    let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();

    /* Only do this once */
    if HalpProcessedACPIPhase0 != 0 {
        return STATUS_SUCCESS;
    }

    /* Setup the ACPI table cache */
    Status = HalpAcpiTableCacheInit(LoaderBlock);
    if !nt_success(Status) {
        return Status;
    }

    /* Grab the FADT */
    Fadt = HalAcpiGetTable(LoaderBlock, FADT_SIGNATURE) as PFADT;
    if Fadt.is_null() {
        /* Fail */
        return STATUS_NOT_FOUND;
    }

    /* Assume typical size, otherwise whatever the descriptor table says */
    TableLength = core::mem::size_of::<FADT>() as ULONG;
    if (*Fadt).Header.Length < core::mem::size_of::<FADT>() as ULONG {
        TableLength = (*Fadt).Header.Length;
    }

    /* Copy it in the HAL static buffer */
    core::ptr::copy_nonoverlapping(
        Fadt as *const u8,
        &raw mut HalpFixedAcpiDescTable as *mut u8,
        TableLength as usize,
    );

    /* Anything special this HAL needs to do? */
    HalpAcpiDetectMachineSpecificActions(LoaderBlock, &raw mut HalpFixedAcpiDescTable);

    /* Get the debug table for KD */
    HalpDebugPortTable = HalAcpiGetTable(LoaderBlock, DBGP_SIGNATURE) as _;

    /* Initialize NUMA through the SRAT */
    HalpNumaInitializeStaticConfiguration(LoaderBlock);

    /* Initialize hotplug through the SRAT */
    HalpDynamicSystemResourceConfiguration(LoaderBlock);
    if !HalpAcpiSrat.is_null() {
        // DPRINT1 omitted
    }

    /* Can there be memory higher than 4GB? */
    if (*(&raw mut HalpMaxHotPlugMemoryAddress))
        .__bindgen_anon_1
        .HighPart
        >= 1
    {
        /* We'll need this for DMA later */
        HalpPhysicalMemoryMayAppearAbove4GB = 1;
    }

    /* Setup the ACPI timer */
    HaliAcpiTimerInit(0, 0);

    /* Do we have a low stub address yet? */
    if (*(&raw mut HalpLowStubPhysicalAddress)).QuadPart == 0 {
        /* Allocate it */
        (*(&raw mut HalpLowStubPhysicalAddress)).QuadPart =
            HalpAllocPhysicalMemory(LoaderBlock, 0x100000, HALP_LOW_STUB_SIZE_IN_PAGES as _, 0)
                as i64;
        if (*(&raw mut HalpLowStubPhysicalAddress)).QuadPart != 0 {
            /* Map it */
            HalpLowStub =
                HalpMapPhysicalMemory64(HalpLowStubPhysicalAddress, HALP_LOW_STUB_SIZE_IN_PAGES);

            /* SMP AP-start page-table scratch — anywhere <4GB (see decl). */
            (*(&raw mut HalpApPtScratchPhysicalAddress)).QuadPart =
                HalpAllocPhysicalMemory(
                    LoaderBlock,
                    0xFFFF_F000,
                    HALP_AP_PT_SCRATCH_PAGES as _,
                    0,
                ) as i64;
            if (*(&raw mut HalpApPtScratchPhysicalAddress)).QuadPart != 0 {
                HalpApPtScratch = HalpMapPhysicalMemory64(
                    HalpApPtScratchPhysicalAddress,
                    HALP_AP_PT_SCRATCH_PAGES,
                );
            }
        } else {
            /* SMP AP start depends on this page (smp_processor.rs). Dump the
             * <2MB loader descriptor map so the failure is diagnosable from
             * serial instead of a bare "low stub bad" at Phase 1. */
            DbgPrint(
                b"HAL: low-stub alloc FAILED (%u pages <1MB); low descriptors:\n\0".as_ptr()
                    as PCSTR,
                HALP_LOW_STUB_SIZE_IN_PAGES,
            );
            let head = &raw mut (*LoaderBlock).MemoryDescriptorListHead;
            let mut e = (*head).Flink;
            while e != head {
                let md = crate::x86::memory::list_entry_to_descriptor(e);
                if (*md).BasePage < 0x200 {
                    DbgPrint(
                        b"HAL:   base=%05x pages=%05x type=%u\n\0".as_ptr() as PCSTR,
                        (*md).BasePage as ULONG,
                        (*md).PageCount as ULONG,
                        (*md).MemoryType as ULONG,
                    );
                }
                e = (*e).Flink;
            }
        }
    }

    /* Grab a page for flushes */
    PhysicalAddress.QuadPart = 0x100000;
    HalpVirtAddrForFlush = HalpMapPhysicalMemory64(PhysicalAddress, 1);
    HalpPteForFlush = HalAddressToPte(HalpVirtAddrForFlush) as PHARDWARE_PTE;

    /* Don't do this again */
    HalpProcessedACPIPhase0 = 1;

    /* Setup the boot table */
    HalpInitBootTable(LoaderBlock);

    /* Log some ACPI data */
    {
        let mut NextEntry: PLIST_ENTRY;
        let mut AcpiVersion: PCSTR = core::ptr::null();

        /* Find the ACPI version (range) out */
        match (*Fadt).Header.Revision {
            0 => {
                AcpiVersion = b"Unknown_0\0".as_ptr() as PCSTR;
            }
            1 => {
                AcpiVersion = b"1.0-1.0b\0".as_ptr() as PCSTR;
            }
            2 => {
                AcpiVersion = b"Unknown_2\0".as_ptr() as PCSTR;
            }
            3 => {
                AcpiVersion = b"1.5-2.0_C\0".as_ptr() as PCSTR;
            }
            4 => {
                AcpiVersion = b"3.0-4.0_A\0".as_ptr() as PCSTR;
            }
            5 => {
                if (*Fadt).minor_revision == 0 {
                    AcpiVersion = b"5.0-5.0_B\0".as_ptr() as PCSTR;
                } else if (*Fadt).minor_revision == 1 {
                    AcpiVersion = b"5.1-5.1_B\0".as_ptr() as PCSTR;
                }
            }
            6 => {
                if (*Fadt).minor_revision == 0 {
                    AcpiVersion = b"6.0-6.0_A\0".as_ptr() as PCSTR;
                } else if (*Fadt).minor_revision == 1 {
                    AcpiVersion = b"6.1-6.1_A\0".as_ptr() as PCSTR;
                } else if (*Fadt).minor_revision == 2 {
                    AcpiVersion = b"6.2-6.2_B\0".as_ptr() as PCSTR;
                } else if (*Fadt).minor_revision == 3 {
                    AcpiVersion = b"6.3-6.3_A\0".as_ptr() as PCSTR;
                } else if ((*Fadt).minor_revision & 0x0F) == 0x04 {
                    if ((*Fadt).minor_revision & 0xF0) == 0x00 {
                        AcpiVersion = b"6.4\0".as_ptr() as PCSTR;
                    } else if ((*Fadt).minor_revision & 0xF0) == 0x10 {
                        AcpiVersion = b"6.4_A\0".as_ptr() as PCSTR;
                    }
                } else if (*Fadt).minor_revision == 5 {
                    AcpiVersion = b"6.5-6.6\0".as_ptr() as PCSTR;
                }
            }
            _ => {}
        }

        /* Print the ACPI version */
        if AcpiVersion.is_null() {
            DbgPrint(
                b"ACPI vUnknown_%u_%u detected. Tables:\0".as_ptr() as PCSTR,
                (*Fadt).Header.Revision as u32,
                (*Fadt).minor_revision as u32,
            );
        } else {
            DbgPrint(
                b"ACPI v%s detected. Tables:\0".as_ptr() as PCSTR,
                AcpiVersion,
            );
        }

        /* List cached tables */
        NextEntry = (*(&raw mut HalpAcpiTableCacheList)).Flink;
        while NextEntry != &raw mut HalpAcpiTableCacheList {
            let CachedTable: PACPI_CACHED_TABLE = ((NextEntry as ULONG_PTR)
                - core::mem::offset_of!(ACPI_CACHED_TABLE, Links) as ULONG_PTR)
                as PACPI_CACHED_TABLE;

            /* Print the table signature */
            DbgPrint(
                b" [%c%c%c%c]\0".as_ptr() as PCSTR,
                ((*CachedTable).Header.Signature & 0x000000FF) as u32,
                (((*CachedTable).Header.Signature & 0x0000FF00) >> 8) as u32,
                (((*CachedTable).Header.Signature & 0x00FF0000) >> 16) as u32,
                (((*CachedTable).Header.Signature & 0xFF000000) >> 24) as u32,
            );

            NextEntry = (*NextEntry).Flink;
        }
        DbgPrint(b"\n\0".as_ptr() as PCSTR);
    }

    /* Return success */
    STATUS_SUCCESS
}

unsafe extern "C" {
    fn DbgPrint(Format: PCSTR, ...) -> ULONG;
}
// HalAddressToPte = MiAddressToPte (FORCEINLINE in halp.h → internal/
// {amd64,i386}/mm.h). Arch-split self-map math, matching the established
// in-tree convention (cf. hal-rust/src/x86/memory.rs + bios.rs):
//   amd64: 4-level walk, PTE_BASE 0xFFFFF68000000000, 8-byte PTEs
//   i386 : recursive self-map, PTE_BASE 0xC0000000, 4-byte (non-PAE) PTEs
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn HalAddressToPte(va: PVOID) -> PVOID {
    const PTE_BASE_AMD64: usize = 0xFFFFF68000000000;
    let offset = ((va as u64) >> 9) & (0xFFFFFFFFFu64 << 3);
    (PTE_BASE_AMD64 + offset as usize) as PVOID
}
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn HalAddressToPte(va: PVOID) -> PVOID {
    const PTE_BASE_I386: usize = 0xC000_0000;
    (PTE_BASE_I386 + (((va as usize) >> 12) << 2)) as PVOID
}

// i386 keeps its own PCI-bus/bus-handler init in bussupp.rs (the i386
// HAL bus infra; busemul.rs is amd64-only — see its module doc). These
// amd64-ACPI variants are gated to x86_64 so bussupp's #[no_mangle]
// definitions win the link on i386. ACPI table provision + ACPI_HAL PnP
// (halacpi/halpnpdd) still run; only the bus-handler registration stays
// the established i386 path. Task #4.
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn HalpInitializePciBus() {
    unsafe {
        HalpInitializePciStubs();
        HalpGetNMICrashFlag();
    }
}

// Only called by the x86_64-gated HalpInitBusHandlers; on i386 the
// bus-handler dispatch is owned by bussupp.rs. Gating this out on i386
// also drops its references to HalpTranslateBusAddress /
// HalpFindBusAddressTranslation (amd64 busemul symbols absent on i386).
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn HalpInitNonBusHandler() {
    use ntkrnl_sys::hal_ddk::HalPrivateDispatchTable;
    let dispatch = &raw mut HalPrivateDispatchTable;
    (*dispatch).HalPciTranslateBusAddress =
        Some(core::mem::transmute(HalpTranslateBusAddress as usize));
    (*dispatch).HalPciAssignSlotResources =
        Some(core::mem::transmute(HalpAssignSlotResources as usize));
    (*dispatch).HalFindBusAddressTranslation =
        Some(core::mem::transmute(HalpFindBusAddressTranslation as usize));
}

#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub unsafe extern "system" fn HalpInitBusHandlers() {
    unsafe {
        HalpInitNonBusHandler();
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpBuildAddressMap() {
    /* ACPI is magic baby */
}

#[no_mangle]
pub unsafe extern "system" fn HalpGetDebugPortTable() -> BOOLEAN {
    unsafe {
        (!HalpDebugPortTable.is_null() && (*HalpDebugPortTable).BaseAddress.AddressSpaceID == 1)
            as BOOLEAN
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpIs16BitPortDecodeSupported() -> ULONG {
    CM_RESOURCE_PORT_16_BIT_DECODE
}

#[no_mangle]
pub unsafe extern "system" fn HalpAcpiDetectResourceListSize(ListSize: *mut ULONG) {
    let _ = ListSize;

    unsafe {
        *ListSize = if (*core::ptr::addr_of!(HalpFixedAcpiDescTable)).sci_int_vector != 0 {
            1
        } else {
            0
        };
    }
}

#[no_mangle]
pub unsafe extern "system" fn HalpBuildAcpiResourceList(
    ResourceList: PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    let Interrupt: ULONG;

    // Initialize the list
    (*ResourceList).BusNumber = -1i32 as ULONG;
    (*ResourceList).AlternativeLists = 1;
    // The ACPI SCI interrupt is reported under this InterfaceType, then
    // translated via the matching HAL bus handler. amd64's ACPI HAL
    // (busemul) registers a PNPBus handler. The i386 backport keeps the
    // legacy bussupp handlers, which register only Internal(0)/Isa(1) —
    // there is NO PNPBus(15) handler, so HalReferenceHandlerForBus(PNPBus)
    // returns null → HalGetInterruptVector → 0 → IopTranslateDeviceResources
    // STATUS_UNSUCCESSFUL → ACPI_HAL stuck at DriversAdded. The SCI is a
    // fixed motherboard PIC IRQ, so the `Internal` handler (identity PIC
    // translation) is correct on i386. Task #4.
    #[cfg(target_arch = "x86_64")]
    {
        (*ResourceList).InterfaceType = PNPBus;
    }
    #[cfg(target_arch = "x86")]
    {
        (*ResourceList).InterfaceType = _INTERFACE_TYPE_Internal as i32;
    }
    (*ResourceList).List[0].Version = 1;
    (*ResourceList).List[0].Revision = 1;
    (*ResourceList).List[0].Count = 0;

    // Is there a SCI?
    if HalpFixedAcpiDescTable.sci_int_vector != 0 {
        // Fill out the entry for it
        (*ResourceList).List[0].Descriptors[0].Flags =
            CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE as USHORT;
        (*ResourceList).List[0].Descriptors[0].Type = CmResourceTypeInterrupt as u8;
        (*ResourceList).List[0].Descriptors[0].ShareDisposition = CmResourceShareShared as u8;

        // Get the interrupt number
        Interrupt = HalpPicVectorRedirect[HalpFixedAcpiDescTable.sci_int_vector as usize];
        (*ResourceList).List[0].Descriptors[0]
            .u
            .Interrupt
            .MinimumVector = Interrupt;
        (*ResourceList).List[0].Descriptors[0]
            .u
            .Interrupt
            .MaximumVector = Interrupt;

        // One more
        (*ResourceList).List[0].Count += 1;
    }

    // All good
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn HalpQueryAcpiResourceRequirements(
    Requirements: *mut PIO_RESOURCE_REQUIREMENTS_LIST,
) -> NTSTATUS {
    let mut RequirementsList: PIO_RESOURCE_REQUIREMENTS_LIST;
    let mut Count: ULONG = 0;
    let ListSize: usize;
    let mut Status: NTSTATUS;

    HalpAcpiDetectResourceListSize(&raw mut Count);

    ListSize = core::mem::offset_of!(IO_RESOURCE_REQUIREMENTS_LIST, List)
        + core::mem::offset_of!(IO_RESOURCE_LIST, Descriptors)
        + (Count as usize * core::mem::size_of::<IO_RESOURCE_DESCRIPTOR>());

    RequirementsList = ExAllocatePoolWithTag(PagedPool, ListSize as _, TAG_HAL) as _;
    if !RequirementsList.is_null() {
        RtlZeroMemory(RequirementsList as *mut _, ListSize);
        (*RequirementsList).ListSize = ListSize as ULONG;

        Status = HalpBuildAcpiResourceList(RequirementsList);
        if Status >= 0 {
            *Requirements = RequirementsList;
        } else {
            ExFreePoolWithTag(RequirementsList as *mut _, TAG_HAL);
            Status = STATUS_NO_SUCH_DEVICE;
        }
    } else {
        Status = STATUS_INSUFFICIENT_RESOURCES;
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn HalReportResourceUsage() {
    let mut InterfaceType: INTERFACE_TYPE;
    let mut HalString: _UNICODE_STRING = core::mem::zeroed();

    HalpInitializePciBus();

    InterfaceType = match HalpBusType {
        x if x == MACHINE_TYPE_ISA => _INTERFACE_TYPE_Isa,
        x if x == MACHINE_TYPE_EISA => _INTERFACE_TYPE_Eisa,
        x if x == MACHINE_TYPE_MCA => _INTERFACE_TYPE_MicroChannel,
        _ => _INTERFACE_TYPE_Internal,
    };

    RtlInitUnicodeString(
        &raw mut HalString as *mut _,
        core::ptr::read_volatile(&raw const HalName) as _,
    );
    HalpReportResourceUsage(&raw mut HalString as PUNICODE_STRING, InterfaceType);

    HalpRegisterPciDebuggingDeviceInfo();
}

// ============================================================================
// ORIGINAL C SOURCE
// ============================================================================
//
// /*
//  * PROJECT:         ReactOS HAL
//  * LICENSE:         BSD - See COPYING.ARM in the top level directory
//  * FILE:            hal/halx86/acpi/halacpi.c
//  * PURPOSE:         HAL ACPI Code
//  * PROGRAMMERS:     ReactOS Portable Systems Group
//  */
//
// /* INCLUDES *******************************************************************/
//
// #include <hal.h>
// #define NDEBUG
// #include <debug.h>
//
// /* GLOBALS ********************************************************************/
//
// LIST_ENTRY HalpAcpiTableCacheList;
// FAST_MUTEX HalpAcpiTableCacheLock;
//
// BOOLEAN HalpProcessedACPIPhase0;
// BOOLEAN HalpPhysicalMemoryMayAppearAbove4GB;
//
// FADT HalpFixedAcpiDescTable;
// PDEBUG_PORT_TABLE HalpDebugPortTable;
// PACPI_SRAT HalpAcpiSrat;
// PBOOT_TABLE HalpSimpleBootFlagTable;
//
// PHYSICAL_ADDRESS HalpMaxHotPlugMemoryAddress;
// PHYSICAL_ADDRESS HalpLowStubPhysicalAddress;
// PHARDWARE_PTE HalpPteForFlush;
// PVOID HalpVirtAddrForFlush;
// PVOID HalpLowStub;
//
// PACPI_BIOS_MULTI_NODE HalpAcpiMultiNode;
//
// LIST_ENTRY HalpAcpiTableMatchList;
//
// ULONG HalpInvalidAcpiTable;
//
// ULONG HalpPicVectorRedirect[] = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15};
//
// /* This determines the HAL type */
// BOOLEAN HalDisableFirmwareMapper = TRUE;
// PWCHAR HalHardwareIdString = L"acpipic_up";
// PWCHAR HalName = L"ACPI Compatible Eisa/Isa HAL";
//
// /* PRIVATE FUNCTIONS **********************************************************/
//
// PDESCRIPTION_HEADER
// NTAPI
// HalpAcpiGetCachedTable(IN ULONG Signature)
// {
//     PLIST_ENTRY ListHead, NextEntry;
//     PACPI_CACHED_TABLE CachedTable;
//
//     /* Loop cached tables */
//     ListHead = &HalpAcpiTableCacheList;
//     NextEntry = ListHead->Flink;
//     while (NextEntry != ListHead)
//     {
//         /* Get the table */
//         CachedTable = CONTAINING_RECORD(NextEntry, ACPI_CACHED_TABLE, Links);
//
//         /* Compare signatures */
//         if (CachedTable->Header.Signature == Signature) return &CachedTable->Header;
//
//         /* Keep going */
//         NextEntry = NextEntry->Flink;
//     }
//
//     /* Nothing found */
//     return NULL;
// }
//
// VOID
// NTAPI
// HalpAcpiCacheTable(IN PDESCRIPTION_HEADER TableHeader)
// {
//     PACPI_CACHED_TABLE CachedTable;
//
//     /* Get the cached table and link it */
//     CachedTable = CONTAINING_RECORD(TableHeader, ACPI_CACHED_TABLE, Header);
//     InsertTailList(&HalpAcpiTableCacheList, &CachedTable->Links);
// }
//
// PVOID
// NTAPI
// HalpAcpiCopyBiosTable(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                       IN PDESCRIPTION_HEADER TableHeader)
// {
//     ULONG Size;
//     PFN_COUNT PageCount;
//     PHYSICAL_ADDRESS PhysAddress;
//     PACPI_CACHED_TABLE CachedTable;
//     PDESCRIPTION_HEADER CopiedTable;
//
//     /* Size we'll need for the cached table */
//     Size = TableHeader->Length + FIELD_OFFSET(ACPI_CACHED_TABLE, Header);
//     if (LoaderBlock)
//     {
//         /* Phase 0: Convert to pages and use the HAL heap */
//         PageCount = bytes_to_pages(Size);
//         PhysAddress.QuadPart = HalpAllocPhysicalMemory(LoaderBlock,
//                                                        0x1000000,
//                                                        PageCount,
//                                                        FALSE);
//         if (PhysAddress.QuadPart)
//         {
//             /* Map it */
//             CachedTable = HalpMapPhysicalMemory64(PhysAddress, PageCount);
//         }
//         else
//         {
//             /* No memory, so nothing to map */
//             CachedTable = NULL;
//         }
//     }
//     else
//     {
//         /* Use Mm pool */
//         CachedTable = ExAllocatePoolWithTag(NonPagedPool, Size, TAG_HAL);
//     }
//
//     /* Do we have the cached table? */
//     if (CachedTable)
//     {
//         /* Copy the data */
//         CopiedTable = &CachedTable->Header;
//         RtlCopyMemory(CopiedTable, TableHeader, TableHeader->Length);
//     }
//     else
//     {
//         /* Nothing to return */
//         CopiedTable = NULL;
//     }
//
//     /* Return the table */
//     return CopiedTable;
// }
//
// PVOID
// NTAPI
// HalpAcpiGetTableFromBios(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                          IN ULONG Signature)
// {
//     PHYSICAL_ADDRESS PhysicalAddress;
//     PXSDT Xsdt;
//     PRSDT Rsdt;
//     PFADT Fadt;
//     PDESCRIPTION_HEADER Header = NULL;
//     ULONG TableLength;
//     CHAR CheckSum = 0;
//     ULONG Offset;
//     ULONG EntryCount, CurrentEntry;
//     PCHAR CurrentByte;
//     PFN_COUNT PageCount;
//
//     /* Should not query the RSDT/XSDT by itself */
//     if ((Signature == RSDT_SIGNATURE) || (Signature == XSDT_SIGNATURE)) return NULL;
//
//     /* Special case request for DSDT, because the FADT points to it */
//     if (Signature == DSDT_SIGNATURE)
//     {
//         /* Grab the FADT */
//         Fadt = HalpAcpiGetTable(LoaderBlock, FADT_SIGNATURE);
//         if (Fadt)
//         {
//             /* Grab the DSDT address and assume 2 pages */
//             PhysicalAddress.HighPart = 0;
//             PhysicalAddress.LowPart = Fadt->dsdt;
//             TableLength = 2 * PAGE_SIZE;
//
//             /* Map it */
//             if (LoaderBlock)
//             {
//                 /* Phase 0, use HAL heap */
//                 Header = HalpMapPhysicalMemory64(PhysicalAddress, 2u);
//             }
//             else
//             {
//                 /* Phase 1, use Mm */
//                 Header = MmMapIoSpace(PhysicalAddress, 2 * PAGE_SIZE, 0);
//             }
//
//             /* Fail if we couldn't map it */
//             if (!Header)
//             {
//                 DPRINT1("HAL: Failed to map ACPI table.\n");
//                 return NULL;
//             }
//
//             /* Validate the signature */
//             if (Header->Signature != DSDT_SIGNATURE)
//             {
//                 /* Fail and unmap */
//                 if (LoaderBlock)
//                 {
//                     /* Using HAL heap */
//                     HalpUnmapVirtualAddress(Header, 2);
//                 }
//                 else
//                 {
//                     /* Using Mm */
//                     MmUnmapIoSpace(Header, 2 * PAGE_SIZE);
//                 }
//
//                 /* Didn't find anything */
//                 return NULL;
//             }
//         }
//         else
//         {
//             /* Couldn't find it */
//             return NULL;
//         }
//     }
//     else
//     {
//         /* To find tables, we need the RSDT */
//         Rsdt = HalpAcpiGetTable(LoaderBlock, RSDT_SIGNATURE);
//         if (Rsdt)
//         {
//             /* Won't be using the XSDT */
//             Xsdt = NULL;
//         }
//         else
//         {
//             /* Only other choice is to use the XSDT */
//             Xsdt = HalpAcpiGetTable(LoaderBlock, XSDT_SIGNATURE);
//             if (!Xsdt) return NULL;
//
//             /* Won't be using the RSDT */
//             Rsdt = NULL;
//         }
//
//         /* Smallest RSDT/XSDT is one without table entries */
//         Offset = FIELD_OFFSET(RSDT, Tables);
//         if (Xsdt)
//         {
//             /* Figure out total size of table and the offset */
//             TableLength = Xsdt->Header.Length;
//             if (TableLength < Offset) Offset = Xsdt->Header.Length;
//
//             /* The entries are each 64-bits, so count them */
//             EntryCount = (TableLength - Offset) / sizeof(PHYSICAL_ADDRESS);
//         }
//         else
//         {
//             /* Figure out total size of table and the offset */
//             TableLength = Rsdt->Header.Length;
//             if (TableLength < Offset) Offset = Rsdt->Header.Length;
//
//             /* The entries are each 32-bits, so count them */
//             EntryCount = (TableLength - Offset) / sizeof(ULONG);
//         }
//
//         /* Start at the beginning of the array and loop it */
//         for (CurrentEntry = 0; CurrentEntry < EntryCount; CurrentEntry++)
//         {
//             /* Are we using the XSDT? */
//             if (!Xsdt)
//             {
//                 /* Read the 32-bit physical address */
//                 PhysicalAddress.LowPart = Rsdt->Tables[CurrentEntry];
//                 PhysicalAddress.HighPart = 0;
//             }
//             else
//             {
//                 /* Read the 64-bit physical address */
//                 PhysicalAddress = Xsdt->Tables[CurrentEntry];
//             }
//
//             /* Had we already mapped a table? */
//             if (Header)
//             {
//                 /* Yes, unmap it */
//                 if (LoaderBlock)
//                 {
//                     /* Using HAL heap */
//                     HalpUnmapVirtualAddress(Header, 2);
//                 }
//                 else
//                 {
//                     /* Using Mm */
//                     MmUnmapIoSpace(Header, 2 * PAGE_SIZE);
//                 }
//             }
//
//             /* Now map this table */
//             if (!LoaderBlock)
//             {
//                 /* Phase 1: Use HAL heap */
//                 Header = MmMapIoSpace(PhysicalAddress, 2 * PAGE_SIZE, MmNonCached);
//             }
//             else
//             {
//                 /* Phase 0: Use Mm */
//                 Header = HalpMapPhysicalMemory64(PhysicalAddress, 2);
//             }
//
//             /* Check if we mapped it */
//             if (!Header)
//             {
//                 /* Game over */
//                 DPRINT1("HAL: Failed to map ACPI table.\n");
//                 return NULL;
//             }
//
//             /* We found it, break out */
//             DPRINT("Found ACPI table %c%c%c%c at 0x%p\n",
//                     Header->Signature & 0xFF,
//                     (Header->Signature & 0xFF00) >> 8,
//                     (Header->Signature & 0xFF0000) >> 16,
//                     (Header->Signature & 0xFF000000) >> 24,
//                     Header);
//             if (Header->Signature == Signature) break;
//         }
//
//         /* Did we end up here back at the last entry? */
//         if (CurrentEntry == EntryCount)
//         {
//             /* Yes, unmap the last table we processed */
//             if (LoaderBlock)
//             {
//                 /* Using HAL heap */
//                 HalpUnmapVirtualAddress(Header, 2);
//             }
//             else
//             {
//                 /* Using Mm */
//                 MmUnmapIoSpace(Header, 2 * PAGE_SIZE);
//             }
//
//             /* Didn't find anything */
//             return NULL;
//         }
//     }
//
//     /* Past this point, we assume something was found */
//     ASSERT(Header);
//
//     /* How many pages do we need? */
//     PageCount = bytes_to_pages(Header->Length);
//     if (PageCount != 2)
//     {
//         /* We assumed two, but this is not the case, free the current mapping */
//         if (LoaderBlock)
//         {
//             /* Using HAL heap */
//             HalpUnmapVirtualAddress(Header, 2);
//         }
//         else
//         {
//             /* Using Mm */
//             MmUnmapIoSpace(Header, 2 * PAGE_SIZE);
//         }
//
//         /* Now map this table using its correct size */
//         if (!LoaderBlock)
//         {
//             /* Phase 1: Use HAL heap */
//             Header = MmMapIoSpace(PhysicalAddress, PageCount << PAGE_SHIFT, MmNonCached);
//         }
//         else
//         {
//             /* Phase 0: Use Mm */
//             Header = HalpMapPhysicalMemory64(PhysicalAddress, PageCount);
//         }
//     }
//
//     /* Fail if the remapped failed */
//     if (!Header) return NULL;
//
//     /* All tables in ACPI 3.0 other than the FACP should have correct checksum */
//     if ((Header->Signature != FADT_SIGNATURE) || (Header->Revision > 2))
//     {
//         /* Go to the end of the table */
//         CheckSum = 0;
//         CurrentByte = (PCHAR)Header + Header->Length;
//         while (CurrentByte-- != (PCHAR)Header)
//         {
//             /* Add this byte */
//             CheckSum += *CurrentByte;
//         }
//
//         /* The correct checksum is always 0, anything else is illegal */
//         if (CheckSum)
//         {
//             HalpInvalidAcpiTable = Header->Signature;
//             DPRINT1("Checksum failed on ACPI table %c%c%c%c\n",
//                     (Signature & 0xFF),
//                     (Signature & 0xFF00) >> 8,
//                     (Signature & 0xFF0000) >> 16,
//                     (Signature & 0xFF000000) >> 24);
//         }
//     }
//
//     /* Return the table */
//     return Header;
// }
//
// PVOID
// NTAPI
// HalpAcpiGetTable(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                  IN ULONG Signature)
// {
//     PFN_COUNT PageCount;
//     PDESCRIPTION_HEADER TableAddress, BiosCopy;
//
//     /* See if we have a cached table? */
//     TableAddress = HalpAcpiGetCachedTable(Signature);
//     if (!TableAddress)
//     {
//         /* No cache, search the BIOS */
//         TableAddress = HalpAcpiGetTableFromBios(LoaderBlock, Signature);
//         if (TableAddress)
//         {
//             /* Found it, copy it into our own memory */
//             BiosCopy = HalpAcpiCopyBiosTable(LoaderBlock, TableAddress);
//
//             /* Get the pages, and unmap the BIOS copy */
//             PageCount = bytes_to_pages(TableAddress->Length);
//             if (LoaderBlock)
//             {
//                 /* Phase 0, use the HAL heap */
//                 HalpUnmapVirtualAddress(TableAddress, PageCount);
//             }
//             else
//             {
//                 /* Phase 1, use Mm */
//                 MmUnmapIoSpace(TableAddress, PageCount << PAGE_SHIFT);
//             }
//
//             /* Cache the bios copy */
//             TableAddress = BiosCopy;
//             if (BiosCopy) HalpAcpiCacheTable(BiosCopy);
//         }
//     }
//
//     /* Return the table */
//     return TableAddress;
// }
//
// PVOID
// NTAPI
// HalAcpiGetTable(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                 IN ULONG Signature)
// {
//     PDESCRIPTION_HEADER TableHeader;
//
//     /* Is this phase0 */
//     if (LoaderBlock)
//     {
//         /* Initialize the cache first */
//         if (!nt_success(HalpAcpiTableCacheInit(LoaderBlock))) return NULL;
//     }
//     else
//     {
//         /* Lock the cache */
//         ExAcquireFastMutex(&HalpAcpiTableCacheLock);
//     }
//
//     /* Get the table */
//     TableHeader = HalpAcpiGetTable(LoaderBlock, Signature);
//
//     /* Release the lock in phase 1 */
//     if (!LoaderBlock) ExReleaseFastMutex(&HalpAcpiTableCacheLock);
//
//     /* Return the table */
//     return TableHeader;
// }
//
// VOID
// NTAPI
// HalpNumaInitializeStaticConfiguration(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PACPI_SRAT SratTable;
//
//     /* Get the SRAT, bail out if it doesn't exist */
//     SratTable = HalAcpiGetTable(LoaderBlock, SRAT_SIGNATURE);
//     HalpAcpiSrat = SratTable;
//     if (!SratTable) return;
// }
//
// VOID
// NTAPI
// HalpGetHotPlugMemoryInfo(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PACPI_SRAT SratTable;
//
//     /* Get the SRAT, bail out if it doesn't exist */
//     SratTable = HalAcpiGetTable(LoaderBlock, SRAT_SIGNATURE);
//     HalpAcpiSrat = SratTable;
//     if (!SratTable) return;
// }
//
// VOID
// NTAPI
// HalpDynamicSystemResourceConfiguration(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     /* For this HAL, it means to get hot plug memory information */
//     HalpGetHotPlugMemoryInfo(LoaderBlock);
// }
//
// VOID
// NTAPI
// HalpAcpiDetectMachineSpecificActions(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                                      IN PFADT DescriptionTable)
// {
//     /* Does this HAL specify something? */
//     if (HalpAcpiTableMatchList.Flink)
//     {
//         /* Great, but we don't support it */
//         DPRINT1("WARNING: Your HAL has specific ACPI hacks to apply!\n");
//     }
// }
//
// VOID
// NTAPI
// HalpInitBootTable(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PBOOT_TABLE BootTable;
//
//     /* Get the boot table */
//     BootTable = HalAcpiGetTable(LoaderBlock, BOOT_SIGNATURE);
//     HalpSimpleBootFlagTable = BootTable;
//
//     /* Validate it */
//     if ((BootTable) &&
//         (BootTable->Header.Length >= sizeof(BOOT_TABLE)) &&
//         (BootTable->CMOSIndex >= 9))
//     {
//         DPRINT1("ACPI Boot table found, but not supported!\n");
//     }
//     else
//     {
//         /* Invalid or doesn't exist, ignore it */
//         HalpSimpleBootFlagTable = 0;
//     }
//
//     /* Install the end of boot handler */
// //    HalEndOfBoot = HalpEndOfBoot;
// }
//
// NTSTATUS
// NTAPI
// HalpAcpiFindRsdtPhase0(IN PLOADER_PARAMETER_BLOCK LoaderBlock,
//                        OUT PACPI_BIOS_MULTI_NODE* AcpiMultiNode)
// {
//     PCONFIGURATION_COMPONENT_DATA ComponentEntry;
//     PCONFIGURATION_COMPONENT_DATA Next = NULL;
//     PCM_PARTIAL_RESOURCE_LIST ResourceList;
//     PACPI_BIOS_MULTI_NODE NodeData;
//     SIZE_T NodeLength;
//     PFN_COUNT PageCount;
//     PVOID MappedAddress;
//     PHYSICAL_ADDRESS PhysicalAddress;
//
//     /* Did we already do this once? */
//     if (HalpAcpiMultiNode)
//     {
//         /* Return what we know */
//         *AcpiMultiNode = HalpAcpiMultiNode;
//         return STATUS_SUCCESS;
//     }
//
//     /* Assume failure */
//     *AcpiMultiNode = NULL;
//
//     /* Find the multi function adapter key */
//     ComponentEntry = KeFindConfigurationNextEntry(LoaderBlock->ConfigurationRoot,
//                                                   AdapterClass,
//                                                   MultiFunctionAdapter,
//                                                   0,
//                                                   &Next);
//     while (ComponentEntry)
//     {
//         /* Find the ACPI BIOS key */
//         if (!_stricmp(ComponentEntry->ComponentEntry.Identifier, "ACPI BIOS"))
//         {
//             /* Found it */
//             break;
//         }
//
//         /* Keep searching */
//         Next = ComponentEntry;
//         ComponentEntry = KeFindConfigurationNextEntry(LoaderBlock->ConfigurationRoot,
//                                                       AdapterClass,
//                                                       MultiFunctionAdapter,
//                                                       NULL,
//                                                       &Next);
//     }
//
//     /* Make sure we found it */
//     if (!ComponentEntry)
//     {
//         DPRINT1("**** HalpAcpiFindRsdtPhase0: did NOT find RSDT\n");
//         return STATUS_NOT_FOUND;
//     }
//
//     /* The configuration data is a resource list, and the BIOS node follows */
//     ResourceList = ComponentEntry->ConfigurationData;
//     NodeData = (PACPI_BIOS_MULTI_NODE)(ResourceList + 1);
//
//     /* How many E820 memory entries are there? */
//     NodeLength = sizeof(ACPI_BIOS_MULTI_NODE) +
//                  (NodeData->Count - 1) * sizeof(ACPI_E820_ENTRY);
//
//     /* Convert to pages */
//     PageCount = (PFN_COUNT)bytes_to_pages(NodeLength);
//
//     /* Allocate the memory */
//     PhysicalAddress.QuadPart = HalpAllocPhysicalMemory(LoaderBlock,
//                                                        0x1000000,
//                                                        PageCount,
//                                                        FALSE);
//     if (PhysicalAddress.QuadPart)
//     {
//         /* Map it if the allocation worked */
//         MappedAddress = HalpMapPhysicalMemory64(PhysicalAddress, PageCount);
//     }
//     else
//     {
//         /* Otherwise we'll have to fail */
//         MappedAddress = NULL;
//     }
//
//     /* Save the multi node, bail out if we didn't find it */
//     HalpAcpiMultiNode = MappedAddress;
//     if (!MappedAddress) return STATUS_INSUFFICIENT_RESOURCES;
//
//     /* Copy the multi-node data */
//     RtlCopyMemory(MappedAddress, NodeData, NodeLength);
//
//     /* Return the data */
//     *AcpiMultiNode = HalpAcpiMultiNode;
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// HalpAcpiTableCacheInit(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     PACPI_BIOS_MULTI_NODE AcpiMultiNode;
//     NTSTATUS Status = STATUS_SUCCESS;
//     PHYSICAL_ADDRESS PhysicalAddress;
//     PVOID MappedAddress;
//     ULONG TableLength;
//     PRSDT Rsdt;
//     PLOADER_PARAMETER_EXTENSION LoaderExtension;
//
//     /* Only initialize once */
//     if (HalpAcpiTableCacheList.Flink) return Status;
//
//     /* Setup the lock and table */
//     ExInitializeFastMutex(&HalpAcpiTableCacheLock);
//     InitializeListHead(&HalpAcpiTableCacheList);
//
//     /* Find the RSDT */
//     Status = HalpAcpiFindRsdtPhase0(LoaderBlock, &AcpiMultiNode);
//     if (!nt_success(Status)) return Status;
//
//     PhysicalAddress.QuadPart = AcpiMultiNode->RsdtAddress.QuadPart;
//
//     /* Map the RSDT */
//     if (LoaderBlock)
//     {
//         /* Phase0: Use HAL Heap to map the RSDT, we assume it's about 2 pages */
//         MappedAddress = HalpMapPhysicalMemory64(PhysicalAddress, 2);
//     }
//     else
//     {
//         /* Use an I/O map */
//         MappedAddress = MmMapIoSpace(PhysicalAddress, PAGE_SIZE * 2, MmNonCached);
//     }
//
//     /* Get the RSDT */
//     Rsdt = MappedAddress;
//     if (!MappedAddress)
//     {
//         /* Fail, no memory */
//         DPRINT1("HAL: Failed to map RSDT\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Validate it */
//     if ((Rsdt->Header.Signature != RSDT_SIGNATURE) &&
//         (Rsdt->Header.Signature != XSDT_SIGNATURE))
//     {
//         /* Very bad: crash */
//         HalDisplayString("Bad RSDT pointer\r\n");
//         KeBugCheckEx(MISMATCHED_HAL, 4, __LINE__, 0, 0);
//     }
//
//     /* We assumed two pages -- do we need less or more? */
//     TableLength = ADDRESS_AND_SIZE_TO_SPAN_PAGES(PhysicalAddress.LowPart,
//                                                  Rsdt->Header.Length);
//     if (TableLength != 2)
//     {
//         /* Are we in phase 0 or 1? */
//         if (!LoaderBlock)
//         {
//             /* Unmap the old table, remap the new one, using Mm I/O space */
//             MmUnmapIoSpace(MappedAddress, 2 * PAGE_SIZE);
//             MappedAddress = MmMapIoSpace(PhysicalAddress,
//                                          TableLength << PAGE_SHIFT,
//                                          MmNonCached);
//         }
//         else
//         {
//             /* Unmap the old table, remap the new one, using HAL heap */
//             HalpUnmapVirtualAddress(MappedAddress, 2);
//             MappedAddress = HalpMapPhysicalMemory64(PhysicalAddress, TableLength);
//         }
//
//         /* Get the remapped table */
//         Rsdt = MappedAddress;
//         if (!MappedAddress)
//         {
//             /* Fail, no memory */
//             DPRINT1("HAL: Couldn't remap RSDT\n");
//             return STATUS_INSUFFICIENT_RESOURCES;
//         }
//     }
//
//     /* Now take the BIOS copy and make our own local copy */
//     Rsdt = HalpAcpiCopyBiosTable(LoaderBlock, &Rsdt->Header);
//     if (!Rsdt)
//     {
//         /* Fail, no memory */
//         DPRINT1("HAL: Couldn't remap RSDT\n");
//         return STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Get rid of the BIOS mapping */
//     if (LoaderBlock)
//     {
//         /* Use HAL heap */
//         HalpUnmapVirtualAddress(MappedAddress, TableLength);
//
//         LoaderExtension = LoaderBlock->Extension;
//     }
//     else
//     {
//         /* Use Mm */
//         MmUnmapIoSpace(MappedAddress, TableLength << PAGE_SHIFT);
//
//         LoaderExtension = NULL;
//     }
//
//     /* Cache the RSDT */
//     HalpAcpiCacheTable(&Rsdt->Header);
//
//     /* Check for compatible loader block extension */
//     if (LoaderExtension && (LoaderExtension->Size >= 0x58))
//     {
//         /* Compatible loader: did it provide an ACPI table override? */
//         if ((LoaderExtension->AcpiTable) && (LoaderExtension->AcpiTableSize))
//         {
//             /* Great, because we don't support it! */
//             DPRINT1("ACPI Table Overrides Not Supported!\n");
//         }
//     }
//
//     /* Done */
//     return Status;
// }
//
// VOID
// NTAPI
// HaliAcpiTimerInit(IN ULONG TimerPort,
//                   IN ULONG TimerValExt)
// {
//     PAGED_CODE();
//
//     /* Is this in the init phase? */
//     if (!TimerPort)
//     {
//         /* Get the data from the FADT */
//         TimerPort = HalpFixedAcpiDescTable.pm_tmr_blk_io_port;
//         TimerValExt = HalpFixedAcpiDescTable.flags & ACPI_TMR_VAL_EXT;
//         DPRINT1("ACPI Timer at: %lXh (EXT: %lu)\n", TimerPort, TimerValExt);
//     }
//
//     /* FIXME: Now proceed to the timer initialization */
//     //HalaAcpiTimerInit(TimerPort, TimerValExt);
// }
//
// CODE_SEG("INIT")
// NTSTATUS
// NTAPI
// HalpSetupAcpiPhase0(IN PLOADER_PARAMETER_BLOCK LoaderBlock)
// {
//     NTSTATUS Status;
//     PFADT Fadt;
//     ULONG TableLength;
//     PHYSICAL_ADDRESS PhysicalAddress;
//
//     /* Only do this once */
//     if (HalpProcessedACPIPhase0) return STATUS_SUCCESS;
//
//     /* Setup the ACPI table cache */
//     Status = HalpAcpiTableCacheInit(LoaderBlock);
//     if (!nt_success(Status)) return Status;
//
//     /* Grab the FADT */
//     Fadt = HalAcpiGetTable(LoaderBlock, FADT_SIGNATURE);
//     if (!Fadt)
//     {
//         /* Fail */
//         DPRINT1("HAL: Didn't find the FACP\n");
//         return STATUS_NOT_FOUND;
//     }
//
//     /* Assume typical size, otherwise whatever the descriptor table says */
//     TableLength = sizeof(FADT);
//     if (Fadt->Header.Length < sizeof(FADT)) TableLength = Fadt->Header.Length;
//
//     /* Copy it in the HAL static buffer */
//     RtlCopyMemory(&HalpFixedAcpiDescTable, Fadt, TableLength);
//
//     /* Anything special this HAL needs to do? */
//     HalpAcpiDetectMachineSpecificActions(LoaderBlock, &HalpFixedAcpiDescTable);
//
//     /* Get the debug table for KD */
//     HalpDebugPortTable = HalAcpiGetTable(LoaderBlock, DBGP_SIGNATURE) as _;
//
//     /* Initialize NUMA through the SRAT */
//     HalpNumaInitializeStaticConfiguration(LoaderBlock);
//
//     /* Initialize hotplug through the SRAT */
//     HalpDynamicSystemResourceConfiguration(LoaderBlock);
//     if (HalpAcpiSrat)
//     {
//         DPRINT1("Your machine has a SRAT, but NUMA/HotPlug are not supported!\n");
//     }
//
//     /* Can there be memory higher than 4GB? */
//     if (HalpMaxHotPlugMemoryAddress.HighPart >= 1)
//     {
//         /* We'll need this for DMA later */
//         HalpPhysicalMemoryMayAppearAbove4GB = TRUE;
//     }
//
//     /* Setup the ACPI timer */
//     HaliAcpiTimerInit(0, 0);
//
//     /* Do we have a low stub address yet? */
//     if (!HalpLowStubPhysicalAddress.QuadPart)
//     {
//         /* Allocate it */
//         HalpLowStubPhysicalAddress.QuadPart = HalpAllocPhysicalMemory(LoaderBlock,
//                                                                       0x100000,
//                                                                       HALP_LOW_STUB_SIZE_IN_PAGES,
//                                                                       FALSE);
//         if (HalpLowStubPhysicalAddress.QuadPart)
//         {
//             /* Map it */
//             HalpLowStub = HalpMapPhysicalMemory64(HalpLowStubPhysicalAddress, HALP_LOW_STUB_SIZE_IN_PAGES);
//         }
//     }
//
//     /* Grab a page for flushes */
//     PhysicalAddress.QuadPart = 0x100000;
//     HalpVirtAddrForFlush = HalpMapPhysicalMemory64(PhysicalAddress, 1);
//     HalpPteForFlush = HalAddressToPte(HalpVirtAddrForFlush) as PHARDWARE_PTE;
//
//     /* Don't do this again */
//     HalpProcessedACPIPhase0 = TRUE;
//
//     /* Setup the boot table */
//     HalpInitBootTable(LoaderBlock);
//
//     /* Log some ACPI data */
//     {
//         PLIST_ENTRY NextEntry;
//         PCSTR AcpiVersion = NULL;
//
//         /* Find the ACPI version (range) out */
//         // v1.0+: Revision is major version.
//         // v5.1+: minor_revision is minor version.
//         // v6.4+: errata bits are errata version.
//         switch (Fadt->Header.Revision)
//         {
//             case 0: // Should not happen.
//                 AcpiVersion = "Unknown_0";
//                 break;
//             case 1:
//                 AcpiVersion = "1.0-1.0b";
//                 break;
//             case 2: // Should not happen.
//                 AcpiVersion = "Unknown_2";
//                 break;
//             case 3:
//                 AcpiVersion = "1.5-2.0_C";
//                 break;
//             case 4:
//                 AcpiVersion = "3.0-4.0_A";
//                 break;
//             case 5:
//                 if (Fadt->minor_revision == 0)
//                     AcpiVersion = "5.0-5.0_B";
//                 else if (Fadt->minor_revision == 1)
//                     AcpiVersion = "5.1-5.1_B";
//                 break;
//             case 6:
//                 if (Fadt->minor_revision == 0)
//                     AcpiVersion = "6.0-6.0_A";
//                 else if (Fadt->minor_revision == 1)
//                     AcpiVersion = "6.1-6.1_A";
//                 else if (Fadt->minor_revision == 2)
//                     AcpiVersion = "6.2-6.2_B";
//                 else if (Fadt->minor_revision == 3)
//                     AcpiVersion = "6.3-6.3_A";
//                 else if ((Fadt->minor_revision & 0x0F) == 0x04)
//                 {
//                     if ((Fadt->minor_revision & 0xF0) == 0x00)
//                         AcpiVersion = "6.4";
//                     else if ((Fadt->minor_revision & 0xF0) == 0x10)
//                         AcpiVersion = "6.4_A";
//                 }
//                 else if (Fadt->minor_revision == 5) // v6.5_A too is documented as errata=0.
//                     AcpiVersion = "6.5-6.6";
//                 break;
//         }
//
//         /* Print the ACPI version */
//         DPRINT1("ACPI v");
//         if (AcpiVersion == NULL)
//         {
//             // Unknown past values, or newer than v6.6 (documented as 6.5).
//             DbgPrint("Unknown_%u_%u", Fadt->Header.Revision, Fadt->minor_revision);
//         }
//         else
//         {
//             DbgPrint("%s", AcpiVersion);
//         }
//         DbgPrint(" detected. Tables:");
//
//         /* List cached tables */
//         for (NextEntry = HalpAcpiTableCacheList.Flink;
//              NextEntry != &HalpAcpiTableCacheList;
//              NextEntry = NextEntry->Flink)
//         {
//             PACPI_CACHED_TABLE CachedTable = CONTAINING_RECORD(NextEntry, ACPI_CACHED_TABLE, Links);
//
//             /* Print the table signature */
//             DbgPrint(" [%c%c%c%c]",
//                       CachedTable->Header.Signature & 0x000000FF,
//                      (CachedTable->Header.Signature & 0x0000FF00) >>  8,
//                      (CachedTable->Header.Signature & 0x00FF0000) >> 16,
//                      (CachedTable->Header.Signature & 0xFF000000) >> 24);
//         }
//         DbgPrint("\n");
//     }
//
//     /* Return success */
//     return STATUS_SUCCESS;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// HalpInitializePciBus(VOID)
// {
//     /* Setup the PCI stub support */
//     HalpInitializePciStubs();
//
//     /* Set the NMI crash flag */
//     HalpGetNMICrashFlag();
// }
//
// VOID
// NTAPI
// HalpInitNonBusHandler(VOID)
// {
//     /* These should be written by the PCI driver later, but we give defaults */
//     HalPciTranslateBusAddress = HalpTranslateBusAddress;
//     HalPciAssignSlotResources = HalpAssignSlotResources;
//     HalFindBusAddressTranslation = HalpFindBusAddressTranslation;
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// HalpInitBusHandlers(VOID)
// {
//     /* On ACPI, we only have a fake PCI bus to worry about */
//     HalpInitNonBusHandler();
// }
//
// CODE_SEG("INIT")
// VOID
// NTAPI
// HalpBuildAddressMap(VOID)
// {
//     /* ACPI is magic baby */
// }
//
// CODE_SEG("INIT")
// BOOLEAN
// NTAPI
// HalpGetDebugPortTable(VOID)
// {
//     return ((HalpDebugPortTable) &&
//             (HalpDebugPortTable->BaseAddress.AddressSpaceID == 1));
// }
//
// CODE_SEG("INIT")
// ULONG
// NTAPI
// HalpIs16BitPortDecodeSupported(VOID)
// {
//     /* All ACPI systems are at least "EISA" so they support this */
//     return CM_RESOURCE_PORT_16_BIT_DECODE;
// }
//
// VOID
// NTAPI
// HalpAcpiDetectResourceListSize(OUT PULONG ListSize)
// {
//     PAGED_CODE();
//
//     /* One element if there is a SCI */
//     *ListSize = HalpFixedAcpiDescTable.sci_int_vector ? 1: 0;
// }
//
// NTSTATUS
// NTAPI
// HalpBuildAcpiResourceList(IN PIO_RESOURCE_REQUIREMENTS_LIST ResourceList)
// {
//     ULONG Interrupt;
//     PAGED_CODE();
//     ASSERT(ResourceList != NULL);
//
//     /* Initialize the list */
//     ResourceList->BusNumber = -1;
//     ResourceList->AlternativeLists = 1;
//     ResourceList->InterfaceType = PNPBus;
//     ResourceList->List[0].Version = 1;
//     ResourceList->List[0].Revision = 1;
//     ResourceList->List[0].Count = 0;
//
//     /* Is there a SCI? */
//     if (HalpFixedAcpiDescTable.sci_int_vector)
//     {
//         /* Fill out the entry for it */
//         ResourceList->List[0].Descriptors[0].Flags = CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE;
//         ResourceList->List[0].Descriptors[0].Type = CmResourceTypeInterrupt;
//         ResourceList->List[0].Descriptors[0].ShareDisposition = CmResourceShareShared;
//
//         /* Get the interrupt number */
//         Interrupt = HalpPicVectorRedirect[HalpFixedAcpiDescTable.sci_int_vector];
//         ResourceList->List[0].Descriptors[0].u.Interrupt.MinimumVector = Interrupt;
//         ResourceList->List[0].Descriptors[0].u.Interrupt.MaximumVector = Interrupt;
//
//         /* One more */
//         ++ResourceList->List[0].Count;
//     }
//
//     /* All good */
//     return STATUS_SUCCESS;
// }
//
// NTSTATUS
// NTAPI
// HalpQueryAcpiResourceRequirements(OUT PIO_RESOURCE_REQUIREMENTS_LIST *Requirements)
// {
//     PIO_RESOURCE_REQUIREMENTS_LIST RequirementsList;
//     ULONG Count, ListSize;
//     NTSTATUS Status;
//
//     PAGED_CODE();
//
//     /* Get ACPI resources */
//     HalpAcpiDetectResourceListSize(&Count);
//     DPRINT("Resource count: %lu\n", Count);
//
//     /* Compute size of the list and allocate it */
//     ListSize = FIELD_OFFSET(IO_RESOURCE_REQUIREMENTS_LIST, List[0].Descriptors) +
//                (Count * sizeof(IO_RESOURCE_DESCRIPTOR));
//     DPRINT("Resource list size: %lu\n", ListSize);
//     RequirementsList = ExAllocatePoolWithTag(PagedPool, ListSize, TAG_HAL);
//     if (RequirementsList)
//     {
//         /* Initialize it */
//         RtlZeroMemory(RequirementsList, ListSize);
//         RequirementsList->ListSize = ListSize;
//
//         /* Build it */
//         Status = HalpBuildAcpiResourceList(RequirementsList);
//         if (nt_success(Status))
//         {
//             /* It worked, return it */
//             *Requirements = RequirementsList;
//
//             /* Validate the list */
//             ASSERT(RequirementsList->List[0].Count == Count);
//         }
//         else
//         {
//             /* Fail */
//             ExFreePoolWithTag(RequirementsList, TAG_HAL);
//             Status = STATUS_NO_SUCH_DEVICE;
//         }
//     }
//     else
//     {
//         /* Not enough memory */
//         Status = STATUS_INSUFFICIENT_RESOURCES;
//     }
//
//     /* Return the status */
//     return Status;
// }
//
// /*
//  * @implemented
//  */
// CODE_SEG("INIT")
// VOID
// NTAPI
// HalReportResourceUsage(VOID)
// {
//     INTERFACE_TYPE InterfaceType;
//     UNICODE_STRING HalString;
//
//     /* FIXME: Initialize DMA 64-bit support */
//
//     /* FIXME: Initialize MCA bus */
//
//     /* Initialize PCI bus. */
//     HalpInitializePciBus();
//
//     /* What kind of bus is this? */
//     switch (HalpBusType)
//     {
//         /* ISA Machine */
//         case MACHINE_TYPE_ISA:
//             InterfaceType = Isa;
//             break;
//
//         /* EISA Machine */
//         case MACHINE_TYPE_EISA:
//             InterfaceType = Eisa;
//             break;
//
//         /* MCA Machine */
//         case MACHINE_TYPE_MCA:
//             InterfaceType = MicroChannel;
//             break;
//
//         /* Unknown */
//         default:
//             InterfaceType = Internal;
//             break;
//     }
//
//     /* Build HAL usage */
//     RtlInitUnicodeString(&HalString, HalName);
//     HalpReportResourceUsage(&HalString, InterfaceType);
//
//     /* Setup PCI debugging and Hibernation */
//     HalpRegisterPciDebuggingDeviceInfo();
// }
//
// /* EOF */
//
// ============================================================================
// END ORIGINAL C SOURCE
// ============================================================================
