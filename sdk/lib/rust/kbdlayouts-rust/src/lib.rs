//! kbdlayouts — the keyboard layout DLLs of dll/keyboard, as Rust data.
//! staticlib → libkbdlayouts_rust.a, linked into each of the 91 layout DLLs.
//!
//! Every layout is ONE C data TU (the tables `KbdLayerDescriptor` returns), lowered for $0
//! by `rustconv data-port` (c_data_port.py): clang decides every byte. Each module exports
//! its accessor as `<layout>_KbdLayerDescriptor`; a layout's .spec names that symbol, and
//! the linker drops every other layout's sections from its DLL.
#![cfg_attr(not(test), no_std)]
#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]
#![allow(dead_code, unused_imports, static_mut_refs)]
#![allow(clippy::all)]

pub mod types; // bindgen island, per arch (gen_driver_types.sh kbdlayouts)
pub mod prelude; // what the generated layout modules glob

// One module per layout, written by `python -m rustconv.kbd_layouts`; do not hand-edit.
// BEGIN GENERATED: layouts (python -m rustconv.kbd_layouts)
pub mod kbd101;
pub mod kbd101a;
pub mod kbd103;
pub mod kbd106;
pub mod kbd106n;
pub mod kbda1;
pub mod kbda2;
pub mod kbda3;
pub mod kbdal;
pub mod kbdarme;
pub mod kbdarmw;
pub mod kbdaze;
pub mod kbdazel;
pub mod kbdbe;
pub mod kbdbga;
pub mod kbdbgt;
pub mod kbdblr;
pub mod kbdbr;
pub mod kbdbu;
pub mod kbdbur;
pub mod kbdcan;
pub mod kbdcmk;
pub mod kbdcr;
pub mod kbdcz;
pub mod kbdcz1;
pub mod kbdda;
pub mod kbddv;
pub mod kbdeo;
pub mod kbdest;
pub mod kbdfc;
pub mod kbdfi;
pub mod kbdfo;
pub mod kbdfr;
pub mod kbdgeo;
pub mod kbdgerg;
pub mod kbdgneo;
pub mod kbdgr;
pub mod kbdgr1;
pub mod kbdgre1;
pub mod kbdgrist;
pub mod kbdhe;
pub mod kbdheb;
pub mod kbdhu;
pub mod kbdic;
pub mod kbdinasa;
pub mod kbdinben;
pub mod kbdindev;
pub mod kbdinguj;
pub mod kbdinmal;
pub mod kbdir;
pub mod kbdit;
pub mod kbdjpn;
pub mod kbdkaz;
pub mod kbdkor;
pub mod kbdla;
pub mod kbdlt1;
pub mod kbdlv;
pub mod kbdmac;
pub mod kbdne;
pub mod kbdnec;
pub mod kbdnec95;
pub mod kbdnecat;
pub mod kbdnecnt;
pub mod kbdno;
pub mod kbdpl;
pub mod kbdpl1;
pub mod kbdpo;
pub mod kbdro;
pub mod kbdrost;
pub mod kbdru;
pub mod kbdru1;
pub mod kbdsf;
pub mod kbdsg;
pub mod kbdsl;
pub mod kbdsl1;
pub mod kbdsp;
pub mod kbdsw;
pub mod kbdtat;
pub mod kbdth0;
pub mod kbdth1;
pub mod kbdth2;
pub mod kbdth3;
pub mod kbdtuf;
pub mod kbdtuq;
pub mod kbduk;
pub mod kbdur;
pub mod kbdurs;
pub mod kbdus;
pub mod kbdusa;
pub mod kbdusl;
pub mod kbdusr;
pub mod kbdusx;
pub mod kbduzb;
pub mod kbdvntc;
pub mod kbdycc;
pub mod kbdycl;
// END GENERATED: layouts

// staticlib needs a panic handler + eh personality (no_std cohab); std supplies them under test.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    panic_common::panic_common(env!("CARGO_PKG_NAME"), info)
}

#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}
