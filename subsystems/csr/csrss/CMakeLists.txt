
include_directories(
    ${REACTOS_SOURCE_DIR}/subsystems/win32/csrsrv/include
    ${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys)

add_executable(csrss csrss.c csrss.rc)
set_module_type(csrss nativecui)
target_link_libraries(csrss nt)
add_importlibs(csrss csrsrv ntdll)
if(ARCH STREQUAL "arm64")
    # ARM64: llvm-dlltool converts delay-imports to regular imports, causing
    # all DLL dependencies to load at once. Deep recursive LdrpLoadDll calls
    # overflow the default 1MB/4KB-commit stack. Reserve 2MB, commit 256KB.
    # SizeOfStackCommit is patched to 0x40000 by qemu/mkramdisk.sh.
    target_link_options(csrss PRIVATE "-Wl,--stack,2097152")
endif()
add_dependencies(csrss psdk bugcodes)
add_cd_file(TARGET csrss DESTINATION reactos/system32 FOR all)
