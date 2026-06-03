
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

# Rust cohabitation (smss-csrss-rust): the 7 SM-client Sm* exports are
# provided by the Rust translation (smss-csrss-rust/src/smlib/smclient.rs).
# The three consumers each pull it from a Rust archive — csrsrv.dll +
# smss.exe from their existing archives, ntdll_vista.dll from the new
# smlib_only libsmlib_rust.a (dll/ntdll/nt_0600/CMakeLists.txt). The .c
# stays on disk (never delete the C); smlib_rust_stub.c keeps this
# static-lib target valid. See project_smlib_csrlib_client_flip_deferred
# (2026-06-02).
list(APPEND SOURCE
    # smclient.c — replaced by Rust (smss-csrss-rust/src/smlib/smclient.rs)
    smlib_rust_stub.c
    precomp.h)

add_library(smlib ${SOURCE})
add_pch(smlib precomp.h SOURCE)
add_dependencies(smlib psdk)
