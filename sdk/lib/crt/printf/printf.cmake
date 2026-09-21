# NOTE (upstream-sync 2026-09-20): upstream's "Stop using non-conforming swprintf /
# vswprintf" removed the tree-wide -D_CRT_NON_CONFORMING_SWPRINTFS from the top-level
# CMakeLists and fixed every call site, so <stdio.h> now always declares the C99 3-arg
# swprintf. We took that change. But swprintf.c / vswprintf.c ARE the non-conforming
# 2-arg engines behind the msvcrt/ntdll exports of those names, so they alone still need
# the old declaration — hence the per-source define at the bottom of this file. Upstream
# instead deleted swprintf.c and renamed vswprintf.c to _vswprintf.c; we keep ours,
# because our cargo modules (rosload-cargo, ntdll-cargo) replay these exact
# compile_commands entries and reactos-link hard-fails on a source with no cc entry.


list(APPEND LIBCNTPR_PRINTF_SOURCE
    printf/_snprintf.c
    printf/_snwprintf.c
    printf/_swprintf.c
    printf/_vscprintf.c
    printf/_vscwprintf.c
    printf/_vsnprintf.c
    printf/_vsnwprintf.c
    printf/sprintf.c
    printf/streamout.c
    printf/swprintf.c
    printf/vprintf.c
    printf/vsprintf.c
    printf/vswprintf.c
    printf/wstreamout.c
)

list(APPEND CRT_PRINTF_SOURCE
    ${LIBCNTPR_PRINTF_SOURCE}
    printf/_cprintf.c
    printf/_cwprintf.c
    printf/_scprintf.c
    printf/_scwprintf.c
    printf/_snprintf_s.c
    printf/_snwprintf_s.c
    printf/_vcprintf.c
    printf/_vcwprintf.c
    printf/_vsnprintf_s.c
    printf/_vsnwprintf_s.c
    printf/_vsprintf_p.c
    printf/fprintf.c
    printf/fprintf_s.c
    printf/fwprintf.c
    printf/fwprintf_s.c
    printf/printf.c
    printf/printf_s.c
    printf/sprintf_s.c
    printf/swprintf_s.c
    printf/vfprintf.c
    printf/vfprintf_s.c
    printf/vfwprintf.c
    printf/vfwprintf_s.c
    printf/vprintf_s.c
    printf/vsprintf_s.c
    printf/vswprintf_s.c
    printf/vwprintf.c
    printf/vwprintf_s.c
    printf/wprintf.c
    printf/wprintf_s.c
)
