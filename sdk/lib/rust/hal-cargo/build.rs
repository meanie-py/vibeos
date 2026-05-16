//! Cargo-driven `hal.dll` link (i386). Manifest for the shared
//! `reactos-link` engine — all machinery lives there.
//!
//! Out-of-band inputs (regenerate when their sources change):
//!   - `tools/gen_preprocessed_asm_hal.py` → data/asm/hal-*-i386.s
//!   - `tools/gen_hal_res.py`              → data/hal-i386.rc.res
//!
//! Post-link pefixup / fix_lld_forwarding.py run separately (see
//! build-and-test.sh).

use reactos_link::{AsmDefineSet, Linker, ModuleKind};

fn main() {
    Linker::new("i686-kernel.json")
        .rust_staticlib("hal-rust")
        .asm(
            &[
                "sdk/lib/rust/hal-cargo/data/asm/hal-v86-i386.s",
                "sdk/lib/rust/hal-cargo/data/asm/hal-trap-i386.s",
                "sdk/lib/rust/hal-cargo/data/asm/hal-systimer-i386.s",
                "sdk/lib/rust/hal-cargo/data/asm/hal-pic-i386.s",
            ],
            AsmDefineSet::Hal,
        )
        .resource("data/hal-i386.rc.res")
        .spec2def("hal/hal.spec", "hal.dll")
        // cmake link order (from the `hal.dll` ninja line). libhal_rust.a
        // is emitted first by the engine. The ntoskrnl import lib is
        // self-generated from ntoskrnl.spec (.import_lib below) — no
        // cmake-built libntoskrnl.a needed. The remaining leaf static
        // libs still come from cmake (deferred mechanical pass).
        .static_libs(&[
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/vcruntime/libsetjmp.a",
            "sdk/lib/drivers/arbiter/libarbiter.a",
            "sdk/lib/fast486/libfast486.a",
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/crt/libchkstk.a",
            "sdk/lib/crt/libmemcmp.a",
        ])
        .import_lib("ntoskrnl/ntoskrnl.spec", "ntoskrnl.exe", "ntoskrnl")
        .link(ModuleKind::KernelDll {
            entry: "_HalInitSystem@8".into(),
            image_base: 0x0001_0000,
        });
}
