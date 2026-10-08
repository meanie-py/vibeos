
include_directories(
    ${REACTOS_SOURCE_DIR}/sdk/lib/crt/include
    ${REACTOS_SOURCE_DIR}/sdk/include/c++)

# We need different things, depending on the compiler
list(APPEND SOURCE
    __cxa_pure_virtual.cpp)
# ARM64 with Clang needs C++ runtime stubs
if(ARCH STREQUAL "arm64")
    list(APPEND SOURCE arm64/cpprt.cpp)
    # Disable exceptions for this file to avoid circular dependencies
    set_source_files_properties(arm64/cpprt.cpp PROPERTIES COMPILE_OPTIONS "-fno-exceptions")
endif()

add_library(cpprt ${SOURCE} ${cpprt_asm})
set_target_cpp_properties(cpprt WITH_EXCEPTIONS)
add_dependencies(cpprt xdk)

# On GCC builds, we need to link against libsupc++
target_link_libraries(cpprt libsupc++)
