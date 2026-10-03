
add_definitions(
    -DENTRY_PREFIX=WIC_
    -DPROXY_DELEGATION
    -DWINE_REGISTER_DLL)

remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

# upstream dropped these, relying on the vendored libs publishing their headers through
# their own targets. Our libpng/libjpeg/libtiff are at different versions (we declined the
# libjpeg v10 API break and are AHEAD on libpng) and do not, so the paths stay spelled out.
include_directories(
    ${REACTOS_SOURCE_DIR}/vendor/libjpeg/include
    ${REACTOS_SOURCE_DIR}/vendor/zlib/include
    ${REACTOS_SOURCE_DIR}/vendor/libpng/include
    ${REACTOS_SOURCE_DIR}/vendor/libtiff/include)

spec2def(windowscodecs.dll windowscodecs.spec ADD_IMPORTLIB)
add_rpcproxy_files(windowscodecs_wincodec.idl)

list(APPEND SOURCE
    bmpdecode.c
    bmpencode.c
    clipper.c
    clsfactory.c
    colorcontext.c
    colortransform.c
    converter.c
    ddsformat.c
    decoder.c
    encoder.c
    fliprotate.c
    gifformat.c
    icoformat.c
    imgfactory.c
    info.c
    libjpeg.c
    libpng.c
    libtiff.c
    main.c
    metadatahandler.c
    metadataquery.c
    palette.c
    pngformat.c
    propertybag.c
    proxy.c
    regsvr.c
    scaler.c
    stream.c
    tgaformat.c
    ungif.c
    uuid.c
    wincodecs_common.c)

list(APPEND SOURCE bitmap.c)

list(APPEND PCH_SKIP_SOURCE
    guid.c
    ${CMAKE_CURRENT_BINARY_DIR}/proxy.dlldata.c
    ${CMAKE_CURRENT_BINARY_DIR}/windowscodecs_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/windowscodecs_wincodec_p.c)

add_library(windowscodecs MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    version.rc
    ${CMAKE_CURRENT_BINARY_DIR}/windowscodecs.def)
add_pch(windowscodecs precomp.h "${PCH_SKIP_SOURCE}")

set_module_type(windowscodecs win32dll)
add_idl_reg_scripts(windowscodecs registry windowscodecs_wincodec.idl)
target_link_libraries(windowscodecs wine uuid ${PSEH_LIB} oldnames)
add_importlibs(windowscodecs libjpeg libpng libtiff ole32 oleaut32 rpcrt4 shlwapi user32 gdi32 advapi32 advapi32_vista propsys msvcrt kernel32 ntdll)
add_cd_file(TARGET windowscodecs DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(windowscodecs) # CORE-5743: No CONST_VTABLE
