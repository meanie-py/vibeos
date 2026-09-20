//! Hand-written substrate for `CFolderItemVerbs.cpp`'s two classes.
//!
//! WHY THIS FILE EXISTS — a naming collision you cannot spell your way out of.
//!
//! `CFolderItemVerbs::Init` CALLS through an `IContextMenu` it holds. Two different Rust
//! types of that name are in scope in the generated module:
//!
//!   * `crate::island`'s, reached through the prelude glob, which bindgen renders OPAQUE for
//!     a C++ class — a `vtable_` blob with no callable slots, and
//!   * `crate::shellcom`'s, the crate's hand-typed `lpVtbl` shape, which is the one with
//!     `QueryContextMenu` on it.
//!
//! Writing `crate::shellcom::IContextMenu` in a worker body does NOT survive assembly:
//! `normalize_worker_body` strips the `crate::shellcom::` prefix wholesale (cpp_com.py), and
//! the bare name then resolves through the prelude glob to the ISLAND's opaque one. The
//! result is `no field lpVtbl on type types_arch::IContextMenu` — a deliberate
//! disambiguation silently redirected to the wrong type.
//!
//! So the callable shape is re-exported here under a name the stripper has no opinion about.
//! The generated bodies name `ShellIContextMenu`, and the assembler's import resolver wires
//! the `use` automatically because it scans the crate for unresolved names.
//!
//! The better fix is in the layer — the support closure should render an `lpVtbl` interface
//! when the island's same-named type is opaque, rather than treating "the island has this
//! name" as "the island provides a usable interface". Until then, this indirection is the
//! honest way to say which of the two types the call means.

// Real `pub type` aliases rather than `pub use` re-exports: the assembler's import resolver
// scans the crate for DEFINITIONS (`pub struct|union|enum|type`) to auto-wire a `use` into a
// generated module, and a re-export is not one.
pub type ShellIContextMenu = crate::shellcom::IContextMenu;
pub type ShellIContextMenuVtbl = crate::shellcom::IContextMenuVtbl;
