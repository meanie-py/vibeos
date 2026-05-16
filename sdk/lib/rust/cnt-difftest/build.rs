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
         static double cnt_log10(double f){{ if(f==0.0)return 0.0;\
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
         {}\
         /* genuine format_float (above) driven directly — SysV-safe */\n\
         void format_float(char,unsigned,int,char**,const char**,va_list*);\n\
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
}
