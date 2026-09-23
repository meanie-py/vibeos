project(SHELL)

# THE WHOLE `shelldesktop` OBJECT LIBRARY IS RUST NOW — shell32-rust/src/shelldesktop/.
#
# The SECOND whole-target swap after shellrecyclebin, and the first for a SHARED object
# library: shelldesktop was compiled into shell32.dll AND rshell.dll, so both parents drop it
# (see the notes in each). The target itself goes away, `add_subdirectory(shelldesktop)` in the
# parent is commented out, so nothing below is processed; the list is kept, commented, as the
# record of what the library contained, and the .cpp stay on disk inert like every cohab flip.
#
# Capture set measured per TU with `llvm-nm --defined-only` on the object files, intersected
# with what the rest of the build leaves undefined:
#
#   CDesktopBrowser.cpp    -> SHCreateDesktop, SHDesktopMessageLoop, SHIsTempDisplayMode.
#       All three are shell32.dll EXPORTS (shell32.spec ordinals 200/201/724) and Rust
#       publishes all three. `CDesktopBrowser_CreateInstance` was C++-MANGLED in the object
#       (`__Z30CDesktopBrowser_CreateInstance...`), i.e. never externally callable, and a
#       tree-wide search finds no caller outside its own TU — so it needs no export.
#   CDirectoryWatcher.cpp  -> the five `CDirectoryWatcher_*_RustBridge` shims, now published
#       from shell32-rust/src/shelldesktop/CDirectoryWatcher_bridge.rs under identical names
#       and cdecl ABI. The shims MOVED, they did not vanish.
#   CDirectoryList.cpp     -> only C++-mangled `CDirectoryList::*` methods, whose sole
#       consumer was CDirectoryWatcher.cpp. It leaves with it.
#
# WHY THE PAIR HAD TO GO TOGETHER. CDirectoryWatcher holds a `CDirectoryList m_dir_list` BY
# VALUE, so flipping the list alone left four C++-mangled undefined symbols, and the watcher
# would not translate because that same member read as "not proven C scalar/pointer state".
# Breaking the cycle needed the state-field parser to look for a sibling class in the TU's own
# include closure (CDirectoryList.h is next door) and to recurse into what IT names, so
# `CSimpleArray<CDirectoryItem>` resolves too. Both modules now carry layout-identical
# `#[repr(C)]` copies of the list and its item, the same bridging CDirectoryList.rs already
# did for CDirectoryItem.
#
# RSHELL. rshell.dll links this library and does NOT link libshell32_rust.a, and its .spec
# exports SHCreateDesktop and SHDesktopMessageLoop — so after the flip it has no local
# definition of either. It resolves them through shell32's import lib, exactly as it already
# does for ShellDDEInit (ordinal 188) since dde.cpp flipped. That is why `add_importlibs`
# there already lists shell32 and why no new export was needed for this flip.
#
# add_definitions(
#     -DUNICODE
#     -D_UNICODE)
#
# list(APPEND SOURCE
#     CChangeNotifyServer.cpp  -> shelldesktop/CChangeNotifyServer{,_window,_free}.rs
#     CDesktopBrowser.cpp      -> shelldesktop/CDesktopBrowser{,_window,_free,_exports}.rs
#     CDirectoryWatcher.cpp    -> shelldesktop/CDirectoryWatcher{,_free,_bridge}.rs
#     CDirectoryList.cpp       -> shelldesktop/CDirectoryList.rs + CDirectoryItem.rs
#     dde.cpp                  -> shelldesktop/dde.rs
#     )
#
# add_library(shelldesktop OBJECT ${SOURCE})
# target_link_libraries(shelldesktop PRIVATE atl_classes)
# add_dependencies(shelldesktop xdk psdk)
