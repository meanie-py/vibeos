
include_directories(include/internal/mingw-w64)

list(APPEND MSVCRTEX_SOURCE
    ${CRT_STARTUP_SOURCE}
    math/sincos.c
    misc/fltused.c
    misc/isblank.c
    misc/iswblank.c
    misc/ofmt_stub.c
    stdio/acrt_iob_func.c
    recalloc.c
    wcscpy_s.c)

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
    if (CMAKE_C_COMPILER_ID STREQUAL "Clang" AND NOT MSVC)
        list(APPEND MSVCRTEX_ASM_SOURCE
            math/i386/ceilf.S
            math/i386/floorf.S)
        list(APPEND MSVCRTEX_SOURCE
            math/i386/sqrtf.c)
    endif()
    if(MSVC AND DLL_EXPORT_VERSION LESS 0x600)
        list(APPEND MSVCRTEX_ASM_SOURCE
            except/i386/__CxxFrameHandler3.s
            math/i386/ftol2_asm.s
            math/i386/ftoul2_legacy_asm.s)
        list(APPEND MSVCRTEX_SOURCE
            except/i386/CxxHandleV8Frame.c)
    endif()
elseif(ARCH STREQUAL "amd64")
    list(APPEND MSVCRTEX_ASM_SOURCE
        except/amd64/chkstk_ms.s)
    if(MSVC AND DLL_EXPORT_VERSION LESS 0x600)
        list(APPEND MSVCRTEX_ASM_SOURCE
            except/amd64/__CxxFrameHandler3.s
        )
    endif()
endif()

set_source_files_properties(${MSVCRTEX_ASM_SOURCE} PROPERTIES COMPILE_DEFINITIONS "_DLL;_MSVCRTEX_")
add_asm_files(msvcrtex_asm ${MSVCRTEX_ASM_SOURCE})

add_library(msvcrtex STATIC ${MSVCRTEX_SOURCE} ${msvcrtex_asm})
target_compile_definitions(msvcrtex PRIVATE _DLL _MSVCRTEX_)

# msvcrtex depends on msvcrt and kernel32
target_link_libraries(msvcrtex INTERFACE libmsvcrt libkernel32)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(msvcrtex PRIVATE $<$<COMPILE_LANGUAGE:C>:-Wno-main>)
    if(LTCG)
        target_compile_options(msvcrtex PRIVATE -fno-lto)
    endif()
endif()

set_source_files_properties(startup/crtdll.c PROPERTIES COMPILE_DEFINITIONS CRTDLL)
set_source_files_properties(startup/crtexe.c
                            startup/wcrtexe.c PROPERTIES COMPILE_DEFINITIONS _M_CEE_PURE)

if(NOT MSVC)
    target_link_libraries(msvcrtex INTERFACE oldnames)
endif()

# For clang/llvm-mingw, link compiler-rt builtins for i386 64-bit division
# (__divdi3, __udivdi3, __umoddi3, __moddi3) and other builtins
if(USE_LLVM_MINGW AND TARGET libclang_rt_builtins)
    target_link_libraries(msvcrtex INTERFACE libclang_rt_builtins)
endif()

add_dependencies(msvcrtex psdk asm)
