
spec2def(wldap32.dll wldap32.spec ADD_IMPORTLIB)

# wldap32.dll — Rust cohabitation flip (FULL: all 20 compiled src *.c → wldap32-rust).
#
# In THIS build wldap32 is a STUB DLL. wine's config.h carries `/* #undef HAVE_LDAP */`
# (ReactOS vendors no OpenLDAP), so every entry point compiles to
# `return WLDAP32_LDAP_NOT_SUPPORTED`. The translation bought BOTH arms and carries the C's
# eight `HAVE_LDAP*` probes across as cargo features — all off by default, exactly matching
# config.h. Turn `have_ldap` on the day libldap is vendored and the real marshalling is there,
# already translated. See rustconv/cfg_gate.py and the crate's Cargo.toml.
#
# TWO bindgen islands, and they must never merge: the 19 PCH'd TUs compile against
# "winldap_private.h" while ber.c (the add_pch SKIP source) compiles against <winldap.h>, and
# the two declare BerElement/berval/LDAPControlA/... incompatibly (field manual §9.108).
set(WLDAP32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(WLDAP32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(WLDAP32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(WLDAP32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_storage_target(WLDAP32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-wldap32-target" wldap32-rust)
set(WLDAP32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libwldap32_rust.a")
rust_crate_sources(WLDAP32_RUST_SRCS "${WLDAP32_RUST_WORKSPACE_DIR}" wldap32-rust)
add_custom_command(
    OUTPUT ${WLDAP32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${WLDAP32_RUST_TARGET_DIR}
        cargo build --release --target ${WLDAP32_RUST_TARGET}
            -p wldap32-rust
            --manifest-path ${WLDAP32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy_if_different
        ${WLDAP32_RUST_TARGET_DIR}/${WLDAP32_RUST_TARGET}/release/libwldap32_rust.a
        ${WLDAP32_RUST_LIB}
    DEPENDS ${WLDAP32_RUST_SRCS}
            ${WLDAP32_RUST_WORKSPACE_DIR}/wldap32-rust/Cargo.toml
    WORKING_DIRECTORY ${WLDAP32_RUST_WORKSPACE_DIR}
    COMMENT "Building wldap32-rust (${WLDAP32_RUST_TARGET})"
)
add_custom_target(rust_wldap32 DEPENDS ${WLDAP32_RUST_LIB})

# All 20 compiled src *.c → wldap32-rust (src/wldap32/*.rs), kept on disk per the
# never-delete rule. wldap32.h's 62 `static inline` marshalling helpers — which no .c
# defined, because every TU inlined its own copy — became the shared `helpers` module.
#   add.c bind.c compare.c control.c delete.c dn.c error.c extended.c init.c main.c
#   misc.c modify.c modrdn.c option.c page.c parse.c rename.c search.c value.c  → wldap32-rust
#   ber.c (PCH_SKIP)                                                            → wldap32-rust
# list(APPEND SOURCE
#     add.c
#     bind.c
#     compare.c
#     control.c
#     delete.c
#     dn.c
#     error.c
#     extended.c
#     init.c
#     main.c
#     misc.c
#     modify.c
#     modrdn.c
#     option.c
#     page.c
#     parse.c
#     rename.c
#     search.c
#     value.c)
# list(APPEND PCH_SKIP_SOURCE
#     ber.c)

# ber_printf/ber_scanf are CDECL VARARGS exports and stable Rust can neither define a
# C-variadic fn nor do va_arg (rust-lang/rust#44930). rust_varargs_shim.c is a 3-line
# trampoline per entry point that hands `&ap` straight back to the Rust `*_v` twin in
# ber.rs — a shim for a language gap, NOT a translation reverted to C.
# Template: sdk/lib/rust/varargs-rust/rust_varargs_shim.c.template.
list(APPEND SOURCE
    rust_varargs_shim.c
    ${CMAKE_CURRENT_BINARY_DIR}/wldap32_stubs.c)

add_library(wldap32 MODULE
    ${SOURCE}
    wldap32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/wldap32.def)

set_module_type(wldap32 win32dll)
# libwldap32_rust.a FIRST so its #[no_mangle] symbols win.
target_link_libraries(wldap32 ${WLDAP32_RUST_LIB})
add_dependencies(wldap32 rust_wldap32)
add_importlibs(wldap32 user32 msvcrt kernel32 ntdll)
# add_pch dropped with precomp.h: the only C left is rust_varargs_shim.c (stdarg.h only) and
# the generated wldap32_stubs.c, neither of which includes it.
# `wine` dropped with the C: the Rust prelude has no ros_dbg_log extern — TRACE/ERR/WARN/FIXME
# are no-op macro_rules!, never out-of-line fns (AGENTS.md: an out-of-line no-op is an
# optimization barrier that uncloaks latent ABI mismatches).
# set_wine_module_FIXME dropped for the same reason: it only added wine include dirs and
# __WINESRC__ to C compilation, and no wine-derived C compiles here any more.
add_cd_file(TARGET wldap32 DESTINATION reactos/system32 FOR all)
