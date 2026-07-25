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
/// Symbols the refstub deliberately routes into the private `msvcrt_ref_*`
/// namespace (refstub/msvcrt.h) rather than renaming to `ref_*`.
///
/// That namespace is the harness's anti-circularity guarantee: it is what stops
/// the REFERENCE from ever calling into the CODE UNDER TEST. A symbol in here is
/// therefore NOT an oracle — the compiled reference does not define an msvcrt
/// version of it — and anything relying on `ref_<name>` will fail to link.
///
/// Consequence: these functions CANNOT be flipped, because they cannot be proved.
/// Exempt from the drift assertion below (they are intentionally absent from the
/// rename lists, not missing from them).
const NAMESPACED_SYMBOLS: &[&str] = &["memmove"];

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
    "_mbsdup",
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
    // ── mbcs.c, and exactly the <stdlib.h> story above ────────────────
    // Both are declared in <stdlib.h>, which the refstub pulls BEFORE the 16-bit
    // wchar_t remap, so glibc types their `wchar_t*` with the HOST's 32-bit one while
    // mbcs.c defines them with the Windows 16-bit one. Without the two-step these are
    // a conflicting-types error, which is how they were found.
    ("mbtowc", "int ref_mbtowc(wchar_t*, const char*, size_t);"),
    (
        "mbstowcs",
        "size_t ref_mbstowcs(wchar_t*, const char*, size_t);",
    ),
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
    "swprintf",
    "_scwprintf_p",
    "iswalpha_l",
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

/// `ctype.c` — reference TU #3 (wave 10).
///
/// This is the TU that makes *locale-aware* flips gradeable. Every one of these is
/// either a `locinfo->pctype[]` lookup (the `is*` family) or a `locinfo->pcumap[]` /
/// `pclmap[]` lookup (the case-mapping family) — i.e. they read the locale's OWN
/// tables. A Rust body that reads the same tables inherits the C's locale-correctness
/// by construction, which is precisely why the "C"-locale pinning (§9.60) does NOT
/// block this family the way it blocks a hardcoded-ASCII implementation.
///
/// ctype.c also brings two DATA symbols with it, deliberately NOT renamed:
///   * `MSVCRT__ctype[257]`  — the narrow ctype table; `pctype` is this **+ 1**
///     (which is why `_isctype_l` accepts `c >= -1`: `pctype[-1]` is the EOF slot).
///   * `MSVCRT__wctype[]` / `MSVCRT__pwctype` — the wide table.
/// Both are shared ENVIRONMENT, not code under test — the same argument that makes
/// `get_locinfo()` shared. Reference and subject must agree on what the locale *is*,
/// or the comparison is meaningless. (Wave 9 hand-wrote a 256-entry `MSVCRT__pwctype`
/// in subjectimpl.c precisely because ctype.c was not yet a reference TU; that
/// approximation is now deleted in favour of msvcrt's real table.)
const CTYPE_PUBLIC_SYMBOLS: &[&str] = &[
    "__p__pctype",
    "__pctype_func",
    "__p__pwctype",
    "__pwctype_func",
    "_isctype_l",
    "_isctype",
    "_isalnum_l",
    "isalnum",
    "_isalpha_l",
    "isalpha",
    "_iscntrl_l",
    "iscntrl",
    "_isdigit_l",
    "isdigit",
    "_isgraph_l",
    "isgraph",
    "_isleadbyte_l",
    "isleadbyte",
    "_islower_l",
    "islower",
    "_isprint_l",
    "isprint",
    "ispunct",
    "_isspace_l",
    "isspace",
    "_isupper_l",
    "isupper",
    "_isxdigit_l",
    "isxdigit",
    "_isblank_l",
    "isblank",
    "__isascii",
    "__toascii",
    "iswascii",
    "__iscsym",
    "__iscsymf",
    "__iswcsym",
    "__iswcsymf",
    "_toupper_l",
    "toupper",
    "_toupper",
    "_tolower_l",
    "tolower",
    "_tolower",
];

/// Lift a `static const unsigned char NAME[256] = { 0x00, 0x01, ... };` table out of
/// a C source, verbatim.
///
/// Used for locale.c's `cloc_clmap`/`cloc_cumap` — the C locale's lower/upper mapping
/// tables. They are `static`, so the harness cannot link them; parsing them is how we
/// get the REAL bytes without hand-transcribing (see the call site for why a hand copy
/// would be invisible-wrong).
fn extract_u8_table(src: &str, name: &str) -> Option<Vec<u8>> {
    let decl = format!("{name}[256]");
    let start = src.find(&decl)?;
    let open = src[start..].find('{')? + start;
    let close = src[open..].find('}')? + open;
    let mut out = Vec::new();
    for tok in src[open + 1..close].split(',') {
        let t = tok.trim();
        if t.is_empty() {
            continue;
        }
        let v = if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
            u8::from_str_radix(hex, 16).ok()?
        } else {
            t.parse::<u8>().ok()?
        };
        out.push(v);
    }
    Some(out)
}

/// Every `NAME` in a Wine export banner: ` *  NAME (MSVCRT.@)`.
fn exported_symbols(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix('*') else { continue };
        let rest = rest.trim();
        let Some(open) = rest.find('(') else { continue };
        if !rest[open..].starts_with("(MSVCRT.@)") {
            continue;
        }
        let name = rest[..open].trim();
        if !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_') {
            out.push(name.to_string());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Every non-`static` function the TU DEFINES, by scanning for definitions rather than
/// for `(MSVCRT.@)` banners.
///
/// The banner answers "what does msvcrt export", which is not the question the rename
/// list is asking. The hazard is a symbol that lands under its PLAIN name inside the
/// test binary, where the subject may also define it — and that set is what the TU
/// DEFINES. The two differ: mbcs.c defines seven functions with no banner at all
/// (`create_mbcinfo`, `_mbsncpy_s`, `_mbsncpy_s_l`, `_mbsnicmp_l`, `_mbsnset_l`,
/// `_mbstowcs_s`, `___mb_cur_max_l_func`), and the shipping Rust `#[no_mangle]`-exports
/// them. Renaming only the banner set leaves those as duplicate definitions — either a
/// link error, or one side silently satisfying the other's internal calls.
fn defined_symbols(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = src.as_bytes();
    for (i, line) in src.lines().enumerate() {
        // A definition starts at column 0 (msvcrt's house style) and is not `static`.
        if line.starts_with([' ', '\t', '#', '/', '*', '}']) || line.trim().is_empty() {
            continue;
        }
        if line.starts_with("static") || line.starts_with("typedef") {
            continue;
        }
        let Some(open) = line.find('(') else { continue };
        // The identifier immediately before '(' …
        let head = &line[..open];
        let name: String = head
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        if name.is_empty() || name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            continue;
        }
        // … and it is a DEFINITION only if a `{` follows the parameter list before any
        // `;` — otherwise it is a prototype or a call.
        let start = src.lines().take(i).map(|l| l.len() + 1).sum::<usize>() + open;
        let mut depth = 0i32;
        let mut is_def = false;
        for &b in &bytes[start..] {
            match b {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b';' if depth == 0 => break,
                b'{' if depth == 0 => {
                    is_def = true;
                    break;
                }
                _ => {}
            }
        }
        if is_def {
            out.push(name);
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Does the TU actually DEFINE `name` (`... name ( ... ) {`), as opposed to only
/// carrying its banner above an `#if`-disabled body?
fn defines_symbol(src: &str, name: &str) -> bool {
    let mut from = 0;
    while let Some(i) = src[from..].find(name) {
        let at = from + i;
        from = at + name.len();
        // must be a whole identifier
        let before_ok = at == 0
            || !src[..at].chars().next_back().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false);
        let after = &src[from..];
        if !before_ok {
            continue;
        }
        let after_trim = after.trim_start();
        if !after_trim.starts_with('(') {
            continue;
        }
        // scan to the matching ')' then require a '{' (a definition, not a call/decl)
        let mut depth = 0usize;
        for (k, c) in after_trim.char_indices() {
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        if after_trim[k + 1..].trim_start().starts_with('{') {
                            return true;
                        }
                        break;
                    }
                }
                ';' if depth == 0 => break,
                _ => {}
            }
        }
    }
    false
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/msvcrt-difftest → repo root is 4 up.
    let repo = manifest.ancestors().nth(4).unwrap().to_path_buf();
    let refstub = manifest.join("refstub");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    let msvcrt_dir = repo.join("dll/win32/msvcrt");
    let string_c = msvcrt_dir.join("string.c");
    let wcs_c = msvcrt_dir.join("wcs.c");
    let ctype_c = msvcrt_dir.join("ctype.c");
    let locale_c = msvcrt_dir.join("locale.c");
    let mbcs_c = msvcrt_dir.join("mbcs.c");
    for src in [&string_c, &wcs_c, &ctype_c, &locale_c, &mbcs_c] {
        assert!(
            src.is_file(),
            "msvcrt reference source not found: {}",
            src.display()
        );
    }

    // The union of all three TUs' public symbols.  They are disjoint by
    // construction (see WCS_PUBLIC_SYMBOLS / CTYPE_PUBLIC_SYMBOLS) — assert it,
    // because an overlap would silently become a duplicate-symbol link error much
    // later.
    // mbcs.c's 175 exports are DERIVED, not listed.
    //
    // The other three TUs carry hand-maintained lists plus a drift check that scrapes
    // the C and fails the build when one falls behind. Scraping is the check, so for a
    // new TU it may as well be the source: `exported_symbols` + `defines_symbol` are
    // the same two functions the drift check already trusts, and a list that is derived
    // cannot drift from the file it describes. (This is the difftest generator's own
    // thesis — the harness's symbol inventory was its first hardcoding — applied to the
    // harness that hosts it.)
    let mbcs_src = fs::read_to_string(&mbcs_c).unwrap();
    let mbcs_symbols: Vec<String> = defined_symbols(&mbcs_src)
        .into_iter()
        .filter(|name| !NAMESPACED_SYMBOLS.contains(&name.as_str()))
        .collect();
    assert!(
        mbcs_symbols.len() > 150,
        "mbcs.c scrape found only {} definitions — the scan is broken, and a short list \
         would silently leave the rest un-renamed and colliding with the subject",
        mbcs_symbols.len()
    );
    // The banner set must be a SUBSET of the definition set. If a banner names something
    // the scan did not find, the scan has a hole — and a hole here is a plain symbol
    // shared with the code under test, which is the one failure this list exists to
    // prevent.
    {
        let defined: std::collections::HashSet<&str> =
            mbcs_symbols.iter().map(String::as_str).collect();
        let missed: Vec<String> = exported_symbols(&mbcs_src)
            .into_iter()
            .filter(|n| defines_symbol(&mbcs_src, n))
            .filter(|n| !NAMESPACED_SYMBOLS.contains(&n.as_str()))
            .filter(|n| !defined.contains(n.as_str()))
            .collect();
        assert!(
            missed.is_empty(),
            "mbcs.c exports these with (MSVCRT.@) and defines them, but the definition \
             scan missed them — they would keep their plain names:\n    {}",
            missed.join("\n    ")
        );
    }

    let all_symbols: Vec<&str> = PUBLIC_SYMBOLS
        .iter()
        .chain(WCS_PUBLIC_SYMBOLS.iter())
        .chain(CTYPE_PUBLIC_SYMBOLS.iter())
        .copied()
        .chain(mbcs_symbols.iter().map(String::as_str))
        .collect();
    {
        let mut seen = std::collections::HashSet::new();
        for s in &all_symbols {
            assert!(
                seen.insert(*s),
                "two reference TUs both define `{s}` — the union rename prelude \
                 would emit two `ref_{s}` definitions"
            );
        }
    }

    // ── the C locale's case-mapping tables, LIFTED FROM THE SOURCE ──
    //
    // `_tolower_l`/`_toupper_l` are, for c < 256, exactly `locinfo->pclmap[c]` /
    // `pcumap[c]`. To grade them the harness's locinfo must carry the REAL tables —
    // but msvcrt keeps them `static const` inside locale.c, so they cannot be linked.
    //
    // Do NOT hand-transcribe them. A hand copy is invisible-wrong: reference and
    // subject share ONE locinfo, so a mistyped byte makes them both wrong the SAME
    // way and classify still reports MATCH. Parse them out of locale.c instead, so
    // the harness's tables move when msvcrt's do.
    let locale_src = fs::read_to_string(&locale_c).unwrap();
    let mut tables = String::from(
        "/* GENERATED by msvcrt-difftest/build.rs — lifted verbatim from\n\
         * dll/win32/msvcrt/locale.c so they cannot drift. Do not edit. */\n",
    );
    for name in ["cloc_clmap", "cloc_cumap"] {
        let body = extract_u8_table(&locale_src, name).unwrap_or_else(|| {
            panic!(
                "could not find `static const unsigned char {name}[256]` in locale.c — \
                 msvcrt's C-locale case-mapping tables moved. The harness cannot grade \
                 _tolower_l/_toupper_l without them."
            )
        });
        assert_eq!(
            body.len(),
            256,
            "{name} in locale.c has {} entries, expected 256",
            body.len()
        );
        tables.push_str(&format!(
            "static const unsigned char {name}[256] = {{{}}};\n",
            body.iter()
                .map(|b| b.to_string())
                .collect::<Vec<_>>()
                .join(",")
        ));
    }
    fs::write(out.join("cloc_tables.h"), &tables).unwrap();

    // ── <mbctype.h> / <mbstring.h>: forward to the GENUINE headers ──────────
    //
    // mbcs.c includes these, and they must be the real ones: mbctype.h is where `_M1`,
    // `_M2`, `_SBUP`, `_SBLOW`, `_MP` and `_MS` are DEFINED, and those flag bits are
    // the entire content of a `threadmbcinfo`. Transcribing them into refstub would put
    // the harness's idea of "lead byte" in a different file from msvcrt's, free to
    // drift — and a drifted flag is invisible, because both sides read the same fixture
    // and would read the same wrong bit.
    //
    // They cannot be reached by putting dll/win32/msvcrt/include on the include path:
    // that directory also holds assert.h, ctype.h, stdlib.h &c., which would capture
    // refstub/msvcrt.h's `#include <assert.h>` — and those MUST resolve to the host's.
    // (Tried it; it fails immediately on msvcrt's own assert.h.) So: a one-line
    // forwarder in OUT_DIR naming the real file absolutely, with refstub/corecrt.h
    // standing in for the PSDK chain it would otherwise drag in.
    for hdr in ["mbctype.h", "mbstring.h"] {
        fs::write(
            out.join(hdr),
            format!(
                "/* GENERATED by msvcrt-difftest/build.rs — forwards to the real header.\n\
                 * See build.rs for why this is a forwarder and not a transcription. */\n\
                 #include \"{}\"\n",
                msvcrt_dir.join("include").join(hdr).display()
            ),
        )
        .unwrap();
    }

    // ── the generated rename prelude ────────────────────────────────
    let mut prelude = String::new();
    prelude.push_str(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included (-include) at the top of ALL THREE reference TUs\n\
         * (string.c, wcs.c, ctype.c).  One union prelude, so a cross-TU call\n\
         * (wcs.c -> string.c's fpnum_parse/memcpy/strlen; string.c -> ctype.c's\n\
         * _tolower_l) resolves ref_* -> ref_* instead of escaping to the host libc.\n\
         */\n\
         #ifndef MSVCRT_REF_PRELUDE_H\n\
         #define MSVCRT_REF_PRELUDE_H\n\n\
         /* ═══ stage -1: glibc's ctype MACROS must not exist. ══════════════════\n\
          *\n\
          * <ctype.h> defines the whole isXXX/toXXX family as FUNCTION-LIKE MACROS:\n\
          *\n\
          *     #define isalnum(c) __isctype((c), _ISalnum)\n\
          *\n\
          * Stage 1 below renames `isalnum` -> `ref_isalnum` before the host headers\n\
          * are read, so glibc's line above is macro-expanded on the way in and we\n\
          * get `#define ref_isalnum(c) ...` — a function-like macro named exactly\n\
          * what ctype.c is about to DEFINE.  ctype.c's `int CDECL isalnum(int c)`\n\
          * then renames to `ref_isalnum(int c)`, which the preprocessor happily\n\
          * treats as an invocation with c = `int c`:\n\
          *\n\
          *     error: expected ')' before 'c'\n\
          *\n\
          * __NO_CTYPE is glibc's own documented switch for exactly this: it keeps\n\
          * the prototypes and suppresses the macros.  (This is why ctype.c could\n\
          * not become a reference TU until now — the string/wcs TUs never *defined*\n\
          * a name glibc macro-ises.)\n\
          *\n\
          * __NO_CTYPE does NOT cover the double-underscore spellings, which glibc\n\
          * defines unconditionally because it uses them internally:\n\
          *\n\
          *     #define __isascii(c) (((c) & ~0x7f) == 0)        <ctype.h>:99\n\
          *\n\
          * and ctype.c *defines functions* named `__isascii` / `__toascii`.  Those we\n\
          * have to #undef — and the ORDER matters, which cost a link error to learn:\n\
          * a macro name inside a `#define` directive is NOT macro-expanded, so glibc's\n\
          * line above does not become `#define ref___isascii(...)`; it CLOBBERS our\n\
          * stage-1 rename of the same name.  Undef it AFTER stage 1 and the symbol is\n\
          * left un-renamed, so ctype.c defines a PLAIN `__isascii` that collides with\n\
          * the Rust under test.\n\
          *\n\
          * So: pull <ctype.h> here, strip its macros here, and only THEN rename.  The\n\
          * plain-named prototypes it leaves behind are harmless — nothing calls them. */\n\
         #define __NO_CTYPE 1\n\
         #include <ctype.h>\n\
         #undef __isascii\n\
         #undef __toascii\n\
         #undef isascii\n\
         #undef toascii\n\n\
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
        "\n/* ═══ stage 1.5: DISARM glibc's optimization attributes. ═══════════════\n\
         *\n\
         * THIS IS LOAD-BEARING. Without it the reference is MISCOMPILED and the\n\
         * whole harness is lying to you. (§9.66 — it shipped this way through\n\
         * waves 1-5 and was found in wave 6a.)\n\
         *\n\
         * Stage 1 renames `atoi` -> `ref_atoi` BEFORE the host headers, which is\n\
         * deliberate: glibc then declares `ref_atoi` and we inherit a correct\n\
         * prototype for free.  But a declaration carries more than a prototype.\n\
         * glibc's <stdlib.h> says:\n\
         *\n\
         *     extern int atoi (const char *) __THROW __attribute_pure__ __nonnull ((1));\n\
         *\n\
         * and after stage 1 that is a declaration of *ref_atoi* carrying glibc's\n\
         * attributes.  Two of them are actively FALSE for msvcrt:\n\
         *\n\
         *   __nonnull((1))      glibc promises the arg is never NULL, so the\n\
         *                       optimizer DELETES msvcrt's `if(!str) return 0;`\n\
         *                       guard (string.c:1720).  Verified: the guard is\n\
         *                       absent from the emitted `ref_atoi` — the ONLY\n\
         *                       reason a NULL fuzz case segfaulted the reference\n\
         *                       instead of returning 0 like the real DLL does.\n\
         *\n\
         *   __attribute_pure__  glibc promises no side effects.  msvcrt's strtol\n\
         *                       family SETS ERRNO.  A `pure` errno-setting call\n\
         *                       may legally be CSE'd or elided outright — which\n\
         *                       would silently corrupt the very errno oracle the\n\
         *                       `_s`/parse rows are built on.  This one does not\n\
         *                       crash; it quietly reports the wrong answer.\n\
         *\n\
         * The fix is not -fno-delete-null-pointer-checks: that patches the NULL\n\
         * symptom and leaves `pure` armed.  Instead pull glibc's macro header\n\
         * FIRST and neuter the macros, so every subsequent declaration comes out\n\
         * attribute-free.  Prototypes (the thing we actually wanted) survive.\n\
         *\n\
         * `tests/ref_integrity.rs` asserts this stays true.\n\
         */\n\
         #include <sys/cdefs.h>\n\
         #undef  __nonnull\n\
         #define __nonnull(params)\n\
         #undef  __attribute_pure__\n\
         #define __attribute_pure__\n\
         #undef  __attribute_const__\n\
         #define __attribute_const__\n\
         #undef  __attr_access\n\
         #define __attr_access(x)\n\
         #undef  __attr_access_none\n\
         #define __attr_access_none(argno)\n\
         #undef  __wur\n\
         #define __wur\n\n\
         /* stage 2: host libc + the Windows type model + the real header's\n\
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

    // ── the rename list must not DRIFT from the C ────────────────────────────
    //
    // The lists above are hand-maintained. The assertion above only proves the
    // prelude covers THE LIST — it says nothing about whether the list still
    // covers the SOURCE. It had already drifted: `memmove`, `_mbsdup`,
    // `swprintf`, `_scwprintf_p` and `iswalpha_l` were exported by the C and
    // absent from the list, so the reference TU defined them under their PLAIN
    // names inside the test binary.
    //
    // That is not a cosmetic gap — it silently breaks the harness's central
    // guarantee. The whole point of the ref_* namespace is that the reference can
    // NEVER bind to the code under test and vice versa. An unrenamed `memmove` is
    // a plain `memmove` that either side may call, and the moment the subject
    // flips its own `memmove` it becomes a duplicate-symbol link error (which is
    // how this was found) — or, worse, silently satisfies the subject's internal
    // copies with the reference's C.
    //
    // So: scrape the C for every `(MSVCRT.@)` export that the file actually
    // DEFINES, and fail the build if one is missing from the lists. A hand-kept
    // list that nothing checks is a list that is already wrong.
    let mut undeclared: Vec<String> = Vec::new();
    // mbcs.c is deliberately absent: its list is DERIVED by this same scrape a few
    // dozen lines above, so checking it against itself would prove nothing.
    for (path, file) in [("string.c", &string_c), ("wcs.c", &wcs_c), ("ctype.c", &ctype_c)] {
        let src = fs::read_to_string(file).unwrap();
        for name in exported_symbols(&src) {
            // A banner can precede an #if'd-out body; only count real definitions.
            if !defines_symbol(&src, &name) {
                continue;
            }
            if NAMESPACED_SYMBOLS.contains(&name.as_str()) {
                continue; // intentionally private, not drifted — see the const's doc
            }
            if !all_symbols.iter().any(|s| *s == name) {
                undeclared.push(format!("{name} ({path})"));
            }
        }
    }
    assert!(
        undeclared.is_empty(),
        "\n\n  RENAME LIST HAS DRIFTED FROM THE C.\n\n  \
         These symbols are exported AND defined by the reference TUs but are not in\n  \
         PUBLIC_SYMBOLS / WCS_PUBLIC_SYMBOLS / LP32_LONG_SYMBOLS, so they would be\n  \
         compiled under their PLAIN names — escaping the ref_* namespace that keeps\n  \
         the reference and the subject from binding to each other:\n\n    {}\n\n  \
         Add each to the appropriate list (or to LP32_LONG_SYMBOLS if its glibc\n  \
         prototype disagrees).\n",
        undeclared.join("\n    ")
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
        .include(&out) // cloc_tables.h — the C locale's pclmap/pcumap, lifted from locale.c
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
        // refstub/ is on the path ONLY for locale_layout.h + the generated cloc tables.
        // subjectimpl.c must NEVER include refstub/msvcrt.h — that header carries the
        // anti-circularity namespace block, and pulling it here would rename the
        // SUBJECT's _errno/get_locinfo into msvcrt_ref_*, i.e. bind the code under test
        // to the reference's state. locale_layout.h is deliberately namespace-free.
        .include(&refstub)
        .include(&out)
        .warnings(false)
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .file(manifest.join("subjectimpl.c"));
    subject_build.cargo_metadata(false);

    // The codepage-aware NLS environment (iconv-backed) + the pinned MBCS fixture.
    //
    // Its own TU with NO refstub/msvcrt.h, like subjectimpl.c but for the opposite
    // reason: it must see the HOST's <wctype.h>/<iconv.h> under the host's 32-bit
    // wchar_t, and refstub/msvcrt.h remaps wchar_t to 16 bits. It reaches the shared
    // struct layouts through locale_layout.h, which is deliberately namespace-free.
    let mut nls_build = cc::Build::new();
    nls_build
        .include(&refstub)
        .include(&out)
        .warnings(false)
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .file(manifest.join("nlsimpl.c"));
    nls_build.cargo_metadata(false);

    // ── the printf PROBE unit: ReactOS `printf/streamout.c` as `so_*` ────
    //
    // NOT an oracle and NOT under test — a THIRD party, for one question:
    // does cnt-rust's already-Rust printf engine (`cabi::streamout`) agree
    // with msvcrt's `pf_printf_a` closely enough to ASSEMBLE rather than
    // port? (tests/printf_probe.rs — assemble-vs-port decision, wave 14.)
    //
    // We can't link cnt-rust here (its `cabi` feature ships the same 200+
    // `#[no_mangle]` names this crate's subject exports), so we compare BY
    // TRANSITIVITY: this TU is a VERBATIM copy of cnt-difftest's proven
    // `ref_streamout` recipe (same flat va_list <stdarg.h>, same bit-exact
    // floor/log10/pow model, same -mlong-double-64), i.e. the exact
    // configuration cnt-difftest proved byte-identical to the Rust engine.
    // so_streamout ≡ streamout-Rust, so `so_* vs ref_sprintf` answers
    // `streamout-Rust vs msvcrt`. It lives in the `so_*` namespace: no
    // rename prelude, no ref_* — nothing binds to it by accident.
    //
    // mbtowc/wctomb are stubbed to fail (-1) exactly as in cnt-difftest;
    // the Rust engine imports the same two externs, so wide-conversion rows
    // compare msvcrt's real MBCS against the engine's failure path — those
    // rows probe the GAP, not a bug in the recipe.
    let refstub_printf = manifest.join("refstub_printf");
    let cnt_refstub = repo.join("sdk/lib/rust/cnt-difftest/refstub");
    assert!(
        cnt_refstub.join("stdio.h").is_file(),
        "cnt-difftest refstub moved: {}",
        cnt_refstub.display()
    );
    let inc = |p: PathBuf| format!("#include \"{}\"\n", p.display());
    let so_body = format!(
        "#include <stddef.h>\n#include <math.h>\n#include <float.h>\n\
         #define _LIBCNT_\n#define __declspec(x)\n#define __int64 long long\n\
         #define _TCHAR char\n#define _T(x) x\n\
         #define _tcscpy strcpy\n#define _tcslen strlen\n#define _tcschr strchr\n\
         #define _isnan(x) isnan(x)\n#define _finite(x) isfinite(x)\n\
         static double cnt_floor(double x){{ if(!isfinite(x))return x;\
           long long i=(long long)x; double di=(double)i;\
           return di>x?di-1.0:di; }}\n\
         static double cnt_pow10i(int n){{ double r=1.0,b=10.0;\
           unsigned k=(n<0)?(unsigned)(-(long)n):(unsigned)n;\
           while(k){{if(k&1)r*=b;b*=b;k>>=1;}} return n<0?1.0/r:r; }}\n\
         static double cnt_powm(double a,double e){{ (void)a;\
           return cnt_pow10i((int)e); }}\n\
         static double cnt_log10(double f){{ if(!isfinite(f))return 308.;\
           if(f==0.0)return 0.0;\
           if(f<0.0)f=-f; union{{double d;unsigned long long u;}} v;\
           v.d=f; int biased=(int)((v.u>>52)&0x7ff);\
           int e=(int)(((double)(biased-1023))*0.3010299956639812);\
           while(cnt_pow10i(e)>f)e--; while(cnt_pow10i(e+1)<=f)e++;\
           return (double)e; }}\n\
         #define floor(x) cnt_floor(x)\n#define log10(x) cnt_log10(x)\n\
         #define pow(a,b) cnt_powm((a),(b))\n\
         static int so_mbtowc(wchar_t*w,const char*m,size_t n){{(void)w;(void)m;(void)n;return -1;}}\n\
         static int so_wctomb(char*m,wchar_t w){{(void)m;(void)w;return -1;}}\n\
         #define mbtowc so_mbtowc\n#define wctomb so_wctomb\n\
         #define streamout so_streamout\n#define format_float so_format_float\n\
         {}\
         /* driver: build a FILE and run the genuine engine over a flat\
            va_list (refstub_printf stdarg). `cnt` is the initial\
            _iobuf._cnt, i.e. the output bound. */\n\
         int so_run(char* base,int cnt,const char* fmt,char* va){{\
           FILE f; f._base=base; f._ptr=base; f._cnt=cnt;\
           f._flag=0x40|0x2; f._charbuf=0; f._bufsiz=0; f._tmpfname=0; f._file=0;\
           return so_streamout(&f, fmt, (va_list)va); }}\n",
        inc(repo.join("sdk/lib/crt/printf/streamout.c"))
    );
    let so_c = out.join("so_streamout.c");
    fs::write(&so_c, &so_body).unwrap();
    let mut so_build = cc::Build::new();
    so_build
        .include(&refstub_printf)
        .include(&cnt_refstub)
        .warnings(false)
        .flag("-w")
        .flag("-mlong-double-64")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&so_c);

    // The msvcrt side of the probe: wcs.c compiled a SECOND time, in the
    // `pfflat_*` namespace, under the FLAT `char*` va_list
    // (refstub_printf/stdarg.h). Two reasons this is a separate
    // instantiation instead of a call into the existing `ref_*` one:
    //
    //  1. The ref_* engine is not CALLABLE on this host. Two escapes were
    //     eaten before this landed: `ref__snprintf` forwards to plain
    //     `vsnprintf` (a symbol wcs.c never defines under _MSVCR_VER=0),
    //     which binds to GLIBC — the probe's first "msvcrt" column was
    //     the host libc ((nil), 2-digit exponents, no %I64). And
    //     `ref__vsnprintf` SIGSEGVs by construction: it forwards
    //     `&valist` as `va_list*`, coherent only when va_list is a plain
    //     pointer — under the SysV register-save-area va_list, `*valist`
    //     reinterprets the decayed pointer variable as the tag struct.
    //  2. The flat model IS the Windows-x64 va_list (char* over 8-byte
    //     slots), so this instantiation is the target-faithful one, and
    //     it consumes the SAME hand-packed buffer as so_run — a true
    //     apples-to-apples drive of both engines.
    //
    // prelude2 = prelude with every wcs.c-owned symbol retargeted
    // ref_→pfflat_; string.c/ctype.c symbols STAY ref_, so cross-TU calls
    // (memcpy, fpnum_parse, _isdigit_l…) resolve into the existing
    // reference objects and are not duplicated. tests/printf_probe.rs
    // carries an %I64d oracle-honesty canary so any future escape back
    // to the host libc fails loudly.
    let mut prelude2 = prelude.replace("MSVCRT_REF_PRELUDE_H", "MSVCRT_PFFLAT_PRELUDE_H");
    for sym in WCS_PUBLIC_SYMBOLS {
        prelude2 = prelude2.replace(
            &format!("#define {sym} ref_{sym}\n"),
            &format!("#define {sym} pfflat_{sym}\n"),
        );
    }
    // The five wide LP32 symbols are wcs.c-owned too; their stage-3 blocks
    // (rename + spelled-out prototype) both mention ref_<name>.
    for sym in ["wcstok", "wcstol", "wcstoul", "wcstombs", "wctomb"] {
        prelude2 = prelude2.replace(&format!("ref_{sym}"), &format!("pfflat_{sym}"));
    }
    assert!(
        prelude2.contains("#define _vsnprintf pfflat__vsnprintf"),
        "pfflat prelude transform failed — probe entry would collide with ref_*"
    );
    let prelude2_h = out.join("pfflat_prelude.h");
    fs::write(&prelude2_h, &prelude2).unwrap();

    let mut pfflat_build = cc::Build::new();
    pfflat_build
        // Own object dir: cc names intermediates by a hash of the SOURCE
        // path, and this is wcs.c's SECOND compilation — sharing OUT_DIR
        // would overwrite the ref_* object and link this one twice.
        .out_dir(out.join("pfflat"))
        .include(&refstub_printf) // flat <stdarg.h> FIRST — the whole point
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        .warnings(false)
        // Same tripwires + genuineness flags as ref_tu, same reasons.
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-fno-builtin")
        .flag("-fno-tree-loop-distribute-patterns")
        .flag("-ffp-contract=off")
        .flag("-include")
        .flag(prelude2_h.to_str().unwrap())
        .opt_level(2)
        .cargo_metadata(false)
        .file(&wcs_c);

    // One archive, `libmsvcrt_ref.a` in OUT_DIR, holding all the objects.
    let mut objects: Vec<PathBuf> = Vec::new();
    refimpl_build.cargo_metadata(false);
    objects.extend(ref_tu(&string_c).compile_intermediates());
    objects.extend(ref_tu(&wcs_c).compile_intermediates());
    objects.extend(ref_tu(&ctype_c).compile_intermediates());
    objects.extend(ref_tu(&mbcs_c).compile_intermediates());
    objects.extend(refimpl_build.compile_intermediates());
    objects.extend(subject_build.compile_intermediates());
    objects.extend(nls_build.compile_intermediates());
    objects.extend(so_build.compile_intermediates());
    objects.extend(pfflat_build.compile_intermediates());

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
    println!("cargo:rerun-if-changed={}", ctype_c.display());
    // Not compiled — PARSED, for the C-locale cloc_clmap/cloc_cumap tables.
    println!("cargo:rerun-if-changed={}", locale_c.display());
    println!("cargo:rerun-if-changed={}", msvcrt_dir.join("bnum.h").display());
    // printf.h is REAL logic (wcs.c instantiates it twice), not a stub.
    println!("cargo:rerun-if-changed={}", msvcrt_dir.join("printf.h").display());
    println!("cargo:rerun-if-changed={}", manifest.join("refimpl.c").display());
    println!("cargo:rerun-if-changed={}", manifest.join("subjectimpl.c").display());
    println!("cargo:rerun-if-changed={}", manifest.join("nlsimpl.c").display());
    println!("cargo:rerun-if-changed={}", manifest.join("nlsimpl.h").display());
    println!("cargo:rerun-if-changed={}", mbcs_c.display());
    println!("cargo:rerun-if-changed={}", refstub.display());
    println!("cargo:rerun-if-changed={}", refstub_printf.display());
    println!(
        "cargo:rerun-if-changed={}",
        repo.join("sdk/lib/crt/printf/streamout.c").display()
    );
    println!("cargo:rerun-if-changed=build.rs");
}
