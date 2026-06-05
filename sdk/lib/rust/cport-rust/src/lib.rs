//! Rust translation of `sdk/lib/cportlib/cport.c` — the ReactOS ComPort
//! serial library (NS16550 UART) used by KDCOM, INIT (ntoskrnl) and FREELDR.
//!
//! Faithful port of the C: same Microsoft-documented init/probe sequence
//! (see the C comments — "Do not change this code to improve it"). The only
//! substrate is `READ_PORT_UCHAR`/`WRITE_PORT_UCHAR` (ddk/ioaccess.h): on
//! x86 those are `__inbyte`/`__outbyte` port I/O (the PUCHAR "Address" is a
//! port number cast to a pointer), and on arm64 they fall back to
//! `READ/WRITE_REGISTER_UCHAR` (MMIO at the address). Pointer arithmetic on
//! the `*mut u8` Address + a register offset works for both forms.
//!
//! crate-type = staticlib → `libcport_rust.a`; the `#[no_mangle] extern
//! "system"` NTAPI exports (`Cp*`) replace the C `cport.o` in the `cportlib`
//! target (cohabitation). `extern "system"` = stdcall (`_Cp…@N`) on i386,
//! cdecl on amd64/arm64 — matching the C NTAPI ABI per arch.

#![no_std]
#![allow(non_snake_case, non_upper_case_globals)]

use core::sync::atomic::{AtomicU8, Ordering};

// ── NS16550 register offsets (drivers/serial/ns16550.h, STRIDE=1) ──────────
const RECEIVE_BUFFER_REGISTER: usize = 0x00;
const TRANSMIT_HOLDING_REGISTER: usize = 0x00;
const INTERRUPT_ENABLE_REGISTER: usize = 0x01;
const FIFO_CONTROL_REGISTER: usize = 0x02;
const LINE_CONTROL_REGISTER: usize = 0x03;
const MODEM_CONTROL_REGISTER: usize = 0x04;
const LINE_STATUS_REGISTER: usize = 0x05;
const MODEM_STATUS_REGISTER: usize = 0x06;
const SCRATCH_REGISTER: usize = 0x07;
const DIVISOR_LATCH_LSB: usize = 0x00;
const DIVISOR_LATCH_MSB: usize = 0x01;

const CLOCK_RATE: u32 = 115200;

// FIFO control
const SERIAL_FCR_DISABLE: u8 = 0x00;
const SERIAL_FCR_ENABLE: u8 = 0x01;
const SERIAL_FCR_RCVR_RESET: u8 = 0x02;
const SERIAL_FCR_TXMT_RESET: u8 = 0x04;
// Line control
const SERIAL_LCR_DLAB: u8 = 0x80;
const SERIAL_8_DATA: u8 = 0x03;
const SERIAL_1_STOP: u8 = 0x00;
const SERIAL_NONE_PARITY: u8 = 0x00;
// Modem control
const SERIAL_MCR_DTR: u8 = 0x01;
const SERIAL_MCR_RTS: u8 = 0x02;
const SERIAL_MCR_OUT1: u8 = 0x04;
const SERIAL_MCR_OUT2: u8 = 0x08;
const SERIAL_MCR_LOOP: u8 = 0x10;
// Line status
const SERIAL_LSR_DR: u8 = 0x01;
const SERIAL_LSR_PE: u8 = 0x04;
const SERIAL_LSR_FE: u8 = 0x08;
const SERIAL_LSR_OE: u8 = 0x02;
const SERIAL_LSR_THRE: u8 = 0x20;
// Modem status
const SERIAL_MSR_CTS: u8 = 0x10;
const SERIAL_MSR_DSR: u8 = 0x20;
const SERIAL_MSR_RI: u8 = 0x40;
const SERIAL_MSR_DCD: u8 = 0x80;

// cportlib.h
const CP_GET_SUCCESS: u16 = 0;
const CP_GET_NODATA: u16 = 1;
const CP_GET_ERROR: u16 = 2;
const CPPORT_FLAG_MODEM_CONTROL: u16 = 0x02;

const STATUS_SUCCESS: i32 = 0;
const STATUS_INVALID_PARAMETER: i32 = 0xC000000Du32 as i32;
const STATUS_NOT_FOUND: i32 = 0xC0000225u32 as i32;

const TIMEOUT_COUNT: u32 = 1024 * 200;

/// `_CPPORT` — matches the C layout (PUCHAR / ULONG / USHORT).
#[repr(C)]
pub struct CPPORT {
    pub Address: *mut u8,
    pub BaudRate: u32,
    pub Flags: u16,
}

// `UCHAR RingIndicator;` — file-global probe state in the C. Single-threaded
// boot/KD context; an atomic keeps it sound without `static mut`.
static RING_INDICATOR: AtomicU8 = AtomicU8::new(0);

// ── READ_PORT_UCHAR / WRITE_PORT_UCHAR (ddk/ioaccess.h) ────────────────────
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn read_port_uchar(addr: *mut u8) -> u8 {
    // H2I: the PUCHAR is a port number; in al, dx.
    let port = addr as usize as u16;
    let val: u8;
    core::arch::asm!("in al, dx", out("al") val, in("dx") port,
                     options(nomem, nostack, preserves_flags));
    val
}
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
unsafe fn write_port_uchar(addr: *mut u8, val: u8) {
    let port = addr as usize as u16;
    core::arch::asm!("out dx, al", in("dx") port, in("al") val,
                     options(nomem, nostack, preserves_flags));
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
unsafe fn read_port_uchar(addr: *mut u8) -> u8 {
    // arm64: READ_REGISTER_UCHAR — volatile MMIO read at the address.
    core::ptr::read_volatile(addr)
}
#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
#[inline]
unsafe fn write_port_uchar(addr: *mut u8, val: u8) {
    core::ptr::write_volatile(addr, val)
}

#[inline]
unsafe fn rd(addr: *mut u8, off: usize) -> u8 {
    read_port_uchar(addr.add(off))
}
#[inline]
unsafe fn wr(addr: *mut u8, off: usize, val: u8) {
    write_port_uchar(addr.add(off), val)
}

#[no_mangle]
pub unsafe extern "system" fn CpEnableFifo(Address: *mut u8, Enable: u8) {
    // Set FIFO and clear the receive/transmit buffers.
    wr(
        Address,
        FIFO_CONTROL_REGISTER,
        if Enable != 0 {
            SERIAL_FCR_ENABLE | SERIAL_FCR_RCVR_RESET | SERIAL_FCR_TXMT_RESET
        } else {
            SERIAL_FCR_DISABLE
        },
    );
}

#[no_mangle]
pub unsafe extern "system" fn CpSetBaud(Port: *mut CPPORT, BaudRate: u32) {
    let port = &mut *Port;
    let mode = CLOCK_RATE / BaudRate;

    // Set the DLAB on.
    let lcr = rd(port.Address, LINE_CONTROL_REGISTER);
    wr(port.Address, LINE_CONTROL_REGISTER, lcr | SERIAL_LCR_DLAB);

    // Set the baud rate divisor.
    wr(port.Address, DIVISOR_LATCH_LSB, (mode & 0xFF) as u8);
    wr(port.Address, DIVISOR_LATCH_MSB, ((mode >> 8) & 0xFF) as u8);

    // Reset DLAB.
    wr(port.Address, LINE_CONTROL_REGISTER, lcr);

    port.BaudRate = BaudRate;
}

#[no_mangle]
pub unsafe extern "system" fn CpInitialize(
    Port: *mut CPPORT,
    Address: *mut u8,
    BaudRate: u32,
) -> i32 {
    if Port.is_null() || Address.is_null() || BaudRate == 0 {
        return STATUS_INVALID_PARAMETER;
    }
    if CpDoesPortExist(Address) == 0 {
        return STATUS_NOT_FOUND;
    }

    let port = &mut *Port;
    port.Address = Address;
    port.BaudRate = 0;
    port.Flags = 0;

    // Disable interrupts.
    wr(Address, LINE_CONTROL_REGISTER, 0);
    wr(Address, INTERRUPT_ENABLE_REGISTER, 0);

    // Turn on DTR, RTS and OUT2.
    wr(
        Address,
        MODEM_CONTROL_REGISTER,
        SERIAL_MCR_DTR | SERIAL_MCR_RTS | SERIAL_MCR_OUT2,
    );

    CpSetBaud(Port, BaudRate);

    // 8 data bits, 1 stop bit, no parity, no break.
    wr(
        Address,
        LINE_CONTROL_REGISTER,
        SERIAL_8_DATA | SERIAL_1_STOP | SERIAL_NONE_PARITY,
    );

    // Turn on FIFO (16550).
    CpEnableFifo(Address, 1);

    // Read junk out of the RBR.
    let _ = rd(Address, RECEIVE_BUFFER_REGISTER);

    STATUS_SUCCESS
}

unsafe fn ComPortTest1(Address: *mut u8) -> bool {
    // 16550/16450 loopback (diagnostic) probe — the MS Headless sequence.
    let mut ret = false;

    let mcr = rd(Address, MODEM_CONTROL_REGISTER);

    // Enable loop (diagnostic) mode, clear all modem output bits.
    wr(Address, MODEM_CONTROL_REGISTER, SERIAL_MCR_LOOP);
    wr(Address, MODEM_CONTROL_REGISTER, SERIAL_MCR_LOOP);

    let msr = rd(Address, MODEM_STATUS_REGISTER);
    if (msr & (SERIAL_MSR_CTS | SERIAL_MSR_DSR | SERIAL_MSR_RI | SERIAL_MSR_DCD)) == 0x00 {
        // Set OUT1 in loopback (Windows variant).
        wr(
            Address,
            MODEM_CONTROL_REGISTER,
            SERIAL_MCR_OUT1 | SERIAL_MCR_LOOP,
        );
        let msr = rd(Address, MODEM_STATUS_REGISTER);
        if msr & SERIAL_MSR_RI != 0 {
            ret = true;
        }
    }

    // Restore the MCR.
    wr(Address, MODEM_CONTROL_REGISTER, mcr);
    ret
}

unsafe fn ComPortTest2(Address: *mut u8) -> bool {
    // Scratch-register availability probe; absent → no port.
    let mut byte: u8 = 0;
    loop {
        wr(Address, SCRATCH_REGISTER, byte);
        if rd(Address, SCRATCH_REGISTER) != byte {
            return false;
        }
        byte = byte.wrapping_add(1);
        if byte == 0 {
            break;
        }
    }
    true
}

#[no_mangle]
pub unsafe extern "system" fn CpDoesPortExist(Address: *mut u8) -> u8 {
    (ComPortTest1(Address) || ComPortTest2(Address)) as u8
}

#[no_mangle]
pub unsafe extern "system" fn CpReadLsr(Port: *mut CPPORT, ExpectedValue: u8) -> u8 {
    let port = &mut *Port;
    let lsr = rd(port.Address, LINE_STATUS_REGISTER);
    if lsr & ExpectedValue == 0 {
        // Track ring-indicator toggles to detect modem-control mode.
        let msr = rd(port.Address, MODEM_STATUS_REGISTER);
        let bit = if msr & SERIAL_MSR_RI != 0 { 1 } else { 2 };
        let ri = RING_INDICATOR.fetch_or(bit, Ordering::Relaxed) | bit;
        if ri == 3 {
            port.Flags |= CPPORT_FLAG_MODEM_CONTROL;
        }
    }
    lsr
}

#[no_mangle]
pub unsafe extern "system" fn CpGetByte(
    Port: *mut CPPORT,
    Byte: *mut u8,
    Wait: u8,
    Poll: u8,
) -> u16 {
    let port = &mut *Port;
    let mut limit: u32 = if Wait != 0 { TIMEOUT_COUNT } else { 1 };

    // Early read-before-init.
    if port.Address.is_null() {
        return CP_GET_NODATA;
    }

    while limit != 0 {
        limit -= 1;
        let lsr = CpReadLsr(Port, SERIAL_LSR_DR);
        if (lsr & SERIAL_LSR_DR) == SERIAL_LSR_DR {
            // Error → clear the byte and fail.
            if lsr & (SERIAL_LSR_FE | SERIAL_LSR_PE | SERIAL_LSR_OE) != 0 {
                *Byte = 0;
                return CP_GET_ERROR;
            }
            // Poll-only: report data present without consuming.
            if Poll != 0 {
                return CP_GET_SUCCESS;
            }
            *Byte = rd(port.Address, RECEIVE_BUFFER_REGISTER);
            // CPPORT_FLAG_MODEM_CONTROL CD handling: not implemented (as in C).
            return CP_GET_SUCCESS;
        }
    }

    // No data — reset LSR tracking.
    CpReadLsr(Port, 0);
    CP_GET_NODATA
}

#[no_mangle]
pub unsafe extern "system" fn CpPutByte(Port: *mut CPPORT, Byte: u8) {
    let port = &mut *Port;
    // Modem-control CD handling: not implemented (as in C).

    // Wait for THRE (transmit holding register empty).
    while (CpReadLsr(Port, SERIAL_LSR_THRE) & SERIAL_LSR_THRE) == 0x00 {}

    wr(port.Address, TRANSMIT_HOLDING_REGISTER, Byte);
}

// ── staticlib runtime stubs (mirror kdcom-rust / hal-rust) ─────────────────
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("wfe", options(nomem, nostack));
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        core::hint::spin_loop();
    }
}

#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
