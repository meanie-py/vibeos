
if(ARCH STREQUAL "i386")
    list(APPEND LIBCNTPR_EXCEPT_ASM_SOURCE
        except/i386/chkstk_asm.s
    )
elseif(ARCH STREQUAL "amd64")
    list(APPEND LIBCNTPR_EXCEPT_SOURCE
        except/amd64/ehandler.c
    )
    list(APPEND LIBCNTPR_EXCEPT_ASM_SOURCE
        except/amd64/chkstk_ms.s
        except/amd64/seh.s
    )
    list(APPEND CRT_EXCEPT_ASM_SOURCE
        except/amd64/seh.s
    )
    if(MSVC)
        list(APPEND CRT_EXCEPT_ASM_SOURCE
            except/amd64/cpp.s
            except/amd64/cpp_alias.s)
    endif()
    # arm32 (except/arm/*) deliberately declined: arch/arm is a pruned tree.
endif()

if(ARCH STREQUAL "i386")
    list(APPEND CHKSTK_ASM_SOURCE except/i386/chkstk_asm.s)
elseif(ARCH STREQUAL "amd64")
    list(APPEND CHKSTK_ASM_SOURCE except/amd64/chkstk_ms.s)
elseif(ARCH STREQUAL "arm64")
    list(APPEND CHKSTK_ASM_SOURCE except/arm64/chkstk_asm.s)
endif()

add_asm_files(chkstk_lib_asm ${CHKSTK_ASM_SOURCE})
add_library(chkstk ${CHKSTK_SOURCE} ${chkstk_lib_asm})
set_target_properties(chkstk PROPERTIES LINKER_LANGUAGE "C")
add_dependencies(chkstk asm)
