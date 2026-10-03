
set(CMAKE_CXX_STANDARD 17)

# Replace the old CRT include directory with the UCRT include directory
get_property(INCLUDE_DIRS DIRECTORY . PROPERTY INCLUDE_DIRECTORIES)
list(REMOVE_ITEM INCLUDE_DIRS "${REACTOS_SOURCE_DIR}/sdk/include/crt")
set_property(DIRECTORY . PROPERTY INCLUDE_DIRECTORIES ${INCLUDE_DIRS})
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/ucrt)

if(MSVC)
    # Disable warning C4083: expected ')'; found identifier '<warning identifier>'
    add_compile_options($<$<COMPILE_LANGUAGE:C,CXX>:/wd4083>)

    # Disable warning C4189: 'cvt': local variable is initialized but not referenced
    add_compile_options($<$<COMPILE_LANGUAGE:C,CXX>:/wd4189>)
endif()

# Internal includes. ucrt_reactos/inc comes first and holds the headers ReactOS ADDED to
# this drop (corecrt_internal_state_isolation.h) — kept outside the vendored tree so
# sdk/lib/ucrt stays a pure Microsoft drop and a re-vendor is a copy, not a merge.
# corecrt_internal.h:2119 includes it, which is itself one of this tree's adaptation
# sites; see vendor.yaml's ucrt entry.
include_directories(BEFORE inc ${REACTOS_SOURCE_DIR}/sdk/lib/ucrt_reactos/inc)

if(${ARCH} STREQUAL "i386")
    include_directories(inc/i386)
endif()

remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_compile_definitions(
    WINVER=0x600
    _WIN32_WINNT=0x600
    _UCRT
    _CORECRT_BUILD
    _CRT_DECLARE_GLOBAL_VARIABLES_DIRECTLY
    _GCC_NO_SAL_ATTRIIBUTES
    CRTDLL
)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR
   CMAKE_C_COMPILER_ID STREQUAL "Clang")
    # Silence GCC/Clang warnings
    add_compile_options(
        -Wno-unknown-warning-option
        -Wno-unused-function
        -Wno-unknown-pragmas
        -Wno-builtin-declaration-mismatch
        -Wno-parentheses
        -Wno-unused-variable
        -Wno-sign-compare
        -Wno-enum-compare
        -Wno-switch
        -Wno-write-strings
        -Wno-comment
        -Wno-narrowing
        -Wno-misleading-indentation
        -Wno-missing-braces
        -Wno-unused-value
        -Wno-unused-local-typedef
        -Wno-unused-function
        -Wno-writable-strings
        -Wno-microsoft-template
        -Wno-switch
        -Wno-ignored-pragmas
        -Wno-empty-body
        -Wno-tautological-constant-out-of-range-compare
        -Wno-ignored-attributes
        -Wno-uninitialized
    )
    add_compile_options($<$<COMPILE_LANGUAGE:CXX>:-Wno-reorder>)
endif()

include(convert/convert.cmake)
include(dll/dll.cmake)
include(float/float.cmake)
include(internal/internal.cmake)
include(locale/locale.cmake)
include(math/math.cmake)
include(misc/misc.cmake)
include(startup/startup.cmake)

# TODO: build a _DEBUG version
add_library(ucrt OBJECT
    ${UCRT_CONVERT_SOURCES}
    ${UCRT_DLL_SOURCES}
    ${UCRT_FLOAT_SOURCES}
    ${UCRT_INTERNAL_SOURCES}
    ${UCRT_LOCALE_SOURCES}
    ${UCRT_MATH_SOURCES}
    ${UCRT_MISC_SOURCES}
    ${UCRT_STARTUP_SOURCES}
    ${UCRT_VCRUNTIME_SOURCES}
)

target_link_libraries(ucrt vcruntime)
add_dependencies(ucrt psdk asm)
