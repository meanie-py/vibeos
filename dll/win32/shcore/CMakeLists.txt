
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

spec2def(shcore.dll shcore.spec ADD_IMPORTLIB)

add_definitions(-DCONST_VTABLE)

list(APPEND SOURCE
    main.c
    ${CMAKE_CURRENT_BINARY_DIR}/shcore.def
    ${CMAKE_CURRENT_BINARY_DIR}/shcore_stubs.c
)

add_library(shcore MODULE ${SOURCE})
set_module_type(shcore win32dll ENTRYPOINT 0)
target_link_libraries(shcore crtheap wine wine_dll_canunload)
# combase is not built here (see dll/win32/CMakeLists.txt): what main.c takes from it
# (CoTaskMem*, CoCreateInstance, ...) ole32 exports too. The DPI calls forward to
# shell32 (shcore.spec), which hosts them for the api sets and is graded against
# Windows; main.c's Wine versions of those are not exported.
add_importlibs(shcore ole32 advapi32 advapi32_vista kernel32 ntdll)
add_dependencies(shcore psdk)
add_cd_file(TARGET shcore DESTINATION reactos/system32 FOR all)
set_wine_module(shcore)
