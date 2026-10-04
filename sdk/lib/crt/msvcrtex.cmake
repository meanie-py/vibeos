
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
# Only the MSVC integer/stack-probe intrinsics stay assembly (msvcrtex_asm).
set(MSVCRTEX_RUST_WORKSPACE "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
if(ARCH STREQUAL "i386")
    set(MSVCRTEX_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(MSVCRTEX_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(MSVCRTEX_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()

# One staticlib build of `crate` with exactly `features` (no defaults unless `defaults`).
# Each variant gets its own target dir: rust_storage_target allows one owner per artifact,
# and these are several feature sets of the same crate.
function(msvcrtex_rust_lib name crate features defaults)
    set(_dir "${CMAKE_CURRENT_BINARY_DIR}/rust-${name}-target")
    set(_lib "${CMAKE_CURRENT_BINARY_DIR}/lib${name}.a")
    string(REPLACE "-" "_" _artifact "${crate}")
    rust_crate_sources(_srcs "${MSVCRTEX_RUST_WORKSPACE}" ${crate})
    set(_feat)
    if(NOT defaults)
        list(APPEND _feat --no-default-features)
    endif()
    if(features)
        list(APPEND _feat --features ${features})
    endif()
    add_custom_command(
        OUTPUT ${_lib}
        COMMAND ${CMAKE_COMMAND} -E env CARGO_TARGET_DIR=${_dir}
            cargo build --release --target ${MSVCRTEX_RUST_TARGET} -p ${crate} ${_feat}
                --manifest-path ${MSVCRTEX_RUST_WORKSPACE}/Cargo.toml
        COMMAND ${CMAKE_COMMAND} -E copy
            ${_dir}/${MSVCRTEX_RUST_TARGET}/release/lib${_artifact}.a ${_lib}
        DEPENDS ${_srcs} ${MSVCRTEX_RUST_WORKSPACE}/${crate}/Cargo.toml
        WORKING_DIRECTORY ${MSVCRTEX_RUST_WORKSPACE}
        COMMENT "Building ${crate} [${features}] for msvcrtex (${MSVCRTEX_RUST_TARGET})"
        VERBATIM)
    add_custom_target(rust_${name} DEPENDS ${_lib})
    set(${name}_LIB ${_lib} PARENT_SCOPE)
endfunction()

msvcrtex_rust_lib(msvcrtex_dll crt-startup-rust "crtdll" FALSE)
msvcrtex_rust_lib(msvcrtex_exe crt-startup-rust "crtexe" FALSE)
msvcrtex_rust_lib(msvcrtex_wexe crt-startup-rust "wcrtexe" FALSE)
msvcrtex_rust_lib(msvcrtex_bare crt-startup-rust "" FALSE)
msvcrtex_rust_lib(msvcrtex_dllmain crt-dllmain-rust "" TRUE)
msvcrtex_rust_lib(msvcrtex_crt0 crt0-rust "ansi" FALSE)
msvcrtex_rust_lib(msvcrtex_wcrt0 crt0-rust "wide" FALSE)

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

if(MSVCRTEX_ASM_SOURCE)
    set_source_files_properties(${MSVCRTEX_ASM_SOURCE} PROPERTIES COMPILE_DEFINITIONS "_DLL;_MSVCRTEX_")
    add_asm_files(msvcrtex_asm_obj ${MSVCRTEX_ASM_SOURCE})
    add_library(msvcrtex_asm STATIC ${msvcrtex_asm_obj})
    set_target_properties(msvcrtex_asm PROPERTIES LINKER_LANGUAGE C)
    add_dependencies(msvcrtex_asm asm)
    target_link_libraries(msvcrtex INTERFACE msvcrtex_asm)
endif()

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
