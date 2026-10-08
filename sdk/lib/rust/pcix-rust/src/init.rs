#[allow(unused_imports)] use crate::types::*;
#[allow(unused_imports)] use crate::ddk_shims::*;
#[allow(unused_imports)] use crate::glue::*;
#[allow(unused_imports)] use crate::guids::*;
use ntkrnl_sys::globals::*;
use ntkrnl_sys::base::*;
use ntkrnl_sys::constants::*;
use ntkrnl_sys::driver_ddk::*;
use ntkrnl_sys::ex::*;
use ntkrnl_sys::mm::*;
use ntkrnl_sys::rtl::*;
use ntkrnl_sys::ddk::{CM_FULL_RESOURCE_DESCRIPTOR, ExFreePool, HalDisplayString, KEVENT, KEY_VALUE_FULL_INFORMATION, KEY_VALUE_PARTIAL_INFORMATION, KeInitializeEvent, MmMapIoSpace, MmUnmapIoSpace, _OBJECT_ATTRIBUTES, PDRIVER_OBJECT, PHYSICAL_ADDRESS, PKEY_BASIC_INFORMATION, PKEY_FULL_INFORMATION, PKEY_VALUE_FULL_INFORMATION, PKEY_VALUE_PARTIAL_INFORMATION, PULONGLONG, PUNICODE_STRING, RtlInitUnicodeString, _EVENT_TYPE_SynchronizationEvent, ZwClose, ZwEnumerateKey, ZwEnumerateValueKey, ZwOpenKey, ZwQueryKey, ZwQueryValueKey};
use ntkrnl_sys::ddk::{
    _KEY_INFORMATION_CLASS_KeyBasicInformation as KeyBasicInformation,
    _KEY_INFORMATION_CLASS_KeyFullInformation as KeyFullInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValueFullInformation as KeyValueFullInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation,
};
use core::ffi::c_void;
// resolve_imports.py: explicit kernel/private-type imports
use crate::types::{_PCI_IRQ_ROUTING_TABLE as PCI_IRQ_ROUTING_TABLE};
use ntkrnl_sys::ddk::{DISPATCHER_HEADER, KEY_FULL_INFORMATION, PCSTR, PCUNICODE_STRING, PHANDLE, RtlAssert, _DRIVER_OBJECT, _LIST_ENTRY, _UNICODE_STRING};
use ntkrnl_sys::ke::{DbgBreakPoint};
use ntkrnl_sys::ddk::LARGE_INTEGER;

// w! — NUL-terminated UTF-16 array from an ASCII literal (2-aligned 'static;
// a b"...\0" byte string cast to *const u16 is both ANSI-not-wide and
// potentially odd-aligned — see field manual / basesrv init.rs CSDVersion).
macro_rules! w {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; { BYTES.len() + 1 }];
            let mut i = 0;
            while i < BYTES.len() {
                out[i] = BYTES[i] as u16;
                i += 1;
            }
            out
        };
        __W
    }};
}


// ---- ACPI private types (from sdk/include/reactos/drivers/acpi/acpi.h) ----
// gen_driver_types did not bind these (acpi.h not in pcix bindgen allowlist);
// defined here faithfully so c2rust's references resolve. Layout matches the C.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _ACPI_E820_ENTRY {
    pub Base: PHYSICAL_ADDRESS,
    pub Length: LARGE_INTEGER,
    pub Type: ULONGLONG,
}
pub type ACPI_E820_ENTRY = _ACPI_E820_ENTRY;
pub type PACPI_E820_ENTRY = *mut _ACPI_E820_ENTRY;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct _ACPI_BIOS_MULTI_NODE {
    pub RsdtAddress: PHYSICAL_ADDRESS,
    pub Count: ULONGLONG,
    pub E820Entry: [ACPI_E820_ENTRY; 1usize],
}
pub type ACPI_BIOS_MULTI_NODE = _ACPI_BIOS_MULTI_NODE;
pub type PACPI_BIOS_MULTI_NODE = *mut _ACPI_BIOS_MULTI_NODE;

#[repr(C)]
pub struct _RSDT {
    pub Header: DESCRIPTION_HEADER,
    pub Tables: [ULONG; 1usize],
}
pub type RSDT = _RSDT;
pub type PRSDT = *mut _RSDT;

#[repr(C)]
pub struct _XSDT {
    pub Header: DESCRIPTION_HEADER,
    pub Tables: [PHYSICAL_ADDRESS; 1usize],
}
pub type XSDT = _XSDT;
pub type PXSDT = *mut _XSDT;

// CRT wide-string compare, exported by ntoskrnl. Not in ntkrnl-sys; forward-declared.
extern "C" {
    fn wcsncmp(s1: *const u16, s2: *const u16, n: usize) -> ::core::ffi::c_int;
}

pub const RSDT_SIGNATURE: ULONG = 0x54445352;
// XSDT_SIGNATURE = 'TDSX' multi-char literal → 0x54445358
pub const XSDT_SIGNATURE: ULONG = 0x54445358;

// ---- file-scope preamble (translated) ----
// @c2rust file-scope data (compiler-exact tables/statics)
pub const PCI_HACK_ENTRY_SIZE: usize = (::core::mem::size_of::<[::core::ffi::c_ushort; 9]>()
    as usize)
    .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize);

pub const PCI_HACK_ENTRY_REV_SIZE: usize = (::core::mem::size_of::<[::core::ffi::c_ushort; 11]>()
    as usize)
    .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize);

pub const PCI_HACK_ENTRY_SUBSYS_SIZE: usize =
    (::core::mem::size_of::<[::core::ffi::c_ushort; 17]>() as usize)
        .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize);

pub const PCI_HACK_ENTRY_FULL_SIZE: usize = (::core::mem::size_of::<[::core::ffi::c_ushort; 19]>()
    as usize)
    .wrapping_sub(::core::mem::size_of::<WCHAR>() as usize);

pub static mut PciRunningDatacenter: BOOLEAN = 0;

#[no_mangle]
pub static mut PciDriverObject: PDRIVER_OBJECT =
    ::core::ptr::null::<_DRIVER_OBJECT>() as *mut _DRIVER_OBJECT;

// C: `KEVENT PciGlobalLock;` — zero-initialized BSS; re-initialized via
// KeInitializeEvent in PciInitializeGlobals. c2rust emitted a verbose union
// literal with anon-struct names that don't match bindgen's layout; a zeroed
// static is the faithful equivalent.
#[no_mangle]
pub static mut PciGlobalLock: KEVENT = unsafe { core::mem::zeroed() };

pub static mut PciBusLock: KEVENT = unsafe { core::mem::zeroed() };

pub static mut PciLegacyDescriptionLock: KEVENT = unsafe { core::mem::zeroed() };

pub static mut PciLockDeviceResources: BOOLEAN = 0;

#[no_mangle]
pub static mut PciEnableNativeModeATA: BOOLEAN = 0;

pub static mut PciSystemWideHackFlags: ULONG = 0;

#[no_mangle]
pub static mut PciIrqRoutingTable: PPCI_IRQ_ROUTING_TABLE =
    ::core::ptr::null::<_PCI_IRQ_ROUTING_TABLE>() as *mut _PCI_IRQ_ROUTING_TABLE;

pub static mut WdTable: PWATCHDOG_TABLE =
    ::core::ptr::null::<_WATCHDOG_TABLE>() as *mut _WATCHDOG_TABLE;

#[no_mangle]
pub static mut PciHackTable: PPCI_HACK_ENTRY =
    ::core::ptr::null::<_PCI_HACK_ENTRY>() as *mut _PCI_HACK_ENTRY;

// C: `DRIVER_UNLOAD PciDriverUnload;` is a FUNCTION forward-declaration, not a
// static. c2rust misread it as data; the real fn is defined below. Dropped.

#[repr(C)]
struct PackageStruct {
    Descriptor: CM_FULL_RESOURCE_DESCRIPTOR,
    Table: PCI_IRQ_ROUTING_TABLE,
}
#[no_mangle]
pub unsafe extern "system" fn PciAcpiFindRsdt(
    AcpiMultiNode: *mut PACPI_BIOS_MULTI_NODE,
) -> NTSTATUS {
    let mut Result: BOOLEAN;
    let mut Status: NTSTATUS = 0;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut SubKey: HANDLE = core::ptr::null_mut();
    let mut NumberOfBytes: ULONG = 0;
    let mut i: ULONG;
    let mut Length: ULONG;
    let mut FullInfo: PKEY_FULL_INFORMATION = core::ptr::null_mut();
    let mut KeyInfo: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut ValueInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut NodeData: PACPI_BIOS_MULTI_NODE;
    let mut ValueName: _UNICODE_STRING = core::mem::zeroed();
    let mut Package: *mut core::ffi::c_void = core::ptr::null_mut();

    ValueInfo = core::ptr::null_mut();
    KeyInfo = core::ptr::null_mut();
    KeyHandle = core::ptr::null_mut();
    FullInfo = core::ptr::null_mut();
    Package = core::ptr::null_mut();

    'body: loop {
        Result = PciOpenKey(
            win32macros_rust::wide_bytes!(b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0H\0A\0R\0D\0W\0A\0R\0E\0\\\0D\0E\0S\0C\0R\0I\0P\0T\0I\0O\0N\0\\\0S\0y\0s\0t\0e\0m\0\\\0M\0u\0l\0t\0i\0F\0u\0n\0c\0t\0i\0o\0n\0A\0d\0a\0p\0t\0e\0r\0\0\0") as PWCHAR,
            core::ptr::null_mut(),
            KEY_QUERY_VALUE,
            &mut KeyHandle,
            &mut Status,
        );
        if Result == 0 {
            break 'body;
        }

        Status = ZwQueryKey(
            KeyHandle,
            KeyFullInformation,
            core::ptr::null_mut(),
            core::mem::size_of::<ULONG>() as ULONG,
            &mut NumberOfBytes,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'body;
        }

        Status = STATUS_INSUFFICIENT_RESOURCES;
        FullInfo = ExAllocatePoolWithTag(PagedPool, NumberOfBytes as SIZE_T, PCI_POOL_TAG as ULONG)
            as PKEY_FULL_INFORMATION;
        if FullInfo.is_null() {
            break 'body;
        }

        Status = ZwQueryKey(
            KeyHandle,
            KeyFullInformation,
            FullInfo as PVOID,
            NumberOfBytes,
            &mut NumberOfBytes,
        );
        if !NT_SUCCESS(Status) {
            break 'body;
        }

        Status = STATUS_INSUFFICIENT_RESOURCES;
        Length = ((*FullInfo).MaxNameLen).wrapping_add(26);
        KeyInfo = ExAllocatePoolWithTag(PagedPool, Length as SIZE_T, PCI_POOL_TAG as ULONG)
            as PKEY_BASIC_INFORMATION;
        if KeyInfo.is_null() {
            break 'body;
        }

        ValueInfo = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as SIZE_T)
                .wrapping_add(10 * core::mem::size_of::<u16>() as SIZE_T),
            PCI_POOL_TAG as ULONG,
        ) as PKEY_VALUE_PARTIAL_INFORMATION;
        if ValueInfo.is_null() {
            break 'body;
        }

        i = 0;
        loop {
            Status = ZwEnumerateKey(
                KeyHandle,
                i,
                KeyBasicInformation,
                KeyInfo as PVOID,
                Length,
                &mut NumberOfBytes,
            );
            i = i.wrapping_add(1);
            if Status == STATUS_NO_MORE_ENTRIES {
                break;
            }

            let name_len = ((*KeyInfo).NameLength as usize) / core::mem::size_of::<u16>();
            *((*KeyInfo).Name.as_mut_ptr().add(name_len)) = 0u16;

            Result = PciOpenKey(
                (*KeyInfo).Name.as_mut_ptr(),
                KeyHandle,
                KEY_QUERY_VALUE,
                &mut SubKey,
                &mut Status,
            );
            if Result == 0 {
                continue;
            }

            RtlInitUnicodeString(
                &mut ValueName,
                w!("Identifier").as_ptr() as PCWSTR,
            );
            Status = ZwQueryValueKey(
                SubKey,
                &mut ValueName,
                KeyValuePartialInformation,
                ValueInfo as PVOID,
                (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>() as ULONG)
                    .wrapping_add(10 * core::mem::size_of::<u16>() as ULONG),
                &mut NumberOfBytes,
            );

            if NT_SUCCESS(Status) {
                if wcsncmp(
                    (*ValueInfo).Data.as_ptr() as *const u16,
                    w!("ACPI BIOS").as_ptr() as *const u16,
                    (*ValueInfo).DataLength as usize,
                ) == 0
                {
                    Status = PciGetRegistryValue(
                        win32macros_rust::wide_bytes!(b"C\0o\0n\0f\0i\0g\0u\0r\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0")
                            as PWCHAR,
                        (*KeyInfo).Name.as_mut_ptr() as PWCHAR,
                        KeyHandle,
                        REG_FULL_RESOURCE_DESCRIPTOR as ULONG,
                        &mut Package as *mut PVOID,
                        &mut NumberOfBytes,
                    );
                    ZwClose(SubKey);
                    break;
                }
            }
            ZwClose(SubKey);
        }

        if !NT_SUCCESS(Status) {
            break 'body;
        }
        if Package.is_null() {
            break 'body;
        }

        NodeData = Package.cast::<u8>()
            .add(core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>())
            as PACPI_BIOS_MULTI_NODE;

        Length = (core::mem::size_of::<ACPI_BIOS_MULTI_NODE>() as ULONG)
            .wrapping_add(
                ((*NodeData).Count as ULONG).wrapping_sub(1)
                    * core::mem::size_of::<ACPI_E820_ENTRY>() as ULONG,
            );

        Status = STATUS_INSUFFICIENT_RESOURCES;
        *AcpiMultiNode = ExAllocatePoolWithTag(NonPagedPool, Length as SIZE_T, PCI_POOL_TAG as ULONG)
            as PACPI_BIOS_MULTI_NODE;
        if (*AcpiMultiNode).is_null() {
            break 'body;
        }

        core::ptr::copy_nonoverlapping(
            NodeData as *const u8,
            *AcpiMultiNode as *mut u8,
            Length as usize,
        );
        Status = STATUS_SUCCESS;
        break 'body;
    }

    if !Package.is_null() {
        ExFreePoolWithTag(Package as PVOID, 0);
    }
    if !ValueInfo.is_null() {
        ExFreePoolWithTag(ValueInfo as PVOID, 0);
    }
    if !KeyInfo.is_null() {
        ExFreePoolWithTag(KeyInfo as PVOID, 0);
    }
    if !FullInfo.is_null() {
        ExFreePoolWithTag(FullInfo as PVOID, 0);
    }
    if !KeyHandle.is_null() {
        ZwClose(KeyHandle);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciGetAcpiTable(TableCode: u32) -> *mut core::ffi::c_void {
    let mut Header: *mut DESCRIPTION_HEADER = core::ptr::null_mut();
    let mut AcpiMultiNode: *mut _ACPI_BIOS_MULTI_NODE = core::ptr::null_mut();
    let mut Rsdt: *mut RSDT = core::ptr::null_mut();
    let mut Xsdt: *mut XSDT = core::ptr::null_mut();
    let mut EntryCount: u32;
    let mut TableLength: u32;
    let mut Offset: u32;
    let mut CurrentEntry: u32;
    let mut TableBuffer: *mut core::ffi::c_void = core::ptr::null_mut();
    let mut MappedAddress: *mut core::ffi::c_void;
    let mut PhysicalAddress: PHYSICAL_ADDRESS = core::mem::zeroed();
    let Status: NTSTATUS = PciAcpiFindRsdt(&raw mut AcpiMultiNode);

    if Status < 0 {
        if DbgPrint(
            (b"(%s:%d) AcpiFindRsdt() Failed!\n\0".as_ptr() as *const u8).cast(),
            b"drivers/bus/pcix/init.c\0".as_ptr() as *const i8,
            214i32,
        ) != 0 {
            DbgPrint(
                (b"(%s:%d) DbgPrint() failed!\n\0".as_ptr() as *const u8).cast(),
                b"drivers/bus/pcix/init.c\0".as_ptr() as *const i8,
                214i32,
            );
        }
        return core::ptr::null_mut();
    }

    MappedAddress = MmMapIoSpace(
        (*AcpiMultiNode).RsdtAddress,
        core::mem::size_of::<DESCRIPTION_HEADER>() as _,
        MmNonCached,
    );
    Header = MappedAddress as *mut DESCRIPTION_HEADER;
    if Header.is_null() {
        return core::ptr::null_mut();
    }

    TableLength = (*Header).Length;
    MmUnmapIoSpace(
        Header as *mut core::ffi::c_void,
        core::mem::size_of::<DESCRIPTION_HEADER>() as _,
    );
    Header = core::ptr::null_mut();

    MappedAddress = MmMapIoSpace(
        (*AcpiMultiNode).RsdtAddress,
        TableLength as _,
        MmNonCached,
    );
    Rsdt = MappedAddress as *mut RSDT;
    Xsdt = MappedAddress as *mut XSDT;
    ExFreePoolWithTag(AcpiMultiNode as *mut core::ffi::c_void, 0u32);
    if Rsdt.is_null() {
        return core::ptr::null_mut();
    }

    if (*Rsdt).Header.Signature != RSDT_SIGNATURE && (*Rsdt).Header.Signature != XSDT_SIGNATURE {
        HalDisplayString((b"RSDT table contains invalid signature\r\n\0".as_ptr() as *const u8).cast());
        MmUnmapIoSpace(Rsdt as *mut core::ffi::c_void, TableLength as _);
        return core::ptr::null_mut();
    }

    Offset = 36u32;
    if (*Rsdt).Header.Signature == XSDT_SIGNATURE {
        TableLength = (*Xsdt).Header.Length;
        if TableLength < Offset {
            Offset = (*Xsdt).Header.Length;
        }
        EntryCount = (TableLength.wrapping_sub(Offset) as usize
            / core::mem::size_of::<PHYSICAL_ADDRESS>()) as u32;
    } else {
        TableLength = (*Rsdt).Header.Length;
        if TableLength < Offset {
            Offset = (*Rsdt).Header.Length;
        }
        EntryCount = (TableLength.wrapping_sub(Offset) as usize
            / core::mem::size_of::<u32>()) as u32;
    }

    CurrentEntry = 0u32;
    while CurrentEntry < EntryCount {
        if (*Rsdt).Header.Signature != XSDT_SIGNATURE {
            PhysicalAddress.QuadPart = *((*Rsdt).Tables.as_ptr() as *const u32)
                .offset(CurrentEntry as isize) as i64;
        } else {
            PhysicalAddress = *((*Xsdt).Tables.as_ptr() as *const PHYSICAL_ADDRESS)
                .offset(CurrentEntry as isize);
        }

        Header = MmMapIoSpace(
            PhysicalAddress,
            core::mem::size_of::<DESCRIPTION_HEADER>() as _,
            MmNonCached,
        ) as *mut DESCRIPTION_HEADER;
        if Header.is_null() {
            break;
        }

        if (*Header).Signature == TableCode {
            TableBuffer = ExAllocatePoolWithTag(
                PagedPool,
                (*Header).Length as usize,
                PCI_POOL_TAG as u32,
            );
            if TableBuffer.is_null() {
                break;
            }
            RtlCopyMemory(
                TableBuffer,
                Header as *const core::ffi::c_void,
                (*Header).Length as usize,
            );
        }

        MmUnmapIoSpace(
            Header as *mut core::ffi::c_void,
            core::mem::size_of::<DESCRIPTION_HEADER>() as _,
        );
        CurrentEntry = CurrentEntry.wrapping_add(1);
    }

    if !Header.is_null() {
        MmUnmapIoSpace(
            Header as *mut core::ffi::c_void,
            core::mem::size_of::<DESCRIPTION_HEADER>() as _,
        );
    }

    core::ptr::null_mut()
}
#[no_mangle]
pub unsafe extern "system" fn PciGetIrqRoutingTableFromRegistry(
    PciRoutingTable: *mut PPCI_IRQ_ROUTING_TABLE,
) -> NTSTATUS {
    let mut Status: NTSTATUS = 0;
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut SubKey: HANDLE = core::ptr::null_mut();
    let mut NumberOfBytes: ULONG = 0;
    let mut i: ULONG = 0;
    let mut Length: ULONG = 0;
    let mut FullInfo: PKEY_FULL_INFORMATION = core::ptr::null_mut();
    let mut KeyInfo: PKEY_BASIC_INFORMATION = core::ptr::null_mut();
    let mut ValueInfo: PKEY_VALUE_PARTIAL_INFORMATION = core::ptr::null_mut();
    let mut Package: *mut PackageStruct = core::ptr::null_mut();
    let mut value_name: _UNICODE_STRING = core::mem::zeroed();

    'body: {
        let result = PciOpenKey(
            win32macros_rust::wide_bytes!(b"\\\0R\0e\0g\0i\0s\0t\0r\0y\0\\\0M\0a\0c\0h\0i\0n\0e\0\\\0H\0A\0R\0D\0W\0A\0R\0E\0\\\0D\0E\0S\0C\0R\0I\0P\0T\0I\0O\0N\0\\\0S\0y\0s\0t\0e\0m\0\\\0M\0u\0l\0t\0i\0F\0u\0n\0c\0t\0i\0o\0n\0A\0d\0a\0p\0t\0e\0r\0\0\0") as PWCHAR,
            core::ptr::null_mut(),
            KEY_QUERY_VALUE,
            &mut KeyHandle,
            &mut Status,
        );
        if result == 0 {
            break 'body;
        }

        Status = ZwQueryKey(
            KeyHandle,
            KeyFullInformation,
            core::ptr::null_mut(),
            core::mem::size_of::<ULONG>() as ULONG,
            &mut NumberOfBytes,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break 'body;
        }

        Status = STATUS_INSUFFICIENT_RESOURCES;
        FullInfo = ExAllocatePoolWithTag(PagedPool, NumberOfBytes as usize, PCI_POOL_TAG) as PKEY_FULL_INFORMATION;
        if FullInfo.is_null() {
            break 'body;
        }

        Status = ZwQueryKey(
            KeyHandle,
            KeyFullInformation,
            FullInfo as PVOID,
            NumberOfBytes,
            &mut NumberOfBytes,
        );
        if !NT_SUCCESS(Status) {
            break 'body;
        }

        Status = STATUS_INSUFFICIENT_RESOURCES;
        Length = (*FullInfo).MaxNameLen.wrapping_add(26);
        KeyInfo = ExAllocatePoolWithTag(PagedPool, Length as usize, PCI_POOL_TAG) as PKEY_BASIC_INFORMATION;
        if KeyInfo.is_null() {
            break 'body;
        }

        ValueInfo = ExAllocatePoolWithTag(
            PagedPool,
            (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>()
                + ::core::mem::size_of::<[u16; 9]>()) as SIZE_T,
            PCI_POOL_TAG,
        ) as PKEY_VALUE_PARTIAL_INFORMATION;
        if ValueInfo.is_null() {
            break 'body;
        }

        i = 0;
        'enumerate: loop {
            let current_i = i;
            i = i.wrapping_add(1);
            Status = ZwEnumerateKey(
                KeyHandle,
                current_i,
                KeyBasicInformation,
                KeyInfo as PVOID,
                Length,
                &mut NumberOfBytes,
            );
            if Status == STATUS_NO_MORE_ENTRIES {
                break 'enumerate;
            }

            let name_len_chars = (*KeyInfo).NameLength as usize / core::mem::size_of::<u16>();
            *(*KeyInfo).Name.as_mut_ptr().add(name_len_chars) = 0;

            let open_result = PciOpenKey(
                (*KeyInfo).Name.as_mut_ptr() as PWCHAR,
                KeyHandle,
                KEY_QUERY_VALUE,
                &mut SubKey,
                &mut Status,
            );
            if open_result != 0 {
                RtlInitUnicodeString(
                    &mut value_name,
                    w!("Identifier").as_ptr() as PCWSTR,
                );
                Status = ZwQueryValueKey(
                    SubKey,
                    &mut value_name,
                    KeyValuePartialInformation,
                    ValueInfo as PVOID,
                    (core::mem::size_of::<KEY_VALUE_PARTIAL_INFORMATION>()
                        + core::mem::size_of::<[u16; 9]>()) as ULONG,
                    &mut NumberOfBytes,
                );
                if NT_SUCCESS(Status) {
                    let data_ptr = (*ValueInfo).Data.as_ptr() as *const u16;
                    let pci_bios_ptr =
                        w!("PCI BIOS").as_ptr() as *const u16;
                    if wcsncmp(data_ptr, pci_bios_ptr, (*ValueInfo).DataLength as usize) == 0 {
                        Status = PciGetRegistryValue(
                            win32macros_rust::wide_bytes!(b"C\0o\0n\0f\0i\0g\0u\0r\0a\0t\0i\0o\0n\0 \0D\0a\0t\0a\0\0\0") as PWCHAR,
                            win32macros_rust::wide_bytes!(b"R\0e\0a\0l\0M\0o\0d\0e\0I\0r\0q\0R\0o\0u\0t\0i\0n\0g\0T\0a\0b\0l\0e\0\\\x000\0\0\0") as PWCHAR,
                            SubKey,
                            REG_FULL_RESOURCE_DESCRIPTOR as ULONG,
                            &mut Package as *mut _ as *mut PVOID,
                            &mut NumberOfBytes,
                        );
                        ZwClose(SubKey);
                        break 'enumerate;
                    }
                }
                ZwClose(SubKey);
            }
        }

        if !NT_SUCCESS(Status) {
            break 'body;
        }
        if Package.is_null() {
            break 'body;
        }
        if (NumberOfBytes as usize) < core::mem::size_of::<PackageStruct>()
            || ((*Package).Table.TableSize as usize)
                > (NumberOfBytes as usize)
                    .wrapping_sub(core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>())
        {
            Status = STATUS_UNSUCCESSFUL;
            break 'body;
        }

        Status = STATUS_INSUFFICIENT_RESOURCES;
        *PciRoutingTable = ExAllocatePoolWithTag(
            PagedPool,
            NumberOfBytes as usize,
            PCI_POOL_TAG,
        ) as PPCI_IRQ_ROUTING_TABLE;
        if (*PciRoutingTable).is_null() {
            break 'body;
        }

        let src = core::ptr::addr_of!((*Package).Table) as *const u8;
        let dst = *PciRoutingTable as *mut u8;
        let copy_size = NumberOfBytes as usize - core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>();
        core::ptr::copy_nonoverlapping(src, dst, copy_size);
        Status = STATUS_SUCCESS;
    }

    if !Package.is_null() {
        ExFreePoolWithTag(Package as PVOID, 0);
    }
    if !ValueInfo.is_null() {
        ExFreePoolWithTag(ValueInfo as PVOID, 0);
    }
    if !KeyInfo.is_null() {
        ExFreePoolWithTag(KeyInfo as PVOID, 0);
    }
    if !FullInfo.is_null() {
        ExFreePoolWithTag(FullInfo as PVOID, 0);
    }
    if !KeyHandle.is_null() {
        ZwClose(KeyHandle);
    }

    Status
}
#[no_mangle]
pub unsafe extern "system" fn PciBuildHackTable(KeyHandle: HANDLE) -> NTSTATUS {
    let mut FullInfo: *mut KEY_FULL_INFORMATION = core::ptr::null_mut();
    let mut ValueInfo: *mut KEY_VALUE_FULL_INFORMATION = core::ptr::null_mut();
    let mut Status: NTSTATUS;
    let mut ResultLength: u32;
    let mut HackCount: u32;
    let mut i: u32;
    let mut Entry: *mut PCI_HACK_ENTRY;
    let mut NameLength: u32;
    let mut HackFlags: u64;

    loop {
        ResultLength = 0;
        Status = ZwQueryKey(
            KeyHandle,
            KeyFullInformation,
            core::ptr::null_mut(),
            0,
            &mut ResultLength,
        );
        if Status != STATUS_BUFFER_TOO_SMALL {
            break;
        }

        Status = STATUS_INSUFFICIENT_RESOURCES;
        if !(ResultLength > 0) {
            RtlAssert(
                b"ResultLength > 0\0".as_ptr() as PVOID,
                b"\\init.c\0".as_ptr() as PVOID,
                505,
                core::ptr::null_mut::<i8>(),
            );
        }

        FullInfo = ExAllocatePoolWithTag(
            PagedPool,
            ResultLength as usize,
            PCI_POOL_TAG,
        ) as *mut KEY_FULL_INFORMATION;
        if FullInfo.is_null() {
            break;
        }

        Status = ZwQueryKey(
            KeyHandle,
            KeyFullInformation,
            FullInfo as PVOID,
            ResultLength,
            &mut ResultLength,
        );
        if !NT_SUCCESS(Status) {
            break;
        }

        HackCount = (*FullInfo).Values;
        ExFreePoolWithTag(FullInfo as PVOID, 0);
        FullInfo = core::ptr::null_mut();

        Status = STATUS_INSUFFICIENT_RESOURCES;
        ResultLength = (core::mem::size_of::<PCI_HACK_ENTRY>() * HackCount as usize) as u32;

        PciHackTable = ExAllocatePoolWithTag(
            NonPagedPool,
            ResultLength as usize + core::mem::size_of::<PCI_HACK_ENTRY>(),
            PCI_POOL_TAG,
        ) as *mut PCI_HACK_ENTRY;
        if PciHackTable.is_null() {
            break;
        }

        ValueInfo = ExAllocatePoolWithTag(
            NonPagedPool,
            core::mem::size_of::<KEY_VALUE_FULL_INFORMATION>() + PCI_HACK_ENTRY_FULL_SIZE,
            PCI_POOL_TAG,
        ) as *mut KEY_VALUE_FULL_INFORMATION;
        if ValueInfo.is_null() {
            break;
        }

        Entry = &mut *PciHackTable;
        for i in 0..HackCount {
            Entry = &mut *PciHackTable.add(i as usize);

            Status = ZwEnumerateValueKey(
                KeyHandle,
                i,
                KeyValueFullInformation,
                ValueInfo as PVOID,
                (core::mem::size_of::<KEY_VALUE_FULL_INFORMATION>() + PCI_HACK_ENTRY_FULL_SIZE)
                    as u32,
                &mut ResultLength,
            );
            if !NT_SUCCESS(Status) {
                if Status != STATUS_BUFFER_OVERFLOW && Status != STATUS_BUFFER_TOO_SMALL {
                    break;
                }
                continue;
            }

            if (*ValueInfo).Type != REG_BINARY
                || (*ValueInfo).DataLength as usize != core::mem::size_of::<u64>()
            {
                continue;
            }

            HackFlags =
                *((ValueInfo as *mut u8).add((*ValueInfo).DataOffset as usize) as *mut u64);

            NameLength = (*ValueInfo).NameLength;
            if NameLength as usize != PCI_HACK_ENTRY_SIZE
                && NameLength as usize != PCI_HACK_ENTRY_REV_SIZE
                && NameLength as usize != PCI_HACK_ENTRY_SUBSYS_SIZE
                && NameLength as usize != PCI_HACK_ENTRY_FULL_SIZE
            {
                DbgPrint(b"Skipping hack entry with invalid length name\n\0".as_ptr() as PCSTR);
                continue;
            }

            RtlZeroMemory(
                Entry as PVOID,
                core::mem::size_of::<PCI_HACK_ENTRY>(),
            );

            if PciStringToUSHORT((*ValueInfo).Name.as_ptr() as PWCHAR, &mut (*Entry).VendorID) == 0
                || PciStringToUSHORT(
                    (*ValueInfo).Name.as_ptr().add(4) as PWCHAR,
                    &mut (*Entry).DeviceID,
                ) == 0
            {
                continue;
            }

            if NameLength as usize == PCI_HACK_ENTRY_SUBSYS_SIZE
                || NameLength as usize == PCI_HACK_ENTRY_FULL_SIZE
            {
                if PciStringToUSHORT(
                    (*ValueInfo).Name.as_ptr().add(8) as PWCHAR,
                    &mut (*Entry).SubVendorID,
                ) == 0
                    || PciStringToUSHORT(
                        (*ValueInfo).Name.as_ptr().add(12) as PWCHAR,
                        &mut (*Entry).SubSystemID,
                    ) == 0
                {
                    continue;
                }
                (*Entry).Flags |= PCI_HACK_HAS_SUBSYSTEM_INFO as u8;
            }

            if NameLength as usize == PCI_HACK_ENTRY_REV_SIZE
                || NameLength as usize == PCI_HACK_ENTRY_FULL_SIZE
            {
                if PciStringToUSHORT(
                    (*ValueInfo).Name.as_ptr().add(16) as PWCHAR,
                    &mut (*Entry).RevisionID,
                ) == 0
                {
                    continue;
                }
                (*Entry).Flags |= PCI_HACK_HAS_REVISION_INFO as u8;
            }

            if (*Entry).VendorID == PCI_INVALID_VENDORID as u16 {
                RtlAssert(
                    b"Entry->VendorID != PCI_INVALID_VENDORID\0".as_ptr() as PVOID,
                    b"\\init.c\0".as_ptr() as PVOID,
                    639,
                    core::ptr::null_mut::<i8>(),
                );
            }
            (*Entry).HackFlags = HackFlags;
        }

        if !NT_SUCCESS(Status) {
            break;
        }

        if !(Entry < PciHackTable.add(HackCount as usize + 1)) {
            RtlAssert(
                b"Entry < (PciHackTable + HackCount + 1)\0".as_ptr() as PVOID,
                b"\\init.c\0".as_ptr() as PVOID,
                661,
                core::ptr::null_mut::<i8>(),
            );
        }
        (*Entry).VendorID = PCI_INVALID_VENDORID as u16;
        ExFreePoolWithTag(ValueInfo as PVOID, 0);
        return STATUS_SUCCESS;
    }

    if !NT_SUCCESS(Status) {
        RtlAssert(
            b"!NT_SUCCESS(Status)\0".as_ptr() as PVOID,
            b"\\init.c\0".as_ptr() as PVOID,
            670,
            core::ptr::null_mut::<i8>(),
        );
    }
    if !FullInfo.is_null() {
        ExFreePool(FullInfo as PVOID);
    }
    if !ValueInfo.is_null() {
        ExFreePool(ValueInfo as PVOID);
    }
    if !PciHackTable.is_null() {
        ExFreePool(PciHackTable as PVOID);
    }

    Status
}

#[no_mangle]
pub unsafe extern "system" fn PciGetDebugPorts(DebugKey: HANDLE) -> NTSTATUS {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as *const core::ffi::c_char,
        b"PciGetDebugPorts\0".as_ptr() as *const core::ffi::c_char,
        b"drivers/bus/pcix/init.c\0".as_ptr() as *const core::ffi::c_char,
        683i32,
    );
    DbgPrint(b"\0".as_ptr() as *const core::ffi::c_char);
    DbgBreakPoint();
    STATUS_SUCCESS
}

#[no_mangle]
pub unsafe extern "system" fn PciDriverUnload(DriverObject: PDRIVER_OBJECT) {
    DbgPrint(
        b"ERROR:  %s at %s:%d is unimplemented!()\n\0".as_ptr() as PCSTR,
        b"PciDriverUnload\0".as_ptr() as PCSTR,
        b"drivers/bus/pcix/init.c\0".as_ptr() as PCSTR,
        695 as i32,
    );
    DbgPrint(b"PCI: Unload\n\0".as_ptr() as PCSTR);
    DbgBreakPoint();
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: PDRIVER_OBJECT,
    RegistryPath: PUNICODE_STRING,
) -> NTSTATUS {
    let mut KeyHandle: HANDLE = core::ptr::null_mut();
    let mut ParametersKey: HANDLE = core::ptr::null_mut();
    let mut DebugKey: HANDLE = core::ptr::null_mut();
    let mut ControlSetKey: HANDLE = core::ptr::null_mut();
    let mut Status: NTSTATUS = STATUS_UNSUCCESSFUL;

    // Debug message
    DbgPrint(b"PCI: DriverEntry!\n\0".as_ptr() as PCSTR);

    loop {
        PciDriverObject = DriverObject;

        // Set up IRP dispatch functions
        (*DriverObject).MajorFunction[IRP_MJ_POWER as usize] = Some(PciDispatchIrp);
        (*DriverObject).MajorFunction[IRP_MJ_DEVICE_CONTROL as usize] = Some(PciDispatchIrp);
        (*DriverObject).MajorFunction[IRP_MJ_SYSTEM_CONTROL as usize] = Some(PciDispatchIrp);
        (*DriverObject).MajorFunction[IRP_MJ_PNP as usize] = Some(PciDispatchIrp);
        (*DriverObject).DriverUnload =
            Some(core::mem::transmute::<*const (), _>(PciDriverUnload as *const ()));

        // Set up AddDevice
        (*(*DriverObject).DriverExtension).AddDevice = Some(PciAddDevice);

        // Open the PCI key
        let mut ObjectAttributes = _OBJECT_ATTRIBUTES {
            Length: core::mem::size_of::<_OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: core::ptr::null_mut(),
            ObjectName: RegistryPath,
            Attributes: OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            SecurityDescriptor: core::ptr::null_mut(),
            SecurityQualityOfService: core::ptr::null_mut(),
        };
        Status = ZwOpenKey(
            core::ptr::addr_of_mut!(KeyHandle) as PHANDLE,
            KEY_QUERY_VALUE,
            &mut ObjectAttributes,
        );
        if !NT_SUCCESS(Status) {
            break;
        }

        // Open Parameters subkey
        let parameters_name: [u16; 11] = [0x50, 0x61, 0x72, 0x61, 0x6D, 0x65, 0x74, 0x65, 0x72, 0x73, 0x0000];
        PciOpenKey(
            parameters_name.as_ptr() as PWCHAR,
            KeyHandle,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(ParametersKey) as PHANDLE,
            &mut Status,
        );

        // Build hack table
        Status = PciBuildHackTable(ParametersKey);

        // Open Debug subkey
        let debug_name: [u16; 6] = [0x44, 0x65, 0x62, 0x75, 0x67, 0x0000];
        let Result = PciOpenKey(
            debug_name.as_ptr() as PWCHAR,
            KeyHandle,
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(DebugKey) as PHANDLE,
            &mut Status,
        );
        if Result != 0 {
            Status = PciGetDebugPorts(DebugKey);
            if !NT_SUCCESS(Status) {
                break;
            }
        }

        // Initialize synchronization events
        KeInitializeEvent(&raw mut PciGlobalLock, _EVENT_TYPE_SynchronizationEvent, 1u8);
        KeInitializeEvent(&raw mut PciBusLock, _EVENT_TYPE_SynchronizationEvent, 1u8);
        KeInitializeEvent(&raw mut PciLegacyDescriptionLock, _EVENT_TYPE_SynchronizationEvent, 1u8);

        // Open control set key
        let control_set_name: [u16; 43] = [
            0x5C, 0x52, 0x65, 0x67, 0x69, 0x73, 0x74, 0x72, 0x79, 0x5C, 0x4D, 0x61, 0x63, 0x68,
            0x69, 0x6E, 0x65, 0x5C, 0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x5C, 0x43, 0x75, 0x72,
            0x72, 0x65, 0x6E, 0x74, 0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x53, 0x65, 0x74,
            0x0000,
        ];
        let Result = PciOpenKey(
            control_set_name.as_ptr() as PWCHAR,
            core::ptr::null_mut(),
            KEY_QUERY_VALUE,
            core::ptr::addr_of_mut!(ControlSetKey) as PHANDLE,
            &mut Status,
        );
        if Result == 0 {
            break;
        }

        // Read SystemStartOptions
        let start_options_name: [u16; 19] = [
            0x53, 0x79, 0x73, 0x74, 0x65, 0x6D, 0x53, 0x74, 0x61, 0x72, 0x74, 0x4F, 0x70, 0x74,
            0x69, 0x6F, 0x6E, 0x73, 0x0000,
        ];
        let control_subkey: [u16; 8] = [0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x0000];
        let mut ResultLength: u32 = 0;
        let mut StartOptions: PWCHAR = core::ptr::null_mut();
        Status = PciGetRegistryValue(
            start_options_name.as_ptr() as PWCHAR,
            control_subkey.as_ptr() as PWCHAR,
            ControlSetKey,
            REG_SZ,
            &raw mut StartOptions as *mut PVOID,
            &mut ResultLength,
        );
        if NT_SUCCESS(Status) {
            let mut OptionString = _UNICODE_STRING {
                Length: ResultLength as u16,
                MaximumLength: ResultLength as u16,
                Buffer: StartOptions,
            };
            let mut PciLockString: _UNICODE_STRING = _UNICODE_STRING {
                Length: 0,
                MaximumLength: 0,
                Buffer: core::ptr::null_mut(),
            };
            let pci_lock_literal: [u16; 8] = [0x50, 0x43, 0x49, 0x4C, 0x4F, 0x43, 0x4B, 0x0000];
            RtlInitUnicodeString(&mut PciLockString, pci_lock_literal.as_ptr() as PCWSTR);
            if PciUnicodeStringStrStr(&mut OptionString, &mut PciLockString as PCUNICODE_STRING, 1u8) != 0 {
                PciLockDeviceResources = 1u8;
            }
            ExFreePoolWithTag(StartOptions as PVOID, 0);
        }

        // Read PCILock override
        let mut Value: PULONG = core::ptr::null_mut();
        let pci_lock_value_name: [u16; 8] = [0x50, 0x43, 0x49, 0x4C, 0x6F, 0x63, 0x6B, 0x0000];
        let bios_info_subkey: [u16; 21] = [
            0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x5C, 0x42, 0x69, 0x6F, 0x73, 0x49, 0x6E,
            0x66, 0x6F, 0x5C, 0x50, 0x43, 0x49, 0x0000,
        ];
        Status = PciGetRegistryValue(
            pci_lock_value_name.as_ptr() as PWCHAR,
            bios_info_subkey.as_ptr() as PWCHAR,
            ControlSetKey,
            REG_DWORD,
            &raw mut Value as *mut PVOID,
            &mut ResultLength,
        );
        if NT_SUCCESS(Status) {
            if ResultLength as usize == core::mem::size_of::<u32>() {
                PciLockDeviceResources = *Value as BOOLEAN;
            }
            ExFreePoolWithTag(Value as PVOID, 0);
        }

        // Read HackFlags
        let hack_flags_name: [u16; 10] = [0x48, 0x61, 0x63, 0x6B, 0x46, 0x6C, 0x61, 0x67, 0x73, 0x0000];
        let pnp_pci_subkey: [u16; 16] = [
            0x43, 0x6F, 0x6E, 0x74, 0x72, 0x6F, 0x6C, 0x5C, 0x50, 0x6E, 0x50, 0x5C, 0x50, 0x43,
            0x49, 0x0000,
        ];
        Status = PciGetRegistryValue(
            hack_flags_name.as_ptr() as PWCHAR,
            pnp_pci_subkey.as_ptr() as PWCHAR,
            ControlSetKey,
            REG_DWORD,
            &raw mut Value as *mut PVOID,
            &mut ResultLength,
        );
        if NT_SUCCESS(Status) {
            if ResultLength as usize == core::mem::size_of::<u32>() {
                PciSystemWideHackFlags = *Value;
            }
            ExFreePoolWithTag(Value as PVOID, 0);
        }

        // Read EnableNativeModeATA
        let ata_name: [u16; 20] = [
            0x45, 0x6E, 0x61, 0x62, 0x6C, 0x65, 0x4E, 0x61, 0x74, 0x69, 0x76, 0x65, 0x4D, 0x6F,
            0x64, 0x65, 0x41, 0x54, 0x41, 0x0000,
        ];
        Status = PciGetRegistryValue(
            ata_name.as_ptr() as PWCHAR,
            pnp_pci_subkey.as_ptr() as PWCHAR,
            ControlSetKey,
            REG_DWORD,
            &raw mut Value as *mut PVOID,
            &mut ResultLength,
        );
        if NT_SUCCESS(Status) {
            if ResultLength as usize == core::mem::size_of::<u32>() {
                PciEnableNativeModeATA = *Value as BOOLEAN;
            }
            ExFreePoolWithTag(Value as PVOID, 0);
        }

        // Build default exclusion lists
        Status = PciBuildDefaultExclusionLists();
        if !NT_SUCCESS(Status) {
            break;
        }

        // IRQ routing table
        PciGetIrqRoutingTableFromRegistry(&raw mut PciIrqRoutingTable);

        // Hook HAL
        PciHookHal();

        // Verifier init
        PciVerifierInit(DriverObject);

        // Check Datacenter
        PciRunningDatacenter = PciIsDatacenter();
        if PciRunningDatacenter != 0 {
            DbgPrint(b"PCI running on datacenter build\n\0".as_ptr() as PCSTR);
        }

        Status = STATUS_SUCCESS;
        break;
    }

    // Cleanup handles
    if !KeyHandle.is_null() {
        ZwClose(KeyHandle);
    }
    if !ControlSetKey.is_null() {
        ZwClose(ControlSetKey);
    }
    if !ParametersKey.is_null() {
        ZwClose(ParametersKey);
    }
    if !DebugKey.is_null() {
        ZwClose(DebugKey);
    }

    Status
}
