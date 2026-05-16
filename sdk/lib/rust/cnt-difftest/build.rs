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

    // (ref symbol, full host TU body). Each renames the public symbol
    // to ref_<name> and includes the genuine ReactOS logic.
    let inc = |p: PathBuf| format!("#include \"{}\"\n", p.display());
    let s = |p: &str| crt.join(p);
    let units: Vec<(&str, String)> = vec![
        ("strlen", format!("typedef char _TCHAR;\n#define _tcslen ref_strlen\n#include <stddef.h>\n{}", inc(s("string/tcslen.h")))),
        ("strncpy", format!("typedef char _TCHAR;\n#define _tcsncpy ref_strncpy\n#include <stddef.h>\n{}", inc(s("string/tcsncpy.h")))),
        ("strcmp", format!("typedef char _TCHAR;\n#define _tcscmp ref_strcmp\n{}", inc(s("string/tcscmp.h")))),
        ("strstr", format!("#define strstr ref_strstr\n{}", inc(s("string/strstr.c")))),
        ("atoi64", format!("#define _atoi64 ref_atoi64\n{}", inc(s("string/atoi64.c")))),
        ("memcpy", format!("#define __cdecl\n#define memcpy ref_memcpy\n{}", inc(s("mem/memcpy.c")))),
        ("memchr", format!("#define __cdecl\n#define memchr ref_memchr\n{}", inc(s("mem/memchr.c")))),
        ("memcmp", format!("#define __cdecl\n#define memcmp ref_memcmp\n{}", inc(s("mem/memcmp.c")))),
        ("memmove", format!("#define __cdecl\n#define memmove ref_memmove\n{}", inc(s("mem/memmove.c")))),
        ("memset", format!("#define __cdecl\n#define memset ref_memset\n{}", inc(s("mem/memset.c")))),
        // atol = (long)_atoi64; skip atol.c's _atoldbl (needs _LDOUBLE).
        // Rename the helper so it doesn't collide with cnt-rust's own
        // #[no_mangle] `_atoi64` shim linked into this test binary.
        ("atol", format!("#define _atoi64 ref_atol_atoi64\n{}long ref_atol(const char*s){{return (long)ref_atol_atoi64(s);}}\n", inc(s("string/atoi64.c")))),
    ];

    let mut build = cc::Build::new();
    build
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false);
    for (sym, body) in &units {
        let w = out.join(format!("ref_{sym}.c"));
        fs::write(&w, body).unwrap();
        build.file(&w);
    }
    println!("cargo:rerun-if-changed={}", crt.display());
    println!("cargo:rerun-if-changed={}", refstub.display());
    // Link the reference .o files *directly* (not as a static archive).
    // build-script rustc-link-lib doesn't reliably reach integration
    // tests; trailing link-args do, and direct objects sidestep
    // archive member-pull ordering entirely.
    for obj in build.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}
