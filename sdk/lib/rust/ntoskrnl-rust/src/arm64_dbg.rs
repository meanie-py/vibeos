//! Reusable arm64 UART debug traces that SURVIVE release optimization.
//!
//! Plain `*uart = byte` (non-volatile) stores are dead-code-eliminated by LLVM in
//! release builds (the result is never read), taking the string constant with them —
//! so a missing serial line never proves the code didn't run. These helpers use
//! `write_volatile` + `#[inline(never)]` so the writes (and their strings) always
//! survive. ALWAYS gate on `strings ntoskrnl.exe | grep -c <marker>` >= 1 before booting.

#[cfg(target_arch = "aarch64")]
extern "system" {
    static mut Arm64UartKernelBase: usize;
}

/// PL011 UARTFR (the flag register) as an INDEX into the register block viewed
/// as 32-bit registers - every PL011 register is 32 bits wide, so indexing a
/// `*mut u32` is the block's natural unit and needs no byte arithmetic. Byte
/// offset 0x18 from the base (ARM PrimeCell PL011 TRM table 3-1); the data
/// register is index 0.
const PL011_FR_INDEX: usize = 0x18 / 4;
/// UARTFR.TXFF - set while the transmit FIFO is full.
const PL011_FR_TXFF: u32 = 1 << 5;
/// UARTFR.RXFE - set while the receive FIFO is empty.
const PL011_FR_RXFE: u32 = 1 << 4;

/// The console UART's register model, from the loader (ARM_LOADER_BLOCK.MachineType
/// on arm64; freeldr's arm64_serial_acpi.rs `Uart::encode`): 0 a PL011, else a
/// 16550 in memory space with its register shift in bits 11:8 and access width in
/// bytes in bits 19:16. EC2's Graviton console is the latter (Amazon's PCI UART).
pub static ARM64_UART_KIND: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
pub const UART_KIND_NS16550: u32 = 1;
// 16550 registers, by number: THR/RBR at 0, LSR at 5.
const NS16550_LSR: usize = 5;
const NS16550_LSR_DR: u32 = 1 << 0;
const NS16550_LSR_THRE: u32 = 1 << 5;

/// A 16550 register `n`, read with the access width the loader found.
#[inline(always)]
unsafe fn ns16550_read(base: *mut u8, kind: u32, n: usize) -> u32 {
    let at = base.add(n << ((kind >> 8) & 0xF));
    if (kind >> 16) & 0xF == 4 { core::ptr::read_volatile(at as *const u32) } else { core::ptr::read_volatile(at) as u32 }
}

#[inline(always)]
unsafe fn ns16550_write(base: *mut u8, kind: u32, n: usize, v: u8) {
    let at = base.add(n << ((kind >> 8) & 0xF));
    if (kind >> 16) & 0xF == 4 { core::ptr::write_volatile(at as *mut u32, v as u32) } else { core::ptr::write_volatile(at, v) }
}

/// One received byte from the console UART, if one is waiting.
pub unsafe fn uart_getc(base: *mut u32) -> Option<u8> {
    if base.is_null() {
        return None;
    }
    let kind = ARM64_UART_KIND.load(core::sync::atomic::Ordering::Relaxed);
    if kind & 0xFF == UART_KIND_NS16550 {
        let b = base as *mut u8;
        return (ns16550_read(b, kind, NS16550_LSR) & NS16550_LSR_DR != 0).then(|| ns16550_read(b, kind, 0) as u8);
    }
    (core::ptr::read_volatile(base.add(PL011_FR_INDEX)) & PL011_FR_RXFE == 0)
        .then(|| core::ptr::read_volatile(base) as u8)
}

/// Write one byte to the debug UART, waiting for room in the TX FIFO.
///
/// The wait is not optional on real hardware. QEMU's PL011 drains instantly, so
/// unsynchronised `write_volatile` never lost a byte there; a physical PL011 at
/// 115200 overruns almost immediately, and the Pi 4B's first kernel output came
/// back as "MiIniti"/"MiBuild" - traces shredded mid-word, which reads exactly
/// like a crash at that point rather than a serial problem.
///
/// The spin is bounded: a UART that never reports room must not convert missing
/// output into a hang, which would be a worse failure than the one this fixes.
#[inline(always)]
pub unsafe fn uart_putc(base: *mut u32, c: u8) {
    if base.is_null() {
        return;
    }
    let kind = ARM64_UART_KIND.load(core::sync::atomic::Ordering::Relaxed);
    if kind & 0xFF == UART_KIND_NS16550 {
        let b = base as *mut u8;
        let mut spins = 0u32;
        while ns16550_read(b, kind, NS16550_LSR) & NS16550_LSR_THRE == 0 {
            spins += 1;
            if spins > 1_000_000 {
                break;
            }
            core::hint::spin_loop();
        }
        ns16550_write(b, kind, 0, c);
        return;
    }
    let flags = base.add(PL011_FR_INDEX) as *const u32;
    let mut spins = 0u32;
    while core::ptr::read_volatile(flags) & PL011_FR_TXFF != 0 {
        spins += 1;
        if spins > 1_000_000 {
            break;
        }
        core::hint::spin_loop();
    }
    core::ptr::write_volatile(base, c as u32);
}

/// `uart_putc_default` for the kernel's other crates (iomgr, se, ps, lpc, ...),
/// which cannot name this module. Every debug byte on arm64 goes through here:
/// a bare store to the data register is silently dropped once the transmit FIFO
/// is full, which QEMU's instantly-draining PL011 never shows. On a physical
/// Pi 4 it cut every Phase-1 trace line down to its first `[`, so the last
/// readable line of a hang was not the last thing the kernel said.
#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub unsafe extern "C" fn Arm64UartPutChar(c: u8) {
    uart_putc_default(c);
}

/// `uart_putc` against the kernel's current debug UART base.
#[inline(always)]
pub unsafe fn uart_putc_default(c: u8) {
    #[cfg(target_arch = "aarch64")]
    uart_putc(Arm64UartKernelBase as *mut u32, c);
    #[cfg(not(target_arch = "aarch64"))]
    let _ = c;
}

/// Whole-message serialization for the debug UART across CPUs.
///
/// Every writer of a complete message (the DbgPrint BRK handler,
/// ARM64_UART_TRACE, ARM64_UART_TRACE_HEX) takes this around the message, so
/// four cores' output arrives as lines instead of interleaved bytes. IRQs are
/// masked while it is held: a writer running with IRQs on could otherwise be
/// interrupted by a handler that prints on the same CPU and spin on its own
/// lock forever. Returns the DAIF value to hand back to `uart_line_unlock`.
#[cfg(target_arch = "aarch64")]
static UART_LINE_LOCK: core::sync::atomic::AtomicBool = core::sync::atomic::AtomicBool::new(false);

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn uart_line_lock() -> u64 {
    let daif: u64;
    core::arch::asm!("mrs {}, daif", "msr daifset, #2", out(reg) daif, options(nomem, nostack));
    while UART_LINE_LOCK
        .compare_exchange_weak(false, true, core::sync::atomic::Ordering::Acquire, core::sync::atomic::Ordering::Relaxed)
        .is_err()
    {
        core::hint::spin_loop();
    }
    daif
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
pub unsafe fn uart_line_unlock(daif: u64) {
    UART_LINE_LOCK.store(false, core::sync::atomic::Ordering::Release);
    core::arch::asm!("msr daif, {}", in(reg) daif, options(nomem, nostack));
}

#[inline(never)]
pub unsafe fn kdbg_str(s: &[u8]) {
    #[cfg(target_arch = "aarch64")]
    {
        let u = Arm64UartKernelBase as *mut u32;
        if !u.is_null() {
            for &c in s {
                uart_putc(u, c);
            }
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = s;
    }
}

#[inline(never)]
pub unsafe fn kdbg_hex(v: u64) {
    #[cfg(target_arch = "aarch64")]
    {
        let u = Arm64UartKernelBase as *mut u32;
        if !u.is_null() {
            uart_putc(u, b'0');
            uart_putc(u, b'x');
            for i in (0..16).rev() {
                let n = ((v >> (i * 4)) & 0xf) as u8;
                let c = if n < 10 { b'0' + n } else { b'a' + n - 10 };
                uart_putc(u, c);
            }
            uart_putc(u, b'\r');
            uart_putc(u, b'\n');
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = v;
    }
}

/// Dump the bindgen IRP / IO_STACK_LOCATION layout (ground truth vs the hardcoded
/// arm64 offsets) to the UART.
#[inline(never)]
pub unsafe fn dump_irp_layout() {
    use ntkrnl_sys::ddk;
    kdbg_str(b"[LAYOUT] IRP.CurrentStackLocation off=");
    kdbg_hex(core::mem::offset_of!(ddk::_IRP, Tail.Overlay.__bindgen_anon_2.__bindgen_anon_1.CurrentStackLocation) as u64);
    kdbg_str(b"[LAYOUT] IRP.CurrentLocation off=");
    kdbg_hex(core::mem::offset_of!(ddk::_IRP, CurrentLocation) as u64);
    kdbg_str(b"[LAYOUT] sizeof(IO_STACK_LOCATION)=");
    kdbg_hex(core::mem::size_of::<ddk::_IO_STACK_LOCATION>() as u64);
    kdbg_str(b"[LAYOUT] sizeof(IRP)=");
    kdbg_hex(core::mem::size_of::<ddk::_IRP>() as u64);
    kdbg_str(b"[LAYOUT] isl.DeviceObject off=");
    kdbg_hex(core::mem::offset_of!(ddk::_IO_STACK_LOCATION, DeviceObject) as u64);
    kdbg_str(b"[LAYOUT] isl.FileObject off=");
    kdbg_hex(core::mem::offset_of!(ddk::_IO_STACK_LOCATION, FileObject) as u64);
    kdbg_str(b"[LAYOUT] isl.CompletionRoutine off=");
    kdbg_hex(core::mem::offset_of!(ddk::_IO_STACK_LOCATION, CompletionRoutine) as u64);
    kdbg_str(b"[LAYOUT] isl.Context off=");
    kdbg_hex(core::mem::offset_of!(ddk::_IO_STACK_LOCATION, Context) as u64);
}

/// Print the current thread's kernel-APC list head Flink (to catch APC-list corruption).
#[inline(never)]
pub unsafe fn kdbg_apclist(tag: &[u8]) {
    #[cfg(target_arch = "aarch64")]
    {
        use ntkrnl_sys::ddk;
        extern "system" { fn PsGetCurrentThread() -> ddk::PETHREAD; }
        let t = PsGetCurrentThread();
        kdbg_str(tag);
        if t.is_null() { kdbg_hex(0); return; }
        let apc_state = crate::ke::ke_defs::kt_ApcState(core::ptr::addr_of_mut!((*t).Tcb));
        kdbg_hex(apc_state.ApcListHead[0].Flink as u64);
    }
    #[cfg(not(target_arch = "aarch64"))]
    { let _ = tag; }
}

/// Print a UNICODE_STRING's raw UTF-16 buffer as ASCII (best-effort) to the UART,
/// `len_bytes` = the string's Length field (bytes, not chars).
#[inline(never)]
pub unsafe fn kdbg_wstr(buffer: *const u16, len_bytes: u16) {
    #[cfg(target_arch = "aarch64")]
    {
        let u = Arm64UartKernelBase as *mut u32;
        if !u.is_null() && !buffer.is_null() {
            let n = (len_bytes / 2) as isize;
            let mut i = 0isize;
            while i < n {
                let w = core::ptr::read_volatile(buffer.offset(i));
                let c = if w >= 0x20 && w < 0x7f { w as u8 } else { b'?' };
                uart_putc(u, c);
                i += 1;
            }
            uart_putc(u, b'\r');
            uart_putc(u, b'\n');
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (buffer, len_bytes);
    }
}
