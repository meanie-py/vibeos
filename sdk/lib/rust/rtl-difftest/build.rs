//! Compile the *real* ReactOS `sdk/lib/rtl/unicode.c` for the host,
//! the Wave RTL-1 proof symbols renamed `ref_*`, so the differential
//! tests can call rtl-strings' NTAPI shim and the genuine reference
//! in one process and assert parity.
//!
//! The genuine `unicode.c` pulls the huge `<rtl.h>` chain; we isolate
//! the string logic with a minimal `refstub/` (`rtl.h`, `debug.h`,
//! `wine/unicode.h`). The NLS/heap link surface (Nls* tables,
//! Rtlp{Allocate,Free}StringMemory, Rtlp{Up,Down}caseUnicodeChar,
//! RtlUpperChar, RtlInt64ToUnicodeString, RtlpDuplicateUnicodeString)
//! is referenced by ~55 of unicode.c's other functions but NOT by
//! Wave RTL-1's 8 proof functions — so it's stubbed (unprototyped
//! `int f(){...}` defs accept any call shape) just so the whole TU
//! links. Those waves come later on the shared CP1252 model.
//! WCHAR is forced to u16 (host wchar_t is 32-bit) in refstub/rtl.h.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Scrape forward prototypes for the ReactOS-style function
/// definitions in `unicode.c` so call-before-definition no longer
/// produces "conflicting types" (the genuine ndk/rtlfuncs.h
/// prototypes aren't pulled by the minimal refstub). The style is
/// very regular: `<ret-type-line>` `NTAPI|FASTCALL` `Name(` `…)` `{`.
/// SAL tokens in the captured text are `#define`d to nothing, so the
/// emitted proto preprocesses to the exact genuine signature.
fn scrape_protos(c: &str) -> String {
    let lines: Vec<&str> = c.lines().collect();
    let mut out = String::new();
    for i in 0..lines.len() {
        let t = lines[i].trim();
        if t != "NTAPI" && t != "FASTCALL" && t != "WINAPI" {
            continue;
        }
        // return type + optional `static`: walk back over the type
        // block until a boundary line.
        let mut ret: Vec<&str> = Vec::new();
        let mut is_static = false;
        let mut j = i;
        while j > 0 {
            j -= 1;
            let p = lines[j].trim();
            if p.is_empty()
                || p.ends_with(['{', '}', ';', '/'])
                || p.starts_with(['#', '*'])
                || p.starts_with("//")
                || p.ends_with(')')
            {
                break;
            }
            if p == "static" || p.starts_with("static ") {
                is_static = true;
                break;
            }
            ret.push(p);
            // a single-token type line is the normal case; stop there.
            if !p.contains(' ') {
                break;
            }
        }
        if ret.is_empty() {
            continue;
        }
        ret.reverse();
        let ret = ret.join(" ");
        // name + params: from the line after NTAPI, accumulate until
        // the param parens balance to zero; require a `{` after.
        if i + 1 >= lines.len() {
            continue;
        }
        let mut sig = String::new();
        let mut depth: i32 = 0;
        let mut started = false;
        let mut k = i + 1;
        let mut closed_at = None;
        while k < lines.len() {
            for ch in lines[k].chars() {
                sig.push(ch);
                if ch == '(' {
                    depth += 1;
                    started = true;
                } else if ch == ')' {
                    depth -= 1;
                    if started && depth == 0 {
                        closed_at = Some(k);
                        break;
                    }
                }
            }
            if closed_at.is_some() {
                break;
            }
            sig.push(' ');
            k += 1;
        }
        if closed_at.is_none() || !started {
            continue;
        }
        // must be a DEFINITION: next non-blank token is `{`.
        let mut is_def = false;
        let mut m = closed_at.unwrap();
        let after = &lines[m][lines[m].rfind(')').map(|x| x + 1).unwrap_or(0)..];
        if after.trim_start().starts_with('{') {
            is_def = true;
        }
        while !is_def && m + 1 < lines.len() {
            m += 1;
            let s = lines[m].trim();
            if s.is_empty() {
                continue;
            }
            is_def = s.starts_with('{');
            break;
        }
        if !is_def {
            continue;
        }
        // sig is "Name( ...params... )"; the name must be an ident.
        let name_end = match sig.find('(') {
            Some(x) => x,
            None => continue,
        };
        let name = sig[..name_end].trim();
        if name.is_empty()
            || !name
                .chars()
                .all(|ch| ch.is_alphanumeric() || ch == '_')
        {
            continue;
        }
        if is_static {
            out.push_str("static ");
        }
        out.push_str(&ret);
        out.push(' ');
        out.push_str(sig.trim());
        out.push_str(";\n");
    }
    out
}

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // sdk/lib/rust/rtl-difftest → repo root is 4 up.
    let repo = manifest.ancestors().nth(4).unwrap().to_path_buf();
    let refstub = manifest.join("refstub");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let unicode_c = repo.join("sdk/lib/rtl/unicode.c");

    // ref TU: refstub <rtl.h> (types) → <stdlib.h> → Nls* storage +
    // unprototyped stubs for the unused-by-RTL-1 link surface →
    // rename the 8 proof symbols → the genuine unicode.c verbatim.
    let body = format!(
        "#include <rtl.h>\n#include <stdlib.h>\n\
         /* Nls* externs unicode.c re-declares (lines 22-30) — provide\n\
            storage so the whole TU links; the RTL-1 proof fns never\n\
            touch these. */\n\
         BOOLEAN NlsMbCodePageTag=0; BOOLEAN NlsMbOemCodePageTag=0;\n\
         USHORT NlsOemDefaultChar=0; USHORT NlsUnicodeDefaultChar=0;\n\
         PUSHORT NlsLeadByteInfo=0; PUSHORT NlsOemLeadByteInfo=0;\n\
         PUSHORT NlsUnicodeToMbOemTable=0;\n\
         PWCHAR NlsOemToUnicodeTable=0; PCHAR NlsUnicodeToOemTable=0;\n\
         /* Unprototyped stubs (`f()` accepts any call shape) for the\n\
            NLS/heap leaves the other ~55 fns reference. */\n\
         int RtlUpperChar(){{return 0;}}\n\
         int RtlpUpcaseUnicodeChar(){{return 0;}}\n\
         int RtlpDowncaseUnicodeChar(){{return 0;}}\n\
         void* RtlpAllocateStringMemory(){{return 0;}}\n\
         void RtlpFreeStringMemory(){{}}\n\
         int RtlpSafeCopyMemory(){{return 0;}}\n\
         #define RtlInitString ref_RtlInitString\n\
         #define RtlInitAnsiString ref_RtlInitAnsiString\n\
         #define RtlInitAnsiStringEx ref_RtlInitAnsiStringEx\n\
         #define RtlInitUnicodeString ref_RtlInitUnicodeString\n\
         #define RtlInitUnicodeStringEx ref_RtlInitUnicodeStringEx\n\
         #define RtlCompareString ref_RtlCompareString\n\
         #define RtlEqualString ref_RtlEqualString\n\
         #define RtlPrefixString ref_RtlPrefixString\n\
         {}\
         #include \"{}\"\n",
        scrape_protos(&fs::read_to_string(&unicode_c).unwrap()),
        unicode_c.display()
    );

    let src = out.join("ref_unicode.c");
    fs::write(&src, body).unwrap();

    let mut build = cc::Build::new();
    build
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&src);

    println!("cargo:rerun-if-changed={}", unicode_c.display());
    println!("cargo:rerun-if-changed={}", refstub.display());
    // Link the reference .o directly (build-script rustc-link-lib
    // doesn't reach integration tests; trailing link-args do — the
    // cnt-difftest precedent).
    for obj in build.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}
