//! Cargo-driven `ntoskrnl.exe` link (i386). Manifest for the shared
//! `reactos-link` engine — all machinery lives there.
//!
//! Tier-B sibling Rust crates (rtl-rust / cnt-rust-lib) are now cargo-built
//! by the engine via `rust_lib()`; cmake still owns the deep-C / cohab static
//! libs (librtl.a, liblibcntpr.a, libhal.a, …). Run `ninja ntoskrnl`
//! once first (also generates the asm includes: ks386.inc, sysfuncs.h).
//! The asm is assembled from SOURCE `.S` by the engine (kernel asm -I
//! chain), so there is no offline pre-expansion. Only out-of-band input:
//!   - `tools/gen_ntoskrnl_res.py`      → data/ntoskrnl-i386.rc.res
//!
//! Post-link pefixup / fix_lld_forwarding.py run separately.

use reactos_link::{AsmDefineSet, Linker, ModuleKind};

fn main() {
    Linker::new("i686-kernel.json")
        .rust_staticlib("kernel-rust")
        // Tier-B sibling Rust crates: cargo-compiled directly by the engine (was
        // reuse of the cmake-built librtl_rust.a / libcntpr_rust.a /
        // libcnt_rust_lib.a). Built against the kernel json (x86-softfloat) +
        // build-std, exactly like the main kernel-rust crate. The engine emits
        // these AFTER the cmake C archives, preserving the required C-first cohab
        // precedence (liblibcntpr.a / librtl.a still own the shared mem/string +
        // not-yet-flipped Rtl* symbols). cnt-rust-lib gets the i386 -sse +
        // class-wide `-C no-prepopulate-passes` guard from the engine, and is
        // built WITHOUT the `float` feature (no-SSE softfloat kernel) — mirroring
        // sdk/lib/crt/libcntpr.cmake's CNT_RUST_FEATURES="" on i386.
        .rust_lib("rtl-rust", &[], true)
        .rust_lib("cnt-rust-lib", &[], true)
        // 8 header-free C shims (no kernel -I; clang's own
        // stdarg/stddef/stdint suffices).
        .c_shims(&[
            "ntoskrnl/io/iomgr/io_rust_shims.c",
            "ntoskrnl/ke/ke_rust_shims.c",
            "ntoskrnl/mm/mm_rust_shims.c",
            "ntoskrnl/variadic_shims.c",
            "ntoskrnl/lto_workaround.c",
            "ntoskrnl/mm_lock_outlines.c",
            "ntoskrnl/ntprintf_trampoline.c",
            "ntoskrnl/kdbg/kdbg_rust_shims.c",
        ])
        // Self-hosted asm: the engine assembles the SOURCE `.S` directly via its
        // kernel asm -I chain (asm.inc + the generated build-i386 ks386.inc +
        // ntoskrnl/include's asmmacro.S / sysfuncs.h) — no offline pre-expansion
        // (the old tools/gen_preprocessed_asm.py + checked-in data/asm/*.s).
        .asm(
            &[
                "ntoskrnl/ex/zw.S",
                "ntoskrnl/ex/i386/fastinterlck_asm.S",
                "ntoskrnl/ex/i386/ioport.S",
                "ntoskrnl/ke/i386/ctxswitch.S",
                "ntoskrnl/ke/i386/trap.s",
                "ntoskrnl/ke/i386/usercall_asm.S",
            ],
            AsmDefineSet::Kernel,
        )
        .resource("data/ntoskrnl-i386.rc.res")
        .spec2def("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe")
        // cmake link order; libkernel_rust.a is emitted first by the
        // engine so its symbols are visible to the cmake-built libs.
        .static_libs(&[
            "sdk/lib/drivers/ntoskrnl_vista/libntoskrnl_vista.a",
            "sdk/lib/cportlib/libcportlib.a",
            "sdk/lib/drivers/csq/libcsq.a",
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/drivers/arbiter/libarbiter.a",
            "sdk/lib/cmlib/libcmlib.a",
            "sdk/lib/lsalib/libntlsalib.a",
            "sdk/lib/rtl/librtl.a",
            "sdk/lib/rossym/librossym.a",
            // cnt CRT: only the C liblibcntpr.a here now — the cnt-rust archives
            // (ctype _isupper/_isxdigit + mem/string/printf umbrella) are built
            // by the engine via rust_lib("cnt-rust-lib") above and emitted after
            // this group, preserving the C-first --allow-multiple-definition
            // first-match cmake's `libcntpr` target relies on.
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/vcruntime/libsetjmp.a",
            "sdk/lib/wdmguid/libwdmguid.a",
            "sdk/lib/poguid/libpoguid.a",
            "sdk/lib/ioevent/libioevent.a",
            "hal/halx86/CMakeFiles/libhal.dir/libhal.a",
            "drivers/base/kdrosdbg/CMakeFiles/libkdcom.dir/libkdcom.a",
            "drivers/base/bootvid/CMakeFiles/libbootvid.dir/libbootvid.a",
            "sdk/lib/crt/libchkstk.a",
            // librtl_rust.a dropped — rtl-rust is now built by rust_lib() above.
        ])
        .link(ModuleKind::KernelExe {
            entry: "_KiSystemStartup@4".into(),
            image_base: 0x0040_0000,
        });
}
