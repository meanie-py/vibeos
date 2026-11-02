
list(APPEND SOURCE
    # enum/drivers.c  # replaced by Rust (sdk/lib/rust/epsapi-rust/src/epsapi/drivers.rs)
    # enum/modules.c  # replaced by Rust (sdk/lib/rust/epsapi-rust/src/epsapi/modules.rs)
    # enum/processes.c  # replaced by Rust (sdk/lib/rust/epsapi-rust/src/epsapi/processes.rs)
    rust_cohab_stub.c
    enum/precomp.h)

# Every translation unit is Rust now (epsapi-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(epsapi_rust epsapi-rust "" TRUE)
add_library(epsapi ${SOURCE})
add_dependencies(epsapi rust_epsapi_rust)
target_link_libraries(epsapi ${epsapi_rust_LIB})
add_pch(epsapi enum/precomp.h SOURCE)
add_dependencies(epsapi psdk)
