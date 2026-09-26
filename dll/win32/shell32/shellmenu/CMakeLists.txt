project(SHELL)

add_definitions(
    -DUNICODE
    -D_UNICODE)

if(DBG)
    add_definitions(-D_DEBUG=1) # CORE-17505
endif()

list(APPEND SOURCE
    # CMenuBand.cpp — replaced by Rust (sdk/lib/rust/menuband-rust). Unlike the shell32-
    # private COM classes, CMenuBand lives in this SHARED shellmenu object library and is
    # compiled into BOTH shell32.dll and rshell.dll, so its Rust replacement is a standalone
    # staticlib (libmenuband_rust.a, built below) linked by both DLLs — not a shell32-rust
    # module. The retained CMenuToolbars/CMenuFocusManager companions call its Rust exports.
    # CMenuDeskBar.cpp — replaced by menuband-rust/src/menu_desk_bar.rs
    # CMenuFocusManager.cpp — replaced by menuband-rust/src/menu_focus_manager.rs. A PLAIN
    # class (--plain): no COM map, no vtable. Its class-static TlsIndex and the two HOOKPROCs
    # live in menu_focus_manager_support.rs, because neither can be owned by a worker body
    # (an ABI declaration in a body is refused, and three functions must share the TLS slot).
    # CMenuSite.cpp — replaced by menuband-rust/src/menu_site.rs
    # CMenuToolbars.cpp — replaced by menuband-rust/src/menu_toolbars.rs, HAND-translated: its
    # bespoke (non-COM) virtuals and the constructors on both base and derived put it outside
    # what the deterministic scaffold can hold. It publishes the same 29 *_RustBridge entry
    # points menu_band.rs was already calling. It was the LAST source in this library.
    # CMergedFolder.cpp — replaced by menuband-rust/src/merged_folder.rs (CMergedFolder)
    # plus enum_merged_folder.rs (CEnumMergedFolder). The six
    # CEnumMergedFolder_*_RustBridge entry points the .cpp used to define for
    # merged_folder.rs are now published from enum_merged_folder_support.rs under the same
    # names and ABI; the creator among them reproduces `new CComObject`'s refcount of ZERO,
    # which the generated COM activation contract does not give you.
    # CStartMenu.cpp — replaced by menuband-rust/src/shell_menu_callback.rs (the TU's one
    # class, whose methods are defined INLINE in the .cpp — which is why a `^Class::method`
    # survey reports this TU as free-functions-only and misses it) plus start_menu.rs (the
    # five free functions). Two modules for one TU, the usual class/free split.
    )

# The WHOLE shellmenu object library is Rust now, so the target goes away rather than being
# emptied (CMake refuses an OBJECT library with no sources) — the same whole-library swap as
# shellrecyclebin and shelldesktop. Both of its consumers, shell32 and rshell, already link
# ${MENUBAND_RUST_LIB}, whose build is hoisted to the top-level CMakeLists.txt, so nothing
# about reaching the Rust changes. The .cpp files stay on disk, inert.
# add_library(shellmenu OBJECT ${SOURCE})
# target_link_libraries(shellmenu PRIVATE atl_classes)
# add_dependencies(shellmenu xdk psdk reactos_idls)

# The libmenuband_rust.a build (CMenuBand's Rust replacement) is hoisted to the top-level
# CMakeLists.txt (before add_subdirectory(base)) because BOTH shell32 and rshell link it and
# `base` configures before `dll` — so the MENUBAND_RUST_LIB var + rust_menuband target must
# exist before either DLL's link recipe runs. See MENUBAND_RUST_* there.
