
remove_definitions(-DWINVER=0x502 -D_WIN32_WINNT=0x502)
add_definitions(-DWINVER=0x0A00 -D_WIN32_WINNT=0x0A00 -DNTDDI_VERSION=0x0A000005)

list(APPEND SOURCE
    broadcast.c
    class.c
    clipboard.c
    combo.c
    cursoricon.c
    dce.c
    dde.c
    dialog.c
    edit.c
    generated.c
    input.c
    listbox.c
    menu.c
    monitor.c
    msg.c
    resource.c
    scroll.c
    static.c
    sysparams.c
    text.c
    uitools.c
    win.c
    winstation.c
    wsprintf.c)

list(APPEND PCH_SKIP_SOURCE
    testlist.c)

# resource.rc gives two CUSTOM_TEST_DIALOG controls trailing creation data
# (`CONTROL ... ,10,10,100,50 { 1,2,3,4,5 }`), which is what dialog.c's
# test_extra_data reads back. llvm-rc has never implemented that syntax, so the
# pinned toolchain's windres cannot build this file -- and resource.rc also
# carries the accelerators, string tables, menus and every other dialog the
# suite uses, so one unparseable construct costs all of user32_winetest. It is
# Wine's oracle, not ours to rewrite, so this directory compiles resources with
# GNU windres instead. See use_gnu_rc() in sdk/cmake/gcc.cmake.
use_gnu_rc()

if(NOT GNU_RC_COMPILER)
    # use_gnu_rc() already said why, loudly. Skip only this target: the other
    # 127 winetest suites build fine on llvm-rc and must not be collateral.
    return()
endif()

add_executable(user32_winetest
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    resource.rc)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    target_compile_options(user32_winetest PRIVATE
    -Wno-format
    -Wno-int-conversion)
endif()

target_compile_definitions(user32_winetest PRIVATE wcsicmp=_wcsicmp)
set_module_type(user32_winetest win32cui)
add_importlibs(user32_winetest user32 gdi32 advapi32 imm32 hid msvcrt kernel32 ntdll)
add_pch(user32_winetest precomp.h "${PCH_SKIP_SOURCE}")
add_rostests_file(TARGET user32_winetest)
