
# enum/drivers.c  # replaced by Rust (sdk/lib/rust/epsapi-rust/src/epsapi/drivers.rs)
# enum/modules.c  # replaced by Rust (sdk/lib/rust/epsapi-rust/src/epsapi/modules.rs)
# enum/processes.c  # replaced by Rust (sdk/lib/rust/epsapi-rust/src/epsapi/processes.rs)

# Every translation unit is Rust now (epsapi-rust, `rustconv port`); the C stays on
# disk. The target is an INTERFACE library over the Rust archive (`cargo-cell --interface`).
rust_staticlib_variant(epsapi_rust epsapi-rust "" TRUE)
add_library(epsapi INTERFACE)
add_dependencies(epsapi rust_epsapi_rust)
target_link_libraries(epsapi INTERFACE ${epsapi_rust_LIB})
add_dependencies(epsapi psdk)
