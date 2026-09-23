//! Wave-5 free-function translation units from the root of `dll/win32/shell32/`.
//!
//! These are whole plain-C++ TUs with no COM class of their own, translated through
//! `--free-functions`. They take their substrate from `crate::prelude` — the bindgen island —
//! rather than the hand-written `types.rs`/`win32.rs`; see prelude.rs for why that is
//! "instead of" and not "as well as".

/// SDK constants the island does NOT provide. Everything it did provide was pruned; see that
/// module's header for the value-by-value comparison that justified the pruning.
pub mod sdk_consts;

pub mod stubs;

/// `shldataobject.cpp`, flipped. Its four C++-mangled exports needed `EXTERN_C` in precomp.h
/// first — see the comment there.
pub mod shldataobject;

/// `shlfolder.cpp`, flipped. 19 of its exports had live C++ callers and needed `EXTERN_C` on
/// their declarations (shfldr.h, plus two in precomp.h). One of the 19 was an OVERLOAD PAIR of
/// `SHELL32_CoCreateInitSF`, which `EXTERN_C` cannot express — one name, one symbol — so the
/// csidl-taking form is now `SHELL32_CoCreateInitSFWithCsidl` on both sides. The C++ `const
/// GUID&` / `REFIID` reference parameters here are NOT an obstacle to `EXTERN_C`: it changes
/// linkage, not the type system, and a reference is a pointer at the ABI.
pub mod shlfolder;

/// `iconcache.cpp`, flipped. No `EXTERN_C` work was needed: its four shell32-internal exports
/// (`SIC_Initialize`, `SIC_Destroy`, `SIC_GetIconIndex`, `PidlToSicIndex`) are declared in
/// `wine/shell32_main.h`, which already wraps the whole header in `extern "C"` for C++.
pub mod iconcache;

// ---------------------------------------------------------------------------------------------
// ALL SIX ARE FLIPPED (shlexec, the last, on 2026-08-01). Nothing here is parked any more, so
// the counts and per-file blocker lists that used to live in this header are gone with the
// blockers; each module's own doc comment records what it needed and what it was hiding.
//
// What the batch cost, kept because it is the reusable part: the C++ SUBSTRATE these TUs lean
// on is `crate::atl` (CComPtr/CHeapPtr/CComHeapPtr/CSimpleArray/CStringW), `crate::shellutils`
// (the shellutils.h/utils.h header inlines, plus the CCoInit/COleInit RAII pair), `crate::layout`
// (ui/layout.h) and `crate::shellcom` (the `lpVtbl` shape for interfaces the island renders as
// opaque C++ classes). A header inline or a C++ template reaches no TU, so rustconv never sees a
// body for one and the name simply does not resolve — those are ports, not translations.
//
// Two traps worth keeping, both of which cost real time here:
//
// Do NOT add ULARGE_INTEGER or the ERROR_*/FILE_ATTRIBUTE_*/LANG_* constants to this crate:
// they already arrive through `ntdll_sys::nt::*`, which a scan of the `island_*.rs` files
// alone does not reveal. A second definition is E0659 at every use, not a shadow.
//
// If you bulk-apply casts from rustc spans, GUARD MATCH PATTERNS. A cast is not legal in a
// pattern; `crate-repair` declines those by design and a hand-rolled pass that forgets to
// silently writes `(SE_ERR_FNF) as usize => ...`, which does not compile. Cast the SCRUTINEE.
//
// WARNING for whoever runs `crate-repair` on this crate: **it edits shared MACRO DEFINITIONS
// to satisfy errors reported at their EXPANSION sites, once per call site.** Seen three times,
// on two different macros:
//   * `_countof!` — rewritten `u32` -> `i32` plus compensating casts in three ALREADY-LANDED
//     files. C's `sizeof(a)/sizeof(*a)` is unsigned.
//   * `ARRAY_SIZE!` — stacked into
//     `(((((((($a).len()) as u32) as usize) as u32) as usize) as u32) as usize) as u32`
//     while repairing shlexec (2026-07-31), and rustc asks for the same edit AGAIN at the three
//     `strlenW(x) + 1 > ARRAY_SIZE!(y)` sites in that file (it wants `.try_into().unwrap()` in
//     the macro body). Cast at the call site: `> ARRAY_SIZE!(y) as usize`.
// Revert the macro every time and cast at the call site instead. The test that it was pure
// blast radius: re-run `cargo check` after reverting — the error count does not move (it did
// not, on any of the three occasions).
//
// One measurement worth remembering: resolving a name does not only REMOVE errors, it also
// un-masks the ones that were hiding behind it. brfolder went 235 -> 244 when the ATL types
// started resolving, and shlexec's E0308 count went 56 -> 61 when its externs landed. A rising
// count on a file mid-grind is progress, not regression.
//
// These were PAID artifacts — do not delete a translated `.rs` to tidy up.
//
/// brfolder.cpp — the SHBrowseForFolder dialog. FLIPPED 2026-07-31: 86 -> 0. The residue was
/// this file's own casts plus four defects a green C build hides: amd64 widths baked into
/// `LRESULT`/`UINT_PTR`/`SIZE_T` casts, a C++ `!` turned into Rust's BITWISE not, one of two
/// `BrFolder_GetName` OVERLOADS dropped, and a `COleInit` RAII local left uninitialised so
/// neither OleInitialize nor OleUninitialize ran.
pub mod brfolder;
/// shlexec.cpp — ShellExecute and the whole verb / association / DDE launch path, and the last
/// of the six to flip (2026-08-01: 94 -> 0). Six defects a green C build hides, every one of
/// them somewhere C's type system declines to have an opinion:
///   * THREE dropped `sizeof`s. `sizeof(STARTUPINFOW)` became `size_of_val(&STARTUPINFOW)`, and
///     both `cbSize != sizeof(SHELLEXECUTEINFO{A,W})` entry gates lost the `sizeof` outright —
///     that comparison is what validates a caller's struct before any field of it is read.
///   * The `SHELL_ExecuteW32` typedef was `extern "system"` where neither the C typedef nor
///     `SHELL_ExecuteW` carries WINAPI. stdcall vs cdecl: on i386, caller and callee would have
///     disagreed about who pops the arguments.
///   * `SUCCEEDED(hr)` translated to `(hr as u32) >= 0`, which is unconditionally true, so a
///     FAILED `AssocQueryStringW` still selected the "runas" verb.
///   * `goto done` became a self-referential `'done: { break 'done; }` — a jump to itself, so
///     the runas-cancelled path fell through and overwrote its `retval = 33` with an error.
///   * `GetUIObjectOf(.., addr_of_mut!(cm))` took the address of the out-parameter instead of
///     passing it, so `shellex_get_contextmenu` never filled its caller's pointer.
///   * `RealShellExecuteA`/`ExA` (SHELL32.265/266) had HWND, PHANDLE and HINSTANCE flattened to
///     `isize` while their W twins were typed correctly.
/// `UrlIsFileUrlW` was one more phantom symbol: shlwapi.h:643 `#define`s it to
/// `UrlIsW(x, URLIS_FILEURL)`, so no export of that name exists to link against.
pub mod shlexec;
pub mod shlfileop;
/// changenotify.cpp — the CLIENT half of shell change notification (the server half is the
/// separate `shelldesktop/CChangeNotifyServer.cpp`, still C++). Its one ATL `CWindowImpl<>`
/// stays behind in `rust_changenotify_broker_stub.cpp`; the module header says why.
///
/// The first TU in this wave to need NO `EXTERN_C` lockstep at all — every one of its 15
/// declared names was already inside an `extern "C"` block, and its other 8 exports have no
/// header declaration to mangle.
pub mod changenotify;
/// CIDLDataObj.cpp — the shell's `IDataObject`/`IAsyncOperation` for a PIDL selection, plus
/// the `IEnumFORMATETC` it hands out and the TU's four constructors. Split across three
/// modules because one TU holds two COM classes and a set of free functions; the C++ file is
/// the unit that leaves CMake, not the module.
///
/// The TU that unblocked ATL CONTAINER state. `CSimpleArray<FORMATETC>`/`<STGMEDIUM>` cannot
/// be flattened to a pointer the way `CComPtr` is — a container's destructor walks storage the
/// scaffold does not own — so the field keeps its type and `atl.rs`'s `Drop` restores the C++
/// member destructor. The explicit dtor's own `RemoveAll()` is not a double free: it is
/// idempotent, and C++ runs the body before the member destructors either way.
pub mod CIDLDataObj;
pub mod IEnumFORMATETCImpl;
pub mod cidldataobj_free;

/// `utils.cpp`, translated through `--free-functions`: 95 shell helpers with no COM class of
/// their own — PIDL/path helpers, the Net* computer-description lookups, token-privilege
/// probes, the DFM verb map, and the `*_RustBridge` entry points C++ callers still reach.
///
/// It needs no ATL window substrate: the file's one `CWindowImpl` mention is a COMMENT
/// explaining why `CStubWindow32` is BRIDGED rather than reimplemented (its thunk and
/// registered class must have exactly one owner, and two C++ TUs still use it).
/// Hand-written substrate for utils.cpp: SDK constants outside the island's closure, plus the
/// shellutils.h `static inline`s (SHELL_ErrorBoxHelper) that have no symbol to import.
pub mod utils_support;
/// `CFileSysBindData.cpp`, cohab-flipped. Two modules for one TU because the class half and
/// the free-function half are separate rustconv runs and each overwrites its own file
/// wholesale — the same split as `CIDLDataObj` / `cidldataobj_free` above.
pub mod CFileSysBindData;
pub mod filesysbinddata_free;
/// `CExtractIcon.cpp`, cohab-flipped. Same two-module split as CFileSysBindData above.
pub mod CExtractIcon;
pub mod extracticon_free;
/// `CFolder.cpp` and `CFolderItems.cpp`, cohab-flipped — the Shell.Application automation
/// objects. THREE classes, one module each, because rustconv names its output after the .cpp
/// and translating a TU's second class would otherwise overwrite the first wholesale.
///
/// These were unblocked by a one-character fix, not by new machinery: the `IDispatchImpl<T,
/// &IID_T>` recognizer required the interface argument to be named I-something, and shldisp.idl
/// calls these `Folder2` / `FolderItem` / `FolderItems`. The `&IID_\1` backreference and the
/// `in interfaces` membership test were always the real guards, so widening the name pattern
/// lost nothing and admitted 62 functions.
///
/// `CFolderItemVerbs.cpp` is deliberately still C++ — `new CComObject<T>` activation and
/// `CComBSTR` state are both unmapped. The Rust `CFolderItem::Verbs` still has to construct
/// one, so `dll/win32/shell32/rust_folderitemverbs_bridge.cpp` publishes the four symbols
/// rustconv kept symbolic rather than inventing. That file is deleted when those two classes
/// translate; see rust-port-status.md, where the debt is recorded as OWED.
/// `CEnumIDListBase.cpp`, cohab-flipped — the shared PIDL enumerator base every shell folder's
/// IEnumIDList derives from. 7 of its 9 functions were already banked from an earlier run and
/// replayed at $0; the other two lowered deterministically, so the whole TU cost nothing.
pub mod CEnumIDListBase;
/// `CFolderItemVerbs.cpp`, cohab-flipped — both its classes. These were the two the
/// deterministic layer could not take last night, and closing them took three recognizer
/// additions, all free: `CComBSTR` state (kept as a TYPE so atl.rs's Drop reproduces
/// ~CComBSTR's SysFreeString, rather than flattened to a BSTR that leaks), `new CComObject<T>`
/// as a third internal-activation shape, and `--activation-source` for a class whose
/// construction lives in ANOTHER TU (CFolderItem::Verbs, over in CFolderItems.cpp).
/// `rust_folderitemverbs_bridge.cpp` existed only to stand in for these and is deleted.
/// Hand-written substrate for the pair below; its header explains the IContextMenu
/// naming collision it exists to resolve.
pub mod folderitemverbs_support;
pub mod CFolderItemVerb;
pub mod CFolderItemVerbs;
pub mod CFolder;
// CShellLibrary.cpp / CUserEventTimer.cpp — post-fork upstream COM objects, both still pure
// UNIMPLEMENTED stubs upstream, so every body lowered deterministically ($0). Activated from
// shell32.cpp's object map through `*RustEntry` shims like the classes above.
pub mod CShellLibrary;
pub mod CUserEventTimer;
// shell32.cpp, Phase 1: its plain logic. The COM server half (the object map, the class
// factory table, DllMain/DllGetClassObject/DllRegisterServer) is still C++ in that file.
pub mod shell32_free;
pub mod IDefClFImpl;
// shell32.cpp, Phase 2: the COM server — class-object table, cached class factories,
// DllGetClassObject, DllCanUnloadNow and the module lock count. Hand-written; its header
// states the ATL semantics it reproduces.
pub mod com_server;
// shell32.cpp, Phase 3: registration, DllMain and the last trampolines — the file is gone.
pub mod statreg;
pub mod registration;
pub mod dllmain;
pub mod CFolderItem;
pub mod CFolderItems;
/// `CFolder.cpp`'s free half: one `extern "C"` factory, hand-written because the
/// `--free-functions` PREAMBLE step fails on this TU. Its own header says why.
pub mod cfolder_free;
/// `utils.cpp`'s free functions, COHAB-FLIPPED 2026-09-19 -- wiring and the flip as ONE change,
/// which is what the ORPHAN-BY-DESIGN guard that used to sit here was protecting. Wired alone,
/// this module's `#[no_mangle]` names shadowed the live C++ ones with no duplicate-symbol error
/// and explorer froze inside taskbar `OnCreate`, launching 0 of 91 kmtest tests.
///
/// The guard also said the flip needed all 16 `*_RustBridge` wrappers rehomed. It did not:
/// utils.rs publishes 14 of them itself, so only the CStubWindow32 pair moved (to
/// dll/win32/shell32/rust_stubwindow32_stub.cpp), because only that pair reaches a C++ ATL
/// class that propsheet.cpp and CDrivesFolder.cpp still construct. See the CMakeLists entry for
/// the measured capture set.
pub mod utils;
