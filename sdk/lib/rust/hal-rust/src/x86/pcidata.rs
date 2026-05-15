//! PCI class/vendor name databases.
//!
//! Replaces the build-generated `pci_classes.c` / `pci_vendors.c` (which
//! `native-bin2c` produced from the `.ids` text files via `hal/halx86/
//! pcidata.cmake`). The `.ids` files are the single source of truth; we
//! embed them directly with `include_bytes!` and a trailing NUL so
//! `bussupp.rs`'s `strstr` walk still works. No codegen tool involved.
//!
//! i386-only: matches the original C, which was only compiled on i386
//! (`legacy.cmake`/`pcidata.cmake` were only `include()`d for i386; the
//! amd64 reference in `bussupp.rs` is dead-stripped).
//!
//! `#[link_section = "INITDATA"]` mirrors the C `DATA_SEG("INITDATA")`
//! (`__attribute__((section("INITDATA")))`) — the tables live in the
//! discardable INIT segment and are freed after boot.
//!
//! The NUL is appended via a `#[repr(C)]` `{ data: [u8; N], nul: u8 }`
//! struct rather than a const copy loop: a 1.5 MB per-byte `while` in
//! const-eval blows rustc's step budget ("constant evaluation is taking
//! a long time"). `*include_bytes!()` is a single const memcpy, and the
//! exported symbol address is `&data` (offset 0, repr(C), no padding).

#[repr(C)]
pub struct NulTerminated<const N: usize> {
    data: [u8; N],
    _nul: u8,
}

#[no_mangle]
#[link_section = "INITDATA"]
pub static ClassTable: NulTerminated<
    {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../hal/halx86/legacy/bus/pci_classes.ids"
        ))
        .len()
    },
> = NulTerminated {
    data: *include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../hal/halx86/legacy/bus/pci_classes.ids"
    )),
    _nul: 0,
};

#[no_mangle]
#[link_section = "INITDATA"]
pub static VendorTable: NulTerminated<
    {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../hal/halx86/legacy/bus/pci_vendors.ids"
        ))
        .len()
    },
> = NulTerminated {
    data: *include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../hal/halx86/legacy/bus/pci_vendors.ids"
    )),
    _nul: 0,
};
