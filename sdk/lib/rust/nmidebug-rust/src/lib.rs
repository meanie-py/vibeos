//! nmidebug-rust — Rust translation of `drivers/base/nmidebug/nmidebug.c`, the
//! NMI Debug Driver. Produces `nmidebug.sys`; the final link is driven by
//! `nmidebug-cargo` (reactos-link `ModuleKind::KernelDll`, entry =
//! `DriverEntry`, no exports). Builds for i386 + amd64 + arm64, matching the C
//! (which has no per-arch CMake guard): the NMI signature/clear Easter egg is
//! `_M_IX86`/`_M_AMD64` only; on arm64 `NmiDbgCallback` is a no-op that returns
//! TRUE, but the driver still registers it.
//!
//! Faithful low-level details preserved from the C:
//!   * `KiBugCheckData` is a cross-DLL ntoskrnl DATA export → declared via
//!     raw-dylib so rustc emits the `__imp_` thunk (a plain extern static won't
//!     link cross-DLL on i386).
//!   * `__indwordstring(0x80, NmiBegin, 1)` → `rep insd` inline asm.
//!   * i386 `__asm rcr KiBugCheckData[4], 8` → `rcr dword ptr` inline asm.
//!   * `((void(*)())&KiBugCheckData[4])()` — calling the (just-mutated) bytes at
//!     `&KiBugCheckData[4]` AS code — is a transmute of that address to a fn
//!     pointer followed by a call. `NmiBegin` points at a *writable* static
//!     buffer because `__indwordstring` writes the port reading into it.

#![no_std]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(unused_unsafe, dead_code, unused_variables)]

use ntkrnl_sys::ddk::*;

const STATUS_SUCCESS: NTSTATUS = 0;

// "NMI4NMI@" + NUL. The C points `NmiBegin` (PCHAR) at the string literal, but
// __indwordstring writes the port-0x80 reading into its first dword, so the
// backing store must be writable — hence a `static mut` array, not rodata.
static mut NMI_BEGIN_STORAGE: [u8; 9] = *b"NMI4NMI@\0";

// Cross-DLL ntoskrnl DATA exports. KiBugCheckData (ULONG_PTR[5]) is used on
// x86/amd64; KiEnableTimerWatchdog is `@ extern -arch=i386,arm` (absent on
// amd64) and only touched under the i386 path.
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut KiBugCheckData: [ULONG_PTR; 5];
}
#[cfg(target_arch = "x86")]
#[link(name = "ntoskrnl.exe", kind = "raw-dylib", modifiers = "+verbatim")]
extern "C" {
    static mut KiEnableTimerWatchdog: BOOLEAN;
}

extern "system" {
    fn KeRegisterNmiCallback(CallbackRoutine: PVOID, Context: PVOID) -> PVOID;
    #[cfg(target_arch = "x86")]
    fn RtlCompareMemory(Source1: PVOID, Source2: PVOID, Length: usize) -> usize;
}

/// `__indwordstring(Port, Buffer, Count)` == `rep insd`.
#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn indwordstring(port: u16, buffer: *mut u32, count: u32) {
    core::arch::asm!(
        "rep insd",
        in("dx") port,
        inout("edi") buffer => _,
        inout("ecx") count => _,
        options(nostack),
    );
}
#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn indwordstring(port: u16, buffer: *mut u32, count: u32) {
    core::arch::asm!(
        "rep insd",
        in("dx") port,
        inout("rdi") buffer => _,
        inout("ecx") count => _,
        options(nostack),
    );
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline(always)]
unsafe fn NmiClearFlag() {
    let nmi = core::ptr::addr_of!(NMI_BEGIN_STORAGE) as *const u8;
    // &KiBugCheckData[4] viewed as bytes.
    let p4 = (core::ptr::addr_of_mut!(KiBugCheckData) as *mut ULONG_PTR).add(4) as *mut u8;

    *p4 = (*p4).wrapping_sub(*nmi.add(3) | *nmi.add(7));
    *p4.add(3) |= 1;

    #[cfg(target_arch = "x86")]
    {
        // __asm rcr KiBugCheckData[4], 8
        let v = (core::ptr::addr_of_mut!(KiBugCheckData) as *mut ULONG_PTR).add(4);
        core::arch::asm!("rcr dword ptr [{}], 8", in(reg) v, options(nostack));
    }
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
unsafe extern "system" fn NmiDbgCallback(Context: PVOID, Handled: BOOLEAN) -> BOOLEAN {
    // Clear the NMI flag.
    NmiClearFlag();

    // Get NMI status signature.
    let nmi = core::ptr::addr_of_mut!(NMI_BEGIN_STORAGE) as *mut u32;
    indwordstring(0x80, nmi, 1);

    // ((void(*)())&KiBugCheckData[4])() — execute the just-mutated bytes.
    let target = (core::ptr::addr_of!(KiBugCheckData) as *const ULONG_PTR).add(4) as *const ();
    let f: extern "C" fn() = core::mem::transmute(target);
    f();

    // Handle the NMI safely.
    #[cfg(target_arch = "x86")]
    {
        let begin = core::ptr::addr_of!(NMI_BEGIN_STORAGE) as *const u8;
        let differ = RtlCompareMemory(begin as PVOID, begin.add(4) as PVOID, 4) != 4;
        *core::ptr::addr_of_mut!(KiEnableTimerWatchdog) = differ as BOOLEAN;
    }

    1 as BOOLEAN
}

#[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
unsafe extern "system" fn NmiDbgCallback(Context: PVOID, Handled: BOOLEAN) -> BOOLEAN {
    let _ = (Context, Handled);
    1 as BOOLEAN
}

#[no_mangle]
pub unsafe extern "system" fn DriverEntry(
    DriverObject: *mut _DRIVER_OBJECT,
    RegistryPath: *mut _UNICODE_STRING,
) -> NTSTATUS {
    // PAGED_CODE();
    let _ = (DriverObject, RegistryPath);

    // Register NMI callback.
    KeRegisterNmiCallback(NmiDbgCallback as *const () as PVOID, core::ptr::null_mut());

    // Return success.
    STATUS_SUCCESS
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        unsafe {
            core::arch::asm!("hlt");
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        core::hint::spin_loop();
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
