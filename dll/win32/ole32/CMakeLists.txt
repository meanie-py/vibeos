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
# CoGetCurrentProcess asks rpcss for a machine-wide thread sequence id (wine-10 combase).
add_rpc_files(client ${REACTOS_SOURCE_DIR}/sdk/include/wine/irpcss.idl)

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

# The stub manager's guarded object release and the guarded irpcss calls (see each
# file's header): a real __try on amd64/arm64; i386 uses ntdll's RtlSehTryExcept trampoline.
if(ARCH STREQUAL "amd64" OR ARCH STREQUAL "arm64")
    set_source_files_properties(stubmanager_seh.c PROPERTIES COMPILE_DEFINITIONS _USE_NATIVE_SEH)
    set_source_files_properties(rpcss_seh.c PROPERTIES COMPILE_DEFINITIONS _USE_NATIVE_SEH)
endif()

# --- Rust cohabitation (ole32-rust) -----------------------------------------
# Build libole32_rust.a from the workspace crate (STANDARD user-mode per-arch
# targets; advapi32-rust pattern). The #[no_mangle] bodies replace the
# commented-out .c TUs above; lld first-match (--allow-multiple-definition)
# lets the staticlib's panic_handler/eh_personality coexist with the C cohab.
# Plan: docs/engineering/active-lanes/ole-com-translation-audit.md.

# -----------------------------------------------------------------------------

# The proxy/stub registration (every Interface\{iid}\ProxyStubClsid32 that CoGetPSClsid reads)
# is generated from the IDL: upstream 7675017f60d deleted the hand-written .rgs resources from
# ole32res.rc and added this line, and the upstream sync kept the .rc change but lost the line.
# DllRegisterServer then wrote nothing and returned S_OK, so no interface could be marshalled.
add_idl_reg_objects(ole32 registry dcom.idl ole32_objidl.idl ole32_oleidl.idl ole32_unknwn.idl)

# ole32.dll is all Rust (ole32-rust) and cargo links it:
# sdk/lib/rust/ole32-cargo (reactos-link, generated by tools/cargo_cell_gen.py).
# The C stays on disk, uncompiled.
# The C still compiled, in an OBJECT library whose compile the cargo link replays.
# How cmake compiled the target's sources, now on the OBJECT libraries.
# Generated inputs those compiles read, ordered before them.
add_library(ole32_shims OBJECT stubmanager_seh.c rpcss_seh.c guid.c ${CMAKE_CURRENT_BINARY_DIR}/dcom_p.c ${CMAKE_CURRENT_BINARY_DIR}/ole32_objidl_p.c ${CMAKE_CURRENT_BINARY_DIR}/ole32_oleidl_p.c ${CMAKE_CURRENT_BINARY_DIR}/ole32_unknwn_p.c ${CMAKE_CURRENT_BINARY_DIR}/proxy.dlldata.c ${CMAKE_CURRENT_BINARY_DIR}/dcom_i.c ${CMAKE_CURRENT_BINARY_DIR}/irot_c.c ${CMAKE_CURRENT_BINARY_DIR}/irpcss_c.c)
target_link_libraries(ole32_shims PRIVATE wine uuid oldnames libadvapi32 libuser32 libgdi32 librpcrt4 libmsvcrt libkernel32 libkernel32_vista libntdll)
set_wine_module_fixme(ole32_shims) # CORE-5743: No ARRAY_SIZE and CONST_VTABLE
add_dependencies(ole32_shims ole32idl)
set(OLE32_REUSED
    ${CMAKE_BINARY_DIR}/dll/win32/ole32/dcom_r.o
    ${CMAKE_BINARY_DIR}/dll/win32/ole32/ole32_objidl_r.o
    ${CMAKE_BINARY_DIR}/dll/win32/ole32/ole32_oleidl_r.o
    ${CMAKE_BINARY_DIR}/dll/win32/ole32/ole32_unknwn_r.o)
add_cargo_module(ole32 ole32-cargo ole32.dll
    KIND win32dll
    BODY_CRATE ole32-rust
    DESTINATION reactos/system32
    DELAY_IMPORTS
    EXTRA_DEPENDS ole32_shims psdk ole32_r ${OLE32_REUSED} uuid_sources)
