
remove_definitions(-DWINVER=0x502 -D_WIN32_IE=0x600 -D_WIN32_WINNT=0x502)
remove_definitions(-DDXGKDDI_INTERFACE_VERSION=0x1052)

include_directories(BEFORE ${REACTOS_SOURCE_DIR}/sdk/include/wine)

list(APPEND SOURCE
    bitmap.c
    brush.c
    clipping.c
    dc.c
    dib.c
    driver.c
    font.c
    gdiobj.c
    generated.c
    icm.c
    mapping.c
    metafile.c
    palette.c
    path.c
    pen.c)

list(APPEND PCH_SKIP_SOURCE
    testlist.c)

add_executable(gdi32_winetest
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    resource.rc)

target_compile_definitions(gdi32_winetest PRIVATE
    _USE_MATH_DEFINES
    DXGKDDI_INTERFACE_VERSION=DXGKDDI_INTERFACE_VERSION_WDDM2_2
)

set_module_type(gdi32_winetest win32cui)
target_link_libraries(gdi32_winetest oldnames)
add_importlibs(gdi32_winetest gdi32 gdi32_vista user32 setupapi advapi32 msvcrt kernel32 ntdll)

#add_pch(gdi32_winetest precomp.h "${PCH_SKIP_SOURCE}")
add_rostests_file(TARGET gdi32_winetest)
