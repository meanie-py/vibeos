PROJECT(SHELL)

add_subdirectory(evalcmd)
add_subdirectory(shelldesktop)
add_subdirectory(shellmenu)
# shellrecyclebin/ — the WHOLE object library is Rust now (sdk/lib/rust/recyclebin-rust,
# linked as ${RECYCLEBIN_RUST_LIB} below). Its six sources were the entire `recyclebin`
# static library, so the target goes away rather than being emptied; the .c/.cpp stay on
# disk, inert, like every other cohab flip.
# add_subdirectory(shellrecyclebin)

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
    # CIDLDataObj.cpp — replaced by Rust: shell32/CIDLDataObj.rs (the IDataObject/
    # IAsyncOperation), shell32/IEnumFORMATETCImpl.rs (the enumerator it hands out) and
    # shell32/cidldataobj_free.rs (the TU's four constructors). Three modules for one TU
    # because the assembler writes one file per class slice. No bridge: both constructors
    # are declared inside wine/shell32_main.h's own `extern "C"` block and both
    # SHCreate*DataObject are WINAPI in shlobj.h, so C++ callers already expect the
    # undecorated/stdcall names the Rust exports.
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
    # CEnumIDListBase.cpp — replaced by Rust (shell32-rust/src/shell32/CEnumIDListBase.rs);
    # cohab via libshell32_rust.a. The shared PIDL-enumerator base every shell folder's
    # IEnumIDList derives from. Cost $0: 7 of its 9 functions were already banked from an
    # earlier run and the other two lowered deterministically.
    # CExtractIcon.cpp — replaced by Rust (shell32-rust/src/shell32/CExtractIcon.rs for the
    # COM class, shell32/extracticon_free.rs for DuplicateString + the two SHCreate*ExtractIcon
    # entry points); cohab via libshell32_rust.a. Whole-TU flip. Activation is shellutils.h's
    # ShellObjectCreator<CExtractIcon> (CExtractIcon.cpp:339), i.e. internal, no CLSID.
    # Capture set: the only symbol the rest of the DLL takes from this TU is
    # SHCreateDefaultExtractIcon, which Rust publishes; SHCreateFileExtractIconW is a spec
    # export and Rust publishes that too.
    # NOTE this file is ISO-8859, not UTF-8, so a bare `grep` finds NOTHING in it — see
    # agent-reference.md §"Searching the tree".
    # folders.cpp — replaced by Rust (shell32-rust/src/folders.rs); cohab via libshell32_rust.a
    # iconcache.cpp — replaced by Rust (shell32-rust/src/shell32/iconcache.rs); cohab via libshell32_rust.a
    # propsheet.cpp — replaced by Rust (shell32-rust/src/propsheet.rs); reaches
    #   CStubWindow32 via CStubWindow32_*_RustBridge in utils.cpp
    shell32.cpp
    # utils.cpp — replaced by Rust (shell32-rust/src/shell32/utils.rs, all 95 free functions);
    # cohab via libshell32_rust.a. Whole-TU flip, gated together with the wiring because doing
    # either alone is a PARTIAL cohab flip: utils.rs publishes names the live C++ objects also
    # defined, the linker picks one without a duplicate-symbol error, and live entry points
    # reroute into Rust that never ran. Measured 2026-09-19 when it was wired alone: the guest
    # booted, passed smoke on all three arches, then froze between
    # `[EXPLORER] taskbar: OnCreate begin` and `child controls created`, launching 0 of 91
    # kmtest tests. Smoke cannot see that; `vibeos-cli conformance` can.
    # Capture set at flip time, measured against the LIVE CMake source list plus
    # libshell32_rust.a: 36 symbols leave this TU, and utils.rs publishes 32 of them. The other
    # four are the CStubWindow32 create/destroy bridges, rehomed to
    # rust_stubwindow32_stub.cpp below, and SHELL_GetCommandStringImpl /
    # SHELL_MapContextMenuVerbToCmdId, whose utils.h declarations became EXTERN_C so
    # shell32.cpp names the unmangled symbols Rust emits.
    # NOTE the earlier plan said all 16 `*_RustBridge` wrappers needed rehoming. They did not:
    # utils.rs publishes 14 of them itself. Only the pair that reaches a C++ ATL class does.
    rust_stubwindow32_stub.cpp
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
    # folders/CDrivesFolder.cpp — replaced by Rust; cohab via libshell32_rust.a. Whole-TU flip:
    # the CDrivesFolder class was already shell32-rust/src/drives_folder.rs, the free helpers
    # (drives context menu/eject/format cluster, CDrivesExtractIcon, icon helpers, the
    # *_RustBridge shims) are shell32-rust/src/folders/CDrivesFolder.rs, and the file-local
    # CDrivesFolderEnum is shell32-rust/src/folders/CDrivesFolderEnum.rs — an INTERNAL
    # ShellObjectCreator object with no OBJECT_ENTRY, so it needed `--cpp-com --internal`.
    # Its retained closure was 8: 7 `_RustBridge` entries plus the `g_IsFloppyCache` DATA
    # symbol, which the free-function slice now owns with `#[no_mangle]` (drives_folder.rs
    # still reaches it as an `extern static mut` under the unmangled C name). No C++ edits:
    # every consumer was already going through a bridge.
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
    # folders/CRecycleBin.cpp — replaced by Rust; cohab via libshell32_rust.a. Whole-TU flip: the
    # CRecycleBin class was already shell32-rust/src/recycle_bin.rs, its free helpers (the trash
    # engine glue, BBITEMDATA accessors, SHEmptyRecycleBin*) are
    # shell32-rust/src/folders/CRecycleBin.rs, and BOTH file-local COM classes came across too —
    # CRecycleBinEnum.rs and CRecycleBinItemContextMenu.rs, each an internal ShellObjectCreator
    # object with no OBJECT_ENTRY, so both needed `--cpp-com --internal`.
    # TWO of its five retained bridges still could not come across, both for CRecycleBinFolderViewCB,
    # which cannot be bought at all — its scaffold preflight refuses it twice. They are now
    # rust_recyclebin_bridges_stub.cpp (below), which also documents the layout bug that the
    # move exposed in that class. (The third, CRecyclerDropTarget_CreateInstance, has since
    # gone Rust with its own .cpp.)
    # SHQueryRecycleBin{A,W} are hand-written in the Rust slice: bindgen renders IRecycleBin,
    # IRecycleBinEnumList and IRecycleBinFile as opaque, so there was no slot to translate into.
    folders/CRecycleBinFolderViewCB.cpp
    rust_recyclebin_bridges_stub.cpp
    # folders/CRegFolder.cpp — replaced by Rust; cohab via libshell32_rust.a. Whole-TU flip
    # across THREE modules, because one C++ TU holds two COM classes plus file-scope helpers
    # and each rustconv run owns its own output file: folders/CRegFolder.rs (CRegFolder, 40
    # fns), folders/CRegFolderEnum.rs (CRegFolderEnum, 26), folders/regfolder_free.rs (the
    # file-scope functions).
    # Capture set: the only symbol the rest of the DLL takes from this TU is
    # CRegFolder_CreateInstance — still called from the live C++ folders/CDesktopFolder.cpp:287
    # — and the cpp-com scaffold publishes it `#[no_mangle]`. Measured against the C++ objects
    # AND libshell32_rust.a; the archive has to be in that sweep or it reports a false 0.
    # droptargets/CexeDropHandler.cpp — replaced by Rust
    # (shell32-rust/src/exe_drop_handler.rs).
    # droptargets/CFSDropTarget.cpp — replaced by Rust
    # (shell32-rust/src/droptargets/CFSDropTarget.rs); cohab via libshell32_rust.a. Whole-TU
    # flip: CFSDropTarget is an internal ShellObjectCreator class with no OBJECT_ENTRY, and the
    # only outside entry point was CFSDropTarget_CreateInstance, whose sole callers are the
    # already-flipped CFSFolder. Rust now publishes the
    # CFSDropTarget_CreateInstance_RustBridge that cfs_folder.rs links against, so the C++
    # bridge here would have been a duplicate symbol rather than a peer.
    # droptargets/CRecyclerDropTarget.cpp — replaced by Rust
    # (shell32-rust/src/droptargets/CRecyclerDropTarget.rs); cohab via libshell32_rust.a.
    # Whole-TU flip: an internal ShellObjectCreator class with no OBJECT_ENTRY. Its two callers
    # (the Rust CRecycleBin and the still-C++ CDefaultContextMenu) both go through
    # CRecyclerDropTarget_CreateInstance_RustBridge, which Rust publishes; the C++ forwarder
    # that used to live in rust_recyclebin_bridges_stub.cpp is gone with it.
    # shldataobject.cpp — replaced by Rust (shell32-rust/src/shell32/shldataobject.rs); cohab via libshell32_rust.a
    # shlexec.cpp — replaced by Rust (shell32-rust/src/shell32/shlexec.rs); cohab via libshell32_rust.a
    # shlfileop.cpp — replaced by Rust (shell32-rust/src/shell32/shlfileop.rs); cohab via libshell32_rust.a
    # shlfolder.cpp — replaced by Rust (shell32-rust/src/shell32/shlfolder.rs); cohab via libshell32_rust.a
    # CFileSysBindData.cpp — replaced by Rust (shell32-rust/src/shell32/CFileSysBindData.rs
    # for the COM class, shell32/filesysbinddata_free.rs for the three file-scope bind-context
    # helpers); cohab via libshell32_rust.a. Whole-TU flip. The class has NO CLSID — ATL's own
    # `_CreatorClass::CreateInstance` builds it — which is the activation shape rustconv's
    # cpp-com preflight refused until it was mapped.
    # Capture set measured as EMPTY: the only symbol anything else takes from this TU is
    # `IFileSystemBindData_Constructor`, and its caller is libshell32_rust.a (wine/pidl.rs),
    # not a C++ object — so the llvm-nm sweep has to include the staticlib's undefined set or
    # it reports a false 0. Rust publishes that name `#[no_mangle]`.
    # CDefView.cpp — replaced by Rust (shell32-rust/src/shell32/CDefView.rs); cohab via
    # libshell32_rust.a. Whole-TU flip. This one was held ORPHAN-BY-DESIGN for a while because
    # WIRING it while the .cpp was still compiled is a partial cohab flip: the Rust
    # `#[no_mangle]` names shadow the live C++ ones with no duplicate-symbol error, and live
    # entry points reroute into Rust that never ran (measured 2026-09-19 — booted, passed smoke
    # on all three arches, then froze after `[EXPLORER] startup: shell window ready`).
    # The flip is safe because the TU is a LEAF: `llvm-nm --undefined-only` over every other
    # object linked into shell32.dll (shell32.dir + dialogs + folders + shelldesktop +
    # shellmenu + recyclebin) references NOTHING defined in CDefView.cpp.obj — 0 symbols. Its
    # only outward surface is the spec export SHCreateShellFolderView (ordinal 256), which Rust
    # publishes; CDefView_CreateInstance was C++-mangled and had no caller outside this TU.
    # CDefViewUtil.cpp — replaced by Rust (shell32-rust/src/def_view_util.rs); cohab via
    # libshell32_rust.a. Whole-TU flip: CShellFolderViewCBWrapper is referenced by nothing
    # outside this TU, so the class moved wholesale. The two view-mode helpers are still
    # called from the C++ CDefView.cpp, whose forward declarations became EXTERN_C so they
    # name the unmangled cdecl symbols Rust publishes.
    #   CDefViewUtil.cpp
    # CDefViewDual.cpp — replaced by Rust (shell32-rust/src/shell32/CDefViewDual.rs); cohab via
    # libshell32_rust.a. Whole-TU flip: the only symbol the rest of the link still took from it
    # was CDefViewDual_Constructor (CDefView.cpp:3583 is the sole caller), and Rust publishes
    # that now. Its IDispatchImpl<> template arguments — plibid/piid/wMajor/wMinor — are baked
    # into the Rust translation, since a template reaches no TU and so has no body to translate.
    # CDefViewBckgrndMenu.cpp — replaced by Rust
    # (shell32-rust/src/shell32/CDefViewBckgrndMenu.rs); cohab via libshell32_rust.a. Whole-TU
    # flip: an internal ShellObjectCreator class with no OBJECT_ENTRY, whose only outside entry
    # point is CDefViewBckgrndMenu_CreateInstance. Its one caller is the still-C++
    # CDefView.cpp:3578, so shfldr.h's forward declaration became EXTERN_C to name the
    # unmangled cdecl symbol Rust publishes — same treatment CDefViewUtil's helpers got.
    # stubs.cpp — replaced by Rust (shell32-rust/src/shell32/stubs.rs); cohab via libshell32_rust.a
    # systray.cpp — replaced by Rust (shell32-rust/src/systray.rs); cohab via libshell32_rust.a
    # CUserNotification.cpp — replaced by Rust (shell32-rust/src/user_notification.rs); cohab via libshell32_rust.a
    # CDefaultContextMenu.cpp — replaced by Rust; cohab via libshell32_rust.a. Whole-TU flip,
    # assembled in two halves because one rustconv run cannot cover a TU that holds both a COM
    # class and free functions: shell32/CDefaultContextMenu.rs is the `--cpp-com --class` half
    # and shell32/CDefaultContextMenu_tu.rs the `--free-functions` half, with the header inlines
    # they share hand-written in shell32/cdefaultcontextmenu_support.rs.
    #   The exported entry points (SHCreateDefaultContextMenu, CDefFolderMenu_Create2) are
    # declared extern "C" WINAPI in shlobj.h, so Rust's `#[no_mangle] extern "system"` already
    # matches what CRegFolder.cpp links against. `_InsertMenuItemW` likewise, via
    # wine/shell32_main.h's extern "C" block. The two that did NOT match were MapFcidmCmdToVerb
    # and MapVerbToDfmCmd, declared with C++ linkage in precomp.h and still called from the
    # C++ CDefView.cpp and utils.cpp — both declarations are now EXTERN_C.
    #   The three DCMA_*_RustBridge wrappers moved with the file: they existed because the DCMA
    # entry points had C++ linkage and the Rust CFSDropTarget could not name them, and Rust now
    # publishes the bridges itself under the same names.
    #   CDefaultContextMenu.cpp
    # COpenWithMenu.cpp — replaced by Rust; cohab via libshell32_rust.a.
    #   Both remaining classes moved: COpenWithList (shell32/COpenWithList.rs) and
    # COpenWithDialog (shell32/COpenWithDialog.rs), with the free functions and the 13
    # opaque *_RustBridge wrappers in shell32/COpenWithMenu_tu.rs. The bridges existed so
    # the Rust open_with_menu.rs could reach a C++ class it could not name; they keep the
    # same names and now forward Rust-to-Rust, so open_with_menu.rs is untouched.
    #   The two exported entry points, SHOpenWithDialog and SH32_InvokeOpenWith, are
    # `#[no_mangle]` on the Rust side. Their other C++ callers (shlexec.cpp,
    # dialogs/filedefext.cpp) are already flipped, so the only surviving declarations of
    # either are the extern ones in Rust.
    # CNewMenu.cpp — replaced by Rust (shell32-rust/src/new_menu.rs); cohab via libshell32_rust.a
    # CSendToMenu.cpp — replaced by Rust (shell32-rust/src/send_to_menu.rs); cohab via libshell32_rust.a
    # CCopyMoveToMenu.cpp — shared base, four file-local helpers/callbacks, and
    # both CCopyToMenu/CMoveToMenu derived objects are Rust-owned.
    # CCopyAsPathMenu.cpp — replaced by Rust (shell32-rust/src/copy_as_path_menu.rs); cohab via libshell32_rust.a
    # CShellDispatch.cpp — replaced by Rust (shell32-rust/src/shell_dispatch.rs);
    # CShellDispatch_Constructor remains exported by the Rust module for CFolder/CDefViewDual.
    # CFolder.cpp — replaced by Rust (shell32-rust/src/shell32/CFolder.rs); cohab via
    # libshell32_rust.a. The Shell.Application `Folder2` object.
    # CFolderItems.cpp — replaced by Rust; its TWO classes are one file each,
    # shell32/CFolderItem.rs and shell32/CFolderItems.rs, because rustconv names its output
    # after the .cpp and a second class would overwrite the first.
    #   CFolderItemVerbs.cpp stays C++ below: `new CComObject<T>` activation and CComBSTR state
    # are both unmapped. The Rust CFolderItem::Verbs still constructs it, through the four
    # EXTERN_C symbols in rust_folderitemverbs_bridge.cpp — which is deleted the moment those
    # two classes translate.
    # CFolderItemVerbs.cpp — replaced by Rust (shell32-rust/src/shell32/CFolderItemVerb.rs and
    # CFolderItemVerbs.rs); cohab via libshell32_rust.a. Both classes, and with them the last
    # of CFolder.cpp/CFolderItems.cpp's C++ closure.
    #   rust_folderitemverbs_bridge.cpp is DELETED with this flip: it existed only because the
    # deterministic layer could not map `new CComObject<T>` activation or `CComBSTR` state, so
    # the Rust CFolderItem::Verbs had to build a C++ object. All three gaps are closed now
    # (the third being that this class is constructed by a DIFFERENT TU, which needs
    # `--activation-source dll/win32/shell32/CFolderItems.cpp`), and CFolderItem::Verbs calls
    # the generated CFolderItemVerbs_CreateInstance directly.
    )

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
rust_storage_target(SHELL32_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-shell32-target" shell32-rust)
set(SHELL32_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libshell32_rust.a")
if(ARCH STREQUAL "i386")
    set(SHELL32_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(SHELL32_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(SHELL32_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
# GLOB_RECURSE so .rs under src/<subdir>/ also retrigger the libshell32_rust.a rebuild.
rust_crate_sources(SHELL32_RUST_SRCS "${SHELL32_RUST_WORKSPACE_DIR}" shell32-rust)
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
target_link_libraries(shell32 shellmenu shelldesktop wine uuid cpprt atl_classes oldnames ${SHELL32_RUST_LIB} ${MENUBAND_RUST_LIB} ${RECYCLEBIN_RUST_LIB})
add_dependencies(shell32 rust_menuband rust_recyclebin)
add_delay_importlibs(shell32 powrprof shdocvw devmgr winspool.drv winmm mpr uxtheme ole32 oleaut32 userenv browseui version fmifs netapi32 secur32)
add_importlibs(shell32 advapi32 gdi32 user32 comctl32 comdlg32 shlwapi msvcrt kernel32 ntdll)
add_dependencies(shell32 stdole2 rust_shell32) # shell32_shldisp.tlb needs stdole2.tlb; rust_shell32 = cohab staticlib
# NOTE (upstream-sync 2026-09-20): upstream now links shell32 against a new
# `evalcmd` static lib when DLL_EXPORT_VERSION >= 0x600; its source landed at
# dll/win32/shell32/evalcmd/ in this merge but is NOT wired up here, because
# shell32 is mid-translation to Rust. Handed to the shell32 lane.
add_pch(shell32 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET shell32 DESTINATION reactos/system32 FOR all)
