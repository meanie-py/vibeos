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

/// Write one byte to the debug PL011, waiting for room in the TX FIFO.
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
    // PL011 UARTFR bit 5 (TXFF) is set while the TX FIFO is full.
    let flags = (base as usize + 0x18) as *const u32;
    let mut spins = 0u32;
    while core::ptr::read_volatile(flags) & (1 << 5) != 0 {
        spins += 1;
        if spins > 1_000_000 {
            break;
        }
        core::hint::spin_loop();
    }
    core::ptr::write_volatile(base, c as u32);
}

/// `uart_putc` against the kernel's current debug UART base.
#[inline(always)]
pub unsafe fn uart_putc_default(c: u8) {
    #[cfg(target_arch = "aarch64")]
    uart_putc(Arm64UartKernelBase as *mut u32, c);
    #[cfg(not(target_arch = "aarch64"))]
    let _ = c;
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
