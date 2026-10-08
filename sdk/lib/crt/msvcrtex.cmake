
include_directories(include/internal/mingw-w64)

list(APPEND MSVCRTEX_SOURCE
    ${CRT_STARTUP_SOURCE}
    math/sincos.c
    misc/fltused.c
    misc/isblank.c
    misc/iswblank.c
    misc/ofmt_stub.c
    # recalloc.c is NEW upstream (CORE-20596): it defines recalloc AND the
    # __imp__recalloc / _imp___recalloc thunk variable, which is what satisfies the
    # `__declspec(dllimport) _recalloc` that <malloc.h> gives every caller. urlmon's
    # sec_mgr.c started calling it in this merge.
    recalloc.c
    stdio/acrt_iob_func.c)

if(DLL_EXPORT_VERSION LESS 0x600)
    list(APPEND MSVCRTEX_SOURCE
        misc/dbgrpt.cpp
        stdlib/_invalid_parameter.c
        stdlib/rand_s.c
        wstring/mbrtowc.c
        wstring/wcrtomb.c
    )
endif()

if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Clang performs some optimizations requiring those funtions
    list(APPEND MSVCRTEX_SOURCE
        math/round.c
        math/roundf.c
        math/exp2.c
        math/exp2f.c
        )
endif()

if(ARCH STREQUAL "i386")
    # Clang wants __aulldiv for its optimizations
    list(APPEND MSVCRTEX_ASM_SOURCE
        except/i386/chkstk_asm.s
        except/i386/chkstk_ms.s
        math/i386/alldiv_asm.s
        math/i386/aulldiv_asm.s
        )
    if(CMAKE_C_COMPILER_ID STREQUAL "Clang")
        list(APPEND MSVCRTEX_ASM_SOURCE
            math/i386/ceilf.S
            math/i386/floorf.S)
        list(APPEND MSVCRTEX_SOURCE
            math/i386/sqrtf.c)
    endif()
elseif(ARCH STREQUAL "amd64")
    list(APPEND MSVCRTEX_ASM_SOURCE
        except/amd64/chkstk_ms.s)
endif()

# msvcrtex is Rust (2026-10-04): crt-startup-rust + crt0-rust + crt-dllmain-rust, the
# translation cargo modules already linked through reactos-link's .msvcrtex_crt(). The C
# lists above stay as the record of what it replaced; nothing compiles them.
#
# The C archive held every personality at once (crtexe.o, wcrtexe.o, crtdll.o, crt0_c.o,
# dllmain.o, ...) and the linker pulled whichever the entry point reached. The Rust crate
# picks ONE by cargo feature, so it is built once per personality and each consumer links
# the one matching its entry point: set_entrypoint (gcc.cmake) records REACTOS_CRT_KIND
# (dll / exe / wexe / bare), and the generator expressions below read it on the CONSUMING
# target. crt0-rust (default main -> WinMain bridge) and crt-dllmain-rust (default DllMain)
# are separate archives after the module's own objects, so a module's own definition wins.
#
set(MSVCRTEX_RUST_WORKSPACE "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_staticlib_variant(msvcrtex_dll crt-startup-rust "crtdll" FALSE)
rust_staticlib_variant(msvcrtex_exe crt-startup-rust "crtexe" FALSE)
rust_staticlib_variant(msvcrtex_wexe crt-startup-rust "wcrtexe" FALSE)
rust_staticlib_variant(msvcrtex_bare crt-startup-rust "" FALSE)
rust_staticlib_variant(msvcrtex_dllmain crt-dllmain-rust "" TRUE)
rust_staticlib_variant(msvcrtex_crt0 crt0-rust "ansi" FALSE)
rust_staticlib_variant(msvcrtex_wcrt0 crt0-rust "wide" FALSE)

add_library(msvcrtex INTERFACE)
add_dependencies(msvcrtex rust_msvcrtex_dll rust_msvcrtex_exe rust_msvcrtex_wexe rust_msvcrtex_bare
    rust_msvcrtex_dllmain rust_msvcrtex_crt0 rust_msvcrtex_wcrt0)
set(_kind "$<TARGET_PROPERTY:REACTOS_CRT_KIND>")
target_link_libraries(msvcrtex INTERFACE
    "$<$<STREQUAL:${_kind},dll>:${msvcrtex_dll_LIB}>"
    "$<$<STREQUAL:${_kind},dll>:${msvcrtex_dllmain_LIB}>"
    "$<$<STREQUAL:${_kind},exe>:${msvcrtex_exe_LIB}>"
    "$<$<STREQUAL:${_kind},exe>:${msvcrtex_crt0_LIB}>"
    "$<$<STREQUAL:${_kind},wexe>:${msvcrtex_wexe_LIB}>"
    "$<$<STREQUAL:${_kind},wexe>:${msvcrtex_wcrt0_LIB}>"
    "$<$<NOT:$<IN_LIST:${_kind},dll;exe;wexe>>:${msvcrtex_bare_LIB}>")

# MSVCRTEX_ASM_SOURCE (the stack probes, i386 _alldiv/_aulldiv, i386 ceilf/floorf) is
# crt-startup-rust's `intrinsics` module (global_asm!), in every variant above; no
# msvcrtex_asm library is built.

# msvcrtex depends on msvcrt and kernel32
# + ntdll: crt-startup-rust's panic path (panic-common) imports DbgPrint,
# NtTerminateProcess and RtlWalkFrameChain, which the C archive never referenced.
target_link_libraries(msvcrtex INTERFACE libmsvcrt libkernel32 libntdll)
target_link_libraries(msvcrtex INTERFACE oldnames)

# For clang/llvm-mingw, link compiler-rt builtins for i386 64-bit division
# (__divdi3, __udivdi3, __umoddi3, __moddi3) and other builtins
if(USE_LLVM_MINGW AND TARGET libclang_rt_builtins)
    target_link_libraries(msvcrtex INTERFACE libclang_rt_builtins)
endif()
