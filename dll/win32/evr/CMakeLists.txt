
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(evr.dll evr.spec ADD_IMPORTLIB)

add_idl_headers(evr_classes_header evr_classes.idl)

list(APPEND SOURCE
    evr.c
    main.c
    mixer.c
    presenter.c
    sample.c
    ${CMAKE_CURRENT_BINARY_DIR}/evr_stubs.c)

add_library(evr MODULE
    ${SOURCE}
    ${CMAKE_CURRENT_BINARY_DIR}/evr.def)

set_module_type(evr win32dll)
add_idl_reg_scripts(evr registry evr_classes.idl)
target_link_libraries(evr wine wine_dll_register wine_dll_canunload strmbase mfuuid strmiids uuid dxguid oldnames)
add_delay_importlibs(evr mfplat)
add_importlibs(evr ole32 oleaut32 user32 d3d9 dxva2 msvcrt kernel32 ntdll)
add_dependencies(evr psdk wineheaders evr_classes_header)
add_cd_file(TARGET evr DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(evr)
