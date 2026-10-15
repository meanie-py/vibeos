
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x0A00)
spec2def(mf.dll mf.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    clock.c
    copier.c
    evr.c
    main.c
    quality.c
    sac.c
    samplegrabber.c
    sar.c
    scheme_handler.c
    session.c
    topology.c
    topology_loader.c
    ${CMAKE_CURRENT_BINARY_DIR}/mf_stubs.c)

add_library(mf MODULE
    ${SOURCE}
    mf.rc
    ${CMAKE_CURRENT_BINARY_DIR}/mf.def)

set_module_type(mf win32dll)
add_idl_reg_scripts(mf registry mf.idl)
target_link_libraries(mf wine wine_dll_register wine_dll_canunload uuid mfuuid strmiids dxguid oldnames)
add_delay_importlibs(mf evr urlmon user32)
add_importlibs(mf advapi32 mfplat ole32 oleaut32 shlwapi kernelbase msvcrt kernel32 kernel32_vista ntdll)
add_dependencies(mf psdk wineheaders)
add_cd_file(TARGET mf DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(mf)
