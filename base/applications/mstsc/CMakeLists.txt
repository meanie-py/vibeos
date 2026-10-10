
# The RDP engine is rdesktop's (plus one xrdp file for the TLS calls) and
# lives in vendor/rdesktop; the Win32 front end below is ReactOS's.
include_directories(${REACTOS_SOURCE_DIR}/vendor/rdesktop)

list(APPEND SOURCE
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/asn.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/bitmap.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/bsops.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/cache.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/channels.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/iso.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/licence.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/mcs.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/mppc.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/orders.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/pstcache.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/rdp5.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/rdp.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/secure.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/ssl_calls.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/tcp.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/uimain.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/win32.c
    # connectdialog.c  # replaced by Rust (sdk/lib/rust/mstsc-rust/src/mstsc/connectdialog.rs)
    # settings.c  # replaced by Rust (sdk/lib/rust/mstsc-rust/src/mstsc/settings.rs)
    rust_cohab_stub.c
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/bsops.h
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/constants.h
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/orders.h
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/proto.h
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/rdesktop.h
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/types.h
    ${REACTOS_SOURCE_DIR}/vendor/rdesktop/uimain.h
    resource.h
    precomp.h)

add_definitions(-DWITH_SSL)
file(GLOB mstsc_rc_deps res/*.*)
add_rc_deps(rdc.rc ${mstsc_rc_deps})
# Every translation unit is Rust now (mstsc-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(mstsc_rust mstsc-rust "" TRUE)
add_executable(mstsc ${SOURCE} rdc.rc)
add_dependencies(mstsc rust_mstsc_rust)
target_link_libraries(mstsc ${mstsc_rust_LIB})
set_module_type(mstsc win32gui UNICODE)
target_compile_definitions(mstsc PRIVATE _CRT_NON_CONFORMING_SWPRINTFS)
add_importlibs(mstsc user32 gdi32 comctl32 ws2_32 crypt32 secur32 advapi32 shell32 ole32 comdlg32 msvcrt kernel32)
add_pch(mstsc precomp.h SOURCE)
add_cd_file(TARGET mstsc DESTINATION reactos/system32 FOR all)
