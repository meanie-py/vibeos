//! Compile the *real* ReactOS libcntpr reference logic for the host,
//! each public symbol renamed to `ref_<name>`, so the differential
//! tests can call cnt-rust's shim and the genuine reference in one
//! process and assert parity.
//!
//! The reference `.c` normally pull the NT crt header chain
//! (`crtdefs.h` …); we isolate the actual algorithm by including the
//! tiny logic header (`tcs*.h`) or the `.c` body with a minimal
//! `refstub/` (`tchar.h`, `precomp.h`) on the include path + host
//! libc. No QEMU, no kernel headers.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/cnt-difftest → repo root is 4 up.
    let repo = manifest.ancestors().nth(4).unwrap().to_path_buf();
    let refstub = manifest.join("refstub");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let crt = repo.join("sdk/lib/crt");

    // (ref symbol, preamble, included reference source)
    let units: &[(&str, String, PathBuf)] = &[
        (
            "strlen",
            "typedef char _TCHAR;\n#define _tcslen ref_strlen\n#include <stddef.h>".into(),
            crt.join("string/tcslen.h"),
        ),
        (
            "strncpy",
            "typedef char _TCHAR;\n#define _tcsncpy ref_strncpy\n#include <stddef.h>".into(),
            crt.join("string/tcsncpy.h"),
        ),
        (
            "strcmp",
            "typedef char _TCHAR;\n#define _tcscmp ref_strcmp".into(),
            crt.join("string/tcscmp.h"),
        ),
        (
            "strstr",
            "#define strstr ref_strstr".into(),
            crt.join("string/strstr.c"),
        ),
        (
            "atoi64",
            "#define _atoi64 ref_atoi64".into(),
            crt.join("string/atoi64.c"),
        ),
        (
            "memcpy",
            "#define __cdecl\n#define memcpy ref_memcpy".into(),
            crt.join("mem/memcpy.c"),
        ),
    ];

    let mut build = cc::Build::new();
    build
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false);
    for (sym, preamble, src) in units {
        let w = out.join(format!("ref_{sym}.c"));
        fs::write(&w, format!("{preamble}\n#include \"{}\"\n", src.display())).unwrap();
        build.file(&w);
        println!("cargo:rerun-if-changed={}", src.display());
    }
    println!("cargo:rerun-if-changed={}", refstub.display());
    // Link the reference .o files *directly* (not as a static archive).
    // build-script rustc-link-lib doesn't reliably reach integration
    // tests; trailing link-args do, and direct objects sidestep
    // archive member-pull ordering entirely.
    for obj in build.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}
