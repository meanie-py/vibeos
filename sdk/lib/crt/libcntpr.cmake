
list(APPEND LIBCNTPR_SOURCE
    ${LIBCNTPR_EXCEPT_SOURCE}
    ${LIBCNTPR_FLOAT_SOURCE}
    ${LIBCNTPR_MATH_SOURCE}
    ${LIBCNTPR_MBSTRING_SOURCE}
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
# wctype/winesup/wcs/bsearch/lfind/mbstrlen/fltused (UNCOVERED) +
# atol (_atoldbl) + scanf (variadic) — PARTIAL. wctype.c is only the
# dead wine_wctype_table data (no funcs; unused by the linked iswctype).
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
)

# Wave PRINTF (float residual): the float-formatting half of streamout
# (format_float/format_floatw) is irreducibly floating-point. On i386
# the kernel builds Rust no-SSE (no by-value f64 ABI), so it stays C —
# the extracted printf/format_float.c + format_floatw.c, which the Rust
# `streamout` calls as externs. On amd64/arm64 cnt-rust's Rust
# `format_float`/`format_floatw` (behind the `float` feature) own those
# symbols, so these .c are NOT compiled there.
if(ARCH STREQUAL "i386")
    list(APPEND LIBCNTPR_SOURCE
        printf/format_float.c
        printf/format_floatw.c
    )
endif()

list(APPEND LIBCNTPR_ASM_SOURCE
    ${LIBCNTPR_EXCEPT_ASM_SOURCE}
    ${LIBCNTPR_FLOAT_ASM_SOURCE}
    ${LIBCNTPR_MATH_ASM_SOURCE}
    ${LIBCNTPR_MEM_ASM_SOURCE}
    ${LIBCNTPR_STRING_ASM_SOURCE}
)

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
set(RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/cnt-rust-target")
if(ARCH STREQUAL "i386")
    set(RUST_TARGET "i686-pc-windows-gnu")
    # i386 kernel runs no-SSE (CR4.OSFXSR unset in Phase 0); cnt-rust
    # built WITHOUT the `float` feature (no by-value f64 ABI).
    set(RUST_EXTRA_ENV "RUSTFLAGS=-C target-feature=-sse,-sse2 -C no-prepopulate-passes")
    set(CNT_RUST_FEATURES "")
elseif(ARCH STREQUAL "amd64")
    set(RUST_TARGET "x86_64-pc-windows-gnu")
    set(RUST_EXTRA_ENV "")
    set(CNT_RUST_FEATURES "--features" "float")
elseif(ARCH STREQUAL "arm64")
    set(RUST_TARGET "aarch64-pc-windows-gnullvm")
    set(RUST_EXTRA_ENV "")
    set(CNT_RUST_FEATURES "--features" "float")
endif()

set(RUST_CNT_LIB "${CMAKE_CURRENT_BINARY_DIR}/libcntpr_rust.a")
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
            ${RUST_WORKSPACE_DIR}/cnt-rust-lib/src/lib.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust-lib/Cargo.toml
            ${RUST_WORKSPACE_DIR}/cnt-rust/src/lib.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust/src/cabi/mod.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust/src/cabi/generated.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust/src/cabi/manual.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust/src/cabi/streamout.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust/src/core_rs/floatcvt.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust/src/core_rs/mod.rs
            ${RUST_WORKSPACE_DIR}/cnt-ctype/src/lib.rs
            ${RUST_WORKSPACE_DIR}/cnt-ctype/Cargo.toml
            ${RUST_WORKSPACE_DIR}/cnt-rust-lib/src/lib.rs
            ${RUST_WORKSPACE_DIR}/cnt-rust/Cargo.toml
            ${RUST_WORKSPACE_DIR}/Cargo.toml
    COMMENT "Building Rust libcntpr umbrella (${RUST_TARGET})"
)
add_custom_target(rust_cnt DEPENDS ${RUST_CNT_LIB})
add_dependencies(libcntpr rust_cnt)
target_link_libraries(libcntpr PRIVATE ${RUST_CNT_LIB})
