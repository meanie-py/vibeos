
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/lib/crt/include)

list(APPEND SOURCE
    __mingw_vsprintf.c
    StringCchCopyNA.c
    guard_check_icall.c
    ucrt_compat.c)

# __throw_out_of_range_fmt is a GCC libstdc++ internal function, not needed for Clang/libc++
if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
    list(APPEND SOURCE __throw_out_of_range_fmt.cpp)
endif()

if(DLL_EXPORT_VERSION LESS 0x600)
    list(APPEND SOURCE
        rand_s.c)
endif()

add_library(stdc++compat ${SOURCE})
if(NOT CMAKE_C_COMPILER_ID STREQUAL "Clang")
    set_target_cpp_properties(stdc++compat WITH_EXCEPTIONS)
    target_link_libraries(stdc++compat libmsvcrt cppstl)
    # Note: linking with cppstl prevents -nostdinc from being added for C++ files
    target_compile_definitions(stdc++compat PRIVATE "$<$<COMPILE_LANGUAGE:CXX>:PAL_STDCPP_COMPAT>" _CRT_DECLARE_NONSTDC_NAMES=1)
else()
    # libc++ threading uses Vista+ APIs (SRW locks, condition variables)
    target_link_libraries(stdc++compat libmsvcrt libkernel32_vista)
endif()
add_dependencies(stdc++compat xdk)
