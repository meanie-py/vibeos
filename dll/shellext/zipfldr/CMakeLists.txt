
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

spec2def(zipfldr.dll zipfldr.spec ADD_IMPORTLIB)

list(APPEND SOURCE
    # zipfldr.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/zipfldr_cpp_h.rs)
    # zippidl.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/zippidl_cpp_h.rs)
    zippidl.hpp
    IZip.hpp
    # CConfirmReplace.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CConfirmReplace_cpp_h.rs)
    # CExplorerCommand.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CExplorerCommand_cpp_h.rs)
    # CEnumZipContents.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CEnumZipContents_cpp_h.rs)
    # CFolderViewCB.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CFolderViewCB_cpp_h.rs)
    # CSendToZip.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CSendToZip_cpp_h.rs)
    # CZipCreator.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CZipCreator_cpp_h.rs)
    CZipEnumerator.hpp
    # CZipEnumerator.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CZipEnumerator_cpp_h.rs)
    # CZipExtract.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CZipExtract_cpp_h.rs)
    CZipFolder.hpp
    # CZipFolder.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CZipFolder_cpp_h.rs)
    # CZipPassword.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/CZipPassword_cpp_h.rs)
    # Debug.cpp  # replaced by Rust (sdk/lib/rust/zipfldr-rust/src/zipfldr/Debug_cpp_h.rs)
    rust_cohab_stub.cpp
    zipfldr.spec
    precomp.h
    resource.h)

# zipfldr-rust passes floats by value: i386 user code needs SSE (see oleaut32).
set(ZIPFLDR_RUST_ENV)
if(ARCH STREQUAL "i386")
    set(ZIPFLDR_RUST_ENV ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
# Every translation unit is Rust now (zipfldr-rust, `rustconv port`); the C stays on
# disk. The target keeps a stub TU and links the Rust archive.
rust_staticlib_variant(zipfldr_rust zipfldr-rust "" TRUE ${ZIPFLDR_RUST_ENV})
add_library(zipfldr MODULE
    ${SOURCE}
    zipfldr.rc
    ${CMAKE_CURRENT_BINARY_DIR}/zipfldr.def)
add_dependencies(zipfldr rust_zipfldr_rust)
target_link_libraries(zipfldr ${zipfldr_rust_LIB})
add_pch(zipfldr precomp.h SOURCE)

set_module_type(zipfldr win32dll UNICODE)
target_link_libraries(zipfldr minizip zlib uuid cpprt atl_classes)
set_target_cpp_properties(zipfldr WITH_EXCEPTIONS)
add_importlibs(zipfldr oleaut32 ole32 shlwapi comctl32 shell32 user32 advapi32 msvcrt kernel32 ntdll)
add_cd_file(TARGET zipfldr DESTINATION reactos/system32 FOR all)

include(c_oracle.cmake)  # the pre-flip C as <target>_c: python -m rustconv.c_oracle
