
project(SHELL)

add_definitions(
    -D__WINESRC__
    -D_SHLWAPI_)

spec2def(shlwapi.dll shlwapi.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # appcompat.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/appcompat.rs)
    # clist.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/clist.rs)
    # istream.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/istream.rs)
    # msgbox.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/msgbox.rs)
    # ordinal.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/ordinal.rs)
    # path.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/path.rs)
    # reg.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/reg.rs)
    # regstream.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/regstream.rs)
    # rosordinal.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/rosordinal.rs)
    # shlwapi_main.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/shlwapi_main.rs)
    # stopwatch.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/stopwatch.rs)
    # string.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/string.rs)
    # thread.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/thread.rs)
    # url.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/url.rs)
    )

# New upstream TUs taken with this merge: zonechk.c implements ZoneCheckUrlEx* for real
# (our ordinal.c only had a guessed-prototype FIXME stub), and policy.cpp / querysrc.cpp
# supply SKGetValueW/SKSetValueW/SHWindowsPolicyEx/QuerySourceCreateFromKey, which the
# merged browseui, shdocvw, shellbars and shelldesktop now call. The rest of upstream's
# shlwapi change — forwarding ~60 exports to the declined wine-kernelbase/shcore split —
# is NOT taken, so kernelbase.c stays out and the .spec stays ours.
list(APPEND PCH_SKIP_SOURCE
    policy.cpp
    querysrc.cpp
    # zonechk.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/zonechk.rs)
    # assoc.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/assoc.rs)
    propbag.cpp
    utils.cpp
    # wsprintf.c  # replaced by Rust (sdk/lib/rust/shlwapi-rust/src/shlwapi/wsprintf.rs)
    rust_cohab_stub.c
    rust_varargs_shim.c
    ${CMAKE_CURRENT_BINARY_DIR}/shlwapi_stubs.c)

# shlwapi-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(SHLWAPI_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(SHLWAPI_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (shlwapi-rust, `rustconv port`) except policy.cpp, propbag.cpp, querysrc.cpp, utils.cpp,
# kept C by `port derive --keep-c`; the C stays on disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(shlwapi_rust shlwapi-rust "" TRUE ${SHLWAPI_RUST_ENV})
add_library(shlwapi MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    $<TARGET_OBJECTS:shlwapi_autocomp>
    shlwapi.rc
    ${CMAKE_CURRENT_BINARY_DIR}/shlwapi.def)
add_dependencies(shlwapi rust_shlwapi_rust)
target_link_libraries(shlwapi ${shlwapi_rust_LIB})

# our C++ atlbase.h conflicts with the one from wine, so only use wine includes for C
# Unfortunately, we can't use different includes for C & C++ in VS generator, so use an object library to achieve this
target_include_directories(shlwapi BEFORE PRIVATE
    ${REACTOS_SOURCE_DIR}/sdk/include/wine
    ${REACTOS_SOURCE_DIR}/sdk/lib/atl)

add_library(shlwapi_autocomp OBJECT autocomp.cpp)
target_link_libraries(shlwapi_autocomp PRIVATE atl_classes)
target_compile_definitions(shlwapi_autocomp PRIVATE $<TARGET_PROPERTY:shlwapi,COMPILE_DEFINITIONS>)
target_compile_options(shlwapi_autocomp PRIVATE $<TARGET_PROPERTY:shlwapi,COMPILE_OPTIONS>)
add_dependencies(shlwapi_autocomp psdk)

set_module_type(shlwapi win32dll UNICODE)
target_link_libraries(shlwapi uuid wine cpprt)
add_delay_importlibs(shlwapi userenv oleaut32 ole32 comctl32 comdlg32 mpr mlang urlmon shell32 winmm version)
add_importlibs(shlwapi user32 gdi32 advapi32 wininet msvcrt kernel32 ntdll)
add_pch(shlwapi precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET shlwapi DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
