PROJECT(SHELL)

if(DBG)
    add_definitions(-D_DEBUG=1) # CORE-17505
endif()

list(APPEND SOURCE
    # mediabtns.cpp — new upstream file (media/mixer buttons backend), no Rust
    # twin exists yet; kept live as C++.
    mediabtns.cpp
    # appbar.cpp — replaced by Rust (folded into explorer-rust traywnd.rs);
    # CAppBarManager is a base class of CTrayWindow, translated with it.
    # desktop.cpp — replaced by Rust (explorer-rust desktop.rs)
    # explorer.cpp — replaced by Rust (explorer-rust explorer.rs); wWinMain (the
    # process entry point) + the 4 process globals + CExplorerModule (ATL no-op).
    # notifyiconscust.cpp — replaced by Rust (explorer-rust notifyiconscust.rs)
    # rshell.cpp — replaced by Rust (explorer-rust rshell.rs)
    # settings.cpp — replaced by Rust (explorer-rust settings.rs); the mangled
    # TaskbarSettings::Save/Load method entry points forward from settings_shim.cpp.
    settings_shim.cpp
    # shellservice.cpp — replaced by Rust (explorer-rust shellservice.rs)
    # startctxmnu.cpp — replaced by Rust (explorer-rust startctxmnu.rs); the
    # single-interface IContextMenu Start-button context menu.
    # startmnu.cpp — replaced by Rust (explorer-rust startmnu.rs)
    # startmnucust.cpp — replaced by Rust (explorer-rust startmnucust.rs); the
    # Customize Classic Start Menu treeview dialog.
    # startmnusite.cpp — replaced by Rust (explorer-rust startmnusite.rs); the
    # start-menu site COM object (IServiceProvider/ITrayPriv/IOleCommandTarget/
    # IMenuPopup on explorer_sys::com).
    # startup.cpp — replaced by Rust (explorer-rust startup.rs)
    # syspager.cpp — replaced by Rust (explorer-rust syspager.rs); systray icon
    # pager (CSysPagerWnd + CIconWatcher/CBalloonQueue/CNotifyToolbar helpers).
    # taskband.cpp — replaced by Rust (explorer-rust taskband.rs); the
    # multi-interface COM object template (IObjectWithSite/IDeskBand/
    # IPersistStream/IWinEventHandler/IOleCommandTarget on explorer_sys::com).
    # taskswnd.cpp — replaced by Rust (explorer-rust taskswnd.rs); the
    # task-switch / window-list toolbar (CTaskSwitchWnd + CHardErrorThread).
    # tbsite.cpp — replaced by Rust (explorer-rust tbsite.rs); CTrayBandSite,
    # the tray band site (wraps the shell RebarBandSite, hosts the task band).
    # trayclock.cpp — replaced by Rust (explorer-rust trayclock.rs); first
    # tier-C CWindowImpl+CComObjectRootEx+IOleWindow class on explorer_sys::server.
    # traydeskbtn.cpp — replaced by Rust (folded into explorer-rust trayntfy.rs);
    # CTrayShowDesktopButton is embedded by value in CTrayNotifyWnd. The 7
    # TrayShowDesktopButton_* extern "C" shims are now Rust #[no_mangle] exports,
    # so traydeskbtn_shim.cpp is retired (deleted).
    # trayntfy.cpp — replaced by Rust (explorer-rust trayntfy.rs); CTrayNotifyWnd.
    # trayprop.cpp — replaced by Rust (explorer-rust trayprop.rs); the Taskbar &
    # Start-Menu Properties dialog (3 WTL property pages → flat DLGPROCs).
    # traywnd.cpp — replaced by Rust (explorer-rust traywnd.rs) — the taskbar
    # core CTrayWindow + CAppBarManager base + embedded CStartButton.
    # util.cpp — replaced by Rust (sdk/lib/rust/explorer-rust/src/explorer/util.rs);
    # only the C-variadic FormatMenuString shell stays C++ (util_vararg.cpp).
    util_vararg.cpp
    precomp.h)

# --- Rust cohabitation (explorer-rust) -------------------------------------
# Build libexplorer_rust.a from the workspace crate (standard user-mode
# per-arch targets, same pattern as advapi32-rust). The #[no_mangle] bodies
# replace the commented-out .cpp TUs above; lld first-match
# (--allow-multiple-definition) lets the staticlib's panic_handler/
# eh_personality coexist with the C++ side. Plan:
# docs/engineering/explorer-rust-translation-plan.md.
set(EXPLORER_RUST_WORKSPACE_DIR "${CMAKE_SOURCE_DIR}/sdk/lib/rust")
rust_storage_target(EXPLORER_RUST_TARGET_DIR "${CMAKE_CURRENT_BINARY_DIR}/rust-explorer-target" explorer-rust)
set(EXPLORER_RUST_LIB "${CMAKE_CURRENT_BINARY_DIR}/libexplorer_rust.a")
if(ARCH STREQUAL "i386")
    set(EXPLORER_RUST_TARGET "i686-pc-windows-gnu")
elseif(ARCH STREQUAL "amd64")
    set(EXPLORER_RUST_TARGET "x86_64-pc-windows-gnu")
elseif(ARCH STREQUAL "arm64")
    set(EXPLORER_RUST_TARGET "aarch64-pc-windows-gnullvm")
endif()
rust_crate_sources(EXPLORER_RUST_SRCS "${EXPLORER_RUST_WORKSPACE_DIR}" explorer-rust)
add_custom_command(
    OUTPUT ${EXPLORER_RUST_LIB}
    COMMAND ${CMAKE_COMMAND} -E env
        CARGO_TARGET_DIR=${EXPLORER_RUST_TARGET_DIR}
        cargo build --release --target ${EXPLORER_RUST_TARGET}
            -p explorer-rust
            --manifest-path ${EXPLORER_RUST_WORKSPACE_DIR}/Cargo.toml
    COMMAND ${CMAKE_COMMAND} -E copy
        ${EXPLORER_RUST_TARGET_DIR}/${EXPLORER_RUST_TARGET}/release/libexplorer_rust.a
        ${EXPLORER_RUST_LIB}
    DEPENDS ${EXPLORER_RUST_SRCS}
            ${EXPLORER_RUST_WORKSPACE_DIR}/explorer-rust/Cargo.toml
            ${EXPLORER_RUST_WORKSPACE_DIR}/explorer-sys/Cargo.toml
    WORKING_DIRECTORY ${EXPLORER_RUST_WORKSPACE_DIR}
    COMMENT "Building explorer-rust (${EXPLORER_RUST_TARGET})"
    VERBATIM)
add_custom_target(rust_explorer DEPENDS ${EXPLORER_RUST_LIB})
# ----------------------------------------------------------------------------

file(GLOB explorer_rc_deps res/*.*)
add_rc_deps(explorer.rc ${explorer_rc_deps})
add_executable(explorer ${SOURCE} explorer.rc)
# EXPLORER_RUST_LIB first so its #[no_mangle] bodies win over any lingering C++.
target_link_libraries(explorer ${EXPLORER_RUST_LIB} uuid wine cpprt atl_classes)
add_dependencies(explorer rust_explorer)
set_module_type(explorer win32gui UNICODE)
add_importlibs(explorer setupapi advapi32 gdi32 user32 comctl32 ole32 oleaut32 shell32 browseui shlwapi shdocvw version uxtheme msvcrt kernel32 ntdll)
add_delay_importlibs(explorer psapi)
add_pch(explorer precomp.h SOURCE)
add_cd_file(TARGET explorer DESTINATION reactos FOR all)
