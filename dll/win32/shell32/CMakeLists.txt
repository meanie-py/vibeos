PROJECT(SHELL)

add_subdirectory(shelldesktop)
add_subdirectory(shellmenu)
add_subdirectory(shellrecyclebin)

spec2def(shell32.dll shell32.spec ADD_IMPORTLIB)

remove_definitions(-D_WIN32_WINNT=0x502)
add_definitions(-D_WIN32_WINNT=0x600)

add_definitions(
    -D_SHELL32_
    -D_WINE)

if(DBG)
    add_definitions(-D_DEBUG=1) # CORE-17505
endif()

list(APPEND SOURCE
    # brfolder.cpp — replaced by Rust (shell32-rust/src/shell32/brfolder.rs); cohab via libshell32_rust.a
    # CActiveDesktop.cpp — replaced by Rust (shell32-rust/src/active_desktop.rs).
    CActiveDesktop.h
    CIDLDataObj.cpp
    # CQueryAssociations.cpp — class plus association description helpers are Rust-owned
    # (shell32-rust/src/query_associations.rs).
    # changenotify.cpp — replaced by Rust (shell32-rust/src/shell32/changenotify.rs); cohab via
    # libshell32_rust.a. Its one ATL CWindowImpl<> (CChangeNotifyBroker) could not come along,
    # and neither could the CreateNotificationBroker that news and subclasses it; both live in
    # rust_changenotify_broker_stub.cpp below.
    rust_changenotify_broker_stub.cpp
    # debughlp.cpp — replaced by Rust (shell32-rust/src/debughlp.rs); cohab via libshell32_rust.a
    # dialogs/dialogs.cpp — replaced by Rust (shell32-rust/src/dialogs/dialogs.rs); cohab via
    # libshell32_rust.a. Whole-TU flip: of its 145 defined symbols only ExitWindowsDialog and
    # PickIconDlg had callers outside it, and Rust exports both. 24 of its 26 functions came
    # from the paid bank at $0; FillList and RunDlgProc are hand-written because the call-seal
    # audit refused both, twice each, on different tokens — see the module for what it caught.
    # dialogs/drive.cpp — replaced by Rust (shell32-rust/src/dialogs/drive.rs); cohab via libshell32_rust.a
    # dialogs/drvdefext.cpp — replaced by Rust; cohab via libshell32_rust.a. Whole-TU flip: the
    # CDrvDefExt class was already shell32-rust/src/dialogs/drvdefext.rs, and its retained
    # closure was 3 symbols — ChkDskDlg_RustBridge and SH_FormatByteSize_RustBridge plus the
    # C++-mangled IsDriveFloppyW. The free helpers are now
    # shell32-rust/src/dialogs/drvdefext_tu.rs. Two C++ edits came with it: IsDriveFloppyW's
    # local declaration in folders/CDrivesFolder.cpp is `extern "C"` now, and the
    # SH_FormatByteSize forwarder MOVED to dialogs/filedefext.cpp — the TU that owns the
    # function it forwards to — rather than being published from Rust, which would have made
    # its body a call to itself.
    # dialogs/filedefext.cpp — replaced by Rust; cohab via libshell32_rust.a. Whole-TU flip: the
    # CFileDefExt and CFileVersionInfo classes were already shell32-rust/src/dialogs/filedefext.rs,
    # and the free helpers this file retained are now shell32-rust/src/dialogs/filedefext_tu.rs.
    # Its retained closure was 10 symbols, all `_RustBridge` free functions the Rust now publishes
    # — including the SH_FormatByteSize forwarder that moved here when drvdefext.cpp flipped, which
    # leaves with this file. Three C++ edits came with it: PathIsDotOrDotDotW, SHELL_DisplayNameOf
    # and SHELL32_GetDllFromRundll32CommandLine are EXTERN_C in precomp.h now, because the
    # C-linkage forwarders that used to reach those C++ functions lived HERE and went away with
    # the file; that is the same fix already documented for the DataObject_GetHIDACount group.
    dialogs/filetypes.cpp
    # dialogs/folder_options.cpp — replaced by Rust (shell32-rust/src/dialogs/folder_options.rs);
    # cohab via libshell32_rust.a. Its one ATL CWindowImpl<> could not come along and lives in
    # rust_folder_options_stub.cpp below.
    rust_folder_options_stub.cpp
    # dialogs/item_prop.cpp — replaced by Rust (shell32-rust/src/dialogs/item_prop.rs); cohab
    # via libshell32_rust.a. Its ShellPropSheetDialog "class" is a namespace of statics, so the
    # four of them are plain free functions on the Rust side.
    # dialogs/general.cpp — replaced by Rust (shell32-rust/src/dialogs/general.rs); cohab via libshell32_rust.a
    # dialogs/recycler_prop.cpp — replaced by Rust (shell32-rust/src/dialogs/recycler_prop.rs); cohab via libshell32_rust.a
    # dialogs/view.cpp — replaced by Rust (shell32-rust/src/dialogs/view.rs); cohab via libshell32_rust.a
    # CDropTargetHelper.cpp — COM class plus SHSimulateDropOnClsid ordinal closure
    # are Rust-owned (shell32-rust/src/drop_target_helper.rs).
    CEnumIDListBase.cpp
    CExtractIcon.cpp
    # folders.cpp — replaced by Rust (shell32-rust/src/folders.rs); cohab via libshell32_rust.a
    # iconcache.cpp — replaced by Rust (shell32-rust/src/shell32/iconcache.rs); cohab via libshell32_rust.a
    # propsheet.cpp — replaced by Rust (shell32-rust/src/propsheet.rs); reaches
    #   CStubWindow32 via CStubWindow32_*_RustBridge in utils.cpp
    shell32.cpp
    utils.cpp
    # CShellItem.cpp — CShellItem, CShellItemArray, SHCreateShellItem, and
    # SHCreateShellItemArrayFromDataObject are Rust-owned.
    # CShellLink.cpp — removed. The COM class was already Rust-owned
    # (shell32-rust/src/shell_link.rs): activation, all interfaces, and the
    # C-linkage IShellLink_ConstructFromPath/FromFile. What kept the file
    # compiled was believed to be its free helpers, but nothing referenced them:
    # llvm-nm shows the object defining 273 symbols (including all 75 mangled
    # CShellLink methods) with ZERO of them undefined in any other shell32 object
    # or in libshell32_rust.a, and no shell32.spec export coming from it.
    # PathFullyUnExpandEnvStringsW, named in the old note, is inside `#if 0`.
    # CShellLinkRustEntry derives from CShellLink only to give ATL's OBJECT_ENTRY
    # a _CreatorClass forwarding to the Rust factory; it is never instantiated, so
    # no vtable is emitted — shell32.cpp.obj's only undefined CShellLink symbol is
    # CShellLink_CreateInstance, the Rust one.
    # folders/CDesktopFolder.cpp — replaced by Rust; cohab via libshell32_rust.a. Whole-TU
    # flip: the activated class was already shell32-rust/src/desktop_folder.rs; its retained
    # closure was 4 extern_c bridges. The free helpers are
    # shell32-rust/src/folders/CDesktopFolder.rs and the enumerator is
    # shell32-rust/src/folders/CDesktopFolderEnum.rs. CDesktopFolderViewCB — the
    # IShellFolderViewCB/IFolderFilter callback the desktop view runs on — had no Rust form at
    # all and was translated for this flip. CRegFolder_CreateInstance needed EXTERN_C in
    # shfldr.h: it is C++-mangled otherwise and no Rust declaration can name it.
    # folders/CFSFolder.cpp — replaced by Rust; cohab via libshell32_rust.a. The activated class
    # had already moved to shell32-rust/src/cfs_folder.rs; what kept the TU alive was its retained
    # closure, measured against the real link as 21 symbols — the 14 *_RustBridge C entry points
    # (now shell32-rust/src/cfs_folder_tu.rs) plus 7 C++-mangled ones. The two free functions of
    # those 7 became EXTERN_C in shfldr.h; the five CFSFolder:: statics became inline forwarders
    # to the bridges in CFSFolder.h, because a static MEMBER has no EXTERN_C spelling. The
    # file-local CFileSysEnum enumerator is shell32-rust/src/file_sys_enum.rs.
    # Hybrid cohab: CDrivesFolder activation/methods are Rust-owned
    # (shell32-rust/src/drives_folder.rs); this file retains CDrivesFolderEnum,
    # the drives context menu/eject/format cluster, CDrivesExtractIcon + icon
    # helpers, and the explicit *_RustBridge shims.
    folders/CDrivesFolder.cpp
    # folders/CMyDocsFolder.cpp — replaced by Rust (shell32-rust/src/my_docs_folder.rs); cohab via libshell32_rust.a
    # Hybrid cohab: CPrinterFolder activation/methods are Rust-owned; this file
    # retains CPrintersEnum and three explicit source-local Rust bridge helpers.
    # folders/CPrinterFolder.cpp — CPrinterFolder, CPrintersEnum, and retained
    # printer icon/PIDL helpers are Rust-owned.
    # folders/CAdminToolsFolder.cpp — replaced by Rust (shell32-rust/src/admin_tools_folder.rs); cohab via libshell32_rust.a
    # folders/CNetFolder.cpp — replaced by Rust; cohab via libshell32_rust.a. Whole-TU flip: the
    # activated class was already shell32-rust/src/net_folder.rs; its retained closure, measured
    # against the real link by `rustconv hybrid-closure`, was 7 *_RustBridge entry points and
    # nothing else — no C++-mangled symbol at all. Those are now
    # shell32-rust/src/folders/CNetFolder.rs, and the file-local CNetFolderEnum enumerator is
    # shell32-rust/src/folders/CNetFolderEnum.rs.
    # folders/CFontsFolder.cpp — replaced by Rust (shell32-rust/src/fonts_folder.rs); cohab via libshell32_rust.a
    # folders/CControlPanelFolder.cpp — replaced by Rust; cohab via libshell32_rust.a.
    # Whole-TU flip: the activated class was already shell32-rust/src/control_panel_folder.rs;
    # its retained closure was 4 extern_c bridges. This TU needed THREE slices, so its shared
    # preamble (shvheader, ControlPanelSFHeader, ...) is owned by
    # shell32-rust/src/folders/CControlPanelFolder.rs and imported by the other two —
    # CControlPanelEnum.rs and CCPLItemMenu.rs — rather than re-declared, which would make one
    # layout into three distinct nominal types.
    folders/CRecycleBin.cpp
    folders/CRecycleBinFolderViewCB.cpp
    folders/CRegFolder.cpp
    # droptargets/CexeDropHandler.cpp — replaced by Rust
    # (shell32-rust/src/exe_drop_handler.rs).
    droptargets/CFSDropTarget.cpp
    droptargets/CRecyclerDropTarget.cpp
    # shldataobject.cpp — replaced by Rust (shell32-rust/src/shell32/shldataobject.rs); cohab via libshell32_rust.a
    # shlexec.cpp — replaced by Rust (shell32-rust/src/shell32/shlexec.rs); cohab via libshell32_rust.a
    # shlfileop.cpp — replaced by Rust (shell32-rust/src/shell32/shlfileop.rs); cohab via libshell32_rust.a
    # shlfolder.cpp — replaced by Rust (shell32-rust/src/shell32/shlfolder.rs); cohab via libshell32_rust.a
    CFileSysBindData.cpp
    CDefView.cpp
    CDefViewUtil.cpp
    # CDefViewDual.cpp — replaced by Rust (shell32-rust/src/shell32/CDefViewDual.rs); cohab via
    # libshell32_rust.a. Whole-TU flip: the only symbol the rest of the link still took from it
    # was CDefViewDual_Constructor (CDefView.cpp:3583 is the sole caller), and Rust publishes
    # that now. Its IDispatchImpl<> template arguments — plibid/piid/wMajor/wMinor — are baked
    # into the Rust translation, since a template reaches no TU and so has no body to translate.
    CDefViewBckgrndMenu.cpp
    # stubs.cpp — replaced by Rust (shell32-rust/src/shell32/stubs.rs); cohab via libshell32_rust.a
    # systray.cpp — replaced by Rust (shell32-rust/src/systray.rs); cohab via libshell32_rust.a
    # CUserNotification.cpp — replaced by Rust (shell32-rust/src/user_notification.rs); cohab via libshell32_rust.a
    CDefaultContextMenu.cpp
    # Hybrid cohab: COpenWithMenu activation/methods are Rust-owned; this file
    # retains COpenWithList, COpenWithDialog, exported helpers, and opaque bridges.
    COpenWithMenu.cpp
    # CNewMenu.cpp — replaced by Rust (shell32-rust/src/new_menu.rs); cohab via libshell32_rust.a
    # CSendToMenu.cpp — replaced by Rust (shell32-rust/src/send_to_menu.rs); cohab via libshell32_rust.a
    # CCopyMoveToMenu.cpp — shared base, four file-local helpers/callbacks, and
    # both CCopyToMenu/CMoveToMenu derived objects are Rust-owned.
    # CCopyAsPathMenu.cpp — replaced by Rust (shell32-rust/src/copy_as_path_menu.rs); cohab via libshell32_rust.a
    # CShellDispatch.cpp — replaced by Rust (shell32-rust/src/shell_dispatch.rs);
    # CShellDispatch_Constructor remains exported by the Rust module for CFolder/CDefViewDual.
    CFolder.cpp
    CFolderItems.cpp
    CFolderItemVerbs.cpp)

list(APPEND PCH_SKIP_SOURCE
    # appbar.c — replaced by Rust (shell32-rust/src/appbar.rs); cohab via libshell32_rust.a
    # wine/classes.c — replaced by Rust (shell32-rust/src/wine/classes.rs); cohab via libshell32_rust.a
    # wine/clipboard.c — replaced by Rust (shell32-rust/src/wine/clipboard.rs); cohab via libshell32_rust.a
    # wine/control.c — replaced by Rust (shell32-rust/src/wine/control.rs); cohab via libshell32_rust.a
    # wine/pidl.c — replaced by Rust (shell32-rust/src/wine/pidl.rs); cohab via libshell32_rust.a
    # wine/shell32_main.c — replaced by Rust (shell32-rust/src/wine/shell32_main.rs); cohab via libshell32_rust.a
    # wine/shellole.c — replaced by Rust (shell32-rust/src/wine/shellole.rs); cohab via libshell32_rust.a
    # wine/shellord.c — replaced by Rust (shell32-rust/src/wine/shellord.rs); cohab via libshell32_rust.a
    # wine/shellpath.c — replaced by Rust (shell32-rust/src/wine/shellpath.rs); cohab via libshell32_rust.a
    # wine/shellreg.c — replaced by Rust (shell32-rust/src/wine/shellreg.rs); cohab via libshell32_rust.a
    # wine/shellstring.c — replaced by Rust (shell32-rust/src/wine/shellstring.rs); cohab via libshell32_rust.a
    # wine/shlmenu.c — replaced by Rust (shell32-rust/src/wine/shlmenu.rs); cohab via libshell32_rust.a
    # wine/shpolicy.c — replaced by Rust (shell32-rust/src/wine/shpolicy.rs); cohab via libshell32_rust.a
    # vista.c — replaced by Rust (shell32-rust/src/vista.rs); cohab via libshell32_rust.a
    #
    # rust_varargs_shim.c — the ONE thing stable Rust cannot do: DEFINE the C-variadic
    # ShellMessageBoxA entry point (`va_start`; exported ordinal 183, so the symbol name +
    # cdecl variadic ABI must be preserved). It trampolines straight into shell32-rust's
    # fixed-arity ShellMessageBoxA_v, which keeps ALL the logic. This is what lets
    # shell32-rust build on plain stable rustc — no bootstrap escape hatch on the cargo
    # env line below, no nightly feature gate in the crate root.
    # Plain C: belongs in PCH_SKIP_SOURCE (precomp.h is the C++ PCH), like shell32_stubs.c.
    rust_varargs_shim.c
    #
    # rust_strsafe_shim.c — linkage, not language: strsafe.h's StringCchPrintfW is a `static
    # __inline`, so it is a PHANTOM SYMBOL with nothing for Rust to bind to. One forwarding
    # trampoline into the header's own StringCchVPrintfW. Its own TU because rust_varargs_shim.c
    # deliberately includes no Windows headers. Also plain C, so also PCH_SKIP_SOURCE.
    rust_strsafe_shim.c
    ${CMAKE_CURRENT_BINARY_DIR}/shell32_stubs.c)

file(GLOB_RECURSE shell32_rc_deps res/*.*)
list(APPEND shell32_rc_deps ${CMAKE_CURRENT_BINARY_DIR}/shell32_shldisp.tlb)
add_rc_deps(shell32.rc ${shell32_rc_deps})

# --- shell32-rust cohab staticlib (libshell32_rust.a) ----------------------------------
# Rust translation of the shell32 wine-derived .c sources, built as a staticlib and cohab-
# linked into shell32.dll with the translated .c commented out of PCH_SKIP_SOURCE above
# (per-file flip, mirrors dll/win32/gdi32 / kernel32 / win32ss/user/user32). User-mode
# STANDARD per-arch targets; i386 gets -sse,-sse2 via sdk/lib/rust/.cargo/config.toml
# (user-mode no-SSE rule). The crate's #[no_mangle] symbols are the sole providers of the
# commented-out .c functions, so no --allow-multiple-definition is needed. Imports (Reg*/
# SHQueryValueEx*/...) resolve via shell32's existing advapi32/shlwapi/kernel32 importlibs.
set(SHELL32_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
set(SHELL32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-shell32-target")
set(SHELL32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libshell32_rust.a")
if(ARCH STREQUAL "i386")
    set(SHELL32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SHELL32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SHELL32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
# GLOB_RECURSE so .rs under src/<subdir>/ also retrigger the libshell32_rust.a rebuild.
file(GLOB_RECURSE SHELL32_RUST_SRCS "${SHELL32_RUST_WORKSPACE_DIR}/shell32-rust/src/*.rs")
add_custom_command(
    OUTPUT ${SHELL32_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${SHELL32_RUST_TARGET_DIR}
        cargo build --release --target ${SHELL32_RUST_TARGET}
            -p shell32-rust
            --manifest-path ${SHELL32_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${SHELL32_RUST_TARGET_DIR}/${SHELL32_RUST_TARGET}/release/libshell32_rust.a
        ${SHELL32_RUST_LIB}
    DEPENDS ${SHELL32_RUST_SRCS}
            ${SHELL32_RUST_WORKSPACE_DIR}/shell32-rust/Cargo.toml
    WORKING_DIRECTORY ${SHELL32_RUST_WORKSPACE_DIR}
    COMMENT "Building shell32-rust (${SHELL32_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_shell32 DEPENDS ${SHELL32_RUST_LIB})

add_library(shell32 MODULE
    ${SOURCE}
    ${PCH_SKIP_SOURCE}
    shell32.rc
    ${CMAKE_CURRENT_BINARY_DIR}/shell32.def)

if(MSVC)
    # error C4311: 'type cast': pointer truncation from 'HANDLE' to 'INT'
    remove_target_compile_option(shell32 "/we4311")
endif()

add_typelib(shell32_shldisp.idl)
set_source_files_properties(shell32.rc PROPERTIES OBJECT_DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/shell32_shldisp.tlb)

set_module_type(shell32 win32dll UNICODE)
target_link_libraries(shell32 shellmenu shelldesktop wine uuid recyclebin cpprt atl_classes oldnames ${SHELL32_RUST_LIB} ${MENUBAND_RUST_LIB})
add_dependencies(shell32 rust_menuband)
add_delay_importlibs(shell32 powrprof shdocvw devmgr winspool.drv winmm mpr uxtheme ole32 oleaut32 userenv browseui version fmifs netapi32 secur32)
add_importlibs(shell32 advapi32 gdi32 user32 comctl32 comdlg32 shlwapi msvcrt kernel32 ntdll)
add_dependencies(shell32 stdole2 rust_shell32) # shell32_shldisp.tlb needs stdole2.tlb; rust_shell32 = cohab staticlib
add_pch(shell32 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET shell32 DESTINATION reactos/system32 FOR all)
