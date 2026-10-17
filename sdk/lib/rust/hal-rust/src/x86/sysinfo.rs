//! HAL information routines (Hali{Query,Set}SystemInformation) +
//! the AMLI bad-I/O-port table (HalAMLIBadIOAddressList) consulted
//! by ACPI AML when a control method tries to touch reserved port
//! ranges. Most query classes are stubs; only HalQueryAMLI-
//! IllegalIOPortAddresses returns real data (the table).
//!
//! Translated from `hal/halx86/generic/sysinfo.c`.

use ntkrnl_sys::ddk::KeFlushWriteBuffer;
use ntkrnl_sys::hal_ddk::{
    BOOLEAN, HAL_QUERY_INFORMATION_CLASS, HAL_SET_INFORMATION_CLASS, NTSTATUS, PULONG, PVOID,
    ULONG,
};

const STATUS_SUCCESS: NTSTATUS = 0;
const STATUS_NOT_IMPLEMENTED: NTSTATUS = 0xC0000002u32 as i32;
const STATUS_UNSUCCESSFUL: NTSTATUS = 0xC0000001u32 as i32;
const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC0000004u32 as i32;

/// AMLI bad-I/O-port table entry.  Mirrors the C `HAL_AMLI_BAD_IO_-
/// ADDRESS_LIST` byte layout, but uses an `extern "system"` fn
/// pointer (the C typedef is `NTAPI`-annotated; bindgen drops the
/// cc annotation on typedefs which is wrong for i386 — write our
/// own type to keep the cc correct).
#[repr(C)]
struct HalAmliBadIoAddress {
    bad_addr_begin: ULONG,
    bad_addr_size: ULONG,
    os_version_trigger: ULONG,
    io_handler: Option<
        unsafe extern "system" fn(IsRead: BOOLEAN, Port: ULONG, Length: ULONG, Buffer: PULONG)
            -> NTSTATUS,
    >,
}

/// PCI port range (0xCF8..0xCFF).  When AML touches it, ACPI calls
/// `HaliHandlePCIConfigSpaceAccess`; we return STATUS_NOT_IMPLEMENTED
/// since real PCI config-space arbitration isn't wired up.
static HAL_AMLI_BAD_IO_ADDRESS_LIST: [HalAmliBadIoAddress; 19] = [
    HalAmliBadIoAddress { bad_addr_begin: 0x0000, bad_addr_size: 0x10, os_version_trigger: 1, io_handler: None }, // DMA controller
    HalAmliBadIoAddress { bad_addr_begin: 0x0020, bad_addr_size: 0x02, os_version_trigger: 0, io_handler: None }, // PIC (8259A)
    HalAmliBadIoAddress { bad_addr_begin: 0x0040, bad_addr_size: 0x04, os_version_trigger: 1, io_handler: None }, // System Timer 1
    HalAmliBadIoAddress { bad_addr_begin: 0x0048, bad_addr_size: 0x04, os_version_trigger: 1, io_handler: None }, // System Timer 2 failsafe
    HalAmliBadIoAddress { bad_addr_begin: 0x0070, bad_addr_size: 0x02, os_version_trigger: 1, io_handler: None }, // RTC
    HalAmliBadIoAddress { bad_addr_begin: 0x0074, bad_addr_size: 0x03, os_version_trigger: 1, io_handler: None }, // Extended CMOS
    HalAmliBadIoAddress { bad_addr_begin: 0x0081, bad_addr_size: 0x03, os_version_trigger: 1, io_handler: None }, // DMA 1 page registers
    HalAmliBadIoAddress { bad_addr_begin: 0x0087, bad_addr_size: 0x01, os_version_trigger: 1, io_handler: None }, // DMA 1 Ch 0 low page
    HalAmliBadIoAddress { bad_addr_begin: 0x0089, bad_addr_size: 0x01, os_version_trigger: 1, io_handler: None }, // DMA 2 page registers
    HalAmliBadIoAddress { bad_addr_begin: 0x008A, bad_addr_size: 0x02, os_version_trigger: 1, io_handler: None }, // DMA 2 page registers
    HalAmliBadIoAddress { bad_addr_begin: 0x008F, bad_addr_size: 0x01, os_version_trigger: 1, io_handler: None }, // DMA 2 low page refresh
    HalAmliBadIoAddress { bad_addr_begin: 0x0090, bad_addr_size: 0x02, os_version_trigger: 1, io_handler: None }, // Arbitration control
    HalAmliBadIoAddress { bad_addr_begin: 0x0093, bad_addr_size: 0x02, os_version_trigger: 1, io_handler: None }, // Reserved sysboard setup
    HalAmliBadIoAddress { bad_addr_begin: 0x0096, bad_addr_size: 0x02, os_version_trigger: 1, io_handler: None }, // POS channel select
    HalAmliBadIoAddress { bad_addr_begin: 0x00A0, bad_addr_size: 0x02, os_version_trigger: 0, io_handler: None }, // Cascaded PIC
    HalAmliBadIoAddress { bad_addr_begin: 0x00C0, bad_addr_size: 0x20, os_version_trigger: 1, io_handler: None }, // ISA DMA
    HalAmliBadIoAddress { bad_addr_begin: 0x04D0, bad_addr_size: 0x02, os_version_trigger: 0, io_handler: None }, // PIC edge/level registers
    HalAmliBadIoAddress { bad_addr_begin: 0x0CF8, bad_addr_size: 0x08, os_version_trigger: 1, io_handler: Some(HaliHandlePCIConfigSpaceAccess) }, // PCI configuration space
    HalAmliBadIoAddress { bad_addr_begin: 0x0000, bad_addr_size: 0x00, os_version_trigger: 0, io_handler: None }, // Reserved
];

/// HAL-internal: AML hook invoked when an AML method tries to touch
/// the PCI config-space port pair (0xCF8..0xCFF).  Stub: returns
/// STATUS_NOT_IMPLEMENTED.  ASSERT(FALSE) is commented out in the C
/// source so we leave it out here too.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HaliHandlePCIConfigSpaceAccess(
    _IsRead: BOOLEAN,
    Port: ULONG,
    _Length: ULONG,
    _Buffer: PULONG,
) -> NTSTATUS {
    // A port outside the range this handler claims (0xCF8, 8 ports) is refused with
    // STATUS_UNSUCCESSFUL and the buffer left alone, as Windows does (HalSystemInfo.c:143
    // calls it one below the range). Emulating AML's own config-space cycles inside the
    // range is still not implemented, as in the C.
    if !(0x0CF8..0x0CF8 + 8).contains(&Port) {
        return STATUS_UNSUCCESSFUL;
    }
    STATUS_NOT_IMPLEMENTED
}

/// HAL-internal: backs the `HalQuerySystemInformation` callback that
/// halinit.c assigns at boot.  Most classes are stubs; only the
/// AMLI port-arbitration request returns real data.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HaliQuerySystemInformation(
    InformationClass: HAL_QUERY_INFORMATION_CLASS,
    BufferSize: ULONG,
    Buffer: PVOID,
    ReturnedLength: PULONG,
) -> NTSTATUS {
    // Use the bindgen-generated enum constants to match (HAL_QUERY_-
    // INFORMATION_CLASS values).  The C source uses a switch with
    // REPORT_THIS_CASE for stub branches; in Rust we just check the
    // one we handle and stub-fall the rest.
    use ntkrnl_sys::hal_ddk::_HAL_QUERY_INFORMATION_CLASS_HalQueryAMLIIllegalIOPortAddresses
        as HalQueryAMLIIllegalIOPortAddresses;

    if InformationClass == HalQueryAMLIIllegalIOPortAddresses {
        let size = core::mem::size_of_val(&HAL_AMLI_BAD_IO_ADDRESS_LIST) as ULONG;
        let mut status: NTSTATUS = STATUS_INFO_LENGTH_MISMATCH;

        if BufferSize >= size {
            unsafe {
                core::ptr::copy_nonoverlapping(
                    HAL_AMLI_BAD_IO_ADDRESS_LIST.as_ptr() as *const u8,
                    Buffer as *mut u8,
                    size as usize,
                );
            }
            status = STATUS_SUCCESS;
        }

        if !ReturnedLength.is_null() {
            unsafe { core::ptr::write_volatile(ReturnedLength, size); }
        }
        unsafe { KeFlushWriteBuffer(); }
        return status;
    }

    // Every other class falls through to UNIMPLEMENTED in the C source.
    STATUS_NOT_IMPLEMENTED
}

/// HAL-internal: backs the `HalSetSystemInformation` callback.
/// Pure stub in the C source — every set is unimplemented.
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HaliSetSystemInformation(
    _InformationClass: HAL_SET_INFORMATION_CLASS,
    _BufferSize: ULONG,
    _Buffer: PVOID,
) -> NTSTATUS {
    STATUS_NOT_IMPLEMENTED
}
