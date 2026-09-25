//! Translations from `dll/win32/shell32/shelldesktop/` — the desktop/notification object
//! library that links into shell32.dll alongside shell32.dir's own objects.
//!
//! First module from that directory. The Rust lives in `shell32-rust` rather than a crate of
//! its own because `shelldesktop` is a static library linked INTO shell32.dll, so the symbols
//! resolve at the same DLL link that `libshell32_rust.a` already participates in.

/// `CDirectoryList.cpp`, cohab-flipped. A PLAIN C++ class (`--plain`): no BEGIN_COM_MAP, no
/// vtable, no activation — reached only by direct call from its own library. The preflight
/// said so in three findings at once ("no BEGIN_COM_MAP entries", aggregation, no
/// OBJECT_ENTRY), which is the signature of a non-COM helper rather than three separate gaps.
pub mod CDirectoryList;
/// The TU's SECOND plain class — `CDirectoryList.cpp` holds both, and CDirectoryList's bodies
/// call `CDirectoryItem_EqualPath_body`, so the pair only compiles together.
pub mod CDirectoryItem;

/// `dde.cpp`, the shell's DDE server — 13 free functions, no classes, and the first
/// shelldesktop TU to pass `cpp-tu` preflight outright.
pub mod dde;

/// `CChangeNotifyServer.cpp` — the shell change-notification server window. A COM class with
/// an internal activation shape (no OBJECT_ENTRY), unblocked once the state-field parser
/// learned to read a container of RAW POINTERS (`CSimpleArray<CWatchItem*> m_items`).
/// Shims for reaching the still-C++ `CDirectoryWatcher`; see the module docs.
pub mod CDirectoryWatcher_bridge;
pub mod CChangeNotifyServer;
/// Its ATL-window half — message map and `AtlWindow` impl. See the module docs.
pub mod CChangeNotifyServer_window;
/// Its one file-scope function, which is only a symbol — see the module docs.
pub mod CChangeNotifyServer_free;

/// `CDirectoryWatcher.cpp`, cohab-flipped together with `CDirectoryList.cpp` — the two are a
/// CYCLE and neither could move alone. `CDirectoryWatcher` holds a `CDirectoryList m_dir_list`
/// BY VALUE, so flipping the list alone left four C++-mangled undefined symbols, and the
/// watcher would not translate because that same member read as unproven state. What broke the
/// cycle was teaching the state-field parser to find a sibling class in the TU's own include
/// closure (`CDirectoryList.h` is next door) and to recurse into what IT names, so
/// `CSimpleArray<CDirectoryItem>` resolves too. Both modules now carry layout-identical
/// `#[repr(C)]` copies of the list and its item, exactly as CDirectoryList.rs already did for
/// CDirectoryItem.
pub mod CDirectoryWatcher;
/// The TU's file-scope half: the APC thread procedure and the four APC callbacks, plus the
/// file-scope state they share with the class half (`s_hThreadAPC`, `s_buffer`,
/// `s_fTerminateAllWatchers`). Separate module because one rustconv run owns one output file.
pub mod CDirectoryWatcher_free;

/// `CDesktopBrowser.cpp` — the desktop's Progman window and IShellBrowser implementation.
/// It was reported blocked on "internal COM-object construction is not mapped", which was a
/// DETECTION gap and not a missing feature: the class is built by
/// `ShellObjectCreatorInit<CDesktopBrowser, IShellDesktopTray*>`, and shellutils.h overloads
/// that template on the init-argument types as well as the class, so keying the scan on
/// `<T>` alone could not see a class named in a template list that did not end after it.
///
/// An ATL window like CChangeNotifyServer, but an OVERLAPPED one (CFrameWinTraits), which is
/// the case `atlwin.rs` flagged for checking when it arrived: our `WM_NCCREATE` pickup means a
/// message delivered BEFORE it has no `this`. Checked and safe — the message map handles no
/// such message. `"Progman"` is also registered in exactly one place in the tree (every other
/// mention is a `FindWindowW` lookup), so the one-owner rule for a registered window class
/// holds and a Rust implementation splits nothing.
pub mod CDesktopBrowser;
/// Its ATL-window half — `DECLARE_WND_CLASS_EX`, the twelve-entry message map and the
/// `AtlWindow` impl, none of which the COM scaffold models. See the module docs for the two
/// ownership checks (sole registrar of "Progman"; no pre-WM_NCCREATE handler) it discharges.
pub mod CDesktopBrowser_window;
/// Its file-scope half, as rustconv assembled it: the Progman class/window names and
/// `SHIsTempDisplayMode`.
pub mod CDesktopBrowser_free;
/// `SHCreateDesktop` / `SHDesktopMessageLoop`, hand-written and kept OUT of the generated
/// module so a reassembly cannot silently drop them. Both drive a `CComPtr` local through
/// `Attach`/`Detach`, which rustconv deliberately refuses to lower — see the module docs.
pub mod CDesktopBrowser_exports;
