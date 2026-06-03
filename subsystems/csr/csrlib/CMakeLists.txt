
add_definitions(-D_NTSYSTEM_)

include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys/csr)

# Rust cohabitation (smss-csrss-rust, csrlib_only): the 16 client Csr*
# exports are provided by libcsrlib_rust.a, built + linked in
# dll/ntdll/CMakeLists.txt. The .c stay on disk (never delete the C);
# csrlib_rust_stub.c keeps this static-lib target valid (a direct .obj
# defining the exports would block the linker from pulling the Rust
# archive). See project_smlib_csrlib_client_flip_deferred (2026-06-02).
list(APPEND SOURCE
    # api.c     — replaced by Rust (smss-csrss-rust/src/csrlib/api.rs)
    # capture.c — replaced by Rust (smss-csrss-rust/src/csrlib/capture.rs)
    # connect.c — replaced by Rust (smss-csrss-rust/src/csrlib/connect.rs)
    csrlib_rust_stub.c)

add_library(csrlib ${SOURCE})
add_pch(csrlib csrlib.h SOURCE)
add_dependencies(csrlib psdk)
