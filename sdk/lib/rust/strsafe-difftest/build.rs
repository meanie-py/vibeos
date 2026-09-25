//! Compile the genuine strsafe.h printf entry points for the host, wrapped as fixed-arity
//! `ref_*` functions, plus the shared wide-printf engine both sides call.
use std::env;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/strsafe-difftest -> repo root is 4 up.
    let repo = manifest.ancestors().nth(4).unwrap().to_path_buf();
    let strsafe = repo.join("sdk/include/psdk/strsafe.h");
    assert!(strsafe.is_file(), "missing {}", strsafe.display());
    for f in ["csrc/engine.c", "csrc/ref.c"] {
        println!("cargo:rerun-if-changed={f}");
    }
    println!("cargo:rerun-if-changed={}", strsafe.display());
    cc::Build::new()
        .file(manifest.join("csrc/engine.c"))
        .file(manifest.join("csrc/ref.c"))
        // refstub first: strsafe.h pulls <_mingw_unicode.h> and <specstrings.h>, which are
        // PSDK plumbing with nothing strsafe's logic needs.
        .include(manifest.join("refstub"))
        .define("STRSAFE_H_PATH", Some(format!("\"{}\"", strsafe.display()).as_str()))
        // WCHAR is 16-bit on every target we ship; make the host's wchar_t agree.
        .flag("-fshort-wchar")
        .warnings(false)
        .compile("strsafe_ref");
}
