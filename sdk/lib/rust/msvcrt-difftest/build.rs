//! Compile the *real* msvcrt string reference (`dll/win32/msvcrt/string.c`)
//! for the host, with every one of its 106 public symbols renamed to
//! `ref_<name>`, so the differential tests can call the genuine C msvcrt
//! and cnt-rust's `#[no_mangle]` shims in one process and assert parity.
//!
//! Nothing is transcribed: `string.c` is handed to `cc` *as the
//! translation unit*, exactly as it sits in the tree.  Two mechanisms make
//! that work on linux-x86_64:
//!
//!  1. A generated `-include ref_prelude.h` that (a) renames the 106
//!     public symbols with `#define <name> ref_<name>`, and (b) pulls
//!     `refstub/msvcrt.h`, which reuses the REAL header's include guard
//!     (`__WINE_MSVCRT_H`) so string.c's own `#include "msvcrt.h"` — which
//!     resolves to the real same-dir header and would drag in the whole
//!     NT/PSDK chain — expands to nothing.  See refstub/msvcrt.h.
//!
//!  2. `refstub/` on the include path, catching the three quoted includes
//!     that have no same-dir file: `winnls.h`, `wine/asm.h`, `wine/debug.h`.
//!     `bnum.h` is deliberately NOT stubbed — it is real logic (the base-10
//!     bignum behind strtod) and resolves to the genuine same-dir header.
//!
//! `refimpl.c` (compiled WITHOUT the rename prelude) supplies the externals
//! the host libc doesn't have.

use std::env;
use std::fs;
use std::path::PathBuf;

/// The 106 public symbols of `dll/win32/msvcrt/string.c`.
///
/// Ground truth (regenerate if string.c changes):
/// ```text
/// llvm-nm build-i386/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/string.c.obj \
///   | awk '$2=="T"{print $3}' | sed 's/^_//' | sed 's/@.*//' | sort -u
/// ```
/// Hardcoded rather than re-derived at build time so this host crate never
/// depends on an OS build having been run.
const PUBLIC_SYMBOLS: &[&str] = &[
    "I10_OUTPUT",
    "__STRINGTOLD",
    "__STRINGTOLD_L",
    "__strncnt",
    "_atodbl",
    "_atodbl_l",
    "_atof_l",
    "_atoflt",
    "_atoflt_l",
    "_atoi64",
    "_atoi64_l",
    "_atoi_l",
    "_atol_l",
    "_atoldbl",
    "_atoldbl_l",
    "_i64toa",
    "_i64toa_s",
    "_i64tow",
    "_i64tow_s",
    "_itoa",
    "_itoa_s",
    "_itow",
    "_itow_s",
    "_ltoa",
    "_ltoa_s",
    "_ltow",
    "_ltow_s",
    "_memccpy",
    "_memicmp",
    "_memicmp_l",
    "_strcoll_l",
    "_strdup",
    "_stricmp",
    "_stricmp_l",
    "_stricoll",
    "_stricoll_l",
    "_strlwr",
    "_strlwr_l",
    "_strlwr_s",
    "_strlwr_s_l",
    "_strncoll",
    "_strncoll_l",
    "_strnicmp",
    "_strnicmp_l",
    "_strnicoll",
    "_strnicoll_l",
    "_strnset",
    "_strnset_s",
    "_strrev",
    "_strset",
    "_strtod_l",
    "_strtoi64",
    "_strtoi64_l",
    "_strtol_l",
    "_strtoui64",
    "_strtoui64_l",
    "_strtoul_l",
    "_strupr",
    "_strupr_l",
    "_strupr_s",
    "_strupr_s_l",
    "_strxfrm_l",
    "_swab",
    "_ui64toa",
    "_ui64toa_s",
    "_ui64tow",
    "_ui64tow_s",
    "_ultoa",
    "_ultoa_s",
    "_ultow",
    "_ultow_s",
    "atof",
    "atoi",
    "atol",
    "fpnum_double",
    "fpnum_ldouble",
    "fpnum_parse",
    "memchr",
    "memcmp",
    "memcpy",
    "memset",
    "strcat",
    "strcat_s",
    "strchr",
    "strcmp",
    "strcoll",
    "strcpy",
    "strcpy_s",
    "strcspn",
    "strlen",
    "strncat",
    "strncat_s",
    "strncmp",
    "strncpy",
    "strncpy_s",
    "strnlen",
    "strpbrk",
    "strrchr",
    "strspn",
    "strstr",
    "strtod",
    "strtok",
    "strtok_s",
    "strtol",
    "strtoul",
    "strxfrm",
];

/// The three symbols whose msvcrt prototype uses the Windows 32-bit
/// `long` (`__msvcrt_long` == `int` on an LP64 host) but which glibc also
/// declares, with a 64-bit `long`.
///
/// The rename prelude lands *before* the host headers (that is the whole
/// point: glibc then declares `ref_memcpy`, `ref_strlen`, … with correct
/// prototypes, so string.c's forward calls to functions it defines later
/// are never implicit).  For these three that backfires — glibc would
/// declare `long ref_atol(const char*)` and string.c would then define
/// `int ref_atol(const char*)`: a hard conflicting-types error.
///
/// So they are hidden from glibc behind a throwaway name, then re-renamed
/// to `ref_*` after the host headers, with the Windows prototypes spelled
/// out.  Keeping `__msvcrt_long == int` is worth the ceremony: it is what
/// makes `_ltoa(-1, buf, 16)` yield "ffffffff" (the 32-bit unsigned
/// reinterpret msvcrt actually performs) instead of the host's
/// "ffffffffffffffff".
const LP32_LONG_SYMBOLS: &[(&str, &str)] = &[
    ("atol", "int ref_atol(const char*);"),
    ("strtol", "int ref_strtol(const char*, char**, int);"),
    ("strtoul", "unsigned int ref_strtoul(const char*, char**, int);"),
];

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/msvcrt-difftest → repo root is 4 up.
    let repo = manifest.ancestors().nth(4).unwrap().to_path_buf();
    let refstub = manifest.join("refstub");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let msvcrt_dir = repo.join("dll/win32/msvcrt");
    let string_c = msvcrt_dir.join("string.c");
    assert!(
        string_c.is_file(),
        "msvcrt reference source not found: {}",
        string_c.display()
    );

    // ── the generated rename prelude ────────────────────────────────
    let mut prelude = String::new();
    prelude.push_str(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included (-include) at the top of the string.c TU.\n\
         */\n\
         #ifndef MSVCRT_REF_PRELUDE_H\n\
         #define MSVCRT_REF_PRELUDE_H\n\n\
         /* stage 0: hide glibc's LP64 prototypes for the msvcrt functions\n\
          * whose Windows signature uses a 32-bit `long`. */\n",
    );
    for (sym, _) in LP32_LONG_SYMBOLS {
        prelude.push_str(&format!("#define {sym} msvcrt_ref_glibc_{sym}\n"));
    }
    prelude.push_str(
        "\n/* stage 1: rename every other public symbol.  This lands BEFORE the\n\
         * host headers, so glibc declares `ref_memcpy`, `ref_strlen`, … with\n\
         * the right prototypes and string.c's forward calls stay typed. */\n",
    );
    let mut renamed = 0usize;
    for sym in PUBLIC_SYMBOLS {
        if LP32_LONG_SYMBOLS.iter().any(|(s, _)| s == sym) {
            continue;
        }
        prelude.push_str(&format!("#define {sym} ref_{sym}\n"));
        renamed += 1;
    }
    prelude.push_str(
        "\n/* stage 2: host libc + the Windows type model + the real header's\n\
         * include guard (so string.c's `#include \"msvcrt.h\"` no-ops). */\n\
         #include <msvcrt.h>\n\n\
         /* stage 3: un-hide the LP32 `long` trio and rename it for real. */\n",
    );
    for (sym, proto) in LP32_LONG_SYMBOLS {
        prelude.push_str(&format!(
            "#undef {sym}\n#define {sym} ref_{sym}\n{proto}\n"
        ));
        renamed += 1;
    }
    prelude.push_str("\n#endif /* MSVCRT_REF_PRELUDE_H */\n");
    assert_eq!(
        renamed,
        PUBLIC_SYMBOLS.len(),
        "rename prelude must cover every public symbol"
    );

    let prelude_h = out.join("ref_prelude.h");
    fs::write(&prelude_h, &prelude).unwrap();

    // ── the reference TU: the GENUINE string.c, renamed ─────────────
    let mut build = cc::Build::new();
    build
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        .warnings(false)
        // NOTE: NO `-w` here, on purpose.  `-w` silently defeats the two
        // -Werror= flags below (verified: gcc drops the diagnostic entirely).
        //
        // Those two are the harness's single most important safety net.  A
        // missing stub macro degrades, under C's implicit-declaration rule,
        // into a *call* to an undefined int-returning function — code that
        // still links and is silently wrong.  Both known instances were
        // exactly this shape:
        //   * `ARRAY_SIZE(lps)` in strstr's KMP path → a bogus needle length;
        //   * `get_locinfo()` → an `int` return, truncating the locinfo
        //     pointer to 32 bits → SIGSEGV on the first deref.
        // Fail loud instead.
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        // glibc's _FORTIFY_SOURCE inline redirects (Ubuntu turns it on at
        // -O2) would fight the `#define memcpy ref_memcpy` rename.
        .flag("-U_FORTIFY_SOURCE")
        // Keep the genuine bodies genuine: without this, gcc's loop-idiom
        // recognition happily rewrites string.c's own strlen/memset loops
        // into calls to the *host* strlen/memset.
        .flag("-fno-builtin")
        .flag("-fno-tree-loop-distribute-patterns")
        // The FP reference must not fuse — the Rust side doesn't.
        .flag("-ffp-contract=off")
        .flag("-include")
        .flag(prelude_h.to_str().unwrap())
        .opt_level(2)
        .file(&string_c)
        // The externals (errno / locale / thread data / NLS).  Compiled in
        // its own cc::Build below so the rename prelude does NOT apply to it.
        ;

    let mut refimpl_build = cc::Build::new();
    refimpl_build
        .include(&refstub)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        .warnings(false)
        // Same tripwire, same reason (see above) — refimpl.c is where a
        // typo'd stub name would otherwise become an implicit `int f()`.
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .file(manifest.join("refimpl.c"));

    // One archive, `libmsvcrt_ref.a` in OUT_DIR, holding both objects.
    let mut objects: Vec<PathBuf> = Vec::new();
    build.cargo_metadata(false);
    refimpl_build.cargo_metadata(false);
    objects.extend(build.compile_intermediates());
    objects.extend(refimpl_build.compile_intermediates());

    let archive = out.join("libmsvcrt_ref.a");
    let _ = fs::remove_file(&archive);
    let ar = env::var("AR").unwrap_or_else(|_| "ar".to_string());
    let status = std::process::Command::new(&ar)
        .arg("crs")
        .arg(&archive)
        .args(&objects)
        .status()
        .expect("failed to run ar");
    assert!(status.success(), "ar failed building {}", archive.display());

    // Link the reference objects DIRECTLY rather than as an archive: a
    // build-script `rustc-link-lib` doesn't reliably reach integration
    // tests, and direct objects sidestep archive member-pull ordering (the
    // ref_* symbols are pulled in even though nothing in the rlib
    // references them yet).  Same trick as cnt-difftest.
    for obj in &objects {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
    // floor / log10 (fpnum + I10_OUTPUT).
    println!("cargo:rustc-link-lib=dylib=m");

    println!("cargo:rerun-if-changed={}", string_c.display());
    println!("cargo:rerun-if-changed={}", msvcrt_dir.join("bnum.h").display());
    println!("cargo:rerun-if-changed={}", manifest.join("refimpl.c").display());
    println!("cargo:rerun-if-changed={}", refstub.display());
    println!("cargo:rerun-if-changed=build.rs");
}
