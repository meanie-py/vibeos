
remove_definitions(-D_WIN32_WINNT=0x502 -DWINVER=0x502)
add_definitions(-D_USE_MATH_DEFINES -D_WIN32_WINNT=0x0600 -DWINVER=0x0600)

include_directories(${VKD3D_INCLUDE_DIRS})

spec2def(d2d1.dll d2d1.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    bitmap_render_target.c
	bitmap.c
	brush.c
	command_list.c
	dc_render_target.c
	device.c
	effect.c
	factory.c
	geometry.c
	hwnd_render_target.c
	layer.c
	mesh.c
	state_block.c
	stroke.c
	wic_render_target.c)

add_library(d2d1 MODULE
    ${SOURCE}
    version.rc
    ${CMAKE_CURRENT_BINARY_DIR}/d2d1_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/d2d1.def)

set_module_type(d2d1 win32dll UNICODE ENTRYPOINT 0)
add_delay_importlibs(d2d1 dwrite xmllite ole32)
target_link_libraries(d2d1 wine dxguid dx10guid uuid oldnames chkstk)
add_importlibs(d2d1 d3d10_1 d3dcompiler_47 gdi32 advapi32 advapi32_vista user32 ucrtbase kernel32 kernel32_vista ntdll)
add_dependencies(d2d1 wineheaders d3d_idl_headers)
if(ARCH STREQUAL "i386")
    # i386 ucrtbase exports no ceilf or sqrtf; math.h makes both inlines over the double
    # functions, and clang emits the float libcall anyway. sqrtf is a builtin it calls as
    # such (math-errno), so -fno-builtin-sqrtf lets the inline body stand; ceil() becomes
    # llvm.ceil, which shrinks (float)ceil((double)x) to llvm.ceil.f32 and, without
    # SSE4.1, lowers to ceilf, so -fno-builtin-ceil keeps a real call to ceil().
    target_compile_options(d2d1 PRIVATE -fno-builtin-ceil -fno-builtin-sqrtf)
endif()
add_cd_file(TARGET d2d1 DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(d2d1)
