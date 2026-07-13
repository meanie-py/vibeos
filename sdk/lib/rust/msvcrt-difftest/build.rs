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
    // ── the WIDE half (wcs.c) ─────────────────────────────────────────
    // Same two-step, same reason: glibc would declare these `ref_*` with a
    // prototype msvcrt does not share, and the definition in wcs.c would then
    // be a conflicting-types error.  Note this is NOT the wchar_t width — the
    // refstub pulls <wchar.h> under the 16-bit remap precisely so that the
    // OTHER ~30 wide functions line up for free (see refstub/msvcrt.h).  These
    // three disagree in *shape*:
    //   wcstok   glibc is 3-arg (s, delim, saveptr); msvcrt's is 2-arg here.
    //   wcstol   glibc returns LP64 `long`; msvcrt's is __msvcrt_long (32-bit).
    //   wcstoul  ditto, unsigned.
    ("wcstok", "wchar_t *ref_wcstok(wchar_t*, const wchar_t*);"),
    (
        "wcstol",
        "__msvcrt_long ref_wcstol(const wchar_t*, wchar_t**, int);",
    ),
    (
        "wcstoul",
        "__msvcrt_ulong ref_wcstoul(const wchar_t*, wchar_t**, int);",
    ),
    // These two clash for a THIRD reason, and it is a subtle one: they are
    // declared in <stdlib.h> — a NARROW header, which the refstub pulls BEFORE
    // the 16-bit wchar_t remap.  So glibc types their `wchar_t*` parameter with
    // the HOST's 32-bit wchar_t, while wcs.c defines them with the Windows
    // 16-bit one.  (Their siblings `wcsrtombs`/`wcrtomb` live in <wchar.h>,
    // which is pulled AFTER the remap, and therefore need no help at all.)
    (
        "wcstombs",
        "size_t ref_wcstombs(char*, const wchar_t*, size_t);",
    ),
    ("wctomb", "int ref_wctomb(char*, wchar_t);"),
];

/// The 176 public symbols of `dll/win32/msvcrt/wcs.c` — the SECOND reference
/// TU, added for wave 4 (the wide-string family).
///
/// Ground truth (regenerate if wcs.c changes):
/// ```text
/// llvm-nm build-i386/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/wcs.c.obj \
///   | awk '$2=="T"{print $3}' | sed 's/^_//' | sed 's/@.*//' | sort -u
/// ```
/// The two TUs define DISJOINT symbol sets (verified: `comm -12` is empty), so
/// one union rename prelude covers both and cross-TU calls resolve ref_*→ref_*.
/// That is what lets wcs.c reach string.c's `fpnum_parse` / `memcpy` / `strlen`
/// — it imports all three — instead of silently binding to the host's.
///
/// Only ~14 of these are under test (the pure wide-string family). The rest —
/// the printf engine wcs.c instantiates twice from the real "printf.h", the
/// isw* ctype family, the NLS collation paths — are compiled as genuine,
/// untranscribed code and are INERT: nothing under test calls them. They are
/// here so the TU links.
const WCS_PUBLIC_SYMBOLS: &[&str] = &[
    "__swprintf_l",
    "_iswalnum_l",
    "_iswalpha_l",
    "_iswblank_l",
    "_iswcntrl_l",
    "_iswctype_l",
    "_iswdigit_l",
    "_iswgraph_l",
    "_iswlower_l",
    "_iswprint_l",
    "_iswpunct_l",
    "_iswspace_l",
    "_iswupper_l",
    "_iswxdigit_l",
    "_scprintf",
    "_scprintf_l",
    "_scprintf_p",
    "_scprintf_p_l",
    "_scwprintf",
    "_scwprintf_l",
    "_scwprintf_p_l",
    "_snprintf",
    "_snprintf_c",
    "_snprintf_c_l",
    "_snprintf_l",
    "_snprintf_s",
    "_snprintf_s_l",
    "_snwprintf",
    "_snwprintf_l",
    "_snwprintf_s",
    "_snwprintf_s_l",
    "_sprintf_l",
    "_sprintf_p_l",
    "_sprintf_s_l",
    "_swprintf",
    "_swprintf_c",
    "_swprintf_c_l",
    "_swprintf_p",
    "_swprintf_p_l",
    "_swprintf_s_l",
    "_towlower_l",
    "_towupper_l",
    "_vscprintf",
    "_vscprintf_l",
    "_vscprintf_p",
    "_vscprintf_p_l",
    "_vscwprintf",
    "_vscwprintf_l",
    "_vscwprintf_p_l",
    "_vsnprintf",
    "_vsnprintf_c",
    "_vsnprintf_c_l",
    "_vsnprintf_l",
    "_vsnprintf_s",
    "_vsnprintf_s_l",
    "_vsnwprintf",
    "_vsnwprintf_l",
    "_vsnwprintf_s",
    "_vsnwprintf_s_l",
    "_vsprintf_l",
    "_vsprintf_p",
    "_vsprintf_p_l",
    "_vsprintf_s_l",
    "_vswprintf",
    "_vswprintf_c",
    "_vswprintf_c_l",
    "_vswprintf_l",
    "_vswprintf_p_l",
    "_vswprintf_s_l",
    "_wcscoll_l",
    "_wcsdup",
    "_wcsicmp",
    "_wcsicmp_l",
    "_wcsicoll",
    "_wcsicoll_l",
    "_wcslwr",
    "_wcslwr_l",
    "_wcslwr_s",
    "_wcslwr_s_l",
    "_wcsncoll",
    "_wcsncoll_l",
    "_wcsnicmp",
    "_wcsnicmp_l",
    "_wcsnicoll",
    "_wcsnicoll_l",
    "_wcsnset",
    "_wcsnset_s",
    "_wcsrev",
    "_wcsset",
    "_wcsset_s",
    "_wcstod_l",
    "_wcstoi64",
    "_wcstoi64_l",
    "_wcstol_l",
    "_wcstombs_l",
    "_wcstombs_s_l",
    "_wcstoui64",
    "_wcstoui64_l",
    "_wcstoul_l",
    "_wcsupr",
    "_wcsupr_l",
    "_wcsupr_s",
    "_wcsupr_s_l",
    "_wcsxfrm_l",
    "_wctomb_l",
    "_wctomb_s_l",
    "_wtof",
    "_wtof_l",
    "_wtoi",
    "_wtoi64",
    "_wtoi64_l",
    "_wtoi_l",
    "_wtol",
    "_wtol_l",
    "arg_clbk_positional",
    "arg_clbk_valist",
    "create_positional_ctx_a",
    "create_positional_ctx_w",
    "iswalnum",
    "iswalpha",
    "iswblank",
    "iswcntrl",
    "iswctype",
    "iswdigit",
    "iswgraph",
    "iswlower",
    "iswprint",
    "iswpunct",
    "iswspace",
    "iswupper",
    "iswxdigit",
    "pf_printf_a",
    "pf_printf_w",
    "sprintf",
    "sprintf_s",
    "swprintf_s",
    "towctrans",
    "towlower",
    "towupper",
    "vsprintf",
    "vsprintf_s",
    "vswprintf_s",
    "wcrtomb",
    "wcrtomb_s",
    "wcscat",
    "wcscat_s",
    "wcschr",
    "wcscmp",
    "wcscoll",
    "wcscpy",
    "wcscpy_s",
    "wcscspn",
    "wcslen",
    "wcsncat",
    "wcsncat_s",
    "wcsncmp",
    "wcsncpy",
    "wcsncpy_s",
    "wcsnlen",
    "wcspbrk",
    "wcsrchr",
    "wcsrtombs",
    "wcsrtombs_s",
    "wcsspn",
    "wcsstr",
    "wcstod",
    "wcstok",
    "wcstok_s",
    "wcstol",
    "wcstombs",
    "wcstombs_s",
    "wcstoul",
    "wcsxfrm",
    "wctob",
    "wctomb",
    "wctomb_s",
];

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/msvcrt-difftest → repo root is 4 up.
    let repo = manifest.ancestors().nth(4).unwrap().to_path_buf();
    let refstub = manifest.join("refstub");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let msvcrt_dir = repo.join("dll/win32/msvcrt");
    let string_c = msvcrt_dir.join("string.c");
    let wcs_c = msvcrt_dir.join("wcs.c");
    for src in [&string_c, &wcs_c] {
        assert!(
            src.is_file(),
            "msvcrt reference source not found: {}",
            src.display()
        );
    }

    // The union of both TUs' public symbols.  They are disjoint by
    // construction (see WCS_PUBLIC_SYMBOLS) — assert it, because an overlap
    // would silently become a duplicate-symbol link error much later.
    let all_symbols: Vec<&str> = PUBLIC_SYMBOLS
        .iter()
        .chain(WCS_PUBLIC_SYMBOLS.iter())
        .copied()
        .collect();
    {
        let mut seen = std::collections::HashSet::new();
        for s in &all_symbols {
            assert!(
                seen.insert(*s),
                "string.c and wcs.c both define `{s}` — the union rename prelude \
                 would emit two `ref_{s}` definitions"
            );
        }
    }

    // ── the generated rename prelude ────────────────────────────────
    let mut prelude = String::new();
    prelude.push_str(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included (-include) at the top of BOTH reference TUs\n\
         * (string.c and wcs.c).  One union prelude, so a cross-TU call\n\
         * (wcs.c -> string.c's fpnum_parse/memcpy/strlen) resolves\n\
         * ref_* -> ref_* instead of escaping to the host libc.\n\
         */\n\
         #ifndef MSVCRT_REF_PRELUDE_H\n\
         #define MSVCRT_REF_PRELUDE_H\n\n\
         /* stage 0: hide the symbols whose glibc prototype DISAGREES with\n\
          * msvcrt's, so glibc never gets to declare them (a 32-bit `long`\n\
          * return, or — for wcstok — a different arity). */\n",
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
    for sym in &all_symbols {
        if LP32_LONG_SYMBOLS.iter().any(|(s, _)| s == sym) {
            continue;
        }
        prelude.push_str(&format!("#define {sym} ref_{sym}\n"));
        renamed += 1;
    }
    prelude.push_str(
        "\n/* stage 2: host libc + the Windows type model + the real header's\n\
         * include guard (so the TU's `#include \"msvcrt.h\"` no-ops).  This\n\
         * also pulls glibc's <wchar.h>/<wctype.h> UNDER the 16-bit wchar_t\n\
         * remap, which is what makes ~30 of wcs.c's wide definitions agree\n\
         * with glibc's declarations for free. */\n\
         #include <msvcrt.h>\n\n\
         /* stage 3: un-hide the stage-0 set and rename it for real, spelling\n\
          * out the Windows prototype the host would have gotten wrong. */\n",
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
        all_symbols.len(),
        "rename prelude must cover every public symbol of BOTH reference TUs"
    );

    let prelude_h = out.join("ref_prelude.h");
    fs::write(&prelude_h, &prelude).unwrap();

    // ── the reference TUs: the GENUINE string.c + wcs.c, renamed ────
    //
    // Both get the IDENTICAL flag set — the two -Werror= tripwires below are
    // the harness's best safety net, and a reference TU compiled without them
    // is a reference you cannot trust.  Hence one builder, called twice.
    let ref_tu = |src: &PathBuf| {
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
            .file(src);
        build.cargo_metadata(false);
        build
    };

    // The externals (errno / locale / thread data / NLS / heap).  Compiled in
    // its own cc::Build below so the rename prelude does NOT apply to it.
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

    // The msvcrt internals the CRATE UNDER TEST imports (_errno /
    // _invalid_parameter). Deliberately its OWN TU with NO refstub include, so
    // these land un-namespaced — the reference's copies are `msvcrt_ref_*` and
    // must stay unreachable from the subject (and vice versa: separate errno
    // storage is what lets classify assert each side independently).
    let mut subject_build = cc::Build::new();
    subject_build
        .warnings(false)
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .file(manifest.join("subjectimpl.c"));
    subject_build.cargo_metadata(false);

    // One archive, `libmsvcrt_ref.a` in OUT_DIR, holding all three objects.
    let mut objects: Vec<PathBuf> = Vec::new();
    refimpl_build.cargo_metadata(false);
    objects.extend(ref_tu(&string_c).compile_intermediates());
    objects.extend(ref_tu(&wcs_c).compile_intermediates());
    objects.extend(refimpl_build.compile_intermediates());
    objects.extend(subject_build.compile_intermediates());

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
    println!("cargo:rerun-if-changed={}", wcs_c.display());
    println!("cargo:rerun-if-changed={}", msvcrt_dir.join("bnum.h").display());
    // printf.h is REAL logic (wcs.c instantiates it twice), not a stub.
    println!("cargo:rerun-if-changed={}", msvcrt_dir.join("printf.h").display());
    println!("cargo:rerun-if-changed={}", manifest.join("refimpl.c").display());
    println!("cargo:rerun-if-changed={}", manifest.join("subjectimpl.c").display());
    println!("cargo:rerun-if-changed={}", refstub.display());
    println!("cargo:rerun-if-changed=build.rs");
}
