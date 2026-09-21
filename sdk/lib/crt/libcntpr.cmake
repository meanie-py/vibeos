
list(APPEND LIBCNTPR_SOURCE
    ${LIBCNTPR_EXCEPT_SOURCE}
    ${LIBCNTPR_FLOAT_SOURCE}
    ${LIBCNTPR_MATH_SOURCE}
    ${LIBCNTPR_MEM_SOURCE}
    ${LIBCNTPT_MISC_SOURCE}
    ${LIBCNTPR_PRINTF_SOURCE}
    ${LIBCNTPR_SEARCH_SOURCE}
    ${LIBCNTPR_STDLIB_SOURCE}
    ${LIBCNTPR_STRING_SOURCE}
    ${LIBCNTPR_WSTRING_SOURCE}
)

# ── cnt-rust cohabitation swap (libcntpr ONLY) ──────────────────────
# These .c are now provided by Rust (sdk/lib/rust/cnt-rust/ cabi, via
# libcntpr_rust.a below). REMOVE_ITEM here — NOT commenting the family
# cmakes — because CRT_*_SOURCE (user-mode msvcrt) embeds the same
# ${LIBCNTPR_*_SOURCE} vars; the swap must touch libcntpr alone.
# Files kept on disk (never-delete rule). Kept C (NOT removed):
# wctype/winesup/mbstrlen/exp2f/_invoke_matherr/fltused (UNCOVERED) +
# atol (_atoldbl). bsearch/lfind are now Rust (Wave SEARCH below).
# mbstrlen/exp2f/_invoke_matherr are kernel-dead (no callers) and each
# carries a wrinkle (NlsLeadByteInfo global / no powf / static-mut+struct)
# so they stay C. wctype.c is only the dead wine_wctype_table data
# (no funcs; unused by the linked iswctype). winesup.c stays C but its
# _internal_handle_float is now dead (the Rust scanf engine does its own
# f64 synth — see the Wave SCANF block below).
# Wave WCS flips the kernel (_LIBCNT_) slice of wcs.c — the 8 pure wide
# leaves (_wcsnset/_wcsrev/wcscoll/wcspbrk/wcs{cpy,ncpy,cat,ncat}_s) →
# cabi/wcs.rs. The rest of wcs.c is #ifndef _LIBCNT_ (msvcrt-only:
# _wcsdup/wcstod/wcstombs/…) so CRT keeps the full .c; libcntpr drops it.
# The mem/mem{chr,cpy,move,set}.c entries are amd64/arm64-only (i386
# uses *_asm.s, so REMOVE_ITEM is a harmless no-op there); mem/memccpy.c
# + mem/memicmp.c + stdlib/rand_nt.c + string/ctype.c + iswctype_nt.c +
# is_wctype.c are all-arch (Wave MEM2/RAND/CTYPE, cnt-difftest-gated).
# Wave CTYPE flips the whole functional ctype cluster to cnt-ctype:
# ctype.c (tables _ctype/_wctype + _pctype/_pwctype + _isctype + narrow
# is*/_tolower/_toupper/toupper + wide isw*/towlower) + iswctype_nt.c
# (iswctype) + is_wctype.c (is_wctype).
list(REMOVE_ITEM LIBCNTPR_SOURCE
    string/_splitpath.c
    string/_wsplitpath.c
    string/strcspn.c
    string/strrev.c
    string/strset.c
    string/strstr.c
    string/strpbrk.c
    string/strspn.c
    string/wcs.c
    string/scanf.c

    string/atoi64.c
    string/atoi.c
    string/itoa.c
    string/itow.c
    string/strtoi64.c
    string/strtol.c
    string/strtoul.c
    string/strtoull.c
    string/wcstol.c
    string/wcstoul.c
    string/wtoi64.c
    string/wtoi.c
    string/wtol.c
    string/_stricmp_nt.c
    string/_strlwr_nt.c
    string/_strnicmp_nt.c
    string/_strupr_nt.c
    string/mbstowcs_nt.c
    string/tolower_nt.c
    string/toupper_nt_mb.c
    string/towupper_nt.c
    string/wcstombs_nt.c
    wstring/wcscspn.c
    wstring/wcsspn.c
    wstring/wcsstr.c
    wstring/_wcsicmp_nt.c
    wstring/_wcslwr_nt.c
    wstring/_wcsnicmp_nt.c
    wstring/_wcsupr_nt.c
    mem/memcmp.c
    mem/memchr.c
    mem/memcpy.c
    mem/memmove.c
    mem/memset.c
    mem/memccpy.c
    mem/memicmp.c
    stdlib/rand_nt.c
    stdlib/qsort.c
    stdlib/_invalid_parameter_nt.c
    string/ctype.c
    string/iswctype_nt.c
    string/is_wctype.c
    # Wave CORE-OPS: string core ops — amd64/arm64 compile these .c
    # (i386 uses *_asm.s, REMOVE_ITEM'd from the asm list above so the
    # REMOVE_ITEM here is a no-op on i386). cnt-rust provides all 20
    # (difftest-gated) → uniform Rust impl across the 3 arches.
    string/strcat.c
    string/strchr.c
    string/strcmp.c
    string/strcpy.c
    string/strlen.c
    string/strncat.c
    string/strncmp.c
    string/strncpy.c
    string/strnlen.c
    string/strrchr.c
    string/wcscat.c
    string/wcschr.c
    string/wcscmp.c
    string/wcscpy.c
    string/wcslen.c
    string/wcsncat.c
    string/wcsncmp.c
    string/wcsncpy.c
    string/wcsnlen.c
    string/wcsrchr.c
    # Wave PRINTF: the streamout/wstreamout format engine flips to
    # cnt-rust (cabi/streamout.rs). The thin variadic + va_list wrapper
    # .c (sprintf/_snprintf/_vsnprintf/_vscprintf/vsprintf/… + wide
    # twins) STAY C — they are irreducible `va_start; streamout(...);
    # va_end` shells that call the Rust engine.
    printf/streamout.c
    printf/wstreamout.c
    # Wave MATH-CLASS: IEEE-754 predicates + integer abs flip to cnt-rust
    # (cabi/math.rs) — pure bit ops, no FP arithmetic, uniform on all 3
    # arches (i386 no-SSE safe). All-arch common LIBCNTPR_MATH_SOURCE.
    math/abs.c
    math/labs.c
    math/_isnan.c
    math/_isnanf.c
    math/_finite.c
    math/_finitef.c
    math/_dclass.c
    math/_fdclass.c
    math/_dsign.c
    math/_fdsign.c
    math/_dtest.c
    math/_fdtest.c
    # Wave MATH-TIERB: `div` is pure integer (div_t of two ints, no float
    # in its signature) → cnt-rust owns it on all 3 arches.
    math/div.c
    # Wave SEARCH: bsearch + _lfind are pure pointer-walk leaves over an
    # opaque comparator (like qsort) → cnt-rust (cabi/search.rs), uniform
    # on all 3 arches. _lsearch stays C — it is #ifndef _LIBCNT_
    # (msvcrt-only) so the kernel never linked it.
    search/bsearch.c
    search/lfind.c
)

# Wave FLOATCVT-i386 (float printf): the float-formatting half of
# streamout (format_float/format_floatw) is now cnt-rust on ALL 3 arches.
# It used to stay C on i386 (no-SSE: by-value f64 RETURN was a hard ABI
# error), but the float engine is now no-SSE-safe — `cabi/streamout.rs`
# `mod ff` + `core_rs::floatcvt` use `extern "C"` x87 returns + the
# fb64/fb32 transmute shims (see core_rs/floatbits.rs). So i386 no longer
# appends printf/format_float.c + format_floatw.c — the Rust
# format_float/format_floatw own those symbols. (amd64/arm64 never
# compiled them; the C streamout.c/wstreamout.c are already Rust above.)

# Wave MATH-TIERB (float residual): the float-RETURNING shims
# (_chgsignf/scalbn/scalbnf/round/roundf/nextafter/nextafterf/nexttowardf)
# flip to cnt-rust behind the `float` feature, owned on amd64/arm64. On
# i386 the crate builds no-SSE where an `f32`/`f64` *return* is a hard ABI
# error (float *args* — Tier A's predicates — only warn), so i386 keeps
# the C exactly like format_float above. `nexttowardf` is additionally
# i386-only-C by ABI (80-bit `long double` arg). div.c is NOT here — it's
# pure integer and already removed all-arch above.
if(NOT ARCH STREQUAL "i386")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/_chgsignf.c
        math/scalbn.c
        math/scalbnf.c
        math/round.c
        math/roundf.c
        math/nextafter.c
        math/nextafterf.c
        math/nexttowardf.c
    )
endif()

# Wave MATH-TIERC (sub-wave 1): sin/cos/sqrt/sqrtf → cnt-rust (float
# feature, amd64/arm64; i386 keeps its x87 *_asm.s). sin/cos are the same
# portable math/{sin,cos}.c on both arches → one Rust impl, bit-exact.
# sqrt/sqrtf paths differ per arch (amd64 = libm_sse2 hardware; arm64 =
# the portable math/sqrt.c fast-invsqrt approximation) — cnt-rust replaces
# both with the hardware sqrt instruction.
if(NOT ARCH STREQUAL "i386")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/sin.c
        math/cos.c
    )
endif()
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/sqrt.c
        math/libm_sse2/sqrtf.c
    )
elseif(ARCH STREQUAL "arm64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/sqrt.c
        math/sqrtf.c
    )
endif()

# Wave MATH-TIERC (sub-wave 2): exp/log/log10/pow/fmod → cnt-rust (float
# feature, amd64/arm64; i386 keeps its x87 *_asm.s). These are the SAME
# portable math/arm64/*.c used on both amd64-llvm-mingw and arm64 (see
# math.cmake: the USE_LLVM_MINGW amd64 branch and the arm64 branch both
# pull them into LIBCNTPR_MATH_SOURCE → LIBCNTPR_SOURCE). exp/log/fmod are
# real algorithms; log10/pow are the upstream's rough printf stubs — all
# faithfully translated and difftest-pinned bit-exact against the C.
if(NOT ARCH STREQUAL "i386")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/arm64/exp.c
        math/arm64/log.c
        math/arm64/log10.c
        math/arm64/pow.c
        math/arm64/fmod.c
    )
endif()

# Wave MATH-TIERC (sub-wave 3): tan = sin/cos → cnt-rust (float feature,
# amd64/arm64; i386 keeps tan_asm.s). Uniform sin/cos quotient per the
# "give every arch the same thing" directive — replaces amd64's
# hand-optimized AMD-libm tan AND arm64's arm64/tan.c (already sin/cos).
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE math/libm_sse2/tan.c)
elseif(ARCH STREQUAL "arm64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE math/arm64/tan.c)
endif()

# Wave MATH-TIERC (sub-wave 4): ceil/floor/ceilf/floorf → cnt-rust (float
# feature, amd64/arm64; i386 keeps its x87 {ceil,floor}_asm). Pure
# bit-manipulation directed rounding, uniform across arches (exact IEEE).
# amd64 had the AMD-libm libm_sse2 versions; arm64 had arm64/{ceil,floor}.c
# + math/{ceilf,floorf}.c.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/ceil.c
        math/libm_sse2/ceilf.c
        math/libm_sse2/floor.c
        math/libm_sse2/floorf.c
    )
elseif(ARCH STREQUAL "arm64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/arm64/ceil.c
        math/ceilf.c
        math/arm64/floor.c
        math/floorf.c
    )
endif()

# Wave MATH-TIERC (sub-wave 5): ldexp + _logb/_logbf → cnt-rust (float
# feature, amd64/arm64; i386 keeps its C). ldexp = x*2^n (portable arm64
# stepping, replaces amd64's AMD-libm errno ldexp); _logb/_logbf = exponent
# extract. amd64 had libm_sse2/{ldexp,logb,logbf}.c; arm64 had
# arm64/{ldexp,logb}.c (logb.c holds both _logb and _logbf).
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/ldexp.c
        math/libm_sse2/logb.c
        math/libm_sse2/logbf.c
    )
elseif(ARCH STREQUAL "arm64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/arm64/ldexp.c
        math/arm64/logb.c
    )
endif()

# Wave MATH-TIERC (sub-wave 6): fabs/fabsf → cnt-rust (float feature,
# amd64/arm64; i386 keeps the portable C math/{fabs,fabsf}.c — an f64/f32
# RETURN is a hard no-SSE ABI error there). Pure sign-bit clear with the
# function-specific NaN handling (fabs returns NaN unchanged; fabsf ORs in
# the quiet bit) — bit-exact across both arches. arm64 had the C
# math/{fabs,fabsf}.c (removed here); amd64 had asm math/amd64/{fabs,fabsf}.S
# (removed from LIBCNTPR_ASM_SOURCE below, after it is composed).
if(ARCH STREQUAL "arm64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/fabs.c
        math/fabsf.c
    )
endif()

# Wave MATH-TIERC (sub-wave 7): modf + atan/atan2 → cnt-rust (float feature,
# amd64/arm64; i386 keeps its x87 atan_asm.s / atan2_asm.s — modf is not in
# i386 libcntpr at all). modf (the f64 version) is amd64-ONLY in libcntpr
# (arm64/i386 ship only modff in CRT): an exact integer/fraction split,
# faithful bit-exact port of math/libm_sse2/modf.c. atan/atan2 are the
# portable fdlibm math/arm64/{atan,atan2}.c flipped uniformly — replacing
# amd64's AMD-libm libm_sse2/{atan,atan2}.c with the same Rust algorithm.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/modf.c
        math/libm_sse2/atan.c
        math/libm_sse2/atan2.c
    )
elseif(ARCH STREQUAL "arm64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/arm64/atan.c
        math/arm64/atan2.c
    )
endif()

# Wave MATH-TIERC (sub-wave 8): exp2 + tanh → cnt-rust (float feature). Both
# amd64-ONLY in libcntpr (arm64/i386 don't ship them). Faithful ports of the
# AMD-libm math/libm_sse2/{exp2,tanh}.c (splitexp + scaleDouble + two_to_jby32
# tables, all ported to cabi/math.rs) — difftest-pinned bit-exact vs the
# genuine .c compiled against its own headers. amd64-only removal.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/exp2.c
        math/libm_sse2/tanh.c
    )
endif()

# Wave MATH-TIERC (sub-wave 9): sinh + cosh → cnt-rust (float feature). Both
# amd64-ONLY in libcntpr. Faithful ports of the AMD-libm math/libm_sse2/
# {sinh,cosh}.c (shared sinh/cosh lead/tail tables + the splitexp exp/2 path,
# all in cabi/math.rs) — difftest-pinned bit-exact. amd64-only removal.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/sinh.c
        math/libm_sse2/cosh.c
    )
endif()

# Wave MATH-TIERC (sub-wave 10): acos + asin → cnt-rust (float feature). Both
# amd64-ONLY in libcntpr. Faithful ports of the AMD-libm math/libm_sse2/
# {acos,asin}.c (shared rational R(r) + sqrt half-angle path, in cabi/math.rs)
# — difftest-pinned bit-exact. amd64-only removal.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/acos.c
        math/libm_sse2/asin.c
    )
endif()

# Wave MATH-TIERC (sub-wave 11): _hypot → cnt-rust (float feature). amd64-ONLY
# in libcntpr. Faithful port of the AMD-libm math/libm_sse2/hypot.c (the
# accurate non-FAST path: exponent scaling + head/tail split + hardware sqrt,
# in cabi/math.rs) — difftest-pinned bit-exact. amd64-only removal.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/hypot.c
    )
endif()

# Wave MATH-TIERC (sub-wave 12): sincos + modff + _hypotf → cnt-rust (float
# feature). sincos = the combined sin/cos (math/sincos.c, all-arch common);
# it reuses the Rust sin/cos so it's removed on non-i386 (like sin/cos), i386
# keeps the C. modff (exact f32 split) and _hypotf (f64-internal) are
# amd64-ONLY in libcntpr. All difftest-pinned bit-exact.
if(NOT ARCH STREQUAL "i386")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/sincos.c
    )
endif()
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/modff.c
        math/libm_sse2/hypotf.c
    )
endif()

# Wave MATH-TIERC (sub-wave 13): the f32 transcendental twins → cnt-rust
# (float feature). These are the AMD-libm math/libm_sse2/*f.c — amd64-ONLY
# in libcntpr (arm64's libcntpr has none of them; its CRT uses portable
# math/*f.c in a different archive). The Rust reuses the f64 foundation
# (splitexp/scale_double + full-precision sinh/cosh tables) plus the new
# splitexpf/scaleFloat_2 + atan(j/256)/Payne-Hanek tables. difftest-pinned
# bit-exact vs the genuine C.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/libm_sse2/acosf.c
        math/libm_sse2/asinf.c
        math/libm_sse2/atanf.c
        math/libm_sse2/atan2f.c
        math/libm_sse2/coshf.c
        math/libm_sse2/sinhf.c
        math/libm_sse2/tanf.c
        math/libm_sse2/tanhf.c
    )
endif()

# Wave MATH-TIERC (cohab cleanup): with every amd64 libm_sse2 transcendental
# now Rust (sw1–sw13), the remaining libm_sse2 C are ORPHANED support files —
# the AMD-libm *_special / lead-tail+inverse TABLES / sincos arrays /
# remainder_piby2 / two_to_jby64 tables / _handle_error / _set_statfp / the
# fma3 dispatch flag. Their only consumers were (a) the per-op *.asm
# implementations, which llvm-mingw never builds (it uses the portable
# math/{sin,cos}.c + math/arm64/*.c — themselves now Rust), and (b) the
# transcendental .c that are now REMOVE_ITEM'd above. So they form a closed
# cluster referencing only each other; nothing the kernel links pulls any of
# them (dead archive members). cnt-rust inlines _handle_error→from_bits and
# _set_statfp→no-op, so the Rust side references none of them either. Drop the
# dead weight → amd64 libcntpr math is 100% Rust. (amd64-only: i386 keeps its
# x87 C/asm math; arm64 never compiled libm_sse2.) The cold-link is the oracle.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        math/amd64/_set_statfp.c
        math/libm_sse2/_handle_error.c
        math/libm_sse2/exp_special.c
        math/libm_sse2/fma3_available.c
        math/libm_sse2/L2_by_pi_bits.c
        math/libm_sse2/log_128_lead_tail_table.c
        math/libm_sse2/log_256_lead_tail_table.c
        math/libm_sse2/log_F_inv_dword_table.c
        math/libm_sse2/log_F_inv_qword_table.c
        math/libm_sse2/log_special.c
        math/libm_sse2/log10_128_lead_tail_table.c
        math/libm_sse2/log10_256_lead_tail_table.c
        math/libm_sse2/Lsincos_array.c
        math/libm_sse2/Lsincosf_array.c
        math/libm_sse2/pow_special.c
        math/libm_sse2/remainder_piby2.c
        math/libm_sse2/sincos_special.c
        math/libm_sse2/two_to_jby64_head_tail_table.c
        math/libm_sse2/two_to_jby64_table.c
    )
endif()

list(APPEND LIBCNTPR_ASM_SOURCE
    ${LIBCNTPR_EXCEPT_ASM_SOURCE}
    ${LIBCNTPR_FLOAT_ASM_SOURCE}
    ${LIBCNTPR_MATH_ASM_SOURCE}
    ${LIBCNTPR_MEM_ASM_SOURCE}
    ${LIBCNTPR_STRING_ASM_SOURCE}
)

# Wave MATH-TIERC (sub-wave 6) amd64 half: cnt-rust's Rust fabs/fabsf own
# the symbols, so drop amd64's asm versions from the composed asm list (the
# C/arm64 half is REMOVE_ITEM'd from LIBCNTPR_SOURCE above). i386 untouched.
if(ARCH STREQUAL "amd64")
    list(REMOVE_ITEM LIBCNTPR_ASM_SOURCE
        math/amd64/fabs.S
        math/amd64/fabsf.S
    )
endif()

# Wave MATH-i386: the i386 no-SSE math flip — the last arch to keep C/asm
# math now joins amd64/arm64. cnt-rust compiles its WHOLE transcendental
# surface on the no-SSE i386 kernel build (CR4.OSFXSR unset in Phase 0):
# every exported symbol is `extern "C"`, whose `f64`/`f32` RETURN goes
# through the x87 stack (ST(0)), not an xmm register, so it never needs the
# `sse` target feature; the lone bit→float reinterpret routes through the
# fb64/fb32 `transmute` shims because `f64::from_bits` has a Rust-ABI f64
# return that WOULD need SSE (see the header in cabi/math.rs). So drop the
# x87 asm + portable C that owned these symbols on i386, exactly as
# amd64/arm64 did — cnt-rust (libcntpr_rust.a, now built unconditionally,
# no `float` feature) is the sole provider.
#
# KEEP (cnt-rust does NOT provide these): the _CIxxx MSVC intrinsic
# wrappers (ci*.c — they CALL the now-Rust libm fns: e.g. _CIsqrt→sqrt),
# the i386 libm_sse2.c SSE-register wrappers, nexttowardf.c (80-bit
# `long double` arg — cnt-rust gates nexttowardf out on x86), exp2f.c (f32
# exp2 — cnt-rust has only the f64 exp2), _invoke_matherr.c, and the
# integer/conversion asm (alldiv/allmul/…/ftol/ftol2/ftoul2 — not math
# symbols). The cold-link is the oracle.
if(ARCH STREQUAL "i386")
    list(REMOVE_ITEM LIBCNTPR_SOURCE
        # all-arch-common float shims now Rust on i386 too
        math/_chgsignf.c
        math/scalbn.c
        math/scalbnf.c
        math/round.c
        math/roundf.c
        math/nextafter.c
        math/nextafterf.c
        math/sincos.c
        # i386-specific C providers now Rust
        math/fabs.c
        math/fabsf.c
        math/i386/ldexp.c
    )
    list(REMOVE_ITEM LIBCNTPR_ASM_SOURCE
        # the 11 x87 libm asm (_sin/_cos/_tan/_atan/_atan2/_log/_log10/
        # _pow/_sqrt/_ceil/_floor) — now cnt-rust. The integer 64-bit and
        # float→int conversion asm stay (alldiv/allmul/ftol/… are kept).
        math/i386/atan_asm.s
        math/i386/atan2_asm.s
        math/i386/ceil_asm.s
        math/i386/cos_asm.s
        math/i386/floor_asm.s
        math/i386/log_asm.s
        math/i386/log10_asm.s
        math/i386/pow_asm.s
        math/i386/sin_asm.s
        math/i386/sqrt_asm.s
        math/i386/tan_asm.s
    )
endif()

# ── Wave CORE-OPS: i386 mem+string core ops — flip hand asm → cnt-rust ─
# libcntpr-ONLY (REMOVE_ITEM from libcntpr's asm list, NOT from the
# shared LIBCNTPR_{MEM,STRING}_ASM_SOURCE vars which also feed
# CRT_{MEM,STRING}_ASM_SOURCE for user-mode msvcrt). cnt-rust already
# provides all these #[no_mangle] core ops (difftest-gated); on
# amd64/arm64 cnt-rust is the sole provider (the matching .c are
# REMOVE_ITEM'd from LIBCNTPR_SOURCE below). Dropping the i386 asm gives
# every arch one Rust impl. (i386 memcpy already came from cnt-rust —
# no memcpy_asm ever existed.)
if(ARCH STREQUAL "i386")
    list(REMOVE_ITEM LIBCNTPR_ASM_SOURCE
        mem/i386/memchr_asm.s
        mem/i386/memmove_asm.s
        mem/i386/memset_asm.s
        string/i386/strcat_asm.s
        string/i386/strchr_asm.s
        string/i386/strcmp_asm.s
        string/i386/strcpy_asm.s
        string/i386/strlen_asm.s
        string/i386/strncat_asm.s
        string/i386/strncmp_asm.s
        string/i386/strncpy_asm.s
        string/i386/strnlen_asm.s
        string/i386/strrchr_asm.s
        string/i386/wcscat_asm.s
        string/i386/wcschr_asm.s
        string/i386/wcscmp_asm.s
        string/i386/wcscpy_asm.s
        string/i386/wcslen_asm.s
        string/i386/wcsncat_asm.s
        string/i386/wcsncmp_asm.s
        string/i386/wcsncpy_asm.s
        string/i386/wcsnlen_asm.s
        string/i386/wcsrchr_asm.s
    )
endif()

set_source_files_properties(${LIBCNTPR_ASM_SOURCE} PROPERTIES COMPILE_DEFINITIONS "NO_RTL_INLINES;_NTSYSTEM_;_NTDLLBUILD_;_LIBCNT_;__CRT__NO_INLINE;CRTDLL")
add_asm_files(libcntpr_asm ${LIBCNTPR_ASM_SOURCE})

# Wave SCANF: the kernel's variadic scanf shell. scanf.c (full C engine)
# is REMOVE_ITEM'd above; cnt-rust owns the engine (cnt_v*scanf in
# cabi/manual.rs, no-SSE-safe). The irreducible C-variadic public
# wrappers (sscanf/swscanf/_snscanf/_snwscanf + cnt_va_next_ptr) are the
# tiny scanf_trampoline.c — Rust can't DEFINE variadics on stable. It
# gets libcntpr's _LIBCNT_ compile defs (below) and links against the
# cnt_v*scanf in libcntpr_rust.a. (CRT/msvcrt keeps the full C scanf.c.)
list(APPEND LIBCNTPR_SOURCE
    ${CMAKE_SOURCE_DIR}/sdk/lib/rust/cnt-rust/src/cabi/scanf_trampoline.c)

add_library(libcntpr STATIC ${LIBCNTPR_SOURCE} ${libcntpr_asm})
target_compile_definitions(libcntpr
 PRIVATE    NO_RTL_INLINES
    _NTSYSTEM_
    _NTDLLBUILD_
    _LIBCNT_
    __CRT__NO_INLINE
    CRTDLL)
add_dependencies(libcntpr psdk asm)

# ── Rust integration: cnt-rust → libcntpr cohabitation swap ──────────
# cnt-rust-lib is a staticlib umbrella over cnt-rust's `cabi`
# #[no_mangle] libcntpr-replacement symbols. libcntpr feeds BOTH
# ntoskrnl and hal, so this must be a libcntpr-level link (the
# rtl-rust route serves librtl only). Mirrors librtl's rust_rtl hook.
set(RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/cnt-rust-target" cnt-rust-lib)
if(ARCH STREQUAL "i386")
    set(RUST_TARGET "i686-pc-windows-gnu")
    # i386 kernel runs no-SSE (CR4.OSFXSR unset in Phase 0). cnt-rust is
    # built WITHOUT the `float` feature — but that now gates ONLY the
    # core_rs forbid-unsafe surface (floatcvt/scanf/float-printf) which
    # genuinely can't be bitcast no-SSE. The whole math surface
    # (sin/cos/exp/… + f32 twins) is compiled unconditionally: it is
    # no-SSE-safe via `extern "C"` x87 returns + the fb64/fb32 transmute
    # shims (see cabi/math.rs), and the Wave MATH-i386 block above
    # REMOVE_ITEMs the C/asm it replaces.
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2 -C no-prepopulate-passes")
    set(CNT_RUST_FEATURES "")
elseif(ARCH STREQUAL "amd64")
    set(RUST_TARGET "x86_64-pc-windows-gnu")
    # -C no-prepopulate-passes: match i386. The per-fn cgu-split (one tiny
    # #[no_mangle] shim per cgu) lets LLVM's aggressive whole-pipeline
    # optimization miscompile isolated cnt shims on amd64 — memset's stores
    # were DCE'd into an infinite spin, _strnicmp's compare collapsed to
    # "always equal" (made DissectArcPath match ramdisk(0) for a CD path,
    # so freeldr couldn't open freeldr.ini → amd64 boot hang). Disabling the
    # prepopulated pass pipeline (as i386 already does) tames it class-wide.
    # amd64 keeps SSE (user/kernel both use it post-Phase-0).
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C no-prepopulate-passes")
    set(CNT_RUST_FEATURES "--features" "float")
elseif(ARCH STREQUAL "arm64")
    set(RUST_TARGET "aarch64-pc-windows-gnullvm")
    # Same class-wide miscompile guard as amd64 (see above); arm64 keeps NEON.
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C no-prepopulate-passes")
    set(CNT_RUST_FEATURES "--features" "float")
endif()

set(RUST_CNT_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcntpr_rust.a")
# Track EVERY .rs under the three cnt-* crates, not a hand-curated list.
# The old explicit DEPENDS named `core_rs/mod.rs` but none of its
# submodules (mem.rs, str.rs, …), so editing e.g. the memset core never
# re-ran cargo — a stale libcntpr_rust.a silently masked source fixes
# (cost a debug session: a memset miscompile fix appeared to "not take").
# CONFIGURE_DEPENDS re-globs each build so new files are picked up too.
file(GLOB_RECURSE CNT_RUST_SOURCES CONFIGURE_DEPENDS
    ${RUST_WORKSPACE_DIR}/cnt-rust/src/*.rs
    ${RUST_WORKSPACE_DIR}/cnt-rust-lib/src/*.rs
    ${RUST_WORKSPACE_DIR}/cnt-ctype/src/*.rs
)
add_custom_command(
    OUTPUT ${RUST_CNT_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${RUST_TARGET_DIR} ${RUST_EXTRA_ENV}
        cargo build --release --target ${RUST_TARGET}
            -p cnt-rust-lib ${CNT_RUST_FEATURES}
            --manifest-path ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${RUST_TARGET_DIR}/${RUST_TARGET}/release/libcnt_rust_lib.a
        ${RUST_CNT_LIB}
    DEPENDS
            ${CNT_RUST_SOURCES}
            ${RUST_WORKSPACE_DIR}/cnt-rust-lib/Cargo.toml
            ${RUST_WORKSPACE_DIR}/cnt-ctype/Cargo.toml
            ${RUST_WORKSPACE_DIR}/cnt-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust libcntpr umbrella (${RUST_TARGET})"
)
add_custom_target(rust_cnt DEPENDS ${RUST_CNT_LIB})
add_dependencies(libcntpr rust_cnt)
target_link_libraries(libcntpr PRIVATE ${RUST_CNT_LIB})
