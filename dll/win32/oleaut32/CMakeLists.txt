
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

add_definitions(
    -D_OLEAUT32_
    -DPROXY_DELEGATION
    -DWINE_REGISTER_DLL
    -DENTRY_PREFIX=OLEAUTPS_
    -DPROXY_CLSID=CLSID_PSFactoryBuffer)

spec2def(oleaut32.dll oleaut32.spec ADD_IMPORTLIB)
add_rpcproxy_files(oleaut32_oaidl.idl oleaut32_ocidl.idl)

# NOTE on cohab link order (dll/win32/oleaut32/CMakeLists.txt): the C .obj files
# are linked BEFORE liboleaut32_rust.a, and with --allow-multiple-definition lld
# takes the FIRST definition — so a C TU's Rust twin only takes effect when that
# C TU is commented out here (its undefined refs then pull the Rust archive
# member). Therefore ONLY comment out TUs whose Rust twin is a COMPLETE export
# match. hash/variant/vartype/usrmarshal flipped first; safearray (now has
# SafeArrayCreate*), oleaut, dispatch and typelib flipped 2026-07-12 once their
# Rust twins reached full export coverage.
list(APPEND SOURCE
    # connpt.c — replaced by Rust (oleaut32-rust connpt.rs — IConnectionPoint/IEnumConnections + CreateConnectionPoint; full)
    # dispatch.c — replaced by Rust (oleaut32-rust dispatch.rs — CreateStdDispatch/DispInvoke/DispGetIDsOfNames/DispGetParam + StdDispatch vtbl; full)
    # hash.c — replaced by Rust (oleaut32-rust hash.rs — LHashValOfNameSys A/W; full)
    # oleaut.c — replaced by Rust (oleaut32-rust oleaut.rs — BSTR SysAlloc* family + DllGetClassObject/proxy-stub + OleTranslateColor; full)
    # olefont.c — replaced by Rust (oleaut32-rust olefont.rs — IFont/IFontDisp/IPersistStream(Init)/IPersistPropertyBag + OleCreateFontIndirect; full)
    # olepicture.c — replaced by Rust (oleaut32-rust olepicture.rs — IPicture/IPersistStream + OleLoadPicture{,Ex,File,Path}/OleCreatePictureIndirect/OleSavePictureFile; full)
    # olepropframe.c — replaced by Rust (oleaut32-rust olepropframe.rs — OleCreatePropertyFrame{,Indirect}; full)
    # recinfo.c — replaced by Rust (oleaut32-rust recinfo.rs — IRecordInfo + GetRecordInfoFrom{Guids,TypeInfo}; full)
    # safearray.c — replaced by Rust (oleaut32-rust safearray.rs — SAFEARRAY incl. Create/CreateEx/CreateVector*/CopyData; full)
    # typelib.c — replaced by Rust (oleaut32-rust typelib.rs + typelib/ submodules — LoadTypeLib/RegisterTypeLib/CreateTypeLib*/DispCallFunc/ClearCustData + ITypeInfo/ITypeLib/MSFT/SLTG; full)
    # usrmarshal.c — replaced by Rust (oleaut32-rust usrmarshal.rs — VARIANT/BSTR/LPSAFEARRAY user-marshal + proxy/stub; full)
    # varformat.c — replaced by Rust (oleaut32-rust varformat.rs — VarFormat* family/VarTokenizeFormatString/VarMonthName/VarWeekdayName; full)
    # variant.c — replaced by Rust (oleaut32-rust variant.rs — VariantInit/Clear/Copy/Change + arithmetic; full)
    # vartype.c — replaced by Rust (oleaut32-rust vartype.rs — VarXxxFromYyy coercion matrix, 253 exports; full)
    # thunks.c — new upstream file (asm call_method/call_float_method/
    # call_double_method trampolines factored out of dispatch.c for
    # DispCallFunc); declined — dispatch.rs is a full Rust replacement of
    # DispCallFunc and doesn't call these.
    )

# --- Rust cohabitation (oleaut32-rust) --------------------------------------
# Build liboleaut32_rust.a from the workspace crate (STANDARD user-mode per-arch
# targets; advapi32-rust/ole32-rust pattern). The #[no_mangle] bodies replace
# the commented-out .c TUs above; lld first-match (--allow-multiple-definition)
# lets the staticlib's panic_handler/eh_personality coexist, and lets the
# partial dispatch.rs/typelib.rs win over their still-compiled C backstops.
# No RUSTC_BOOTSTRAP: oleaut32-rust builds on plain stable. Its debug shims are
# NOT C-variadic — the TRACE!/WARN!/ERR!/FIXME! macros reach them with a single
# format pointer, so there is nothing to be variadic for (same as ole32-rust).
# Defining a C-variadic fn is the ONE thing that would need nightly here; don't
# re-introduce it (stable-toolchain lane; see sdk/lib/rust/varargs-rust).
# Plan: docs/engineering/active-lanes/ole-com-translation-audit.md.
set(OLEAUT32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(OLEAUT32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-oleaut32-target")
set(OLEAUT32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/liboleaut32_rust.a")
if(ARCH STREQUAL "i386")
    set(OLEAUT32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(OLEAUT32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(OLEAUT32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(OLEAUT32_RUST_SRCS "${OLEAUT32_RUST_WORKSPACE_DIR}" oleaut32-rust)
# oleaut32 is user-mode and its VARIANT/date/currency math passes f64 by value.
# The workspace .cargo/config.toml disables SSE for the whole i686-pc-windows-gnu
# triple (a KERNEL requirement — CR4.OSFXSR isn't set until Phase 1), which makes
# rustc reject any by-value f64 ("SIMD vector type f64 requires the sse target
# feature"). Re-enable SSE for THIS user-mode build only (RUSTFLAGS overrides the
# per-target config rustflags). The extern "system" FFI return ABI (float in st0)
# is unaffected — SSE only changes internal codegen. amd64/arm64 already keep
# SSE/NEON, so no override is needed there.
set(OLEAUT32_RUST_ENV CARGO_TARGET_DIR=${OLEAUT32_RUST_TARGET_DIR})
if(ARCH STREQUAL "i386")
    list(APPEND OLEAUT32_RUST_ENV "RUSTFLAGS=-C target-feature=+sse,+sse2")
endif()
add_custom_command(
    OUTPUT ${OLEAUT32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        ${OLEAUT32_RUST_ENV}
        cargo build --release --target ${OLEAUT32_RUST_TARGET}
            -p oleaut32-rust
            --manifest-path ${OLEAUT32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${OLEAUT32_RUST_TARGET_DIR}/${OLEAUT32_RUST_TARGET}/release/liboleaut32_rust.a
        ${OLEAUT32_RUST_LIB}
    DEPENDS ${OLEAUT32_RUST_SRCS}
            ${OLEAUT32_RUST_WORKSPACE_DIR}/oleaut32-rust/Cargo.toml
    WORKING_DIRECTORY ${OLEAUT32_RUST_WORKSPACE_DIR}
    COMMENT "Building oleaut32-rust (${OLEAUT32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_oleaut32 DEPENDS ${OLEAUT32_RUST_LIB})
# -----------------------------------------------------------------------------

list(APPEND PCH_SKIP_SOURCE
    ${CMAKE_CURRENT_BINARY_DIR}/oleaut32_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/proxy.dlldata.c
    ${CMAKE_CURRENT_BINARY_DIR}/oleaut32_oaidl_p.c
    ${CMAKE_CURRENT_BINARY_DIR}/oleaut32_ocidl_p.c)

add_library(oleaut32 MODULE
    ${SOURCE}
    ${oleaut32_asm}
    ${PCH_SKIP_SOURCE}
    oleaut32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/oleaut32.def)

if(CMAKE_C_COMPILER_ID STREQUAL "GNU")
    target_compile_options(oleaut32 PRIVATE -Wno-misleading-indentation)
endif()

add_idl_headers(oleaut32_idlheader oleaut32_oaidl.idl)
add_dependencies(oleaut32 oleaut32_idlheader rust_oleaut32)
# Upstream's wine-10.0 sync (e05d78ede06) took resources 2/3 (the hand-written
# oleaut32_oaidl.rgs / oleaut32_ocidl.rgs WINE_REGISTRY scripts) out of
# oleaut32.rc and generates them from the idl here instead. The September merge
# kept the .rc change and lost this line, so DllRegisterServer registered no
# StdFont/StdPicture/PSFactoryBuffer classes and CoGetClassObject(CLSID_StdFont)
# failed REGDB_E_CLASSNOTREG (Wine's olefont/olepicture tests).
add_idl_reg_scripts(oleaut32 registry oleaut32_oaidl.idl oleaut32_ocidl.idl)
set_module_type(oleaut32 win32dll)
# OLEAUT32_RUST_LIB first so its #[no_mangle] bodies win over any lingering C.
target_link_libraries(oleaut32 ${OLEAUT32_RUST_LIB} wine wineldr uuid ${PSEH_LIB} oldnames)
target_link_options(oleaut32 PRIVATE LINKER:--allow-multiple-definition)
add_delay_importlibs(oleaut32 comctl32 urlmon windowscodecs)
add_importlibs(oleaut32 ole32 rpcrt4 user32 gdi32 advapi32 kernel32_vista msvcrt kernel32 ntdll)
add_pch(oleaut32 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET oleaut32 DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(oleaut32) # CORE-5743: No ARRAY_SIZE and CONST_VTABLE
