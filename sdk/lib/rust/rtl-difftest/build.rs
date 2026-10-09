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
        "#include <rtl.h>\n#include <stdlib.h>\n#include <string.h>\n\
         /* Nls* externs unicode.c re-declares (lines 22-30) — provide\n\
            storage so the whole TU links; the RTL-1 proof fns never\n\
            touch these. */\n\
         BOOLEAN NlsMbCodePageTag=0; BOOLEAN NlsMbOemCodePageTag=0;\n\
         USHORT NlsOemDefaultChar='?'; USHORT NlsUnicodeDefaultChar=0;\n\
         PUSHORT NlsLeadByteInfo=0; PUSHORT NlsOemLeadByteInfo=0;\n\
         PUSHORT NlsUnicodeToMbOemTable=0;\n\
         PWCHAR NlsOemToUnicodeTable=0; PCHAR NlsUnicodeToOemTable=0;\n\
         /* Wave RTL-2 SHARED NLS MODEL — byte-identical to\n\
            cnt_rust::core_rs::mbcp::upcase (rtl-strings reuses that\n\
            same fn). The genuine RtlpUpcaseUnicodeChar walks a runtime\n\
            NLS table; modelling it identically on BOTH sides makes the\n\
            diff pin the unicode.c WRAPPER (min/sizeof/signed-char/\n\
            tiebreak/NULL-guard), not the table. */\n\
         static unsigned short model_upcase(unsigned short w){{\n\
            if(w>=0x61&&w<=0x7A)return w-0x20;\n\
            if((w>=0xE0&&w<=0xF6)||(w>=0xF8&&w<=0xFE))return w-0x20;\n\
            if(w==0xFF)return 0x0178;\n\
            if(w==0xB5)return 0x039C;\n\
            if(w==0x0161)return 0x0160;\n\
            if(w==0x0153)return 0x0152;\n\
            if(w==0x017E)return 0x017D;\n\
            return w;}}\n\
         WCHAR RtlpUpcaseUnicodeChar(WCHAR s){{return model_upcase((unsigned short)s);}}\n\
         WCHAR ref_RtlUpcaseUnicodeChar(WCHAR s){{return RtlpUpcaseUnicodeChar(s);}}\n\
         CHAR ref_RtlUpperChar(CHAR s){{unsigned char c=(unsigned char)s; if(c>=0x61&&c<=0x7A)return (CHAR)(c-0x20); return s;}}\n\
         #define RtlUpperChar ref_RtlUpperChar\n\
         /* Wave RTL-3 SHARED MODEL — the single-byte CP1252 converter\n\
            core, byte-identical to cnt_rust::core_rs::mbcp dec/enc\n\
            (rtl-strings reuses them). The DBCS branch is unreachable\n\
            (NlsMbCodePageTag=0). Same discipline as RTL-2: diff pins\n\
            the converter STRUCTURE + the unicode.c wrappers, not the\n\
            runtime NLS table values. */\n\
         static const unsigned short CP_HIGH[32]={{\n\
            0x20AC,0x0081,0x201A,0x0192,0x201E,0x2026,0x2020,0x2021,\n\
            0x02C6,0x2030,0x0160,0x2039,0x0152,0x008D,0x017D,0x008F,\n\
            0x0090,0x2018,0x2019,0x201C,0x201D,0x2022,0x2013,0x2014,\n\
            0x02DC,0x2122,0x0161,0x203A,0x0153,0x009D,0x017E,0x0178}};\n\
         static unsigned short cp_dec(unsigned char b){{\n\
            if(b>=0x80&&b<0xA0)return CP_HIGH[b-0x80]; return b;}}\n\
         static unsigned char cp_enc(unsigned short w){{\n\
            int i; if(w<0x80||(w>=0xA0&&w<=0xFF))return (unsigned char)w;\n\
            for(i=0;i<32;i++) if(CP_HIGH[i]==w) return (unsigned char)(0x80+i);\n\
            return (unsigned char)'?';}}\n\
         NTSTATUS ref_RtlMultiByteToUnicodeN(WCHAR* u,ULONG us,ULONG* rs,const char* mb,ULONG mbs){{\n\
            ULONG cap=us/2,n=(mbs<cap)?mbs:cap,i; for(i=0;i<n;i++)u[i]=cp_dec((unsigned char)mb[i]); if(rs)*rs=n*2; return 0;}}\n\
         NTSTATUS ref_RtlOemToUnicodeN(WCHAR* u,ULONG us,ULONG* rs,const char* mb,ULONG mbs){{\n\
            ULONG cap=us/2,n=(mbs<cap)?mbs:cap,i; for(i=0;i<n;i++)u[i]=cp_dec((unsigned char)mb[i]); if(rs)*rs=n*2; return 0;}}\n\
         NTSTATUS ref_RtlUnicodeToMultiByteN(char* mb,ULONG mbs,ULONG* rs,const WCHAR* u,ULONG us){{\n\
            ULONG uu=us/2,n=(mbs<uu)?mbs:uu,i; for(i=0;i<n;i++)mb[i]=(char)cp_enc(u[i]); if(rs)*rs=n; return 0;}}\n\
         NTSTATUS ref_RtlUnicodeToOemN(char* o,ULONG os,ULONG* rs,const WCHAR* u,ULONG us){{\n\
            ULONG uu=us/2,n=(os<uu)?os:uu,i; for(i=0;i<n;i++)o[i]=(char)cp_enc(u[i]); if(rs)*rs=n; return (uu>n)?0x80000005:0;}}\n\
         NTSTATUS ref_RtlUpcaseUnicodeToMultiByteN(char* mb,ULONG mbs,ULONG* rs,const WCHAR* u,ULONG us){{\n\
            ULONG uu=us/2,n=(mbs<uu)?mbs:uu,i; for(i=0;i<n;i++)mb[i]=(char)cp_enc(model_upcase(u[i])); if(rs)*rs=n; return 0;}}\n\
         NTSTATUS ref_RtlUpcaseUnicodeToOemN(char* o,ULONG os,ULONG* rs,const WCHAR* u,ULONG us){{\n\
            ULONG uu=us/2,n=(os<uu)?os:uu,i; for(i=0;i<n;i++)o[i]=(char)cp_enc(model_upcase(u[i])); if(rs)*rs=n; return 0;}}\n\
         NTSTATUS ref_RtlMultiByteToUnicodeSize(ULONG* us,const char* mb,ULONG mbs){{(void)mb; *us=mbs*2; return 0;}}\n\
         NTSTATUS ref_RtlUnicodeToMultiByteSize(ULONG* mbs,const WCHAR* u,ULONG us){{(void)u; *mbs=us/2; return 0;}}\n\
         #define RtlMultiByteToUnicodeN ref_RtlMultiByteToUnicodeN\n\
         #define RtlOemToUnicodeN ref_RtlOemToUnicodeN\n\
         #define RtlUnicodeToMultiByteN ref_RtlUnicodeToMultiByteN\n\
         #define RtlUnicodeToOemN ref_RtlUnicodeToOemN\n\
         #define RtlUpcaseUnicodeToMultiByteN ref_RtlUpcaseUnicodeToMultiByteN\n\
         #define RtlUpcaseUnicodeToOemN ref_RtlUpcaseUnicodeToOemN\n\
         #define RtlMultiByteToUnicodeSize ref_RtlMultiByteToUnicodeSize\n\
         #define RtlUnicodeToMultiByteSize ref_RtlUnicodeToMultiByteSize\n\
         /* Wave RTL-3 slice 2: genuine unicode.c ANSI/OEM/Upcase\n\
            wrappers (real code, pinning the wrapper logic around the\n\
            modelled converters). */\n\
         #define RtlAnsiCharToUnicodeChar ref_RtlAnsiCharToUnicodeChar\n\
         #define RtlAnsiStringToUnicodeString ref_RtlAnsiStringToUnicodeString\n\
         #define RtlxAnsiStringToUnicodeSize ref_RtlxAnsiStringToUnicodeSize\n\
         #define RtlUnicodeStringToAnsiString ref_RtlUnicodeStringToAnsiString\n\
         #define RtlxUnicodeStringToAnsiSize ref_RtlxUnicodeStringToAnsiSize\n\
         #define RtlxUnicodeStringToOemSize ref_RtlxUnicodeStringToOemSize\n\
         #define RtlxOemStringToUnicodeSize ref_RtlxOemStringToUnicodeSize\n\
         #define RtlOemStringToUnicodeString ref_RtlOemStringToUnicodeString\n\
         #define RtlUnicodeStringToOemString ref_RtlUnicodeStringToOemString\n\
         #define RtlOemStringToCountedUnicodeString ref_RtlOemStringToCountedUnicodeString\n\
         #define RtlUnicodeStringToCountedOemString ref_RtlUnicodeStringToCountedOemString\n\
         #define RtlUpcaseUnicodeString ref_RtlUpcaseUnicodeString\n\
         #define RtlUpcaseUnicodeStringToAnsiString ref_RtlUpcaseUnicodeStringToAnsiString\n\
         #define RtlUpcaseUnicodeStringToCountedOemString ref_RtlUpcaseUnicodeStringToCountedOemString\n\
         /* RTL-6: unicode.c's genuine RtlIsValidOemCharacter walks NULL\n\
            Nls* tables in the harness — rename it away (no in-file\n\
            callers) so the dos8dot3 ref TU can provide a CP1252 model\n\
            under the real name (Wave-E2 shared-model discipline). */\n\
         #define RtlIsValidOemCharacter ref_unused_RtlIsValidOemCharacter\n\
         #define RtlUpcaseUnicodeStringToOemString ref_RtlUpcaseUnicodeStringToOemString\n\
         /* Unprototyped stubs (`f()` accepts any call shape) for the\n\
            NLS/heap leaves the other ~50 fns reference (RTL-2 unused). */\n\
         /* Wave RTL-3b: shared symmetric downcase model (byte-identical\n\
            to cnt_rust::core_rs::mbcp::downcase). */\n\
         static unsigned short model_downcase(unsigned short w){{\n\
            if(w>=0x41&&w<=0x5A)return w+0x20;\n\
            if((w>=0xC0&&w<=0xD6)||(w>=0xD8&&w<=0xDE))return w+0x20;\n\
            if(w==0x0178)return 0xFF;\n\
            if(w==0x039C)return 0xB5;\n\
            if(w==0x0160)return 0x0161;\n\
            if(w==0x0152)return 0x0153;\n\
            if(w==0x017D)return 0x017E;\n\
            return w;}}\n\
         WCHAR RtlpDowncaseUnicodeChar(WCHAR s){{return model_downcase((unsigned short)s);}}\n\
         WCHAR ref_RtlDowncaseUnicodeChar(WCHAR s){{return RtlpDowncaseUnicodeChar(s);}}\n\
         void ref_RtlGetDefaultCodePage(USHORT* a,USHORT* o){{if(a)*a=1252; if(o)*o=1252;}}\n\
         #define RtlDowncaseUnicodeString ref_RtlDowncaseUnicodeString\n\
         /* Wave RTL-4a: integer ⇄ string + copy/append (genuine\n\
            unicode.c, no NLS deps; RtlCompareUnicodeStrings folds via\n\
            the modelled RtlpUpcaseUnicodeChar). */\n\
         #define RtlCharToInteger ref_RtlCharToInteger\n\
         #define RtlUnicodeStringToInteger ref_RtlUnicodeStringToInteger\n\
         #define RtlIntegerToUnicode ref_RtlIntegerToUnicode\n\
         #define RtlCopyString ref_RtlCopyString\n\
         #define RtlCopyUnicodeString ref_RtlCopyUnicodeString\n\
         #define RtlAppendStringToString ref_RtlAppendStringToString\n\
         #define RtlAppendUnicodeStringToString ref_RtlAppendUnicodeStringToString\n\
         #define RtlAppendUnicodeToString ref_RtlAppendUnicodeToString\n\
         #define RtlAppendAsciizToString ref_RtlAppendAsciizToString\n\
         #define RtlCompareUnicodeStrings ref_RtlCompareUnicodeStrings\n\
         /* upstream-sync 2026-09-20: RtlStringFromGUID (unicode.c:2153) now calls the\n\
            count-less _swprintf, which glibc does not have. The GUID format writes a\n\
            fixed 38 chars into a buffer RtlStringFromGUID sized for exactly that, so a\n\
            generously-bounded vswprintf is a faithful stand-in. */\n\
         #include <stdarg.h>\n\
         static int ref_unicode_swprintf(wchar_t* out, const wchar_t* fmt, ...) {{\n\
            va_list ap; va_start(ap, fmt);\n\
            int n = vswprintf(out, 0x1000, fmt, ap);\n\
            va_end(ap); return n;\n\
         }}\n\
         #define _swprintf ref_unicode_swprintf\n\
         /* Wave RTL-4b slice 1: validate/find/erase/free/GUID (genuine\n\
            unicode.c; free calls the RtlpFreeStringMemory no-op stub;\n\
            RtlpIsCharInUnicodeString is file-static, exercised via\n\
            RtlFindCharInUnicodeString). */\n\
         #define RtlValidateUnicodeString ref_RtlValidateUnicodeString\n\
         #define RtlFindCharInUnicodeString ref_RtlFindCharInUnicodeString\n\
         #define RtlEraseUnicodeString ref_RtlEraseUnicodeString\n\
         #define RtlFreeAnsiString ref_RtlFreeAnsiString\n\
         #define RtlFreeOemString ref_RtlFreeOemString\n\
         #define RtlFreeUnicodeString ref_RtlFreeUnicodeString\n\
         #define RtlGUIDFromString ref_RtlGUIDFromString\n\
         /* Wave RTL-4b slice 2: RtlIsTextUnicode (genuine unicode.c;\n\
            DBCS branch dead w/ NlsMbCodePageTag=0; strchrW from\n\
            wine/unicode.h). */\n\
         #define RtlIsTextUnicode ref_RtlIsTextUnicode\n\
         /* Wave RTL-4b slice 3: alloc-edge group (genuine unicode.c;\n\
            RtlpAllocateStringMemory NULL stub → failure with the\n\
            pre-null-check field state). RtlMultiAppendUnicodeString\n\
            Buffer is UNIMPLEMENTED→STATUS_NOT_IMPLEMENTED. */\n\
         #define RtlCreateUnicodeString ref_RtlCreateUnicodeString\n\
         #define RtlCreateUnicodeStringFromAsciiz ref_RtlCreateUnicodeStringFromAsciiz\n\
         #define RtlDuplicateUnicodeString ref_RtlDuplicateUnicodeString\n\
         #define RtlStringFromGUID ref_RtlStringFromGUID\n\
         #define RtlpEnsureBufferSize ref_RtlpEnsureBufferSize\n\
         #define RtlMultiAppendUnicodeStringBuffer ref_RtlMultiAppendUnicodeStringBuffer\n\
         /* Wave RTL-4b slice 4: computer-name (genuine unicode.c;\n\
            RtlpDidUnicodeToOemWork stays genuine, NlsOemDefaultChar='?'\n\
            matches the shared enc default). */\n\
         #define RtlEqualComputerName ref_RtlEqualComputerName\n\
         #define RtlEqualDomainName ref_RtlEqualDomainName\n\
         #define RtlDnsHostNameToComputerName ref_RtlDnsHostNameToComputerName\n\
         void* RtlpAllocateStringMemory(){{return 0;}}\n\
         void RtlpFreeStringMemory(){{}}\n\
         /* RtlpSafeCopyMemory is genuinely just an SEH-guarded memcpy;\n\
            model it as a real memcpy returning STATUS_SUCCESS (a no-op\n\
            stub would leave RtlLargeIntegerToChar's output\n\
            uninitialized — only it calls this). */\n\
         int RtlpSafeCopyMemory(void* d,const void* s,unsigned long n){{memcpy(d,s,n);return 0;}}\n\
         /* Wave RTL-4c: integer→char + *ToUnicodeString (genuine\n\
            unicode.c; RtlIntegerToChar IS in unicode.c, not cross-file\n\
            as earlier assumed). */\n\
         #define RtlIntegerToChar ref_RtlIntegerToChar\n\
         #define RtlLargeIntegerToChar ref_RtlLargeIntegerToChar\n\
         #define RtlIntegerToUnicodeString ref_RtlIntegerToUnicodeString\n\
         #define RtlInt64ToUnicodeString ref_RtlInt64ToUnicodeString\n\
         #define RtlInitString ref_RtlInitString\n\
         #define RtlInitAnsiString ref_RtlInitAnsiString\n\
         #define RtlInitAnsiStringEx ref_RtlInitAnsiStringEx\n\
         #define RtlInitUnicodeString ref_RtlInitUnicodeString\n\
         #define RtlInitUnicodeStringEx ref_RtlInitUnicodeStringEx\n\
         #define RtlCompareString ref_RtlCompareString\n\
         #define RtlEqualString ref_RtlEqualString\n\
         #define RtlPrefixString ref_RtlPrefixString\n\
         #define RtlCompareUnicodeString ref_RtlCompareUnicodeString\n\
         #define RtlEqualUnicodeString ref_RtlEqualUnicodeString\n\
         #define RtlPrefixUnicodeString ref_RtlPrefixUnicodeString\n\
         #define RtlUpperString ref_RtlUpperString\n\
         #define RtlHashUnicodeString ref_RtlHashUnicodeString\n\
         /* upstream-sync 2026-09-20: upstream MOVED RtlCompareUnicodeStrings out of\n\
            unicode.c into unicode_vista.c. The test still exercises it, so the ref TU\n\
            compiles both files -- unicode_vista.c is 64 self-contained lines whose only\n\
            outside needs (RtlUpcaseUnicodeChar, min) unicode.c already supplies. */\n\
         #define RtlCompareUnicodeStrings ref_RtlCompareUnicodeStrings\n\
         {}\
         #include \"{}\"\n\
         #include \"{}\"\n",
        scrape_protos(&fs::read_to_string(&unicode_c).unwrap()),
        unicode_c.display(),
        repo.join("sdk/lib/rtl/unicode_vista.c").display()
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
    println!("cargo:rerun-if-changed={}", repo.join("sdk/lib/rtl/unicode_vista.c").display());
    println!("cargo:rerun-if-changed={}", refstub.display());
    // Link the reference .o directly (build-script rustc-link-lib
    // doesn't reach integration tests; trailing link-args do — the
    // cnt-difftest precedent).
    for obj in build.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // Wave RTL-5: genuine sdk/lib/rtl/utf8.c — a clean, standalone TU
    // (no NLS/heap/scrape needed; the 2 fns don't call each other or
    // anything but memcpy). Renamed ref_* like unicode.c.
    let utf8_c = repo.join("sdk/lib/rtl/utf8.c");
    let utf8_body = format!(
        "#include <rtl_vista.h>\n#include <string.h>\n\
         #define RtlUnicodeToUTF8N ref_RtlUnicodeToUTF8N\n\
         #define RtlUTF8ToUnicodeN ref_RtlUTF8ToUnicodeN\n\
         #include \"{}\"\n",
        utf8_c.display()
    );
    let utf8_src = out.join("ref_utf8.c");
    fs::write(&utf8_src, utf8_body).unwrap();
    let mut ubuild = cc::Build::new();
    ubuild
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&utf8_src);
    println!("cargo:rerun-if-changed={}", utf8_c.display());
    for obj in ubuild.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // Wave RTL-6: genuine sdk/lib/rtl/dos8dot3.c — own ref TU.
    // RtlpUpcaseUnicodeChar / Nls* globals / RtlUpcaseUnicodeStringTo
    // CountedOemString resolve at link (the unicode ref TU's model
    // symbols + the diff-proven rtl-strings cabi export). Only
    // RtlIsValidOemCharacter is modelled here (CP1252, byte-identical
    // to cnt_rust::core_rs::mbcp — same discipline as RtlpUpcase*).
    let dos_c = repo.join("sdk/lib/rtl/dos8dot3.c");
    let dos_body = format!(
        "#include <rtl.h>\n#include <string.h>\n\
         /* storage lives in the unicode ref TU; declare only. */\n\
         extern BOOLEAN NlsMbOemCodePageTag;\n\
         static const unsigned short CP_HIGH[32]={{\n\
            0x20AC,0x0081,0x201A,0x0192,0x201E,0x2026,0x2020,0x2021,\n\
            0x02C6,0x2030,0x0160,0x2039,0x0152,0x008D,0x017D,0x008F,\n\
            0x0090,0x2018,0x2019,0x201C,0x201D,0x2022,0x2013,0x2014,\n\
            0x02DC,0x2122,0x0161,0x203A,0x0153,0x009D,0x017E,0x0178}};\n\
         static unsigned short cp_dec(unsigned char b){{\n\
            if(b>=0x80&&b<0xA0)return CP_HIGH[b-0x80]; return b;}}\n\
         static unsigned char cp_enc(unsigned short w){{\n\
            int i; if(w<0x80||(w>=0xA0&&w<=0xFF))return (unsigned char)w;\n\
            for(i=0;i<32;i++) if(CP_HIGH[i]==w) return (unsigned char)(0x80+i);\n\
            return (unsigned char)'?';}}\n\
         static unsigned short m_up(unsigned short w){{\n\
            if(w>=0x61&&w<=0x7A)return w-0x20;\n\
            if((w>=0xE0&&w<=0xF6)||(w>=0xF8&&w<=0xFE))return w-0x20;\n\
            if(w==0xFF)return 0x0178; if(w==0xB5)return 0x039C;\n\
            if(w==0x0161)return 0x0160; if(w==0x0153)return 0x0152;\n\
            if(w==0x017E)return 0x017D; return w;}}\n\
         BOOLEAN NTAPI ref_RtlIsValidOemCharacter(PWCHAR c){{\n\
            unsigned short u=m_up(cp_dec(cp_enc(*c)));\n\
            if(cp_enc(u)==(unsigned char)'?')return FALSE;\n\
            *c=u; return TRUE;}}\n\
         #define RtlIsValidOemCharacter ref_RtlIsValidOemCharacter\n\
         #define RtlGenerate8dot3Name ref_RtlGenerate8dot3Name\n\
         #define RtlIsNameLegalDOS8Dot3 ref_RtlIsNameLegalDOS8Dot3\n\
         #include \"{}\"\n",
        dos_c.display()
    );
    let dos_src = out.join("ref_dos8dot3.c");
    fs::write(&dos_src, dos_body).unwrap();
    let mut dbuild = cc::Build::new();
    dbuild
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&dos_src);
    println!("cargo:rerun-if-changed={}", dos_c.display());
    for obj in dbuild.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // Wave SID-1: genuine sdk/lib/rtl/sid.c — own ref TU. SID types +
    // DUMMY_PSEH-equivalent _SEH2 stubs + FIELD_OFFSET (refstub/rtl.h
    // has none of these). RtlpAllocateMemory→NULL pins the
    // RtlAllocateAndInitializeSid NO_MEMORY edge (rtl-strings is
    // no-alloc — the RtlpAllocateStringMemory precedent). swprintf /
    // RtlCreateUnicodeString are link-only stubs: SID-1 does NOT test
    // RtlConvertSidToUnicodeString (Wave SID-2 models swprintf).
    let sid_c = repo.join("sdk/lib/rtl/sid.c");
    let sid_body = format!(
        "#include <rtl.h>\n#include <string.h>\n\
         #define SID_REVISION 1\n\
         #define SID_MAX_SUB_AUTHORITIES 15\n\
         #define STATUS_INVALID_SID ((NTSTATUS)0xC0000078L)\n\
         #define ANYSIZE_ARRAY 1\n\
         #define FIELD_OFFSET(t,f) ((ULONG)(ULONG_PTR)&(((t*)0)->f))\n\
         #define RtlEqualMemory(a,b,n) (!memcmp((a),(b),(n)))\n\
         /* SAL neutralization for RtlCreateServiceSid's annotations\n\
            (refstub/rtl.h covers IN/OUT but not these). */\n\
         #define _In_\n#define _Out_\n#define _Inout_\n\
         #define _In_opt_\n#define _Out_opt_\n#define _Inout_opt_\n\
         #define _In_z_\n#define _Reserved_\n\
         #define _Out_writes_bytes_opt_(x)\n\
         typedef struct _SID_IDENTIFIER_AUTHORITY {{ UCHAR Value[6]; }}\n\
            SID_IDENTIFIER_AUTHORITY, *PSID_IDENTIFIER_AUTHORITY;\n\
         typedef struct _SID {{ UCHAR Revision; UCHAR SubAuthorityCount;\n\
            SID_IDENTIFIER_AUTHORITY IdentifierAuthority;\n\
            ULONG SubAuthority[ANYSIZE_ARRAY]; }} SID, *PISID;\n\
         typedef PVOID PSID;\n\
         typedef struct _SID_AND_ATTRIBUTES {{ PSID Sid; ULONG Attributes; }}\n\
            SID_AND_ATTRIBUTES, *PSID_AND_ATTRIBUTES;\n\
         /* DUMMY_PSEH-equivalent: body runs, _SEH2_EXCEPT is dead. */\n\
         #define _SEH2_TRY\n\
         #define _SEH2_EXCEPT(x) if(0)\n\
         #define _SEH2_FINALLY\n\
         #define _SEH2_END\n\
         #define _SEH2_YIELD(s) s\n\
         #define _SEH2_GetExceptionCode() 0\n\
         #define EXCEPTION_EXECUTE_HANDLER 1\n\
         /* RtlpAllocateMemory→NULL pins the no-alloc edge; rtl-strings\n\
            is no-alloc so AllocateBuffer=TRUE ⇒ NO_MEMORY (the\n\
            sid_RtlCreateUnicodeString FALSE stub matches). */\n\
         void* RtlpAllocateMemory(unsigned long s,unsigned long t){{(void)s;(void)t;return 0;}}\n\
         void RtlpFreeMemory(void* p,unsigned long t){{(void)p;(void)t;}}\n\
         #include <stdarg.h>\n\
         /* Wave SID-2: faithful swprintf model for the EXACT directives\n\
            sid.c uses — L\"S-1-\", L\"%lu\", L\"0x%02hx\"x6, L\"-%u\".\n\
            fmt is host 32-bit wchar_t (no -fshort-wchar); out is the\n\
            forced-u16 WCHAR. Shared model (Wave-E2): byte-identical to\n\
            core_rs::sid::convert_text — the diff pins the wrapper. */\n\
         static int sid_swprintf(WCHAR* out,const wchar_t* fmt,...){{\n\
            va_list ap; va_start(ap,fmt); int n=0; const wchar_t* p=fmt;\n\
            while(*p){{\n\
              if(*p!=L'%'){{ out[n++]=(WCHAR)*p++; continue; }}\n\
              p++; int width=0;\n\
              while(*p>=L'0'&&*p<=L'9'){{ width=width*10+(int)(*p-L'0'); p++; }}\n\
              int isx=0;\n\
              while(*p==L'l'||*p==L'h'){{ p++; }}\n\
              if(*p==L'x'){{ isx=1; }} /* else 'u' */\n\
              p++;\n\
              unsigned int v=va_arg(ap,unsigned int);\n\
              char tmp[16]; int t=0;\n\
              if(isx){{\n\
                if(v==0){{ tmp[t++]='0'; }}\n\
                while(v){{ int d=v&0xF; tmp[t++]=(char)(d<10?'0'+d:'a'+d-10); v>>=4; }}\n\
              }} else {{\n\
                if(v==0){{ tmp[t++]='0'; }}\n\
                while(v){{ tmp[t++]=(char)('0'+v%10); v/=10; }}\n\
              }}\n\
              while(t<width){{ tmp[t++]='0'; }}\n\
              while(t>0){{ out[n++]=(WCHAR)tmp[--t]; }}\n\
            }}\n\
            out[n]=0; va_end(ap); return n; }}\n\
         #define swprintf sid_swprintf\n\
         /* upstream-sync 2026-09-20: upstream's non-conforming-swprintf cleanup\n\
            renamed these call sites to _swprintf, so the model needs both names. */\n\
         #define _swprintf sid_swprintf\n\
         static BOOLEAN sid_RtlCreateUnicodeString(PUNICODE_STRING d,const WCHAR* s){{(void)d;(void)s;return 0;}}\n\
         #define RtlCreateUnicodeString sid_RtlCreateUnicodeString\n\
         #define RtlValidSid ref_RtlValidSid\n\
         #define RtlLengthRequiredSid ref_RtlLengthRequiredSid\n\
         #define RtlInitializeSid ref_RtlInitializeSid\n\
         #define RtlSubAuthoritySid ref_RtlSubAuthoritySid\n\
         #define RtlSubAuthorityCountSid ref_RtlSubAuthorityCountSid\n\
         #define RtlIdentifierAuthoritySid ref_RtlIdentifierAuthoritySid\n\
         #define RtlEqualSid ref_RtlEqualSid\n\
         #define RtlLengthSid ref_RtlLengthSid\n\
         #define RtlCopySid ref_RtlCopySid\n\
         #define RtlFreeSid ref_RtlFreeSid\n\
         #define RtlEqualPrefixSid ref_RtlEqualPrefixSid\n\
         #define RtlCopySidAndAttributesArray ref_RtlCopySidAndAttributesArray\n\
         #define RtlAllocateAndInitializeSid ref_RtlAllocateAndInitializeSid\n\
         #define RtlConvertSidToUnicodeString ref_RtlConvertSidToUnicodeString\n\
         #define RtlCreateServiceSid ref_RtlCreateServiceSid\n\
         {}\
         #include \"{}\"\n",
        scrape_protos(&fs::read_to_string(&sid_c).unwrap()),
        sid_c.display()
    );
    let sid_src = out.join("ref_sid.c");
    fs::write(&sid_src, sid_body).unwrap();
    let mut sbuild = cc::Build::new();
    sbuild
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&sid_src);
    println!("cargo:rerun-if-changed={}", sid_c.display());
    for obj in sbuild.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // Wave ACL: genuine sdk/lib/rtl/acl.c — own ref TU. acl.c does
    // `#include <../../ntoskrnl/include/internal/se.h>`; resolve that
    // to an EMPTY stub via a crafted -I dir (`<-I>/../../ntoskrnl/
    // include/internal/se.h`). All ACL/ACE/SID types + SE constants +
    // _SEH2 (DUMMY_PSEH) stubs live in the preamble; RtlValidSid/
    // RtlLengthSid/RtlCopySid resolve to the diff-proven rtl-strings
    // cabi (NOT renamed). scrape_protos: acl.c has call-before-def
    // (RtlValidAcl used by RtlpAddKnownAce, defined far below).
    let acl_c = repo.join("sdk/lib/rtl/acl.c");
    // `<-I aclinc/a/b>/../../ntoskrnl/include/internal/se.h`
    // resolves to `aclinc/ntoskrnl/include/internal/se.h`.
    let se_root = out.join("aclinc/a/b");
    fs::create_dir_all(&se_root).unwrap();
    let se_stub = out.join("aclinc/ntoskrnl/include/internal");
    fs::create_dir_all(&se_stub).unwrap();
    fs::write(se_stub.join("se.h"), "/* stub: types in ref preamble */\n").unwrap();
    let acl_body = format!(
        "#include <rtl.h>\n#include <string.h>\n\
         typedef ULONG ACCESS_MASK;\n\
         #define SID_REVISION 1\n\
         #define SID_MAX_SUB_AUTHORITIES 15\n\
         #define STATUS_INVALID_SID ((NTSTATUS)0xC0000078L)\n\
         #define STATUS_REVISION_MISMATCH ((NTSTATUS)0xC0000059L)\n\
         #define STATUS_INVALID_ACL ((NTSTATUS)0xC0000077L)\n\
         #define STATUS_ALLOTTED_SPACE_EXCEEDED ((NTSTATUS)0xC0000099L)\n\
         #define STATUS_INVALID_INFO_CLASS ((NTSTATUS)0xC0000003L)\n\
         #define ANYSIZE_ARRAY 1\n\
         #define MAXUSHORT 0xffff\n\
         #define min(a,b) (((a)<(b))?(a):(b))\n\
         #define ROUND_DOWN(n,a) (((ULONG_PTR)(n)) & ~((ULONG_PTR)(a)-1))\n\
         #define ROUND_UP(n,a) ROUND_DOWN(((ULONG_PTR)(n))+(a)-1,(a))\n\
         #define ACL_REVISION2 2\n#define ACL_REVISION3 3\n#define ACL_REVISION4 4\n\
         #define MIN_ACL_REVISION 2\n#define MAX_ACL_REVISION 4\n\
         #define ACCESS_ALLOWED_ACE_TYPE 0x0\n#define ACCESS_DENIED_ACE_TYPE 0x1\n\
         #define SYSTEM_AUDIT_ACE_TYPE 0x2\n#define ACCESS_MAX_MS_V2_ACE_TYPE 0x3\n\
         #define ACCESS_ALLOWED_COMPOUND_ACE_TYPE 0x4\n#define ACCESS_MAX_MS_V3_ACE_TYPE 0x4\n\
         #define ACCESS_MIN_MS_OBJECT_ACE_TYPE 0x5\n#define ACCESS_ALLOWED_OBJECT_ACE_TYPE 0x5\n\
         #define ACCESS_DENIED_OBJECT_ACE_TYPE 0x6\n#define SYSTEM_AUDIT_OBJECT_ACE_TYPE 0x7\n\
         #define ACCESS_MAX_MS_OBJECT_ACE_TYPE 0x8\n#define ACCESS_MAX_MS_ACE_TYPE 0x8\n\
         #define SYSTEM_AUDIT_CALLBACK_OBJECT_ACE_TYPE 0xF\n\
         #define VALID_INHERIT_FLAGS 0x1F\n#define SUCCESSFUL_ACCESS_ACE_FLAG 0x40\n\
         #define FAILED_ACCESS_ACE_FLAG 0x80\n\
         #define ACE_OBJECT_TYPE_PRESENT 0x1\n#define ACE_INHERITED_OBJECT_TYPE_PRESENT 0x2\n\
         #define RtlZeroMemory(d,n) memset((d),0,(n))\n\
         #define _SEH2_TRY\n#define _SEH2_EXCEPT(x) if(0)\n#define _SEH2_FINALLY\n\
         #define _SEH2_END\n#define _SEH2_YIELD(s) s\n#define EXCEPTION_EXECUTE_HANDLER 1\n\
         typedef struct _SID_IDENTIFIER_AUTHORITY {{ UCHAR Value[6]; }} SID_IDENTIFIER_AUTHORITY,*PSID_IDENTIFIER_AUTHORITY;\n\
         typedef struct _SID {{ UCHAR Revision; UCHAR SubAuthorityCount;\n\
            SID_IDENTIFIER_AUTHORITY IdentifierAuthority; ULONG SubAuthority[ANYSIZE_ARRAY]; }} SID,*PISID;\n\
         typedef PVOID PSID;\n\
         typedef struct _ACL {{ UCHAR AclRevision; UCHAR Sbz1; USHORT AclSize;\n\
            USHORT AceCount; USHORT Sbz2; }} ACL,*PACL;\n\
         typedef struct _ACE_HEADER {{ UCHAR AceType; UCHAR AceFlags; USHORT AceSize; }} ACE_HEADER,*PACE_HEADER;\n\
         typedef struct _ACE {{ ACE_HEADER Header; ACCESS_MASK AccessMask; }} ACE,*PACE;\n\
         typedef struct _KNOWN_ACE {{ ACE_HEADER Header; ACCESS_MASK Mask; ULONG SidStart; }} KNOWN_ACE,*PKNOWN_ACE;\n\
         typedef struct _KNOWN_OBJECT_ACE {{ ACE_HEADER Header; ACCESS_MASK Mask; ULONG Flags; ULONG SidStart; }} KNOWN_OBJECT_ACE,*PKNOWN_OBJECT_ACE;\n\
         typedef enum _ACL_INFORMATION_CLASS {{ AclRevisionInformation=1, AclSizeInformation }} ACL_INFORMATION_CLASS;\n\
         typedef struct _ACL_REVISION_INFORMATION {{ ULONG AclRevision; }} ACL_REVISION_INFORMATION,*PACL_REVISION_INFORMATION;\n\
         typedef struct _ACL_SIZE_INFORMATION {{ ULONG AceCount; ULONG AclBytesInUse; ULONG AclBytesFree; }} ACL_SIZE_INFORMATION,*PACL_SIZE_INFORMATION;\n\
         /* RtlValidSid/RtlLengthSid/RtlCopySid are NOT renamed →\n\
            resolve to the diff-proven rtl-strings cabi exports. */\n\
         #define RtlFirstFreeAce ref_RtlFirstFreeAce\n\
         #define RtlpAddData ref_RtlpAddData\n\
         #define RtlpDeleteData ref_RtlpDeleteData\n\
         #define RtlpAddKnownAce ref_RtlpAddKnownAce\n\
         #define RtlpAddKnownObjectAce ref_RtlpAddKnownObjectAce\n\
         #define RtlAddAccessAllowedAce ref_RtlAddAccessAllowedAce\n\
         #define RtlAddAccessAllowedAceEx ref_RtlAddAccessAllowedAceEx\n\
         #define RtlAddAccessAllowedObjectAce ref_RtlAddAccessAllowedObjectAce\n\
         #define RtlAddAccessDeniedAce ref_RtlAddAccessDeniedAce\n\
         #define RtlAddAccessDeniedAceEx ref_RtlAddAccessDeniedAceEx\n\
         #define RtlAddAccessDeniedObjectAce ref_RtlAddAccessDeniedObjectAce\n\
         #define RtlAddAuditAccessAce ref_RtlAddAuditAccessAce\n\
         #define RtlAddAuditAccessAceEx ref_RtlAddAuditAccessAceEx\n\
         #define RtlAddAuditAccessObjectAce ref_RtlAddAuditAccessObjectAce\n\
         #define RtlGetAce ref_RtlGetAce\n\
         #define RtlAddAce ref_RtlAddAce\n\
         #define RtlDeleteAce ref_RtlDeleteAce\n\
         #define RtlCreateAcl ref_RtlCreateAcl\n\
         #define RtlQueryInformationAcl ref_RtlQueryInformationAcl\n\
         #define RtlSetInformationAcl ref_RtlSetInformationAcl\n\
         #define RtlValidAcl ref_RtlValidAcl\n\
         {}\
         #include \"{}\"\n",
        scrape_protos(&fs::read_to_string(&acl_c).unwrap()),
        acl_c.display()
    );
    let acl_src = out.join("ref_acl.c");
    fs::write(&acl_src, acl_body).unwrap();
    let mut abuild = cc::Build::new();
    abuild
        .include(&refstub)
        .include(&se_root)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&acl_src);
    println!("cargo:rerun-if-changed={}", acl_c.display());
    for obj in abuild.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // Wave SD: genuine sdk/lib/rtl/sd.c — own ref TU. Same se.h include
    // trick as acl.c (reuses the aclinc stub already created above).
    // SECURITY_DESCRIPTOR (absolute) bears pointers, so the host x86_64
    // layout has 4 bytes of header pad between Control and Owner just
    // like the kernel amd64/arm64 — sizeof = 40 here.
    // RtlValidSid/RtlValidAcl/RtlLengthSid are NOT renamed → they
    // resolve to the diff-proven rtl-strings cabi exports.
    // RtlAllocateHeap → NULL stub pins the RtlCopySecurityDescriptor
    // NO_MEMORY edge (same no-alloc discipline as SID/ACL waves).
    let sd_c = repo.join("sdk/lib/rtl/sd.c");
    let sd_body = format!(
        "#include <rtl.h>\n#include <string.h>\n\
         typedef ULONG ACCESS_MASK;\n\
         typedef USHORT SECURITY_DESCRIPTOR_CONTROL, *PSECURITY_DESCRIPTOR_CONTROL;\n\
         typedef ULONG SECURITY_INFORMATION;\n\
         #define ANYSIZE_ARRAY 1\n\
         #define FIELD_OFFSET(t,f) ((ULONG)(ULONG_PTR)&(((t*)0)->f))\n\
         #define ALIGN_DOWN(v,t) ((ULONG)(v) & ~(sizeof(t)-1))\n\
         #define ROUND_DOWN(n,a) (((ULONG_PTR)(n)) & ~((ULONG_PTR)(a)-1))\n\
         #define ROUND_UP(n,a) ROUND_DOWN(((ULONG_PTR)(n))+(a)-1,(a))\n\
         #define PAGED_CODE_RTL() ((void)0)\n\
         #define OWNER_SECURITY_INFORMATION 0x1\n\
         #define GROUP_SECURITY_INFORMATION 0x2\n\
         #define SECURITY_DESCRIPTOR_REVISION 1\n\
         #define SE_OWNER_DEFAULTED 0x0001\n#define SE_GROUP_DEFAULTED 0x0002\n\
         #define SE_DACL_PRESENT    0x0004\n#define SE_DACL_DEFAULTED  0x0008\n\
         #define SE_SACL_PRESENT    0x0010\n#define SE_SACL_DEFAULTED  0x0020\n\
         #define SE_DACL_UNTRUSTED  0x0040\n#define SE_SERVER_SECURITY 0x0080\n\
         #define SE_DACL_AUTO_INHERIT_REQ 0x0100\n#define SE_SACL_AUTO_INHERIT_REQ 0x0200\n\
         #define SE_DACL_AUTO_INHERITED   0x0400\n#define SE_SACL_AUTO_INHERITED   0x0800\n\
         #define SE_DACL_PROTECTED  0x1000\n#define SE_SACL_PROTECTED  0x2000\n\
         #define SE_RM_CONTROL_VALID 0x4000\n#define SE_SELF_RELATIVE   0x8000\n\
         #define STATUS_UNKNOWN_REVISION        ((NTSTATUS)0xC0000058L)\n\
         #define STATUS_INVALID_SECURITY_DESCR  ((NTSTATUS)0xC0000079L)\n\
         #define STATUS_BAD_DESCRIPTOR_FORMAT   ((NTSTATUS)0xC00000E7L)\n\
         #define STATUS_BUFFER_TOO_SMALL        ((NTSTATUS)0xC0000023L)\n\
         #define STATUS_NO_MEMORY               ((NTSTATUS)0xC0000017L)\n\
         #define STATUS_INVALID_PARAMETER_1     ((NTSTATUS)0xC00000EFL)\n\
         #define STATUS_INVALID_PARAMETER_2     ((NTSTATUS)0xC00000F0L)\n\
         #define _SEH2_TRY\n#define _SEH2_EXCEPT(x) if(0)\n#define _SEH2_FINALLY\n\
         #define _SEH2_END\n#define _SEH2_YIELD(s) s\n#define EXCEPTION_EXECUTE_HANDLER 1\n\
         #define RtlZeroMemory(d,n) memset((d),0,(n))\n\
         #define RtlCopyMemory(d,s,n) memcpy((d),(s),(n))\n\
         #define RtlMoveMemory(d,s,n) memmove((d),(s),(n))\n\
         typedef struct _SID_IDENTIFIER_AUTHORITY {{ UCHAR Value[6]; }} SID_IDENTIFIER_AUTHORITY,*PSID_IDENTIFIER_AUTHORITY;\n\
         typedef struct _SID {{ UCHAR Revision; UCHAR SubAuthorityCount;\n\
            SID_IDENTIFIER_AUTHORITY IdentifierAuthority; ULONG SubAuthority[ANYSIZE_ARRAY]; }} SID,*PISID;\n\
         typedef PVOID PSID;\n\
         typedef struct _ACL {{ UCHAR AclRevision; UCHAR Sbz1; USHORT AclSize;\n\
            USHORT AceCount; USHORT Sbz2; }} ACL,*PACL;\n\
         typedef PVOID PSECURITY_DESCRIPTOR;\n\
         typedef struct _SECURITY_DESCRIPTOR_RELATIVE {{\n\
            UCHAR Revision; UCHAR Sbz1; SECURITY_DESCRIPTOR_CONTROL Control;\n\
            ULONG Owner; ULONG Group; ULONG Sacl; ULONG Dacl;\n\
         }} SECURITY_DESCRIPTOR_RELATIVE,*PISECURITY_DESCRIPTOR_RELATIVE;\n\
         typedef struct _SECURITY_DESCRIPTOR {{\n\
            UCHAR Revision; UCHAR Sbz1; SECURITY_DESCRIPTOR_CONTROL Control;\n\
            PSID Owner; PSID Group; PACL Sacl; PACL Dacl;\n\
         }} SECURITY_DESCRIPTOR,*PISECURITY_DESCRIPTOR;\n\
         /* Heap-backed via posix_memalign (page-aligned with a vm_header_t\n\
            prefix that NtQueryVirtualMemory in the env ref TU also relies\n\
            on). Wave SEC needs working alloc for RtlpSetSecurityObject's\n\
            build path; the SD wave's NO_MEMORY edge for RtlCopySecurityDescriptor\n\
            is now covered via the in-tree Rust harness rather than via stub. */\n\
         #include <stdlib.h>\n\
         typedef struct {{ SIZE_T region_size; }} sd_vm_header_t;\n\
         PVOID RtlAllocateHeap(PVOID h, ULONG f, SIZE_T n){{\n\
            (void)h; (void)f;\n\
            SIZE_T req = (n + 0xFFF) & ~((SIZE_T)0xFFF); if (req == 0) req = 0x1000;\n\
            void* raw = NULL;\n\
            if (posix_memalign(&raw, 0x1000, req + sizeof(sd_vm_header_t)) != 0) return 0;\n\
            sd_vm_header_t* hdr = (sd_vm_header_t*)raw; hdr->region_size = req;\n\
            void* user = (void*)(hdr + 1); memset(user, 0, n);\n\
            return user;\n\
         }}\n\
         BOOLEAN RtlFreeHeap(PVOID h, ULONG f, PVOID p){{\n\
            (void)h; (void)f; if (!p) return 1;\n\
            sd_vm_header_t* hdr = ((sd_vm_header_t*)p) - 1; free(hdr); return 1;\n\
         }}\n\
         PVOID RtlGetProcessHeap(void){{ return (PVOID)0x12345678; }}\n\
         /* SepGet*FromDescriptor — FORCEINLINE in the real internal/se.h\n\
            (which the empty stub bypasses). Verbatim ports here so the\n\
            ref TU links and behaves byte-identically to the kernel. */\n\
         static inline PSID SepGetOwnerFromDescriptor(PVOID _d){{\n\
            PISECURITY_DESCRIPTOR d=(PISECURITY_DESCRIPTOR)_d;\n\
            if(d->Control&SE_SELF_RELATIVE){{\n\
              PISECURITY_DESCRIPTOR_RELATIVE r=(PISECURITY_DESCRIPTOR_RELATIVE)d;\n\
              if(!r->Owner)return 0; return (PSID)((ULONG_PTR)d+r->Owner);\n\
            }} return d->Owner; }}\n\
         static inline PSID SepGetGroupFromDescriptor(PVOID _d){{\n\
            PISECURITY_DESCRIPTOR d=(PISECURITY_DESCRIPTOR)_d;\n\
            if(d->Control&SE_SELF_RELATIVE){{\n\
              PISECURITY_DESCRIPTOR_RELATIVE r=(PISECURITY_DESCRIPTOR_RELATIVE)d;\n\
              if(!r->Group)return 0; return (PSID)((ULONG_PTR)d+r->Group);\n\
            }} return d->Group; }}\n\
         static inline PACL SepGetDaclFromDescriptor(PVOID _d){{\n\
            PISECURITY_DESCRIPTOR d=(PISECURITY_DESCRIPTOR)_d;\n\
            if(!(d->Control&SE_DACL_PRESENT))return 0;\n\
            if(d->Control&SE_SELF_RELATIVE){{\n\
              PISECURITY_DESCRIPTOR_RELATIVE r=(PISECURITY_DESCRIPTOR_RELATIVE)d;\n\
              if(!r->Dacl)return 0; return (PACL)((ULONG_PTR)d+r->Dacl);\n\
            }} return d->Dacl; }}\n\
         static inline PACL SepGetSaclFromDescriptor(PVOID _d){{\n\
            PISECURITY_DESCRIPTOR d=(PISECURITY_DESCRIPTOR)_d;\n\
            if(!(d->Control&SE_SACL_PRESENT))return 0;\n\
            if(d->Control&SE_SELF_RELATIVE){{\n\
              PISECURITY_DESCRIPTOR_RELATIVE r=(PISECURITY_DESCRIPTOR_RELATIVE)d;\n\
              if(!r->Sacl)return 0; return (PACL)((ULONG_PTR)d+r->Sacl);\n\
            }} return d->Sacl; }}\n\
         /* RtlValidSid/RtlValidAcl/RtlLengthSid stay UN-renamed →\n\
            resolve at link to the diff-proven rtl-strings cabi. */\n\
         #define RtlCreateSecurityDescriptor ref_RtlCreateSecurityDescriptor\n\
         #define RtlCreateSecurityDescriptorRelative ref_RtlCreateSecurityDescriptorRelative\n\
         #define RtlLengthSecurityDescriptor ref_RtlLengthSecurityDescriptor\n\
         #define RtlGetDaclSecurityDescriptor ref_RtlGetDaclSecurityDescriptor\n\
         #define RtlGetSaclSecurityDescriptor ref_RtlGetSaclSecurityDescriptor\n\
         #define RtlGetOwnerSecurityDescriptor ref_RtlGetOwnerSecurityDescriptor\n\
         #define RtlGetGroupSecurityDescriptor ref_RtlGetGroupSecurityDescriptor\n\
         #define RtlSetDaclSecurityDescriptor ref_RtlSetDaclSecurityDescriptor\n\
         #define RtlSetSaclSecurityDescriptor ref_RtlSetSaclSecurityDescriptor\n\
         #define RtlSetOwnerSecurityDescriptor ref_RtlSetOwnerSecurityDescriptor\n\
         #define RtlSetGroupSecurityDescriptor ref_RtlSetGroupSecurityDescriptor\n\
         #define RtlGetControlSecurityDescriptor ref_RtlGetControlSecurityDescriptor\n\
         #define RtlSetControlSecurityDescriptor ref_RtlSetControlSecurityDescriptor\n\
         #define RtlGetSecurityDescriptorRMControl ref_RtlGetSecurityDescriptorRMControl\n\
         #define RtlSetSecurityDescriptorRMControl ref_RtlSetSecurityDescriptorRMControl\n\
         #define RtlSetAttributesSecurityDescriptor ref_RtlSetAttributesSecurityDescriptor\n\
         #define RtlValidSecurityDescriptor ref_RtlValidSecurityDescriptor\n\
         #define RtlValidRelativeSecurityDescriptor ref_RtlValidRelativeSecurityDescriptor\n\
         #define RtlCopySecurityDescriptor ref_RtlCopySecurityDescriptor\n\
         #define RtlMakeSelfRelativeSD ref_RtlMakeSelfRelativeSD\n\
         #define RtlAbsoluteToSelfRelativeSD ref_RtlAbsoluteToSelfRelativeSD\n\
         #define RtlSelfRelativeToAbsoluteSD ref_RtlSelfRelativeToAbsoluteSD\n\
         #define RtlSelfRelativeToAbsoluteSD2 ref_RtlSelfRelativeToAbsoluteSD2\n\
         {}\
         #include \"{}\"\n",
        scrape_protos(&fs::read_to_string(&sd_c).unwrap()),
        sd_c.display()
    );
    let sd_src = out.join("ref_sd.c");
    fs::write(&sd_src, sd_body).unwrap();
    let mut dbuild2 = cc::Build::new();
    dbuild2
        .include(&refstub)
        .include(&se_root)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&sd_src);
    println!("cargo:rerun-if-changed={}", sd_c.display());
    for obj in dbuild2.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // Wave ENV-1: genuine sdk/lib/rtl/env.c — own ref TU.
    // env.c has 7 fns; only the 2 PURE string-walk fns are byte-diff
    // tested (RtlQueryEnvironmentVariable_U + RtlExpandEnvironmentStrings_U).
    // The other 5 hit PEB / Nt*VirtualMemory; we stub those so the TU
    // *links* but the tests never drive the syscall path (we always
    // pass a non-NULL Environment). RtlCompareUnicodeString /
    // RtlEqualUnicodeString stay UN-renamed → resolve at link to the
    // diff-proven rtl-strings cabi exports.
    let env_c = repo.join("sdk/lib/rtl/env.c");
    let env_body = format!(
        "#include <rtl.h>\n#include <string.h>\n\
         #define PAGE_SIZE 0x1000\n\
         #define MEM_RESERVE 0x2000\n#define MEM_COMMIT 0x1000\n\
         #define MEM_RELEASE 0x8000\n#define PAGE_READWRITE 0x04\n\
         #define ROUND_UP(n,a) ((((ULONG_PTR)(n))+(a)-1) & ~((ULONG_PTR)(a)-1))\n\
         #define min(a,b) ((a)<(b)?(a):(b))\n\
         #define STATUS_VARIABLE_NOT_FOUND      ((NTSTATUS)0xC0000100L)\n\
         #define STATUS_NOT_IMPLEMENTED         ((NTSTATUS)0xC0000002L)\n\
         #define STATUS_BUFFER_TOO_SMALL        ((NTSTATUS)0xC0000023L)\n\
         #define STATUS_INVALID_PARAMETER       ((NTSTATUS)0xC000000DL)\n\
         /* upstream-sync 2026-09-20: upstream added overflow guards to env.c\n\
            (ResultLength > UNICODE_STRING_MAX_CHARS -> STATUS_UNSUCCESSFUL) in both\n\
            RtlExpandEnvironmentStrings_U and RtlQueryEnvironmentVariable_U. Real\n\
            fixes, and the ref TU compiles the genuine C, so the harness needs the\n\
            two names. UNICODE_STRING_MAX_CHARS is ntdef.h's 32767. */\n\
         #define STATUS_UNSUCCESSFUL            ((NTSTATUS)0xC0000001L)\n\
         #define UNICODE_STRING_MAX_CHARS       (32767)\n\
         #define UNIMPLEMENTED ((void)0)\n\
         #define MemoryBasicInformation 0\n\
         typedef PVOID HANDLE;\n\
         typedef SIZE_T* PSIZE_T;\n\
         /* u16-friendly wcschr (host wchar_t is 32-bit). */\n\
         static __inline WCHAR* refenv_wcschr(const WCHAR* s, WCHAR c){{\n\
            while(*s){{ if(*s==c) return (WCHAR*)s; s++; }} return 0; }}\n\
         #define wcschr refenv_wcschr\n\
         /* upstream-sync 2026-09-20: env.c's variable lookup now uses _wcsnicmp.\n\
            NOT glibc's wcsncasecmp: host wchar_t is 32-bit and WCHAR here is 16-bit,\n\
            the exact reason refenv_wcschr above exists. Mapping it at wcsncasecmp made\n\
            every name comparison mismatch, so the ORACLE returned VARIABLE_NOT_FOUND\n\
            for every query and its output was not evidence about anything. */\n\
         static __inline int refenv_wcsnicmp(const WCHAR* a, const WCHAR* b, size_t n){{\n\
            for(; n; --n, ++a, ++b){{\n\
               WCHAR ca=*a, cb=*b;\n\
               if(ca>=L'a'&&ca<=L'z') ca=(WCHAR)(ca-32);\n\
               if(cb>=L'a'&&cb<=L'z') cb=(WCHAR)(cb-32);\n\
               if(ca!=cb) return ca<cb?-1:1;\n\
               if(!ca) return 0;\n\
            }}\n\
            return 0; }}\n\
         #define _wcsnicmp refenv_wcsnicmp\n\
         /* rtl-strings now exports the raw-buffer cores upstream added (they are in\n\
            ntdll.spec at -version=0x600+), so the ref TU must rename them too or they\n\
            collide with the genuine C at link. */\n\
         #define RtlQueryEnvironmentVariable ref_RtlQueryEnvironmentVariable\n\
         #define RtlExpandEnvironmentStrings ref_RtlExpandEnvironmentStrings\n\
         typedef struct _MEMORY_BASIC_INFORMATION {{\n\
            PVOID BaseAddress; PVOID AllocationBase; ULONG AllocationProtect;\n\
            SIZE_T RegionSize; ULONG State; ULONG Protect; ULONG Type;\n\
         }} MEMORY_BASIC_INFORMATION,*PMEMORY_BASIC_INFORMATION;\n\
         /* Tiny PEB / ProcessParameters layout — just enough for the C\n\
            to compile. The 5 syscall-bearing fns are never exercised. */\n\
         typedef struct _ENV_RPP {{ PWSTR Environment; }} ENV_RPP,*PENV_RPP;\n\
         typedef struct _ENV_PEB {{ PENV_RPP ProcessParameters; }} ENV_PEB,*PENV_PEB;\n\
         static ENV_RPP g_rpp = {{ 0 }};\n\
         static ENV_PEB g_peb = {{ &g_rpp }};\n\
         #define PEB ENV_PEB\n#define PPEB PENV_PEB\n\
         #define NtCurrentProcess() ((HANDLE)0)\n\
         #define NtCurrentPeb() (&g_peb)\n\
         PPEB RtlGetCurrentPeb(void){{ return &g_peb; }}\n\
         VOID NTAPI RtlAcquirePebLock(void){{ }}\n\
         VOID NTAPI RtlReleasePebLock(void){{ }}\n\
         /* PEB-env helper called by the rtl-strings cabi shim — host\n\
            never exercises the NULL-Env path, just needs the link. */\n\
         PWSTR NTAPI RtlpGetPebEnvironment(void){{ return 0; }}\n\
         VOID NTAPI RtlpSetPebEnvironment(PWSTR e){{ (void)e; }}\n\
         /* Heap-backed Nt*VirtualMemory stubs: enough for the env\n\
            Set-side fuzzer to exercise reallocation paths. Each\n\
            allocation gets a vm_header_t prefix that NtQuery reads\n\
            to report the region size faithfully. The cabi-Rust shim\n\
            and the C ref TU both link to these — they're the unified\n\
            VM oracle. PAGE_SIZE-rounded so ROUND_UP semantics match\n\
            the genuine RtlSetEnvironmentVariable path. */\n\
         #include <stdlib.h>\n\
         #include <stdio.h>\n\
         typedef struct {{ SIZE_T region_size; }} vm_header_t;\n\
         static SIZE_T vm_round_up(SIZE_T s) {{\n\
            return (s + 0xFFF) & ~((SIZE_T)0xFFF);\n\
         }}\n\
         NTSTATUS NTAPI NtQueryVirtualMemory(HANDLE p, PVOID base, int cls,\n\
                                              PVOID info, SIZE_T ilen, PSIZE_T ret) {{\n\
            (void)p; (void)cls; (void)ilen; if (ret) *ret = 0;\n\
            vm_header_t* h = ((vm_header_t*)base) - 1;\n\
            MEMORY_BASIC_INFORMATION* mbi = (MEMORY_BASIC_INFORMATION*)info;\n\
            mbi->BaseAddress = base; mbi->AllocationBase = base;\n\
            mbi->AllocationProtect = 4; mbi->RegionSize = h->region_size;\n\
            mbi->State = 0x1000; mbi->Protect = 4; mbi->Type = 0x20000;\n\
            return 0;\n\
         }}\n\
         NTSTATUS NTAPI NtAllocateVirtualMemory(HANDLE p, PVOID* base, ULONG_PTR z,\n\
                                                  PSIZE_T sz, ULONG type, ULONG prot) {{\n\
            (void)p; (void)z; (void)type; (void)prot;\n\
            SIZE_T req = vm_round_up(*sz); if (req == 0) req = 0x1000;\n\
            void* raw = NULL;\n\
            if (posix_memalign(&raw, 0x1000, req + sizeof(vm_header_t)) != 0) {{\n\
               return (NTSTATUS)0xC0000017L;\n\
            }}\n\
            vm_header_t* h = (vm_header_t*)raw; h->region_size = req;\n\
            *base = (PVOID)(h + 1); *sz = req;\n\
            memset(*base, 0, req);\n\
            return 0;\n\
         }}\n\
         NTSTATUS NTAPI NtFreeVirtualMemory(HANDLE p, PVOID* base, PSIZE_T sz, ULONG t) {{\n\
            (void)p; (void)t; if (!base || !*base) return (NTSTATUS)0xC0000005L;\n\
            vm_header_t* h = ((vm_header_t*)(*base)) - 1;\n\
            free(h); *base = NULL; if (sz) *sz = 0; return 0;\n\
         }}\n\
         /* Workaround for an x86_64 SysV ABI quirk: Rust `extern\n\
            \"system\" fn -> u8` doesn't zero-extend EAX, and gcc at\n\
            the BOOLEAN call sites emits `test %eax,%eax` against the\n\
            full register — garbage upper bits then falsely signal\n\
            TRUE and the walk mis-matches the first var. We wrap the\n\
            call so the `(unsigned int)result & 1u` cast forces a\n\
            `movzbl` between the cabi call and the env.c test. The\n\
            real kernel build (Microsoft x64 / i386 stdcall) isn't\n\
            affected; this is a host-difftest-only worry. */\n\
         static __inline BOOLEAN refenv_zext_eq(PCUNICODE_STRING a, PCUNICODE_STRING b, BOOLEAN ci) {{\n\
            extern BOOLEAN NTAPI RtlEqualUnicodeString(PCUNICODE_STRING,PCUNICODE_STRING,BOOLEAN);\n\
            return (BOOLEAN)((unsigned int)RtlEqualUnicodeString(a,b,ci) & 1u);\n\
         }}\n\
         static __inline LONG refenv_zext_cmp(PCUNICODE_STRING a, PCUNICODE_STRING b, BOOLEAN ci) {{\n\
            extern LONG NTAPI RtlCompareUnicodeString(PCUNICODE_STRING,PCUNICODE_STRING,BOOLEAN);\n\
            return RtlCompareUnicodeString(a,b,ci); /* LONG return is fine */\n\
         }}\n\
         #define RtlEqualUnicodeString refenv_zext_eq\n\
         #define RtlCompareUnicodeString refenv_zext_cmp\n\
         /* Rename all 7 env.c exports to ref_*. The 2 we test live\n\
            entirely in their string-walk path — no PEB / Nt* touched. */\n\
         #define RtlSetEnvironmentStrings ref_RtlSetEnvironmentStrings\n\
         #define RtlCreateEnvironment ref_RtlCreateEnvironment\n\
         #define RtlDestroyEnvironment ref_RtlDestroyEnvironment\n\
         #define RtlExpandEnvironmentStrings_U ref_RtlExpandEnvironmentStrings_U\n\
         #define RtlSetCurrentEnvironment ref_RtlSetCurrentEnvironment\n\
         #define RtlSetEnvironmentVariable ref_RtlSetEnvironmentVariable\n\
         #define RtlQueryEnvironmentVariable_U ref_RtlQueryEnvironmentVariable_U\n\
         {}\
         #include \"{}\"\n",
        scrape_protos(&fs::read_to_string(&env_c).unwrap()),
        env_c.display()
    );
    let env_src = out.join("ref_env.c");
    fs::write(&env_src, env_body).unwrap();
    let mut dbuild3 = cc::Build::new();
    dbuild3
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&env_src);
    println!("cargo:rerun-if-changed={}", env_c.display());
    for obj in dbuild3.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // Wave SEC: genuine sdk/lib/rtl/security.c — own ref TU. Reuses the
    // SD-wave aclinc/se_root stub trick + the env-wave Nt* token stubs.
    // The byte-diff tests exercise only RtlpSetSecurityObject and
    // RtlQuerySecurityObject — both are pure SD bytes-in/bytes-out
    // (no syscalls); the rest of security.c links via no-op stubs.
    // Rtl{Allocate,Free}Heap stay heap-backed (same vm_header_t trick
    // as env stubs, so realloc paths work end-to-end).
    let sec_c = repo.join("sdk/lib/rtl/security.c");
    let sec_body = format!(
        "#include <rtl.h>\n#include <string.h>\n\
         typedef ULONG ACCESS_MASK;\n\
         typedef USHORT SECURITY_DESCRIPTOR_CONTROL, *PSECURITY_DESCRIPTOR_CONTROL;\n\
         typedef ULONG SECURITY_INFORMATION;\n\
         #define ANYSIZE_ARRAY 1\n\
         #define FIELD_OFFSET(t,f) ((ULONG)(ULONG_PTR)&(((t*)0)->f))\n\
         #define RTL_FIELD_SIZE(t,f) sizeof(((t*)0)->f)\n\
         #define ROUND_UP(n,a) (((ULONG_PTR)(n)+(a)-1) & ~((ULONG_PTR)(a)-1))\n\
         #define ASSERT(x) ((void)0)\n\
         #define UNIMPLEMENTED ((void)0)\n\
         #define PAGED_CODE_RTL() ((void)0)\n\
         #define OWNER_SECURITY_INFORMATION 0x1\n\
         #define GROUP_SECURITY_INFORMATION 0x2\n\
         #define DACL_SECURITY_INFORMATION 0x4\n\
         #define SACL_SECURITY_INFORMATION 0x8\n\
         #define SECURITY_DESCRIPTOR_REVISION 1\n\
         #define SE_DACL_PRESENT 0x0004\n#define SE_DACL_DEFAULTED 0x0008\n\
         #define SE_SACL_PRESENT 0x0010\n#define SE_SACL_DEFAULTED 0x0020\n\
         #define SE_SELF_RELATIVE 0x8000\n\
         #define STATUS_INVALID_OWNER         ((NTSTATUS)0xC000005AL)\n\
         #define STATUS_INVALID_PRIMARY_GROUP ((NTSTATUS)0xC000005BL)\n\
         #define STATUS_NO_TOKEN              ((NTSTATUS)0xC00000A9L)\n\
         #define STATUS_NO_MEMORY             ((NTSTATUS)0xC0000017L)\n\
         #define STATUS_NOT_IMPLEMENTED       ((NTSTATUS)0xC0000002L)\n\
         #define STATUS_BUFFER_TOO_SMALL      ((NTSTATUS)0xC0000023L)\n\
         #define TOKEN_QUERY 0x0008\n\
         #define TokenOwner 4\n#define TokenStatistics 10\n\
         #define PagedPool 1\n\
         #define ACL_REVISION2 2\n\
         #define GENERIC_ALL 0x10000000\n#define GENERIC_READ 0x80000000\n\
         #define ACCESS_SYSTEM_SECURITY 0x01000000\n\
         #define SE_SECURITY_PRIVILEGE 8\n\
         #define SE_PRIVILEGE_USED_FOR_ACCESS 0x80000000\n\
         #define SECURITY_NT_AUTHORITY {{0,0,0,0,0,5}}\n\
         #define SECURITY_WORLD_SID_AUTHORITY {{0,0,0,0,0,1}}\n\
         #define SECURITY_LOCAL_SYSTEM_RID 0x12\n\
         #define SECURITY_BUILTIN_DOMAIN_RID 0x20\n\
         #define DOMAIN_ALIAS_RID_ADMINS 0x220\n\
         #define SECURITY_ANONYMOUS_LOGON_RID 7\n#define SECURITY_WORLD_RID 0\n\
         #define _SEH2_TRY\n#define _SEH2_EXCEPT(x) if(0)\n\
         #define _SEH2_FINALLY\n#define _SEH2_END\n\
         #define RtlZeroMemory(d,n) memset((d),0,(n))\n\
         #define RtlCopyMemory(d,s,n) memcpy((d),(s),(n))\n\
         typedef PVOID HANDLE;\n\
         typedef SIZE_T* PSIZE_T;\n\
         typedef struct _SID_IDENTIFIER_AUTHORITY {{ UCHAR Value[6]; }} SID_IDENTIFIER_AUTHORITY,*PSID_IDENTIFIER_AUTHORITY;\n\
         typedef struct _SID {{ UCHAR Revision; UCHAR SubAuthorityCount;\n\
            SID_IDENTIFIER_AUTHORITY IdentifierAuthority; ULONG SubAuthority[ANYSIZE_ARRAY]; }} SID,*PISID;\n\
         typedef PVOID PSID;\n\
         typedef struct _ACL {{ UCHAR AclRevision; UCHAR Sbz1; USHORT AclSize;\n\
            USHORT AceCount; USHORT Sbz2; }} ACL,*PACL;\n\
         typedef struct _ACE {{ UCHAR AceType; UCHAR AceFlags; USHORT AceSize; }} ACE,*PACE;\n\
         typedef struct _ACCESS_ALLOWED_ACE {{ ACE Header; ACCESS_MASK Mask; ULONG SidStart; }} ACCESS_ALLOWED_ACE;\n\
         typedef PVOID PSECURITY_DESCRIPTOR;\n\
         typedef struct _SECURITY_DESCRIPTOR_RELATIVE {{\n\
            UCHAR Revision; UCHAR Sbz1; SECURITY_DESCRIPTOR_CONTROL Control;\n\
            ULONG Owner; ULONG Group; ULONG Sacl; ULONG Dacl;\n\
         }} SECURITY_DESCRIPTOR_RELATIVE,*PISECURITY_DESCRIPTOR_RELATIVE;\n\
         typedef struct _SECURITY_DESCRIPTOR {{\n\
            UCHAR Revision; UCHAR Sbz1; SECURITY_DESCRIPTOR_CONTROL Control;\n\
            PSID Owner; PSID Group; PACL Sacl; PACL Dacl;\n\
         }} SECURITY_DESCRIPTOR,*PISECURITY_DESCRIPTOR;\n\
         typedef struct _GENERIC_MAPPING {{ ACCESS_MASK r,w,x,a; }} GENERIC_MAPPING,*PGENERIC_MAPPING;\n\
         typedef struct _LUID {{ ULONG LowPart; LONG HighPart; }} LUID,*PLUID;\n\
         typedef struct _LUID_AND_ATTRIBUTES {{ LUID Luid; ULONG Attributes; }} LUID_AND_ATTRIBUTES;\n\
         typedef struct _PRIVILEGE_SET {{ ULONG PrivilegeCount; ULONG Control;\n\
            LUID_AND_ATTRIBUTES Privilege[1]; }} PRIVILEGE_SET,*PPRIVILEGE_SET;\n\
         typedef struct _TOKEN_OWNER {{ PSID Owner; }} TOKEN_OWNER,*PTOKEN_OWNER;\n\
         typedef struct _TOKEN_STATISTICS {{ LUID TokenId; LUID AuthenticationId;\n\
            LARGE_INTEGER ExpirationTime; LONG TokenType; LONG ImpersonationLevel;\n\
            ULONG DynamicCharged; ULONG DynamicAvailable; ULONG GroupCount;\n\
            ULONG PrivilegeCount; LUID ModifiedId;\n\
         }} TOKEN_STATISTICS,*PTOKEN_STATISTICS;\n\
         typedef PVOID PRTL_ACE_DATA;\n\
         typedef PVOID PRTL_SECURE_MEMORY_CACHE_CALLBACK;\n\
         typedef ACCESS_MASK* PACCESS_MASK;\n\
         /* Heap-backed via env-wave vm_header_t (already provided by ref_env.c). */\n\
         extern PVOID NTAPI RtlAllocateHeap(PVOID, ULONG, SIZE_T);\n\
         extern BOOLEAN NTAPI RtlFreeHeap(PVOID, ULONG, PVOID);\n\
         extern PVOID NTAPI RtlGetProcessHeap(void);\n\
         /* Forward-decl Rtl* used in non-renamed call sites. The cabi\n\
            exports the real impls. */\n\
         extern PULONG NTAPI RtlSubAuthoritySid(PSID, ULONG);\n\
         extern NTSTATUS NTAPI RtlInitializeSid(PSID, PSID_IDENTIFIER_AUTHORITY, UCHAR);\n\
         extern NTSTATUS NTAPI RtlCreateAcl(PACL, ULONG, ULONG);\n\
         extern NTSTATUS NTAPI RtlAddAccessAllowedAce(PACL, ULONG, ACCESS_MASK, PSID);\n\
         extern ULONG NTAPI RtlLengthSid(PSID);\n\
         extern ULONG NTAPI RtlLengthRequiredSid(ULONG);\n\
         extern NTSTATUS NTAPI RtlCreateSecurityDescriptor(PSECURITY_DESCRIPTOR, ULONG);\n\
         extern NTSTATUS NTAPI RtlCreateSecurityDescriptorRelative(PISECURITY_DESCRIPTOR_RELATIVE, ULONG);\n\
         extern NTSTATUS NTAPI RtlGetOwnerSecurityDescriptor(PSECURITY_DESCRIPTOR, PSID*, PBOOLEAN);\n\
         extern NTSTATUS NTAPI RtlGetGroupSecurityDescriptor(PSECURITY_DESCRIPTOR, PSID*, PBOOLEAN);\n\
         extern NTSTATUS NTAPI RtlGetDaclSecurityDescriptor(PSECURITY_DESCRIPTOR, PBOOLEAN, PACL*, PBOOLEAN);\n\
         extern NTSTATUS NTAPI RtlGetSaclSecurityDescriptor(PSECURITY_DESCRIPTOR, PBOOLEAN, PACL*, PBOOLEAN);\n\
         extern NTSTATUS NTAPI RtlSetOwnerSecurityDescriptor(PSECURITY_DESCRIPTOR, PSID, BOOLEAN);\n\
         extern NTSTATUS NTAPI RtlSetGroupSecurityDescriptor(PSECURITY_DESCRIPTOR, PSID, BOOLEAN);\n\
         extern NTSTATUS NTAPI RtlSetDaclSecurityDescriptor(PSECURITY_DESCRIPTOR, BOOLEAN, PACL, BOOLEAN);\n\
         extern NTSTATUS NTAPI RtlSetSaclSecurityDescriptor(PSECURITY_DESCRIPTOR, BOOLEAN, PACL, BOOLEAN);\n\
         extern NTSTATUS NTAPI RtlAbsoluteToSelfRelativeSD(PSECURITY_DESCRIPTOR, PISECURITY_DESCRIPTOR_RELATIVE, PULONG);\n\
         extern VOID NTAPI RtlMapGenericMask(PACCESS_MASK, PGENERIC_MAPPING);\n\
         extern BOOLEAN NTAPI RtlEqualLuid(PLUID, PLUID);\n\
         /* Nt* token syscalls: no-op stubs (the byte-diff tests don't\n\
            exercise NtOpenToken / NtQueryInformationToken paths). */\n\
         NTSTATUS NTAPI NtOpenThreadToken(HANDLE t, ULONG da, BOOLEAN oa, HANDLE* h) {{\n\
            (void)t;(void)da;(void)oa;*h=0; return STATUS_NO_TOKEN;\n\
         }}\n\
         NTSTATUS NTAPI NtOpenProcessToken(HANDLE p, ULONG da, HANDLE* h) {{\n\
            (void)p;(void)da;*h=0; return STATUS_NO_TOKEN;\n\
         }}\n\
         NTSTATUS NTAPI NtQueryInformationToken(HANDLE t, ULONG c, PVOID i, ULONG il, PULONG rl) {{\n\
            (void)t;(void)c;(void)i;(void)il; if(rl) *rl=0;\n\
            return STATUS_NOT_IMPLEMENTED;\n\
         }}\n\
         NTSTATUS NTAPI NtClose(HANDLE h) {{ (void)h; return 0; }}\n\
         /* rtl-strings' RtlDefaultNpAcl ASSERTs like security.c; the real\n\
            RtlAssert lives in rtl-diag, which this harness does not link. */\n\
         VOID NTAPI RtlAssert(PVOID a, PVOID f, ULONG l, PCHAR m) {{\n\
            (void)a;(void)f;(void)l;(void)m;\n\
         }}\n\
         #define NtCurrentProcess() ((HANDLE)0)\n\
         #define NtCurrentThread() ((HANDLE)0)\n\
         /* Mirror the env-wave gcc movzbl workaround for the BOOLEAN\n\
            return ABI (see env body). */\n\
         static __inline BOOLEAN refsec_zext_vsid(PSID s) {{\n\
            extern BOOLEAN NTAPI RtlValidSid(PSID);\n\
            return (BOOLEAN)((unsigned int)RtlValidSid(s) & 1u);\n\
         }}\n\
         #define RtlValidSid refsec_zext_vsid\n\
         /* Rename security.c exports to ref_*. */\n\
         #define RtlpSetSecurityObject ref_RtlpSetSecurityObject\n\
         #define RtlpNewSecurityObject ref_RtlpNewSecurityObject\n\
         #define RtlpConvertToAutoInheritSecurityObject ref_RtlpConvertToAutoInheritSecurityObject\n\
         #define RtlCreateAndSetSD ref_RtlCreateAndSetSD\n\
         #define RtlDefaultNpAcl ref_RtlDefaultNpAcl\n\
         #define RtlDeleteSecurityObject ref_RtlDeleteSecurityObject\n\
         #define RtlNewSecurityObject ref_RtlNewSecurityObject\n\
         #define RtlNewSecurityObjectEx ref_RtlNewSecurityObjectEx\n\
         #define RtlNewSecurityObjectWithMultipleInheritance ref_RtlNewSecurityObjectWithMultipleInheritance\n\
         #define RtlNewInstanceSecurityObject ref_RtlNewInstanceSecurityObject\n\
         #define RtlCreateUserSecurityObject ref_RtlCreateUserSecurityObject\n\
         #define RtlNewSecurityGrantedAccess ref_RtlNewSecurityGrantedAccess\n\
         #define RtlQuerySecurityObject ref_RtlQuerySecurityObject\n\
         #define RtlSetSecurityObject ref_RtlSetSecurityObject\n\
         #define RtlSetSecurityObjectEx ref_RtlSetSecurityObjectEx\n\
         #define RtlConvertToAutoInheritSecurityObject ref_RtlConvertToAutoInheritSecurityObject\n\
         #define RtlRegisterSecureMemoryCacheCallback ref_RtlRegisterSecureMemoryCacheCallback\n\
         #define RtlFlushSecureMemoryCache ref_RtlFlushSecureMemoryCache\n\
         {}\
         #include \"{}\"\n",
        scrape_protos(&fs::read_to_string(&sec_c).unwrap()),
        sec_c.display()
    );
    let sec_src = out.join("ref_sec.c");
    fs::write(&sec_src, sec_body).unwrap();
    let mut dbuild4 = cc::Build::new();
    dbuild4
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&sec_src);
    println!("cargo:rerun-if-changed={}", sec_c.display());
    for obj in dbuild4.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}
