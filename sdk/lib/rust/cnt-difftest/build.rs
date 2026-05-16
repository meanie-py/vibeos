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
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let crt = repo.join("sdk/lib/crt");

    // (ref symbol, full host TU body). Each renames the public symbol
    // to ref_<name> and includes the genuine ReactOS logic.
    let inc = |p: PathBuf| format!("#include \"{}\"\n", p.display());
    let s = |p: &str| crt.join(p);
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
        ("atoi64", format!("#define _atoi64 ref_atoi64\n{}", inc(s("string/atoi64.c")))),
        ("memcpy", format!("#define __cdecl\n#define memcpy ref_memcpy\n{}", inc(s("mem/memcpy.c")))),
        ("memchr", format!("#define __cdecl\n#define memchr ref_memchr\n{}", inc(s("mem/memchr.c")))),
        ("memcmp", format!("#define __cdecl\n#define memcmp ref_memcmp\n{}", inc(s("mem/memcmp.c")))),
        ("memmove", format!("#define __cdecl\n#define memmove ref_memmove\n{}", inc(s("mem/memmove.c")))),
        ("memset", format!("#define __cdecl\n#define memset ref_memset\n{}", inc(s("mem/memset.c")))),
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
        ("scanf", format!(
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
             #define sscanf ref_sscanf\n#define swscanf ref_swscanf\n\
             #define _snscanf ref__snscanf\n#define _snwscanf ref__snwscanf\n\
             {}{}",
            inc(s("string/winesup.c")), inc(s("string/scanf.c")))),
    ];

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
        .warnings(false)
        .flag("-w")
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
    for obj in build.compile_intermediates() {
        let _ = std::process::Command::new("objcopy")
            .arg("--redefine-sym")
            .arg("__isoc99_sscanf=ref_sscanf")
            .arg(&obj)
            .status();
        println!("cargo:rustc-link-arg={}", obj.display());
    }
}
