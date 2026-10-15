project(rapps)

include_directories(
    include
    ${REACTOS_SOURCE_DIR}/sdk/lib/cryptlib)

list(APPEND SOURCE
    # appdb.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/appdb_cpp_h.rs)
    # appinfo.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/appinfo_cpp_h.rs)
    # appview.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/appview_cpp_h.rs)
    # asyncinet.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/asyncinet_cpp_h.rs)
    # cabinet.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/cabinet_cpp_h.rs)
    # configparser.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/configparser_cpp_h.rs)
    # geninst.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/geninst_cpp_h.rs)
    # gui.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/gui_cpp_h.rs)
    # integrity.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/integrity_cpp_h.rs)
    # loaddlg.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/loaddlg_cpp_h.rs)
    # misc.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/misc_cpp_h.rs)
    # settings.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/settings_cpp_h.rs)
    # settingsdlg.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/settingsdlg_cpp_h.rs)
    # unattended.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/unattended_cpp_h.rs)
    # winmain.cpp  # replaced by Rust (sdk/lib/rust/rapps-rust/src/rapps/winmain_cpp_h.rs)
    rust_cohab_stub.cpp
    include/appdb.h
    include/appinfo.h
    include/appview.h
    include/asyncinet.h
    include/configparser.h
    include/crichedit.h
    include/defines.h
    include/dialogs.h
    include/gui.h
    include/misc.h
    include/rapps.h
    include/resource.h
    include/rosui.h
    include/settings.h
    include/unattended.h
    include/winmain.h
)

add_definitions(
    -DUSE_CERT_PINNING)

file(GLOB_RECURSE rapps_rc_deps res/*.*)
add_rc_deps(rapps.rc ${rapps_rc_deps})
# rapps-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(RAPPS_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(RAPPS_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (rapps-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(rapps_rust rapps-rust "" TRUE ${RAPPS_RUST_ENV})
add_executable(rapps ${SOURCE} rapps.rc)
add_dependencies(rapps rust_rapps_rust)
target_link_libraries(rapps ${rapps_rust_LIB})
add_pch(rapps include/rapps.h SOURCE)
set_module_type(rapps win32gui UNICODE)
target_link_libraries(rapps conutils uuid cpprt atl_classes minizip zlib)
add_importlibs(rapps advapi32 comctl32 gdi32 wininet user32 shell32 shlwapi ole32 setupapi gdiplus msvcrt kernel32 ntdll)
add_dependencies(rapps rappsmsg)
add_message_headers(ANSI rappsmsg.mc)
add_cd_file(TARGET rapps DESTINATION reactos/system32 FOR all)
