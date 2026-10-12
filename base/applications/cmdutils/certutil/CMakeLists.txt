
list(APPEND SOURCE
    # asn.cpp  # replaced by Rust (sdk/lib/rust/certutil-rust/src/certutil/asn_cpp_h.rs)
    # certutil.cpp  # replaced by Rust (sdk/lib/rust/certutil-rust/src/certutil/certutil_cpp_h.rs)
    # hashfile.cpp  # replaced by Rust (sdk/lib/rust/certutil-rust/src/certutil/hashfile_cpp_h.rs)
    rust_cohab_stub.cpp
    precomp.h)

# certutil-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(CERTUTIL_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(CERTUTIL_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (certutil-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(certutil_rust certutil-rust "" TRUE ${CERTUTIL_RUST_ENV})
add_executable(certutil ${SOURCE})
add_dependencies(certutil rust_certutil_rust)
target_link_libraries(certutil ${certutil_rust_LIB})
set_module_type(certutil win32cui UNICODE)
target_link_libraries(certutil conutils)
add_importlibs(certutil crypt32 advapi32 msvcrt kernel32)
add_pch(certutil precomp.h SOURCE)
add_cd_file(TARGET certutil DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
