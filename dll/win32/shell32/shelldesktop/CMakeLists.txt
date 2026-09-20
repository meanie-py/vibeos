project(SHELL)

add_definitions(
    -DUNICODE
    -D_UNICODE)


list(APPEND SOURCE
    CChangeNotifyServer.cpp
    CDesktopBrowser.cpp
    CDirectoryWatcher.cpp
    # CDirectoryList.cpp — TRANSLATED (shell32-rust/src/shelldesktop/CDirectoryList.rs) but
    # NOT YET FLIPPED, and it cannot be flipped alone. CDirectoryWatcher holds a
    # `CDirectoryList m_dir_list` BY VALUE (CDirectoryWatcher.h:35) and calls its methods
    # through CDirectoryList.h's inline constructor, so commenting this out leaves four
    # C++-MANGLED undefined symbols (CDirectoryList::AddPathsFromDirectory(wchar_t const*)
    # and friends). A bridge cannot help: a by-value member needs the real class layout,
    # not an extern shim.
    #   And the pair is circular — CDirectoryWatcher will not translate either, because
    # that same member is `state field type is not proven C scalar/pointer state:
    # CDirectoryList m_dir_list`. Breaking the cycle needs the deterministic layer to model
    # a translated SIBLING CLASS as by-value state; until then both stay C++ and the paid
    # CDirectoryList.rs sits wired-but-unused, which is harmless because the Rust publishes
    # unmangled `#[no_mangle]` names and the live C++ defines mangled ones — no collision,
    # so this is NOT a partial cohab flip.
    CDirectoryList.cpp
    dde.cpp)

add_library(shelldesktop OBJECT ${SOURCE})
target_link_libraries(shelldesktop PRIVATE atl_classes)
add_dependencies(shelldesktop xdk psdk)
