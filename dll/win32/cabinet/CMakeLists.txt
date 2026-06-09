
add_definitions(
    -DHAVE_ZLIB)

include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/libs/zlib)

spec2def(cabinet.dll cabinet.spec ADD_IMPORTLIB)

# NOTE: cabinet_main.c / fci.c / fdi.c / stubs.c have a Rust translation in
# sdk/lib/rust/cabinet-rust (kept on disk per the never-delete rule), but it is
# NOT yet link-complete (the LZX/QTM/MSZIP decompression engine + FDICopy are
# still being hand-ported). Until cabinet-rust links, cabinet.dll is built from
# C so the livecd/smoke is not blocked. Re-wire to the Rust staticlib cohab
# (see git history of this file + dll/win32/cabinet/rust_cohab_stub.c) once the
# decomp engine port is finished.

list(APPEND SOURCE
    cabinet_main.c
    fci.c
    fdi.c)

list(APPEND PCH_SKIP_SOURCE
    stubs.c)

add_library(cabinet MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    cabinet.rc
    ${CMAKE_CURRENT_BINARY_DIR}/cabinet.def)

set_module_type(cabinet win32dll)
target_link_libraries(cabinet wine zlib)
add_importlibs(cabinet msvcrt kernel32 ntdll)
add_pch(cabinet precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET cabinet DESTINATION reactos/system32 FOR all)
set_wine_module(cabinet)
