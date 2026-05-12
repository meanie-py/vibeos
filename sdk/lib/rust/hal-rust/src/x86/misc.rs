//! HAL grab-bag: serial-number reporting, ACPI mapper-mark, registry
//! key open helper, TLB flush, IRQL/spinlock NTAPI wrappers around the
//! FASTCALL primitives, plus a couple of x86 no-op hooks.
//!
//! Translated from `hal/halx86/generic/misc.c`.  Two HAL globals stay
//! defined here:
//!
//!   HalpSerialLen      — UCHAR, length of the serial-number blob in
//!                        HalpSerialNumber (set by the loader / firmware
//!                        path before HalpReportSerialNumber runs)
//!   HalpSerialNumber   — CHAR[31] backing storage
//!
//! The `_M_IX86`-only KeRaiseIrql / KeLowerIrql / KeAcquireSpinLock /
//! KeReleaseSpinLock NTAPI wrappers exist because their headers expose
//! the names with stdcall lowering while the actual implementations
//! are FASTCALL — the wrappers do the ABI bridge.

use ntkrnl_sys::ddk::{
    NTSTATUS, OBJECT_ATTRIBUTES, RtlInitUnicodeString, ZwClose, ZwCreateKey, ZwOpenKey,
    ZwSetValueKey,
};
use ntkrnl_sys::hal_ddk::{
    BOOLEAN, HANDLE, HalDisableFirmwareMapper, KIRQL, PHANDLE, PKINTERRUPT_ROUTINE, PKIRQL,
    PKSPIN_LOCK, PUNICODE_STRING, ULONG, ULONG_PTR, UNICODE_STRING, OBJ_CASE_INSENSITIVE,
};

// REG_BINARY / REG_DWORD / KEY_ALL_ACCESS / REG_OPTION_VOLATILE come from
// the kernel's xdk headers; numeric values are stable across NT versions.
const REG_BINARY: ULONG = 3;
const REG_DWORD: ULONG = 4;
const KEY_ALL_ACCESS: u32 = 0xF003F;
const REG_OPTION_VOLATILE: ULONG = 1;

// CR4.PGE — page-global-enable bit; clearing it forces a TLB flush of
// global entries on the next CR3 reload.
const CR4_PGE: ULONG_PTR = 0x80;

// ---------------------------------------------------------------------------
// Globals (UCHAR HalpSerialLen, CHAR HalpSerialNumber[31] in C).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub static mut HalpSerialLen: u8 = 0;

#[unsafe(no_mangle)]
pub static mut HalpSerialNumber: [i8; 31] = [0; 31];

// ---------------------------------------------------------------------------
// Wide-string helper — like ntoskrnl-rust's `wstr!` but appends a trailing
// zero so the result can flow straight into RtlInitUnicodeString.
// ---------------------------------------------------------------------------

macro_rules! wstrz {
    ($s:literal) => {{
        const __W: &[u16] = &{
            const BYTES: &[u8] = $s.as_bytes();
            let mut out = [0u16; BYTES.len() + 1];
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

// ---------------------------------------------------------------------------
// Inline-asm intrinsics (replicas of MSVC's __readcr3 / __writecr3 /
// __readcr4 / __writecr4 / __readeflags / __writeeflags / _disable / __cpuid).
// Local to misc.rs since spinlock.rs's helpers are private to that module.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: u32;
    unsafe { core::arch::asm!("pushfd", "pop {:e}", out(reg) f, options(nomem, preserves_flags)); }
    f as ULONG_PTR
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_eflags() -> ULONG_PTR {
    let f: u64;
    unsafe { core::arch::asm!("pushfq", "pop {}", out(reg) f, options(nomem, preserves_flags)); }
    f as ULONG_PTR
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    unsafe { core::arch::asm!("push {:e}", "popfd", in(reg) f as u32, options(nomem)); }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_eflags(f: ULONG_PTR) {
    unsafe { core::arch::asm!("push {}", "popfq", in(reg) f as u64, options(nomem)); }
}

#[inline(always)]
unsafe fn disable_interrupts() {
    unsafe { core::arch::asm!("cli", options(nomem, nostack)); }
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn read_cr3() -> ULONG_PTR {
    let v: u32;
    unsafe { core::arch::asm!("mov {:e}, cr3", out(reg) v, options(nomem, nostack, preserves_flags)); }
    v as ULONG_PTR
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_cr3() -> ULONG_PTR {
    let v: u64;
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) v, options(nomem, nostack, preserves_flags)); }
    v as ULONG_PTR
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn write_cr3(v: ULONG_PTR) {
    unsafe { core::arch::asm!("mov cr3, {:e}", in(reg) v as u32, options(nomem, nostack, preserves_flags)); }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_cr3(v: ULONG_PTR) {
    unsafe { core::arch::asm!("mov cr3, {}", in(reg) v as u64, options(nomem, nostack, preserves_flags)); }
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn read_cr4() -> ULONG_PTR {
    let v: u32;
    unsafe { core::arch::asm!("mov {:e}, cr4", out(reg) v, options(nomem, nostack, preserves_flags)); }
    v as ULONG_PTR
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn read_cr4() -> ULONG_PTR {
    let v: u64;
    unsafe { core::arch::asm!("mov {}, cr4", out(reg) v, options(nomem, nostack, preserves_flags)); }
    v as ULONG_PTR
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn write_cr4(v: ULONG_PTR) {
    unsafe { core::arch::asm!("mov cr4, {:e}", in(reg) v as u32, options(nomem, nostack, preserves_flags)); }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn write_cr4(v: ULONG_PTR) {
    unsafe { core::arch::asm!("mov cr4, {}", in(reg) v as u64, options(nomem, nostack, preserves_flags)); }
}

/// `__cpuid(out[4], leaf)` — execute `cpuid` with EAX=leaf, return EAX/EBX/ECX/EDX.
#[inline(always)]
unsafe fn cpuid(leaf: u32) -> [u32; 4] {
    let (a, b, c, d): (u32, u32, u32, u32);
    // amd64 reserves rbx for the LLVM register allocator (xchg dance the same
    // way the slist crate does).  i386 also reserves ebx — same fix.
    unsafe {
        core::arch::asm!(
            "xchg {ebx_save:e}, ebx",
            "cpuid",
            "xchg {ebx_save:e}, ebx",
            ebx_save = out(reg) b,
            inout("eax") leaf => a,
            out("ecx") c,
            out("edx") d,
            options(nostack, preserves_flags),
        );
    }
    [a, b, c, d]
}

// ---------------------------------------------------------------------------
// Registry helpers — !MINIHAL_ in the C, but we always build the full HAL
// here.  HalpOpenRegistryKey is the canonical wrapper that
// HalpReportSerialNumber + HalpMarkAcpiHal both lean on.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpOpenRegistryKey(
    KeyHandle: PHANDLE,
    RootKey: HANDLE,
    KeyName: PUNICODE_STRING,
    DesiredAccess: u32,
    Create: BOOLEAN,
) -> NTSTATUS {
    unsafe {
        // InitializeObjectAttributes is a #define in xdk/wdm.h — inline.
        let mut oa = core::mem::zeroed::<OBJECT_ATTRIBUTES>();
        oa.Length = core::mem::size_of::<OBJECT_ATTRIBUTES>() as u32;
        oa.RootDirectory = RootKey as _;
        oa.Attributes = OBJ_CASE_INSENSITIVE;
        oa.ObjectName = KeyName as _;
        oa.SecurityDescriptor = core::ptr::null_mut();
        oa.SecurityQualityOfService = core::ptr::null_mut();
        if Create != 0 {
            let mut disposition: ULONG = 0;
            ZwCreateKey(
                KeyHandle as _,
                DesiredAccess,
                &mut oa,
                0,
                core::ptr::null_mut(),
                REG_OPTION_VOLATILE,
                &mut disposition,
            )
        } else {
            ZwOpenKey(KeyHandle as _, DesiredAccess, &mut oa)
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpReportSerialNumber() {
    unsafe {
        let len = core::ptr::read_volatile(&raw const HalpSerialLen);
        if len == 0 {
            return;
        }

        let mut name = core::mem::zeroed::<UNICODE_STRING>();
        let key = wstrz!("\\Registry\\Machine\\Hardware\\Description\\System");
        RtlInitUnicodeString(&mut name as *mut _ as _, key.as_ptr());

        let mut handle: HANDLE = core::ptr::null_mut();
        let status = HalpOpenRegistryKey(
            &mut handle,
            core::ptr::null_mut(),
            &mut name as *mut _ as _,
            KEY_ALL_ACCESS,
            0,
        );
        if status >= 0 {
            let value_name = wstrz!("Serial Number");
            RtlInitUnicodeString(&mut name as *mut _ as _, value_name.as_ptr());
            ZwSetValueKey(
                handle,
                &mut name as *mut _ as _,
                0,
                REG_BINARY,
                (&raw const HalpSerialNumber) as *mut _,
                len as u32,
            );
            ZwClose(handle);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpMarkAcpiHal() -> NTSTATUS {
    unsafe {
        let value: ULONG =
            if core::ptr::read_volatile(&raw const HalDisableFirmwareMapper) != 0 { 1 } else { 0 };

        let mut name = core::mem::zeroed::<UNICODE_STRING>();
        let cs_path = wstrz!("\\REGISTRY\\MACHINE\\SYSTEM\\CURRENTCONTROLSET");
        RtlInitUnicodeString(&mut name as *mut _ as _, cs_path.as_ptr());

        let mut root: HANDLE = core::ptr::null_mut();
        let status = HalpOpenRegistryKey(
            &mut root,
            core::ptr::null_mut(),
            &mut name as *mut _ as _,
            KEY_ALL_ACCESS,
            0,
        );
        if status < 0 {
            return status;
        }

        let pnp_path = wstrz!("Control\\Pnp");
        RtlInitUnicodeString(&mut name as *mut _ as _, pnp_path.as_ptr());

        let mut sub: HANDLE = core::ptr::null_mut();
        let status = HalpOpenRegistryKey(
            &mut sub,
            root,
            &mut name as *mut _ as _,
            KEY_ALL_ACCESS,
            1, /* TRUE */
        );
        ZwClose(root);

        if status >= 0 {
            let dfm = wstrz!("DisableFirmwareMapper");
            RtlInitUnicodeString(&mut name as *mut _ as _, dfm.as_ptr());
            let v = value;
            let s = ZwSetValueKey(
                sub,
                &mut name as *mut _ as _,
                0,
                REG_DWORD,
                &v as *const _ as *mut _,
                core::mem::size_of::<ULONG>() as u32,
            );
            ZwClose(sub);
            return s;
        }

        status
    }
}

// ---------------------------------------------------------------------------
// HalpCheckPowerButton — non-ACPI no-op (apic/halinit_xpc.c implements
// the ACPI version separately).
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpCheckPowerButton() {
    // Nothing to do on non-ACPI.
}

// ---------------------------------------------------------------------------
// HalpFlushTLB — disable interrupts, optionally clear CR4.PGE, reload CR3
// to flush TLB, restore CR4 and EFLAGS.  The PGE dance ensures global
// pages also get flushed; if CPUID isn't available or PGE isn't supported,
// fall back to a plain CR3 reload.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalpFlushTLB() {
    use crate::x86::pcr::KeGetCurrentPrcb;
    unsafe {
        let flags = read_eflags();
        disable_interrupts();
        let pdir = read_cr3();

        // KeGetCurrentPrcb()->CpuID — non-zero means CPUID is present.
        if (*KeGetCurrentPrcb()).CpuID != 0 {
            let info = cpuid(1);
            // EDX bit 13 = PGE.
            if info[3] & 0x2000 != 0 {
                let cr4 = read_cr4();
                write_cr4(cr4 & !CR4_PGE);
                write_cr3(pdir);
                write_cr4(cr4);
                write_eflags(flags);
                return;
            }
        }

        // Legacy path — just kick CR3.
        write_cr3(pdir);
        write_eflags(flags);
    }
}

// ---------------------------------------------------------------------------
// HalSystemVectorDispatchEntry — FASTCALL, x86 has no software-vector
// dispatch table to consult.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86")]
#[unsafe(no_mangle)]
pub unsafe extern "fastcall" fn HalSystemVectorDispatchEntry(
    _Vector: ULONG,
    _FlatDispatch: *mut *mut PKINTERRUPT_ROUTINE,
    _NoConnection: *mut PKINTERRUPT_ROUTINE,
) -> u8 {
    0
}

#[cfg(target_arch = "x86_64")]
#[unsafe(no_mangle)]
pub unsafe extern "system" fn HalSystemVectorDispatchEntry(
    _Vector: ULONG,
    _FlatDispatch: *mut *mut PKINTERRUPT_ROUTINE,
    _NoConnection: *mut PKINTERRUPT_ROUTINE,
) -> u8 {
    0
}

// ---------------------------------------------------------------------------
// KeFlushWriteBuffer — x86 doesn't need a software write-combining flush.
// ---------------------------------------------------------------------------

#[unsafe(no_mangle)]
pub unsafe extern "system" fn KeFlushWriteBuffer() {
    // Empty — stronger CPU memory model on x86 makes this a no-op.
}

// ---------------------------------------------------------------------------
// i386-only NTAPI wrappers around the FASTCALL spinlock / IRQL primitives.
// The header (xdk/x86/ke.h) exposes Ke* names that get lowered to stdcall
// by the kernel-mode ABI; the actual fastcall implementations live in
// spinlock.rs / pic.c.  Drivers built against kernel-mode imports go
// through these.
// ---------------------------------------------------------------------------

#[cfg(target_arch = "x86")]
mod i386 {
    use super::*;

    extern "fastcall" {
        fn KfRaiseIrql(NewIrql: KIRQL) -> KIRQL;
        fn KfLowerIrql(NewIrql: KIRQL);
        fn KfAcquireSpinLock(SpinLock: PKSPIN_LOCK) -> KIRQL;
        fn KfReleaseSpinLock(SpinLock: PKSPIN_LOCK, OldIrql: KIRQL);
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn KeRaiseIrql(NewIrql: KIRQL, OldIrql: PKIRQL) {
        unsafe { *OldIrql = KfRaiseIrql(NewIrql); }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn KeLowerIrql(NewIrql: KIRQL) {
        unsafe { KfLowerIrql(NewIrql); }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn KeAcquireSpinLock(SpinLock: PKSPIN_LOCK, OldIrql: PKIRQL) {
        unsafe { *OldIrql = KfAcquireSpinLock(SpinLock); }
    }

    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn KeReleaseSpinLock(SpinLock: PKSPIN_LOCK, NewIrql: KIRQL) {
        unsafe { KfReleaseSpinLock(SpinLock, NewIrql); }
    }
}
