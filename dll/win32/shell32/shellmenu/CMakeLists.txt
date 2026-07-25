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
    CMenuFocusManager.cpp
    # CMenuSite.cpp — replaced by menuband-rust/src/menu_site.rs
    CMenuToolbars.cpp
    CMergedFolder.cpp
    CStartMenu.cpp)

add_library(shellmenu OBJECT ${SOURCE})
target_link_libraries(shellmenu PRIVATE atl_classes)
add_dependencies(shellmenu xdk psdk reactos_idls)

# The libmenuband_rust.a build (CMenuBand's Rust replacement) is hoisted to the top-level
# CMakeLists.txt (before add_subdirectory(base)) because BOTH shell32 and rshell link it and
# `base` configures before `dll` — so the MENUBAND_RUST_LIB var + rust_menuband target must
# exist before either DLL's link recipe runs. See MENUBAND_RUST_* there.
