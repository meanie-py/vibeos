
# A UCRT program needs UCRT headers. With sdk/include/ucrt gone (06f8e54c951) this
# test quietly fell back to the msvcrt-flavoured sdk/include/crt: `stdout` became
# msvcrt's `&_iob[1]`, a 32-byte msvcrt FILE handed to ucrtbase's setvbuf, which
# took the bytes after it as the stream's CRITICAL_SECTION -- the first call in
# main() died with 0xC0000008 in RtlLeaveCriticalSection. Same recipe as the
# ucrtbase winetest: Wine's CRT headers under _UCRT, with ReactOS's intrin.h.
# Two calls: each BEFORE prepends, so include/ must come second to end up first.
include_directories(BEFORE ${REACTOS_SOURCE_DIR}/dll/win32/msvcrt/include)
include_directories(BEFORE ${CMAKE_CURRENT_SOURCE_DIR}/include)

list(APPEND SOURCE
    ../crt/_finite.c
    ../crt/_isnan.c
    ../crt/_wsystem.c
    ../crt/acos.c
    ../crt/asin.c
    ../crt/atan.c
    ../crt/ceil.c
    ../crt/cos.c
    ../crt/exp.c
    ../crt/log.c
    ../crt/log10.c
    ../crt/round.c
    ../crt/setjmp.c
    ../crt/sin.c
    ../crt/sqrt.c
    ../crt/system.c
    ../crt/tan.c
    ../crt/wctomb.c
    testlist.c
)

if(ARCH STREQUAL "i386")
    add_asm_files(ucrtbase_apitest_asm
        ../crt/i386/setjmp_helper.s
    )
elseif(ARCH STREQUAL "amd64")
    add_asm_files(ucrtbase_apitest_asm
        ../crt/amd64/setjmp_helper.s
    )
endif()

# The setjmp test checks that longjmp runs the __finally blocks it unwinds through. Under the
# default _USE_DUMMY_PSEH those blocks are dead code, so the check could only fail. Give
# that one TU real SEH the way the delayimp tests do: _USE_NATIVE_SEH plus the
# -windows-msvc retarget, without which clang will not lower an i386 frame-based __try.
if(ARCH STREQUAL "i386")
    set_source_files_properties(../crt/setjmp.c PROPERTIES
        COMPILE_DEFINITIONS _USE_NATIVE_SEH
        COMPILE_OPTIONS "--target=i686-pc-windows-msvc;-fasync-exceptions")
endif()

add_executable(ucrtbase_apitest ${SOURCE} ${ucrtbase_apitest_asm})
set_module_type(ucrtbase_apitest win32cui)
target_compile_definitions(ucrtbase_apitest PRIVATE USE_WINE_TODOS _UCRT TEST_UCRTBASE TEST_DLL_NAME="ucrtbase.dll")

target_link_libraries(ucrtbase_apitest ${PSEH_LIB} chkstk)
if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    # Add this manually here, so it can link to ucrtbase.dll and vcstartup.
    # GCC-only: under clang the `libgcc` interface target carries libwinpthread,
    # which links libmsvcrt + msvcrtex -- msvcrt's startup and imports in a UCRT
    # exe. compiler-rt's builtins reach every link through add_link_options.
    target_link_libraries(ucrtbase_apitest libgcc)
endif()

add_importlibs(ucrtbase_apitest ucrtbase kernel32 ntdll)
add_rostests_file(TARGET ucrtbase_apitest)
