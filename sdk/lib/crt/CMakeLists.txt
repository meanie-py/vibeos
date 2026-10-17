
include_directories(include)
#include_directories(.)

add_definitions(-D_CRTBLD)

include(conio/conio.cmake)
include(direct/direct.cmake)
include(except/except.cmake)
include(float/float.cmake)
include(math/math.cmake)
include(mbstring/mbstring.cmake)
include(mem/mem.cmake)
include(misc/misc.cmake)
include(printf/printf.cmake)
include(process/process.cmake)
include(search/search.cmake)
include(startup/startup.cmake)
include(stdio/stdio.cmake)
include(stdlib/stdlib.cmake)
include(string/string.cmake)
include(time/time.cmake)
include(wine/wine.cmake)
include(wstring/wstring.cmake)

# The static `crt` library (crt.cmake): nothing links it -- its one consumer,
# modules/rostests/apitests/crt/static_crt_apitest.cmake, is itself commented out -- and
# no build target pulled it, so its ~720 TUs only showed up in compile_commands.json,
# where scanners and coverage counts took them for shipping code (2026-10-16).
#include(crt.cmake)
include(crtmath.cmake)
include(libcntpr.cmake)
include(msvcrtex.cmake)
include(oldnames.cmake)

# user32's wsprintf family is Rust (user32-wsprintf-rust, 2026-10-04): cnt-rust's
# printf engine compiled in its _USER32_WSPRINTF configuration, the wv[s|sn]printf
# string-stream wrappers, wsprintfA/W as naked variadic thunks, and the NT-flavoured
# string helpers. The C it replaces, which nothing compiles any more:
#   printf/streamout.c printf/wstreamout.c printf/wsprintfA.c printf/wsprintfW.c
#   printf/wvsprintfA.c printf/wvsprintfW.c printf/wvsnprintfA.c printf/wvsnprintfW.c
#   string/strnlen.c string/wcsnlen.c string/mbstowcs_nt.c string/wcstombs_nt.c
rust_staticlib_variant(user32_wsprintf_rust user32-wsprintf-rust "" TRUE
    ${MSVCRTEX_RUST_WORKSPACE}/cnt-rust/src/cabi/streamout.rs)
add_library(user32_wsprintf INTERFACE)
add_dependencies(user32_wsprintf rust_user32_wsprintf_rust)
target_link_libraries(user32_wsprintf INTERFACE ${user32_wsprintf_rust_LIB})

add_library(dbgrpt misc/dbgrpt.cpp)
target_link_libraries(dbgrpt chkstk ${PSEH_LIB})
target_compile_definitions(dbgrpt PRIVATE
    _MT
    _DEBUG)
add_dependencies(dbgrpt psdk asm)
