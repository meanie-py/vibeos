//! Cargo-driven `ntoskrnl.exe` link (i386). Manifest for the shared
//! `reactos-link` engine — all machinery lives there.
//!
//! cmake still owns the ~20 sibling static libs; run `ninja ntoskrnl`
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
            // cnt CRT: C liblibcntpr.a then the cnt-rust archives (ctype
            // _isupper/_isxdigit + mem/string/printf), C-first for the
            // --allow-multiple-definition first-match — exactly what cmake's
            // `libcntpr` target links into ntoskrnl. (libcntpr_rust.a was
            // missing here, surfacing as `undefined _isupper` on a cold build.)
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/crt/libcntpr_rust.a",
            "sdk/lib/crt/cnt-rust-target/i686-pc-windows-gnu/release/libcnt_rust_lib.a",
            "sdk/lib/vcruntime/libsetjmp.a",
            "sdk/lib/wdmguid/libwdmguid.a",
            "sdk/lib/poguid/libpoguid.a",
            "sdk/lib/ioevent/libioevent.a",
            "hal/halx86/CMakeFiles/libhal.dir/libhal.a",
            "drivers/base/kdrosdbg/CMakeFiles/libkdcom.dir/libkdcom.a",
            "drivers/base/bootvid/CMakeFiles/libbootvid.dir/libbootvid.a",
            "sdk/lib/crt/libchkstk.a",
            "sdk/lib/rtl/librtl_rust.a",
        ])
        .link(ModuleKind::KernelExe {
            entry: "_KiSystemStartup@4".into(),
            image_base: 0x0040_0000,
        });
}
