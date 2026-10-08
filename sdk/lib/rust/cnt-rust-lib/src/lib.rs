//! `cnt-rust-lib` — staticlib umbrella over `cnt-rust`'s `cabi`
//! layer. `extern crate cnt_rust` force-pulls its `#[no_mangle]`
//! libcntpr-replacement symbols into this `.a` so the cmake libcntpr
//! cohabitation swap gets one staticlib (linked into libcntpr, which
//! feeds both ntoskrnl and hal).

#![no_std]

extern crate cnt_rust;
// ctype is a separate crate (keeps cnt-rust's cgu layout untouched);
// pull its #[no_mangle] libcntpr-replacement symbols into this .a.
extern crate cnt_ctype;

// `rust_eh_personality` stub. The precompiled `core` /
// `compiler_builtins` rlibs ship panic=unwind (we don't rebuild std on
// stable), so their objects in libcntpr_rust.a carry an undefined
// `rust_eh_personality` reference even though the workspace is
// panic=abort. ntoskrnl resolves it via kernel-rust's own stub, but the
// freeldr / driver / ftfd / winsrv links that also pull libcntpr_rust.a
// have no other Rust runtime, so they'd be left with an undefined
// symbol (member-pull-order dependent). Provide the SAME strong stub
// every other Rust staticlib here uses (kernel-rust, winsrv-rust,
// smss-csrss-rust, …). Where libcntpr_rust.a cohabits with another
// staticlib that also defines it (ntoskrnl + kernel-rust, winsrv.dll +
// winsrv-rust), the resulting duplicate is absorbed by
// `--allow-multiple-definition` (now on EXE/SHARED/MODULE link flags —
// lld first-match). panic=abort never unwinds, so the body is never
// called. (A `global_asm ".weak"` here perturbed cnt-rust-lib's CRT
// symbol emission and broke MODULE relinks with lld "X was replaced" —
// the plain strong stub avoids the PE weak-external encoding entirely.)
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

// Single shared #[panic_handler] (cnt-rust is an rlib with none).
// Emits "PANIC:<file>:<line>\n" to QEMU debugcon (0xE9) on x86, then
// loops — same diagnostic body as the rtl-rust umbrella.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    unsafe {
        for &b in b"PANIC:" {
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
        }
        if let Some(loc) = info.location() {
            let file = loc.file().as_bytes();
            let mut i = 0usize;
            while i < file.len() {
                let b = *file.as_ptr().add(i);
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
                i += 1;
            }
            core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b':');
            let line = loc.line();
            if line == 0 {
                core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b'0');
            } else {
                let mut buf = [0u8; 10];
                let mut n = line;
                let mut len = 0u32;
                while n > 0 {
                    *buf.as_mut_ptr().add(len as usize) = b'0' + (n % 10) as u8;
                    n /= 10;
                    len += 1;
                }
                let mut j = len;
                while j > 0 {
                    j -= 1;
                    let b = *buf.as_ptr().add(j as usize);
                    core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b);
                }
            }
        }
        core::arch::asm!("out dx, al", in("dx") 0xe9u16, in("al") b'\n');
    }
    // arm64 has no debugcon port, and this used to be a bare `loop {}` there: a
    // panic in any driver linking libcntpr spun the CPU with nothing on serial
    // (scan_self_recursive flags the self-jump). Print the location through the
    // DebugService brk the kernel routes to the UART, then __fastfail, which the
    // kernel turns into KERNEL_SECURITY_CHECK_FAILURE.
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let mut msg = [0u8; 160];
        let mut n = 0usize;
        let mut put = |bytes: &[u8]| {
            for &b in bytes {
                if n < msg.len() {
                    msg[n] = b;
                    n += 1;
                }
            }
        };
        put(b"PANIC:");
        if let Some(loc) = info.location() {
            put(loc.file().as_bytes());
            put(b":");
            let mut digits = [0u8; 10];
            let mut line = loc.line();
            let mut len = 0usize;
            loop {
                digits[len] = b'0' + (line % 10) as u8;
                line /= 10;
                len += 1;
                if line == 0 {
                    break;
                }
            }
            while len > 0 {
                len -= 1;
                put(&digits[len..len + 1]);
            }
        }
        put(b"\n");
        core::arch::asm!("brk #0xf002", in("x0") 1u64, in("x1") msg.as_ptr(), in("x2") n as u64);
        // FAST_FAIL_FATAL_APP_EXIT
        core::arch::asm!("brk #0xf003", in("x0") 7u64, options(noreturn));
    }
    #[cfg(not(target_arch = "aarch64"))]
    loop {}
}
