list(APPEND SOURCE
    # cicarray.cpp  # replaced by Rust (sdk/lib/rust/cicero-rust/src/cicero/cicarray_cpp_h.rs)
    # cicbase.cpp  # replaced by Rust (sdk/lib/rust/cicero-rust/src/cicero/cicbase_cpp_h.rs)
    # ciccaret.cpp  # replaced by Rust (sdk/lib/rust/cicero-rust/src/cicero/ciccaret_cpp_h.rs)
    # cicfmap.cpp  # replaced by Rust (sdk/lib/rust/cicero-rust/src/cicero/cicfmap_cpp_h.rs)
    # cicimc.cpp  # replaced by Rust (sdk/lib/rust/cicero-rust/src/cicero/cicimc_cpp_h.rs)
    # cicreg.cpp  # replaced by Rust (sdk/lib/rust/cicero-rust/src/cicero/cicreg_cpp_h.rs)
    # cicuif.cpp  # replaced by Rust (sdk/lib/rust/cicero-rust/src/cicero/cicuif_cpp_h.rs)
    rust_cohab_stub.cpp)

# Every translation unit is Rust now (cicero-rust, `rustconv port`), the tagged operator
# new/delete included (exported under their Itanium names); the C++ stays on disk. cicero-rust
# is an rlib with no archive of its own: msctf, msutb, msctfime and ctfmon each link it into
# their own Rust staticlib, so each binary carries exactly one copy. This target keeps a stub
# TU and the include directory the consumers' headers still name.
add_library(cicero STATIC ${SOURCE})
target_compile_definitions(cicero PRIVATE UNICODE _UNICODE)
add_dependencies(cicero psdk reactos_idls)
target_include_directories(cicero INTERFACE
    $<$<COMPILE_LANGUAGE:CXX>:${CMAKE_CURRENT_SOURCE_DIR}>)
