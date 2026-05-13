//! generic/usage.c — HAL resource report routines.
//!
//! Builds the master raw + translated CM_RESOURCE_LIST that the kernel
//! consumes via IoReportHalResourceUsage. Owns the IDT-usage tables and
//! the address-usage list head, plus the static COM/PIC/DMA/PIT/CMOS
//! port-claim tables.
//!
//! Compiled for both i386 and amd64.
//! _MINIHAL_ guards: dropped (the Rust HAL never builds for minihal).

use core::ptr;

use ntkrnl_sys::ddk::{
    ExAllocatePoolWithTag, ExFreePool, KeBugCheckEx, RtlInitUnicodeString, ZwClose, ZwOpenKey,
    ZwQueryValueKey,
};
use ntkrnl_sys::hal_ddk::{
    HalEnableSystemInterrupt, HalTranslateBusAddress, HalpActiveProcessors, HalpBuildAddressMap,
    HalpGetDebugPortTable, HalpIs16BitPortDecodeSupported, HalpMarkAcpiHal, HalpReportSerialNumber,
    BOOLEAN, CM_PARTIAL_RESOURCE_DESCRIPTOR, CM_PARTIAL_RESOURCE_LIST,
    CM_RESOURCE_INTERRUPT_LATCHED, CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE,
    CM_RESOURCE_MEMORY_READ_ONLY, CM_RESOURCE_MEMORY_READ_WRITE, CM_RESOURCE_PORT_IO,
    CM_RESOURCE_PORT_MEMORY, CM_RESOURCE_TYPE, CmResourceTypeInterrupt, CmResourceTypeMemory,
    CmResourceTypePort, HIGH_LEVEL, INTERFACE_TYPE, KIRQL, LARGE_INTEGER, MAXIMUM_IDTVECTOR,
    PCM_FULL_RESOURCE_DESCRIPTOR, PCM_PARTIAL_RESOURCE_DESCRIPTOR, PCM_RESOURCE_LIST,
    PRIMARY_VECTOR_BASE, PVOID, UCHAR, ULONG, ULONG_PTR, USHORT, _IDTUsageFlags as IdtUsageFlags,
    _CM_SHARE_DISPOSITION_CmResourceShareDriverExclusive as SHARE_DRIVER_EXCLUSIVE,
    _INTERFACE_TYPE_Internal as IFT_INTERNAL,
    _INTERFACE_TYPE_InterfaceTypeUndefined as IFT_UNDEFINED, _KINTERRUPT_MODE_Latched as LATCHED,
    _POOL_TYPE_NonPagedPool as NON_PAGED_POOL,
};
// UNICODE_STRING / OBJECT_ATTRIBUTES / KEY_VALUE_PARTIAL_INFORMATION / HANDLE come from ddk
// since the registry/Zw*/Rtl* helpers all expect that flavor.
use ntkrnl_sys::ddk::{
    HANDLE, KEY_VALUE_PARTIAL_INFORMATION, OBJECT_ATTRIBUTES, NTSTATUS, UNICODE_STRING,
};

// KeRegisterInterruptHandler is FORCEINLINE in ntoskrnl/include/internal/{i386,amd64}/ke.h
// (every C caller inlined it).  ntoskrnl.exe does NOT export the symbol, so we
// cannot raw-dylib it — inline the body locally per-arch instead.
//
// HalVectorToIDTEntry is the function-pointer field at HalPrivateDispatchTable's
// HalVectorToIDTEntry slot; the call is indirect through that pointer.

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn register_interrupt_handler(vector: ULONG, handler: PVOID) {
    let pcr = crate::x86::pcr::KeGetPcr() as *mut ntkrnl_sys::hal_ddk::_KIPCR;
    let dispatch = &raw const ntkrnl_sys::hal_ddk::HalPrivateDispatchTable;
    let vec_to_idt = (*dispatch).HalVectorToIDTEntry.unwrap_unchecked();
    let entry = vec_to_idt(vector) as usize;
    let e = &mut *((*pcr).IDT).add(entry);
    let address = handler as usize;
    e.ExtendedOffset = (address >> 16) as u16;
    e.Offset = address as u16;
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn register_interrupt_handler(vector: ULONG, handler: PVOID) {
    let pcr = crate::x86::pcr::KeGetPcr() as *mut ntkrnl_sys::hal_ddk::_KIPCR;
    let dispatch = &raw const ntkrnl_sys::hal_ddk::HalPrivateDispatchTable;
    let vec_to_idt = (*dispatch).HalVectorToIDTEntry.unwrap_unchecked();
    let entry = vec_to_idt(vector) as usize;
    let idt = (*pcr).IdtBase.add(entry);
    let inner = &mut (*idt).__bindgen_anon_1;
    let address = handler as usize;
    inner.OffsetLow = (address & 0xffff) as u16;
    inner.OffsetMiddle = ((address >> 16) & 0xffff) as u16;
    inner.OffsetHigh = (address >> 32) as u32;
    const KGDT64_R0_CODE: u16 = 0x0010;
    inner.Selector = KGDT64_R0_CODE;
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const PAGE_SIZE: usize = 4096;
// HAL_MEMORY_ALLOCATION = 0xAC (sdk/include/reactos/mc/bugcodes.mc)
const HAL_MEMORY_ALLOCATION: ULONG = 0x0000_00AC;
// TAG_HAL = ' laH' as multi-char literal — bytes 'H','a','l',' ' little-endian.
const TAG_HAL: ULONG = u32::from_le_bytes(*b"Hal ");

// halp.h — IDT usage flag bits
const IDT_REGISTERED: u8 = 0x01;
const IDT_LATCHED: u8 = 0x02;
const IDT_READ_ONLY: u8 = 0x04;
const IDT_INTERNAL: u8 = 0x11;
const IDT_DEVICE: u8 = 0x21;

// ZwOpenKey access mask + OBJ_CASE_INSENSITIVE
const KEY_READ: ULONG = 0x0002_0019;
const OBJ_CASE_INSENSITIVE: ULONG = 0x0000_0040;
// KeyValuePartialInformation = 2 (KEY_VALUE_INFORMATION_CLASS, c_uint)
const KEY_VALUE_PARTIAL_INFORMATION_CLASS: u32 = 2;

#[inline]
fn nt_success(s: NTSTATUS) -> bool {
    s >= 0
}

// ---------------------------------------------------------------------------
// Address-usage struct (matches C `_HalAddressUsage` prefix; bindgen models
// the trailing `Element[]` as an `__IncompleteArrayField` so we cannot
// construct one directly in Rust. We define sibling structs with concrete
// arrays for the static tables here, then expose them as the bindgen type
// at link time via #[no_mangle].)
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Copy, Clone)]
pub struct AddrElement {
    pub start: ULONG,
    pub length: ULONG,
}

#[repr(C)]
pub struct AddressUsage<const N: usize> {
    pub next: *mut u8,
    pub type_: CM_RESOURCE_TYPE,
    pub flags: UCHAR,
    pub elements: [AddrElement; N],
}

// SARCH_PC98 not built; non-PC98 layout has 11 entries + null = 12.
const DEFAULT_IO_SPACE_LEN: usize = 12;
const COM_IO_SPACE_LEN: usize = 2;

#[unsafe(no_mangle)]
pub static mut HalpComIoSpace: AddressUsage<COM_IO_SPACE_LEN> = AddressUsage {
    next: ptr::null_mut(),
    type_: CmResourceTypePort as CM_RESOURCE_TYPE,
    flags: IDT_INTERNAL,
    elements: [
        AddrElement { start: 0x2F8, length: 0x8 }, // COM 1 (per C; KdComPort overrides at runtime)
        AddrElement { start: 0, length: 0 },
    ],
};

#[unsafe(no_mangle)]
pub static mut HalpDefaultIoSpace: AddressUsage<DEFAULT_IO_SPACE_LEN> = AddressUsage {
    next: ptr::null_mut(),
    type_: CmResourceTypePort as CM_RESOURCE_TYPE,
    flags: IDT_INTERNAL,
    elements: [
        AddrElement { start: 0x00, length: 0x20 }, // DMA 1
        AddrElement { start: 0xC0, length: 0x20 }, // DMA 2
        AddrElement { start: 0x80, length: 0x10 }, // DMA EPAR
        AddrElement { start: 0x20, length: 0x2 },  // PIC 1
        AddrElement { start: 0xA0, length: 0x2 },  // PIC 2
        AddrElement { start: 0x40, length: 0x4 },  // PIT 1
        AddrElement { start: 0x48, length: 0x4 },  // PIT 2
        AddrElement { start: 0x92, length: 0x1 },  // System Control Port A
        AddrElement { start: 0x70, length: 0x2 },  // CMOS
        AddrElement { start: 0xF0, length: 0x10 }, // x87 Coprocessor
        AddrElement { start: 0xCF8, length: 0x8 }, // PCI 0
        AddrElement { start: 0, length: 0 },
    ],
};

// ---------------------------------------------------------------------------
// Other globals — defined locally with #[no_mangle]; bindgen externs in
// hal_ddk resolve to these symbols at link time.
// ---------------------------------------------------------------------------

// KdComPortInUse — HAL-internal global, was a C definition in usage.c.
// Other Rust crates declare it via raw-dylib extern; this is the storage.
#[unsafe(no_mangle)]
pub static mut KdComPortInUse: *mut u8 = ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpGetInfoFromACPI: BOOLEAN = 0;

#[unsafe(no_mangle)]
pub static mut HalpNMIDumpFlag: BOOLEAN = 0;

#[unsafe(no_mangle)]
pub static mut HalpAddressUsageList: *mut u8 = ptr::null_mut();

#[unsafe(no_mangle)]
pub static mut HalpComPortIrqMapping: [[USHORT; 2]; 5] = [
    [0x3F8, 4],
    [0x2F8, 3],
    [0x3E8, 4],
    [0x2E8, 3],
    [0, 0],
];

#[unsafe(no_mangle)]
pub static mut HalpIDTUsageFlags: [IdtUsageFlags; (MAXIMUM_IDTVECTOR + 1) as usize] =
    [IdtUsageFlags { Flags: 0 }; (MAXIMUM_IDTVECTOR + 1) as usize];

// _IDTUsage is two UCHARs; replicate it locally so the static gets a
// concrete layout and we don't fight the bindgen Debug/Copy derives.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct HalpIdtUsageEntry {
    pub irql: KIRQL,
    pub bus_relative_vector: UCHAR,
}

#[unsafe(no_mangle)]
pub static mut HalpIDTUsage: [HalpIdtUsageEntry; (MAXIMUM_IDTVECTOR + 1) as usize] =
    [HalpIdtUsageEntry { irql: 0, bus_relative_vector: 0 }; (MAXIMUM_IDTVECTOR + 1) as usize];

// ---------------------------------------------------------------------------
// HalpGetResourceSortValue — switch on descriptor type, fill scale + value.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetResourceSortValue(
    descriptor: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    scale: *mut ULONG,
    value: *mut LARGE_INTEGER,
) {
    unsafe {
        let d = &*descriptor;
        let type_ = d.Type as u32;
        if type_ == CmResourceTypeInterrupt as u32 {
            ptr::write(scale, 0);
            let mut li: LARGE_INTEGER = core::mem::zeroed();
            li.QuadPart = d.u.Interrupt.Level as i64;
            ptr::write(value, li);
        } else if type_ == CmResourceTypePort as u32 {
            ptr::write(scale, 1);
            ptr::write(value, d.u.Port.Start);
        } else if type_ == CmResourceTypeMemory as u32 {
            ptr::write(scale, 2);
            ptr::write(value, d.u.Memory.Start);
        } else {
            ptr::write(scale, 4);
            let mut li: LARGE_INTEGER = core::mem::zeroed();
            li.QuadPart = 0;
            ptr::write(value, li);
        }
    }
}

// ---------------------------------------------------------------------------
// HalpBuildPartialFromIdt
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpBuildPartialFromIdt(
    entry: ULONG,
    raw: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    translated: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) {
    unsafe {
        let r = &mut *raw;
        r.Type = CmResourceTypeInterrupt as u8;
        r.ShareDisposition = SHARE_DRIVER_EXCLUSIVE as u8;

        let flags_byte = HalpIDTUsageFlags[entry as usize].Flags;
        r.Flags = if flags_byte & IDT_LATCHED != 0 {
            CM_RESOURCE_INTERRUPT_LATCHED as USHORT
        } else {
            CM_RESOURCE_INTERRUPT_LEVEL_SENSITIVE as USHORT
        };

        let bv = HalpIDTUsage[entry as usize].bus_relative_vector as ULONG;
        r.u.Interrupt.Vector = bv;
        r.u.Interrupt.Level = bv;
        r.u.Interrupt.Affinity = HalpActiveProcessors as _;

        // Translated descriptor = raw bit-for-bit, then patch vector + level.
        ptr::copy_nonoverlapping(
            raw as *const u8,
            translated as *mut u8,
            core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
        );

        let t = &mut *translated;
        t.u.Interrupt.Vector = entry;
        t.u.Interrupt.Level = HalpIDTUsage[entry as usize].irql as ULONG;
    }
}

// ---------------------------------------------------------------------------
// HalpBuildPartialFromAddress
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpBuildPartialFromAddress(
    interface_type: INTERFACE_TYPE,
    current_address: *mut u8, // PADDRESS_USAGE — opaque pointer
    element: ULONG,
    raw: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
    translated: PCM_PARTIAL_RESOURCE_DESCRIPTOR,
) {
    unsafe {
        // Extract type, flags, and element[N] via the prefix layout:
        //   *Next = +0 (pointer)
        //   Type  = +sizeof(ptr)             (CM_RESOURCE_TYPE = c_int, 4 bytes)
        //   Flags = +sizeof(ptr) + 4         (u8)
        //   Element[0] starts at +sizeof(ptr) + 8 (4-byte align maintained)
        let ptr_size = core::mem::size_of::<*mut u8>();
        let cur = current_address;
        let type_off = ptr_size;
        let flags_off = ptr_size + 4;
        let elem_off = ptr_size + 8;

        let cur_type: CM_RESOURCE_TYPE = ptr::read_unaligned(cur.add(type_off) as *const _);
        let cur_flags: UCHAR = *cur.add(flags_off);
        let elem_ptr = cur.add(elem_off) as *mut AddrElement;
        let elem = ptr::read(elem_ptr.add(element as usize));

        let r = &mut *raw;
        r.Type = cur_type as u8;
        r.ShareDisposition = SHARE_DRIVER_EXCLUSIVE as u8;

        let mut address_space: ULONG;
        if r.Type as u32 == CmResourceTypePort as u32 {
            address_space = 1;
            r.Flags = CM_RESOURCE_PORT_IO as USHORT;
            r.u.Port.Start.QuadPart = elem.start as i64;
            r.u.Port.Length = elem.length;

            // ORs in CM_RESOURCE_PORT_16_BIT_DECODE if HALP says so.
            r.Flags |= HalpIs16BitPortDecodeSupported() as USHORT;
        } else {
            address_space = 0;
            r.Flags = if cur_flags & IDT_READ_ONLY != 0 {
                CM_RESOURCE_MEMORY_READ_ONLY as USHORT
            } else {
                CM_RESOURCE_MEMORY_READ_WRITE as USHORT
            };
            r.u.Memory.Start.QuadPart = elem.start as i64;
            r.u.Memory.Length = elem.length;
        }

        ptr::copy_nonoverlapping(
            raw as *const u8,
            translated as *mut u8,
            core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>(),
        );

        let t = &mut *translated;
        if r.Type as u32 == CmResourceTypePort as u32 {
            let raw_start = r.u.Port.Start;
            let mut translated_start: LARGE_INTEGER = core::mem::zeroed();
            HalTranslateBusAddress(
                interface_type,
                0,
                raw_start,
                &mut address_space,
                &mut translated_start,
            );
            ptr::write_unaligned(&raw mut t.u.Port.Start, translated_start);
            if address_space == 0 {
                t.Flags = CM_RESOURCE_PORT_MEMORY as USHORT;
            }
        } else {
            let raw_start = r.u.Memory.Start;
            let mut translated_start: LARGE_INTEGER = core::mem::zeroed();
            HalTranslateBusAddress(
                interface_type,
                0,
                raw_start,
                &mut address_space,
                &mut translated_start,
            );
            ptr::write_unaligned(&raw mut t.u.Memory.Start, translated_start);
        }
    }
}

// ---------------------------------------------------------------------------
// HalpReportResourceUsage
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpReportResourceUsage(
    hal_name: *mut UNICODE_STRING,
    interface_type_outer: INTERFACE_TYPE,
) {
    unsafe {
        // DbgPrint("%wZ Detected\n", HalName) — drop; not worth the trouble.

        let kd_com = ptr::read_volatile(&raw const KdComPortInUse);
        if !kd_com.is_null() {
            // Patch HalpComIoSpace.Element[0].Start = (ULONG)KdComPortInUse
            (*(&raw mut HalpComIoSpace)).elements[0].start = kd_com as ULONG;

            // Insert HalpComIoSpace at head of HalpAddressUsageList.
            (*(&raw mut HalpComIoSpace)).next = HalpAddressUsageList;
            ptr::write_volatile(
                &raw mut HalpAddressUsageList,
                &raw mut HalpComIoSpace as *mut u8,
            );

            // Use ACPI's debug-port table if present; otherwise scan local map.
            HalpGetInfoFromACPI = HalpGetDebugPortTable();
            if HalpGetInfoFromACPI == 0 {
                let mut i = 0usize;
                while HalpComPortIrqMapping[i][0] != 0 {
                    let port = HalpComPortIrqMapping[i][0] as ULONG_PTR;
                    if port == kd_com as ULONG_PTR {
                        HalpRegisterVector(
                            IDT_DEVICE | IDT_LATCHED,
                            HalpComPortIrqMapping[i][1] as ULONG,
                            HalpComPortIrqMapping[i][1] as ULONG + PRIMARY_VECTOR_BASE,
                            HIGH_LEVEL as KIRQL,
                        );
                    }
                    i += 1;
                }
            }
        }

        // Address map for non-ACPI systems
        HalpBuildAddressMap();

        let alloc_size = (PAGE_SIZE * 2) as ULONG_PTR;
        let raw_list = ExAllocatePoolWithTag(NON_PAGED_POOL as _, alloc_size as _, TAG_HAL)
            as PCM_RESOURCE_LIST;
        let translated_list = ExAllocatePoolWithTag(NON_PAGED_POOL as _, alloc_size as _, TAG_HAL)
            as PCM_RESOURCE_LIST;
        if raw_list.is_null() || translated_list.is_null() {
            KeBugCheckEx(
                HAL_MEMORY_ALLOCATION,
                (4 * PAGE_SIZE) as ULONG_PTR,
                1,
                0,
                line!() as ULONG_PTR,
            );
        }
        ptr::write_bytes(raw_list as *mut u8, 0, PAGE_SIZE * 2);
        ptr::write_bytes(translated_list as *mut u8, 0, PAGE_SIZE * 2);

        // RawList->List[0].InterfaceType = InterfaceTypeUndefined
        let raw_first_full: PCM_FULL_RESOURCE_DESCRIPTOR = (*raw_list).List.as_mut_ptr();
        (*raw_first_full).InterfaceType = IFT_UNDEFINED;

        // Mark all non-IRQ vectors as IDT_INTERNAL if not already taken.
        for i in 0..(PRIMARY_VECTOR_BASE as usize) {
            if HalpIDTUsageFlags[i].Flags & IDT_REGISTERED == 0 {
                HalpIDTUsageFlags[i].Flags = IDT_INTERNAL;
                HalpIDTUsage[i].bus_relative_vector = i as UCHAR;
            }
        }

        // Pointers we walk while emitting partials.
        let mut raw_full: PCM_FULL_RESOURCE_DESCRIPTOR = raw_first_full;
        let mut current_raw: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
            (*raw_first_full).PartialResourceList.PartialDescriptors.as_mut_ptr();
        let mut current_translated: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
            (*(*translated_list).List.as_mut_ptr())
                .PartialResourceList
                .PartialDescriptors
                .as_mut_ptr();
        let mut translated_full: PCM_FULL_RESOURCE_DESCRIPTOR =
            (*translated_list).List.as_mut_ptr();

        let mut raw_partial_list: *mut CM_PARTIAL_RESOURCE_LIST = ptr::null_mut();
        let mut translated_partial_list: *mut CM_PARTIAL_RESOURCE_LIST = ptr::null_mut();

        let mut raw_partial: CM_PARTIAL_RESOURCE_DESCRIPTOR = core::mem::zeroed();
        let mut translated_partial: CM_PARTIAL_RESOURCE_DESCRIPTOR = core::mem::zeroed();

        // Two passes: device first, then internal HAL.
        for report_type in 0..2 {
            let (flag_match, interface_) = if report_type == 0 {
                (IDT_DEVICE & !IDT_REGISTERED, interface_type_outer)
            } else {
                (IDT_INTERNAL & !IDT_REGISTERED, IFT_INTERNAL)
            };

            let mut i: ULONG = 0;
            let mut element: ULONG = 0;
            let mut current_address = HalpAddressUsageList;

            loop {
                if i <= MAXIMUM_IDTVECTOR {
                    if HalpIDTUsageFlags[i as usize].Flags & flag_match != 0 {
                        HalpBuildPartialFromIdt(i, &mut raw_partial, &mut translated_partial);
                        i += 1;
                    } else {
                        i += 1;
                        continue;
                    }
                } else {
                    if current_address.is_null() {
                        break;
                    }
                    let ptr_size = core::mem::size_of::<*mut u8>();
                    let next_off = 0;
                    let flags_off = ptr_size + 4;
                    let elem_off = ptr_size + 8;
                    let entry_flags = *current_address.add(flags_off);
                    let elem_ptr = current_address.add(elem_off) as *mut AddrElement;
                    let elem_len = (*elem_ptr.add(element as usize)).length;
                    if entry_flags & flag_match == 0 || elem_len == 0 {
                        element = 0;
                        let next_ptr = current_address.add(next_off) as *mut *mut u8;
                        current_address = *next_ptr;
                        continue;
                    }

                    HalpBuildPartialFromAddress(
                        interface_,
                        current_address,
                        element,
                        &mut raw_partial,
                        &mut translated_partial,
                    );
                    element += 1;
                }

                if (*raw_full).InterfaceType != interface_ {
                    (*raw_list).Count += 1;
                    (*translated_list).Count += 1;

                    raw_full = current_raw as PCM_FULL_RESOURCE_DESCRIPTOR;
                    translated_full = current_translated as PCM_FULL_RESOURCE_DESCRIPTOR;

                    (*raw_full).InterfaceType = interface_;
                    (*translated_full).InterfaceType = interface_;

                    raw_partial_list = &mut (*raw_full).PartialResourceList;
                    translated_partial_list = &mut (*translated_full).PartialResourceList;

                    current_raw =
                        (*raw_full).PartialResourceList.PartialDescriptors.as_mut_ptr();
                    current_translated = (*translated_full)
                        .PartialResourceList
                        .PartialDescriptors
                        .as_mut_ptr();
                }

                (*raw_partial_list).Count += 1;
                (*translated_partial_list).Count += 1;

                ptr::copy_nonoverlapping(&raw_partial, current_raw, 1);
                ptr::copy_nonoverlapping(&translated_partial, current_translated, 1);

                current_raw = current_raw.add(1);
                current_translated = current_translated.add(1);
            }
        }

        let list_size =
            (current_raw as ULONG_PTR).wrapping_sub(raw_list as ULONG_PTR) as ULONG;

        // Insertion-sort partial descriptors within each full descriptor.
        let mut raw_full: PCM_FULL_RESOURCE_DESCRIPTOR = (*raw_list).List.as_mut_ptr();
        let mut translated_full: PCM_FULL_RESOURCE_DESCRIPTOR =
            (*translated_list).List.as_mut_ptr();
        for _ in 0..(*raw_list).Count {
            let mut current_raw: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
                (*raw_full).PartialResourceList.PartialDescriptors.as_mut_ptr();
            let mut current_translated: PCM_PARTIAL_RESOURCE_DESCRIPTOR =
                (*translated_full)
                    .PartialResourceList
                    .PartialDescriptors
                    .as_mut_ptr();
            let count = (*raw_full).PartialResourceList.Count;
            for j in 0..count {
                let mut current_scale: ULONG = 0;
                let mut current_sort: LARGE_INTEGER = core::mem::zeroed();
                HalpGetResourceSortValue(current_raw, &mut current_scale, &mut current_sort);

                let mut sorted_raw = current_raw;
                let mut sorted_translated = current_translated;

                for _ in j..count {
                    let mut sort_scale: ULONG = 0;
                    let mut sort_value: LARGE_INTEGER = core::mem::zeroed();
                    HalpGetResourceSortValue(sorted_raw, &mut sort_scale, &mut sort_value);

                    if sort_scale < current_scale
                        || (sort_scale == current_scale
                            && sort_value.QuadPart <= current_sort.QuadPart)
                    {
                        let tmp_r = ptr::read(current_raw);
                        ptr::write(current_raw, ptr::read(sorted_raw));
                        ptr::write(sorted_raw, tmp_r);

                        let tmp_t = ptr::read(current_translated);
                        ptr::write(current_translated, ptr::read(sorted_translated));
                        ptr::write(sorted_translated, tmp_t);

                        HalpGetResourceSortValue(
                            current_raw,
                            &mut current_scale,
                            &mut current_sort,
                        );
                    }

                    sorted_raw = sorted_raw.add(1);
                    sorted_translated = sorted_translated.add(1);
                }

                current_raw = current_raw.add(1);
                current_translated = current_translated.add(1);
            }

            raw_full = current_raw as PCM_FULL_RESOURCE_DESCRIPTOR;
            translated_full = current_translated as PCM_FULL_RESOURCE_DESCRIPTOR;
        }

        HalpMarkAcpiHal();

        // Cross between hal_ddk and ddk UNICODE_STRING types — bit-identical
        // layout, just different bindgen names.  Cast the pointer.
        IoReportHalResourceUsage(
            hal_name as *mut _,
            raw_list as *mut _,
            translated_list as *mut _,
            list_size,
        );

        ExFreePool(raw_list as PVOID);
        ExFreePool(translated_list as PVOID);

        HalpReportSerialNumber();
    }
}

unsafe extern "system" {
    fn IoReportHalResourceUsage(
        HalName: *mut UNICODE_STRING,
        RawResourceList: *mut core::ffi::c_void,
        TranslatedResourceList: *mut core::ffi::c_void,
        ResourceListSize: ULONG,
    ) -> NTSTATUS;
}

// ---------------------------------------------------------------------------
// HalpRegisterVector
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpRegisterVector(
    flags: UCHAR,
    bus_vector: ULONG,
    system_vector: ULONG,
    irql: KIRQL,
) {
    unsafe {
        HalpIDTUsageFlags[system_vector as usize].Flags = flags;
        HalpIDTUsage[system_vector as usize].irql = irql;
        HalpIDTUsage[system_vector as usize].bus_relative_vector = bus_vector as UCHAR;
    }
}

// ---------------------------------------------------------------------------
// HalpEnableInterruptHandler
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpEnableInterruptHandler(
    flags: UCHAR,
    bus_vector: ULONG,
    system_vector: ULONG,
    irql: KIRQL,
    handler: PVOID,
    mode: u32, // KINTERRUPT_MODE
) {
    unsafe {
        let f = if mode == LATCHED as u32 { flags | IDT_LATCHED } else { flags };
        HalpRegisterVector(f, bus_vector, system_vector, irql);
        register_interrupt_handler(system_vector, handler);
        HalEnableSystemInterrupt(system_vector, irql, mode as _);
    }
}

// ---------------------------------------------------------------------------
// HalpGetNMICrashFlag — read NMICrashDump from registry.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpGetNMICrashFlag() {
    unsafe {
        HalpNMIDumpFlag = 0;

        // L"\\Registry\\Machine\\System\\CurrentControlSet\\Control\\CrashControl"
        const KEY_NAME: [u16; 65] = [
            0x005C, 0x0052, 0x0065, 0x0067, 0x0069, 0x0073, 0x0074, 0x0072, 0x0079, 0x005C,
            0x004D, 0x0061, 0x0063, 0x0068, 0x0069, 0x006E, 0x0065, 0x005C, 0x0053, 0x0079,
            0x0073, 0x0074, 0x0065, 0x006D, 0x005C, 0x0043, 0x0075, 0x0072, 0x0072, 0x0065,
            0x006E, 0x0074, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C, 0x0053,
            0x0065, 0x0074, 0x005C, 0x0043, 0x006F, 0x006E, 0x0074, 0x0072, 0x006F, 0x006C,
            0x005C, 0x0043, 0x0072, 0x0061, 0x0073, 0x0068, 0x0043, 0x006F, 0x006E, 0x0074,
            0x0072, 0x006F, 0x006C, 0x0000, 0x0000,
        ];

        let mut key_name: UNICODE_STRING = core::mem::zeroed();
        // Length excludes trailing null (63 chars * 2 bytes).
        key_name.Length = (63 * 2) as USHORT;
        key_name.MaximumLength = (64 * 2) as USHORT;
        key_name.Buffer = KEY_NAME.as_ptr() as *mut u16;

        let mut object_attributes: OBJECT_ATTRIBUTES = core::mem::zeroed();
        object_attributes.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as ULONG;
        object_attributes.RootDirectory = ptr::null_mut();
        object_attributes.ObjectName = &mut key_name;
        object_attributes.Attributes = OBJ_CASE_INSENSITIVE;
        object_attributes.SecurityDescriptor = ptr::null_mut();
        object_attributes.SecurityQualityOfService = ptr::null_mut();

        let mut handle: HANDLE = ptr::null_mut();
        let status = ZwOpenKey(&mut handle, KEY_READ, &mut object_attributes);
        if !nt_success(status) {
            return;
        }

        // L"NMICrashDump"
        const VALUE_NAME: [u16; 13] = [
            0x004E, 0x004D, 0x0049, 0x0043, 0x0072, 0x0061, 0x0073, 0x0068, 0x0044, 0x0075,
            0x006D, 0x0070, 0x0000,
        ];
        let mut value_name: UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(&mut value_name, VALUE_NAME.as_ptr());

        let mut info: KEY_VALUE_PARTIAL_INFORMATION = core::mem::zeroed();
        let mut result_length: ULONG = 0;
        let status = ZwQueryValueKey(
            handle,
            &mut value_name,
            KEY_VALUE_PARTIAL_INFORMATION_CLASS,
            &mut info as *mut _ as PVOID,
            core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG,
            &mut result_length,
        );
        if nt_success(status)
            && result_length as usize == core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>()
        {
            HalpNMIDumpFlag = info.Data[0];
        }

        ZwClose(handle);
    }
}
