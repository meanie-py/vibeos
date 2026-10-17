
spec2def(netio.sys netio.spec ADD_IMPORTLIB)

# NETIO.SYS is a NT 6 driver
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

# Including AFD is needed by the TDI helpers.
# They have been part of AFD before.

include_directories(
    BEFORE include
    ../afd/include
    ../tdihelpers/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/drivers)

list(APPEND SOURCE
    # netio.c  # replaced by Rust (sdk/lib/rust/netio-rust/src/netio/netio.rs)
    rust_cohab_stub.c
    ../tdihelpers/tdi.c
    ../tdihelpers/tdiconn.c
    ${CMAKE_CURRENT_BINARY_DIR}/netio.def)

# Every translation unit is Rust now (netio-rust, `rustconv port`) except tdi.c, tdiconn.c,
# kept C by `port derive --keep-c`; the C stays on disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(netio_rust netio-rust "" TRUE)
add_library(netio MODULE ${SOURCE})
add_dependencies(netio rust_netio_rust)
target_link_libraries(netio ${netio_rust_LIB})
target_link_libraries(netio ${PSEH_LIB})
set_module_type(netio kernelmodedriver)
add_importlibs(netio ntoskrnl hal)
add_cd_file(TARGET netio DESTINATION reactos/system32/drivers FOR all)
add_registry_inf(netio_reg.inf)
