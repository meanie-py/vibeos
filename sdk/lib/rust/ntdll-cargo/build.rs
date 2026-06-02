//! Cargo-driven `ntdll.dll` link (i386). Manifest for the shared
//! `reactos-link` engine.
//!
//! cmake still owns the sibling static libs; run `ninja` in build-i386
//! once first. Out-of-band inputs (regenerate when their sources change):
//!   - `tools/gen_ntdll_asm.py`  → data/asm/dispatch-i386.s
//!   - `tools/gen_ntdll_res.py`  → data/ntdll-i386.rc.res (TODO; today a
//!     copy of cmake's ntdll.rc.res)

use reactos_link::{AsmDefineSet, Linker, ModuleKind};

fn main() {
    Linker::new("i686-kernel.json")
        // ntdll-rust DEFAULT features = ntdll.dll proper (no `vista`).
        .rust_staticlib("ntdll-rust")
        // The one remaining ntdll asm — pre-expanded (asm.inc + ks386.inc
        // resolved offline). arm64 dispatch.S has no includes; amd64 is a
        // later arch.
        .asm(
            &["sdk/lib/rust/ntdll-cargo/data/asm/dispatch-i386.s"],
            AsmDefineSet::Kernel,
        )
        .resource("data/ntdll-i386.rc.res")
        .spec2def("dll/ntdll/def/ntdll.spec", "ntdll.dll")
        // ntdll exports 48 `-stub` entries (Etw*/…) → link the generated
        // stub bodies or the .def exports go undefined.
        .spec_stubs()
        // Heavy C bodies reused as cmake-built .a (run ninja first). The
        // lean csrlib archive (csrlib_only) is also cmake-built.
        .static_libs(&[
            "dll/ntdll/libcsrlib_rust.a",
            "sdk/lib/rtl/librtl.a",
            "sdk/lib/rtl/librtl_vista.a",
            "sdk/lib/rtl/librtl_rust.a",
            "sdk/lib/rtl/libutf8.a",
            "sdk/lib/crt/liblibcntpr.a",
            "sdk/lib/crt/libcntpr_rust.a",
            "sdk/lib/crt/cnt-rust-target/i686-pc-windows-gnu/release/libcnt_rust_lib.a",
            "ntoskrnl/libntdllsys.a",
            "sdk/lib/apisets/libapisets.a",
            "sdk/lib/uuid/libuuid.a",
            "sdk/lib/vcruntime/libsetjmp.a",
            "sdk/lib/pseh/libpseh.a",
            "sdk/lib/crt/libchkstk.a",
        ])
        // ntdll.dll: user-mode DLL, AddressOfEntryPoint = 0 (entry symbol
        // ___ImageBase), image base 0x7c920000.
        .link(ModuleKind::Win32Dll {
            entry: "___ImageBase".into(),
            image_base: 0x7c92_0000,
            subsystem: Some("console:5.01".into()),
        });
}
