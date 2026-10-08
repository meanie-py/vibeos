
add_definitions(${I18N_DEFS})
if(_WINKD_)
    add_definitions(-D_WINKD_)
endif()
add_definitions(-D_SETUPLIB_)

include_directories(spapisup utils)

spec2def(setuplib.dll setuplib.spec ADD_IMPORTLIB)

# Every translation unit below is Rust now (sdk/lib/rust/setuplib-rust, `rustconv port`);
# the C stays on disk. setuplib_static keeps a stub TU and links the Rust archive, so the
# DLL and setuplib_unittest pick it up unchanged.
list(APPEND SOURCE
    # spapisup/fileqsup.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/fileqsup.rs)
    # spapisup/infsupp.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/infsupp.rs)
    # utils/arcname.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/arcname.rs)
    # utils/bldrsup.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/bldrsup.rs)
    # utils/devutils.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/devutils.rs)
    # utils/filesup.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/filesup.rs)
    # utils/fsrec.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/fsrec.rs)
    # utils/genlist.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/genlist.rs)
    # utils/inicache.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/inicache.rs)
    # utils/ntverrsrc.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/ntverrsrc.rs)
    # utils/osdetect.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/osdetect.rs)
    # utils/partinfo.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/partinfo.rs)
    # utils/partlist.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/partlist.rs)
    # utils/regutil.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/regutil.rs)
    # utils/volutil.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/volutil.rs)
    # bootcode.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/bootcode.rs)
    # bootsup.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/bootsup.rs)
    # fsutil.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/fsutil.rs)
    # install.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/install.rs)
    # mui.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/mui.rs)
    # registry.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/registry.rs)
    # settings.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/settings.rs)
    # setuplib.c  # replaced by Rust (sdk/lib/rust/setuplib-rust/src/setuplib/setuplib.rs)
    # precomp.h
    rust_cohab_stub.c)

rust_staticlib_variant(setuplib_rust setuplib-rust "" TRUE)

# Static library used for the dll and for setuplib_unittest.
add_library(setuplib_static STATIC ${SOURCE})
add_dependencies(setuplib_static xdk rust_setuplib_rust) # psdk
target_link_libraries(setuplib_static PUBLIC ${setuplib_rust_LIB})
target_include_directories(setuplib_static INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

add_library(setuplib MODULE
    main.c
    setuplib.rc
    ${CMAKE_CURRENT_BINARY_DIR}/setuplib.def)
target_link_libraries(setuplib setuplib_static)
target_include_directories(setuplib INTERFACE ${CMAKE_CURRENT_SOURCE_DIR})

add_dependencies(setuplib xdk) # psdk
set_module_type(setuplib nativedll)
target_link_libraries(setuplib vfatlib ${PSEH_LIB})
add_importlibs(setuplib ntdll)
add_cd_file(TARGET setuplib DESTINATION reactos/system32 NO_CAB FOR bootcd regtest)

# modrun's runtime probe of setuplib (tools/payloads/modrun), built only on request. It links
# setuplib_static, so c_oracle.cmake gives it a twin on the pre-flip C: setuplib_probe_c.
add_executable(setuplib_probe EXCLUDE_FROM_ALL modrun_probe.c)
target_link_libraries(setuplib_probe setuplib_static vfatlib ${PSEH_LIB})
set_module_type(setuplib_probe win32cui UNICODE)
add_importlibs(setuplib_probe msvcrt kernel32 ntdll)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
