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
    # CDirectoryList.rs sits wired-but-unused. That is harmless, but NOT for the reason an
    # earlier comment here gave: it claimed the Rust publishes unmangled `#[no_mangle]`
    # names. It does not — CDirectoryList.rs and CDirectoryItem.rs contain ZERO
    # `#[no_mangle]`, only plain `pub unsafe fn`, so the symbols are RUST-mangled and
    # nothing is exported at all. The conclusion survives (there is no collision for the
    # linker to resolve silently, so this is not a partial cohab flip) and is in fact
    # stronger than claimed, but the stated mechanism was wrong.
    CDirectoryList.cpp
    # dde.cpp — replaced by shell32-rust/src/shelldesktop/dde.rs
    )

add_library(shelldesktop OBJECT ${SOURCE})
target_link_libraries(shelldesktop PRIVATE atl_classes)
add_dependencies(shelldesktop xdk psdk)
