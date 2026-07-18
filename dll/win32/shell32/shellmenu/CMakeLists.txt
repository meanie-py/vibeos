project(SHELL)

add_definitions(
    -DUNICODE
    -D_UNICODE)

if(DBG)
    add_definitions(-D_DEBUG=1) # CORE-17505
endif()

list(APPEND SOURCE
    # CMenuBand.cpp — replaced by Rust (shell32-rust/src/menu_band.rs); cohab via
    # libshell32_rust.a. Companions stay C++: CMenuToolbars.cpp/CMenuFocusManager.cpp
    # carry the *_RustBridge shims Rust calls forward through, and call back through
    # the CMenuBand_*_RustExport surface (CMenuBandRustExports.h). Rust also exports
    # RSHELL_CMenuBand_CreateInstance (was CMenuBand.cpp:1324).
    CMenuDeskBar.cpp
    CMenuFocusManager.cpp
    CMenuSite.cpp
    CMenuToolbars.cpp
    CMergedFolder.cpp
    CStartMenu.cpp)

add_library(shellmenu OBJECT ${SOURCE})
target_link_libraries(shellmenu PRIVATE atl_classes)
add_dependencies(shellmenu xdk psdk)
