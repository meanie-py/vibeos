//! arch/arm64/debug.c — ARM64 UEFI debug stubs (PL011 UART).
//!
//! Used only by uefildr.efi (arm64). Provides:
//!   - Rs232PortInitialize / Rs232PortPutByte — for the debug.rs serial path
//!   - DbgBreakPoint / KeGetCurrentIrql / RtlFillMemoryUlong — stubs needed
//!     by the bootloader but not pulled from RTL.
//!   - FrLdrBugCheckWithMessage — variadic NORETURN stub. Stable Rust can't
//!     host c_variadic DEFs, so the variadic args are dropped from the
//!     signature; the symbol still satisfies link references. (Same pattern
//!     used elsewhere — see feedback_degraded_variadic_stubs; arm64 UEFI
//!     freeldr does not exercise this path.)

use ntkrnl_sys::base::*;
use ntkrnl_sys::ddk::PSTR;

use crate::frldr::{EFI_SYSTEM_TABLE, EFI_ACPI_20_TABLE_GUID, RSDP_DESCRIPTOR};
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

// Zero until firmware describes a supported UART. Never probe a QEMU address
// on unknown hardware: that address can be ordinary RAM on a physical board.
static UART_BASE: AtomicUsize = AtomicUsize::new(0);
// The validated XSDT that answered the UART query, kept so later loader stages
// can consult the same tables without re-walking the EFI configuration table.
static XSDT_BASE: AtomicU64 = AtomicU64::new(0);

pub fn uart_base() -> usize { UART_BASE.load(Ordering::Relaxed) }

/// Physical address of the XSDT, or zero if ACPI discovery never succeeded.
pub fn xsdt_base() -> u64 { XSDT_BASE.load(Ordering::Relaxed) }

pub unsafe fn initialize(system: *mut EFI_SYSTEM_TABLE) -> bool {
    UART_BASE.store(0, Ordering::Relaxed);
    XSDT_BASE.store(0, Ordering::Relaxed);
    if system.is_null() { return false; }
    let guid = EFI_ACPI_20_TABLE_GUID;
    for i in 0..(*system).NumberOfTableEntries {
        let entry = &*(*system).ConfigurationTable.add(i as usize);
        let g = &entry.VendorGuid;
        if g.Data1 != guid.Data1 || g.Data2 != guid.Data2
            || g.Data3 != guid.Data3 || g.Data4 != guid.Data4 { continue; }
        let ptr = entry.VendorTable as *const RSDP_DESCRIPTOR;
        if ptr.is_null() { continue; }
        let rsdp = core::ptr::read_unaligned(ptr);
        let first = core::slice::from_raw_parts(ptr.cast::<u8>(), 20);
        if &first[..8] != b"RSD PTR " || rsdp.revision < 2
            || first.iter().fold(0u8, |s,b| s.wrapping_add(*b)) != 0
            || rsdp.length < core::mem::size_of::<RSDP_DESCRIPTOR>() as u32
            || rsdp.length > 4096 { continue; }
        let all = core::slice::from_raw_parts(ptr.cast::<u8>(), rsdp.length as usize);
        if all.iter().fold(0u8, |s,b| s.wrapping_add(*b)) != 0 { continue; }
        if let Some(base) = crate::arm64_serial_acpi::from_xsdt(rsdp.xsdt_physical_address) {
            UART_BASE.store(base as usize, Ordering::Relaxed);
            XSDT_BASE.store(rsdp.xsdt_physical_address, Ordering::Relaxed);
            return true;
        }
    }
    false
}

#[no_mangle]
pub unsafe extern "C" fn Rs232PortInitialize(
    _ComPort: ULONG,
    _BaudRate: ULONG,
) -> BOOLEAN {
    (uart_base() != 0) as BOOLEAN
}

#[no_mangle]
pub unsafe extern "C" fn Rs232PortPutByte(ByteToSend: UCHAR) {
    let base = uart_base();
    // EfiEntry reports discovery failure through ConOut and returns before
    // any normal loader work. A panic before discovery must not touch MMIO.
    if base == 0 { return; }
    // Preserve the firmware's baud and clock settings. Wait for FIFO space.
    let flags = (base + 0x18) as *const u32;
    while core::ptr::read_volatile(flags) & (1 << 5) != 0 {
        core::hint::spin_loop();
    }
    core::ptr::write_volatile(base as *mut u32, ByteToSend as u32);
}

// DECLSPEC_NORETURN — for(;;) in C. Variadic def is unsupported on stable;
// drop the `...` (arm64 path doesn't format the message anyway).
#[no_mangle]
pub unsafe extern "C" fn FrLdrBugCheckWithMessage(
    _BugCode: ULONG,
    _File: PCHAR,
    _Line: ULONG,
    _Format: PSTR,
) -> ! {
    loop {
        core::arch::asm!("wfe", options(nomem, nostack));
    }
}

#[no_mangle]
pub unsafe extern "C" fn DbgBreakPoint() {
    core::arch::asm!("brk #0xF000", options(nostack));
}

#[no_mangle]
pub unsafe extern "C" fn KeGetCurrentIrql() -> KIRQL {
    PASSIVE_LEVEL
}

#[no_mangle]
pub unsafe extern "C" fn RtlFillMemoryUlong(
    Destination: PVOID,
    Length: SIZE_T,
    Pattern: ULONG,
) {
    let mut d = Destination as PULONG;
    let mut Count = Length / core::mem::size_of::<ULONG>();
    while Count > 0 {
        Count -= 1;
        core::ptr::write_volatile(d, Pattern);
        d = d.offset(1);
    }
}
