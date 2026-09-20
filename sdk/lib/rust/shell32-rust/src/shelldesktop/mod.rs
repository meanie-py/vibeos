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
