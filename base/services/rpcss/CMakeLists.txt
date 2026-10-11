
add_rpc_files(server
    ${REACTOS_SOURCE_DIR}/sdk/include/wine/epm.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/wine/irot.idl
    ${REACTOS_SOURCE_DIR}/sdk/include/wine/irpcss.idl)


list(APPEND SOURCE
    # epmp.c  # replaced by Rust (sdk/lib/rust/rpcss-rust/src/rpcss/epmp.rs)
    # irotp.c  # replaced by Rust (sdk/lib/rust/rpcss-rust/src/rpcss/irotp.rs)
    # rpcss_main.c  # replaced by Rust (sdk/lib/rust/rpcss-rust/src/rpcss/rpcss_main.rs)
    # setup.c  # replaced by Rust (sdk/lib/rust/rpcss-rust/src/rpcss/setup.rs)
    rust_cohab_stub.c
    precomp.h
    ${CMAKE_CURRENT_BINARY_DIR}/epm_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/irot_s.c
    ${CMAKE_CURRENT_BINARY_DIR}/irpcss_s.c)

include_directories(${REACTOS_BINARY_DIR}/sdk/include/wine)
# Every translation unit is Rust now (rpcss-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(rpcss_rust rpcss-rust "" TRUE)
add_executable(rpcss ${SOURCE} rpcss.rc)
add_dependencies(rpcss rust_rpcss_rust)
target_link_libraries(rpcss ${rpcss_rust_LIB})

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(rpcss PRIVATE "-Wno-cast-calling-convention")
endif()

target_link_libraries(rpcss ${PSEH_LIB} wine)
set_module_type(rpcss win32cui UNICODE)
add_importlibs(rpcss advapi32 rpcrt4 msvcrt kernel32 ntdll)
add_pch(rpcss precomp.h SOURCE)
add_cd_file(TARGET rpcss DESTINATION reactos/system32 FOR all)
