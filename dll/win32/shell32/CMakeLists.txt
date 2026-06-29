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
    brfolder.cpp
    CActiveDesktop.cpp
    CActiveDesktop.h
    CIDLDataObj.cpp
    CQueryAssociations.cpp
    changenotify.cpp
    debughlp.cpp
    dialogs/dialogs.cpp
    dialogs/drive.cpp
    dialogs/drvdefext.cpp
    dialogs/filedefext.cpp
    dialogs/filetypes.cpp
    dialogs/folder_options.cpp
    dialogs/item_prop.cpp
    dialogs/general.cpp
    dialogs/recycler_prop.cpp
    dialogs/view.cpp
    CDropTargetHelper.cpp
    CEnumIDListBase.cpp
    CExtractIcon.cpp
    CRecycleBinCleaner.cpp
    folders.cpp
    iconcache.cpp
    propsheet.cpp
    shell32.cpp
    utils.cpp
    CShellItem.cpp
    CShellLink.cpp
    CFolderOptions.cpp
    folders/CDesktopFolder.cpp
    folders/CFSFolder.cpp
    folders/CDrivesFolder.cpp
    folders/CMyDocsFolder.cpp
    folders/CPrinterFolder.cpp
    folders/CAdminToolsFolder.cpp
    folders/CNetFolder.cpp
    folders/CFontsFolder.cpp
    folders/CControlPanelFolder.cpp
    folders/CRecycleBin.cpp
    folders/CRecycleBinFolderViewCB.cpp
    folders/CRegFolder.cpp
    droptargets/CexeDropHandler.cpp
    droptargets/CFSDropTarget.cpp
    droptargets/CRecyclerDropTarget.cpp
    shldataobject.cpp
    shlexec.cpp
    shlfileop.cpp
    shlfolder.cpp
    CFileSysBindData.cpp
    CDefView.cpp
    CDefViewUtil.cpp
    CDefViewDual.cpp
    CDefViewBckgrndMenu.cpp
    stubs.cpp
    systray.cpp
    CUserNotification.cpp
    CDefaultContextMenu.cpp
    COpenWithMenu.cpp
    CNewMenu.cpp
    CSendToMenu.cpp
    CCopyMoveToMenu.cpp
    CCopyAsPathMenu.cpp
    CShellDispatch.cpp
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
        RUSTC_BOOTSTRAP=1  # shellord.rs ShellMessageBoxA needs #![feature(c_variadic)]
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
target_link_libraries(shell32 shellmenu shelldesktop wine uuid recyclebin cpprt atl_classes oldnames ${SHELL32_RUST_LIB})
add_delay_importlibs(shell32 powrprof shdocvw devmgr winspool.drv winmm mpr uxtheme ole32 oleaut32 userenv browseui version fmifs netapi32 secur32)
add_importlibs(shell32 advapi32 gdi32 user32 comctl32 comdlg32 shlwapi msvcrt kernel32 ntdll)
add_dependencies(shell32 stdole2 rust_shell32) # shell32_shldisp.tlb needs stdole2.tlb; rust_shell32 = cohab staticlib
add_pch(shell32 precomp.h "${PCH_SKIP_SOURCE}")
add_cd_file(TARGET shell32 DESTINATION reactos/system32 FOR all)
