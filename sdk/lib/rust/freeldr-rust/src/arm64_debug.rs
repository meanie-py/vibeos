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

// PL011 UART base address (QEMU virt machine).
const PL011_UART_BASE: usize = 0x09000000;

#[no_mangle]
pub unsafe extern "C" fn Rs232PortInitialize(
    _ComPort: ULONG,
    _BaudRate: ULONG,
) -> BOOLEAN {
    TRUE
}

#[no_mangle]
pub unsafe extern "C" fn Rs232PortPutByte(ByteToSend: UCHAR) {
    let uart0dr = PL011_UART_BASE as *mut u32;
    core::ptr::write_volatile(uart0dr, ByteToSend as u32);
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
