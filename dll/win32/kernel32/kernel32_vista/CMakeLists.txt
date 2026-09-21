
remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_WIN32_WINNT=0x600 -DWINVER=0x600)

add_definitions(-D_KERNEL32_)
include_directories(${REACTOS_SOURCE_DIR}/sdk/include/reactos/subsys ..)
spec2def(kernel32_vista.dll kernel32_vista.spec ADD_IMPORTLIB)

# firmware.c EnumSystemLocalesEx.c GetLocaleInfoEx.c GetFileInformationByHandleEx.c
# GetFinalPathNameByHandle.c GetNamedPipeClientProcessId.c GetTickCount64.c
# GetUserDefaultLocaleName.c InitOnce.c IsValidLocaleName.c LCIDToLocaleName.c
# LocaleNameToLCID.c SetThreadDescription.c sync.c vista.c
#   — all replaced by Rust (sdk/lib/rust/kernel32-vista-rust → libkernel32_vista_rust.a,
#     built in the parent CMakeLists and linked into BOTH kernel32.dll and
#     kernel32_vista.dll). The .c files stay on disk (never-delete rule).
# threadpool.c — new upstream file (CreateThreadpool*/TrySubmitThreadpoolCallback
# family, thin wrappers over ntdll Tp* exports), no Rust twin exists; kept live.
list(APPEND SOURCE
    threadpool.c)

# NT6 console support (client/console/vista.c) is still C and only compiled when NT6
# exports are enabled; in that case build it as a console-only static lib. (Not reached
# at the default DLL_EXPORT_VERSION 0x502.)
set(KERNEL32_VISTA_C_LIB "")
if(DLL_EXPORT_VERSION GREATER_EQUAL 0x600)
    add_library(kernel32_vista_static ../client/console/vista.c)
    target_link_libraries(kernel32_vista_static ${PSEH_LIB})
    add_dependencies(kernel32_vista_static psdk)
    set(KERNEL32_VISTA_C_LIB kernel32_vista_static)
endif()

add_library(kernel32_vista MODULE
    DllMain.c
    kernel32_vista.rc
    ${CMAKE_CURRENT_BINARY_DIR}/kernel32_vista.def)
set_module_type(kernel32_vista win32dll ENTRYPOINT DllMain 12)
# kernel32_vista_static + kernel32_shared C → Rust (libkernel32_{vista,shared}_rust.a,
# built in the parent CMakeLists). DllMain.c stays C (the module entrypoint).
target_link_libraries(kernel32_vista ${KERNEL32_VISTA_C_LIB} ${KERNEL32_VISTA_RUST_LIB} ${KERNEL32_SHARED_RUST_LIB})
add_importlibs(kernel32_vista kernel32 ntdll_vista ntdll)
add_delay_importlibs(kernel32_vista ntdll_vista)
add_dependencies(kernel32_vista psdk rust_kernel32_shared rust_kernel32_vista)
add_cd_file(TARGET kernel32_vista DESTINATION reactos/system32 FOR all)
