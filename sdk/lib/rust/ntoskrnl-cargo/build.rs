//! Cargo-driven `ntoskrnl.exe` link (i386). Manifest for the shared
//! `reactos-link` engine — all machinery lives there.
//!
//! cmake still owns the ~20 sibling static libs; run `ninja ntoskrnl`
//! once first. Out-of-band inputs (regenerate when their sources
//! change):
//!   - `tools/gen_preprocessed_asm.py`  → data/asm/*-i386.s
//!   - `tools/gen_ntoskrnl_res.py`      → data/ntoskrnl-i386.rc.res
//!
//! The vestigial xdk/bugcodes/ks386 generators from the old hand-written
//! build.rs were dropped: the 8 C shims are header-free and the asm is
//! pre-expanded, so their outputs were already unused.
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
        .asm(
            &[
                "sdk/lib/rust/ntoskrnl-cargo/data/asm/zw-i386.s",
                "sdk/lib/rust/ntoskrnl-cargo/data/asm/fastinterlck-i386.s",
                "sdk/lib/rust/ntoskrnl-cargo/data/asm/ioport-i386.s",
                "sdk/lib/rust/ntoskrnl-cargo/data/asm/ctxswitch-i386.s",
                "sdk/lib/rust/ntoskrnl-cargo/data/asm/trap-i386.s",
                "sdk/lib/rust/ntoskrnl-cargo/data/asm/usercall-i386.s",
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
            "sdk/lib/crt/liblibcntpr.a",
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
