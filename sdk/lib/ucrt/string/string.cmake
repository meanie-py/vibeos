
list(APPEND UCRT_STRING_SOURCES
    string/memcpy_s.cpp
    string/memicmp.cpp
    string/strcat_s.cpp
    string/strcoll.cpp
    string/strcpy_s.cpp
    string/strdup.cpp
    string/stricmp.cpp
    string/stricoll.cpp
    string/strlwr.cpp
    string/strncat_s.cpp
    string/strncnt.cpp
    string/strncoll.cpp
    string/strncpy_s.cpp
    string/strnicmp.cpp
    string/strnicol.cpp
    string/strnlen.cpp
    string/strnset_s.cpp
    string/strset_s.cpp
    string/strtok.cpp
    string/strtok_s.cpp
    string/strupr.cpp
    string/strxfrm.cpp
    string/wcscat.cpp
    string/wcscat_s.cpp
    string/wcscmp.cpp
    string/wcscoll.cpp
    string/wcscpy.cpp
    string/wcscpy_s.cpp
    string/wcscspn.cpp
    string/wcsdup.cpp
    string/wcsicmp.cpp
    string/wcsicoll.cpp
    string/wcslwr.cpp
    string/wcsncat.cpp
    string/wcsncat_s.cpp
    string/wcsncmp.cpp
    string/wcsncnt.cpp
    string/wcsncoll.cpp
    string/wcsncpy.cpp
    string/wcsncpy_s.cpp
    string/wcsnicmp.cpp
    string/wcsnicol.cpp
    string/wcsnset.cpp
    string/wcsnset_s.cpp
    string/wcspbrk.cpp
    string/wcsrev.cpp
    string/wcsset.cpp
    string/wcsset_s.cpp
    string/wcsspn.cpp
    string/wcstok.cpp
    string/wcstok_s.cpp
    string/wcsupr.cpp
    string/wcsxfrm.cpp
    string/wmemcpy_s.cpp
    string/wmemmove_s.cpp
)

# Special handling for GCC and Clang (x86/x64 only - SSE/AVX are x86-specific)
if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    if(ARCH STREQUAL "i386" OR ARCH STREQUAL "amd64")
        list(APPEND UCRT_STRING_SOURCES
            string/strnlen-avx2.cpp
            string/strnlen-sse2.cpp
        )

        set_source_files_properties(string/strnlen-sse2.cpp PROPERTIES COMPILE_OPTIONS "-msse2")
        set_source_files_properties(string/strnlen-avx2.cpp PROPERTIES COMPILE_OPTIONS "-mavx2")
    endif()
endif()

# strcat strcpy strcspn strlen strncat strncpy strpbrk strspn are no longer built
# here on any arch: ucrtbase takes them from crt-portable-rust (the `string`
# family in tools/ucrt-takeover.yaml).
if(${ARCH} STREQUAL "i386")
    list(APPEND UCRT_STRING_ASM_SOURCES
        string/i386/_memicmp.s
        string/i386/_strnicm.s
        string/i386/memccpy.s
        string/i386/strcmp.s
        string/i386/strncmp.s
        string/i386/strnset.s
        string/i386/strrev.s
        string/i386/strset.s
    )
elseif(${ARCH} STREQUAL "amd64")
    list(APPEND UCRT_STRING_ASM_SOURCES
        string/amd64/strcmp.s
        string/amd64/strncmp.s
    )
    list(APPEND UCRT_STRING_SOURCES
        string/memccpy.c
        string/strnset.c
        string/strrev.c
        string/strset.c
    )
elseif(${ARCH} STREQUAL "arm64")
    # ARM64 assembly files use ARMASM syntax which isn't compatible with Clang/GAS
    # Use C fallbacks instead.
    list(APPEND UCRT_STRING_SOURCES
        string/memccpy.c
        string/strcmp.c
        string/strncmp.c
        string/strnset.c
        string/strrev.c
        string/strset.c
    )
endif()

add_asm_files(UCRT_STRING_ASM ${UCRT_STRING_ASM_SOURCES})
list(APPEND UCRT_STRING_SOURCES ${UCRT_STRING_ASM})
