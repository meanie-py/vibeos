
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

spec2def(basesrv.dll basesrv.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    appcompat.c
    dosdev.c
    init.c
    proc.c
    sndsntry.c
    vdm.c
    nls.c
    basesrv.h)

add_library(basesrv MODULE
    ${SOURCE}
    basesrv.rc
    ${CMAKE_CURRENT_BINARY_DIR}/basesrv.def)

set_module_type(basesrv win32dll UNICODE ENTRYPOINT DllMain 12)
#############################################
## HACK FOR MSVC COMPILATION WITH win32dll ##
set_subsystem(basesrv console)
################# END  HACK #################

target_link_libraries(basesrv ${PSEH_LIB})
add_importlibs(basesrv csrsrv ntdll)
add_pch(basesrv basesrv.h SOURCE)
add_dependencies(basesrv bugcodes xdk)
add_cd_file(TARGET basesrv DESTINATION reactos/system32 FOR all)

# Cohabitation flip deferred (task #18). 2026-05-26 debug session
# uncovered:
#   1. `CSR_SERVER_DLL_FULL` Rust mirror was missing the Length/Name/
#      ServerHandle/ServerId/Unknown preamble — corrupted ApiBase
#      writes. FIXED. The auxiliary `CSR_SERVER_DLL_STRUCT` (a
#      2-field fake) was *also* used to read/write ProcessHeap +
#      SharedSection at wrong offsets. FIXED — heap now comes from
#      RtlGetProcessHeap() (TEB inline) and SharedSection from the
#      proper struct offset (0x3C with CSR_DBG=on).
#   2. `BaseServerApiNameTable` was `#[cfg(feature = "CSR_DBG")]`-
#      gated, so it didn't compile by default — but the C build has
#      DBG=1 → CSR_DBG on → NameTable slot is real. FIXED.
#   3. `BASE_STATIC_SERVER_DATA` Rust mirror has completely wrong
#      field order (TermsrvClientTimeZoneId first vs WindowsDirectory
#      first in C) and is missing WindowsMajorVersion / NlsUserInfo /
#      tziTermsrvClientTimeZone / ktTermsrvClientBias / etc. C says
#      sizeof = 0x1AC8. NOT FIXED — every kernel32 read of
#      BaseStaticServerData->X gets garbage; cohab boot now reaches
#      kernel32 DllMain SUCCESS but hangs ~7 DLLs later (msacm32 or
#      similar) trying to consume static-server-data fields.
# The Rust source improvements (CSR_SERVER_DLL layout, no_mangle on
# data globals, NameTable export) ARE landed; the cmake flip stays C
# until #18 completes BASE_STATIC_SERVER_DATA layout repair.
