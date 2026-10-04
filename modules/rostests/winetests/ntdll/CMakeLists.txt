
include_directories(BEFORE ${REACTOS_SOURCE_DIR}/sdk/include/wine)
add_definitions(-D__WINESRC__ -DWINETEST_USE_DBGSTR_LONGLONG)

remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_WIN32_WINNT=0x601)

if(_WINKD_)
    add_definitions(-D_WINKD_)
endif()

list(APPEND SOURCE
    atom.c
    change.c
    directory.c
    env.c
    error.c
    exception.c
    file.c
    generated.c
    info.c
    large_int.c
    om.c
    path.c
    pipe.c
    port.c
    reg.c
    rtl.c
    rtlbitmap.c
    rtlstr.c
    string.c
    sync.c
    testlist.c
    thread.c
    threadpool.c
    time.c
    unwind.c
    virtual.c
    wow64.c
)

add_executable(ntdll_winetest
    ${SOURCE})

target_link_libraries(ntdll_winetest pseh)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU" OR CMAKE_C_COMPILER_ID STREQUAL "Clang")
    target_compile_options(ntdll_winetest PRIVATE "-Wno-format")
    target_compile_options(ntdll_winetest PRIVATE "-Wno-int-conversion")
    target_compile_options(ntdll_winetest PRIVATE "-Wno-unused-variable")
endif()

set_module_type(ntdll_winetest win32cui)
add_importlibs(ntdll_winetest user32 psapi ole32 advapi32 msvcrt kernel32 ntdll)
add_rostests_file(TARGET ntdll_winetest)
target_compile_definitions(ntdll_winetest PRIVATE wcsnicmp=_wcsnicmp)
