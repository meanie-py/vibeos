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
}
