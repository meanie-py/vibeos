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
    // Printf-only include dir: a flat `va_list` <stdarg.h> so the
    // genuine streamout can be driven with cnt-rust's Windows-model
    // (char*) va_list on the SysV host (see the printf ref unit below).
    let refstub_printf = manifest.join("refstub_printf");
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let crt = repo.join("sdk/lib/crt");

    // (ref symbol, full host TU body). Each renames the public symbol
    // to ref_<name> and includes the genuine ReactOS logic.
    let inc = |p: PathBuf| format!("#include \"{}\"\n", p.display());
    let s = |p: &str| crt.join(p);
    // The `_FPCLASS_*` values, read out of the genuine SDK header. The tree
    // no longer carries it (sdk/include/ucrt went with the last of the
    // Microsoft UCRT), so it comes from git history at a pinned revision. If
    // the header stops carrying them, fail here rather than compile a
    // reference whose classifier silently returns 0 for everything.
    let fpclass_consts = {
        const SDK_FLOAT_H: &str = "9c9d469c15278e449b2d37d437d0ebdaf62fc7ce:sdk/include/ucrt/float.h";
        let h = SDK_FLOAT_H;
        let shown = std::process::Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(["show", SDK_FLOAT_H])
            .output()
            .unwrap_or_else(|e| panic!("cannot run git show {SDK_FLOAT_H}: {e}"));
        assert!(
            shown.status.success(),
            "git show {SDK_FLOAT_H} failed: {}",
            String::from_utf8_lossy(&shown.stderr)
        );
        let text = String::from_utf8_lossy(&shown.stdout).into_owned();
        let lines: Vec<&str> = text
            .lines()
            .filter(|l| l.trim_start().starts_with("#define _FPCLASS_"))
            .collect();
        assert!(
            lines.len() >= 10,
            "expected the 10 _FPCLASS_* defines in {}, found {} -- the header moved",
            h,
            lines.len()
        );
        format!("{}\n", lines.join("\n"))
    };

    let mut units: Vec<(&str, String)> = vec![
        ("strlen", format!("typedef char _TCHAR;\n#define _tcslen ref_strlen\n#include <stddef.h>\n{}", inc(s("string/tcslen.h")))),
        ("strncpy", format!("typedef char _TCHAR;\n#define _tcsncpy ref_strncpy\n#include <stddef.h>\n{}", inc(s("string/tcsncpy.h")))),
        ("strcmp", format!("typedef char _TCHAR;\n#define _tcscmp ref_strcmp\n{}", inc(s("string/tcscmp.h")))),
        ("strnlen", format!("typedef char _TCHAR;\n#define _tcsnlen ref_strnlen\n#include <stddef.h>\n{}", inc(s("string/tcsnlen.h")))),
        ("strncmp", format!("typedef char _TCHAR;\n#define _tcsncmp ref_strncmp\n#include <stddef.h>\n{}", inc(s("string/tcsncmp.h")))),
        ("strchr_x", format!("typedef char _TCHAR;\n#define _XINT int\n#define _tcschr ref_strchr\n{}", inc(s("string/tcschr.h")))),
        ("strrchr_x", format!("typedef char _TCHAR;\n#define _XINT int\n#define _tcsrchr ref_strrchr\n{}", inc(s("string/tcsrchr.h")))),
        // _strtoi64_l defined here too — rename so it doesn't collide
        // with cnt-rust's own #[no_mangle] _strtoi64_l shim.
        ("strtoi64", format!("#define _strtoi64 ref_strtoi64\n#define _strtoi64_l ref_strtoi64_l\n{}", inc(s("string/strtoi64.c")))),
        // strtoul.c calls _strtoi64_l (real name) — provide it from a
        // privately-renamed copy of strtoi64.c in the same TU.
        // Host `unsigned long` is 64-bit; Win-i386 is 32. Force the
        // 32-bit ULONG_MAX (the clamp constant) AFTER limits.h, and
        // truncate the return to 32 bits — else the reference diverges
        // from the LP32 contract cnt-rust models.
        ("strtoul", format!(
            // ULONG_MAX as signed-64 so `__int64 ret > ULONG_MAX` is a
            // SIGNED compare — that's how Win-i386 promotes a 32-bit
            // `unsigned long` against `__int64`. (0xFFFFFFFFUL would be
            // 64-bit unsigned on the host and wrongly clamp negatives.)
            "#define _strtoi64 ru_si\n#define _strtoi64_l ru_si_l\n{}#undef ULONG_MAX\n#define ULONG_MAX 0xFFFFFFFFLL\n#define strtoul ru_strtoul_real\n#define _strtoul_l ru_strtoul_l_real\n{}unsigned int ref_strtoul(const char*n,char**e,int b){{return (unsigned int)ru_strtoul_real(n,e,b);}}\n",
            inc(s("string/strtoi64.c")), inc(s("string/strtoul.c")))),
        ("strstr", format!("#define strstr ref_strstr\n{}", inc(s("string/strstr.c")))),
        // Wave WCS — the kernel (_LIBCNT_) slice of string/wcs.c: 8 pure
        // wide leaves. Transcribed VERBATIM from wcs.c with the _LIBCNT_
        // macro reductions (CHECK_PMT→cond, INVALID_PMT→no-op, no errno)
        // and wchar_t→unsigned short (host wchar is 32-bit). Faithful to
        // the genuine bodies; EINVAL/ERANGE/STRUNCATE/_TRUNCATE inlined.
        ("wcs", "#include <string.h>\ntypedef unsigned short WCH;\n\
            static unsigned long wl(const WCH*s){unsigned long n=0;while(s[n])n++;return n;}\n\
            WCH* ref__wcsnset(WCH*str,WCH c,unsigned long n){WCH*ret=str;while((n-->0)&&*str)*str++=c;return ret;}\n\
            WCH* ref__wcsrev(WCH*str){WCH*ret=str;WCH*end=str+wl(str)-1;while(end>str){WCH t=*end;*end--=*str;*str++=t;}return ret;}\n\
            int ref_wcscoll(const WCH*a,const WCH*b){while(*a&&(*a==*b)){a++;b++;}return (int)*a-(int)*b;}\n\
            WCH* ref_wcspbrk(const WCH*str,const WCH*acc){const WCH*p;while(*str){for(p=acc;*p;p++)if(*p==*str)return (WCH*)str;str++;}return 0;}\n\
            int ref_wcscpy_s(WCH*d,unsigned long num,const WCH*sr){unsigned long sz;if(!d||!num)return 22;d[0]=0;if(!sr)return 22;sz=wl(sr)+1;if(sz>num)return 34;memcpy(d,sr,sz*sizeof(WCH));return 0;}\n\
            int ref_wcsncpy_s(WCH*d,unsigned long num,const WCH*sr,unsigned long cnt){unsigned long sz;if(!d||!num)return 22;d[0]=0;if(!sr)return 22;sz=wl(sr)<cnt?wl(sr):cnt;if(sz>=num)return 34;memcpy(d,sr,sz*sizeof(WCH));d[sz]=0;return 0;}\n\
            int ref_wcscat_s(WCH*d,unsigned long elem,const WCH*sr){WCH*ptr=d;if(!d||elem==0)return 22;if(!sr){d[0]=0;return 22;}while(ptr<d+elem&&*ptr)ptr++;while(ptr<d+elem){if((*ptr++=*sr++)==0)return 0;}d[0]=0;return 34;}\n\
            int ref_wcsncat_s(WCH*d,unsigned long elem,const WCH*sr,unsigned long cnt){unsigned long srclen,dststart;int ret=0;if(!(d!=0)||!(elem>0))return 22;if(!(sr!=0||cnt==0))return 22;if(cnt==0)return 0;for(dststart=0;dststart<elem;dststart++){if(d[dststart]==0)break;}if(dststart==elem)return 22;if(cnt==(unsigned long)-1){srclen=wl(sr);if(srclen>=(elem-dststart)){srclen=elem-dststart-1;ret=80;}}else srclen=wl(sr)<cnt?wl(sr):cnt;if(srclen<(elem-dststart)){memcpy(&d[dststart],sr,srclen*sizeof(WCH));d[dststart+srclen]=0;return ret;}d[0]=0;return 34;}\n".to_string()),
        ("atoi64", format!("#define _atoi64 ref_atoi64\n{}", inc(s("string/atoi64.c")))),
        ("memcpy", format!("#define __cdecl\n#define memcpy ref_memcpy\n{}", inc(s("mem/memcpy.c")))),
        ("memchr", format!("#define __cdecl\n#define memchr ref_memchr\n{}", inc(s("mem/memchr.c")))),
        ("memcmp", format!("#define __cdecl\n#define memcmp ref_memcmp\n{}", inc(s("mem/memcmp.c")))),
        ("memmove", format!("#define __cdecl\n#define memmove ref_memmove\n{}", inc(s("mem/memmove.c")))),
        ("memset", format!("#define __cdecl\n#define memset ref_memset\n{}", inc(s("mem/memset.c")))),
        // Wave MEM2 — _memccpy (host <string.h> only, __cdecl) and
        // _memicmp. memicmp.c pulls <precomp.h> (refstub: CDECL, EINVAL,
        // MSVCRT_INVALID_PMT no-op, host <ctype.h> toupper == ASCII fold
        // in the C locale == mem::icmp::up). Supply NtCurrentPeb() with
        // OSMajorVersion=10 (the Windows 10/11 version the tree reports
        // after the flip) so the Vista NULL/_NLSCMPERROR guard is LIVE and
        // graded: NULL with n!=0 → _NLSCMPERROR, NULL with n==0 → 0.
        ("memccpy", format!("#define __cdecl\n#define _memccpy ref_memccpy\n{}", inc(s("mem/memccpy.c")))),
        ("memicmp", format!(
            "struct cnt_peb {{ unsigned long OSMajorVersion; }};\n\
             static struct cnt_peb cnt_peb_v = {{ 10 }};\n\
             #define NtCurrentPeb() (&cnt_peb_v)\n\
             #define _NLSCMPERROR 0x7fffffff\n\
             #define _memicmp ref_memicmp\n{}",
            inc(s("mem/memicmp.c")))),
        // Wave RAND — rand_nt.c defines rand+srand over a static u64
        // `next`. Host <stdlib.h> RAND_MAX is 0x7fffffff; clamp to the
        // Windows 0x7fff (the LP32 ULONG_MAX precedent). Both public
        // symbols renamed in one TU; clang sets __GNUC__ → the LL
        // multiplier branch (== rng::step).
        ("rand_nt", format!(
            "#define __cdecl\n#include <stdlib.h>\n\
             #undef RAND_MAX\n#define RAND_MAX 0x7fff\n\
             #define rand ref_rand\n#define srand ref_srand\n{}",
            inc(s("stdlib/rand_nt.c")))),
        // Wave SORT — qsort.c. Pulls host <stdlib.h>/<search.h> (real
        // `long`) BEFORE its own `#define long intptr_t`, so provide
        // <stdint.h> for intptr_t. `#define qsort ref_qsort` renames the
        // host decl, the definition, AND the internal recursive call.
        ("qsort", format!(
            "#define __cdecl\n#include <stdint.h>\n\
             #define qsort ref_qsort\n{}",
            inc(s("stdlib/qsort.c")))),
        // Wave SEARCH — bsearch.c + lfind.c. Both pull <stdlib.h>; lfind
        // additionally pulls <search.h>. Rename the public symbol so the
        // host decl, the definition, and any internal call all become
        // ref_<name> in this TU.
        ("bsearch", format!(
            "#define __cdecl\n#define bsearch ref_bsearch\n{}",
            inc(s("search/bsearch.c")))),
        ("lfind", format!(
            "#define __cdecl\n#define _lfind ref_lfind\n{}",
            inc(s("search/lfind.c")))),
        // atol = (long)_atoi64; skip atol.c's _atoldbl (needs _LDOUBLE).
        // Rename the helper so it doesn't collide with cnt-rust's own
        // #[no_mangle] `_atoi64` shim linked into this test binary.
        ("atol", format!("#define _atoi64 ref_atol_atoi64\n{}long ref_atol(const char*s){{return (long)ref_atol_atoi64(s);}}\n", inc(s("string/atoi64.c")))),
        // itoa.c — _i64toa/_ui64toa/_ltoa/_ultoa/_itoa (+ _s). Host
        // `long` is 64-bit (LP64) vs Win-i386 LP32 (the cnt-rust model
        // = i32/u32). `__int64`/`int` widths DO match the host, so the
        // i64/u64 (and int-input) symbols diff over the full domain;
        // the genuine `long`/`unsigned long` symbols are reached via
        // fixed-width adapters and fuzzed only over the width-agnostic
        // domain (positive + decimal-negative; the neg+radix!=10
        // unsigned-reinterpret misfeature is pinned by _i64toa where
        // host width == model width). Same spirit as the strtoul
        // ULONG_MAX LP32 note.
        ("itoa", format!(
            "#define _i64toa ref_i64toa\n#define _i64toa_s ref_i64toa_s\n\
             #define _ui64toa ref_ui64toa\n#define _ui64toa_s ref_ui64toa_s\n\
             #define _itoa ref_itoa\n#define _itoa_s ref_itoa_s\n\
             #define _ltoa ref_ltoa\n#define _ltoa_s ref_ltoa_s\n\
             #define _ultoa ref_ultoa\n\
             #include <stddef.h>\n\
             char* ref_ltoa(long,char*,int);\n\
             int ref_ltoa_s(long,char*,size_t,int);\n{}\
             char* ref_ltoa32(int v,char*s,int r){{return ref_ltoa(v,s,r);}}\n\
             char* ref_ultoa32(unsigned int v,char*s,int r){{return ref_ultoa(v,s,r);}}\n\
             int ref_ltoa_s32(int v,char*s,size_t z,int r){{return ref_ltoa_s(v,s,z,r);}}\n",
            inc(s("string/itoa.c")))),
        // itow.c — wide twin. Host wchar_t is 32-bit; force it to
        // unsigned short (Windows/cnt-rust `*mut u16`) for the whole
        // TU so sizeof(wchar_t)==2 and the buffers diff byte-for-byte.
        // <stddef.h> first (size_t + glibc's guarded wchar_t typedef),
        // THEN the macro shadows every later `wchar_t` use. Same LP32
        // long-family adapter trick as itoa.
        ("itow", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define _i64tow ref_i64tow\n#define _i64tow_s ref_i64tow_s\n\
             #define _ui64tow ref_ui64tow\n#define _ui64tow_s ref_ui64tow_s\n\
             #define _itow ref_itow\n#define _itow_s ref_itow_s\n\
             #define _ltow ref_ltow\n#define _ltow_s ref_ltow_s\n\
             #define _ultow ref_ultow\n\
             wchar_t* ref_ltow(long,wchar_t*,int);\n\
             int ref_ltow_s(long,wchar_t*,size_t,int);\n{}\
             wchar_t* ref_ltow32(int v,wchar_t*s,int r){{return ref_ltow(v,s,r);}}\n\
             wchar_t* ref_ultow32(unsigned int v,wchar_t*s,int r){{return ref_ultow(v,s,r);}}\n\
             int ref_ltow_s32(int v,wchar_t*s,size_t z,int r){{return ref_ltow_s(v,s,z,r);}}\n",
            inc(s("string/itow.c")))),
        // Wave D — case-insensitive NT variants (libcntpr links these,
        // NOT the toupper() stricmp.c). They #include <string.h> only;
        // stub the SAL annotations + __cdecl. Wide twins force
        // wchar_t→unsigned short (host is 32-bit) for u16 parity.
        ("stricmp_nt", format!(
            "#define __cdecl\n#define _Check_return_\n#define _In_z_\n\
             #define _In_\n#define _In_reads_or_z_(x)\n\
             #define _stricmp ref_stricmp\n{}",
            inc(s("string/_stricmp_nt.c")))),
        ("strnicmp_nt", format!(
            "#define __cdecl\n#define _Check_return_\n#define _In_z_\n\
             #define _In_\n#define _In_reads_or_z_(x)\n\
             #define _strnicmp ref_strnicmp\n{}",
            inc(s("string/_strnicmp_nt.c")))),
        ("wcsicmp_nt", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define __cdecl\n#define _Check_return_\n#define _In_z_\n\
             #define _In_\n#define _In_reads_or_z_(x)\n\
             #define _wcsicmp ref_wcsicmp\n{}",
            inc(s("wstring/_wcsicmp_nt.c")))),
        ("wcsnicmp_nt", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define __cdecl\n#define _Check_return_\n#define _In_z_\n\
             #define _In_\n#define _In_reads_or_z_(x)\n\
             #define _wcsnicmp ref_wcsnicmp\n{}",
            inc(s("wstring/_wcsnicmp_nt.c")))),
        // Wave E1 — wcstoul.c (BSD cutoff/cutlim). precomp FIRST (so
        // its limits.h runs, guard set), THEN clamp ULONG_MAX to the
        // 32-bit value (Windows `unsigned long` is 32-bit on every
        // arch; the LP64 host needs this + a (unsigned int) return
        // truncation, same as the strtoul ref). wchar_t→unsigned
        // short. iswctype is provided ASCII-faithful via a macro
        // (avoids clashing with any <wctype.h> prototype); the diff
        // fuzzes ASCII only — iswctype for the ASCII range is
        // standard. Rename the symbol so it doesn't collide with
        // cnt-rust's own #[no_mangle] wcstoul.
        ("wcstoul", format!(
            "#include <precomp.h>\n#undef ULONG_MAX\n\
             #define ULONG_MAX 0xFFFFFFFFLL\n\
             #define wchar_t unsigned short\n\
             #define _SPACE 1\n#define _DIGIT 2\n#define _ALPHA 4\n#define _UPPER 8\n\
             static int cnt_iswctype(int c,int t){{\
             if(c<0||c>127)return 0;\
             if(t==_SPACE)return c==' '||c=='\\t'||c=='\\n'||c=='\\v'||c=='\\f'||c=='\\r';\
             if(t==_DIGIT)return c>='0'&&c<='9';\
             if(t==_UPPER)return c>='A'&&c<='Z';\
             if(t==_ALPHA)return (c>='A'&&c<='Z')||(c>='a'&&c<='z');\
             return 0;}}\n\
             #define iswctype(c,t) cnt_iswctype((c),(t))\n\
             #define wcstoul ru_wcstoul_real\n{}\
             unsigned int ref_wcstoul(const wchar_t*n,wchar_t**e,int b)\
             {{return (unsigned int)ru_wcstoul_real(n,e,b);}}\n",
            inc(s("string/wcstoul.c")))),
        // Wave E2 — multibyte ⇄ wide. The genuine .c are thin Rtl*
        // NLS wrappers; refstub/ndk/_cp1252.h supplies a deterministic
        // single-byte CP1252 Rtl* model identical to core_rs/mbcp.rs,
        // so the diff pins the wrapper logic over 0x00-0xFF. wchar_t→
        // unsigned short (host is 32-bit). Rename the 5 symbols so
        // they don't collide with cnt-rust's own #[no_mangle].
        ("mbstowcs_nt", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define mbtowc ref_mbtowc\n#define mbstowcs ref_mbstowcs\n{}",
            inc(s("string/mbstowcs_nt.c")))),
        ("wcstombs_nt", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define wctomb ref_wctomb\n#define wcstombs ref_wcstombs\n{}",
            inc(s("string/wcstombs_nt.c")))),
        ("toupper_nt_mb", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define toupper_nt_mb ref_toupper_nt_mb\n{}",
            inc(s("string/toupper_nt_mb.c")))),
        // Wave F — charset span/search. Narrow strspn/strcspn pull
        // strxspn.h (relative #include resolves to the .c's dir);
        // strpbrk.c uses host strchr. Wide trio is precomp-based.
        ("strspn", format!("#define __cdecl\n#define strspn ref_strspn\n{}", inc(s("string/strspn.c")))),
        ("strcspn", format!("#define __cdecl\n#define strcspn ref_strcspn\n{}", inc(s("string/strcspn.c")))),
        ("strpbrk", format!("#define __cdecl\n#define strpbrk ref_strpbrk\n{}", inc(s("string/strpbrk.c")))),
        ("wcsspn", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define wcsspn ref_wcsspn\n{}", inc(s("wstring/wcsspn.c")))),
        ("wcscspn", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define wcscspn ref_wcscspn\n{}", inc(s("wstring/wcscspn.c")))),
        ("wcsstr", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define wcsstr ref_wcsstr\n{}", inc(s("wstring/wcsstr.c")))),
        // Wave G — BSD parse + endptr. strtol/wcstol: precomp FIRST
        // (limits.h runs, guard set) THEN clamp LONG_MAX/MIN to 32-bit
        // (Win long; the LP64 host needs this + an (int) return
        // truncation, exactly the strtoul precedent). strtoull:
        // unsigned long long is 64-bit on host AND target → faithful
        // directly, no trick. wcstol adds the ASCII iswctype shim.
        ("strtol", format!(
            "#include <precomp.h>\n#undef LONG_MAX\n#define LONG_MAX 0x7FFFFFFFL\n\
             #undef LONG_MIN\n#define LONG_MIN (-0x7FFFFFFF - 1)\n\
             #define strtol ru_strtol_real\n{}\
             int ref_strtol(const char*n,char**e,int b)\
             {{return (int)ru_strtol_real(n,e,b);}}\n",
            inc(s("string/strtol.c")))),
        ("strtoull", format!(
            "#include <precomp.h>\n#define strtoull ref_strtoull\n{}",
            inc(s("string/strtoull.c")))),
        ("wcstol", format!(
            "#include <precomp.h>\n#undef LONG_MAX\n#define LONG_MAX 0x7FFFFFFFL\n\
             #undef LONG_MIN\n#define LONG_MIN (-0x7FFFFFFF - 1)\n\
             #define wchar_t unsigned short\n\
             #define _SPACE 1\n#define _DIGIT 2\n#define _ALPHA 4\n#define _UPPER 8\n\
             static int cnt_iswctype(int c,int t){{\
             if(c<0||c>127)return 0;\
             if(t==_SPACE)return c==' '||c=='\\t'||c=='\\n'||c=='\\v'||c=='\\f'||c=='\\r';\
             if(t==_DIGIT)return c>='0'&&c<='9';\
             if(t==_UPPER)return c>='A'&&c<='Z';\
             if(t==_ALPHA)return (c>='A'&&c<='Z')||(c>='a'&&c<='z');\
             return 0;}}\n\
             #define iswctype(c,t) cnt_iswctype((c),(t))\n\
             #define wcstol ru_wcstol_real\n{}\
             int ref_wcstol(const wchar_t*n,wchar_t**e,int b)\
             {{return (int)ru_wcstol_real(n,e,b);}}\n",
            inc(s("string/wcstol.c")))),
        // Wave G2 — wide-atoi parse. wtoi64.c: host LONGLONG /
        // unsigned __int64 == target i64/u64 so NO LONG_MAX trick
        // (unlike strtol). precomp FIRST (pulls stddef so the later
        // `#define wchar_t unsigned short` is safe — the wcstoul
        // precedent). LONGLONG/_UI64_MAX/towlower(ASCII)/iswctype(ASCII)
        // supplied here (precomp lacks them); <internal/wine/msvcrt.h>
        // resolves to the refstub stub (its TRACE/_errno are all under
        // `#ifndef _LIBCNT_`, never compiled). One TU defines all six
        // public symbols + the (int)_wtoi64 wrapper that IS atoi.c's
        // body under _UNICODE.
        ("wtoi64", format!(
            "#include <precomp.h>\n#define wchar_t unsigned short\n\
             #define LONGLONG long long\n#define _UI64_MAX 0xffffffffffffffffULL\n\
             #define towlower(c) (((c)>='A'&&(c)<='Z')?((c)+32):(c))\n\
             #define _SPACE 1\n#define _DIGIT 2\n#define _ALPHA 4\n#define _UPPER 8\n\
             static int cnt_iswctype(int c,int t){{\
             if(c<0||c>127)return 0;\
             if(t==_SPACE)return c==' '||c=='\\t'||c=='\\n'||c=='\\v'||c=='\\f'||c=='\\r';\
             if(t==_DIGIT)return c>='0'&&c<='9';\
             if(t==_UPPER)return c>='A'&&c<='Z';\
             if(t==_ALPHA)return (c>='A'&&c<='Z')||(c>='a'&&c<='z');\
             return 0;}}\n\
             #define iswctype(c,t) cnt_iswctype((c),(t))\n\
             #define _wtoi64 ref_wtoi64\n#define _wtoi64_l ref_wtoi64_l\n\
             #define _wcstoi64 ref_wcstoi64\n#define _wcstoi64_l ref_wcstoi64_l\n\
             #define _wcstoui64 ref_wcstoui64\n#define _wcstoui64_l ref_wcstoui64_l\n{}\
             int ref_wtoi(const unsigned short*n){{return (int)ref_wtoi64(n);}}\n",
            inc(s("string/wtoi64.c")))),
        // _wtol — wine wtol.c. No precomp (it pulls host string.h/
        // ctype.h + <basetsd.h>→refstub stub). ASCII iswctype shim.
        // host `unsigned long` is 64-bit → wrap result in `(int)` to
        // pin the 32-bit Windows `long` semantics (strtol precedent).
        ("wtol", format!(
            "#define wchar_t unsigned short\n\
             #define _SPACE 1\n\
             static int cnt_iswctype(int c,int t){{\
             if(c<0||c>127)return 0;\
             if(t==_SPACE)return c==' '||c=='\\t'||c=='\\n'||c=='\\v'||c=='\\f'||c=='\\r';\
             return 0;}}\n\
             #define iswctype(c,t) cnt_iswctype((c),(t))\n\
             #define _wtol ru_wtol_real\n{}\
             int ref_wtol(const unsigned short*n){{return (int)ru_wtol_real(n);}}\n",
            inc(s("string/wtol.c")))),
        // Wave H — ASCII case fold. _strlwr/_wcsupr: <string.h>, no
        // guard. _strupr/_wcslwr: precomp + NULL guard calling
        // _invalid_parameter/_CRT_WIDE — stubbed to a no-op so the
        // NULL path is just `return NULL` (the handler dispatch is not
        // part of the value contract; manual.rs covers it separately).
        // wide units: stddef THEN `#define wchar_t unsigned short`
        // (host wchar_t is 32-bit). tolower: scalar, SAL stubs.
        // towupper: NLS — windef.h + ndk/rtlfuncs.h pull the identical
        // refstub CP1252 Rtl model (Wave E2's pattern); wint_t/_CRTIMP
        // supplied here.
        ("strlwr_nt", format!(
            "#define __cdecl\n#define _Inout_z_\n\
             #define _strlwr ref_strlwr\n{}",
            inc(s("string/_strlwr_nt.c")))),
        ("strupr_nt", format!(
            "#include <precomp.h>\n\
             #define _invalid_parameter(a,b,c,d,e) ((void)0)\n\
             #define _CRT_WIDE(x) L\"\"\n#define _Inout_z_\n\
             #define _strupr ref_strupr\n{}",
            inc(s("string/_strupr_nt.c")))),
        ("wcslwr_nt", format!(
            "#include <precomp.h>\n#define wchar_t unsigned short\n\
             #define _invalid_parameter(a,b,c,d,e) ((void)0)\n\
             #define _CRT_WIDE(x) L\"\"\n#define _Inout_z_\n\
             #define _wcslwr ref_wcslwr\n{}",
            inc(s("wstring/_wcslwr_nt.c")))),
        ("wcsupr_nt", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #define __cdecl\n#define _Inout_z_\n\
             #define _wcsupr ref_wcsupr\n{}",
            inc(s("wstring/_wcsupr_nt.c")))),
        ("tolower_nt", format!(
            "#define _Check_return_\n#define _CRTIMP\n#define __cdecl\n\
             #define _In_\n#define tolower ref_tolower\n{}",
            inc(s("string/tolower_nt.c")))),
        ("towupper_nt", format!(
            "#define _CRTIMP\n#define wint_t unsigned short\n\
             #define towupper ref_towupper\n{}",
            inc(s("string/towupper_nt.c")))),
        // Wave I — in-place mutate. strrev.c pulls precomp (CDECL).
        // strset.c self-typedefs size_t: force _WIN64 so it picks the
        // 64-bit `unsigned __int64 size_t` (== host usize ABI; needs
        // __int64 since clang skips strset.c's GNU __int64 #define),
        // pinning the genuine `(int)sizeMaxFill` signed-cast.
        ("strrev", format!(
            "#define _strrev ref_strrev\n{}",
            inc(s("string/strrev.c")))),
        ("strset", format!(
            "#define _WIN64\n#define __int64 long long\n\
             #define _strset ref_strset\n#define _strnset ref_strnset\n{}",
            inc(s("string/strset.c")))),
        // Wave PATH — _splitpath/_wsplitpath share _tsplitpath_x.h
        // (non-secure: IS_SECAPI undefined → void, nullable out bufs,
        // INT_MAX caps). _LIBCNT_ (precomp.h) compiles out the
        // _ismbblead MBCS block + WINVER>=0x600 errno path. Include
        // the logic header directly with our own _tsplitpath_x rename
        // (the .c's `#define _tsplitpath_x _splitpath` would win if we
        // pulled the .c) — exactly the tcs* logic-header idiom.
        // Neutralize the SAL annotations the non-secure prototype
        // spells out. <mbctype.h> resolves to the refstub stub.
        ("splitpath", format!(
            "typedef char _TCHAR;\n\
             #define _ismbblead(c) 0\n#define _CRTIMP\n\
             #define _CRT_INSECURE_DEPRECATE(x)\n#define _In_z_\n\
             #define _Out_writes_opt_z_(x)\n#define _In_\n\
             #define _Pre_maybenull_\n#define _Post_z_\n\
             #define _Analysis_assume_(x)\n#define _Check_return_wat_\n\
             #define _CRTIMP_ALTERNATIVE\n\
             #define _tsplitpath_x ref_splitpath\n{}",
            inc(s("string/_tsplitpath_x.h")))),
        ("wsplitpath", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             typedef unsigned short _TCHAR;\n#define _UNICODE\n\
             #define _ismbblead(c) 0\n#define _CRTIMP\n\
             #define _CRT_INSECURE_DEPRECATE(x)\n#define _In_z_\n\
             #define _Out_writes_opt_z_(x)\n#define _In_\n\
             #define _Pre_maybenull_\n#define _Post_z_\n\
             #define _Analysis_assume_(x)\n#define _Check_return_wat_\n\
             #define _CRTIMP_ALTERNATIVE\n\
             #define _tsplitpath_x ref_wsplitpath\n{}",
            inc(s("string/_tsplitpath_x.h")))),
        // makepath.c: host <stdlib.h>/<string.h> (memmove/strlen/
        // strcpy). wmakpath.c: force wchar_t→unsigned short (u16
        // parity) + supply strlenW/strcpyW as u16 helpers (wine's are
        // wcslen/wcscpy which take host 32-bit wchar_t).
        ("makepath", format!(
            "#define _makepath ref_makepath\n{}",
            inc(s("stdlib/makepath.c")))),
        ("wmakepath", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             static unsigned long _wlenW(const wchar_t*s){{unsigned long n=0;while(s[n])n++;return n;}}\n\
             static wchar_t* _wcpyW(wchar_t*d,const wchar_t*s){{unsigned long i=0;do{{d[i]=s[i];}}while(s[i++]);return d;}}\n\
             #define strlenW _wlenW\n#define strcpyW _wcpyW\n\
             #define _wmakepath ref_wmakepath\n{}",
            inc(s("stdlib/wmakpath.c")))),
        // Wave STRTOK — strtok/strtok_s (string.cmake) + wcstok/
        // wcstok_s (wstring.cmake). strtok/wcstok use the genuine
        // per-thread `msvcrt_get_thread_data()->{strtok,wcstok}_next`;
        // host-side that is a per-TU process-global thread_data_t
        // (single-threaded-equivalent — the diff drives identical
        // call sequences so the independent saved ptrs stay in sync).
        // strtok.c/strtok_s.c pull <internal/wine/msvcrt.h> (refstub
        // guard stub) — the typedef/accessor are supplied here. wcstok
        // needs wchar_t→u16, a u16 strchrW (=wcschr, matches the
        // terminator), and _set_errno (no-op; _LIBCNT_).
        ("strtok", format!(
            "typedef struct cnt_td {{ char* strtok_next; unsigned short* wcstok_next; }} thread_data_t;\n\
             static thread_data_t _cnt_td;\n\
             static thread_data_t* msvcrt_get_thread_data(void){{ return &_cnt_td; }}\n\
             #define strtok ref_strtok\n{}",
            inc(s("string/strtok.c")))),
        ("strtok_s", format!(
            "#define strtok_s ref_strtok_s\n{}",
            inc(s("string/strtok_s.c")))),
        ("wcstok", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             typedef struct cnt_td {{ char* strtok_next; unsigned short* wcstok_next; }} thread_data_t;\n\
             static thread_data_t _cnt_td;\n\
             static thread_data_t* msvcrt_get_thread_data(void){{ return &_cnt_td; }}\n\
             static unsigned short* _cnt_wcschr(const unsigned short* s, unsigned short c){{ do {{ if(*s==c) return (unsigned short*)s; }} while(*s++); return 0; }}\n\
             #define strchrW _cnt_wcschr\n#define _set_errno(x) ((void)0)\n\
             #define wcstok ref_wcstok\n#define wcstok_s ref_wcstok_s\n{}",
            inc(s("wstring/wcstok.c")))),
        // env — getenv.c/_wgetenv + putenv.c/wputenv.c + the genuine
        // environ.c SetEnv (duplicate-on-first-write, dual narrow+wide
        // sync, the prefix-asymmetry vs getenv's exact match) + the 4
        // __p__* accessors. ALL FOUR .c in ONE TU (environ.c FIRST so
        // its `char ***__p__environ()` etc. definitions serve as the
        // real prototypes for getenv.c — implicit-int would truncate
        // the returned pointer on 64-bit, and so SetEnv resolves for
        // putenv.c). wchar_t→unsigned short (host is 32-bit; u16
        // parity). The Win32 environment surface is neutralized to a
        // deterministic CP1252 model identical to core_rs/mbcp.rs (the
        // Wave E2 pattern): GetEnvironmentStringsA derives from the
        // wide block via CP1252 so ref's narrow _environ == cnt's;
        // SetEnvironmentVariableW is a TRUE stub so the OS-update
        // result is deterministically 0. `ref_env_set_block(w,n)`
        // seeds (BlockEnvToEnvironW+A) — the diff drives identical
        // op sequences from there. `_strnicmp`/`_wcsnicmp` are the
        // libcntpr NT ASCII fold (A..Z only), matching env::fold.
        ("env", format!(
            "#include <stddef.h>\n#define wchar_t unsigned short\n\
             #include <stdlib.h>\n#include <string.h>\n\
             #define TRACE(...) ((void)0)\n\
             #define CP_ACP 0\n#define errno_t int\n#define _UNKNOWN_APP 0\n\
             static int cnt_fold(int c){{return (c>='A'&&c<='Z')?c+32:c;}}\n\
             static int cnt_strnicmp(const char*a,const char*b,size_t n){{\
               size_t i;for(i=0;i<n;i++){{int x=cnt_fold((unsigned char)a[i]),\
               y=cnt_fold((unsigned char)b[i]);if(x!=y)return x-y;\
               if(a[i]==0)return 0;}}return 0;}}\n\
             static int cnt_wcsnicmp(const unsigned short*a,const unsigned short*b,\
               size_t n){{size_t i;for(i=0;i<n;i++){{int x=cnt_fold(a[i]),\
               y=cnt_fold(b[i]);if(x!=y)return x-y;if(a[i]==0)return 0;}}return 0;}}\n\
             static size_t cnt_wcslen(const unsigned short*s){{size_t n=0;\
               while(s[n])n++;return n;}}\n\
             static unsigned short* cnt_wcschr(const unsigned short*s,\
               unsigned short c){{do{{if(*s==c)return (unsigned short*)s;}}\
               while(*s++);return 0;}}\n\
             static char* cnt_strdup(const char*s){{size_t n=strlen(s)+1;\
               char*p=(char*)malloc(n);if(p)memcpy(p,s,n);return p;}}\n\
             static unsigned short* cnt_wcsdup(const unsigned short*s){{\
               size_t n=cnt_wcslen(s)+1;unsigned short*p=(unsigned short*)\
               malloc(n*sizeof(unsigned short));if(p)memcpy(p,s,\
               n*sizeof(unsigned short));return p;}}\n\
             static struct cnt_li{{int mb_cur_max;}} cnt_locinfo={{1}};\n\
             static int cnt_errno_v; int* _errno(void){{return &cnt_errno_v;}}\n\
             #define get_locinfo() (&cnt_locinfo)\n\
             #define _strnicmp cnt_strnicmp\n#define _wcsnicmp cnt_wcsnicmp\n\
             #define wcslen cnt_wcslen\n#define wcschr cnt_wcschr\n\
             #define _strdup cnt_strdup\n#define _wcsdup cnt_wcsdup\n\
             #define getenv ref_getenv\n#define _wgetenv ref__wgetenv\n\
             #define _putenv ref__putenv\n#define _wputenv ref__wputenv\n\
             #define __p__environ ref___p__environ\n\
             #define __p__wenviron ref___p__wenviron\n\
             #define __p___initenv ref___p___initenv\n\
             #define __p___winitenv ref___p__winitenv\n\
             static const unsigned short CP1252H[32]={{0x20AC,0x0081,0x201A,\
               0x0192,0x201E,0x2026,0x2020,0x2021,0x02C6,0x2030,0x0160,0x2039,\
               0x0152,0x008D,0x017D,0x008F,0x0090,0x2018,0x2019,0x201C,0x201D,\
               0x2022,0x2013,0x2014,0x02DC,0x2122,0x0161,0x203A,0x0153,0x009D,\
               0x017E,0x0178}};\n\
             static unsigned short cp_dec(unsigned char b){{\
               return (b>=0x80&&b<0xA0)?CP1252H[b-0x80]:(unsigned short)b;}}\n\
             static unsigned char cp_enc(unsigned short w){{int i;\
               if(w<0x80||(w>=0xA0&&w<=0xFF))return (unsigned char)w;\
               for(i=0;i<32;i++)if(CP1252H[i]==w)return (unsigned char)(0x80+i);\
               return '?';}}\n\
             int MultiByteToWideChar(unsigned int cp,unsigned int fl,\
               const char*src,int slen,unsigned short*dst,int dlen){{\
               int n=0;const unsigned char*s=(const unsigned char*)src;\
               (void)cp;(void)fl;if(slen<0){{for(;;){{unsigned short w=cp_dec(s[n]);\
               if(dlen>0&&n<dlen)dst[n]=w;n++;if(s[n-1]==0)break;}}return n;}}\
               for(;n<slen;n++){{unsigned short w=cp_dec(s[n]);\
               if(dlen>0&&n<dlen)dst[n]=w;}}return n;}}\n\
             int WideCharToMultiByte(unsigned int cp,unsigned int fl,\
               const unsigned short*src,int slen,char*dst,int dlen,\
               const char*dc,int*udc){{int n=0;(void)cp;(void)fl;(void)dc;\
               (void)udc;if(slen<0){{for(;;){{unsigned char b=cp_enc(src[n]);\
               if(dlen>0&&n<dlen)dst[n]=(char)b;n++;if(src[n-1]==0)break;}}\
               return n;}}for(;n<slen;n++){{unsigned char b=cp_enc(src[n]);\
               if(dlen>0&&n<dlen)dst[n]=(char)b;}}return n;}}\n\
             int SetEnvironmentVariableW(const unsigned short*nm,\
               const unsigned short*vl){{(void)nm;(void)vl;return 1;}}\n\
             static unsigned short g_wblock[16384];static char g_nblock[16384];\n\
             char* GetEnvironmentStringsA(void){{return g_nblock;}}\n\
             unsigned short* GetEnvironmentStringsW(void){{return g_wblock;}}\n\
             int FreeEnvironmentStringsA(char*p){{(void)p;return 1;}}\n\
             int FreeEnvironmentStringsW(unsigned short*p){{(void)p;return 1;}}\n\
             int BlockEnvToEnvironA(void);int BlockEnvToEnvironW(void);\n\
             void ref_env_set_block(const unsigned short*w,int n){{int k;\
               if(n>16382)n=16382;for(k=0;k<n;k++)g_wblock[k]=w[k];\
               g_wblock[n]=0;g_wblock[n+1]=0;\
               for(k=0;k<n;k++)g_nblock[k]=(char)cp_enc(g_wblock[k]);\
               g_nblock[n]=0;g_nblock[n+1]=0;\
               BlockEnvToEnvironW();BlockEnvToEnvironA();}}\n{}{}{}{}",
            inc(s("misc/environ.c")),
            inc(s("stdlib/getenv.c")),
            inc(s("stdlib/putenv.c")),
            inc(s("stdlib/wputenv.c")))),
        // float-cvt Phase 1 — _fcvt/fcvtbuf. Self-contained: the
        // genuine `cvt()` is a pure host-`modf` digit loop (NO
        // streamout sprintf — that's _ecvt/_gcvt, later phases). One
        // TU: fcvtbuf.c (defines fcvtbuf+static cvt) THEN fcvt.c
        // (re-declares fcvtbuf — the macro renames decl+def alike).
        // host modf is exact for binary64 == cnt's bit-modf, so they
        // agree on every finite double; `_isnan`/`_finite` → host
        // isnan/isfinite (irrelevant on the fuzzed finite domain).
        // <malloc.h>/precomp via refstub; <float.h> for DBL_MAX_10_EXP.
        ("fcvt", format!(
            "#include <float.h>\n#include <math.h>\n\
             #define _isnan(x) isnan(x)\n#define _finite(x) isfinite(x)\n\
             #define fcvtbuf ref_fcvtbuf\n#define _fcvt ref__fcvt\n{}{}",
            inc(s("stdlib/fcvtbuf.c")),
            inc(s("stdlib/fcvt.c")))),
        // ── Wave MATH-CLASS — IEEE-754 predicates + integer abs ────────
        // Pure bit ops. abs/labs + the int/short-returning predicates
        // compile the genuine .c (SAL no-ops + empty __cdecl shimmed in;
        // each .c pulls its own <stdint.h>/<float.h>). _dclass/_fdclass/
        // _dtest/_fdtest are transcribed inline because their .c pulls
        // <math.h>, whose FP_* codes diverge host-vs-ReactOS — hardcode
        // the genuine kernel values (sdk/include/crt/math.h: _NANCODE 2,
        // _INFCODE 1, _FINITE -1, _DENORM -2, FP_ZERO 0).
        ("abs", format!("#define __cdecl\n#define CDECL\n#define abs ref_abs\n{}", inc(s("math/abs.c")))),
        ("labs", format!("#define __cdecl\n#define CDECL\n#define labs ref_labs\n{}", inc(s("math/labs.c")))),
        ("isnan", format!("#define _In_\n#define _Check_return_\n#define __cdecl\n#define _isnan ref_isnan\n{}", inc(s("math/_isnan.c")))),
        ("isnanf", format!("#define _In_\n#define _Check_return_\n#define __cdecl\n#define _isnanf ref_isnanf\n{}", inc(s("math/_isnanf.c")))),
        ("finite", format!("#define _In_\n#define _Check_return_\n#define __cdecl\n#define _finite ref_finite\n{}", inc(s("math/_finite.c")))),
        ("finitef", format!("#define _In_\n#define _Check_return_\n#define __cdecl\n#define _finitef ref_finitef\n{}", inc(s("math/_finitef.c")))),
        ("dsign", format!("#define _In_\n#define _Check_return_\n#define __cdecl\n#define _dsign ref_dsign\n{}", inc(s("math/_dsign.c")))),
        ("fdsign", format!("#define _In_\n#define _Check_return_\n#define __cdecl\n#define _fdsign ref_fdsign\n{}", inc(s("math/_fdsign.c")))),
        ("dclass", "#include <stdint.h>\nshort ref_dclass(double x){union{double f;uint64_t u;}v;v.f=x;uint64_t e=v.u&0x7FF0000000000000ULL,m=v.u&0x000FFFFFFFFFFFFFULL;if(e==0x7FF0000000000000ULL)return m?2:1;if(e==0)return m?-2:0;return -1;}".to_string()),
        ("fdclass", "#include <stdint.h>\nshort ref_fdclass(float x){union{float f;uint32_t u;}v;v.f=x;uint32_t e=v.u&0x7F800000u,m=v.u&0x007FFFFFu;if(e==0x7F800000u)return m?2:1;if(e==0)return m?-2:0;return -1;}".to_string()),
        ("dtest", "short ref_dclass(double);\nshort ref_dtest(double*p){return ref_dclass(*p);}".to_string()),
        ("fdtest", "short ref_fdclass(float);\nshort ref_fdtest(float*p){return ref_fdclass(*p);}".to_string()),
        // ── Wave MATH-TIERB — float-returning bit-ops + integer div ────
        // `div` (pure int, struct return) is transcribed inline (its .c
        // pulls <precomp.h>). `_chgsignf` is a sign-bit flip. scalbn/
        // scalbnf are the genuine musl .c (self-contained FP, no libm).
        // round/roundf are the genuine ReactOS .c (call host ceil/floor
        // — libm is linked below). nextafter/nextafterf/nexttowardf are
        // transcribed (their .c pull reactos "libm.h"); the FORCE_EVAL
        // fenv side-effects are dropped (they do not change the return).
        // nexttowardf's reference takes a `double` y, matching the
        // amd64/arm64 ABI the Rust serves (long double == double there).
        ("div", "typedef struct{int quot;int rem;}rdiv_t;\nrdiv_t ref_div(int num,int denom){if(num>0&&denom<0){num=-num;denom=-denom;}rdiv_t r;r.quot=num/denom;r.rem=num%denom;if(num<0&&denom>0){if(r.rem>0){r.quot++;r.rem-=denom;}}return r;}".to_string()),
        ("chgsignf", "#include <stdint.h>\nfloat ref_chgsignf(float x){union{float f;uint32_t u;}v;v.f=x;v.u^=0x80000000u;return v.f;}".to_string()),
        ("scalbn", format!("#define scalbn ref_scalbn\n{}", inc(s("math/scalbn.c")))),
        ("scalbnf", format!("#define scalbnf ref_scalbnf\n{}", inc(s("math/scalbnf.c")))),
        ("round", format!("#define round ref_round\n{}", inc(s("math/round.c")))),
        ("roundf", format!("#define roundf ref_roundf\n{}", inc(s("math/roundf.c")))),

        // ── the last of crtmath's plain-C float/misc surface ──────────────
        //
        // The seven f32 transcendental wrappers (`sinf`/`cosf`/`expf`/`logf`/
        // `log10f`/`powf`/`fmodf`) are deliberately NOT graded here, and the
        // reason is the oracle, not the effort. Each is `return (float)sin(
        // (double)x);` — so a reference TU's `sin` binds the HOST's libm, and
        // the row would compare "glibc sin, narrowed" against "our Rust sin,
        // narrowed". That grades `sin`, which tests/diff.rs already grades
        // directly and bit-exactly, and it would fail on a libm difference
        // that says nothing about the wrapper. There is no third
        // implementation to be the oracle, so a green row would be
        // reassurance rather than evidence.
        //
        // The five below are the opposite case: self-contained bit and
        // integer logic, each with a genuine standalone C body, and each with
        // real transcription risk (a bitfield's width, a truncation fixup's
        // sign condition, a NaN's quiet bit).
        ("copysign", format!("#define _copysign ref_copysign\n{}", inc(s("float/copysign.c")))),
        ("copysignf", format!("#define _In_\n#define _Check_return_\n#define __cdecl\n#define _copysign ref_copysign_priv\n{}#define _copysignf ref_copysignf\n{}",
            inc(s("float/copysign.c")), inc(s("math/_copysignf.c")))),
        ("frexp", format!("#define frexp ref_frexp\n{}", inc(s("math/frexp.c")))),
        // ldiv is LP32-vs-LP64, the same trap strtoul above documents: Windows
        // `long` is 32-bit on all three of our arches, the host's is 64. Left
        // alone, the reference returns a 16-byte {long,long} in RAX:RDX while
        // the Rust side reads an 8-byte {i32,i32} out of RAX, and the rows
        // compare a correct answer against register debris. (Found by this
        // suite on its first run, reading quot=1/rem=-1 as 0/0.) precomp.h is
        // pre-included so ldiv.c's own guarded `#include <precomp.h>` is a
        // no-op and `#define long int` cannot reach its `__int64` macro.
        ("ldiv", format!("#include <precomp.h>\ntypedef struct {{ int quot; int rem; }} ldiv_t;\n#define long int\n#define ldiv ref_ldiv\n{}", inc(s("math/ldiv.c")))),
        // fpclass.c needs float.h's `_FPCLASS_*`, which the host's <float.h>
        // does not have. Including the SDK's ucrt/float.h whole is not an
        // option (it pulls corecrt.h and the rest of the ucrt chain), so the
        // ten `#define _FPCLASS_*` lines are LIFTED FROM THAT HEADER at build
        // time rather than retyped here. The difference matters: a hand copy
        // is a second source of truth that goes stale silently, and these are
        // precisely the constants the function is made of.
        ("fpclass", format!("{}#define _fpclass ref_fpclass\n{}",
            fpclass_consts, inc(s("float/fpclass.c")))),
        ("nextafter", "#include <stdint.h>\ndouble ref_nextafter(double x,double y){union{double f;uint64_t i;}ux={x},uy={y};uint64_t ax,ay;if(x!=x||y!=y)return x+y;if(ux.i==uy.i)return y;ax=ux.i&0x7fffffffffffffffULL;ay=uy.i&0x7fffffffffffffffULL;if(ax==0){if(ay==0)return y;ux.i=(uy.i&(1ULL<<63))|1;}else if(ax>ay||((ux.i^uy.i)&(1ULL<<63)))ux.i--;else ux.i++;return ux.f;}".to_string()),
        ("nextafterf", "#include <stdint.h>\nfloat ref_nextafterf(float x,float y){union{float f;uint32_t i;}ux={x},uy={y};uint32_t ax,ay;if(x!=x||y!=y)return x+y;if(ux.i==uy.i)return y;ax=ux.i&0x7fffffffu;ay=uy.i&0x7fffffffu;if(ax==0){if(ay==0)return y;ux.i=(uy.i&0x80000000u)|1;}else if(ax>ay||((ux.i^uy.i)&0x80000000u))ux.i--;else ux.i++;return ux.f;}".to_string()),
        // Wave MATH-TIERC sub-wave 1: sqrt/sqrtf = libm (correctly rounded),
        // matching cnt-rust's hardware sqrt instruction.
        //
        // sin/cos USED to be graded bit-exact against math/{sin,cos}.c, and
        // that oracle is now GONE rather than relaxed. cnt-rust implements
        // AMD's libm_sse2/{sin,cos}.asm, which no C in this tree expresses, so
        // there is nothing to compile as a reference — and the portable .c is
        // not a weaker oracle, it is a WRONG one: it answers -3.3e-15 for
        // sin(0), carries ~1e8 ULP of error at its best, and returns 1e308
        // above 1e9. Grading a correct implementation against it would only
        // ever report the correct one as broken.
        //
        // So sin/cos join sqrt in being graded against libm, but to a ULP
        // bound rather than bit-exactly, since AMD's algorithm and glibc's are
        // different algorithms. See diff_math_sincos for the bound and why it
        // is a real regression detector rather than a rubber stamp.
        ("sin", "#include <math.h>\ndouble ref_sin(double x){return sin(x);}".to_string()),
        ("cos", "#include <math.h>\ndouble ref_cos(double x){return cos(x);}".to_string()),
        ("sqrt", "#include <math.h>\ndouble ref_sqrt(double x){return sqrt(x);}".to_string()),
        ("sqrtf", "#include <math.h>\nfloat ref_sqrtf(float x){return sqrtf(x);}".to_string()),
        ("nexttowardf", "#include <stdint.h>\nfloat ref_nexttowardf(float x,double y){union{float f;uint32_t i;}ux={x};if(x!=x||y!=y)return x+(float)y;if((double)x==y)return (float)y;if(x==0){ux.i=1;union{double f;uint64_t u;}uy={y};if(uy.u>>63)ux.i|=0x80000000u;}else if((double)x<y){if(ux.i>>31)ux.i--;else ux.i++;}else{if(ux.i>>31)ux.i++;else ux.i--;}return ux.f;}".to_string()),
        // Wave MATH-TIERC sub-wave 2: exp/log/log10/pow/fmod = the genuine
        // portable kernel-CRT .c (the same math/arm64/*.c used on both
        // amd64-llvm-mingw and arm64), renamed to ref_*. exp/log/fmod are
        // real algorithms (log10/pow were upstream's rough stubs and are no longer
        // references: cnt-rust takes them from the libm crate now) — all are
        // self-contained (no libm call) and bit-exact under -ffp-contract=off.
        // The `#define X ref_X` token-renames only the public symbol (e.g.
        // `log` does not touch `log10`/`logb`; `fmod` does not touch `fmodf`).
        // Include <math.h> BEFORE the rename so the genuine .c's own
        // `#include <math.h>` no-ops via its guard — otherwise `#define exp
        // ref_exp` rewrites the token inside glibc's `__MATHCALL(exp,…)`
        // macro and corrupts the header. log10.c has no <math.h> include.
        ("exp", format!("#include <math.h>\n#define exp ref_exp\n{}", inc(s("math/arm64/exp.c")))),
        ("log", format!("#include <math.h>\n#define log ref_log\n{}", inc(s("math/arm64/log.c")))),
        ("fmod", format!("#include <math.h>\n#define fmod ref_fmod\n{}", inc(s("math/arm64/fmod.c")))),
        // Wave MATH-TIERC sub-wave 4: ceil/floor via __builtin_* so clang -O2
        // lowers them to hardware roundsd/roundss (correctly rounded) — an
        // INDEPENDENT reference, not cnt-rust's #[no_mangle] ceil/floor (which
        // would otherwise hijack a plain `ceil(x)` call and make the test
        // circular). Exact directed rounding ⇒ bit-exact vs the Rust bit-math.
        ("ceil", "double ref_ceil(double x){return __builtin_ceil(x);}".to_string()),
        ("floor", "double ref_floor(double x){return __builtin_floor(x);}".to_string()),
        ("ceilf", "float ref_ceilf(float x){return __builtin_ceilf(x);}".to_string()),
        ("floorf", "float ref_floorf(float x){return __builtin_floorf(x);}".to_string()),
        // Wave MATH-TIERC sub-wave 6: fabs/fabsf = the genuine portable
        // math/fabs.c + math/fabsf.c (the same C arm64 ships and that amd64's
        // asm mirrors). math.h FIRST so the rename hits only the definition;
        // the host is x86-64 SysV so `_M_IX86` is undefined → fabs.c's i386
        // error-bit branch is dead, matching the amd64/arm64 NaN-unchanged
        // path. fabsf.c sets the quiet bit unconditionally. Renamed → ref_*.
        ("fabs", format!("#include <math.h>\n#define _Check_return_\n#define _In_\n#define __cdecl\n#define fabs ref_fabs\n{}", inc(s("math/fabs.c")))),
        ("fabsf", format!("#include <math.h>\n#define _Check_return_\n#define _In_\n#define __cdecl\n#define fabsf ref_fabsf\n{}", inc(s("math/fabsf.c")))),
        // Wave MATH-TIERC sub-wave 7: modf = an exact integer/fraction split
        // (no FP rounding). Self-contained C faithful to math/libm_sse2/modf.c
        // (the GET/PUT_BITS_DP64 union-based bit moves inlined) so it needs
        // none of the AMD libm.h headers. Exact ⇒ bit-exact vs the Rust.
        // The PORTABLE atan/atan2 (math/arm64/{atan,atan2}.c, fdlibm) — the
        // algorithm cnt-rust actually implements, so this is the bit-exact
        // oracle. It is NOT what amd64/i386 link; those get AMD's, which is
        // graded alongside it as `ref_atan`/`ref_atan2` to a 1-ULP bound. Both
        // in ONE TU, atan.c first so ref_atan_p is declared before atan2.c
        // calls it (separate TUs give an implicit-int declaration and garbage).
        ("atan_p", format!("#include <math.h>\n#define atan ref_atan_p\n#define atan2 ref_atan2_p\n{}\n{}", inc(s("math/arm64/atan.c")), inc(s("math/arm64/atan2.c")))),
        ("modf", "double ref_modf(double x, double *iptr){\
            union {double d; unsigned long long i;} u; long long xexp;\
            unsigned long long ux, ax, mask;\
            u.d = x; ux = u.i; ax = ux & 0x7fffffffffffffffULL;\
            if (ax >= 0x4340000000000000ULL){\
                if (ax > 0x7ff0000000000000ULL){ *iptr = x; return x + x; }\
                *iptr = x; u.i = ux & 0x8000000000000000ULL; return u.d;\
            } else if (ax < 0x3ff0000000000000ULL){\
                u.i = ux & 0x8000000000000000ULL; *iptr = u.d; return x;\
            }\
            xexp = ((ux & 0x7ff0000000000000ULL) >> 52) - 1023;\
            mask = 1; mask = (mask << (52 - xexp)) - 1;\
            u.i = ux & ~mask; *iptr = u.d; return x - u.d;\
        }".to_string()),
        // float-cvt Phase 2 — _ecvt has its own dedicated cc::Build
        // (`ecvt_build`, like scanf) because format_float casts to
        // `long double`: the SysV host default is 80-bit x87 which
        // diverges from the Windows MSVC ABI (long double == double ==
        // 64-bit == the f64 core). Built with `-mlong-double-64`.
        // See `ecvt_body` / `ecvt_build` below.
        // Wave SCANF (Phase 1) — the genuine engine is winesup.c
        // (_internal_handle_float + _LIBCNT_locinfo) + scanf.c (the
        // _LIBCNT_ STRING/STRING_LEN instantiations + char2digit + the
        // 4 thin variadic wrappers). Compile BOTH in one TU (winesup.c
        // first so its defs are in scope — the streamout.c precedent).
        // wchar_t→unsigned short (host is 32-bit; stddef FIRST so the
        // typedef ran once before the macro). __ms_va_* → host stdarg.
        // _EM_* are inert (the _control87 stub is a no-op). %[ pulls
        // RTL_BITMAP/heap from refstub/wine/winternl.h (faithful to
        // rtl/bitmap.c). Rename the 4 public wrappers to ref_*.
    ];

    // Wave SCANF — its own TU + cc::Build because winesup.c's
    // _internal_handle_float uses `long double`: on the Windows TARGET
    // (MSVC ABI) that is 64-bit == double == cnt-rust's f64, but the
    // SysV host defaults to 80-bit x87 → it would diverge from BOTH
    // the target and the f64 engine. `-mlong-double-64` pins the
    // reference to the Windows ABI (the LP32 `#undef ULONG_MAX`
    // precedent, applied to a type width) so ref == target == engine.
    // genuine engine = winesup.c (_internal_handle_float +
    // _LIBCNT_locinfo) + scanf.c (the _LIBCNT_ STRING/STRING_LEN
    // instantiations + char2digit + the 4 variadic wrappers), one TU
    // (winesup.c first so its defs are in scope). wchar_t→u16 (host is
    // 32-bit; stddef FIRST so the typedef ran once before the macro).
    // %[ pulls RTL_BITMAP/heap from refstub/wine/winternl.h.
    let scanf_body = format!(
        "#define __GLIBC_USE_DEPRECATED_SCANF 1\n\
         #include <stddef.h>\n#define wchar_t unsigned short\n\
         #include <stdarg.h>\n#include <wctype.h>\n\
         #define __ms_va_list va_list\n\
         #define __ms_va_start(ap,last) va_start(ap,last)\n\
         #define __ms_va_end(ap) va_end(ap)\n\
         #define WINAPIV\n#define __declspec(x)\n#define __REACTOS__ 1\n\
         #define _EM_DENORMAL 0\n#define _EM_INVALID 0\n\
         #define _EM_ZERODIVIDE 0\n#define _EM_OVERFLOW 0\n\
         #define _EM_UNDERFLOW 0\n#define _EM_INEXACT 0\n\
         {}\
         /* winesup.c's _internal_handle_float passes va_list* and does\n\
            va_arg(*ap,..) — that MS-pointer-va_list indirection can't\n\
            be formed on the SysV/win64 host harness (segfaults). Pin\n\
            the genuine PARSE (scanf.h, fully exercised) + the genuine\n\
            SYNTHESIS MATH by overriding ONLY the store call site with\n\
            a shim running winesup.c's exact math (copied verbatim,\n\
            -mlong-double-64 == Windows ABI) into a captured pointer.\n\
            The va store mechanism itself is identical-by-inspection to\n\
            the integer path (already diff-pinned). */\n\
         static void* g_cnt_fout; static int g_cnt_fdbl;\n\
         void cnt_set_fout(void* p,int dbl){{g_cnt_fout=p;g_cnt_fdbl=dbl;}}\n\
         static void cnt_ref_ihf(int negative,int exp,int suppress,\n\
             unsigned long long d,int l_or_L,va_list* ap){{\n\
             long double cur=1,expcnt=10; int negexp=(exp<0);\n\
             if(negexp)exp=-exp;\n\
             while(exp){{if(exp&1)cur*=expcnt;exp/=2;expcnt=expcnt*expcnt;}}\n\
             cur=(negexp?d/cur:d*cur);\n\
             {{long double v=negative?-cur:cur;\n\
               if(!suppress&&g_cnt_fout){{ if(l_or_L)*(double*)g_cnt_fout=v;\n\
                 else *(float*)g_cnt_fout=(float)v; }} }}\n\
             (void)ap;}}\n\
         #define _internal_handle_float cnt_ref_ihf\n\
         #define sscanf ref_sscanf\n#define swscanf ref_swscanf\n\
         #define _snscanf ref__snscanf\n#define _snwscanf ref__snwscanf\n\
         {}",
        inc(s("string/winesup.c")),
        inc(s("string/scanf.c")),
    );

    // tcs* tchar family auto-derive: the genuine logic header
    // instantiated narrow(char)/wide(unsigned short, host wchar_t is
    // 32-bit) — exactly how the libcntpr str*/wcs* .c do it. Regular
    // ⇒ table-driven. (sym, tcs header stem, _tcsX name, wide, xint)
    let fam: &[(&str, &str, &str, bool, &str)] = &[
        ("strcpy", "tcscpy", "_tcscpy", false, ""),
        ("wcscpy", "tcscpy", "_tcscpy", true, ""),
        ("strcat", "tcscat", "_tcscat", false, ""),
        ("wcscat", "tcscat", "_tcscat", true, ""),
        ("strncat", "tcsncat", "_tcsncat", false, ""),
        ("wcsncat", "tcsncat", "_tcsncat", true, ""),
        ("wcslen", "tcslen", "_tcslen", true, ""),
        ("wcscmp", "tcscmp", "_tcscmp", true, ""),
        ("wcsncmp", "tcsncmp", "_tcsncmp", true, ""),
        ("wcsnlen", "tcsnlen", "_tcsnlen", true, ""),
        ("wcsncpy", "tcsncpy", "_tcsncpy", true, ""),
        ("wcschr", "tcschr", "_tcschr", true, "#define _XINT wchar_t\n"),
        ("wcsrchr", "tcsrchr", "_tcsrchr", true, "#define _XINT wchar_t\n"),
    ];
    for &(sym, hdr, tcsfn, wide, xint) in fam {
        let prelude = if wide {
            format!(
                "#include <stddef.h>\n#define wchar_t unsigned short\n\
                 typedef unsigned short _TCHAR;\n#define _UNICODE\n{xint}\
                 #define {tcsfn} ref_{sym}\n"
            )
        } else {
            format!(
                "typedef char _TCHAR;\n#include <stddef.h>\n{xint}\
                 #define {tcsfn} ref_{sym}\n"
            )
        };
        units.push((sym, format!("{prelude}{}", inc(s(&format!("string/{hdr}.h"))))));
    }

    let mut build = cc::Build::new();
    build
        .include(&refstub)
        // The genuine `internal/ieee.h` — `float/copysign.c` and
        // `float/fpclass.c` read a double through its `double_s` bitfield,
        // and the whole point of grading them is that THAT header's field
        // widths are the contract. Hand-copying the struct into refstub/
        // would be the difftest grading a copy of itself.
        .include(crt.join("include"))
        .warnings(false)
        .flag("-w")
        // Wave MATH-TIERC: no FMA contraction, so the FP reference matches
        // rustc's non-contracting codegen bit-for-bit (sin/cos's quadrant
        // reduction `x - q*(PI/2)` would otherwise fuse on the host).
        .flag("-ffp-contract=off")
        .opt_level(2)
        .cargo_metadata(false);
    for (sym, body) in &units {
        let w = out.join(format!("ref_{sym}.c"));
        fs::write(&w, body).unwrap();
        build.file(&w);
    }
    // cnt-rust's OWN scanf trampoline (the irreducible C half — Rust
    // can't define variadics on stable). It provides the public
    // sscanf/swscanf + cnt_va_next_ptr and calls cnt_v*scanf from the
    // cnt-rust rlib (resolved at the final test link).
    let trampoline =
        repo.join("sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c");
    build.file(&trampoline);
    println!("cargo:rerun-if-changed={}", trampoline.display());
    println!("cargo:rerun-if-changed={}", crt.display());
    println!("cargo:rerun-if-changed={}", refstub.display());
    println!("cargo:rerun-if-changed={}", refstub_printf.display());
    // Link the reference .o files *directly* (not as a static archive).
    // build-script rustc-link-lib doesn't reliably reach integration
    // tests; trailing link-args do, and direct objects sidestep
    // archive member-pull ordering entirely.
    //
    // glibc <stdio.h> (pulled by winesup.h) `__REDIRECT`s `sscanf` to
    // `__isoc99_sscanf` via an `__asm__` label that overrides our
    // `#define sscanf ref_sscanf` (the asm label wins over the macro,
    // and it's literal so it can't be macro-intercepted). `<wchar.h>`
    // is NOT pulled, so `swscanf` is unaffected. Deterministically
    // rename the genuine narrow symbol back with objcopy (no-op on
    // objects that lack it — e.g. the trampoline / other ref units).
    let redef = |obj: &std::path::Path| {
        let _ = std::process::Command::new("objcopy")
            .arg("--redefine-sym")
            .arg("__isoc99_sscanf=ref_sscanf")
            .arg(obj)
            .status();
        println!("cargo:rustc-link-arg={}", obj.display());
    };
    for obj in build.compile_intermediates() {
        redef(&obj);
    }

    // Wave MATH-TIERC (sub-waves 8+): the amd64-only AMD-libm transcendentals
    // (exp2/sinh/cosh/tanh/acos/asin/hypot) are compiled from their GENUINE
    // math/libm_sse2/*.c against their own headers (real splitexp/scaleDouble
    // + tables) — a non-circular reference for the Rust ports. Each renames
    // its public symbol to ref_<name> (`#define <name> ref_<name>`, which the
    // header's `FN_PROTOTYPE(name)→name` then re-expands). A tiny stubs.c
    // supplies the value-only behavior of the externs the kernel no-ops:
    // _handle_error returns from_bits(value); _invoke_matherr / _set_statfp
    // are side-effect-free here. Built with -ffp-contract=off to match
    // rustc's non-contracting codegen bit-for-bit.
    let libm_dir = crt.join("math/libm_sse2");
    let libm_units: Vec<(&str, String)> = vec![
        // sub-wave 8. <math.h> FIRST so the rename hits only exp2.c's
        // definition, not glibc's __MATHCALL_VEC(exp2,...) declaration.
        ("exp2", format!("#include <math.h>\n#define exp2 ref_exp2\n{}", inc(libm_dir.join("exp2.c")))),
        ("tanh", format!("#include <math.h>\n#define tanh ref_tanh\n{}", inc(libm_dir.join("tanh.c")))),
        // sub-wave 9
        ("sinh", format!("#include <math.h>\n#define sinh ref_sinh\n{}", inc(libm_dir.join("sinh.c")))),
        ("cosh", format!("#include <math.h>\n#define cosh ref_cosh\n{}", inc(libm_dir.join("cosh.c")))),
        // sub-wave 10
        ("asin", format!("#include <math.h>\n#define asin ref_asin\n{}", inc(libm_dir.join("asin.c")))),
        ("acos", format!("#include <math.h>\n#define acos ref_acos\n{}", inc(libm_dir.join("acos.c")))),
        // sub-wave 11 (symbol is _hypot, the MS CRT name)
        ("_hypot", format!("#include <math.h>\n#define _hypot ref__hypot\n{}", inc(libm_dir.join("hypot.c")))),
        // sub-wave 12 (f32 twins; sincos checked directly vs ref_sin/ref_cos)
        ("modff", format!("#include <math.h>\n#define modff ref_modff\n{}", inc(libm_dir.join("modff.c")))),
        ("_hypotf", format!("#include <math.h>\n#define _hypotf ref__hypotf\n{}", inc(libm_dir.join("hypotf.c")))),
        // sub-wave 13 (the f32 transcendental twins). Each pulls libm_inlines.h
        // (real splitexpf/scaleFloat_2/__remainder_piby2f_inline + the f32 +
        // atan(j/256) tables) via its own USE_* set — a non-circular reference.
        ("acosf", format!("#include <math.h>\n#define acosf ref_acosf\n{}", inc(libm_dir.join("acosf.c")))),
        ("asinf", format!("#include <math.h>\n#define asinf ref_asinf\n{}", inc(libm_dir.join("asinf.c")))),
        ("atanf", format!("#include <math.h>\n#define atanf ref_atanf\n{}", inc(libm_dir.join("atanf.c")))),
        ("sinhf", format!("#include <math.h>\n#define sinhf ref_sinhf\n{}", inc(libm_dir.join("sinhf.c")))),
        ("coshf", format!("#include <math.h>\n#define coshf ref_coshf\n{}", inc(libm_dir.join("coshf.c")))),
        ("tanhf", format!("#include <math.h>\n#define tanhf ref_tanhf\n{}", inc(libm_dir.join("tanhf.c")))),
        ("tanf", format!("#include <math.h>\n#define tanf ref_tanf\n{}", inc(libm_dir.join("tanf.c")))),
        ("atan2f", format!("#include <math.h>\n#define atan2f ref_atan2f\n{}", inc(libm_dir.join("atan2f.c")))),
        // ── the OTHER HALF of sub-waves 5 and 7 ──────────────────────────
        // atan/atan2/ldexp/_logb/_logbf were graded against math/arm64/*.c,
        // which is NOT what amd64 (or i386) links: math.cmake lines 95-123 put
        // AMD's math/libm_sse2/*.c on the x86 arches and math/arm64/*.c only on
        // arm64. So the oracle was the very source the Rust was ported FROM —
        // it could only ever confirm the port, never the SHIPPED semantics, and
        // it graded 0% of amd64's real behaviour. The tell was the asymmetry:
        // atan2f was already here while its double sibling atan2 was not.
        //
        // AMD's bodies differ from the portable ones only in their ERROR
        // domains, where they route through `_handle_error` (a NaN/±0/±inf
        // return + the __setusermatherr callback; ReactOS's _handle_error.c
        // drops the EDOM/ERANGE and the AMD_F_* flag word). Grading here is
        // therefore what makes those domains visible at all.
        // tan: cnt-rust hand-ports AMD's body (it used to be arm64's 18-line
        // `sin(x)/cos(x)`, which on top of math/sin.c cost amd64 up to 0.056
        // absolute — see cabi/math.rs). Needs the out-of-line Payne-Hanek
        // reduction, added to the build below; every other unit here gets the
        // inline f32 twin from libm_inlines.h and needs nothing.
        ("tan", format!("#include <math.h>\n#include <errno.h>\n#define tan ref_tan\n{}", inc(libm_dir.join("tan.c")))),
        ("atan", format!("#include <math.h>\n#include <errno.h>\n#define atan ref_atan\n{}", inc(libm_dir.join("atan.c")))),
        ("atan2", format!("#include <math.h>\n#include <errno.h>\n#define atan2 ref_atan2\n{}", inc(libm_dir.join("atan2.c")))),
        ("ldexp", format!("#include <math.h>\n#include <errno.h>\n#define ldexp ref_ldexp\n{}", inc(libm_dir.join("ldexp.c")))),
        ("logb", format!("#include <math.h>\n#include <errno.h>\n#define _logb ref__logb\n{}", inc(libm_dir.join("logb.c")))),
        ("logbf", format!("#include <math.h>\n#include <errno.h>\n#define _logbf ref__logbf\n{}", inc(libm_dir.join("logbf.c")))),
    ];
    let libm_stubs = out.join("ref_libm_stubs.c");
    fs::write(
        &libm_stubs,
        "double _handle_error(char*f,int o,unsigned long long v,int t,int fl,\
           int e,double a,double b,int n){union{double d;unsigned long long u;}x;\
           x.u=v;return x.d;}\n\
         float _handle_errorf(char*f,int o,unsigned long long v,int t,int fl,\
           int e,float a,float b,int n){union{float f;unsigned int u;}x;\
           x.u=(unsigned int)v;return x.f;}\n\
         int _invoke_matherr(int t,char*nm,double a,double b,double r){return 0;}\n\
         void _set_statfp(unsigned int f){(void)f;}\n",
    )
    .unwrap();
    let refstub_libm = manifest.join("refstub_libm");
    println!("cargo:rerun-if-changed={}", refstub_libm.display());
    let mut libm_build = cc::Build::new();
    libm_build
        .include(&refstub_libm)
        .include(&libm_dir)
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .flag("-ffp-contract=off")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&libm_stubs)
        // Out-of-line Payne-Hanek reduction, called by libm_sse2/tan.c.
        .file(libm_dir.join("remainder_piby2.c"));
    for (sym, body) in &libm_units {
        let w = out.join(format!("ref_{sym}.c"));
        fs::write(&w, body).unwrap();
        libm_build.file(&w);
    }
    for obj in libm_build.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // ── the Bessel wrappers ──────────────────────────────────────────────
    //
    // What is graded here is the errno and _fpclass shell in
    // math/{j0_y0,j1_y1,jn_yn}.c. The numerics under it are the `libm` crate's on BOTH
    // sides: tests/diff.rs defines the `__ieee754_*` names the C wrappers call as
    // `libm::j0`.., the same functions cnt-rust calls (the vendored FDLIBM C those names
    // used to bind is gone). So any divergence a row reports is in the shell.
    // refstub/ieee754/ieee754.h stands in for the deleted header's prototypes.
    //
    // `_finite`/`_fpclass`/`sqrt` are likewise left to bind cnt-rust's own
    // shims on both sides. They have their own rows already
    // (diff_math_class_f64), and pinning them here too would blur which of
    // the two layers a failure lives in.
    let bessel_units: Vec<(&str, String)> = vec![
        // The wrappers declare `int *_errno(void);` themselves; <errno.h> is for EDOM,
        // and the _FPCLASS_* values come from the SDK header the same way the fpclass
        // unit above gets them.
        ("bessel_j0_y0", format!("#include <math.h>\n#include <errno.h>\nint _finite(double);\nint _fpclass(double);\n{}#define _j0 ref__j0\n#define _y0 ref__y0\n{}",
            fpclass_consts, inc(crt.join("math/j0_y0.c")))),
        ("bessel_j1_y1", format!("#include <math.h>\n#include <errno.h>\nint _finite(double);\nint _fpclass(double);\n{}#define _j1 ref__j1\n#define _y1 ref__y1\n{}",
            fpclass_consts, inc(crt.join("math/j1_y1.c")))),
        // jn_yn.c declares _fpclass itself (via a `fpclass_t` typedef), so it
        // must NOT get a second declaration.
        ("bessel_jn_yn", format!("#include <math.h>\n#include <errno.h>\nint _finite(double);\n{}#define _jn ref__jn\n#define _yn ref__yn\n{}",
            fpclass_consts, inc(crt.join("math/jn_yn.c")))),
    ];
    let mut bessel_build = cc::Build::new();
    bessel_build
        .include(&refstub)
        // the wrappers' `#include "ieee754/ieee754.h"` finds refstub's stand-in
        .warnings(false)
        .flag("-w")
        .flag("-ffp-contract=off")
        .opt_level(2)
        .cargo_metadata(false);
    for (sym, body) in &bessel_units {
        let w = out.join(format!("ref_{sym}.c"));
        fs::write(&w, body).unwrap();
        bessel_build.file(&w);
    }
    for obj in bessel_build.compile_intermediates() {
        println!("cargo:rustc-link-arg={}", obj.display());
    }

    // Wave MATH-TIERB: ref_round/ref_roundf (genuine ReactOS round.c)
    // and cnt-rust's own `round`/`roundf` both call host `ceil`/`floor`/
    // `ceilf`/`floorf` — link libm so they resolve in the test binary.
    println!("cargo:rustc-link-lib=dylib=m");
    // float-cvt Phase 2 — _ecvt. `sprintf("%-+.*E")` → the genuine
    // `printf/streamout.c::format_float`. We compile the GENUINE
    // streamout.c (for `format_float`, the float→digits algorithm cnt
    // reimplements and we want to pin) but NOT sprintf.c/_sxprintf.c:
    // ReactOS streamout forwards `va_list` THROUGH a by-value
    // parameter into `format_float(..., va_list*)`, which is written
    // for the mingw/MSVC `char*` va_list and is broken under the
    // x86-64 SysV array `va_list` (it reads a constant garbage double
    // regardless of input). So `ref_ecvt_sprintf` drives the genuine
    // `format_float` directly via a proper single-level variadic
    // wrapper (`cnt_fmtf` — va_start in the immediate caller, the
    // standard SysV-safe idiom; the scanf `cnt_ref_ihf`-override
    // precedent) and assembles the result EXACTLY as streamout's
    // float tail does for "%-+.*E" (flags '-'|'+', precision=*,
    // fieldwidth 0 ⇒ output is just `prefix` then the format_float
    // buffer). The float→digits algorithm under test is 100% the
    // genuine streamout code; only the printf-format plumbing (a
    // different symbol) is the harness adapter. floor/log10/pow are
    // `#define`d BIT-IDENTICAL to core_rs::floatcvt::fl (Wave E2
    // shared-model discipline). Built with `-mlong-double-64`
    // (Windows MSVC ABI: long double == double == f64 == the core).
    let ecvt_body = format!(
        "#include <stddef.h>\n#include <stdarg.h>\n#include <math.h>\n\
         #include <float.h>\n#include <stdlib.h>\n#include <string.h>\n\
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
         static int ecvt_mbtowc(wchar_t*w,const char*m,size_t n){{(void)w;(void)m;(void)n;return -1;}}\n\
         static int ecvt_wctomb(char*m,wchar_t w){{(void)m;(void)w;return -1;}}\n\
         #define mbtowc ecvt_mbtowc\n#define wctomb ecvt_wctomb\n\
         #define format_float ecvt_format_float\n#define streamout ecvt_streamout\n\
         {}\
         /* genuine format_float (above) driven directly — SysV-safe */\n\
         void ecvt_format_float(char,unsigned,int,char**,const char**,va_list*);\n\
         #define format_float ecvt_format_float\n\
         static void cnt_fmtf(char chr,unsigned fl,int prec,char**str,\
           const char**pfx,...){{ va_list ap; va_start(ap,pfx);\
           format_float(chr,fl,prec,str,pfx,&ap); va_end(ap); }}\n\
         int ref_ecvt_sprintf(char* buf,const char* fmt,...){{ (void)fmt;\
           va_list a; va_start(a,fmt); int prec=va_arg(a,int);\
           double v=va_arg(a,double); va_end(a);\
           char tmp[96]; tmp[95]=0; char* string=&tmp[95];\
           const char* prefix=0;\
           /* \"%-+.*E\": FLAG_ALIGN_LEFT|FLAG_FORCE_SIGN, conv 'E' */\
           cnt_fmtf('E', 0x01u|0x02u, prec, &string, &prefix, v);\
           char* p=buf; if(prefix){{while(*prefix)*p++=*prefix++;}}\
           while(*string)*p++=*string++; *p=0; return (int)(p-buf); }}\n\
         #define sprintf ref_ecvt_sprintf\n#define _ecvt ref__ecvt\n{}",
        inc(s("printf/streamout.c")),
        inc(s("stdlib/ecvt.c")));
    let ecvt_c = out.join("ref_ecvt.c");
    fs::write(&ecvt_c, &ecvt_body).unwrap();
    let mut ecvt_build = cc::Build::new();
    ecvt_build
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .flag("-mlong-double-64")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&ecvt_c);
    for obj in ecvt_build.compile_intermediates() {
        redef(&obj);
    }
    // Wave STRTOD — strtod.c accumulates r/d in `long double` (80-bit
    // x87 on the SysV host) → built `-mlong-double-64` so the reference
    // uses 64-bit long double == the Windows ABI (amd64/arm64) == the
    // cnt-rust f64 core. That pins the f64 SEMANTICS bit-exact; the
    // documented i386 f80→f64 deviation is precisely the gap between
    // this 64-bit reference and the OLD i386 f80 C (see strtod.rs).
    // strtod.c pulls <precomp.h> via refstub; rename strtod→ref_strtod.
    let strtod_body = format!(
        "#define strtod ref_strtod\n#include <stddef.h>\n{}",
        inc(s("string/strtod.c")));
    let strtod_c = out.join("ref_strtod.c");
    fs::write(&strtod_c, &strtod_body).unwrap();
    let mut strtod_build = cc::Build::new();
    strtod_build
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .flag("-mlong-double-64")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&strtod_c);
    for obj in strtod_build.compile_intermediates() {
        redef(&obj);
    }
    // float-cvt Phase 3 — _gcvt/_gcvt_s. Pure `sprintf("%.*g")` →
    // the same genuine format_float (chr='g'). Same direct-drive
    // harness as ecvt (streamout.c for format_float; SysV-safe
    // single-level variadic wrapper; floor/log10/pow modeled
    // BIT-IDENTICAL to core_rs::floatcvt::fl; -mlong-double-64).
    // "%.*g" has NO flags and fieldwidth 0, so the output is `prefix`
    // ("-" iff <0; %g has no '+') + the format_float buffer. gcvt.c
    // also uses `_scprintf("%.*g")` (= the rendered length) and
    // `*_errno()` (not part of the diffed value contract — stubbed).
    let gcvt_body = format!(
        "#include <stddef.h>\n#include <stdarg.h>\n#include <math.h>\n\
         #include <float.h>\n#include <stdlib.h>\n#include <string.h>\n\
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
         static int gcvt_mbtowc(wchar_t*w,const char*m,size_t n){{(void)w;(void)m;(void)n;return -1;}}\n\
         static int gcvt_wctomb(char*m,wchar_t w){{(void)m;(void)w;return -1;}}\n\
         #define mbtowc gcvt_mbtowc\n#define wctomb gcvt_wctomb\n\
         #define format_float gcvt_format_float\n#define streamout gcvt_streamout\n\
         {}\
         void gcvt_format_float(char,unsigned,int,char**,const char**,va_list*);\n\
         static void cnt_gfmtf(char chr,unsigned fl,int prec,char**str,\
           const char**pfx,...){{ va_list ap; va_start(ap,pfx);\
           format_float(chr,fl,prec,str,pfx,&ap); va_end(ap); }}\n\
         static int gcvt_render(char* buf,int prec,double v){{\
           char tmp[96]; tmp[95]=0; char* string=&tmp[95];\
           const char* prefix=0;\
           cnt_gfmtf('g', 0u, prec, &string, &prefix, v);\
           char* p=buf; if(prefix){{while(*prefix)*p++=*prefix++;}}\
           while(*string)*p++=*string++; *p=0; return (int)(p-buf); }}\n\
         int ref_gcvt_sprintf(char* buf,const char* fmt,...){{ (void)fmt;\
           va_list a; va_start(a,fmt); int prec=va_arg(a,int);\
           double v=va_arg(a,double); va_end(a);\
           return gcvt_render(buf,prec,v); }}\n\
         int _scprintf(const char* fmt,...){{ (void)fmt;\
           va_list a; va_start(a,fmt); int prec=va_arg(a,int);\
           double v=va_arg(a,double); va_end(a);\
           char tmp2[128]; return gcvt_render(tmp2,prec,v); }}\n\
         static int gcvt_errno_v; static int* _errno(void){{return &gcvt_errno_v;}}\n\
         #define sprintf ref_gcvt_sprintf\n\
         #define _gcvt ref__gcvt\n#define _gcvt_s ref__gcvt_s\n{}",
        inc(s("printf/streamout.c")),
        inc(s("stdlib/gcvt.c")));
    let gcvt_c = out.join("ref_gcvt.c");
    fs::write(&gcvt_c, &gcvt_body).unwrap();
    let mut gcvt_build = cc::Build::new();
    gcvt_build
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .flag("-mlong-double-64")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&gcvt_c);
    for obj in gcvt_build.compile_intermediates() {
        redef(&obj);
    }

    // ── printf engine (streamout) — narrow differential ──────────────
    // The genuine `printf/streamout.c` is the AUTHORITATIVE oracle for
    // cnt-rust's Rust `streamout` engine (every printf-family entry
    // funnels through it). We compile it as `ref_streamout` and drive
    // BOTH it and the cnt engine with the SAME hand-built flat va_list
    // (refstub_printf/stdarg.h: one 8-byte slot per arg, the 64-bit
    // host slot cnt-rust uses) over the SAME `FILE` setup — so the diff
    // pins the parser, the va_arg layout, the `_cnt` overflow guard
    // (the signed/unsigned promotion that broke ACPI's unbounded
    // swprintf — see [[feedback]]) and the float path, with NO
    // glibc-vs-ReactOS quirk divergence (ReactOS's own code is the
    // reference). Only NARROW: the wide twin needs 16-bit `L""` wchar
    // literals which the host can't express (host wchar_t literals are
    // 32-bit even under `#define wchar_t unsigned short`); the wide
    // engine is covered end-to-end by the ACPI swprintf boot path.
    // floor/log10/pow are #defined bit-identical to core_rs::floatcvt::fl
    // (the ecvt/gcvt shared-model discipline) so `%f/%e/%g` pin the Rust
    // format_float against the genuine, libm-exactness factored out.
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
         #define streamout ref_streamout\n#define format_float ref_format_float\n\
         {}\
         /* driver: build a FILE and run the genuine engine over a flat\
            va_list (refstub_printf stdarg). `cnt` is the initial\
            _iobuf._cnt; pass a negative value to exercise the unbounded\
            swprintf overflow path. */\n\
         int ref_run(char* base,int cnt,const char* fmt,char* va){{\
           FILE f; f._base=base; f._ptr=base; f._cnt=cnt;\
           f._flag=0x40|0x2; f._charbuf=0; f._bufsiz=0; f._tmpfname=0; f._file=0;\
           return ref_streamout(&f, fmt, (va_list)va); }}\n",
        inc(s("printf/streamout.c")));
    let so_c = out.join("ref_streamout.c");
    fs::write(&so_c, &so_body).unwrap();
    let mut so_build = cc::Build::new();
    so_build
        .include(&refstub_printf)
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .flag("-mlong-double-64")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&so_c);
    for obj in so_build.compile_intermediates() {
        redef(&obj);
    }

    // ── the same engine in its _USER32_WSPRINTF configuration ────────────
    // user32's wsprintf family (user32-wsprintf-rust) compiles cnt-rust's streamout.rs
    // again under cfg(streamout_user32); tests/diff.rs includes it the same way (this
    // crate sets the cfg below). The reference is the same genuine streamout.c built
    // -D_USER32_WSPRINTF instead of -D_LIBCNT_, which is how sdk/lib/crt builds it.
    println!("cargo:rustc-check-cfg=cfg(streamout_user32)");
    println!("cargo:rustc-cfg=streamout_user32");
    let so_u32_body = so_body
        .replace("#define _LIBCNT_\n", "#define _USER32_WSPRINTF\n")
        .replace("#define streamout ref_streamout", "#define streamout ref_u32_streamout")
        .replace("#define format_float ref_format_float", "#define format_float ref_u32_format_float")
        .replace("int ref_run(", "int ref_u32_run(")
        .replace("return ref_streamout(&f,", "return ref_u32_streamout(&f,");
    assert!(so_u32_body.contains("_USER32_WSPRINTF") && so_u32_body.contains("ref_u32_run("));
    let so_u32_c = out.join("ref_u32_streamout.c");
    fs::write(&so_u32_c, &so_u32_body).unwrap();
    let mut so_u32_build = cc::Build::new();
    so_u32_build
        .include(&refstub_printf)
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .flag("-mlong-double-64")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&so_u32_c);
    for obj in so_u32_build.compile_intermediates() {
        redef(&obj);
    }

    // Dedicated scanf build with `-mlong-double-64` (Windows MSVC ABI:
    // long double == double == 64-bit; SysV host default is 80-bit
    // x87, which would diverge from the f64 engine AND the target).
    let scanf_c = out.join("ref_scanf.c");
    fs::write(&scanf_c, &scanf_body).unwrap();
    let mut scanf_build = cc::Build::new();
    scanf_build
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .flag("-mlong-double-64")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&scanf_c);
    for obj in scanf_build.compile_intermediates() {
        redef(&obj);
    }

    // Wave CTYPE — ctype.c (data tables + narrow classification +
    // wide forwarders). Skip glibc <ctype.h> via its `_CTYPE_H` guard
    // and supply the MS masks / wint_t / wctype_t ourselves; `_LIBCNT_`
    // selects the simple `_isctype` (the locale `_isctype_l` branch
    // needs MSVCRT internals). ctype.c `#undef`s the MS-macro names it
    // defines (iswlower, _tolower, __isascii, …) at the top, which would
    // defeat any `#define`-based rename — so the whole symbol set is
    // renamed to `ref_*` at the OBJECT level via `objcopy
    // --redefine-syms` (immune to the preprocessor, keeps internal refs
    // consistent). iswctype_nt.c is pulled in renamed → `ref_iswctype`
    // (reading `ref_wctype`) so the ref wide funcs + the direct
    // iswctype/is_wctype diff have a genuine compiled reference.
    // cnt-ctype now defines `iswctype` itself (the nt flip), so the cnt
    // side needs no external iswctype TU.
    let ctype_masks = "#define _CTYPE_H 1\n\
        typedef unsigned short wint_t;\ntypedef unsigned short wctype_t;\n\
        #define __cdecl\n#define _Check_return_\n#define _LIBCNT_\n\
        #define WEOF ((wint_t)0xFFFF)\n\
        #define _UPPER 0x1\n#define _LOWER 0x2\n#define _DIGIT 0x4\n\
        #define _SPACE 0x8\n#define _PUNCT 0x10\n#define _CONTROL 0x20\n\
        #define _BLANK 0x40\n#define _HEX 0x80\n#define _LEADBYTE 0x8000\n\
        #define _ALPHA (0x0100 | _UPPER | _LOWER)\n";
    let ctype_a = format!(
        "{ctype_masks}int iswctype(wint_t, wctype_t);\n{}{}",
        inc(s("string/ctype.c")),
        inc(s("string/iswctype_nt.c")),
    );
    let ctype_a_c = out.join("ref_ctype.c");
    fs::write(&ctype_a_c, &ctype_a).unwrap();
    // Object-level rename map for TU-A. The C names with leading
    // underscores get unambiguous suffixed ref names.
    let redefs = "\
        _ctype ref_ctype\n_wctype ref_wctype\n\
        _pctype ref_pctype\n_pwctype ref_pwctype\n\
        _isctype ref_isctype\n\
        isalpha ref_isalpha\nisalnum ref_isalnum\niscntrl ref_iscntrl\n\
        isdigit ref_isdigit\nisgraph ref_isgraph\nislower ref_islower\n\
        isprint ref_isprint\nispunct ref_ispunct\nisspace ref_isspace\n\
        isupper ref_isupper\nisxdigit ref_isxdigit\n\
        __isascii ref_isascii_n\niswascii ref_iswascii\n\
        __toascii ref_toascii_n\n__iscsym ref_iscsym_n\n\
        __iscsymf ref_iscsymf_n\n_tolower ref_tolower_u\n\
        _toupper ref_toupper_u\ntoupper ref_toupper\n\
        iswalnum ref_iswalnum\niswalpha ref_iswalpha\niswcntrl ref_iswcntrl\n\
        iswdigit ref_iswdigit\niswgraph ref_iswgraph\niswprint ref_iswprint\n\
        iswpunct ref_iswpunct\niswlower ref_iswlower\niswupper ref_iswupper\n\
        iswspace ref_iswspace\niswxdigit ref_iswxdigit\ntowlower ref_towlower\n\
        iswctype ref_iswctype\n__p__pwctype ref_p_pwctype\n\
        __pwctype_func ref_pwctype_func\n";
    let redefs_file = out.join("ref_ctype_redefs.txt");
    fs::write(&redefs_file, redefs).unwrap();
    // TU-A: full object-level rename → ref_*.
    let mut ctype_a_build = cc::Build::new();
    ctype_a_build
        .include(&refstub)
        .warnings(false)
        .flag("-w")
        .opt_level(2)
        .cargo_metadata(false)
        .file(&ctype_a_c);
    for obj in ctype_a_build.compile_intermediates() {
        let st = std::process::Command::new("objcopy")
            .arg(format!("--redefine-syms={}", redefs_file.display()))
            .arg(&obj)
            .status()
            .expect("objcopy --redefine-syms ref_ctype");
        assert!(st.success(), "objcopy failed on {}", obj.display());
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}
