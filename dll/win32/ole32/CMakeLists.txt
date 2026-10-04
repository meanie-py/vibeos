
remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

add_definitions(
    -D_OLE32_
    -D__ROS_LONG64__
    -DCOM_NO_WINDOWS_H
    -DENTRY_PREFIX=OLE32_
    -DPROXY_CLSID=CLSID_PSFactoryBuffer
    -DWINE_REGISTER_DLL
    -D_ROAPI_)

spec2def(ole32.dll ole32.spec ADD_IMPORTLIB NO_PRIVATE_WARNINGS)

generate_idl_iids(dcom.idl)
add_idl_headers(ole32idl dcom.idl irot.idl)
add_rpc_files(client irot.idl)

add_rpcproxy_files(
    dcom.idl
    ole32_unknwn.idl
    ole32_objidl.idl
    ole32_oleidl.idl)

list(APPEND SOURCE
    # antimoniker.c — replaced by Rust (ole32-rust ring/antimoniker.rs — IMoniker/IPersistStream/IROTData)
    # bindctx.c — replaced by Rust (ole32-rust ring/bindctx.rs — IBindCtx + bind options)
    # classmoniker.c — replaced by Rust (ole32-rust ring/classmoniker.rs — IMoniker over a CLSID)
    # clipboard.c — replaced by Rust (ole32-rust ring/clipboard.rs — OleSet/Get/FlushClipboard + the IDataObject wrapper)
    # comcat.c — replaced by Rust (ole32-rust ring/comcat.rs — ICatRegister/ICatInformation)
    # compobj.c — replaced by Rust (ole32-rust compobj.rs — COM runtime: CoInitialize/apartments/activation/DllMain)
    # compositemoniker.c — replaced by Rust (ole32-rust ring/compositemoniker.rs — the generic composite)
    # datacache.c — replaced by Rust (ole32-rust ring/datacache.rs — the OLE presentation cache: IOleCache2/IViewObject2/IDataObject over cached DIB+metafile presentations)
    # defaulthandler.c — replaced by Rust (ole32-rust ring/defaulthandler.rs — the default object handler: IOleObject/IRunnableObject/IPersistStorage for an embedded object)
    # dictionary.c — replaced by Rust (ole32-rust ring/dictionary.rs — the comcat dictionary helper)
    # enumx.c — replaced by Rust (ole32-rust ring/enumx.rs — the generic IEnumXXX helper)
    # errorinfo.c — replaced by Rust (ole32-rust ring/errorinfo.rs — IErrorInfo/ICreateErrorInfo + the TLS slot)
    # filelockbytes.c — replaced by Rust (ole32-rust ring/filelockbytes.rs — ILockBytes over a file handle)
    # filemoniker.c — replaced by Rust (ole32-rust ring/filemoniker.rs — IMoniker over a path)
    # ftmarshal.c — replaced by Rust (ole32-rust ftmarshal.rs — free-threaded marshaler)
    # git.c — replaced by Rust (ole32-rust git.rs — global interface table)
    # hglobalstream.c — replaced by Rust (ole32-rust ring/hglobalstream.rs — CreateStreamOnHGlobal)
    # ifs.c — replaced by Rust (ole32-rust ring/ifs.rs — CoGetMalloc/CoTaskMemAlloc/Free/Realloc + IMallocSpy)
    # itemmoniker.c — replaced by Rust (ole32-rust ring/itemmoniker.rs — IMoniker over an item name)
    # marshal.c — replaced by Rust (ole32-rust marshal.rs — proxy manager + CoMarshal* family)
    # memlockbytes.c — replaced by Rust (ole32-rust ring/memlockbytes.rs — ILockBytes over an HGLOBAL)
    # moniker.c — replaced by Rust (ole32-rust ring/moniker.rs — the Running Object Table + MonikerMarshal)
    # ole2.c — replaced by Rust (ole32-rust ring/ole2.rs — OleInitialize/DoDragDrop/RegisterDragDrop)
    # ole2impl.c — replaced by Rust (ole32-rust ring/ole2impl.rs — OleCreate/OleCreateFromData)
    # ole2stubs.c — replaced by Rust (ole32-rust ring/ole2stubs.rs — small OLE stubs)
    # ole32_main.c — replaced by Rust (ole32-rust ring/ole32_main.rs — DllMain + OleMetafilePictFromIconAndLabel)
    # oleobj.c — replaced by Rust (ole32-rust ring/oleobj.rs — IOleAdviseHolder/IDataAdviseHolder + IEnumSTATDATA)
    # oleproxy.c — replaced by Rust (ole32-rust ring/oleproxy.rs — the proxy/stub class factories)
    # pointermoniker.c — replaced by Rust (ole32-rust ring/pointermoniker.rs — IMoniker over an IUnknown*)
    # rpc.c — replaced by Rust (ole32-rust rpc.rs — ORPC channel + apartment pumps)
    # stg_prop.c — replaced by Rust (ole32-rust ring/stg_prop.rs — IPropertyStorage/IPropertySetStorage, the property-set on-disk format)
    # stg_stream.c — replaced by Rust (ole32-rust ring/stg_stream.rs — IStream over a stream inside a compound file)
    # storage32.c — replaced by Rust (ole32-rust ring/storage32.rs — the compound-file implementation: IStorage, the dir-entry tree, block chains, transactions)
    # stubmanager.c — replaced by Rust (ole32-rust stubmanager.rs — stub/ifstub lifetime)
    # usrmarshal.c — replaced by Rust (ole32-rust ring/usrmarshal.rs — the NDR *_User{Size,Marshal,Unmarshal,Free} helpers)
    )

# The stub manager's guarded object release (see the file header): a real __try on
# amd64/arm64; i386 keeps the DUMMY pass-through.
if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
    set_source_files_properties(stubmanager_seh.c PROPERTIES COMPILE_DEFINITIONS _USE_NATIVE_SEH)
endif()

list(APPEND PCH_SKIP_SOURCE
    stubmanager_seh.c
    guid.c
    ${CMAKE_CURRENT_BINARY_DIR}/dcom_p.c
    ${CMAKE_CURRENT_BINARY_DIR}/ole32_objidl_p.c
    ${CMAKE_CURRENT_BINARY_DIR}/ole32_oleidl_p.c
    ${CMAKE_CURRENT_BINARY_DIR}/ole32_unknwn_p.c
    ${CMAKE_CURRENT_BINARY_DIR}/ole32_stubs.c
    ${CMAKE_CURRENT_BINARY_DIR}/proxy.dlldata.c)


# --- Rust cohabitation (ole32-rust) -----------------------------------------
# Build libole32_rust.a from the workspace crate (STANDARD user-mode per-arch
# targets; advapi32-rust pattern). The #[no_mangle] bodies replace the
# commented-out .c TUs above; lld first-match (--allow-multiple-definition)
# lets the staticlib's panic_handler/eh_personality coexist with the C cohab.
# Plan: docs/engineering/active-lanes/ole-com-translation-audit.md.
set(OLE32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(OLE32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-ole32-target" ole32-rust)
set(OLE32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libole32_rust.a")
if(ARCH STREQUAL "i386")
    set(OLE32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(OLE32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(OLE32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(OLE32_RUST_SRCS "${OLE32_RUST_WORKSPACE_DIR}" ole32-rust)
add_custom_command(
    OUTPUT ${OLE32_RUST_LIB}
    # No RUSTC_BOOTSTRAP: ole32-rust builds on plain stable. The ring's debug shims are
    # deliberately NOT C-variadic (see debug_shims.rs) precisely so this stays true —
    # the stable-toolchain lane removed the nightly-feature surface workspace-wide and
    # this crate must not re-introduce it.
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${OLE32_RUST_TARGET_DIR}
        cargo build --release --target ${OLE32_RUST_TARGET}
            -p ole32-rust
            --manifest-path ${OLE32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${OLE32_RUST_TARGET_DIR}/${OLE32_RUST_TARGET}/release/libole32_rust.a
        ${OLE32_RUST_LIB}
    DEPENDS ${OLE32_RUST_SRCS}
            ${OLE32_RUST_WORKSPACE_DIR}/ole32-rust/Cargo.toml
    WORKING_DIRECTORY ${OLE32_RUST_WORKSPACE_DIR}
    COMMENT "Building ole32-rust (${OLE32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_ole32 DEPENDS ${OLE32_RUST_LIB})
# -----------------------------------------------------------------------------

add_library(ole32 MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    ole32res.rc
    ${CMAKE_CURRENT_BINARY_DIR}/dcom_i.c
    ${CMAKE_CURRENT_BINARY_DIR}/irot_c.c
    ${CMAKE_CURRENT_BINARY_DIR}/ole32.def)

set_module_type(ole32 win32dll)
# The proxy/stub registration (every Interface\{iid}\ProxyStubClsid32 that CoGetPSClsid reads)
# is generated from the IDL: upstream 7675017f60d deleted the hand-written .rgs resources from
# ole32res.rc and added this line, and the upstream sync kept the .rc change but lost the line.
# DllRegisterServer then wrote nothing and returned S_OK, so no interface could be marshalled.
add_idl_reg_scripts(ole32 registry dcom.idl ole32_objidl.idl ole32_oleidl.idl ole32_unknwn.idl)
# OLE32_RUST_LIB first so its #[no_mangle] bodies win over any lingering C.
target_link_libraries(ole32 ${OLE32_RUST_LIB} wine uuid ${PSEH_LIB} oldnames)
target_link_options(ole32 PRIVATE LINKER:--allow-multiple-definition)
# oleaut32's delay-import lib is ONE object holding a thunk for every oleaut32 export,
# including the three it forwards back to us (`-import` in oleaut32.spec). It gets
# loaded for SysAllocString long before the .def exports are resolved, so its thunks
# took Get/Set/CreateErrorInfo and the Rust errorinfo bodies were never linked: ole32
# exported jumps into oleaut32, which jumped straight back -- an endless tail-call
# loop for any caller. Make them undefined up front so the Rust archive, read first,
# supplies them. `-u` will not do: lld's MinGW driver maps it to -includeoptional,
# resolved too late to matter; --require-defined is a real -include.
# (--allow-multiple-definition is why the clash was silent.)
if(ARCH STREQUAL "i386")
    target_link_options(ole32 PRIVATE "LINKER:--require-defined=_CreateErrorInfo@4,--require-defined=_GetErrorInfo@8,--require-defined=_SetErrorInfo@8")
else()
    target_link_options(ole32 PRIVATE "LINKER:--require-defined=CreateErrorInfo,--require-defined=GetErrorInfo,--require-defined=SetErrorInfo")
endif()
add_delay_importlibs(ole32 oleaut32)
add_importlibs(ole32 advapi32 user32 gdi32 rpcrt4 msvcrt kernel32 kernel32_vista ntdll)
add_dependencies(ole32 ole32idl rust_ole32)
add_pch(ole32 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET ole32 DESTINATION reactos/system32 FOR all)
set_wine_module_FIXME(ole32) # CORE-5743: No ARRAY_SIZE and CONST_VTABLE
