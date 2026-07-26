//! ACPI Component Architecture Operating System Layer (OSL) for ReactOS.
//!
//! Faithful Rust translation of drivers/bus/acpi/osl.c — the 46 `AcpiOs*`
//! callbacks ACPICA (C) calls into. ACPI types/exports come from acpica-sys;
//! kernel services from ntkrnl-sys. Calling convention: ACPI_EXTERNAL_XFACE is
//! empty for this platform, so the AcpiOs* exports use the default C convention
//! (cdecl on i386) → `#[no_mangle] extern "C"`.
//!
//! NDEBUG is set in the C driver build, so the DPRINT/DPRINT1 diagnostics and
//! the AcpiOsVprintf body compile out; they are dropped here. AcpiOsPrintf
//! (C-variadic) is a tiny C shim (acpi_printf_shim.c).

use core::ffi::c_void;
use core::ptr::{null_mut, addr_of_mut};

// Primary kernel surface (types + Zw*/Ex*/Mm*/Io*/Hal*/Ps*/Rtl*/Ke* exports).
use ntkrnl_sys::ddk::*;
// Bindgen emits enum members as `_ENUM_Variant`; alias to the friendly names.
use ntkrnl_sys::ddk::{
    _EVENT_TYPE_SynchronizationEvent as SynchronizationEvent,
    _KWAIT_REASON_Executive as Executive,
    _INTERFACE_TYPE_Internal as Internal,
    _KINTERRUPT_MODE_LevelSensitive as LevelSensitive,
    _BUS_DATA_TYPE_PCIConfiguration as PCIConfiguration,
    _KEY_INFORMATION_CLASS_KeyBasicInformation as KeyBasicInformation,
    _KEY_VALUE_INFORMATION_CLASS_KeyValuePartialInformation as KeyValuePartialInformation,
};
// Hand-written wrappers/consts (explicit → take precedence over the ddk glob).
use ntkrnl_sys::inline::common::InitializeObjectAttributes;
use ntkrnl_sys::ke::{
    KeAcquireSpinLock, KeReleaseSpinLock, KeAcquireSpinLockAtDpcLevel,
    KeReleaseSpinLockFromDpcLevel, KeGetCurrentIrql, KeQueryInterruptTime,
};
use ntkrnl_sys::mm::{NonPagedPool, MmNonCached};
use ntkrnl_sys::base::{FALSE, TRUE, STATUS_SUCCESS, DISPATCH_LEVEL};
use ntkrnl_sys::constants::{
    STATUS_NOT_FOUND, STATUS_NO_MORE_ENTRIES, STATUS_BUFFER_TOO_SMALL,
    STATUS_BUFFER_OVERFLOW, OBJ_CASE_INSENSITIVE, OBJ_KERNEL_HANDLE, KEY_READ,
    REG_SZ, IO_NO_INCREMENT, THREAD_ALL_ACCESS, KernelMode,
};

// Header #defines bindgen could not surface here:
const CmResourceTypeDeviceSpecific: u8 = 5; // wdk: CmResourceTypeDeviceSpecific
const PCI_INVALID_VENDORID: u32 = 0xFFFF;

use acpica_sys::{
    ACPI_STATUS, ACPI_PHYSICAL_ADDRESS, ACPI_SIZE, ACPI_IO_ADDRESS,
    ACPI_OSD_HANDLER, ACPI_OSD_EXEC_CALLBACK, ACPI_EXECUTE_TYPE,
    ACPI_PCI_ID, ACPI_TABLE_HEADER, ACPI_TABLE_RSDP, ACPI_PREDEFINED_NAMES,
    ACPI_STRING,
};

// ---- ACPICA status / constant values (acexcep.h, actypes.h) ----------------
// AE_* are cast-style #defines bindgen can't evaluate; define them explicitly.
const AE_OK: ACPI_STATUS = 0x0000;
const AE_ERROR: ACPI_STATUS = 0x0001; // EXCEP_ENV(0x0001)
const AE_NO_MEMORY: ACPI_STATUS = 0x0004;
const AE_NOT_FOUND: ACPI_STATUS = 0x0005;
const AE_NOT_EXIST: ACPI_STATUS = 0x0006;
const AE_ALREADY_EXISTS: ACPI_STATUS = 0x0007;
const AE_TIME: ACPI_STATUS = 0x0011;
const AE_BAD_PARAMETER: ACPI_STATUS = 0x1001; // EXCEP_PGM(0x0001) = 0x1000|0x0001

const ACPI_DO_NOT_WAIT: u16 = 0;
const ACPI_INTERRUPT_HANDLED: u32 = 1;
const ACPI_SIGNAL_FATAL: u32 = 0;
const ACPI_SIGNAL_BREAKPOINT: u32 = 1;
const ACPI_OEM_ID_SIZE: usize = 6;

// Pool tags — chosen so the bytes read "Acpi"/"Aacp"/"AcpL" in memory, matching
// the C multi-char constants 'ipcA' / 'pcaA' / 'LpcA' respectively.
const TAG_ACPI: u32 = u32::from_le_bytes(*b"Acpi"); // osl allocations  ('ipcA')
const TAG_AACP: u32 = u32::from_le_bytes(*b"Aacp"); // registry scratch ('pcaA')
const TAG_ACPL: u32 = u32::from_le_bytes(*b"AcpL"); // locks/mutex/sem  ('LpcA')

#[inline]
fn nt_success(s: NTSTATUS) -> bool {
    s >= 0
}

// LARGE_INTEGER / PHYSICAL_ADDRESS are 8 bytes; transmute to/from i64 to avoid
// the bindgen union field-path dance.
#[inline]
unsafe fn make_li(v: i64) -> LARGE_INTEGER {
    core::mem::transmute::<i64, LARGE_INTEGER>(v)
}
#[inline]
unsafe fn li_quad(v: LARGE_INTEGER) -> i64 {
    core::mem::transmute::<LARGE_INTEGER, i64>(v)
}

// ExInitializeFastMutex is FORCEINLINE (no ntoskrnl export); inline it.
#[inline]
unsafe fn init_fast_mutex(fm: PFAST_MUTEX) {
    (*fm).Count = 1; // FM_LOCK_BIT — available
    (*fm).Owner = null_mut();
    (*fm).Contention = 0;
    KeInitializeEvent(addr_of_mut!((*fm).Event), SynchronizationEvent, FALSE as BOOLEAN);
}

// ---- OSL interrupt state ---------------------------------------------------
static mut AcpiInterrupt: PKINTERRUPT = null_mut();
static mut AcpiInterruptHandlerRegistered: BOOLEAN = FALSE as BOOLEAN;
static mut AcpiIrqHandler: ACPI_OSD_HANDLER = None;
static mut AcpiIrqContext: PVOID = null_mut();
static mut AcpiIrqNumber: ULONG = 0;

#[no_mangle]
pub unsafe extern "C" fn AcpiOsInitialize() -> ACPI_STATUS {
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsTerminate() -> ACPI_STATUS {
    AE_OK
}

// ---- UEFI fallback for AcpiOsGetRootPointer --------------------------------
static mut AcpipSyntheticRsdp: ACPI_TABLE_RSDP = unsafe { core::mem::zeroed() };
static mut AcpipCachedRootPointer: ACPI_PHYSICAL_ADDRESS = 0;
static mut AcpipRootPointerComputed: BOOLEAN = FALSE as BOOLEAN;

/// Read the RSDT/XSDT physical address from the freeldr-registered "ACPI BIOS"
/// multifunction-adapter Configuration Data in the registry.
unsafe fn AcpipGetRsdtAddressFromRegistry(out: *mut PHYSICAL_ADDRESS) -> NTSTATUS {
    // L"\\Registry\\Machine\\HARDWARE\\DESCRIPTION\\System\\MultifunctionAdapter"
    const MCFG_PATH: &[u16] = &[
        0x5c, b'R' as u16, b'e' as u16, b'g' as u16, b'i' as u16, b's' as u16,
        b't' as u16, b'r' as u16, b'y' as u16, 0x5c, b'M' as u16, b'a' as u16,
        b'c' as u16, b'h' as u16, b'i' as u16, b'n' as u16, b'e' as u16, 0x5c,
        b'H' as u16, b'A' as u16, b'R' as u16, b'D' as u16, b'W' as u16, b'A' as u16,
        b'R' as u16, b'E' as u16, 0x5c, b'D' as u16, b'E' as u16, b'S' as u16,
        b'C' as u16, b'R' as u16, b'I' as u16, b'P' as u16, b'T' as u16, b'I' as u16,
        b'O' as u16, b'N' as u16, 0x5c, b'S' as u16, b'y' as u16, b's' as u16,
        b't' as u16, b'e' as u16, b'm' as u16, 0x5c, b'M' as u16, b'u' as u16,
        b'l' as u16, b't' as u16, b'i' as u16, b'f' as u16, b'u' as u16, b'n' as u16,
        b'c' as u16, b't' as u16, b'i' as u16, b'o' as u16, b'n' as u16, b'A' as u16,
        b'd' as u16, b'a' as u16, b'p' as u16, b't' as u16, b'e' as u16, b'r' as u16,
        0,
    ];
    // L"ACPI BIOS" + NUL
    const ACPI_BIOS_ID: &[u16] = &[
        b'A' as u16, b'C' as u16, b'P' as u16, b'I' as u16, b' ' as u16,
        b'B' as u16, b'I' as u16, b'O' as u16, b'S' as u16, 0,
    ];

    (*out).QuadPart = 0;

    let mut key_name: UNICODE_STRING = core::mem::zeroed();
    RtlInitUnicodeString(&mut key_name, MCFG_PATH.as_ptr());

    let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    InitializeObjectAttributes(
        &mut oa,
        &mut key_name,
        OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
        null_mut(),
        null_mut(),
    );

    let mut root_key: HANDLE = null_mut();
    let mut status = ZwOpenKey(&mut root_key, KEY_READ, &mut oa);
    if !nt_success(status) {
        return status;
    }

    let mut buffer: PVOID = null_mut();
    let mut ret_status: NTSTATUS = STATUS_NOT_FOUND;
    let mut result_length: ULONG = 0;

    // Walk the numbered subkeys "0", "1", "2", ...
    let mut index: ULONG = 0;
    loop {
        // KEY_BASIC_INFORMATION + 16 WCHARs of name.
        let mut basic_buf: [u8; core::mem::size_of::<KEY_BASIC_INFORMATION>() + 16 * 2] =
            [0; core::mem::size_of::<KEY_BASIC_INFORMATION>() + 16 * 2];
        let basic_info = basic_buf.as_mut_ptr() as *mut KEY_BASIC_INFORMATION;

        status = ZwEnumerateKey(
            root_key,
            index,
            KeyBasicInformation,
            basic_info as PVOID,
            basic_buf.len() as ULONG,
            &mut result_length,
        );
        if status == STATUS_NO_MORE_ENTRIES {
            break;
        }
        if !nt_success(status) {
            break;
        }

        let name_len = (*basic_info).NameLength;
        if name_len == 0 || (name_len as usize) >= 16 * 2 {
            index += 1;
            continue;
        }

        // Build a NUL-terminated copy of the subkey name.
        let mut subkey_name_buf: [u16; 16] = [0; 16];
        core::ptr::copy_nonoverlapping(
            addr_of_mut!((*basic_info).Name) as *const u8,
            subkey_name_buf.as_mut_ptr() as *mut u8,
            name_len as usize,
        );
        subkey_name_buf[name_len as usize / 2] = 0;

        let mut subkey_name: UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(&mut subkey_name, subkey_name_buf.as_ptr());

        InitializeObjectAttributes(
            &mut oa,
            &mut subkey_name,
            OBJ_CASE_INSENSITIVE | OBJ_KERNEL_HANDLE,
            root_key,
            null_mut(),
        );

        let mut subkey: HANDLE = null_mut();
        status = ZwOpenKey(&mut subkey, KEY_READ, &mut oa);
        if !nt_success(status) {
            index += 1;
            continue;
        }

        // Query "Identifier" and check it equals "ACPI BIOS".
        // L"Identifier"
        let id_name_buf: [u16; 11] = [
            b'I' as u16, b'd' as u16, b'e' as u16, b'n' as u16, b't' as u16,
            b'i' as u16, b'f' as u16, b'i' as u16, b'e' as u16, b'r' as u16, 0,
        ];
        let mut value_name: UNICODE_STRING = core::mem::zeroed();
        RtlInitUnicodeString(&mut value_name, id_name_buf.as_ptr());

        status = ZwQueryValueKey(
            subkey,
            &mut value_name,
            KeyValuePartialInformation,
            null_mut(),
            0,
            &mut result_length,
        );
        if status == STATUS_BUFFER_TOO_SMALL || status == STATUS_BUFFER_OVERFLOW {
            let id_info =
                ExAllocatePoolWithTag(NonPagedPool, result_length as SIZE_T, TAG_AACP)
                    as *mut KEY_VALUE_PARTIAL_INFORMATION;
            if !id_info.is_null() {
                status = ZwQueryValueKey(
                    subkey,
                    &mut value_name,
                    KeyValuePartialInformation,
                    id_info as PVOID,
                    result_length,
                    &mut result_length,
                );
                let acpi_bios_bytes = ACPI_BIOS_ID.len() * 2;
                if nt_success(status)
                    && (*id_info).Type == REG_SZ
                    && (*id_info).DataLength as usize >= acpi_bios_bytes
                {
                    let id_str = addr_of_mut!((*id_info).Data) as *const u16;
                    if wstr_eq(id_str, ACPI_BIOS_ID.as_ptr(), ACPI_BIOS_ID.len()) {
                        ExFreePoolWithTag(id_info as PVOID, TAG_AACP);

                        // Found the ACPI BIOS adapter. Read Configuration Data.
                        // L"Configuration Data"
                        let cfg_name_buf: [u16; 19] = [
                            b'C' as u16, b'o' as u16, b'n' as u16, b'f' as u16,
                            b'i' as u16, b'g' as u16, b'u' as u16, b'r' as u16,
                            b'a' as u16, b't' as u16, b'i' as u16, b'o' as u16,
                            b'n' as u16, b' ' as u16, b'D' as u16, b'a' as u16,
                            b't' as u16, b'a' as u16, 0,
                        ];
                        RtlInitUnicodeString(&mut value_name, cfg_name_buf.as_ptr());
                        status = ZwQueryValueKey(
                            subkey,
                            &mut value_name,
                            KeyValuePartialInformation,
                            null_mut(),
                            0,
                            &mut result_length,
                        );
                        if status == STATUS_BUFFER_TOO_SMALL
                            || status == STATUS_BUFFER_OVERFLOW
                        {
                            let buffer_size = result_length;
                            buffer = ExAllocatePoolWithTag(
                                NonPagedPool,
                                buffer_size as SIZE_T,
                                TAG_AACP,
                            );
                            if !buffer.is_null() {
                                status = ZwQueryValueKey(
                                    subkey,
                                    &mut value_name,
                                    KeyValuePartialInformation,
                                    buffer,
                                    buffer_size,
                                    &mut result_length,
                                );
                                if nt_success(status) {
                                    ret_status = STATUS_SUCCESS;
                                }
                            }
                        }

                        ZwClose(subkey);
                        break;
                    }
                }

                ExFreePoolWithTag(id_info as PVOID, TAG_AACP);
            }
        }

        ZwClose(subkey);
        index += 1;
    }

    ZwClose(root_key);

    if !nt_success(ret_status) || buffer.is_null() {
        if !buffer.is_null() {
            ExFreePoolWithTag(buffer, TAG_AACP);
        }
        return STATUS_NOT_FOUND;
    }

    // Parse the Configuration Data blob → CM_FULL_RESOURCE_DESCRIPTOR; find the
    // CmResourceTypeDeviceSpecific descriptor; freeldr's ACPI_BIOS_DATA begins
    // immediately after it, with PHYSICAL_ADDRESS RSDTAddress as its first field.
    let value_info = buffer as *mut KEY_VALUE_PARTIAL_INFORMATION;
    ret_status = STATUS_NOT_FOUND;

    let data_length = (*value_info).DataLength;
    let data_start = addr_of_mut!((*value_info).Data) as *mut u8;
    let data_end = data_start.add(data_length as usize);

    if (data_length as usize) < core::mem::size_of::<CM_FULL_RESOURCE_DESCRIPTOR>() {
        ExFreePoolWithTag(buffer, TAG_AACP);
        return STATUS_NOT_FOUND;
    }

    let full_desc = data_start as *mut CM_FULL_RESOURCE_DESCRIPTOR;
    let partial_list = addr_of_mut!((*full_desc).PartialResourceList);
    let count = (*partial_list).Count;

    let mut i: ULONG = 0;
    while i < count {
        let desc = addr_of_mut!((*partial_list).PartialDescriptors).cast::<CM_PARTIAL_RESOURCE_DESCRIPTOR>().add(i as usize);
        let desc_bytes = desc as *mut u8;

        if desc_bytes.add(core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>()) > data_end {
            break;
        }

        if (*desc).Type == CmResourceTypeDeviceSpecific as UCHAR {
            let acpi_bios_data =
                desc_bytes.add(core::mem::size_of::<CM_PARTIAL_RESOURCE_DESCRIPTOR>());
            if acpi_bios_data.add(core::mem::size_of::<PHYSICAL_ADDRESS>()) <= data_end {
                let mut rsdt_address: PHYSICAL_ADDRESS = core::mem::zeroed();
                core::ptr::copy_nonoverlapping(
                    acpi_bios_data,
                    &mut rsdt_address as *mut PHYSICAL_ADDRESS as *mut u8,
                    core::mem::size_of::<PHYSICAL_ADDRESS>(),
                );
                *out = rsdt_address;
                ret_status = STATUS_SUCCESS;
            }
            break;
        }
        i += 1;
    }

    ExFreePoolWithTag(buffer, TAG_AACP);
    ret_status
}

/// Case-sensitive wide-string equality over `len` u16 code units.
#[inline]
unsafe fn wstr_eq(a: *const u16, b: *const u16, len: usize) -> bool {
    for i in 0..len {
        if *a.add(i) != *b.add(i) {
            return false;
        }
    }
    true
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsGetRootPointer() -> ACPI_PHYSICAL_ADDRESS {
    if AcpipRootPointerComputed != 0 {
        return AcpipCachedRootPointer;
    }

    // 1) Legacy BIOS path: scan the BIOS area.
    let mut pa: ACPI_PHYSICAL_ADDRESS = 0;
    acpica_sys::AcpiFindRootPointer(&mut pa);
    if pa != 0 {
        AcpipCachedRootPointer = pa;
        AcpipRootPointerComputed = TRUE as BOOLEAN;
        return pa;
    }

    // 2) UEFI path: BIOS scan failed → registry RSDT/XSDT + synthesize an RSDP.
    let mut rsdt_address: PHYSICAL_ADDRESS = core::mem::zeroed();
    let status = AcpipGetRsdtAddressFromRegistry(&mut rsdt_address);
    if !nt_success(status) || rsdt_address.QuadPart == 0 {
        AcpipCachedRootPointer = 0;
        AcpipRootPointerComputed = TRUE as BOOLEAN;
        return 0;
    }

    // Synthesize a valid ACPI 2.0+ RSDP pointing at the XSDT.
    let rsdp = addr_of_mut!(AcpipSyntheticRsdp);
    core::ptr::write_bytes(rsdp, 0, 1);
    core::ptr::copy_nonoverlapping(b"RSD PTR ".as_ptr(), addr_of_mut!((*rsdp).Signature) as *mut u8, 8);
    core::ptr::copy_nonoverlapping(b"REACTS".as_ptr(), addr_of_mut!((*rsdp).OemId) as *mut u8, ACPI_OEM_ID_SIZE);
    (*rsdp).Revision = 2;
    (*rsdp).Length = core::mem::size_of::<ACPI_TABLE_RSDP>() as u32;
    (*rsdp).RsdtPhysicalAddress = 0;
    (*rsdp).XsdtPhysicalAddress = rsdt_address.QuadPart as u64;

    // ACPI 1.0 checksum: unsigned 8-bit sum of first 20 bytes must be 0.
    (*rsdp).Checksum = 0;
    let bytes = rsdp as *const u8;
    let mut sum: u32 = 0;
    for i in 0..20usize {
        sum += *bytes.add(i) as u32;
    }
    (*rsdp).Checksum = (0u32.wrapping_sub(sum & 0xFF)) as u8;

    // Extended checksum: unsigned 8-bit sum of all 36 bytes must be 0.
    (*rsdp).ExtendedChecksum = 0;
    sum = 0;
    for i in 0..core::mem::size_of::<ACPI_TABLE_RSDP>() {
        sum += *bytes.add(i) as u32;
    }
    (*rsdp).ExtendedChecksum = (0u32.wrapping_sub(sum & 0xFF)) as u8;

    pa = li_quad(MmGetPhysicalAddress(rsdp as PVOID)) as ACPI_PHYSICAL_ADDRESS;

    AcpipCachedRootPointer = pa;
    AcpipRootPointerComputed = TRUE as BOOLEAN;
    pa
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsPredefinedOverride(
    PredefinedObject: *const ACPI_PREDEFINED_NAMES,
    NewValue: *mut ACPI_STRING,
) -> ACPI_STATUS {
    if PredefinedObject.is_null() || NewValue.is_null() {
        return AE_BAD_PARAMETER;
    }
    core::ptr::write_volatile(NewValue, null_mut());
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsTableOverride(
    ExistingTable: *mut ACPI_TABLE_HEADER,
    NewTable: *mut *mut ACPI_TABLE_HEADER,
) -> ACPI_STATUS {
    if ExistingTable.is_null() || NewTable.is_null() {
        return AE_BAD_PARAMETER;
    }
    core::ptr::write_volatile(NewTable, null_mut());
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsPhysicalTableOverride(
    ExistingTable: *mut ACPI_TABLE_HEADER,
    NewAddress: *mut ACPI_PHYSICAL_ADDRESS,
    NewTableLength: *mut UINT32,
) -> ACPI_STATUS {
    if ExistingTable.is_null() || NewAddress.is_null() || NewTableLength.is_null() {
        return AE_BAD_PARAMETER;
    }
    core::ptr::write_volatile(NewAddress, 0);
    core::ptr::write_volatile(NewTableLength, 0);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsMapMemory(
    Where: ACPI_PHYSICAL_ADDRESS,
    Length: ACPI_SIZE,
) -> *mut c_void {
    let address = make_li(Where as i64);
    MmMapIoSpace(address, Length as SIZE_T, MmNonCached) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsUnmapMemory(LogicalAddress: *mut c_void, Size: ACPI_SIZE) {
    MmUnmapIoSpace(LogicalAddress as PVOID, Size as SIZE_T);
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsGetPhysicalAddress(
    LogicalAddress: *mut c_void,
    PhysicalAddress: *mut ACPI_PHYSICAL_ADDRESS,
) -> ACPI_STATUS {
    if LogicalAddress.is_null() || PhysicalAddress.is_null() {
        return AE_BAD_PARAMETER;
    }
    let phys = MmGetPhysicalAddress(LogicalAddress as PVOID);
    core::ptr::write_volatile(PhysicalAddress, li_quad(phys) as ACPI_PHYSICAL_ADDRESS);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsAllocate(Size: ACPI_SIZE) -> *mut c_void {
    ExAllocatePoolWithTag(NonPagedPool, Size as SIZE_T, TAG_ACPI) as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsFree(Memory: *mut c_void) {
    ExFreePoolWithTag(Memory as PVOID, TAG_ACPI);
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsReadable(_Pointer: *mut c_void, _Length: ACPI_SIZE) -> BOOLEAN {
    // C probed via ProbeForRead inside _SEH2_TRY, but the _SEH2_EXCEPT guard is
    // dead code under DUMMY_PSEH (an unprobed fault would bugcheck either way),
    // and ProbeForRead is a DDK macro with no kernel export. ACPICA calls these
    // advisory validators only on debug paths → report readable.
    TRUE as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsWritable(_Pointer: *mut c_void, _Length: ACPI_SIZE) -> BOOLEAN {
    TRUE as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsGetThreadId() -> UINT64 {
    // Thread ID must be non-zero.
    (PsGetCurrentThreadId() as usize as u64) + 1
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsExecute(
    _Type: ACPI_EXECUTE_TYPE,
    Function: ACPI_OSD_EXEC_CALLBACK,
    Context: *mut c_void,
) -> ACPI_STATUS {
    let mut oa: OBJECT_ATTRIBUTES = core::mem::zeroed();
    InitializeObjectAttributes(&mut oa, null_mut(), OBJ_KERNEL_HANDLE, null_mut(), null_mut());

    let mut thread_handle: HANDLE = null_mut();
    let start: PKSTART_ROUTINE = core::mem::transmute(Function);
    let status = PsCreateSystemThread(
        &mut thread_handle,
        THREAD_ALL_ACCESS,
        &mut oa,
        null_mut(),
        null_mut(),
        start,
        Context as PVOID,
    );
    if !nt_success(status) {
        return AE_ERROR;
    }
    ZwClose(thread_handle);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsSleep(Milliseconds: UINT64) {
    KeStallExecutionProcessor((Milliseconds * 1000) as ULONG);
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsStall(Microseconds: UINT32) {
    KeStallExecutionProcessor(Microseconds);
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsCreateMutex(OutHandle: *mut *mut c_void) -> ACPI_STATUS {
    if OutHandle.is_null() {
        return AE_BAD_PARAMETER;
    }
    let mutex = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<FAST_MUTEX>() as SIZE_T, TAG_ACPL)
        as PFAST_MUTEX;
    if mutex.is_null() {
        return AE_NO_MEMORY;
    }
    init_fast_mutex(mutex);
    core::ptr::write_volatile(OutHandle, mutex as *mut c_void);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsDeleteMutex(Handle: *mut c_void) {
    if Handle.is_null() {
        return;
    }
    ExFreePoolWithTag(Handle as PVOID, TAG_ACPL);
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsAcquireMutex(Handle: *mut c_void, Timeout: UINT16) -> ACPI_STATUS {
    if Handle.is_null() {
        return AE_BAD_PARAMETER;
    }
    if Timeout == ACPI_DO_NOT_WAIT {
        if ExTryToAcquireFastMutex(Handle as PFAST_MUTEX) == 0 {
            return AE_TIME;
        }
    } else {
        ExAcquireFastMutex(Handle as PFAST_MUTEX);
    }
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsReleaseMutex(Handle: *mut c_void) {
    if Handle.is_null() {
        return;
    }
    ExReleaseFastMutex(Handle as PFAST_MUTEX);
}

#[repr(C)]
struct ACPI_SEM {
    CurrentUnits: UINT32,
    Event: KEVENT,
    Lock: KSPIN_LOCK,
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsCreateSemaphore(
    _MaxUnits: UINT32,
    InitialUnits: UINT32,
    OutHandle: *mut *mut c_void,
) -> ACPI_STATUS {
    if OutHandle.is_null() {
        return AE_BAD_PARAMETER;
    }
    let sem = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<ACPI_SEM>() as SIZE_T, TAG_ACPL)
        as *mut ACPI_SEM;
    if sem.is_null() {
        return AE_NO_MEMORY;
    }
    (*sem).CurrentUnits = InitialUnits;
    KeInitializeEvent(
        addr_of_mut!((*sem).Event),
        SynchronizationEvent,
        ((*sem).CurrentUnits != 0) as BOOLEAN,
    );
    // KeInitializeSpinLock just zeroes the lock (inline on amd64, i386 export has
    // a decoration mismatch in the bindgen decl) — write it directly.
    (*sem).Lock = 0;
    core::ptr::write_volatile(OutHandle, sem as *mut c_void);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsDeleteSemaphore(Handle: *mut c_void) -> ACPI_STATUS {
    if Handle.is_null() {
        return AE_BAD_PARAMETER;
    }
    ExFreePoolWithTag(Handle as PVOID, TAG_ACPL);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsWaitSemaphore(
    Handle: *mut c_void,
    Units: UINT32,
    Timeout: UINT16,
) -> ACPI_STATUS {
    if Handle.is_null() {
        return AE_BAD_PARAMETER;
    }
    let sem = Handle as *mut ACPI_SEM;
    let mut old_irql: KIRQL = 0;

    KeAcquireSpinLock(addr_of_mut!((*sem).Lock).cast(), &mut old_irql);

    if Timeout == ACPI_DO_NOT_WAIT && (*sem).CurrentUnits < Units {
        KeReleaseSpinLock(addr_of_mut!((*sem).Lock).cast(), old_irql);
        return AE_TIME;
    }

    while (*sem).CurrentUnits < Units {
        KeReleaseSpinLock(addr_of_mut!((*sem).Lock).cast(), old_irql);
        KeWaitForSingleObject(
            addr_of_mut!((*sem).Event) as PVOID,
            Executive,
            KernelMode as i8,
            FALSE as BOOLEAN,
            null_mut(),
        );
        KeAcquireSpinLock(addr_of_mut!((*sem).Lock).cast(), &mut old_irql);
    }

    (*sem).CurrentUnits -= Units;

    if (*sem).CurrentUnits != 0 {
        KeSetEvent(addr_of_mut!((*sem).Event), IO_NO_INCREMENT as i32, FALSE as BOOLEAN);
    }

    KeReleaseSpinLock(addr_of_mut!((*sem).Lock).cast(), old_irql);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsSignalSemaphore(Handle: *mut c_void, Units: UINT32) -> ACPI_STATUS {
    if Handle.is_null() {
        return AE_BAD_PARAMETER;
    }
    let sem = Handle as *mut ACPI_SEM;
    let mut old_irql: KIRQL = 0;

    KeAcquireSpinLock(addr_of_mut!((*sem).Lock).cast(), &mut old_irql);
    (*sem).CurrentUnits += Units;
    KeSetEvent(addr_of_mut!((*sem).Event), IO_NO_INCREMENT as i32, FALSE as BOOLEAN);
    KeReleaseSpinLock(addr_of_mut!((*sem).Lock).cast(), old_irql);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsCreateLock(OutHandle: *mut *mut c_void) -> ACPI_STATUS {
    if OutHandle.is_null() {
        return AE_BAD_PARAMETER;
    }
    let spinlock = ExAllocatePoolWithTag(NonPagedPool, core::mem::size_of::<KSPIN_LOCK>() as SIZE_T, TAG_ACPL)
        as PKSPIN_LOCK;
    if spinlock.is_null() {
        return AE_NO_MEMORY;
    }
    *spinlock = 0; // KeInitializeSpinLock — just zero the lock (see above)
    core::ptr::write_volatile(OutHandle, spinlock as *mut c_void);
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsDeleteLock(Handle: *mut c_void) {
    if Handle.is_null() {
        return;
    }
    ExFreePoolWithTag(Handle as PVOID, TAG_ACPL);
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsAcquireLock(Handle: *mut c_void) -> ACPI_SIZE {
    let old_irql = KeGetCurrentIrql();
    if old_irql >= DISPATCH_LEVEL {
        KeAcquireSpinLockAtDpcLevel(Handle.cast());
        old_irql as ACPI_SIZE
    } else {
        let mut acquired_irql: KIRQL = 0;
        KeAcquireSpinLock(Handle.cast(), &mut acquired_irql);
        acquired_irql as ACPI_SIZE
    }
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsReleaseLock(Handle: *mut c_void, Flags: ACPI_SIZE) {
    let old_irql = Flags as KIRQL;
    if old_irql >= DISPATCH_LEVEL {
        KeReleaseSpinLockFromDpcLevel(Handle.cast());
    } else {
        KeReleaseSpinLock(Handle.cast(), old_irql);
    }
}

unsafe extern "system" fn OslIsrStub(_Interrupt: PKINTERRUPT, _ServiceContext: PVOID) -> BOOLEAN {
    let status = match AcpiIrqHandler {
        Some(h) => h(AcpiIrqContext),
        None => 0,
    };
    if status == ACPI_INTERRUPT_HANDLED {
        TRUE as BOOLEAN
    } else {
        FALSE as BOOLEAN
    }
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsInstallInterruptHandler(
    InterruptNumber: UINT32,
    ServiceRoutine: ACPI_OSD_HANDLER,
    Context: *mut c_void,
) -> ACPI_STATUS {
    if AcpiInterruptHandlerRegistered != 0 {
        return AE_ALREADY_EXISTS;
    }
    if ServiceRoutine.is_none() {
        return AE_BAD_PARAMETER;
    }

    let mut dirql: KIRQL = 0;
    let mut affinity: KAFFINITY = 0;
    let vector = HalGetInterruptVector(
        Internal,
        0,
        InterruptNumber,
        InterruptNumber,
        &mut dirql,
        &mut affinity,
    );

    AcpiIrqNumber = InterruptNumber;
    AcpiIrqHandler = ServiceRoutine;
    AcpiIrqContext = Context as PVOID;
    AcpiInterruptHandlerRegistered = TRUE as BOOLEAN;

    // OslIsrStub is extern "system" (NTAPI — how the kernel invokes the ISR);
    // PKSERVICE_ROUTINE is a per-arch concrete-ABI fn-ptr type, so bridge by
    // transmute (same ABI, distinct Rust fn type).
    let service: PKSERVICE_ROUTINE = core::mem::transmute::<
        unsafe extern "system" fn(PKINTERRUPT, PVOID) -> BOOLEAN,
        PKSERVICE_ROUTINE,
    >(OslIsrStub);
    let status = IoConnectInterrupt(
        addr_of_mut!(AcpiInterrupt),
        service,
        null_mut(),
        null_mut(),
        vector,
        dirql,
        dirql,
        LevelSensitive,
        TRUE as BOOLEAN,
        affinity,
        FALSE as BOOLEAN,
    );

    if !nt_success(status) {
        return AE_ERROR;
    }
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsRemoveInterruptHandler(
    _InterruptNumber: UINT32,
    ServiceRoutine: ACPI_OSD_HANDLER,
) -> ACPI_STATUS {
    if ServiceRoutine.is_none() {
        return AE_BAD_PARAMETER;
    }
    if AcpiInterruptHandlerRegistered != 0 {
        IoDisconnectInterrupt(AcpiInterrupt);
        AcpiInterrupt = null_mut();
        AcpiInterruptHandlerRegistered = FALSE as BOOLEAN;
    } else {
        return AE_NOT_EXIST;
    }
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsReadMemory(
    Address: ACPI_PHYSICAL_ADDRESS,
    Value: *mut UINT64,
    Width: UINT32,
) -> ACPI_STATUS {
    let a = Address as usize;
    match Width {
        8 => core::ptr::write_volatile(Value, *(a as *const u8) as u64),
        16 => core::ptr::write_volatile(Value, *(a as *const u16) as u64),
        32 => core::ptr::write_volatile(Value, *(a as *const u32) as u64),
        64 => core::ptr::write_volatile(Value, *(a as *const u64)),
        _ => return AE_BAD_PARAMETER,
    }
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsWriteMemory(
    Address: ACPI_PHYSICAL_ADDRESS,
    Value: UINT64,
    Width: UINT32,
) -> ACPI_STATUS {
    let a = Address as usize;
    match Width {
        8 => *(a as *mut u8) = Value as u8,
        16 => *(a as *mut u16) = Value as u16,
        32 => *(a as *mut u32) = Value as u32,
        64 => *(a as *mut u64) = Value,
        _ => return AE_BAD_PARAMETER,
    }
    AE_OK
}

// Port I/O. On i386 + arm64 READ/WRITE_PORT_* are HAL exports, but on amd64 they
// are inline __in/__outbyte intrinsics (no export). So use x86 IN/OUT inline asm
// (matching hal-rust's pattern) on x86/amd64, and the HAL extern on arm64.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod portio {
    use core::arch::asm;
    #[inline] pub unsafe fn r8(p: u16) -> u8 { let v: u8; asm!("in al, dx", out("al") v, in("dx") p, options(nomem, nostack, preserves_flags)); v }
    #[inline] pub unsafe fn r16(p: u16) -> u16 { let v: u16; asm!("in ax, dx", out("ax") v, in("dx") p, options(nomem, nostack, preserves_flags)); v }
    #[inline] pub unsafe fn r32(p: u16) -> u32 { let v: u32; asm!("in eax, dx", out("eax") v, in("dx") p, options(nomem, nostack, preserves_flags)); v }
    #[inline] pub unsafe fn w8(p: u16, v: u8) { asm!("out dx, al", in("dx") p, in("al") v, options(nomem, nostack, preserves_flags)); }
    #[inline] pub unsafe fn w16(p: u16, v: u16) { asm!("out dx, ax", in("dx") p, in("ax") v, options(nomem, nostack, preserves_flags)); }
    #[inline] pub unsafe fn w32(p: u16, v: u32) { asm!("out dx, eax", in("dx") p, in("eax") v, options(nomem, nostack, preserves_flags)); }
}

#[cfg(target_arch = "aarch64")]
extern "system" {
    fn READ_PORT_UCHAR(Port: *mut UCHAR) -> UCHAR;
    fn READ_PORT_USHORT(Port: *mut USHORT) -> USHORT;
    fn READ_PORT_ULONG(Port: *mut ULONG) -> ULONG;
    fn WRITE_PORT_UCHAR(Port: *mut UCHAR, Value: UCHAR);
    fn WRITE_PORT_USHORT(Port: *mut USHORT, Value: USHORT);
    fn WRITE_PORT_ULONG(Port: *mut ULONG, Value: ULONG);
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline] unsafe fn port_r8(a: usize) -> u8 { portio::r8(a as u16) }
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline] unsafe fn port_r16(a: usize) -> u16 { portio::r16(a as u16) }
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline] unsafe fn port_r32(a: usize) -> u32 { portio::r32(a as u16) }
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline] unsafe fn port_w8(a: usize, v: u8) { portio::w8(a as u16, v) }
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline] unsafe fn port_w16(a: usize, v: u16) { portio::w16(a as u16, v) }
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline] unsafe fn port_w32(a: usize, v: u32) { portio::w32(a as u16, v) }

#[cfg(target_arch = "aarch64")]
#[inline] unsafe fn port_r8(a: usize) -> u8 { READ_PORT_UCHAR(a as *mut UCHAR) }
#[cfg(target_arch = "aarch64")]
#[inline] unsafe fn port_r16(a: usize) -> u16 { READ_PORT_USHORT(a as *mut USHORT) }
#[cfg(target_arch = "aarch64")]
#[inline] unsafe fn port_r32(a: usize) -> u32 { READ_PORT_ULONG(a as *mut ULONG) }
#[cfg(target_arch = "aarch64")]
#[inline] unsafe fn port_w8(a: usize, v: u8) { WRITE_PORT_UCHAR(a as *mut UCHAR, v) }
#[cfg(target_arch = "aarch64")]
#[inline] unsafe fn port_w16(a: usize, v: u16) { WRITE_PORT_USHORT(a as *mut USHORT, v) }
#[cfg(target_arch = "aarch64")]
#[inline] unsafe fn port_w32(a: usize, v: u32) { WRITE_PORT_ULONG(a as *mut ULONG, v) }

#[no_mangle]
pub unsafe extern "C" fn AcpiOsReadPort(
    Address: ACPI_IO_ADDRESS,
    Value: *mut UINT32,
    Width: UINT32,
) -> ACPI_STATUS {
    let a = Address as usize;
    match Width {
        8 => core::ptr::write_volatile(Value, port_r8(a) as u32),
        16 => core::ptr::write_volatile(Value, port_r16(a) as u32),
        32 => core::ptr::write_volatile(Value, port_r32(a)),
        _ => return AE_BAD_PARAMETER,
    }
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsWritePort(
    Address: ACPI_IO_ADDRESS,
    Value: UINT32,
    Width: UINT32,
) -> ACPI_STATUS {
    let a = Address as usize;
    match Width {
        8 => port_w8(a, Value as u8),
        16 => port_w16(a, Value as u16),
        32 => port_w32(a, Value),
        _ => return AE_BAD_PARAMETER,
    }
    AE_OK
}

/// Detect PCI device presence by reading the PCI configuration space.
unsafe fn OslIsPciDevicePresent(bus_number: ULONG, slot_number: ULONG) -> bool {
    // bindgen flattens _PCI_COMMON_CONFIG to an opaque blob, so use a raw 256-byte
    // PCI config buffer; VendorID is the little-endian USHORT at offset 0.
    let mut pci_config = [0u8; 256];
    let read_length = HalGetBusDataByOffset(
        PCIConfiguration,
        bus_number,
        slot_number,
        pci_config.as_mut_ptr() as PVOID,
        0,
        pci_config.len() as ULONG,
    );
    if read_length == 0 {
        return false;
    }
    let vendor_id = u16::from_le_bytes([pci_config[0], pci_config[1]]);
    if vendor_id as u32 == PCI_INVALID_VENDORID {
        return false;
    }
    true
}

/// PCI slot ULONG from device/function (DeviceNumber:5, FunctionNumber:3).
#[inline]
fn pci_slot(device: u16, function: u16) -> ULONG {
    ((device as ULONG) & 0x1F) | (((function as ULONG) & 0x7) << 5)
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsReadPciConfiguration(
    PciId: *mut ACPI_PCI_ID,
    Reg: UINT32,
    Value: *mut UINT64,
    Width: UINT32,
) -> ACPI_STATUS {
    let slot = pci_slot((*PciId).Device, (*PciId).Function);
    if !OslIsPciDevicePresent((*PciId).Bus as ULONG, slot) {
        return AE_NOT_FOUND;
    }
    // Width is in BITS.
    HalGetBusDataByOffset(
        PCIConfiguration,
        (*PciId).Bus as ULONG,
        slot,
        Value as PVOID,
        Reg,
        Width >> 3,
    );
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsWritePciConfiguration(
    PciId: *mut ACPI_PCI_ID,
    Reg: UINT32,
    Value: UINT64,
    Width: UINT32,
) -> ACPI_STATUS {
    let mut buf: ULONG = Value as ULONG;
    let slot = pci_slot((*PciId).Device, (*PciId).Function);
    if !OslIsPciDevicePresent((*PciId).Bus as ULONG, slot) {
        return AE_NOT_FOUND;
    }
    // Width is in BITS.
    HalSetBusDataByOffset(
        PCIConfiguration,
        (*PciId).Bus as ULONG,
        slot,
        &mut buf as *mut ULONG as PVOID,
        Reg,
        Width >> 3,
    );
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsVprintf(_Format: *const core::ffi::c_char, _Args: *mut c_void) {
    // NDEBUG: vDbgPrintEx compiled out → no-op (AcpiOsPrintf shim calls this).
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsRedirectOutput(_Destination: *mut c_void) {
    // No-op (output redirection not supported).
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsGetTimer() -> UINT64 {
    // C used KeQuerySystemTime (absolute 100ns), which is an inline macro on
    // amd64/arm64 (no export). KeQueryInterruptTime is a portable export and a
    // monotonic 100ns timer — equivalent for ACPICA's elapsed-time use.
    KeQueryInterruptTime()
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsWaitEventsComplete() {
    // Wait for all asynchronous events to complete — no-op.
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsSignal(Function: UINT32, _Info: *mut c_void) -> ACPI_STATUS {
    match Function {
        x if x == ACPI_SIGNAL_FATAL => {}
        x if x == ACPI_SIGNAL_BREAKPOINT => {}
        _ => {}
    }
    // C: ASSERT(FALSE) — keep a hard stop so a fatal ACPI signal is not silent.
    ntkrnl_sys::ASSERT!(false, "AcpiOsSignal");
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsEnterSleep(
    _SleepState: UINT8,
    _RegaValue: UINT32,
    _RegbValue: UINT32,
) -> ACPI_STATUS {
    AE_OK
}

#[no_mangle]
pub unsafe extern "C" fn AcpiOsGetLine(
    _Buffer: *mut core::ffi::c_char,
    _BufferLength: UINT32,
    _BytesRead: *mut UINT32,
) -> ACPI_STATUS {
    // File reading not supported.
    AE_ERROR
}
