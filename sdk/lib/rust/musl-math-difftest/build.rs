//! Compile musl's exp/log/pow sources (sdk/lib/rust/musl-math/ref) as the reference.
//! Their public names are renamed `musl_*` with -D, so they never meet the host libm. That
//! is safe only because ref/shim/math.h stands in for the host <math.h>: glibc's
//! mathcalls.h token-pastes the names and a -D rename corrupts it.
fn main() {
    let r = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../musl-math/ref");
    println!("cargo:rerun-if-changed={}", r.display());
    let mut b = cc::Build::new();
    b.include(r.join("shim")).include(&r).opt_level(2).flag("-ffp-contract=off").flag("-fno-builtin");
    b.define("hidden", Some(""));
    for n in ["exp", "exp2", "log", "log2", "pow", "expf", "exp2f", "logf", "log2f", "powf"] {
        b.define(n, Some(format!("musl_{n}").as_str()));
    }
    for e in std::fs::read_dir(&r).unwrap() {
        let p = e.unwrap().path();
        if p.extension().is_some_and(|x| x == "c") {
            b.file(p);
        }
    }
    b.warnings(false).compile("muslref");
}
