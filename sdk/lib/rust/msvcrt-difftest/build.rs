// Plain `//` and not `//!`, because ../msvcrt-scanf-difftest/build.rs is a one-line
// `include!` of this file and an inner doc comment is only legal at the true start of
// a module — inside an expansion it is a hard error.
//
// Compile the *real* msvcrt string reference (`dll/win32/msvcrt/string.c`)
// for the host, with every one of its 106 public symbols renamed to
// `ref_<name>`, so the differential tests can call the genuine C msvcrt
// and cnt-rust's `#[no_mangle]` shims in one process and assert parity.
//
// Nothing is transcribed: `string.c` is handed to `cc` *as the
// translation unit*, exactly as it sits in the tree.  Two mechanisms make
// that work on linux-x86_64:
//
//  1. A generated `-include ref_prelude.h` that (a) renames the 106
//     public symbols with `#define <name> ref_<name>`, and (b) pulls
//     `refstub/msvcrt.h`, which reuses the REAL header's include guard
//     (`__WINE_MSVCRT_H`) so string.c's own `#include "msvcrt.h"` — which
//     resolves to the real same-dir header and would drag in the whole
//     NT/PSDK chain — expands to nothing.  See refstub/msvcrt.h.
//
//  2. `refstub/` on the include path, catching the three quoted includes
//     that have no same-dir file: `winnls.h`, `wine/asm.h`, `wine/debug.h`.
//     `bnum.h` is deliberately NOT stubbed — it is real logic (the base-10
//     bignum behind strtod) and resolves to the genuine same-dir header.
//
// `refimpl.c` (compiled WITHOUT the rename prelude) supplies the externals
// the host libc doesn't have.

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
    // `atoi` joins them for an adjacent reason, and it took undname.c becoming a reference
    // TU to surface it. glibc's <stdlib.h> defines `atoi` as an EXTERN INLINE that calls
    // `strtol` — and `strtol` is hidden from glibc two lines down, so after stage 1 the
    // inline body reads `msvcrt_ref_glibc_strtol`, a name nothing ever defines. Every TU
    // that calls `atoi` therefore fails to link, which no reference TU did until undname.c.
    // Hiding `atoi` too means glibc never emits the inline at all and string.c's own
    // `ref_atoi` is what gets called — which is the body we want graded anyway.
    ("atoi", "int ref_atoi(const char*);"),
    // `labs` is math.c's, and it is the same LP64-vs-LP32 collision as `atol` above: glibc
    // declares `long labs(long)`, so after stage 1 that is a declaration of
    // `long ref_labs(long)` — and math.c then defines `__msvcrt_long ref_labs(__msvcrt_long)`,
    // i.e. `int(int)`. A hard conflicting-types error, and the reason msvcrt has its own is
    // exactly that Windows `long` is 32-bit.
    ("labs", "int ref_labs(int);"),
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
    // `_ispunct_l` was absent from this list until wave 25, and the omission had the same
    // root cause as the comment in msvcrt-crt-rust that claimed msvcrt exports no such
    // function: it is `(MSVCR80.@)`, so it is NOT in msvcrt.spec — but ctype.c compiles it
    // unconditionally and `llvm-nm --defined-only ctype.c.obj` lists `T __ispunct_l`. The
    // DLL export list is not the object's symbol table. Nothing noticed while the subject
    // side had no `_ispunct_l` of its own; the moment lib.rs grew one, the reference's
    // un-renamed copy became a duplicate symbol and took the whole difftest link down.
    "_ispunct_l",
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

/// Lift the `= { … };` INITIALIZER of a named global out of a C source, verbatim,
/// preprocessor arms and all.
///
/// Used for locale.c's `cloc_time_data` — the C locale's calendar vocabulary, which
/// `strftime` reads for every `%a %A %b %B %p %c %x %X`. Same reasoning as
/// `extract_u8_table` above, one step further: the text comes across UNEVALUATED, so the
/// `#if _MSVCR_VER` arms inside it resolve in the harness exactly as they resolve in the
/// DLL (both compile at `_MSVCR_VER=0`). Hand-copying 43 strings would be the §9.73 trap
/// with 43 chances to fire — both sides read one table, so a mistyped month would make
/// them agree on the wrong answer.
///
/// Brace-balanced rather than "find the next `};`": the initializer is nested three deep
/// and the innermost line ends in `}},`.
fn extract_initializer(src: &str, name: &str) -> Option<String> {
    // The DEFINITION, not a mention: `<type> NAME =`, with only spaces between.
    let at = src.match_indices(name).find(|(i, _)| {
        let after = src[i + name.len()..].trim_start();
        after.starts_with('=')
            && !after.starts_with("==")
            // Not `foo.cloc_time_data` / `&cloc_time_data` — a member or address-of.
            && !src[..*i].ends_with(['.', '&', '>'])
    })?;
    let open = src[at.0..].find('{')? + at.0;
    let mut depth = 0usize;
    for (i, c) in src[open..].char_indices() {
        match c {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(src[open..open + i + 1].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

/// Lift whole `#define` lines (continuations included) out of a header, by name.
///
/// The same move `imagehlp.h`'s `UNDNAME_*` family gets below, generalised: the value
/// comes from the header the real build compiles against, so it cannot drift from what
/// the DLL means. Used for `MAKELCID`/`LANG_ENGLISH`/`SORT_DEFAULT`, which locale.c's
/// `cloc_time_data` initializer spells and which no refstub header carries.
fn lift_defines(src: &str, names: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let mut lines = src.lines().peekable();
    while let Some(line) = lines.next() {
        let t = line.trim_start();
        let Some(rest) = t.strip_prefix("#define ") else { continue };
        let ident: String =
            rest.chars().take_while(|c| c.is_alphanumeric() || *c == '_').collect();
        if !names.contains(&ident.as_str()) {
            continue;
        }
        let mut body = t.to_string();
        while body.trim_end().ends_with('\\') {
            let Some(next) = lines.next() else { break };
            body.push('\n');
            body.push_str(next);
        }
        out.push(body);
    }
    out
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

/// File-scope DATA definitions, sorted — the other half of `defined_symbols`.
///
/// `defined_symbols` requires a `(` on the line, so it finds functions and nothing else.
/// A file-scope array or scalar is just as much a definition, collides at the link just
/// as hard, and — worse — if one side's copy wins, the two implementations silently SHARE
/// one piece of state. That is the same lesson the `nm`-based scrape below already
/// records for file.c's `MSVCRT___pioinfo` / `MSVCRT__iob` / `MSVCRT___badioinfo`; mbcs.c's
/// `MSVCRT_mbctype` is the same shape and was missed because its scrape is source-based.
///
/// House style is the whole grammar: a definition starts at column 0, is not `static`,
/// `typedef` or `extern`, and reaches its `[` or `=` without passing a `(`.
/// Every `.spec` in the repo, for the demangler corpus.
///
/// Skips `build-*` and `.git`: a build directory holds COPIES of the same specs, so walking
/// it would double the scrape time and add nothing, and on a fleet machine those copies may
/// belong to a different commit.
fn collect_specs(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        let name = e.file_name();
        let name = name.to_string_lossy();
        if p.is_dir() {
            if name.starts_with("build-") || name == ".git" || name == "target" {
                continue;
            }
            collect_specs(&p, out);
        } else if name.ends_with(".spec") {
            out.push(p);
        }
    }
}

fn defined_data_symbols(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in src.lines() {
        if line.starts_with([' ', '\t', '#', '/', '*', '}']) || line.trim().is_empty() {
            continue;
        }
        if line.starts_with("static") || line.starts_with("typedef") || line.starts_with("extern")
        {
            continue;
        }
        let Some(end) = line.find(['[', '=']) else { continue };
        let head = &line[..end];
        if head.contains('(') {
            continue; // a function, or a function-pointer declarator we should not guess at
        }
        let name: String = head
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
        // A bare `TYPE name;` is a tentative definition and still emits a symbol, but the
        // head must have held at least a type before the name for this to be a definition
        // at all rather than an assignment at file scope (which C does not allow anyway).
        if name.is_empty()
            || name.chars().next().is_some_and(|c| c.is_ascii_digit())
            || head[..head.len() - name.len()].trim().is_empty()
        {
            continue;
        }
        out.push(name);
    }
    out.sort();
    out.dedup();
    out
}

/// The externally-visible symbols of a BUILT object whose nm type letter is in
/// `kinds`, sorted.  `"T"` is code; `"D"`/`"B"` are initialised/zero data.
///
/// Used for file.c instead of a source scrape.  A scrape has to model the C —
/// the `#if _MSVCR_VER` arms, the 19 `#undef`s, the static/exported split — and
/// file.c breaks every simplifying assumption the other five TUs allowed.  The
/// object is the authority on what the compiler actually emitted, so it cannot
/// disagree with the build the way a regex can.
///
/// DATA matters as much as code here, and asking only for `"T"` is how three
/// symbols escaped: file.c defines `MSVCRT___pioinfo` (the fd table),
/// `MSVCRT__iob` (the first three FILEs) and `MSVCRT___badioinfo`, and the
/// subject defines all three too.  Un-renamed they do not merely collide at the
/// link — if one side's copy won, the two implementations would share ONE fd
/// table and the harness would be grading an implementation against itself.
/// Lowercase letters (`d`/`b`/`t`) are file-static and cannot collide; `.refptr.*`
/// are linker thunks for symbols file.c only REFERENCES, so both are excluded.
///
/// THIS worktree's build dir only.  Resolving a sibling's would name symbols
/// from another lane's commit — the fleet-isolation-applies-to-reads law, and
/// the `verify_signatures` incident that established it.
fn object_symbols(repo: &std::path::Path, obj: &PathBuf, kinds: &[&str]) -> Vec<String> {
    let nm = repo.join(".build-tools/llvm-mingw/bin/llvm-nm");
    assert!(
        obj.is_file(),
        "{} not built.\nfix: ./vibeos-cli build --arch amd64",
        obj.display()
    );
    let out = std::process::Command::new(&nm)
        .arg("--defined-only")
        .arg(obj)
        .output()
        .unwrap_or_else(|e| panic!("failed to run {}: {e}", nm.display()));
    let mut syms: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|ln| {
            let p: Vec<&str> = ln.split_whitespace().collect();
            (p.len() == 3 && kinds.contains(&p[1]) && !p[2].starts_with('.'))
                .then(|| p[2].split('@').next().unwrap().to_string())
        })
        .collect();
    syms.sort();
    syms.dedup();
    syms
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

/// The symbols a TU defines inside `#ifndef __REACTOS__` — that is, the ones ReactOS took out
/// of the Wine original and supplies from elsewhere. `defines_symbol` is textual and cannot
/// see a preprocessor guard, so it reports every one of them as defined, and the rename wall
/// then claims a symbol that will not exist in the object.
///
/// That is harmless for a name nothing references and a LINK ERROR for a name the TU CALLS.
/// math.c is where it bites: `ldexp` lives in such a block, `_scalb`'s entire body is
/// `return ldexp(num, power)`, so the wall rewrites the call to `ref_ldexp` and nothing
/// defines it. The error only appears once a test actually references `_scalb`, which is why
/// the TU landed green and the first row set did not link.
///
/// Left off the wall, such a call resolves to whatever the SUBJECT side resolves it to — the
/// host libm here, cnt-rust's `cabi` in the DLL — which is the correct outcome: the symbol
/// belongs to another translation unit and another difftest crate grades it.
///
/// string.c and process.c carry these blocks too, so this is written generally rather than as
/// a list of names.
fn reactos_guarded_symbols(src: &str) -> std::collections::HashSet<String> {
    let mut out = std::collections::HashSet::new();
    let mut depth = 0usize;
    // The `#if` nesting depths at which an enclosing `#ifndef __REACTOS__` opened.
    let mut guards: Vec<usize> = Vec::new();
    for line in src.lines() {
        let t = line.trim_start();
        if t.starts_with("#ifndef __REACTOS__") {
            guards.push(depth);
        }
        if t.starts_with("#if") {
            depth += 1;
        } else if t.starts_with("#endif") {
            depth = depth.saturating_sub(1);
            if guards.last() == Some(&depth) {
                guards.pop();
            }
        }
        if guards.is_empty() {
            continue;
        }
        // `<type...> CDECL <name>(` at the head of a line — the shape every export in these
        // files is written in.
        if let Some(rest) = line.split_once(" CDECL ") {
            let name: String = rest
                .1
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() && rest.1[name.len()..].trim_start().starts_with('(') {
                out.insert(name);
            }
        }
    }
    out
}

fn main() {
    let this_crate = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/<crate> → repo root is 4 up.
    let repo = this_crate.ancestors().nth(4).unwrap().to_path_buf();

    // The reference-building machinery is SHARED, so it is rooted at msvcrt-difftest
    // rather than at whichever crate is compiling.  msvcrt-scanf-difftest is the second
    // consumer: its build.rs is a one-line `include!` of this file, because the two
    // crates need the identical set of reference TUs and differ ONLY in which features
    // of msvcrt-crt-rust they pull.  That difference is the whole reason they are
    // separate crates — cargo unifies features per crate per build, so a suite that
    // needs `stdio` OFF and a suite that needs it ON cannot share one cargo invocation.
    // See msvcrt-scanf-difftest/Cargo.toml.
    //
    // Everything derived from `out` (OUT_DIR) stays per-crate, so the two get their own
    // generated headers and their own reference objects; only the SOURCES are shared.
    let manifest = repo.join("sdk/lib/rust/msvcrt-difftest");
    let refstub = manifest.join("refstub");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());

    // Does THIS crate's msvcrt-crt-rust carry the `stdio` feature — i.e. is file.rs in
    // the link?  Keyed on the package name because the feature belongs to the
    // DEPENDENCY, and cargo exposes only a crate's OWN features to its build script.
    // The consequence is the varargs-shim TU below: its bodies call `_sopen_dispatch`,
    // which only file.rs defines, so compiling it into a crate without stdio would be
    // an undefined symbol rather than a saving.
    let subject_stdio = match env::var("CARGO_PKG_NAME").unwrap().as_str() {
        "msvcrt-difftest" => true,
        "msvcrt-scanf-difftest" => false,
        other => panic!(
            "{other} include!s msvcrt-difftest/build.rs but is not in its stdio table — \
             add it, and say whether it takes msvcrt-crt-rust with the `stdio` feature"
        ),
    };

    let msvcrt_dir = repo.join("dll/win32/msvcrt");
    let string_c = msvcrt_dir.join("string.c");
    let wcs_c = msvcrt_dir.join("wcs.c");
    let ctype_c = msvcrt_dir.join("ctype.c");
    let locale_c = msvcrt_dir.join("locale.c");
    let mbcs_c = msvcrt_dir.join("mbcs.c");
    let scanf_c = msvcrt_dir.join("scanf.c");
    let undname_c = msvcrt_dir.join("undname.c");
    let misc_c = msvcrt_dir.join("misc.c");
    let math_c = msvcrt_dir.join("math.c");
    let cpp_c = msvcrt_dir.join("cpp.c");
    let except_c = msvcrt_dir.join("except.c");
    for src in [
        &string_c, &wcs_c, &ctype_c, &locale_c, &mbcs_c, &scanf_c, &undname_c, &misc_c,
        &math_c, &cpp_c, &except_c,
    ] {
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
        .chain(defined_data_symbols(&mbcs_src))
        .filter(|name| !NAMESPACED_SYMBOLS.contains(&name.as_str()))
        .collect();
    // `MSVCRT_mbctype` is the reason the data half exists: msvcrt.spec exports it by name,
    // so the subject crate must define it `#[no_mangle]`, and an un-renamed reference copy
    // then collides. Assert it rather than trust the scrape — a silent miss here is a
    // duplicate symbol at best and one shared MBCS table between the two sides at worst.
    //
    // It is renamed, unlike ctype.c's `MSVCRT__ctype`/`MSVCRT__wctype` which are shared on
    // purpose, and the line between them is WHO WRITES IT. ctype's tables are locale
    // ENVIRONMENT the two sides must agree on. This one is written by `_setmbcp` — code
    // under test — so sharing it would let each side read what the other wrote, which is
    // the failure errno.c's stage 5 comment already spells out for `MSVCRT__sys_errlist`:
    // the row would compare a table against itself.
    assert!(
        mbcs_symbols.iter().any(|s| s == "MSVCRT_mbctype"),
        "mbcs.c's data scrape did not find MSVCRT_mbctype — the reference copy would keep \
         its plain name and collide with the subject's exported one"
    );
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

    // undname.c — reference TU #8, and the smallest surface of any of them: TWO globals,
    // `__unDName` and `__unDNameEx`, with every other function in the file `static`.
    // Derived like mbcs.c's rather than listed, for the same reason, and then pinned to
    // exactly those two names — a demangler that suddenly exports a third has either grown
    // an entry point or lost a `static`, and either way the rename wall needs to know.
    let undname_src = fs::read_to_string(&undname_c).unwrap();
    let undname_symbols: Vec<String> = defined_symbols(&undname_src)
        .into_iter()
        .chain(defined_data_symbols(&undname_src))
        .filter(|name| !NAMESPACED_SYMBOLS.contains(&name.as_str()))
        .collect();
    assert_eq!(
        undname_symbols,
        vec!["__unDName".to_string(), "__unDNameEx".to_string()],
        "undname.c's global surface changed; it was exactly the two entry points"
    );

    // misc.c — reference TU #9. Unlike undname.c's clean pair, its source carries a lot of
    // definitions this build never compiles: `_chkesp` and its `__ASM_GLOBAL_FUNC` are
    // `#ifdef __i386__`, and `_byteswap_*`, `_encode_pointer`, `_crt_debugger_hook` and the
    // rest sit behind `_MSVCR_VER` guards this DLL does not open. `defines_symbol` filters
    // the macro INVOCATIONS (no `{` body) but cannot see a preprocessor guard, so the list
    // is a superset of what the object exports.
    //
    // That is fine and is worth stating rather than working around: a rename of a name the
    // TU never defines is an inert `#define`, and if a guard ever opens it is already
    // namespaced instead of colliding with the subject. What would NOT be fine is missing
    // one that IS defined — so the assertion below pins the set that the object actually
    // exports today.
    let misc_src = fs::read_to_string(&misc_c).unwrap();
    let misc_symbols: Vec<String> = defined_symbols(&misc_src)
        .into_iter()
        .chain(defined_data_symbols(&misc_src))
        .filter(|n| defines_symbol(&misc_src, n))
        .filter(|name| !NAMESPACED_SYMBOLS.contains(&name.as_str()))
        .collect();
    for want in [
        "qsort", "qsort_s", "bsearch", "bsearch_s", "_lfind", "_lfind_s", "_lsearch",
        "rand", "srand", "rand_s", "_beep", "_sleep", "_resetstkoflw",
        "_get_output_format", "_set_output_format",
    ] {
        assert!(
            misc_symbols.iter().any(|s| s == want),
            "misc.c's scrape missed `{want}`, which the object exports — it would keep its \
             plain name and collide with the subject's"
        );
    }

    // math.c — reference TU #10. Same `#if`-guarded superset caveat as misc.c, plus one
    // thing no other TU has: `math_error` is RENAMED BY THE BUILD. msvcrt's CMakeLists
    // carries `-Dmath_error=_invoke_matherr`, so the source never spells the symbol the
    // object exports — the same trap verify_cohab_coverage's docstring records for the
    // cohab side, arriving here on the harness side. It is excluded from the scraped
    // rename list and handled by a `-D` on the TU instead (see MATH_DEFINES below);
    // renaming it here as well would collide with that.
    let math_src = fs::read_to_string(&math_c).unwrap();
    let math_reactos_guarded = reactos_guarded_symbols(&math_src);
    let math_symbols: Vec<String> = defined_symbols(&math_src)
        .into_iter()
        .chain(defined_data_symbols(&math_src))
        .filter(|n| defines_symbol(&math_src, n))
        .filter(|n| n != "math_error")
        .filter(|n| !math_reactos_guarded.contains(n.as_str()))
        .filter(|name| !NAMESPACED_SYMBOLS.contains(&name.as_str()))
        .collect();
    // Pinned so that adding or removing one of these blocks in math.c is visible here rather
    // than as a link error in whichever test first references the affected function.
    {
        let mut got: Vec<&str> = math_reactos_guarded.iter().map(String::as_str).collect();
        got.sort_unstable();
        assert_eq!(
            got,
            ["_fpclass", "div", "ldexp", "ldiv"],
            "math.c's `#ifndef __REACTOS__` set changed — the rename wall follows it"
        );
    }
    for want in [
        "_isnan", "_finite", "_chgsign", "_ecvt", "_fcvt", "_gcvt", "_control87", "_controlfp",
        "_matherr", "MSVCRT_exp", "MSVCRT_asin", "_abs64", "abs", "labs", "_cabs", "_scalb",
    ] {
        assert!(
            math_symbols.iter().any(|s| s == want),
            "math.c's scrape missed `{want}`, which the object exports"
        );
    }
    assert!(
        !math_symbols.iter().any(|s| s == "math_error"),
        "`math_error` must stay out of the rename wall — the -D owns it"
    );

    let all_symbols: Vec<&str> = PUBLIC_SYMBOLS
        .iter()
        .chain(WCS_PUBLIC_SYMBOLS.iter())
        .chain(CTYPE_PUBLIC_SYMBOLS.iter())
        .copied()
        .chain(mbcs_symbols.iter().map(String::as_str))
        .chain(undname_symbols.iter().map(String::as_str))
        .chain(misc_symbols.iter().map(String::as_str))
        .chain(math_symbols.iter().map(String::as_str))
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

    // ── the C locale's CALENDAR VOCABULARY, lifted the same way ──────────────
    //
    // `strftime`/`wcsftime`/`_Strftime` render `%a %A %b %B %p %c %x %X` out of
    // `get_locinfo()->lc_time_curr`, and that slot was NULL in both sides' locinfo until
    // now — nothing had ever driven those conversions, so a null deref was waiting for the
    // first row that did. locale.c owns the table (`cloc_time_data`), and locale.c is
    // neither a reference TU nor in the subject's feature set, so the harness must supply
    // it. Lifted, not typed: 43 strings both sides read is 43 chances to be
    // invisibly-wrong-together.
    //
    // ONE copy, shared. That is the same line `MSVCRT__ctype` sits on and for the same
    // reason: this is what the LOCALE IS — environment both sides must agree about before
    // a comparison means anything — and neither side's code writes it at this
    // configuration. What IS graded is time.c's indexing into it, which is where the bugs
    // live (`%A` reading `short_wday`, an off-by-one month, `%p` inverted).
    //
    // It gets its own TU rather than a slot in refimpl.c because of `-fshort-wchar`: the
    // initializer carries the wide arm (`L"Sun"`, …) and refimpl.c is compiled without the
    // flag, so those literals would be 4-byte elements behind 2-byte pointers. Nothing
    // reads the wide arm at `_MSVCR_VER=0` — `STRFTIME_CHAR` is `char` below 90 — but a
    // silently mis-sized table waiting for a version bump is not worth saving a file over.
    {
        let init = extract_initializer(&locale_src, "cloc_time_data").unwrap_or_else(|| {
            panic!(
                "could not find `__lc_time_data cloc_time_data = {{…}}` in locale.c — the C \
                 locale's calendar strings moved. strftime cannot be graded without them."
            )
        });
        // Pin the shape rather than trust the brace walk: these are the first and last
        // strings of the narrow arm and the refcount sentinel, so a truncated or
        // mis-anchored lift cannot pass.
        for want in ["\"Sun\"", "\"December\"", "\"HH:mm:ss\"", "1, -1"] {
            assert!(
                init.contains(want),
                "cloc_time_data's lifted initializer is missing {want:?} — the extraction \
                 anchored on the wrong text or stopped early"
            );
        }
        // winnt.h's `MAKELCID(LANG_ENGLISH, SORT_DEFAULT)` is the one token in that
        // initializer no refstub header carries. Lifted from the header the real build
        // compiles against, exactly like imagehlp.h's UNDNAME_* family below — and from
        // THIS worktree's build dir, never a sibling's (fleet isolation applies to reads).
        let winnt_h = repo.join("build-amd64/sdk/include/psdk/winnt.h");
        let winnt_src = fs::read_to_string(&winnt_h).unwrap_or_else(|e| {
            panic!(
                "{}: {e}\nfix: ./vibeos-cli build --arch amd64 (this worktree's build dir \
                 only — a sibling's is another commit's headers)",
                winnt_h.display()
            )
        });
        let lifted = lift_defines(&winnt_src, &["MAKELCID", "LANG_ENGLISH", "SORT_DEFAULT"]);
        assert_eq!(
            lifted.len(),
            3,
            "winnt.h no longer defines all of MAKELCID / LANG_ENGLISH / SORT_DEFAULT \
             (found {}) — cloc_time_data's lcid cannot be derived",
            lifted.len()
        );
        let mut td = String::from(
            "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
             * `cloc_time_data`, lifted verbatim from dll/win32/msvcrt/locale.c, plus the\n\
             * three winnt.h defines its initializer spells. See build.rs. */\n\
             /* winnt.h's MAKELCID casts through these two; the refstub's Windows type\n\
              * model is not in scope in this TU, and two scalar typedefs are cheaper than\n\
              * pulling it in for a data-only file. */\n\
             typedef unsigned int   DWORD;\n\
             typedef unsigned short USHORT;\n\
             #include \"locale_layout.h\"\n\n\
             /* ── lifted from build-amd64/sdk/include/psdk/winnt.h ── */\n",
        );
        for d in &lifted {
            td.push_str(d);
            td.push('\n');
        }
        td.push_str("\n/* ── lifted from dll/win32/msvcrt/locale.c ── */\n__lc_time_data cloc_time_data =\n");
        td.push_str(&init);
        td.push_str(";\n");
        fs::write(out.join("cloc_time_data.c"), &td).unwrap();
    }

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
    // ── stage 4: the TRUNCATION CONTRACT the host libc does not share ────────────────────
    //
    // `sdk/include/crt/stdio.h:720` carries `#define vsnprintf _vsnprintf`, so in the DLL
    // every `vsnprintf` call in wcs.c reaches msvcrt's OWN engine. The reference TU compiles
    // against the host's <stdio.h>, which has no such line, so those calls bind to glibc —
    // and the two disagree about what happens when the result does not fit:
    //
    //     _snprintf(buf, 80, "%.*f", 80, 0.0)
    //       msvcrt -> returns -1, fills all 80 bytes, writes NO terminator
    //       glibc  -> returns 82, writes 79 bytes and a terminator
    //
    // That is not an abstract difference: `_ecvt`/`_fcvt` PRINT INTO an 80-byte buffer and
    // then read the result back to build their answer, so the extra terminator becomes one
    // missing digit. `_fcvt(0.0, 80)` came back 77 zeros against the subject's 78, and the
    // subject was right. Same family as the RAND_MAX and FP_* leaks: a host CONSTANT arriving
    // by inclusion and a host FUNCTION arriving by omission are one accident, and both make
    // the oracle quietly disagree with the code it claims to define.
    //
    // The pfflat instantiation fixes its copy by pointing `vsnprintf` at msvcrt's
    // `_vsnprintf` (see pfflat stage 4 below) — and explains at length why the ref_
    // instantiation must NOT do that: `_vsnprintf` passes `&valist` as a `va_list*`, which
    // is coherent only under the flat va_list pfflat is compiled with, and faults under the
    // host's SysV one. Both things are true at once, so ref_ takes neither host semantics nor
    // msvcrt's body: a shim that formats with glibc and then applies msvcrt's truncation
    // contract on top. `msvcrt_ref_vsnprintf` lives in refimpl.c, which is compiled WITHOUT
    // this prelude and so still reaches the real glibc symbol.
    //
    // What that leaves un-graded, stated plainly: the DIGITS still come from glibc, so a
    // format msvcrt and glibc spell differently (%I64, %p, exponent width) is still the host's
    // answer here. pfflat is the instantiation that grades formatting; this one only needs
    // its buffer arithmetic to be msvcrt's.
    //
    // The sweep that found it generalises — for any new reference TU, take the symbols its
    // object leaves undefined, drop the ref_/harness ones, and intersect with what msvcrt's
    // own objects define. What remains is a host body standing in for msvcrt's.
    // `vsnprintf` was the only one whose semantics differ; malloc/free/calloc and the
    // harness's locale stubs are reached by BOTH sides and cancel out.
    prelude.push_str(
        "\n/* stage 4: msvcrt's truncation contract over the host's formatter. */\n\
         int msvcrt_ref_vsnprintf(char *str, size_t len, const char *format, va_list ap);\n\
         #undef vsnprintf\n#define vsnprintf msvcrt_ref_vsnprintf\n",
    );

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

    // ══ reference TU #5 — dll/win32/msvcrt/scanf.c (wave 17) ═════════════════════════
    //
    // Three things make this TU unlike the other four, and each of them is a decision
    // the harness would be wrong without.
    //
    // 1. THE SIXTEEN ENGINES ARE `static`.  scanf.c #includes scanf.h sixteen times, and
    //    every instantiation's `_FUNCTION_` macro spells `static int vsscanf_l(...)`.  A
    //    rename prelude cannot reach internal linkage, so the graded functions are
    //    unreachable from the test binary by construction.  The way in is a generated
    //    wrapper TU that `#include`s scanf.c and then defines external forwarders IN THE
    //    SAME TU — `refs_vsscanf_l` &c. below.  (Renaming them instead would just produce
    //    a `static ref_vsscanf_l`, which is no more linkable than what we started with.)
    //
    // 2. THE FLAT `va_list`.  The engine's whole risk surface is the va_arg cursor:
    //    pointer-then-size under SECURE, both skipped when `*`-suppressed.  The shipping
    //    Rust walks a `*mut u8` over 8-byte slots (`printf::va_next`), which IS the
    //    Windows-x64 model; the host's SysV va_list is a register-save-area struct and
    //    the two cannot consume the same argument buffer.  So this TU is compiled against
    //    refstub_printf/<stdarg.h> — the flat `char *` model the pfflat printf probe
    //    already proved — and reference and subject are then handed the SAME hand-packed
    //    slot buffer.  Without this the comparison is not merely awkward, it is
    //    impossible: there is no way to hand a SysV va_list to a Rust `*mut u8`.
    //
    // 3. THE PUBLIC SHELLS GO THROUGH THE STAGE-0 TWO-STEP, not the plain stage-1 rename.
    //    glibc declares `sscanf`/`fscanf`/`swscanf`/… with `__REDIRECT(..., __isoc99_*)`,
    //    so a stage-1 `#define sscanf ref_sscanf` before <stdio.h> yields a `ref_sscanf`
    //    carrying `__asm__("__isoc99_sscanf")` — scanf.c's definition would then EMIT the
    //    glibc symbol.  Hiding them behind a throwaway name first and renaming after the
    //    host headers (exactly what LP32_LONG_SYMBOLS does, for a different reason) keeps
    //    glibc's redirect attached to a name nobody defines or calls.  The 40 shells are
    //    not graded here anyway — tools/scan_varargs_shim_parity.py owns them — they only
    //    have to not collide.
    let scanf_src = fs::read_to_string(&scanf_c).unwrap();
    let scanf_symbols: Vec<String> = defined_symbols(&scanf_src);
    assert!(
        scanf_symbols.len() > 40,
        "scanf.c scrape found only {} definitions — the scan is broken, and a short list \
         would leave a public shell under its PLAIN name next to glibc's",
        scanf_symbols.len()
    );
    {
        // No symbol may be renamed twice: the scanf prelude #includes the union one.
        let union: std::collections::HashSet<&str> = all_symbols.iter().copied().collect();
        let clash: Vec<&String> = scanf_symbols.iter().filter(|s| union.contains(s.as_str())).collect();
        assert!(
            clash.is_empty(),
            "scanf.c and another reference TU both define {clash:?} — the scanf prelude \
             would rename an already-renamed name"
        );
    }

    // The three PSDK headers scanf.c includes directly (string.c/wcs.c/ctype.c/mbcs.c do
    // not, so these are reachable only from this TU).  Generated into OUT_DIR rather than
    // added to refstub/ so the whole scanf reference lives in one file.
    //
    // winternl.h is the interesting one: `%[` drives its scanset through ntdll's bitmap.
    // The functions are declared here and IMPLEMENTED in refimpl.c, in the msvcrt_ref_*
    // namespace like every other thing the reference imports.
    fs::write(
        out.join("windef.h"),
        "/* GENERATED by build.rs — scanf.c's `#include \"windef.h\"`. The Windows type\n\
         * model it wants is already in refstub/msvcrt.h, force-included ahead of this. */\n\
         #ifndef MSVCRT_DIFFTEST_WINDEF_H\n#define MSVCRT_DIFFTEST_WINDEF_H\n#endif\n",
    )
    .unwrap();
    // undname.c's two PSDK includes. `winver.h` contributes nothing it uses; `imagehlp.h`
    // contributes the `UNDNAME_*` flag family and nothing else — which the demangler
    // branches on throughout, so these are the most load-bearing constants in the file.
    //
    // LIFTED FROM THE HEADER, never retyped. A hand-copied flag here would be un-sealed
    // data in exactly the shape the field manual records for rtl-difftest's transposed
    // `STATUS_UNMAPPABLE_CHARACTER`: reference and subject would both branch on our value,
    // agree, and grade a demangler nobody asked for. `sdk/include/psdk/imagehlp.h` is the
    // header the real build compiles against, so it is the one scraped.
    {
        let imagehlp_src = repo.join("sdk/include/psdk/imagehlp.h");
        let text = fs::read_to_string(&imagehlp_src)
            .unwrap_or_else(|e| panic!("{}: {e}", imagehlp_src.display()));
        let mut body = String::from(
            "/* GENERATED by build.rs — the UNDNAME_* family, lifted verbatim from\n\
             * sdk/include/psdk/imagehlp.h. Do not edit; do not retype. */\n\
             #ifndef MSVCRT_DIFFTEST_IMAGEHLP_H\n#define MSVCRT_DIFFTEST_IMAGEHLP_H\n",
        );
        let mut lifted = 0usize;
        for line in text.lines() {
            let t = line.trim_start();
            if t.starts_with("#define UNDNAME_") {
                body.push_str(t);
                body.push('\n');
                lifted += 1;
            }
        }
        body.push_str("#endif\n");
        // Every flag undname.c actually reads must be among them; a rename in the PSDK
        // would otherwise leave the TU compiling against an implicit 0.
        for want in [
            "UNDNAME_NAME_ONLY",
            "UNDNAME_NO_ACCESS_SPECIFIERS",
            "UNDNAME_NO_ALLOCATION_LANGUAGE",
            "UNDNAME_NO_ARGUMENTS",
            "UNDNAME_NO_FUNCTION_RETURNS",
            "UNDNAME_NO_LEADING_UNDERSCORES",
            "UNDNAME_NO_MEMBER_TYPE",
            "UNDNAME_NO_MS_KEYWORDS",
            "UNDNAME_NO_THISTYPE",
        ] {
            assert!(
                body.contains(&format!("#define {want}\t")) || body.contains(&format!("#define {want} ")),
                "imagehlp.h no longer defines {want}, which undname.c branches on"
            );
        }
        assert!(lifted >= 15, "only {lifted} UNDNAME_ flags lifted — the scrape has a hole");
        fs::write(out.join("imagehlp.h"), body).unwrap();
        // misc.c's `#include "ntsecapi.h"`, for `rand_s`'s entropy source. DETERMINISTIC
        // and SHARED (refimpl.c), because it is the environment: a real RNG would make the
        // two sides disagree on every draw for a reason that says nothing about the code,
        // and a per-side one would make them disagree for a worse reason. What `rand_s` is
        // graded on is the CONTRACT around the call — the NULL check, the EINVAL, and
        // whether the failure return matches — not the bits.
        fs::write(
            out.join("ntsecapi.h"),
            "/* GENERATED by build.rs — misc.c's `#include \"ntsecapi.h\"`. Only rand_s\n\
             * uses it, and only for this one entry point. */\n\
             #ifndef MSVCRT_DIFFTEST_NTSECAPI_H\n#define MSVCRT_DIFFTEST_NTSECAPI_H\n\
             BOOLEAN WINAPI SystemFunction036(void *buf, ULONG len);\n\
             /* ntsecapi.h's own spelling: advapi32 EXPORTS SystemFunction036 and\n\
              * the friendly name is a macro over it. misc.c calls RtlGenRandom; the\n\
              * Rust binds SystemFunction036. One definition, both names. */\n\
             #define RtlGenRandom SystemFunction036\n#endif\n",
        )
        .unwrap();
        // misc.c's `#include "windows.h"`. The Windows type model and the three kernel32
        // entry points it wants (`Beep`, `Sleep`, `VirtualProtect`) are declared in
        // refstub/msvcrt.h and implemented once in refimpl.c, so this only has to exist.
        fs::write(
            out.join("windows.h"),
            "/* GENERATED by build.rs — misc.c's `#include \"windows.h\"`. Everything it\n\
             * wants is already in refstub/msvcrt.h, force-included ahead of this. */\n\
             #ifndef MSVCRT_DIFFTEST_WINDOWS_H\n#define MSVCRT_DIFFTEST_WINDOWS_H\n#endif\n",
        )
        .unwrap();
        // math.c's `#include <fpieee.h>`. It uses NOTHING from it — one occurrence in the
        // file, the include itself — so an empty guard is the whole faithful answer, exactly
        // as for undname.c's winver.h. (`_FPIEEE_RECORD` and the filter that consumes it
        // live in except.c, which is not this TU.)
        fs::write(
            out.join("fpieee.h"),
            "/* GENERATED by build.rs — math.c's `#include <fpieee.h>`, which it uses\n\
             * nothing from. */\n\
             #ifndef MSVCRT_DIFFTEST_FPIEEE_H\n#define MSVCRT_DIFFTEST_FPIEEE_H\n#endif\n",
        )
        .unwrap();
        fs::write(
            out.join("winver.h"),
            "/* GENERATED by build.rs — undname.c's `#include \"winver.h\"`. It uses\n\
             * nothing from it; the include is there for the ReactOS build's benefit. */\n\
             #ifndef MSVCRT_DIFFTEST_WINVER_H\n#define MSVCRT_DIFFTEST_WINVER_H\n#endif\n",
        )
        .unwrap();
    }

    fs::write(
        out.join("winbase.h"),
        "/* GENERATED by build.rs — scanf.c's `#include \"winbase.h\"`. HeapAlloc &c. are\n\
         * declared in refstub/msvcrt.h; only the flag scanf.h passes is missing. */\n\
         #ifndef MSVCRT_DIFFTEST_WINBASE_H\n#define MSVCRT_DIFFTEST_WINBASE_H\n\
         #define HEAP_ZERO_MEMORY 0x00000008\n#endif\n",
    )
    .unwrap();
    fs::write(
        out.join("winternl.h"),
        "/* GENERATED by build.rs — ntdll's bitmap, which `%[` builds its scanset in.\n\
         * Layout verbatim from the PSDK; the four entry points are implemented in\n\
         * refimpl.c under the msvcrt_ref_* import namespace (see ref_scanf_prelude.h). */\n\
         #ifndef MSVCRT_DIFFTEST_WINTERNL_H\n#define MSVCRT_DIFFTEST_WINTERNL_H\n\
         typedef struct _RTL_BITMAP { ULONG SizeOfBitMap; ULONG *Buffer; } RTL_BITMAP, *PRTL_BITMAP;\n\
         void RtlInitializeBitMap(PRTL_BITMAP, ULONG*, ULONG);\n\
         void RtlSetBits(PRTL_BITMAP, ULONG, ULONG);\n\
         BOOL RtlAreBitsSet(PRTL_BITMAP, ULONG, ULONG);\n\
         BOOL RtlAreBitsClear(PRTL_BITMAP, ULONG, ULONG);\n#endif\n",
    )
    .unwrap();
    fs::write(
        out.join("conio.h"),
        "/* GENERATED by build.rs — scanf.c's `#include <conio.h>`. `_getch`/`_ungetch`\n\
         * are declared in ref_scanf_prelude.h (they have to land AFTER the import\n\
         * namespace block, and this header is read after it). */\n\
         #ifndef MSVCRT_DIFFTEST_CONIO_H\n#define MSVCRT_DIFFTEST_CONIO_H\n#endif\n",
    )
    .unwrap();

    // ── the scanf prelude: stage 0' | the union prelude | stage 3' + 4' ───────────────
    let mut sp = String::from(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included at the top of the scanf.c reference TU ONLY. */\n\
         #ifndef MSVCRT_REF_SCANF_PRELUDE_H\n#define MSVCRT_REF_SCANF_PRELUDE_H\n\n\
         /* ═══ stage 0': hide scanf.c's public shells from glibc. ═══════════════════\n\
          * See build.rs for why these do NOT take the ordinary stage-1 rename: glibc's\n\
          * __REDIRECT would attach `__asm__(\"__isoc99_sscanf\")` to the renamed name and\n\
          * scanf.c would emit glibc's symbol. */\n",
    );
    for sym in &scanf_symbols {
        sp.push_str(&format!("#define {sym} msvcrt_ref_glibc_{sym}\n"));
    }
    sp.push_str(
        "\n/* ═══ the IMPORT namespace for everything scanf.c calls out to. ═══════════\n\
         * Same anti-circularity rule as refstub/msvcrt.h: nothing the reference imports\n\
         * may resolve to the code under test.  The stdio/conio four matter most — the\n\
         * SHIPPING Rust imports `fgetc`/`ungetc`/`fgetwc`/`ungetwc`/`_getch`/`_ungetch`\n\
         * under those exact plain names, and tests/scanf.rs defines the model stream\n\
         * TWICE (plain for the subject, msvcrt_ref_* for the reference) so one model\n\
         * serves both sides without either binding to the other.\n\
         *\n\
         * Placed BEFORE the union prelude on purpose: it pulls <stdio.h>/<wchar.h>, which\n\
         * then declare `msvcrt_ref_fgetc`/`msvcrt_ref_fgetwc` with the right prototypes\n\
         * (and `wint_t` is refstub/msvcrt.h's 16-bit one, so fgetwc comes out returning\n\
         * `unsigned short` — msvcrt's width, not glibc's). */\n\
         #define fgetc               msvcrt_ref_fgetc\n\
         #define ungetc              msvcrt_ref_ungetc\n\
         #define fgetwc              msvcrt_ref_fgetwc\n\
         #define ungetwc             msvcrt_ref_ungetwc\n\
         #define _lock_file          msvcrt_ref__lock_file\n\
         #define _unlock_file        msvcrt_ref__unlock_file\n\
         #define _getch              msvcrt_ref__getch\n\
         #define _ungetch            msvcrt_ref__ungetch\n\
         #define RtlInitializeBitMap msvcrt_ref_RtlInitializeBitMap\n\
         #define RtlSetBits          msvcrt_ref_RtlSetBits\n\
         #define RtlAreBitsSet       msvcrt_ref_RtlAreBitsSet\n\
         #define RtlAreBitsClear     msvcrt_ref_RtlAreBitsClear\n\n\
         #include \"ref_prelude.h\"\n\n\
         /* ═══ stage 3': un-hide the shells and rename them for real. ═══════════════ */\n",
    );
    for sym in &scanf_symbols {
        sp.push_str(&format!("#undef {sym}\n#define {sym} refs_{sym}\n"));
    }
    sp.push_str(
        "\n/* ═══ stage 4': what the real msvcrt.h / corecrt_stdio_config.h supply. ════\n\
         * refstub/msvcrt.h claims __WINE_MSVCRT_H, so the genuine msvcrt.h expands to\n\
         * nothing and these three macro families have to come from somewhere.  Copied\n\
         * verbatim from dll/win32/msvcrt/include/corecrt_stdio_config.h:19-21 and\n\
         * dll/win32/msvcrt/msvcrt.h:412-415 — the four __stdio_common_* doors branch on\n\
         * them, and a wrong bit would silently route SECURE to the non-secure engine. */\n\
         #define _CRT_INTERNAL_SCANF_SECURECRT                    0x0001ULL\n\
         #define _CRT_INTERNAL_SCANF_LEGACY_WIDE_SPECIFIERS       0x0002ULL\n\
         #define _CRT_INTERNAL_SCANF_LEGACY_MSVCRT_COMPATIBILITY  0x0004ULL\n\
         #define UCRTBASE_SCANF_MASK ( \\\n\
         \x20       _CRT_INTERNAL_SCANF_SECURECRT | \\\n\
         \x20       _CRT_INTERNAL_SCANF_LEGACY_WIDE_SPECIFIERS | \\\n\
         \x20       _CRT_INTERNAL_SCANF_LEGACY_MSVCRT_COMPATIBILITY )\n\n\
         /* The stdio/conio externals the real msvcrt.h and <conio.h> declare.  They land\n\
          * here, after the namespace block above, so each is really a declaration of the\n\
          * msvcrt_ref_* name. */\n\
         void __cdecl _lock_file(FILE*);\n\
         void __cdecl _unlock_file(FILE*);\n\
         int  __cdecl _getch(void);\n\
         int  __cdecl _ungetch(int);\n\n\
         /* scanf.c calls ctype.c's `isspace`, which the wall above has already\n\
          * renamed to `ref_isspace` — so glibc's <ctype.h> declaration no longer\n\
          * applies to it and nothing else did either. Same class as file.c's 53. */\n\
         int __cdecl ref_isspace(int c);\n\n\
         #endif /* MSVCRT_REF_SCANF_PRELUDE_H */\n",
    );
    let scanf_prelude_h = out.join("ref_scanf_prelude.h");
    fs::write(&scanf_prelude_h, &sp).unwrap();

    // ── the wrapper TU: the GENUINE scanf.c, plus external forwarders ────────────────
    //
    // `#include`ing the .c is the whole trick (see (1) above).  A quoted include searches
    // the INCLUDED file's own directory, so scanf.c's `#include "scanf.h"` /
    // `"msvcrt.h"` still resolve exactly as they do in the real build.
    let engines: &[(&str, &str)] = &[
        ("vfscanf_l",     "FILE *f, const char *fmt, _locale_t l, va_list ap"),
        ("vfscanf_s_l",   "FILE *f, const char *fmt, _locale_t l, va_list ap"),
        ("vfwscanf_l",    "FILE *f, const wchar_t *fmt, _locale_t l, va_list ap"),
        ("vfwscanf_s_l",  "FILE *f, const wchar_t *fmt, _locale_t l, va_list ap"),
        ("vsscanf_l",     "const char *f, const char *fmt, _locale_t l, va_list ap"),
        ("vsscanf_s_l",   "const char *f, const char *fmt, _locale_t l, va_list ap"),
        ("vswscanf_l",    "const wchar_t *f, const wchar_t *fmt, _locale_t l, va_list ap"),
        ("vswscanf_s_l",  "const wchar_t *f, const wchar_t *fmt, _locale_t l, va_list ap"),
        ("vsnscanf_l",    "const char *f, size_t n, const char *fmt, _locale_t l, va_list ap"),
        ("vsnscanf_s_l",  "const char *f, size_t n, const char *fmt, _locale_t l, va_list ap"),
        ("vsnwscanf_l",   "const wchar_t *f, size_t n, const wchar_t *fmt, _locale_t l, va_list ap"),
        ("vsnwscanf_s_l", "const wchar_t *f, size_t n, const wchar_t *fmt, _locale_t l, va_list ap"),
        ("vcscanf_l",     "const char *fmt, _locale_t l, va_list ap"),
        ("vcscanf_s_l",   "const char *fmt, _locale_t l, va_list ap"),
        ("vcwscanf_l",    "const wchar_t *fmt, _locale_t l, va_list ap"),
        ("vcwscanf_s_l",  "const wchar_t *fmt, _locale_t l, va_list ap"),
    ];
    let mut tu = format!(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * The GENUINE scanf.c, plus one external forwarder per `static` engine. */\n\
         #include \"{}\"\n\n",
        scanf_c.display()
    );
    for (name, params) in engines {
        let call: Vec<&str> = params
            .split(',')
            .map(|p| p.trim().rsplit(|c: char| !(c.is_alphanumeric() || c == '_')).next().unwrap())
            .collect();
        tu.push_str(&format!(
            "int refs_{name}({params}) {{ return {name}({}); }}\n",
            call.join(", ")
        ));
        assert!(
            scanf_src.contains(&format!("{name}(")),
            "scanf.c no longer mentions `{name}` — the engine set moved"
        );
    }
    let scanf_tu_c = out.join("ref_scanf_tu.c");
    fs::write(&scanf_tu_c, &tu).unwrap();

    // ══ reference TU #6 — dll/win32/msvcrt/file.c ═══════════════════════════════════
    //
    // file.c is 163 of msvcrt's remaining un-ported exports — 30% of what is left, in
    // one file — and until now it could not be graded at all, because the difftest had
    // no way to compile it for the host.
    //
    // Three things make it unlike the other five, and each is why a piece of substrate
    // exists:
    //
    // 1. IT OWNS THE FILE LAYOUT.  file.c reads and writes `_ptr`/`_cnt`/`_base`/
    //    `_flag`/`_file`/`_charbuf`/`_bufsiz`/`_tmpfname` directly, so msvcrt's
    //    `struct _iobuf` is part of the contract under test — where the union prelude
    //    would hand out glibc's `struct _IO_FILE`.  refstub/msvcrt_file_pre.h claims
    //    glibc's guards and supplies msvcrt's, and it MUST be the first line of the
    //    prelude (it works by pre-empting <stdio.h>), which is why the static half is
    //    split in two around the union include.
    //
    // 2. ITS CONSTANTS AND LAYOUTS CANNOT BE HAND-WRITTEN.  Both generated:
    //    tools/gen_msvcrt_difftest_consts.py derives every constant with the pinned
    //    cross-compiler against msvcrt's REAL include set (msvcrt substitutes its own
    //    CRT headers for sdk/include/crt, and `_IOMYBUF` differs between the
    //    `_MSVCR_VER` arms), and emits a wall of _Static_assert proving the transcribed
    //    layouts.  That guardian has already caught two live transcription bugs.
    //
    // 3. IT NEEDS AN OS.  file.c implements the stream layer and calls Win32 for the
    //    bytes; fileimpl.c is that layer, on POSIX.  See its header for the rule that
    //    governs it — a shim must model the API's ERROR behaviour, or the caller's
    //    failure branch goes unreachable and its rows report MATCH while proving
    //    nothing.
    //
    // The prelude rides the union one, and that is load-bearing rather than tidy:
    // file.c calls 23 symbols owned by the other reference TUs, and four of them
    // (`mbtowc`, `mbstowcs`, `wctomb`, `isleadbyte`) would otherwise bind to GLIBC and
    // convert through the HOST locale instead of msvcrt's mbcinfo — a wrong oracle that
    // links, runs, and says nothing.  Riding the union renames them ref_* so they
    // resolve into the reference objects already in the archive.
    let file_c = msvcrt_dir.join("file.c");
    assert!(file_c.is_file(), "msvcrt reference source not found: {}", file_c.display());

    // Public symbols come from the BUILT OBJECT, not a scrape: file.c's `#undef` wall
    // and its static/exported split make the source ambiguous, and the object is the
    // authority on what actually got emitted.
    let file_obj = repo.join("build-amd64/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/file.c.obj");
    let file_symbols: Vec<String> = object_symbols(&repo, &file_obj, &["T"]);
    assert!(
        file_symbols.len() > 150,
        "file.c object scrape found only {} symbols — a short list would leave the rest \
         un-renamed and colliding with the subject",
        file_symbols.len()
    );

    // The STATE, renamed on the same principle as the code.  These three are the whole
    // externally-visible data surface of file.c: `MSVCRT___pioinfo` (the fd table),
    // `MSVCRT__iob` (the first three FILEs) and `MSVCRT___badioinfo` (the sentinel every
    // bad fd resolves to).  The subject defines all three as well, so leaving them plain
    // is not just a duplicate-symbol link error — whichever copy won, BOTH scanf engines
    // and BOTH file layers would then share ONE fd table, and a differential harness
    // comparing an implementation against itself reports MATCH for free.
    //
    // They are renamed EARLY (below, before stage 0) rather than in the stage-1/stage-4
    // pair the functions use, and that ordering is the point: msvcrt_file_prelude.h
    // declares `extern FILE MSVCRT__iob[];` at stage 3 and builds `__acrt_iob_func` on it.
    // Renaming first carries that declaration along, so it still describes the definition
    // file.c emits.  Renaming after would leave the declaration naming a symbol nothing
    // defines and the definition undeclared.  Nothing in glibc is spelled `MSVCRT_*`, so
    // the stage-1 hide the functions need does not apply.
    let file_data: Vec<String> = object_symbols(&repo, &file_obj, &["D", "B"]);
    assert!(
        !file_data.is_empty(),
        "file.c object scrape found no exported data — it defines at least MSVCRT___pioinfo, \
         and an empty list would silently drop the fd-table rename"
    );

    // The names the `#define` wall must NOT touch, generated alongside the prototypes.
    // TWO classes, and the second is the dangerous one: a name that is also a struct tag
    // (`_stat`) makes the wall rewrite the TYPE, which fails loudly — but a name the TU
    // `#undef`s (file.c has 19) has its rename SILENTLY DELETED, and the function is then
    // emitted under its true name, colliding with the subject or leaving ref_* undefined.
    // Ten of file.c's symbols leaked exactly that way before this list existed.  Both get
    // an asm label in msvcrt_file_protos.h instead, which renames at the object level
    // where no `#undef` reaches.
    let no_rename_txt = refstub.join("msvcrt_file_no_rename.txt");
    let no_rename: std::collections::HashSet<String> = fs::read_to_string(&no_rename_txt)
        .unwrap_or_else(|e| panic!("{}: {e}", no_rename_txt.display()))
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    for n in &no_rename {
        assert!(
            file_symbols.contains(n),
            "msvcrt_file_no_rename.txt lists `{n}`, which file.c does not export — \
             regenerate: python3 tools/gen_msvcrt_ref_protos.py file.c"
        );
    }
    let file_wall: Vec<&String> = file_symbols.iter().filter(|s| !no_rename.contains(*s)).collect();

    // Disjointness with the union, asserted rather than assumed: an overlap would be a
    // duplicate-symbol link error a long way from its cause.
    {
        let taken: std::collections::HashSet<&str> = all_symbols
            .iter()
            .copied()
            .chain(scanf_symbols.iter().map(String::as_str))
            .collect();
        let clash: Vec<&str> = file_wall
            .iter()
            .map(|s| s.as_str())
            .chain(file_data.iter().map(String::as_str))
            .filter(|s| taken.contains(s))
            .collect();
        assert!(
            clash.is_empty(),
            "file.c and another reference TU both define {clash:?} — one of them must \
             stop being renamed into ref_*"
        );
    }

    // file.c is compiled TWICE, under two prefixes, so the prelude is built by a closure
    // rather than written out once.  The second instantiation (`fileflat_`) exists for one
    // reason: the sixteen `v*printf` shells are NOT CALLABLE in the ref_ one.  They forward
    // `&valist` as a `va_list*`, an idiom coherent only where a `va_list` IS a plain
    // pointer — true of all three Windows targets and of refstub_printf's flat
    // `<stdarg.h>`, false of the host's SysV register-save-area model, where `*valist`
    // reinterprets the decayed pointer variable as the tag struct and faults.  Same
    // diagnosis, same fix, and the same shape as `pfflat_` for wcs.c; see the long note
    // beside that build.
    //
    // `protos` names the stage-5 header because msvcrt_file_protos.h renames thirteen
    // symbols with an `__asm__` LABEL rather than a `#define` (a name that is also a struct
    // tag, or that the TU `#undef`s, cannot be renamed by the wall).  An asm label is a
    // string baked into the header, so the flat instantiation needs its own copy or those
    // thirteen would be emitted under `ref_*` and collide with the reference TU.
    let file_prelude = |prefix: &str, guard: &str, protos: &str| -> String {
        let mut fp = format!(
            "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
             * Force-included at the top of a file.c reference TU ONLY ({prefix}*). */\n\
             #ifndef {guard}\n#define {guard}\n\n\
             /* ═══ stage -1: file.c's exported STATE. ═════════════════════════════════\n\
              * Before everything, so that msvcrt_file_prelude.h's own\n\
              * `extern FILE MSVCRT__iob[];` and the `__acrt_iob_func` it builds on that\n\
              * array are carried along and still describe the definition file.c emits.\n\
              * No glibc name is spelled MSVCRT_*, so the stage-1 hide below does not\n\
              * apply and a single rename is enough. */\n"
        );
        for sym in &file_data {
            fp.push_str(&format!("#define {sym} {prefix}{sym}\n"));
        }
        fp.push_str(
            "\n/* ═══ stage 0: msvcrt's FILE and fpos_t, before ANY host header. ══════════ */\n\
             #include <msvcrt_file_pre.h>\n\n\
             /* ═══ stage 1: hide file.c's own shells from glibc. ═══════════════════════\n\
              * Same reason as the scanf TU: glibc macro-ises several of these (getc, putc,\n\
              * feof, …) and __REDIRECTs others, and either would attach itself to the name\n\
              * file.c is about to DEFINE. */\n",
        );
        for sym in &file_wall {
            fp.push_str(&format!("#define {sym} msvcrt_ref_glibc_{sym}\n"));
        }
        fp.push_str(
            "\n/* ═══ stage 2: the union prelude — sibling renames + the refstub. ════════\n\
             * This is what makes file.c's 23 cross-TU calls resolve ref_* -> ref_* instead\n\
             * of escaping to the host libc.  Four of them (mbtowc, mbstowcs, wctomb,\n\
             * isleadbyte) would otherwise convert through the HOST locale. */\n\
             #include \"ref_prelude.h\"\n\n\
             /* ═══ stage 3: the types, the layout guardian, the constants. ════════════ */\n\
             #include <msvcrt_file_prelude.h>\n\n\
             /* ═══ stage 4: un-hide the shells and rename them for real. ══════════════ */\n",
        );
        for sym in &file_wall {
            fp.push_str(&format!("#undef {sym}\n#define {sym} {prefix}{sym}\n"));
        }
        fp.push_str(
            "\n/* ═══ stage 4.5: `_time64`, out of the SUBJECT's way. ════════════════════\n\
             * file.c calls `_time64` once (the `_wutime`/`_futime` path) and the harness\n\
             * has always answered it from fileimpl.c under the plain name. That was safe\n\
             * only while nothing else in the link defined it. time.rs does now — msvcrt\n\
             * exports `_time64`, so the flip gives it a `#[no_mangle]` — and two strong\n\
             * definitions of one symbol is a duplicate-symbol link failure, which is the\n\
             * standing tax on wiring a new module into this harness.\n\
             *\n\
             * The stub keeps its job (it is the environment's clock, one copy for both\n\
             * sides, exactly like GetStdHandle) and only changes NAME. Both file.c\n\
             * instantiations map to the SAME `ref__time64` for that reason: there is\n\
             * nothing per-side about what time it is.\n\
             *\n\
             * When time.c becomes reference TU #8 this stage is what changes: the map\n\
             * points at time.c's own body and fileimpl.c's stub is deleted, the way\n\
             * errno.c's arrival deleted the hand-written `msvcrt_set_errno`. */\n\
             #define _time64 ref__time64\n",
        );
        fp.push_str(&format!(
            "\n/* ═══ stage 5: a prototype for every function file.c defines. ════════════\n\
             * Placed AFTER the wall, so each is renamed alongside the definition it\n\
             * describes.  Without them the 65 calls file.c places above their definitions\n\
             * fall to C's implicit-declaration rule and become `int()` — every 64-bit\n\
             * return silently truncated, in code that compiles and links. */\n\
             #include <{protos}>\n\n\
             /* ═══ stage 6: a prototype for every function file.c CALLS. ══════════════\n\
              * Stage 5 covers what file.c DEFINES; this covers the other 53 — the Win32\n\
              * layer under it, the msvcrt internals from TUs we do not compile, and the\n\
              * sibling reference TUs' exports. Same rule, same consequence, and it went\n\
              * unnoticed for four waves because fileimpl.c's HANDLEs are small integers\n\
              * that survive being truncated to 32 bits. `msvcrt_wstrdupa` returns a real\n\
              * malloc pointer and does not: it SIGSEGV'd the first row that reached it.\n\
              * Also after the wall, so the sibling calls name the ref_* definitions. */\n\
             #include <msvcrt_shim_decls.h>\n\n"
        ));
        if prefix != "ref_" {
            // ═══ stage 7, fileflat ONLY: the printf ENGINE must be the flat one. ═══
            //
            // Stage 2 renamed `pf_printf_a` & co. to `ref_*` along with the rest of wcs.c's
            // exports — and that instantiation is compiled under the host's SysV va_list,
            // so calling it from here with a flat one would hand it a `char*` where it
            // expects a tag struct.  `pfflat_*` is the same engine compiled the same way
            // this TU is.  Re-declaring after the redefine is not optional: refstub's
            // msvcrt.h already emitted these prototypes under the ref_ spelling, and
            // -Werror=implicit-function-declaration would otherwise catch the calls.
            fp.push_str(
                "/* ═══ stage 7: the printf engine, in the FLAT va_list namespace. ═════════\n\
                 * These six are the entire wcs.c surface the v*printf helpers reach, and\n\
                 * they are the only cross-TU calls whose ABI depends on how THIS TU was\n\
                 * compiled — everything else file.c calls takes its arguments normally and\n\
                 * resolves into the existing ref_* objects unchanged. */\n",
            );
            for sym in [
                "pf_printf_a",
                "pf_printf_w",
                "create_positional_ctx_a",
                "create_positional_ctx_w",
                "arg_clbk_valist",
                "arg_clbk_positional",
            ] {
                fp.push_str(&format!("#undef {sym}\n#define {sym} pfflat_{sym}\n"));
            }
            fp.push_str(
                "int pf_printf_a(puts_clbk_a, void*, const char*, _locale_t,\n\
                 \x20       DWORD, args_clbk, void*, va_list*);\n\
                 int pf_printf_w(puts_clbk_w, void*, const wchar_t*, _locale_t,\n\
                 \x20       DWORD, args_clbk, void*, va_list*);\n\
                 int create_positional_ctx_a(void*, const char*, va_list);\n\
                 int create_positional_ctx_w(void*, const wchar_t*, va_list);\n\
                 printf_arg arg_clbk_valist(void*, int, int, va_list*);\n\
                 printf_arg arg_clbk_positional(void*, int, int, va_list*);\n\n",
            );
        }
        fp.push_str(&format!("#endif /* {guard} */\n"));
        fp
    };

    let fp = file_prelude("ref_", "MSVCRT_REF_FILE_PRELUDE_H", "msvcrt_file_protos.h");
    let file_prelude_h = out.join("ref_file_prelude.h");
    fs::write(&file_prelude_h, &fp).unwrap();

    // The stage-5 header again, with the thirteen asm labels retargeted. A textual
    // rewrite of the generated header rather than a second generator run: the labels are
    // the ONLY `ref_` in it, and asserting that keeps a future one from being missed.
    let protos_src = fs::read_to_string(refstub.join("msvcrt_file_protos.h"))
        .unwrap_or_else(|e| panic!("msvcrt_file_protos.h: {e}"));
    let label_count = protos_src.matches("__asm__(\"ref_").count();
    assert_eq!(
        label_count,
        no_rename.len(),
        "msvcrt_file_protos.h carries {label_count} asm labels for {} unrenamable symbols — \
         regenerate: python3 tools/gen_msvcrt_ref_protos.py file.c",
        no_rename.len()
    );
    // Every `ref_` in the header is accounted for: one per asm label, plus the two
    // mentions of the generator's own filename in the do-not-edit banner. A third source
    // would mean a prototype naming the reference instantiation by hand, which the flat
    // copy must not inherit — so this is an equality, not a "contains".
    let banner = protos_src.matches("gen_msvcrt_ref_protos").count();
    assert_eq!(
        protos_src.matches("ref_").count(),
        label_count + banner,
        "msvcrt_file_protos.h spells `ref_` somewhere other than an asm label or the \
         generator banner — the flat copy would name the wrong instantiation"
    );
    let flat_protos = protos_src
        .replace("__asm__(\"ref_", "__asm__(\"fileflat_")
        .replace("MSVCRT_FILE_PROTOS_H", "MSVCRT_FILEFLAT_PROTOS_H");
    fs::write(out.join("msvcrt_fileflat_protos.h"), &flat_protos).unwrap();

    let fp_flat =
        file_prelude("fileflat_", "MSVCRT_FILEFLAT_PRELUDE_H", "msvcrt_fileflat_protos.h");
    let fileflat_prelude_h = out.join("fileflat_prelude.h");
    fs::write(&fileflat_prelude_h, &fp_flat).unwrap();

    // ══ reference TU #7 — dll/win32/msvcrt/errno.c ══════════════════════════════════
    //
    // This one is here to DELETE a transcription, not only to grade 18 more exports.
    //
    // Until now the harness owned a hand-written copy of msvcrt's Win32 -> errno map
    // (fileimpl.c's `msvcrt_difftest_win32_to_errno`, 42 ERROR_* cases) and BOTH sides
    // called it.  file.c stores through that map 54 times and file.rs does too, so the
    // single most exercised mapping in the whole file suite was the one thing no row
    // could ever grade: shared substrate is invisible to a differential test, both sides
    // are wrong identically, and every row reports MATCH (§9.73).  With errno.c compiled
    // as a reference TU the reference gets msvcrt's GENUINE mapper and the subject keeps
    // Rust's, so the mapping becomes a difference the harness can see.
    //
    // The same swap fixes the errno SLOT.  refimpl.c used to define `_errno` over a
    // `static __thread int` it invented; errno.c's real one returns
    // `&msvcrt_get_thread_data()->thread_errno`.  Both sides now keep errno where msvcrt
    // keeps it.
    //
    // fileimpl.c's deferral banner priced this at "a change to every existing test's
    // plumbing", because refimpl.c's `_errno`/`_invalid_parameter` are environment with a
    // test-visible call counter.  That is only true if the harness API moves.  It doesn't:
    // ref_errno_get/ref_errno_set/ref_invalid_pmt_count/ref_invalid_pmt_reset keep their
    // names and signatures across ~62 call sites in 8 files, and only the storage beneath
    // them changes.  See refimpl.c for the counting wrapper, and for why installing the
    // counter as a real handler is impossible at _MSVCR_VER = 0.
    let errno_c = msvcrt_dir.join("errno.c");
    assert!(errno_c.is_file(), "msvcrt reference source not found: {}", errno_c.display());

    // From the OBJECT, like file.c and for the same reason: what got emitted is the
    // authority, not what the source appears to say through the _MSVCR_VER arms.
    let errno_obj =
        repo.join("build-amd64/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/errno.c.obj");
    let errno_code: Vec<String> = object_symbols(&repo, &errno_obj, &["T"]);
    let errno_data: Vec<String> = object_symbols(&repo, &errno_obj, &["D", "B"]);
    assert!(
        errno_code.len() > 15 && !errno_data.is_empty(),
        "errno.c object scrape found {} functions / {} data — a short list would leave \
         symbols under their PLAIN names, colliding with the subject",
        errno_code.len(),
        errno_data.len()
    );

    // Three of errno.c's definitions already have a home in the msvcrt_ref_* IMPORT
    // namespace that refstub/msvcrt.h sets up for every TU that includes it, and two of
    // them land there by simply being compiled:
    //
    //   _errno            -> msvcrt_ref__errno            (refimpl.c's copy is deleted)
    //   msvcrt_set_errno  -> msvcrt_ref_msvcrt_set_errno  (fileimpl.c's copy is deleted)
    //
    // The reference TUs and fileimpl.c already CALL those names, so errno.c's bodies drop
    // straight into the sockets the hand-written ones vacated.  That is the whole trick,
    // and it is why the rename wall below must skip them: renaming them to ref_* instead
    // would leave every existing caller pointing at a symbol nothing defines.
    //
    // `_invalid_parameter` is the exception.  refimpl.c still occupies
    // msvcrt_ref__invalid_parameter — it has to, that is where the call counter lives —
    // so errno.c's body needs a name of its own to be delegated to.  It gets the ordinary
    // ref_ wall treatment, which means the prelude has to UNDO refstub's mapping first.
    //
    // `__doserrno` is a fourth, and it is namespaced by a LOCAL `#define` rather than by
    // refstub/msvcrt.h — refstub/msvcrt_file_prelude.h and fileimpl.c each spell it out,
    // because when they were written errno.c was not a reference TU and the accessor was
    // environment. The name they chose is what file.c calls, so errno.c's definition has
    // to land there; the prelude repeats the mapping rather than moving it into
    // refstub/msvcrt.h, which is included by TUs that must keep calling the SUBJECT's.
    const ERRNO_NAMESPACED: &[&str] = &["_errno", "msvcrt_set_errno", "__doserrno"];
    let errno_wall: Vec<&String> = errno_code
        .iter()
        .filter(|s| !ERRNO_NAMESPACED.contains(&s.as_str()))
        .collect();

    // Disjointness against every other reference TU, asserted rather than assumed.
    {
        let taken: std::collections::HashSet<&str> = all_symbols
            .iter()
            .copied()
            .chain(scanf_symbols.iter().map(String::as_str))
            .chain(file_symbols.iter().map(String::as_str))
            .chain(file_data.iter().map(String::as_str))
            .collect();
        let clash: Vec<&str> = errno_wall
            .iter()
            .map(|s| s.as_str())
            .chain(errno_data.iter().map(String::as_str))
            .filter(|s| taken.contains(s))
            .collect();
        assert!(
            clash.is_empty(),
            "errno.c and another reference TU both define {clash:?} — one of them must \
             stop being renamed into ref_*"
        );
    }

    // The two PSDK headers errno.c includes that no other reference TU does.  Generated
    // into OUT_DIR beside the scanf TU's windef/winbase/winternl/conio, for the same
    // reason: the whole errno reference stays derived rather than checked in.
    fs::write(
        out.join("ntstatus.h"),
        "/* GENERATED by build.rs — errno.c's `#include \"ntstatus.h\"`. It includes this\n\
         * only for STATUS_INVALID_CRUNTIME_PARAMETER, used by the RaiseException arm of\n\
         * _invalid_parameter, which is `#if _MSVCR_VER >= 80` and so not compiled here.\n\
         * The value is still supplied rather than omitted: an empty header would make a\n\
         * later _MSVCR_VER bump fail as an undeclared identifier instead of working. */\n\
         #ifndef MSVCRT_DIFFTEST_NTSTATUS_H\n#define MSVCRT_DIFFTEST_NTSTATUS_H\n\
         #define STATUS_INVALID_CRUNTIME_PARAMETER ((NTSTATUS)0xC0000417L)\n\
         #endif\n",
    )
    .unwrap();
    fs::write(
        out.join("excpt.h"),
        "/* GENERATED by build.rs — errno.c's `#include \"excpt.h\"`. Reached only by the\n\
         * `#if _MSVCR_VER >= 80` RaiseException arm, which is not compiled here. */\n\
         #ifndef MSVCRT_DIFFTEST_EXCPT_H\n#define MSVCRT_DIFFTEST_EXCPT_H\n\
         #define EXCEPTION_NONCONTINUABLE 0x01\n#endif\n",
    )
    .unwrap();

    let mut ep = String::from(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included at the top of the errno.c reference TU ONLY. */\n\
         #ifndef MSVCRT_REF_ERRNO_PRELUDE_H\n#define MSVCRT_REF_ERRNO_PRELUDE_H\n\n\
         /* ═══ stage 0: __doserrno, onto the name file.c already calls. ═══════════\n\
          * refstub/msvcrt_file_prelude.h and fileimpl.c each spell this mapping out\n\
          * locally — it predates errno.c being a reference TU — so the definition has to\n\
          * meet them at msvcrt_ref___doserrno rather than at ref___doserrno.\n\
          * BEFORE everything, so the one declaration refstub/msvcrt.h makes and the\n\
          * definition errno.c emits are renamed together; doing it last renames only the\n\
          * definition and the two then collide as conflicting types for one symbol. */\n\
         #define __doserrno msvcrt_ref___doserrno\n\n\
         /* ═══ stage 1: hide errno.c's own shells from glibc. ══════════════════════\n\
          * Same reason as the scanf and file TUs: glibc declares strerror/perror and\n\
          * friends, and several carry __REDIRECT or a macro, either of which would\n\
          * attach itself to the name errno.c is about to DEFINE. */\n",
    );
    for sym in &errno_wall {
        ep.push_str(&format!("#define {sym} msvcrt_ref_glibc_{sym}\n"));
    }
    ep.push_str(
        "\n/* ═══ stage 2: the union prelude — sibling renames + the refstub. ════════\n\
         * strcpy/strlen/wcslen/wcstombs are string.c's and wcs.c's, and this is what\n\
         * binds them to ref_* instead of letting the reference's error strings run\n\
         * through the HOST libc — or, worse, through cnt-rust's #[no_mangle] copies,\n\
         * which is the code under test. */\n\
         #include \"ref_prelude.h\"\n\n\
         /* ═══ stage 3: file.c's renames, for the one symbol errno.c borrows. ═════\n\
          * `perror` writes with `_write(2, ...)`, which file.c owns (reference TU #6).\n\
          * Without this the call escapes to the host's _write — or to the SUBJECT's,\n\
          * since file.rs exports one — and a reference that calls the code under test\n\
          * is not a reference. The whole wall is applied rather than just _write: the\n\
          * set is asserted disjoint from errno.c's own above, so it costs nothing, and\n\
          * a future errno.c that reaches for another stream function stays correct. */\n",
    );
    for sym in &file_wall {
        ep.push_str(&format!("#define {sym} ref_{sym}\n"));
    }
    ep.push_str(
        "\n/* ═══ stage 4: un-hide errno.c's own shells and rename them for real. ════ */\n",
    );
    for sym in &errno_wall {
        ep.push_str(&format!("#undef {sym}\n#define {sym} ref_{sym}\n"));
    }
    ep.push_str(
        "\n/* ═══ stage 5: the exported STATE. ═══════════════════════════════════════\n\
         * MSVCRT__sys_errlist / MSVCRT__sys_nerr. The subject defines both, so leaving\n\
         * them plain is a duplicate-symbol link error — and if one copy won, strerror\n\
         * on both sides would read ONE table and the row would compare a table against\n\
         * itself. */\n",
    );
    for sym in &errno_data {
        ep.push_str(&format!("#define {sym} ref_{sym}\n"));
    }
    ep.push_str(
        "\n/* ═══ stage 6: _invalid_parameter, out of the import namespace. ══════════\n\
         * refstub/msvcrt.h maps this to msvcrt_ref__invalid_parameter for every TU that\n\
         * includes it — but refimpl.c DEFINES that name (the call counter every suite\n\
         * reads), so leaving the mapping in place here is a duplicate symbol. Give\n\
         * errno.c's body its own ref_ name; refimpl.c's wrapper counts, then calls it. */\n\
         #undef _invalid_parameter\n#define _invalid_parameter ref__invalid_parameter\n\n\
         /* …and the rename must NOT capture errno.c's own CALLS, only its definition.\n\
          * MSVCRT_INVALID_PMT is verbatim msvcrt (refstub/msvcrt.h:505, itself a copy of\n\
          * dll/win32/msvcrt/msvcrt.h:376) and spells `_invalid_parameter`, so the #define\n\
          * above redirected every MSVCRT_CHECK_PMT inside errno.c straight to its own body\n\
          * and around refimpl.c's counter. The handler then fired without being counted:\n\
          * `_wcserror_s(nc=0)` reported pmt=0 on the reference and pmt=1 on the subject,\n\
          * and the SUBJECT was right. Naming the wrapper explicitly here restores the\n\
          * composition — macro -> counter -> errno.c's real body — and leaves the bare\n\
          * token free to rename the definition alone. */\n\
         void __cdecl msvcrt_ref__invalid_parameter(const wchar_t *, const wchar_t *,\n\
                                                    const wchar_t *, unsigned int, uintptr_t);\n\
         #undef MSVCRT_INVALID_PMT\n\
         #define MSVCRT_INVALID_PMT(x,err) (*_errno() = (err), \\\n\
             msvcrt_ref__invalid_parameter(NULL, NULL, NULL, 0, 0))\n\n\
         /* ═══ stage 7: the DERIVED Win32/CRT constants. ══════════════════════════\n\
          * errno.c's mapper dispatches on 42 ERROR_* codes and clamps against the errno\n\
          * range. file.c reaches these through msvcrt_file_prelude.h, which also drags in\n\
          * the whole FILE layout; errno.c touches no FILE field, so it takes the constant\n\
          * header directly. Derived by the pinned cross-compiler, never transcribed —\n\
          * a hand-written ERROR_* wall next to a switch on it is precisely the §9.112\n\
          * shape, where a wrong or missing case silently becomes the default arm. */\n\
         #include <msvcrt_crt_consts.h>\n\n\
         /* ═══ stage 8: a prototype for the renamed __doserrno. ═══════════════════\n\
          * errno.c's `msvcrt_set_errno` calls `__doserrno()` at line 135, 88 lines above\n\
          * its definition, so without this the implicit-declaration rule types it `int()`\n\
          * and truncates the returned POINTER to 32 bits. fileimpl.c carries the same\n\
          * declaration for the same reason, and -Werror=implicit-function-declaration is\n\
          * what turns it from a silent truncation into a build failure. Placed here, after\n\
          * the refstub has supplied __msvcrt_ulong and CDECL, and after the stage-0 rename\n\
          * so it describes the symbol errno.c actually emits. */\n\
         __msvcrt_ulong * CDECL __doserrno(void);\n\n\
         /* ═══ stage 9: a prototype for every function errno.c CALLS. ════════════\n\
          * Exactly four, spelled out here rather than pulled from\n\
          * refstub/msvcrt_shim_decls.h: that header declares the whole Win32 surface\n\
          * file.c needs, and errno.c's prelude deliberately carries none of the\n\
          * structs it names (it touches no FILE field, so it takes the constants\n\
          * header directly). Including it here would demand SECURITY_ATTRIBUTES,\n\
          * CRITICAL_SECTION, IO_STATUS_BLOCK and six more that this TU has no reason\n\
          * to know about.\n\
          *\n\
          * Written with the underlying types rather than the LPWSTR/UINT spellings\n\
          * for the same reason — typedefs resolve, so these stay compatible with\n\
          * fileimpl.c's definitions without dragging in windef.\n\
          *\n\
          * The two `lstr*` return a pointer, so the implicit-declaration rule was\n\
          * truncating one here too; harmless only because errno.c discards it. */\n\
         unsigned int WINAPI SetErrorMode(unsigned int mode);\n\
         wchar_t * WINAPI lstrcpyW(wchar_t *dst, const wchar_t *src);\n\
         wchar_t * WINAPI lstrcatW(wchar_t *dst, const wchar_t *src);\n\
         int CDECL ref__write(int fd, const void *buf, unsigned int count);\n\n\
         #endif /* MSVCRT_REF_ERRNO_PRELUDE_H */\n",
    );
    let errno_prelude_h = out.join("ref_errno_prelude.h");
    fs::write(&errno_prelude_h, &ep).unwrap();

    // ══ reference TU #11 — dll/win32/msvcrt/time.c ══════════════════════════════════
    //
    // The substrate landed a wave early: timeimpl.c (the frozen clock and the four zone
    // presets) has been in the link since time.rs was flipped, because the SUBJECT needed
    // it. What was missing was the other half of the pair — msvcrt's own calendar,
    // compiled for the host, so the two can be driven against each other.
    //
    // It is worth being explicit about what that buys, because time.c is the file where a
    // plausible-looking harness grades nothing. Everything interesting here is ARITHMETIC
    // over a Win32 answer — leap years, the DST window, the 1601 vs 1970 epoch shift, the
    // half-hour bias, `_MAX__TIME64_T` — and every one of those is reachable only because
    // the instant and the zone are KNOBS. Against the host's real clock the two sides would
    // be called a microsecond apart and agree by luck; against a frozen instant they agree
    // or they do not.
    //
    // Also, this is the wave that DELETES fileimpl.c's `ref__time64` stub. file.c's
    // `_wutime`/`_futime` path calls `_time64`, and the harness has answered it with a
    // three-line host-clock stub since wave 18 — which meant every file row that stamped a
    // modification time was graded against the harness's idea of "now" rather than
    // msvcrt's. time.c defines the real one, so the stub's socket (`ref__time64`, which
    // ref_file_prelude.h's stage 4.5 already maps `_time64` to) is now filled by the
    // genuine body. Same shape as errno.c's arrival deleting `msvcrt_set_errno`.
    let time_c = msvcrt_dir.join("time.c");
    assert!(time_c.is_file(), "msvcrt reference source not found: {}", time_c.display());

    // From the OBJECT, like file.c and errno.c: `_MSVCR_VER` opens and closes whole
    // families here (`_ctime32_s`, the `_MAX__TIME64_T` clamps, `MAX_SECONDS`), so what got
    // EMITTED is the authority and the source text is not.
    let time_obj =
        repo.join("build-amd64/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/time.c.obj");
    let time_code: Vec<String> = object_symbols(&repo, &time_obj, &["T"]);
    let time_data: Vec<String> = object_symbols(&repo, &time_obj, &["D", "B"]);
    assert!(
        time_code.len() > 45 && time_data.len() == 4,
        "time.c object scrape found {} functions / {} data — expected ~52 and exactly the \
         four exported globals (__daylight/__timezone/_dstbias/_tzname). A short list would \
         leave symbols under their PLAIN names, colliding with the subject",
        time_code.len(),
        time_data.len()
    );
    // The four exported globals, pinned. They are the reason the data half of the scrape
    // exists at all: msvcrt.spec publishes each of them, so time.rs defines them
    // `#[no_mangle]`, and an un-renamed reference copy is a duplicate symbol. They are also
    // WRITTEN by `_tzset` — code under test — so unlike ctype.c's tables they must be two
    // objects, or each side would read back what the other stored (errno.c's stage 5 spells
    // the same failure out for `MSVCRT__sys_errlist`).
    for want in ["MSVCRT___daylight", "MSVCRT___timezone", "MSVCRT__dstbias", "MSVCRT__tzname"]
    {
        assert!(
            time_data.iter().any(|s| s == want),
            "time.c's data scrape missed `{want}` — the reference copy would keep its plain \
             name and share one slot with the subject's"
        );
    }

    // Disjointness against every other reference TU, asserted rather than assumed.
    {
        let taken: std::collections::HashSet<&str> = all_symbols
            .iter()
            .copied()
            .chain(scanf_symbols.iter().map(String::as_str))
            .chain(file_symbols.iter().map(String::as_str))
            .chain(file_data.iter().map(String::as_str))
            .chain(errno_code.iter().map(String::as_str))
            .chain(errno_data.iter().map(String::as_str))
            .collect();
        let clash: Vec<&str> = time_code
            .iter()
            .chain(time_data.iter())
            .map(String::as_str)
            .filter(|s| taken.contains(s))
            .collect();
        assert!(
            clash.is_empty(),
            "time.c and another reference TU both define {clash:?} — one of them must stop \
             being renamed into ref_*"
        );
    }

    // The seven names the `#define` wall must NOT touch, generated alongside the
    // prototypes. All seven are the SILENT class rather than the loud one: time.c opens
    // with `#undef _ctime32` … `#undef _wctime32`, because in the real build those names
    // are time_t-width macros from msvcrt's own <time.h>. Each `#undef` deletes our
    // rename, and the function is then emitted under its true name — which is exactly what
    // happened on the first build here: seven duplicate symbols against time.rs's
    // `#[no_mangle]` copies, and had the subject not defined them the failure would have
    // been the quieter one (a `ref__time32` nothing defines, unnoticed until a row reached
    // it). They get an asm label in msvcrt_time_protos.h instead, which renames at the
    // object level where no `#undef` can reach.
    let time_no_rename_txt = refstub.join("msvcrt_time_no_rename.txt");
    let time_no_rename: std::collections::HashSet<String> =
        fs::read_to_string(&time_no_rename_txt)
            .unwrap_or_else(|e| panic!("{}: {e}", time_no_rename_txt.display()))
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect();
    for n in &time_no_rename {
        assert!(
            time_code.contains(n),
            "msvcrt_time_no_rename.txt lists `{n}`, which time.c does not export — \
             regenerate: python3 tools/gen_msvcrt_ref_protos.py time.c"
        );
    }
    let time_wall: Vec<&String> =
        time_code.iter().filter(|s| !time_no_rename.contains(*s)).collect();

    // `mtdll.h` is msvcrt's lock-index header. time.c includes it for `_TIME_LOCK` (which
    // `_tzset` takes around the zone globals) and nothing else, and the harness cannot
    // reach the real one: dll/win32/msvcrt is not on the include path, precisely so its
    // private ctype.h/stdlib.h/stdio.h do not capture the host's. Forward to the genuine
    // header by absolute path rather than transcribe the index — a lock number that drifted
    // would be invisible, since fileimpl.c's `_lock` is a no-op counter and both sides
    // would pass the same wrong value to it.
    fs::write(
        out.join("mtdll.h"),
        format!(
            "/* GENERATED by build.rs — time.c's `#include \"mtdll.h\"`, forwarded to the\n\
             * GENUINE header. Not transcribed: `_TIME_LOCK` is an index into msvcrt's lock\n\
             * table and a drifted one is invisible to a differential test. */\n\
             #ifndef MSVCRT_DIFFTEST_MTDLL_H\n#define MSVCRT_DIFFTEST_MTDLL_H\n\
             #include \"{}\"\n#endif\n",
            msvcrt_dir.join("mtdll.h").display()
        ),
    )
    .unwrap();

    let mut tp = String::from(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included at the top of the time.c reference TU ONLY. */\n\
         #ifndef MSVCRT_REF_TIME_PRELUDE_H\n#define MSVCRT_REF_TIME_PRELUDE_H\n\n\
         /* ═══ stage 1: hide time.c's own shells from glibc. ═══════════════════════\n\
          * Sharper here than in any earlier TU. time.c includes <time.h>, and glibc\n\
          * declares `clock`, `asctime`, `strftime` and `wcsftime` in it — with\n\
          * __THROW/__nonnull attributes and, for several, a __REDIRECT. Any of those\n\
          * would attach itself to the name time.c is about to DEFINE, and the definition\n\
          * would then collide with its own declaration. Hidden for the duration of the\n\
          * host headers; renamed for real in stage 3. */\n",
    );
    for sym in &time_wall {
        tp.push_str(&format!("#define {sym} msvcrt_ref_glibc_{sym}\n"));
    }
    tp.push_str(
        "\n/* ═══ stage 2: the union prelude — sibling renames + the refstub. ════════\n\
         * strcpy/strlen/memcpy are string.c's; `_snprintf`, `_mbstowcs_l` and\n\
         * `_wcstombs_l` are wcs.c's; `_isleadbyte_l` is mbcs.c's. This is what binds\n\
         * them to ref_* instead of letting the reference's own output run through the\n\
         * host libc — or through cnt-rust's #[no_mangle] copies, which are the code\n\
         * under test.\n\
         *\n\
         * `_snprintf` is the load-bearing one and it is safe for a reason worth stating:\n\
         * `strftime_int` renders every numeric conversion with it and checks for -1, so\n\
         * it needs msvcrt's TRUNCATION CONTRACT, not C99's. `ref__snprintf` is wcs.c's\n\
         * own body forwarding to plain `vsnprintf`, which the shared prelude's stage 4\n\
         * points at `msvcrt_ref_vsnprintf` — the host formatter under msvcrt's contract.\n\
         * The subject reaches the same body through subjectimpl.c's stand-in, so both\n\
         * sides format identically and what is graded is the calendar. */\n\
         #include \"ref_prelude.h\"\n\n\
         /* ═══ stage 2b: the host <time.h>, pulled WHILE the names are still hidden. ═\n\
          * time.c includes it at line 27, i.e. after this prelude, and two things have to\n\
          * happen to it before then.\n\
          *\n\
          * (a) It must be read while stage 1's hiding is in force, so glibc's `clock`,\n\
          *     `asctime`, `strftime` and `wcsftime` declarations attach to the\n\
          *     msvcrt_ref_glibc_* spellings and not to the names time.c defines.\n\
          *\n\
          * (b) Its `typedef __clock_t clock_t` must land BEFORE the pin below. A macro\n\
          *     named `clock_t` in scope when glibc's typedef is parsed rewrites the\n\
          *     typedef itself, which is a conflicting definition rather than a rename.\n\
          *\n\
          * Included explicitly rather than relied on: whether the refstub happens to drag\n\
          * <time.h> in today is not something this TU should depend on. */\n\
         #include <time.h>\n\n\
         /* ═══ stage 3: un-hide time.c's own shells and rename them for real. ═════ */\n",
    );
    for sym in &time_wall {
        tp.push_str(&format!("#undef {sym}\n#define {sym} ref_{sym}\n"));
    }
    tp.push_str(
        "\n/* ═══ stage 4: the four exported globals. ════════════════════════════════\n\
         * `_tzset` WRITES all four, so they must be per-side storage; see build.rs. */\n",
    );
    for sym in &time_data {
        tp.push_str(&format!("#define {sym} ref_{sym}\n"));
    }
    tp.push_str(
        "\n/* ═══ stage 5: the Win32 time vocabulary and the derived constants. ══════\n\
         * SYSTEMTIME/FILETIME/TIME_ZONE_INFORMATION and the two `__timeb` structs, plus\n\
         * the ERROR_, CP_ACP and TIME_ZONE_ID_ families. These come AFTER the refstub\n\
         * (stage 2) because they are written in its Windows type model — WORD, LONG,\n\
         * WCHAR.\n\
         *\n\
         * The host's <sys/timeb.h> is a real header that glibc ships, and time.c includes\n\
         * it: it declares `struct timeb` and `ftime`, neither of which msvcrt has, and it\n\
         * does NOT declare `struct __timeb32`/`__timeb64`. So the two coexist rather than\n\
         * conflict, and `_ftime32`/`_ftime64` fill msvcrt's structs. */\n\
         #include <msvcrt_file_types.h>\n\
         #include <msvcrt_crt_consts.h>\n\
         #include <msvcrt_time_types.h>\n\n\
         /* `_MAX__TIME64_T` — the last instant msvcrt's 64-bit time_t admits, 3000-12-31\n\
          * 23:59:59Z. time.c clamps against it in six places (`_gmtime64_s`'s range test,\n\
          * `_ctime64_s`, `_mktime64`'s overflow arm) and it is the single most interesting\n\
          * boundary in the file, so it is a CONSTANT the reference must have rather than a\n\
          * branch it silently loses. Spelled the same way ref_file_prelude.h spells it —\n\
          * one value, two preludes, and the disagreement would be silent. */\n\
         #ifndef _MAX__TIME64_T\n\
         #define _MAX__TIME64_T    (((__time64_t)0x00000007 << 32) | 0x93406FFF)\n\
         #endif\n\n\
         /* `clock_t` is msvcrt's `__msvcrt_long` — 32 bits — and the host's is 64.\n\
          * `clock()` is four lines and looks impossible to get wrong:\n\
          *\n\
          *     return (systime.QuadPart - init_time) / (TICKSPERSEC / CLOCKS_PER_SEC);\n\
          *\n\
          * but BOTH of the things it depends on differ between the two type models, and\n\
          * neither difference is a compile error. The WIDTH decides whether the division\n\
          * result is truncated to 32 bits — msvcrt's clock() wraps after ~24.9 days and\n\
          * the host's does not — and CLOCKS_PER_SEC decides the UNIT: msvcrt's is 1000,\n\
          * glibc's is 1000000, so an unpinned reference answers in microseconds and every\n\
          * row is off by exactly 1000x. The constant is DERIVED (msvcrt_crt_consts.h,\n\
          * which #undefs glibc's first); the type is pinned here, after <time.h> so it\n\
          * renames time.c's use rather than glibc's typedef. */\n\
         #undef clock_t\n\
         #define clock_t __msvcrt_long\n\n\
         /* ═══ stage 6: a prototype for every function time.c CALLS. ══════════════\n\
          * The Win32 clock/zone layer (timeimpl.c) and the two NLS converters\n\
          * (refimpl.c). Spelled out here rather than pulled from\n\
          * refstub/msvcrt_shim_decls.h for errno.c's reason: that header declares the\n\
          * whole file.c Win32 surface and would demand SECURITY_ATTRIBUTES,\n\
          * CRITICAL_SECTION and six more structs this TU has no reason to know about.\n\
          *\n\
          * Without them C's implicit-declaration rule types each as `int()`, and\n\
          * -Werror=implicit-function-declaration is what turns that from a silently\n\
          * truncated return into a build failure. */\n\
         void    WINAPI GetSystemTimeAsFileTime(FILETIME *ft);\n\
         NTSTATUS WINAPI NtQuerySystemTime(LARGE_INTEGER *t);\n\
         void    WINAPI GetLocalTime(SYSTEMTIME *st);\n\
         BOOL    WINAPI SystemTimeToFileTime(const SYSTEMTIME *st, FILETIME *ft);\n\
         BOOL    WINAPI FileTimeToSystemTime(const FILETIME *ft, SYSTEMTIME *st);\n\
         BOOL    WINAPI SystemTimeToTzSpecificLocalTime(const TIME_ZONE_INFORMATION *tzi,\n\
         \x20                                           const SYSTEMTIME *utc,\n\
         \x20                                           SYSTEMTIME *local);\n\
         DWORD   WINAPI GetTimeZoneInformation(TIME_ZONE_INFORMATION *tzi);\n\
         int     WINAPI GetDateFormatW(LCID lcid, DWORD flags, const SYSTEMTIME *st,\n\
         \x20                          const WCHAR *fmt, WCHAR *out, int cch);\n\
         int     WINAPI GetTimeFormatW(LCID lcid, DWORD flags, const SYSTEMTIME *st,\n\
         \x20                          const WCHAR *fmt, WCHAR *out, int cch);\n\
         LPSTR   WINAPI lstrcpynA(LPSTR dst, LPCSTR src, int n);\n\
         int     WINAPI MultiByteToWideChar(UINT cp, DWORD flags, LPCSTR src, int srclen,\n\
         \x20                               LPWSTR dst, int dstlen);\n\
         int     WINAPI WideCharToMultiByte(UINT cp, DWORD flags, LPCWSTR src, int srclen,\n\
         \x20                               LPSTR dst, int dstlen, LPCSTR defchar,\n\
         \x20                               BOOL *used);\n\
         void    CDECL  _lock(int locknum);\n\
         void    CDECL  _unlock(int locknum);\n\n\
         /* …and the two sibling-TU converters `_wasctime`/`_wstrdate`/`_Strftime` reach\n\
          * for. wcs.c owns `_wcstombs_l` and mbcs.c owns `_mbstowcs_l`, so both are\n\
          * already renamed by stage 2 — these declare the ref_ spelling with the RIGHT\n\
          * prototype. Both return `size_t`, and that is the point: without a declaration\n\
          * C types them `int()`, and `(size_t)-1` — the conversion-failed sentinel every\n\
          * caller here tests for — comes back as 0x00000000FFFFFFFF, which compares\n\
          * unequal to it. The failure arm would simply never be taken. */\n\
         size_t  CDECL  _wcstombs_l(char *mbstr, const wchar_t *wcstr, size_t count,\n\
         \x20                       _locale_t locale);\n\
         size_t  CDECL  _mbstowcs_l(wchar_t *wcstr, const char *mbstr, size_t count,\n\
         \x20                       _locale_t locale);\n\n\
         /* ═══ stage 7: a prototype for every function time.c DEFINES. ═══════════\n\
          * Two jobs in one header. (a) The 45 renamed ones are declared here, AFTER the\n\
          * wall, so each is renamed alongside its definition — without them the calls\n\
          * time.c places above a definition (`_gmtime64_s` from `_ctime64_s`,\n\
          * `_mktime64` from `_mkgmtime64`) fall to C's implicit-declaration rule and\n\
          * every `__time64_t` return is truncated to 32 bits, in code that links.\n\
          * (b) The seven `#undef`ed ones carry their rename as an `__asm__` label, which\n\
          * is the only form a later `#undef` cannot delete. */\n\
         #include <msvcrt_time_protos.h>\n\n\
         #endif /* MSVCRT_REF_TIME_PRELUDE_H */\n",
    );
    let time_prelude_h = out.join("ref_time_prelude.h");
    fs::write(&time_prelude_h, &tp).unwrap();

    // ══ reference TU #12 — dll/win32/msvcrt/process.c ═══════════════════════════════
    //
    // process.c does not create processes.  It MARSHALS — argv array or `...` list into
    // one command line, envp array into a double-NUL block, a bare program name into a
    // full path by walking PATH and trying four extensions, a `_P_*` mode into creation
    // flags and a wait-or-return decision — and then calls CreateProcessW.  Everything
    // the file is FOR happens before that call, which is why processimpl.c's
    // CreateProcessW is a RECORDER: the row compares what the two sides marshalled, not
    // what some child process did.
    //
    // Two things make this TU unlike its predecessors.
    //
    // 1. THE `...` ENTRY POINTS ARE GRADED THROUGH REAL VARIADICS.  Sixteen of the 41
    //    exports are WINAPIV, and the SUBJECT cannot supply them: `va_start` is not on
    //    pinned stable, so the shipping msvcrt has them as C trampolines in
    //    rust_process_varargs_shim.c over `_v`-prefixed Rust twins, and those twins walk
    //    a flat `*mut u8` — the Windows-x64 va_list, not the host's SysV struct.  The
    //    scanf TU solved that shape by compiling against refstub_printf's flat
    //    <stdarg.h>; here the flat header would BREAK the reference instead, because
    //    its `va_start(ap, last)` expands to `ap = NULL` and these bodies genuinely
    //    start their own lists.  So the reference keeps the host's stdarg and is called
    //    with real varargs, the subject's `_v*` twin is handed the same logical
    //    arguments hand-packed into slots, and what is compared is the RECORDED SPAWN.
    //    The two va_list models never have to meet.
    //
    // 2. THE SUCCESS PATH OF `_exec*` CANNOT BE A ROW.  `_exec*` passes `_P_OVERLAY`,
    //    and msvcrt_spawn's `_P_OVERLAY` arm is `_exit(0)` — on success the harness
    //    process terminates.  With CreateProcessW preset to FAIL the marshalling has
    //    already happened and been recorded, and what comes back is the errno mapping,
    //    so the sixteen exec entry points are graded on everything except the two
    //    instructions after the call.  tests/process.rs says so where the rows are.
    let process_c = msvcrt_dir.join("process.c");
    assert!(process_c.is_file(), "msvcrt reference source not found: {}", process_c.display());

    // From the OBJECT, like every TU since file.c.  `_wsystem`/`system` sit inside
    // `#ifndef __REACTOS__` and are NOT built, and three of the file-statics are inlined
    // away entirely — the source text would claim symbols that do not exist.
    let process_obj =
        repo.join("build-amd64/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/process.c.obj");
    let process_code: Vec<String> = object_symbols(&repo, &process_obj, &["T"]);
    let process_data: Vec<String> = object_symbols(&repo, &process_obj, &["D", "B"]);
    assert!(
        process_code.len() > 35 && process_data.is_empty(),
        "process.c object scrape found {} functions / {} data — expected ~41 and NO \
         exported data (popen_handles and popen_handles_size are file-static). A short \
         list would leave exports under their PLAIN names, colliding with the subject",
        process_code.len(),
        process_data.len()
    );
    // The four families, pinned by one representative each. They are what the wall is
    // for: process.rs defines all of them `#[no_mangle]`, so an un-renamed reference
    // copy is a duplicate symbol — and the quieter failure, if the subject had not
    // defined one, would be a `ref__spawnv` nothing defines that no row notices until
    // it reaches for it.
    for want in ["_spawnv", "_wexecl", "_wpopen", "_getdllprocaddr"] {
        assert!(
            process_code.iter().any(|s| s == want),
            "process.c's scrape missed `{want}` — that family would keep its plain name"
        );
    }

    let process_no_rename_txt = refstub.join("msvcrt_process_no_rename.txt");
    let process_no_rename: std::collections::HashSet<String> =
        fs::read_to_string(&process_no_rename_txt)
            .unwrap_or_else(|e| panic!("{}: {e}", process_no_rename_txt.display()))
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(str::to_string)
            .collect();
    for n in &process_no_rename {
        assert!(
            process_code.contains(n),
            "msvcrt_process_no_rename.txt lists `{n}`, which process.c does not export — \
             regenerate: python3 tools/gen_msvcrt_ref_protos.py process.c"
        );
    }
    let process_wall: Vec<&String> =
        process_code.iter().filter(|s| !process_no_rename.contains(*s)).collect();

    // Disjointness against every other reference TU, asserted rather than assumed.
    {
        let taken: std::collections::HashSet<&str> = all_symbols
            .iter()
            .copied()
            .chain(scanf_symbols.iter().map(String::as_str))
            .chain(file_symbols.iter().map(String::as_str))
            .chain(file_data.iter().map(String::as_str))
            .chain(errno_code.iter().map(String::as_str))
            .chain(errno_data.iter().map(String::as_str))
            .chain(time_code.iter().map(String::as_str))
            .chain(time_data.iter().map(String::as_str))
            .collect();
        let clash: Vec<&str> = process_code
            .iter()
            .map(String::as_str)
            .filter(|s| taken.contains(s))
            .collect();
        assert!(
            clash.is_empty(),
            "process.c and another reference TU both define {clash:?} — one of them must \
             stop being renamed into ref_*"
        );
    }

    // process.c's `#include <process.h>`. Everything it wants from it — the five `_P_*`
    // modes and the exec/spawn prototypes — arrives from msvcrt_crt_consts.h (derived)
    // and msvcrt_process_protos.h (generated from the definitions themselves), so this
    // only has to exist. Emitted rather than left out because glibc has no <process.h>
    // at all and the include would simply fail.
    fs::write(
        out.join("process.h"),
        "/* GENERATED by build.rs — process.c's `#include <process.h>`. The _P_* modes\n\
         * come from msvcrt_crt_consts.h and the prototypes from\n\
         * msvcrt_process_protos.h, both derived; this is the empty socket. */\n\
         #ifndef MSVCRT_DIFFTEST_PROCESS_H\n#define MSVCRT_DIFFTEST_PROCESS_H\n#endif\n",
    )
    .unwrap();

    // ── the codepage inlines are TRANSCRIBED; check the transcription ──────────────
    //
    // `get_aw_cp`, `convert_acp_utf8_to_wcs`, `convert_wcs_to_acp_utf8`,
    // `wstrdupa_utf8` and `astrdupw_utf8` are `static inline` in
    // dll/win32/msvcrt/msvcrt.h, and refstub/msvcrt_file_prelude.h carries a hand copy
    // because suppressing that header dropped them.  Its comment says "VERBATIM from
    // that header" — a claim nothing was checking.
    //
    // process.c makes that claim load-bearing in a new way.  These are not plumbing
    // here: `msvcrt_argvtos_aw` picks its codepage with `env ? CP_ACP : get_aw_cp()`,
    // `msvcrt_valisttos_aw` sizes every argument with `convert_acp_utf8_to_wcs`, and
    // process.rs carries its OWN hand-written copies of both.  A transcription that
    // drifted would be §9.66 with both sides on the same wrong side of it — an oracle
    // rewritten by the harness, agreeing with a subject that copied the same
    // misunderstanding, on the two functions whose entire job is the conversion.
    //
    // So: compare the two texts, whitespace-normalised, at build time.  `get_aw_cp` is
    // excluded from the comparison by design — the real one has an `#if
    // _MSVCR_VER>=140` arm that this build does not compile, and the collapse to
    // `return CP_ACP;` is the deliberate part the refstub's own comment explains.  The
    // OTHER four have no conditional arm, so any difference at all is drift.
    {
        let msvcrt_h_src = fs::read_to_string(msvcrt_dir.join("msvcrt.h"))
            .unwrap_or_else(|e| panic!("dll/win32/msvcrt/msvcrt.h: {e}"));
        let prelude_src = fs::read_to_string(refstub.join("msvcrt_file_prelude.h"))
            .unwrap_or_else(|e| panic!("refstub/msvcrt_file_prelude.h: {e}"));
        // Whitespace-normalised so indentation and line breaks do not count as drift;
        // every token, and their order, still does.
        let squash = |s: &str| s.split_whitespace().collect::<Vec<_>>().join(" ");
        let body = |src: &str, sig: &str| -> String {
            let start = src.find(sig).unwrap_or_else(|| {
                panic!(
                    "`{sig}` is gone — the transcription check is blind, and a blind check \
                     over a hand copy of code under test is worse than none"
                )
            });
            let open = src[start..].find('{').expect("no body") + start;
            let mut depth = 0usize;
            let mut end = open;
            for (i, c) in src[open..].char_indices() {
                if c == '{' { depth += 1 }
                if c == '}' {
                    depth -= 1;
                    if depth == 0 { end = open + i + 1; break }
                }
            }
            squash(&src[start..end])
        };
        for sig in [
            "static inline int convert_acp_utf8_to_wcs(",
            "static inline int convert_wcs_to_acp_utf8(",
            "static inline wchar_t* wstrdupa_utf8(",
            "static inline char* astrdupw_utf8(",
        ] {
            let genuine = body(&msvcrt_h_src, sig);
            let copy = body(&prelude_src, sig);
            assert_eq!(
                copy, genuine,
                "refstub/msvcrt_file_prelude.h's copy of `{sig}…` has drifted from \
                 dll/win32/msvcrt/msvcrt.h. It says VERBATIM and it is not. Both the \
                 file.c and process.c reference TUs inline this body, so the drift is a \
                 rewritten oracle, not a cosmetic difference.\nfix: re-copy the body."
            );
        }
        assert!(
            squash(&body(&prelude_src, "static inline UINT get_aw_cp(")).contains("return CP_ACP;"),
            "refstub's collapsed get_aw_cp no longer returns CP_ACP — at _MSVCR_VER=0 that \
             IS the whole function, and the `_aw` marshallers branch on it"
        );
    }

    let mut pp = String::from(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included at the top of the process.c reference TU ONLY. */\n\
         #ifndef MSVCRT_REF_PROCESS_PRELUDE_H\n#define MSVCRT_REF_PROCESS_PRELUDE_H\n\n\
         /* ═══ stage 0: msvcrt's FILE, before ANY host header. ════════════════════\n\
          * `_popen` returns one and `_pclose` reads `popen_handles[i].f == file`, so the\n\
          * reference and the subject must agree on what a FILE pointer IS before either\n\
          * can be compared. Same first-header rule as the file.c TU. */\n\
         #include <msvcrt_file_pre.h>\n\n\
         /* ═══ stage 1: hide process.c's own exports from glibc. ══════════════════\n\
          * Cheaper than deciding which of the 41 glibc happens to declare today.\n\
          * `system` would be the obvious one and it is not even built here (it sits\n\
          * inside `#ifndef __REACTOS__`), which is exactly why the wall is applied by\n\
          * rule rather than by inspection. */\n",
    );
    for sym in &process_wall {
        pp.push_str(&format!("#define {sym} msvcrt_ref_glibc_{sym}\n"));
    }
    pp.push_str(
        "\n/* ═══ stage 2: the union prelude — sibling renames + the refstub. ════════\n\
         * `wcslen`/`wcscpy`/`wcscat`/`memcpy`/`strlen` are string.c's and wcs.c's.\n\
         * `msvcrt_search_executable` builds every candidate path with three of them and\n\
         * `msvcrt_argvtos` measures every argument with a fourth, so without this the\n\
         * reference's path arithmetic would run through cnt-rust's #[no_mangle] copies —\n\
         * the code under test — and the harness would be grading Rust against Rust. */\n\
         #include \"ref_prelude.h\"\n\n\
         /* ═══ stage 3: the types, the layout guardian, the constants. ════════════\n\
          * STARTUPINFOW and PROCESS_INFORMATION arrive here, and both are load-bearing\n\
          * rather than decorative: msvcrt_spawn fills `cbReserved2`/`lpReserved2` from\n\
          * file.c's inherit block, and its `_P_WAIT` arm writes the EXIT CODE into\n\
          * `pi.dwProcessId` and returns that. msvcrt_crt_layout.h asserts every offset\n\
          * against the pinned cross-compiler reading the real PSDK. */\n\
         #include <msvcrt_file_prelude.h>\n\n\
         /* ═══ stage 4: file.c's wall, for the eight symbols process.c borrows. ═══\n\
          * `_pipe`, `_dup`, `_close`, `_wfdopen`, `fclose`, `_get_osfhandle` and\n\
          * `msvcrt_create_io_inherit_block` are file.c's (reference TU #6). Without\n\
          * this, `_wpopen` would build its pipe with the SUBJECT's fd table and then\n\
          * compare the fd numbers against a reference that used the same table — two\n\
          * sides sharing one piece of code under test, which is not a difference.\n\
          * The whole wall is applied rather than the eight, exactly as errno.c's\n\
          * prelude does: the sets are asserted disjoint above, so it costs nothing and\n\
          * stays right if process.c reaches for a ninth. */\n",
    );
    for sym in &file_wall {
        pp.push_str(&format!("#define {sym} ref_{sym}\n"));
    }
    pp.push_str(
        "\n/* ═══ stage 5: un-hide process.c's own exports and rename them for real. ═ */\n",
    );
    for sym in &process_wall {
        pp.push_str(&format!("#undef {sym}\n#define {sym} ref_{sym}\n"));
    }
    pp.push_str(&format!(
        "\n/* ═══ stage 6: a prototype for every function process.c DEFINES. ════════\n\
         * After the wall, so each is renamed alongside its definition. `_wexecve` calls\n\
         * `_wspawnve` 300 lines above it and `_popen` calls `_wpopen` from below —\n\
         * without these, both fall to C's implicit-declaration rule and every returned\n\
         * `intptr_t` and `FILE*` is truncated to 32 bits, in code that links. */\n\
         #include <msvcrt_process_protos.h>\n\n\
         /* ═══ stage 6b: …and for every function file.c defines. ═════════════════\n\
          * The other half of stage 4. Renaming `_pipe` to `ref__pipe` without a\n\
          * declaration in scope just moves WHICH undeclared function gets invented:\n\
          * `ref__wfdopen` would come back as `int` and its FILE* be truncated to 32\n\
          * bits — the exact failure msvcrt_shim_decls.h was written for, one TU over.\n\
          * Included after stage 4 so every prototype is renamed with its definition. */\n\
         #include <msvcrt_file_protos.h>\n\n\
         /* ═══ stage 7: a prototype for every function process.c CALLS. ══════════\n\
          * The Win32 process layer (processimpl.c), the file.c stream layer, and the\n\
          * msvcrt internals from TUs we do not compile. Same header the file.c TU uses,\n\
          * and the seven CreateProcessW-family entries were added to it for this TU. */\n\
         #include <msvcrt_shim_decls.h>\n\n\
         /* ═══ stage 8: `_exit`, onto a terminator that is LOUD. ══════════════════\n\
          * msvcrt_spawn's `_P_OVERLAY` arm is `_exit(0)`, so a spawn that SUCCEEDS with\n\
          * that mode ends the harness process. Every `_exec*` row therefore presets\n\
          * CreateProcessW to fail, which is safe — but \"safe by nobody making a\n\
          * mistake\" is not safe: the failure mode is that the test binary exits ZERO\n\
          * having compared nothing, and `cargo test` reports a pass.\n\
          *\n\
          * Silent-green is the one outcome a differential harness must never have, so\n\
          * the environment's terminator is replaced with one that says what happened\n\
          * and exits non-zero. `_exit` belongs to exit.c, not to process.c — it is the\n\
          * layer BELOW the code under test, exactly like fileimpl.c's Win32 surface —\n\
          * so redirecting it changes the environment, not the reference. Leaving it\n\
          * plain would also bind it to exit.rs's `#[no_mangle]` copy, i.e. the\n\
          * reference calling the code under test. */\n\
         #undef _exit\n\
         #define _exit ref_process_exit_overlay\n\
         void ref_process_exit_overlay(int code);\n\n\
         #endif /* MSVCRT_REF_PROCESS_PRELUDE_H */\n"
    ));
    let process_prelude_h = out.join("ref_process_prelude.h");
    fs::write(&process_prelude_h, &pp).unwrap();

    // ══ reference TUs #14 and #15 — dll/win32/msvcrt/cpp.c + except.c ═══════════════
    //
    // The C++ RTTI/exception-object subsystem and the exception dispatcher: 94 exported
    // symbols between them, flipped to Rust in waves 27 and 29 and graded by NOTHING until
    // now.  They arrive as a PAIR because they are not separable — except.c calls cpp.c's
    // `terminate`, and cppexcept.h inlines that both include reach into each other's types.
    //
    // What is different about these two, and why the machinery below is not just two more
    // `ref_tu` calls: every other reference TU in this file gets its Windows types from
    // refstub/msvcrt.h and its headers from the refstub include path.  These two ask for the
    // NT EXCEPTION model — EXCEPTION_RECORD, DISPATCHER_CONTEXT, the unwind tables, PSEH's
    // __TRY/__EXCEPT_PAGE_FAULT — none of which any previous TU needed.  So they get:
    //
    //   * a SOCKET directory, generated below, shadowing the ten PSDK headers they include
    //     with empty files, exactly as msvcrt-seh-difftest does.  Empty rather than absent so
    //     the `#include` still resolves and a reader of the C sees the same include list
    //     msvcrt.dll compiles.
    //   * refstub/msvcrt_cpp_types.h for the types those sockets no longer supply, PROVED
    //     against the pinned cross-compiler by refstub/msvcrt_cpp_layout.h.
    //   * the REAL sdk/include/wine/exception.h and the REAL sdk/lib/pseh/include/pseh/pseh2.h,
    //     reached rather than stubbed.  Stubbing __TRY would replace the control flow under
    //     test; `-D_USE_DUMMY_PSEH` selects the same arm of pseh2.h that the amd64/arm64 DLL
    //     builds, where `__EXCEPT_PAGE_FAULT` compiles to `if (_SEH2_Volatile0 || …)` and is
    //     never taken.  That is what the SUBJECT models too (see cpp.rs's __RTtypeid), so the
    //     two sides agree about which arm exists — and the fact that the fault path is
    //     unreachable on BOTH is a recorded limit of these rows, not a hidden one.
    let sockets = out.join("cpp_sockets");
    fs::create_dir_all(sockets.join("wine")).unwrap();
    for h in [
        "windef.h",
        "winbase.h",
        "winternl.h",
        "winver.h",
        "imagehlp.h",
        "rtlsupportapi.h",
        "excpt.h",
        "fpieee.h",
        "intrin.h",
        "ntstatus.h",
        "wincon.h",
        // msvcrt's OWN <signal.h> and <float.h>, and these two are the point of the whole
        // socket directory rather than incidental to it.  except.c includes both; without a
        // socket they resolve to GLIBC's, whose SIGABRT is 6 where msvcrt's is 22 and which
        // has no _FPE_* at all.  msvcrt_crt_consts.h supplies the derived values instead.
        "signal.h",
        "float.h",
    ] {
        fs::write(
            sockets.join(h),
            "/* GENERATED by msvcrt-difftest/build.rs: an empty socket. The cpp.c/except.c\n\
             * prelude answers for this header — see refstub/msvcrt_cpp_types.h. */\n",
        )
        .unwrap();
    }

    // The objects, for the rename walls.  From the OBJECT rather than the source, for the
    // reason errno.c and file.c already give: what got EMITTED is the authority, not what the
    // source appears to say through its `_MSVCR_VER` and `__i386__` arms — and except.c has
    // more of those than any other file here.
    let cpp_obj = repo.join("build-amd64/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/cpp.c.obj");
    let except_obj =
        repo.join("build-amd64/dll/win32/msvcrt/CMakeFiles/msvcrt_static.dir/except.c.obj");
    let cpp_code: Vec<String> = object_symbols(&repo, &cpp_obj, &["T"]);
    let cpp_data: Vec<String> = object_symbols(&repo, &cpp_obj, &["D", "B", "R"]);
    let except_code: Vec<String> = object_symbols(&repo, &except_obj, &["T"]);
    let except_data: Vec<String> = object_symbols(&repo, &except_obj, &["D", "B", "R"]);
    // 50 / 22 / 10 is what the amd64 DLL object actually holds, checked against the host
    // compile of the same source: the two symbol sets are IDENTICAL, which is the evidence
    // that the reference TU compiles the same arms the DLL does.  except.c contributes no
    // exported DATA at all — its `sighandlers` table and `float_exception_map` are both
    // file-static — so its data list is deliberately allowed to be empty.
    assert!(
        cpp_code.len() >= 50 && cpp_data.len() >= 10 && except_code.len() >= 22,
        "C++ EH object scrape found {} + {} functions / {} + {} data — a short list would \
         leave symbols under their PLAIN names, where they collide with the subject",
        cpp_code.len(),
        except_code.len(),
        cpp_data.len(),
        except_data.len(),
    );

    // Disjointness against every other reference TU, asserted rather than assumed.
    {
        let taken: std::collections::HashSet<&str> = all_symbols
            .iter()
            .copied()
            .chain(scanf_symbols.iter().map(String::as_str))
            .chain(file_symbols.iter().map(String::as_str))
            .chain(file_data.iter().map(String::as_str))
            .chain(errno_code.iter().map(String::as_str))
            .chain(errno_data.iter().map(String::as_str))
            .chain(time_code.iter().map(String::as_str))
            .collect();
        let clash: Vec<&str> = cpp_code
            .iter()
            .chain(cpp_data.iter())
            .chain(except_code.iter())
            .chain(except_data.iter())
            .map(String::as_str)
            .filter(|s| taken.contains(s))
            .collect();
        assert!(
            clash.is_empty(),
            "cpp.c/except.c and another reference TU both define {clash:?} — one of them must \
             stop being renamed into ref_*"
        );
    }

    /// The shared body of both preludes.  One builder rather than two texts, because the two
    /// TUs differ only in WHOSE symbols get the wall first — everything else about the model
    /// (the socket set, the type substrate, the PSEH arm, the asm renaming) has to be
    /// identical or the two halves of one subsystem are graded under two different worlds.
    fn cpp_pair_prelude(own: &[&String], sibling: &[&String], other_data: &[&String]) -> String {
        let mut p = String::from(
            "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
             * Force-included at the top of the cpp.c / except.c reference TUs ONLY. */\n\
             #ifndef MSVCRT_REF_CPP_PRELUDE_H\n#define MSVCRT_REF_CPP_PRELUDE_H\n\n\
             /* ═══ stage 0: hide this TU's own shells from glibc. ══════════════════════\n\
              * Same reason as the scanf, file and errno TUs, and sharper here: glibc\n\
              * declares `signal`, `raise` and `terminate`, and several carry a __REDIRECT\n\
              * or a macro that would attach itself to the name the TU is about to DEFINE.\n\
              * Undone in stage 3, once glibc's headers have all been read. */\n",
        );
        for sym in own {
            p.push_str(&format!("#define {sym} msvcrt_ref_glibc_{sym}\n"));
        }
        p.push_str(
            "\n/* ═══ stage 1: the union prelude — sibling renames + the refstub. ════════\n\
             * malloc/free/memcpy/memmove/strcmp/strlen are string.c's, and this is what binds\n\
             * them to ref_* instead of letting the reference's exception objects run through\n\
             * the HOST libc — or, worse, through the SUBJECT's #[no_mangle] copies, which is\n\
             * the code under test. cpp.c allocates every exception's name string. */\n\
             #include \"ref_prelude.h\"\n\n\
             /* ═══ stage 2: the DERIVED constants. ═══════════════════════════════════\n\
              * Before the type substrate, which sizes a member with\n\
              * EXCEPTION_MAXIMUM_PARAMETERS. This header is also what displaces glibc's\n\
              * signal numbering with msvcrt's — see the socket list in build.rs. */\n\
             #include <msvcrt_crt_consts.h>\n\n\
             /* ═══ stage 3: un-hide this TU's own shells and rename them for real. ════ */\n",
        );
        for sym in own {
            p.push_str(&format!("#undef {sym}\n#define {sym} ref_{sym}\n"));
        }
        p.push_str(
            "\n/* ═══ stage 4: the SIBLING half of the pair. ═════════════════════════════\n\
             * cpp.c and except.c call each other — except.c's `call_catch_block` calls\n\
             * cpp.c's `terminate`, and both reach cppexcept.h's inlines. Without this the\n\
             * call escapes to the SUBJECT's copy and a reference that calls the code under\n\
             * test is not a reference. The whole wall is applied rather than the one symbol:\n\
             * the two sets are asserted disjoint above, so it costs nothing, and a future\n\
             * edit that reaches for another of the sibling's exports stays correct. */\n",
        );
        for sym in sibling.iter().chain(other_data.iter()) {
            p.push_str(&format!("#define {sym} ref_{sym}\n"));
        }
        p.push_str(
            "\n/* ═══ stage 5: the TYPES, after the rename so declarations follow it. ════\n\
             * Order is load-bearing. msvcrt_cpp_types.h re-declares the pair's OWN entry\n\
             * points (`terminate`, `_CreateFrameInfo`, `__CxxRegisterExceptionObject`, …)\n\
             * because it stands in for the real dll/win32/msvcrt/msvcrt.h, whose include\n\
             * guard the refstub claims. Included BEFORE stage 3 those declarations would keep\n\
             * the plain name while the definitions took the ref_ one — so every intra-pair\n\
             * call would link against the SUBJECT while the reference's own bodies sat\n\
             * unreferenced. That is a silent-green failure, not a build error. */\n\
             #include <msvcrt_cpp_types.h>\n\
             #include <msvcrt_cpp_layout.h>\n\n\
             /* ═══ stage 6: the wine/asm.h substitute, WITH the ref_ prefix. ══════════\n\
              * The guard is pre-claimed so refstub/wine/asm.h expands to nothing and these\n\
              * win — the same trick refstub/msvcrt.h plays on the real msvcrt.h.\n\
              *\n\
              * `__ASM_NAME` is the whole reason this block exists. cxx.h builds each vtable\n\
              * in INLINE ASSEMBLY (`__ASM_VTABLE`), and an asm string is text the `#define`\n\
              * rename wall cannot reach: the `.globl exception_vtable` and the\n\
              * `.quad exception_vector_dtor` inside it would still name the PLAIN symbols\n\
              * while the C definitions beside them became ref_*. The vtable would then be\n\
              * emitted under the subject's name — a duplicate symbol if you are lucky, and a\n\
              * reference dispatching through the code under test if you are not. Prefixing\n\
              * here renames the asm side in lockstep with the C side, in one place.\n\
              *\n\
              * The thiscall macros are the non-i386 arm of the real wine/asm.h, verbatim:\n\
              * on x86-64 `this` is simply the first argument, so the wrapper is nothing and\n\
              * THISCALL is the identity. */\n\
             #define __WINE_ASM_H\n\
             #define __ASM_NAME(name) \"ref_\" name\n\
             #define __ASM_GLOBAL_FUNC(name,code)\n\
             #define __ASM_STDCALL_FUNC(name,args,code)\n\
             #define __ASM_DEFINE_FUNC(name,code)\n\
             #define __ASM_SEH(str)\n\
             #define __ASM_CFI(str)\n\
             #define __ASM_BLOCK_BEGIN(name)\n\
             #define __ASM_BLOCK_END\n\
             #define DEFINE_THISCALL_WRAPPER(func,args) /* nothing off i386 */\n\
             #define THISCALL(func) func\n\
             /* NOT `__ASM_NAME(#func)`, which is what wine/asm.h says and what produced\n\
              * `ref_ref_exception_what` in the five vtables.  The two spellings sit either\n\
              * side of the ONE place a rename wall behaves differently: `#` suppresses macro\n\
              * expansion of its OWN operand and nothing else.  cxx.h reaches the vtable slot\n\
              * name through `VTABLE_ADD_FUNC(name) -> THISCALL_NAME(name)`, so `name` is an\n\
              * ordinary argument at the outer layer and the wall has already rewritten it to\n\
              * `ref_exception_what` by the time `#` sees it — prefixing again doubles it.  The\n\
              * `__ASM_NAME(#name ...)` uses in the same macro DO need the prefix, because\n\
              * there `#name` is direct and the wall never ran.  Same macro, two jobs; splitting\n\
              * them here is the fix, and the linker is what found it. */\n\
             #define THISCALL_NAME(func) #func\n\n\
             /* ═══ stage 7: PSEH, the REAL header. ═══════════════════════════════════\n\
              * Pulled here rather than left to wine/exception.h, which spells __TRY in terms\n\
              * of _SEH2_TRY without including the header that defines it (the DLL build gets\n\
              * it from its own precompiled include set). `_SEH2_Volatile0` is an extern the\n\
              * substrate defines, exactly as sdk/lib/pseh/dummy.c does for the DLL. */\n\
             #include <pseh/pseh2.h>\n\n\
             /* ═══ stage 8: _exit, redirected to the harness's terminator. ════════════\n\
              * except.c's Ctrl-C handler calls `_exit`, which belongs to exit.c — the layer\n\
              * BELOW the code under test, exactly like fileimpl.c's Win32 surface — so\n\
              * redirecting it changes the environment, not the reference. Left plain it would\n\
              * bind to exit.rs's #[no_mangle] copy, i.e. the reference calling the subject,\n\
              * and it would take the whole test binary down mid-suite rather than failing a\n\
              * row. process.c's prelude does the same thing for the same reason. */\n\
             #undef _exit\n\
             #define _exit ref_cpp_exit_overlay\n\
             void ref_cpp_exit_overlay(int code);\n\n\
             /* ═══ stage 9: except_x86_64.c's asm thunks, onto the harness's copies. ═══\n\
              * These four are UNDEFINED in except.c and DEFINED by the subject\n\
              * (except_x86_64.rs's #[no_mangle] copies), so left plain the reference would\n\
              * dispatch through the code under test — the one thing a reference may never do.\n\
              * They cannot be renamed by the wall above because they are not except.c's own\n\
              * exports, so they are named here explicitly. cppimpl.c defines the ref_ side;\n\
              * three of the four are recorded reaches rather than models, because each enters\n\
              * a handler on somebody else's frame and there is no such frame on this host. */\n\
             #define call_catch_handler  ref_call_catch_handler\n\
             #define call_unwind_handler ref_call_unwind_handler\n\
             #define get_exception_pc    ref_get_exception_pc\n\
             #define handle_fpieee_flt   ref_handle_fpieee_flt\n\n\
             #endif /* MSVCRT_REF_CPP_PRELUDE_H */\n",
        );
        p
    }

    let cpp_wall: Vec<&String> = cpp_code.iter().chain(cpp_data.iter()).collect();
    let except_wall: Vec<&String> = except_code.iter().chain(except_data.iter()).collect();
    let cpp_prelude_h = out.join("ref_cpp_prelude.h");
    fs::write(&cpp_prelude_h, cpp_pair_prelude(&cpp_wall, &except_code.iter().collect::<Vec<_>>(), &except_data.iter().collect::<Vec<_>>())).unwrap();
    let except_prelude_h = out.join("ref_except_prelude.h");
    fs::write(&except_prelude_h, cpp_pair_prelude(&except_wall, &cpp_code.iter().collect::<Vec<_>>(), &cpp_data.iter().collect::<Vec<_>>())).unwrap();

    // ── the reference TUs: the GENUINE string.c + wcs.c, renamed ────
    //
    // Both get the IDENTICAL flag set — the two -Werror= tripwires below are
    // the harness's best safety net, and a reference TU compiled without them
    // is a reference you cannot trust.  Hence one builder, called twice.
    // `pre` is an include dir placed BEFORE refstub/ and OUT_DIR, for the one TU family that
    // needs to SHADOW a header those two supply rather than add to them: the cpp.c/except.c
    // pair's socket directory (see reference TU #14/#15 above — refstub carries a wincon.h and
    // OUT_DIR carries an ntstatus.h/excpt.h, and the pair needs its own empty ones).  Every
    // other caller passes None and is unaffected; the flag set below stays shared, which is
    // the invariant this closure exists to keep.
    //
    // `own_prelude` REPLACES the shared `-include ref_prelude.h` rather than adding to it, and
    // that is not a tidiness choice: the cpp.c/except.c prelude has to HIDE its own exports
    // from glibc BEFORE ref_prelude.h pulls glibc's headers, and then include ref_prelude.h
    // itself at its stage 1.  Force-including the shared one first would run those stages in
    // the wrong order and silently defeat the hiding — which is a link-time collision with
    // glibc's `signal`/`raise`, not a compile error.
    let ref_tu_pre = |src: &PathBuf, pre: Option<&PathBuf>, own_prelude: Option<&PathBuf>| {
        let mut build = cc::Build::new();
        if let Some(p) = pre {
            build.include(p);
        }
        build
            .include(&refstub)
            .include(&out)
            .define("__REACTOS__", None)
            .define("_MSVCR_VER", "0")
            // -fshort-wchar: `L"..."` in a reference TU must be 16-bit. refstub/msvcrt.h
            // remaps the wchar_t TYPE with a #define, which cannot reach a wide literal —
            // so without this the host emits 4-byte elements and the TU reads them as
            // 2-byte. `L": "` becomes 3A 00 | 00 00, i.e. ":" then a NUL, and __wcserror
            // silently produced "x:msg" where msvcrt produces "x: msg". Caught by the
            // first graded row that ever reached one of these literals; string.c, wcs.c,
            // locale.c and file.c carry 23 more between them.
            .flag("-fshort-wchar")
        // NOTE: NO `.warnings(false)` here, and that is the whole point.
        //
        // `cc::Build::warnings(false)` emits `-w`, and gcc's `-w` DISARMS the two
        // -Werror= flags below — in either order, and even if the warning is
        // explicitly re-enabled afterwards.  This comment used to say "NO `-w`
        // here, on purpose" with `.warnings(false)` on the line directly above
        // it: the intent was recorded, the call defeated it, and every reference
        // TU compiled for four waves with its most important tripwire off.
        // Omitting the call entirely is what actually works — cc then emits
        // neither `-w` nor `-Wall`, so the build stays quiet and the two errors
        // below stay live.
        //
        // They are the harness's single most important safety net.  A missing
        // declaration degrades, under C's implicit-declaration rule, into a
        // *call* to an undefined int-returning function — code that still links
        // and is silently wrong.  Every known instance is that shape:
        //   * `ARRAY_SIZE(lps)` in strstr's KMP path → a bogus needle length;
        //   * `get_locinfo()` → an `int` return, truncating the locinfo
        //     pointer to 32 bits → SIGSEGV on the first deref;
        //   * and the 53 undeclared shims that motivated refstub/msvcrt_shim_decls.h,
        //     which hid behind small HANDLE values until `msvcrt_wstrdupa`
        //     returned a real malloc pointer and crashed wave 18b's first row.
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
        .flag(own_prelude.unwrap_or(&prelude_h).to_str().unwrap())
        .opt_level(2)
            .file(src);
        build.cargo_metadata(false);
        build
    };
    let ref_tu = |src: &PathBuf| ref_tu_pre(src, None, None);

    // The externals (errno / locale / thread data / NLS / heap).  Compiled in
    // its own cc::Build below so the rename prelude does NOT apply to it.
    let mut refimpl_build = cc::Build::new();
    refimpl_build
        .include(&refstub)
        .include(&out) // cloc_tables.h — the C locale's pclmap/pcumap, lifted from locale.c
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
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
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .file(manifest.join("nlsimpl.c"));
    nls_build.cargo_metadata(false);

    // The C++ EH environment: the observable sinks the pair calls out to (RaiseException and
    // the two Rtl unwind entry points), the image base its RVA arithmetic is relative to, and
    // the recorders the rows read back.
    //
    // Its OWN TU with NO rename prelude, and that is not a formatting choice: it has to name
    // both `ref_terminate` and the harness's own `terminate` in one file, so the wall must not
    // be in scope — the same rule sehimpl.c states.  It takes the socket path and the type
    // header directly instead.
    let mut cppimpl_build = cc::Build::new();
    cppimpl_build
        // NOT the socket dir, deliberately: this file needs the HOST's <signal.h> for
        // `sigaction` (see __wrap_signal at its foot), and the socket that displaces glibc's
        // signal numbering for the reference would leave it with no `struct sigaction` at all.
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .file(manifest.join("cppimpl.c"));
    cppimpl_build.cargo_metadata(false);

    // ── the perf harness's null hypothesis ───────────────────────────────
    //
    // Loop bodies with exact twins in tests/perf.rs, so that harness can measure
    // its OWN resolution instead of assuming it (see perfctl.c). Its own TU with
    // no refstub/msvcrt.h: these are self-contained loops that must not pick up
    // the rename prelude or the msvcrt_ref_* import namespace.
    //
    // `-fno-builtin` is the whole point, not hygiene: without it cc recognizes the
    // copy and scan loops and substitutes glibc's vectorized routine, and the
    // control would then describe the front-end behaviour of hand-written AVX2
    // rather than of the byte loops the suite benches. Counterpart of
    // `#![no_builtins]` on the crate under test.
    //
    // Compiled TWICE into `*_a` / `*_b` symbol sets. Same source, same compiler,
    // same flags — identical code at two addresses, whose measured ratio must be
    // 1.00x. SEPARATE cc::Build instances with separate out_dirs so they land in
    // distinct TUs: gcc's -fipa-icf folds identical functions within a TU at -O2,
    // and a folded pair would make the control vacuously perfect forever.
    let perfctl_twin = |twin: &str| {
        let mut b = cc::Build::new();
        b.flag("-Werror=implicit-function-declaration")
            .flag("-Werror=implicit-int")
            .flag("-fno-builtin")
            .define("PERF_CTL_TWIN", twin)
            .opt_level(2)
            .out_dir(out.join(format!("perfctl-{twin}")))
            .file(manifest.join("perfctl.c"));
        b.cargo_metadata(false);
        b
    };
    let perfctl_a = perfctl_twin("a");
    let perfctl_b = perfctl_twin("b");

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

    // ═══ pfflat stage 4: `vsnprintf` is msvcrt's OWN engine, not the host libc. ═══
    //
    // wcs.c CALLS plain `vsnprintf` (from vsprintf, vsprintf_s, sprintf, sprintf_s,
    // _snprintf) and under _MSVCR_VER=0 never DEFINES it, so the name is in no rename
    // list and the call leaves the TU undefined for the linker to satisfy from
    // **glibc**. The real build has no such gap: sdk/include/crt/stdio.h:719-720 is
    // `#ifndef vsnprintf / #define vsnprintf _vsnprintf`, so msvcrt's own engine is
    // what those five call. Without this, `pfflat_sprintf` & co. grade the host libc —
    // no %I64 conversions, `(nil)` for %p, 2-digit exponents — while claiming to be the
    // genuine msvcrt (§9.66, the reference that is not the code under test).
    //
    // **pfflat ONLY. The ref_ instantiation must keep glibc's, and that is not an
    // oversight.** `_vsnprintf` forwards `&valist` as a `va_list*`, an idiom coherent
    // only where a va_list IS a plain pointer. That is true on all three Windows
    // targets and true of this instantiation, which is compiled against the flat
    // `<stdarg.h>` in refstub_printf/ — but false of ref_, compiled under the host's
    // SysV register-save-area va_list, where `*valist` reinterprets the decayed pointer
    // as the tag struct and faults. Applying this to ref_ too SIGSEGVs immediately and
    // not in the printf tests: `ref_create_mbcinfo` (mbcs.c:266) formats its codepage
    // with `sprintf(buf, ".%d", cp)`, so EVERY mbcs case dies in the fixture. glibc's
    // vsnprintf is the only working narrow formatter available to that TU, and `".%d"`
    // is a format the two agree on exactly, so it stays.
    //
    // Rescan does the rest: `vsnprintf` -> `_vsnprintf` -> the transformed stage-1
    // rename -> `pfflat__vsnprintf`, so this one line needs no plumbing of its own.
    //
    // tests/printf_shells.rs keeps a %I64d canary on every row routed through this
    // chain — glibc prints `I64d` literally, so a regression is a loud row mismatch
    // rather than a quietly wrong oracle.
    let stage4 = "\n#undef vsnprintf\n#define vsnprintf _vsnprintf\n";
    let endif = "\n#endif /* MSVCRT_PFFLAT_PRELUDE_H */\n";
    assert!(
        prelude2.ends_with(endif),
        "pfflat prelude does not end with its include guard — cannot append stage 4"
    );
    prelude2 = format!("{}{stage4}{endif}", &prelude2[..prelude2.len() - endif.len()]);
    assert!(
        !prelude.contains("#define vsnprintf _vsnprintf"),
        "pfflat's stage 4 leaked into the ref_ prelude — that faults ref_create_mbcinfo. \
         ref_ takes `#define vsnprintf msvcrt_ref_vsnprintf` instead; see ref_ stage 4."
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
        // -fshort-wchar: `L"..."` in a reference TU must be 16-bit. refstub/msvcrt.h
        // remaps the wchar_t TYPE with a #define, which cannot reach a wide literal —
        // so without this the host emits 4-byte elements and the TU reads them as
        // 2-byte. `L": "` becomes 3A 00 | 00 00, i.e. ":" then a NUL, and __wcserror
        // silently produced "x:msg" where msvcrt produces "x: msg". Caught by the
        // first graded row that ever reached one of these literals; string.c, wcs.c,
        // locale.c and file.c carry 23 more between them.
        .flag("-fshort-wchar")
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

    // The scanf reference TU.  Same genuineness flags as `ref_tu` (same reasons), plus
    // refstub_printf FIRST for the flat va_list — see the three-point note above.  Its
    // own out_dir because cc keys intermediates on a hash of the source path and this one
    // is generated, not the .c itself.
    let mut scanf_build = cc::Build::new();
    scanf_build
        .out_dir(out.join("scanf"))
        .include(&refstub_printf) // flat <stdarg.h> FIRST — the whole point
        .include(&refstub)
        .include(&out) // windef.h / winbase.h / winternl.h / conio.h / the preludes
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        // -fshort-wchar: `L"..."` in a reference TU must be 16-bit. refstub/msvcrt.h
        // remaps the wchar_t TYPE with a #define, which cannot reach a wide literal —
        // so without this the host emits 4-byte elements and the TU reads them as
        // 2-byte. `L": "` becomes 3A 00 | 00 00, i.e. ":" then a NUL, and __wcserror
        // silently produced "x:msg" where msvcrt produces "x: msg". Caught by the
        // first graded row that ever reached one of these literals; string.c, wcs.c,
        // locale.c and file.c carry 23 more between them.
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-fno-builtin")
        .flag("-fno-tree-loop-distribute-patterns")
        .flag("-ffp-contract=off")
        .flag("-include")
        .flag(scanf_prelude_h.to_str().unwrap())
        .opt_level(2)
        .cargo_metadata(false)
        .file(&scanf_tu_c);

    // The file.c reference TU.  Same genuineness flags and tripwires as `ref_tu`, same
    // reasons; its own out_dir only because cc keys intermediates on a hash of the
    // source path and nothing else would collide.
    let mut file_build = cc::Build::new();
    file_build
        .out_dir(out.join("file"))
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        // -fshort-wchar: `L"..."` in a reference TU must be 16-bit. refstub/msvcrt.h
        // remaps the wchar_t TYPE with a #define, which cannot reach a wide literal —
        // so without this the host emits 4-byte elements and the TU reads them as
        // 2-byte. `L": "` becomes 3A 00 | 00 00, i.e. ":" then a NUL, and __wcserror
        // silently produced "x:msg" where msvcrt produces "x: msg". Caught by the
        // first graded row that ever reached one of these literals; string.c, wcs.c,
        // locale.c and file.c carry 23 more between them.
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-fno-builtin")
        .flag("-fno-tree-loop-distribute-patterns")
        .flag("-ffp-contract=off")
        .flag("-include")
        .flag(file_prelude_h.to_str().unwrap())
        .opt_level(2)
        .cargo_metadata(false)
        .file(&file_c);

    // file.c a SECOND time, in the `fileflat_*` namespace, under the FLAT `char*` va_list.
    // Everything about why is at the prelude closure above; what matters here is the two
    // things this build does differently:
    //
    //   * `refstub_printf` FIRST on the include path, so every `#include <stdarg.h>` in
    //     the TU resolves to the flat one — that is the whole mechanism;
    //   * its own out_dir, because cc keys intermediates on a hash of the SOURCE path and
    //     this is file.c's second compilation. Sharing would overwrite the ref_ object.
    //
    // The FOUR variadic openers (`_open`, `_wopen`, `_sopen`, `_wsopen`) are BROKEN in
    // this instantiation and that is by construction, not an oversight: the flat
    // `va_start` yields NULL, so their `_O_CREAT` arm would read a pmode through it.
    // Nothing calls them here — tests/file.rs reaches this TU's stream layer through
    // `_sopen_s` + `_fdopen`, neither of which is variadic — and pfflat has the identical
    // property for wcs.c's `sprintf` family.
    let mut fileflat_build = cc::Build::new();
    fileflat_build
        .out_dir(out.join("fileflat"))
        .include(&refstub_printf) // flat <stdarg.h> FIRST — the whole point
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        // -fshort-wchar, and the genuineness flags: same set as file_build, same reasons.
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-fno-builtin")
        .flag("-fno-tree-loop-distribute-patterns")
        .flag("-ffp-contract=off")
        .flag("-include")
        .flag(fileflat_prelude_h.to_str().unwrap())
        .opt_level(2)
        .cargo_metadata(false)
        .file(&file_c);

    // The errno.c reference TU.  Same genuineness flags and tripwires; its own out_dir
    // for the same cc-intermediate-collision reason as file.c's.
    let mut errno_build = cc::Build::new();
    errno_build
        .out_dir(out.join("errno"))
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        // -fshort-wchar: `L"..."` in a reference TU must be 16-bit. refstub/msvcrt.h
        // remaps the wchar_t TYPE with a #define, which cannot reach a wide literal —
        // so without this the host emits 4-byte elements and the TU reads them as
        // 2-byte. `L": "` becomes 3A 00 | 00 00, i.e. ":" then a NUL, and __wcserror
        // silently produced "x:msg" where msvcrt produces "x: msg". Caught by the
        // first graded row that ever reached one of these literals; string.c, wcs.c,
        // locale.c and file.c carry 23 more between them.
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-fno-builtin")
        .opt_level(2)
        .cargo_metadata(false)
        .flag("-include")
        .flag(errno_prelude_h.to_str().unwrap())
        .file(&errno_c);

    // The time.c reference TU. Same genuineness flags and tripwires; its own out_dir for
    // the same cc-intermediate-collision reason as file.c's.
    let mut time_build = cc::Build::new();
    time_build
        .out_dir(out.join("time"))
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        // -fshort-wchar: `L"..."` in a reference TU must be 16-bit. time.c carries five —
        // `_wstrdate`/`_wstrtime`'s format pictures (`L"MM'/'dd'/'yy"`, `L"HH':'mm':'ss"`)
        // and `_wasctime`'s `L"%s %s %02d %02d:%02d:%02d %d\n"`. Without the flag the host
        // emits 4-byte elements and timeimpl.c's picture parser reads them as 2-byte: the
        // very first `_wstrdate` row would see `M` followed by a NUL and format nothing.
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-fno-builtin")
        .flag("-fno-tree-loop-distribute-patterns")
        .opt_level(2)
        .cargo_metadata(false)
        .flag("-include")
        .flag(time_prelude_h.to_str().unwrap())
        .file(&time_c);

    // The process.c reference TU. Same genuineness flags and tripwires; its own out_dir
    // for the same cc-intermediate-collision reason as file.c's.
    //
    // NOT compiled against refstub_printf's flat <stdarg.h>, unlike the scanf TU: these
    // bodies START their own variadic lists, and the flat header's `va_start(ap, last)`
    // is `ap = NULL`. See the reference-TU block above for how the two va_list models
    // are kept apart instead.
    let mut process_build = cc::Build::new();
    process_build
        .out_dir(out.join("process"))
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        // -fshort-wchar: `L"..."` in a reference TU must be 16-bit. process.c carries
        // seven — the four `msvcrt_search_executable` suffixes (`L".com"`, `L".exe"`,
        // `L".bat"`, `L".cmd"`), `L"PATH"`, `L"COMSPEC"` and `L" /c "`. Without the flag
        // the host emits 4-byte elements and the very first executable search compares
        // `.` against a NUL and finds nothing, on every row, identically.
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-fno-builtin")
        .opt_level(2)
        .cargo_metadata(false)
        .flag("-include")
        .flag(process_prelude_h.to_str().unwrap())
        .file(&process_c);

    // The recording process layer, on the same terms as fileimpl.c and timeimpl.c: its
    // own TU, no rename prelude, plain Win32 names.
    let mut processimpl_build = cc::Build::new();
    processimpl_build
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        // -fshort-wchar: the ledger stores wide strings and compares COMSPEC by name.
        // Without it every recorded command line would be read back at the wrong width.
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .cargo_metadata(false)
        .file(manifest.join("processimpl.c"));

    // The POSIX-backed Win32 layer underneath it.  Its OWN TU, without the rename
    // prelude — like refimpl.c, and for the same reason: these must land under the
    // plain names file.c calls.
    let mut fileimpl_build = cc::Build::new();
    fileimpl_build
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .cargo_metadata(false)
        .file(manifest.join("fileimpl.c"));

    // The Win32 clock and time-zone layer, on the same terms as fileimpl.c: its own TU,
    // no rename prelude, plain Win32 names.
    //
    // It is in the link BEFORE time.c is a reference TU, because the SUBJECT needs it:
    // time.rs calls `GetTimeZoneInformation`, `NtQuerySystemTime`, `SystemTimeToFileTime`
    // and six more, and nothing in this harness answered any of them. That is the shape
    // of every module flip here — wiring the Rust in is what reveals which parts of the
    // OS the host does not have.
    let mut timeimpl_build = cc::Build::new();
    timeimpl_build
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        // -fshort-wchar: the zone presets carry `L"Pacific Standard Time"` and the
        // format pictures are wide literals. Without it the host emits 4-byte elements
        // and every WCHAR the subject reads is garbage — the same trap the reference
        // TUs carry this flag for.
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .cargo_metadata(false)
        .file(manifest.join("timeimpl.c"));

    // The C locale's calendar vocabulary, in a TU of its own. `-fshort-wchar` is the whole
    // reason it is not a slot in refimpl.c — see the generator above.
    let mut cloc_time_build = cc::Build::new();
    cloc_time_build
        .out_dir(out.join("cloc_time"))
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        .flag("-fshort-wchar")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .opt_level(2)
        .cargo_metadata(false)
        .file(out.join("cloc_time_data.c"));

    // ── the SUBJECT's C-variadic entry points ───────────────────────────────────
    //
    // The one piece of the shipping msvcrt that is neither Rust nor reference C.
    // file.rs declares `_open`/`_wopen`/`_wsopen` and defines none of them — they are
    // C variadics, which pinned-stable Rust cannot express — and `_wfsopen` calls
    // `_wsopen`, so every fopen row runs through this TU.  See
    // refstub/msvcrt_subject_shim.h for why the REAL shim is compiled rather than a
    // host copy of its four bodies.
    //
    // Sixteen of its twenty symbols are renamed away, and that is not tidiness.  The
    // printf shells FORWARD a va_list into file.rs's `_vfprintf_l` &c., and those Rust
    // bodies walk a `*mut u8` over 8-byte slots — the Windows-x64 va_list.  The host's
    // SysV va_list is a register-save-area struct; the two cannot consume the same
    // buffer (the same wall printf_probe hit, which is why the reference TUs get
    // refstub_printf/<stdarg.h>).  The open family is safe because it FORWARDS NOTHING:
    // it reads its own `pmode` with the host's stdarg and calls `_sopen_dispatch` with
    // concrete arguments.  So the four that are ABI-correct here stay live and the
    // sixteen that are not are emitted under dead names, where a stray call becomes a
    // link error instead of a silently mis-read argument.  Leaving them plain would
    // also displace glibc's `printf` for every C file in the harness.
    //
    // The keep-list is spelled as what to KEEP, and the split is driven by the BUILT
    // OBJECT: a shell added to the shim later is renamed away by default rather than
    // quietly becoming the harness's `printf`.
    //
    // Only for a crate that links file.rs; see `subject_stdio`.
    const SHIM_ABI_SAFE: &[&str] = &["_open", "_wopen", "_sopen", "_wsopen"];
    let shim_c = msvcrt_dir.join("rust_file_varargs_shim.c");
    // The shipping DLLs no longer compile this file: since f1658ab724d the `...` entry
    // points are msvcrt-crt-rust's va_entries.rs, varargs-rust va_entry! thunks that exist
    // only for Windows targets (they spill Windows argument registers). On this SysV host
    // the C remains the stand-in for the open family, whose bodies va_entries.rs transcribes.
    // Its symbol list therefore comes from the SOURCE: the Windows object this used to read
    // is no longer built, and a stale copy of it in an old build dir is what let a lane's
    // gate pass while master's fresh tree failed (2026-10-04).
    let shim_symbols: Vec<String> = fs::read_to_string(&shim_c)
        .unwrap_or_else(|e| panic!("{}: {e}", shim_c.display()))
        .lines()
        .filter_map(|ln| {
            let (_, rest) = ln.split_once(" WINAPIV ")?;
            let name = rest.split('(').next()?.trim();
            (!ln.starts_with(char::is_whitespace) && !name.is_empty()).then(|| name.to_string())
        })
        .collect();
    for keep in SHIM_ABI_SAFE {
        assert!(
            shim_symbols.iter().any(|s| s == keep),
            "rust_file_varargs_shim.c no longer defines `{keep}` — the subject's open \
             family moved, and this keep-list is now grading nothing"
        );
    }
    let retired: Vec<&String> =
        shim_symbols.iter().filter(|s| !SHIM_ABI_SAFE.contains(&s.as_str())).collect();
    let mut sp = String::from(
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included at the top of the SUBJECT's varargs-shim TU ONLY. */\n\
         #ifndef MSVCRT_SUBJECT_SHIM_PRELUDE_H\n#define MSVCRT_SUBJECT_SHIM_PRELUDE_H\n\n\
         /* ═══ stage 0: msvcrt's FILE and the decorations, before ANY host header. ═ */\n\
         #include <msvcrt_subject_shim_pre.h>\n\n\
         /* ═══ stage 1: hide the shells from glibc. ═══════════════════════════════\n\
          * The same two-stage wall the file.c reference TU uses, and for a sharper\n\
          * reason than tidiness.  glibc declares `fwprintf` over its own `__FILE`\n\
          * (`struct _IO_FILE`) — a type msvcrt_file_pre.h does NOT displace, because\n\
          * `__FILE` carries a separate guard from `FILE`.  A one-stage rename would\n\
          * rewrite that declaration too, and the shim's definition would then collide\n\
          * with it as a conflicting type.  So hide the name for the duration of the\n\
          * host headers, and rename it for real once they are past. */\n",
    );
    for sym in &retired {
        sp.push_str(&format!("#define {sym} msvcrt_subject_glibc_{sym}\n"));
    }
    sp.push_str(
        "\n/* ═══ stage 2: the host headers the shim would otherwise reach first. ════\n\
         * Pulled here so stage 3 can land AFTER them; the shim's own #includes then\n\
         * find every guard already claimed and expand to nothing. */\n\
         #include <stdarg.h>\n\
         #include <stdio.h>\n\
         #include <wchar.h>\n\
         #include <fcntl.h>\n\n\
         /* ═══ stage 3: retire the va_list-forwarding shells. ═════════════════════ */\n",
    );
    for sym in &retired {
        sp.push_str(&format!("#undef {sym}\n#define {sym} msvcrt_subject_unused_{sym}\n"));
    }
    sp.push_str(
        "\n/* ═══ stage 4: the subject's stdout, on the subject's own FILE array. ════ */\n\
         #include <msvcrt_subject_shim.h>\n\n\
         #endif /* MSVCRT_SUBJECT_SHIM_PRELUDE_H */\n",
    );
    let shim_prelude_h = out.join("subject_shim_prelude.h");
    fs::write(&shim_prelude_h, &sp).unwrap();

    let mut shim_build = cc::Build::new();
    shim_build
        .out_dir(out.join("shim"))
        .include(&refstub)
        .include(&out)
        .define("__REACTOS__", None)
        .define("_MSVCR_VER", "0")
        .flag("-Werror=implicit-function-declaration")
        .flag("-Werror=implicit-int")
        .flag("-U_FORTIFY_SOURCE")
        .flag("-include")
        .flag(shim_prelude_h.to_str().unwrap())
        .opt_level(2)
        .cargo_metadata(false)
        .file(&shim_c);

    // ── fileimpl.c may not call into the code under test ────────────────────────────
    //
    // fileimpl.c is the reference's ENVIRONMENT. It is compiled without the rename
    // prelude, so every libc call it makes lands on a plain name — and the crate under
    // test ships `#[no_mangle]` definitions of ordinary libc names, where a strong
    // definition beats libc. Any name it imports that file.c also exports is therefore a
    // call INTO the subject, from the layer that is supposed to be underneath it.
    //
    // It was "checked, not assumed" in a comment, and the comment was right until file.rs
    // entered the link: `MoveFileExW`'s `rename()` became msvcrt's `rename` -> `_wrename`
    // -> `MoveFileExW`, unbounded recursion, stack overflow before the first row ran. A
    // rule with no command behind it is a rule you will break, so here is the command.
    //
    // Undefined symbols only: a name fileimpl.c DEFINES (its own Win32 surface) is not an
    // escape into anything.
    if subject_stdio {
        let obj = fileimpl_build
            .compile_intermediates()
            .into_iter()
            .next()
            .expect("fileimpl.c produced no object");
        let nm = repo.join(".build-tools/llvm-mingw/bin/llvm-nm");
        let out = std::process::Command::new(&nm)
            .arg("--undefined-only")
            .arg(&obj)
            .output()
            .unwrap_or_else(|e| panic!("failed to run {}: {e}", nm.display()));
        let exported: std::collections::HashSet<&str> = file_symbols
            .iter()
            .map(String::as_str)
            .chain(file_data.iter().map(String::as_str))
            .collect();
        let escapes: Vec<String> = String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter_map(|ln| ln.split_whitespace().last())
            .filter(|s| exported.contains(*s))
            .map(str::to_string)
            .collect();
        assert!(
            escapes.is_empty(),
            "fileimpl.c imports {escapes:?}, which file.c also EXPORTS — so with file.rs \
             in the link those calls bind to the SUBJECT and the reference's environment \
             runs through the code under test.\nfix: call the same operation under a name \
             msvcrt does not have (rename -> renameat, fprintf -> dprintf), or hand-roll \
             it like ref_strlen_local."
        );
    }

    // ── the demangler's corpus, SCRAPED FROM THE TREE ──
    //
    // Every `?`-prefixed export in every `.spec` in the repo is a genuine MSVC-mangled name
    // that some real DLL publishes: constructors, destructors, operators, vtables,
    // templates, and all four calling conventions across three arches. Roughly 870 of them,
    // and they cost nothing to collect.
    //
    // The alternative — a hand-written list of interesting names — is the §9.73 trap in its
    // most ordinary form: a corpus of the cases we thought of, grading the code we wrote,
    // growing only when someone remembers to grow it. This one moves whenever a `.spec`
    // does, which is the only way a demangler corpus stays honest against a tree that keeps
    // adding C++ DLLs.
    {
        let mut names: Vec<String> = Vec::new();
        let mut specs: Vec<PathBuf> = Vec::new();
        collect_specs(&repo, &mut specs);
        specs.sort();
        for spec in &specs {
            let Ok(text) = fs::read_to_string(spec) else { continue };
            for raw in text.lines() {
                let line = raw.split('#').next().unwrap_or("");
                let mut words = line.split_whitespace();
                // `@ cdecl -arch=x ??0foo@@QAE@XZ(ptr) impl` — ordinal, directive, flags,
                // then the export name. The name is the first word that is not a flag.
                let Some(ordinal) = words.next() else { continue };
                if ordinal != "@" && ordinal.parse::<u32>().is_err() {
                    continue;
                }
                let Some(_directive) = words.next() else { continue }; // cdecl / extern / stub
                let Some(export) = words.find(|w| !w.starts_with('-')) else { continue };
                let export = export.split('(').next().unwrap_or(export);
                if export.starts_with('?') {
                    names.push(export.to_string());
                }
            }
        }
        names.sort();
        names.dedup();
        assert!(
            names.len() > 500,
            "only {} mangled names scraped from {} .spec files — the corpus has a hole, and \
             a demangler graded on a thin corpus reports green for the shapes it never saw",
            names.len(),
            specs.len()
        );
        let mut corpus = String::from(
            "// GENERATED by msvcrt-difftest/build.rs — every `?`-prefixed export in every\n\
             // .spec in the repo. Do not edit; add a DLL and it grows.\n\
             pub static CORPUS: &[&str] = &[\n",
        );
        for n in &names {
            // Rust string literals: a mangled name can contain `\` (rarely) and `\"` never,
            // but escape both rather than assume the current corpus is the permanent one.
            corpus.push_str(&format!("    {:?},\n", n));
        }
        corpus.push_str("];\n");
        fs::write(out.join("undname_corpus.rs"), corpus).unwrap();
        for spec in &specs {
            println!("cargo:rerun-if-changed={}", spec.display());
        }
    }

    // One archive, `libmsvcrt_ref.a` in OUT_DIR, holding all the objects.
    let mut objects: Vec<PathBuf> = Vec::new();
    refimpl_build.cargo_metadata(false);
    objects.extend(ref_tu(&string_c).compile_intermediates());
    objects.extend(ref_tu(&wcs_c).compile_intermediates());
    objects.extend(ref_tu(&ctype_c).compile_intermediates());
    objects.extend(ref_tu(&mbcs_c).compile_intermediates());
    objects.extend(ref_tu(&undname_c).compile_intermediates());
    objects.extend(ref_tu(&misc_c).compile_intermediates());
    // math.c carries the module's OWN preprocessor definitions
    // (dll/win32/msvcrt/CMakeLists.txt:24-33). Without them the reference compiles
    // DIFFERENT CODE from the DLL and grades it confidently — `math_error` would be a
    // symbol the object does not have, `asm_sqrt` would not route to the libm one, and
    // ARRAY_SIZE/I64_MIN would not exist at all.
    //
    // `math_error` maps straight to `ref__invoke_matherr`, folding the CMake rename and the
    // ref_ wall into one step. It has to be one step: a `-D` is consumed before the
    // `-include` prelude is read, so `-Dmath_error=_invoke_matherr` would eat the token and
    // the prelude's `#define math_error ref_math_error` would never fire — leaving the
    // reference defining a plain `_invoke_matherr` that collides with the subject's.
    let mut math_tu = ref_tu(&math_c);
    math_tu
        .define("ARRAY_SIZE", "_countof")
        .define("I64_MIN", "_I64_MIN")
        .define("I64_MAX", "_I64_MAX")
        .define("UI64_MAX", "_UI64_MAX")
        .define("x87_asin", "asin")
        .define("math_error", "ref__invoke_matherr")
        .define("asm_sqrt", "sqrt")
        .define("asm_sqrtf", "sqrtf");

    // `_scprintf`, for math.c's TU ALONE — and it has to be per-TU rather than another
    // stage-4 line in the shared prelude, which is the whole reason this file exists.
    //
    // `_gcvt_s` (math.c:2044) asks "how long would this be" with `_scprintf`. In the ref_
    // namespace that is wcs.c's own body, and it walks the chain
    // `_scprintf` -> `_vscprintf` -> `_vsnprintf_l` -> `pf_printf_a` -> `arg_clbk_valist`,
    // whose `va_arg(*valist, int)` is the flat-va_list idiom the pfflat comment above
    // describes. Under the host's SysV va_list that dereference SEGFAULTS — reproducibly,
    // on the very first `_gcvt_s` row, which is how it was found. Nothing had ever driven
    // `_gcvt_s` before, so the refstub's declaration of `_scprintf` had stood unexercised.
    //
    // Redirecting it in the SHARED prelude is not available: wcs.c is a reference TU too and
    // DEFINES `_scprintf`, so a blanket `#define` would rename its definition on top of the
    // harness's and the link would see two. A `-D` cannot do it either — the command line is
    // consumed before the prelude, so the prelude's own `#define _scprintf ref__scprintf`
    // would redefine it (see the `math_error` note above for the same ordering). A second
    // `-include`, read AFTER the prelude and only by this TU, is the one place the redirect
    // both fires and stays local.
    let math_extra_h = out.join("ref_math_extra.h");
    fs::write(
        &math_extra_h,
        "/* GENERATED by msvcrt-difftest/build.rs — do not edit.\n\
         * Force-included AFTER ref_prelude.h, for the math.c reference TU ONLY.\n\
         * See build.rs for why this cannot live in the shared prelude. */\n\
         #undef _scprintf\n\
         #define _scprintf msvcrt_ref_scprintf\n\
         int msvcrt_ref_scprintf(const char *format, ...);\n",
    )
    .unwrap();
    math_tu.flag("-include").flag(math_extra_h.to_str().unwrap());

    objects.extend(math_tu.compile_intermediates());

    // Reference TUs #14 and #15.  One helper, called twice, for the same reason `ref_tu`
    // itself is one builder: the pair has to be compiled under ONE model or the two halves of
    // a single subsystem are graded against two different worlds.
    let cpp_pair_tu = |src: &PathBuf, prelude: &PathBuf| {
        let mut tu = ref_tu_pre(src, Some(&sockets), Some(prelude));
        tu
            // The REAL wine/exception.h and pseh2.h, reached rather than stubbed — see the
            // reference-TU block above for why the socket list deliberately stops short of
            // these two.  Last on the include path, so a socket always wins a name clash.
            .include(repo.join("sdk/lib/pseh/include"))
            .include(repo.join("sdk/include"))
            // The arm of pseh2.h the amd64/arm64 DLL builds, and the one cpp.rs models.
            .define("_USE_DUMMY_PSEH", None)
            // `_WIN64`, which clang predefines for x86_64-w64-mingw32 and does NOT predefine
            // for x86_64-unknown-linux-gnu — so on the host it has to be supplied, and it is
            // the FIRST reference TU in this crate to need it.  Two things branch on it and
            // both are silent if it is missing:
            //
            //   * cxx.h's `__ASM_VTABLE` emits `.quad` slots on a 64-bit target and `.long`
            //     on a 32-bit one.  Without this the reference's five vtables were built with
            //     i386's four-byte slots on an LP64 host — every entry half a pointer wide,
            //     the rtti word before the table truncated.  The LINKER caught it here
            //     ("relocation R_X86_64_32 cannot be used against ref_type_info_rtti"), but
            //     only because a row finally referenced a vtable; nothing about the symbol
            //     SET differs, which is what the earlier fidelity check compared.
            //   * cppexcept.h's `catchblock_info` carries an extra `UINT frame` under _WIN64,
            //     so without it every catch record in the reference is four bytes short of
            //     the one the DLL walks.
            //
            // Scoped to this pair rather than to `ref_tu`: the other reference TUs are green
            // without it and widening the definition would change what they compile for no
            // reason anyone has measured.
            .define("_WIN64", None);
        tu
    };
    objects.extend(cpp_pair_tu(&cpp_c, &cpp_prelude_h).compile_intermediates());
    objects.extend(cpp_pair_tu(&except_c, &except_prelude_h).compile_intermediates());
    objects.extend(cppimpl_build.compile_intermediates());

    objects.extend(refimpl_build.compile_intermediates());
    objects.extend(subject_build.compile_intermediates());
    objects.extend(nls_build.compile_intermediates());
    objects.extend(perfctl_a.compile_intermediates());
    objects.extend(perfctl_b.compile_intermediates());
    objects.extend(so_build.compile_intermediates());
    objects.extend(pfflat_build.compile_intermediates());
    objects.extend(scanf_build.compile_intermediates());
    objects.extend(file_build.compile_intermediates());
    objects.extend(fileflat_build.compile_intermediates());
    objects.extend(errno_build.compile_intermediates());
    objects.extend(time_build.compile_intermediates());
    // process.c and its recording Win32 layer. Only for a crate that links file.rs: the
    // TU calls `_pipe`/`_wfdopen`/`fclose` and stands on file.c's fd table, exactly as
    // the subject's process.rs stands on file.rs's — see `subject_stdio`.
    if subject_stdio {
        objects.extend(process_build.compile_intermediates());
        objects.extend(processimpl_build.compile_intermediates());
    }
    objects.extend(fileimpl_build.compile_intermediates());
    objects.extend(timeimpl_build.compile_intermediates());
    objects.extend(cloc_time_build.compile_intermediates());
    if subject_stdio {
        objects.extend(shim_build.compile_intermediates());
    }

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

    // except.rs exports `#[no_mangle] signal`, and a definition in the binary beats glibc's
    // for EVERYONE — including Rust's own startup, which calls `signal(SIGPIPE, SIG_IGN)` and
    // treats SIG_ERR as fatal.  msvcrt's `signal` rejects SIGPIPE (it is not one of its seven),
    // so without this the harness dies before `main` with a message about SIGPIPE and nothing
    // near the cause.  `--wrap` redirects the REFERENCES rather than replacing the definition,
    // so the subject keeps its `signal` exactly as msvcrt.dll has it — which is what the cpp
    // suite grades — and std lands on cppimpl.c's `__wrap_signal`.  Verbatim the fix
    // msvcrt-seh-difftest already carries.
    println!("cargo:rustc-link-arg=-Wl,--wrap=signal");

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
    println!("cargo:rerun-if-changed={}", manifest.join("perfctl.c").display());
    println!("cargo:rerun-if-changed={}", mbcs_c.display());
    println!("cargo:rerun-if-changed={}", scanf_c.display());
    println!("cargo:rerun-if-changed={}", file_c.display());
    println!("cargo:rerun-if-changed={}", manifest.join("fileimpl.c").display());
    println!("cargo:rerun-if-changed={}", manifest.join("timeimpl.c").display());
    println!("cargo:rerun-if-changed={}", time_c.display());
    println!("cargo:rerun-if-changed={}", process_c.display());
    println!("cargo:rerun-if-changed={}", manifest.join("processimpl.c").display());
    // Not compiled — PARSED, for the four codepage inlines the process TU lifts.
    println!("cargo:rerun-if-changed={}", msvcrt_dir.join("msvcrt.h").display());
    println!("cargo:rerun-if-changed={}", msvcrt_dir.join("mtdll.h").display());
    // scanf.h is the TEMPLATE scanf.c instantiates 16 times — real logic, not a stub.
    println!("cargo:rerun-if-changed={}", msvcrt_dir.join("scanf.h").display());
    println!("cargo:rerun-if-changed={}", refstub.display());
    println!("cargo:rerun-if-changed={}", refstub_printf.display());
    println!(
        "cargo:rerun-if-changed={}",
        repo.join("sdk/lib/crt/printf/streamout.c").display()
    );
    println!("cargo:rerun-if-changed=build.rs");
}
